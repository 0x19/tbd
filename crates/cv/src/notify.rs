//! The mail that tells the owner someone asked, and the one that tells the
//! person they were approved.
//!
//! Sent through the finance service's linked mailbox, over Envoy's internal
//! listener, as this service's own subject (`svc:cv`), which the owner
//! granted `read` on their party once (`cv grant`). Nothing here blocks a
//! request: the caller spawns these after the row is written, and the row
//! records when the owner's mail went out.

use std::sync::Arc;

use base64::Engine as _;
use tbd_common::{metrics::names, telemetry::propagation};
use tbd_proto::finance::v1::{
    ListConnectorsRequest, SendMailRequest, finance_service_client::FinanceServiceClient,
};
use tokio::sync::Mutex;
use tonic::{
    Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, Endpoint},
};

use crate::{config::Notify, store::RequestRow};

/// The subject this service calls finance as.
pub const SERVICE_SUBJECT: &str = "svc:cv";

/// Why a mail did not go out.
#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    /// Finance did not take the call, or refused it.
    #[error("finance: {0}")]
    Upstream(#[from] Status),
    /// No linked mailbox sends as the configured address.
    #[error("no linked mailbox sends as {0}")]
    NoMailbox(String),
    /// Finance recorded the mail as failed at the provider.
    #[error("the provider refused: {0}")]
    Failed(String),
}

/// Marks every call as this service and carries the trace along.
#[derive(Debug, Clone, Copy)]
struct ServiceCaller;

struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(value)) = (
            tonic::metadata::MetadataKey::from_bytes(key.as_bytes()),
            value.parse::<tonic::metadata::MetadataValue<_>>(),
        ) {
            self.0.insert(key, value);
        }
    }
}

impl Interceptor for ServiceCaller {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        propagation::inject(&mut MetadataInjector(request.metadata_mut()));
        let claims = serde_json::json!({ "sub": SERVICE_SUBJECT, "scp": ["tbd.finance"] });
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
        if let Ok(value) = payload.parse() {
            request
                .metadata_mut()
                .insert(tbd_common::principal::PAYLOAD_HEADER, value);
        }
        Ok(request)
    }
}

type Client = FinanceServiceClient<InterceptedService<Channel, ServiceCaller>>;

/// The sender. Cheap to clone behind an `Arc`.
#[derive(Debug)]
pub struct Notifier {
    client: Client,
    notify: Notify,
    /// The connector id once found; dropped when finance no longer knows it.
    mailbox: Mutex<Option<String>>,
}

impl Notifier {
    /// A sender to the finance service at `url`, or `None` when the URL or
    /// the addresses are not configured, in which case nobody is told.
    ///
    /// # Errors
    /// The URL does not parse.
    pub fn new(url: &str, notify: &Notify) -> Result<Option<Arc<Self>>, tonic::transport::Error> {
        if url.trim().is_empty() || !notify.enabled() {
            return Ok(None);
        }
        let channel = Endpoint::from_shared(url.to_owned())?.connect_lazy();
        Ok(Some(Arc::new(Self {
            client: FinanceServiceClient::with_interceptor(channel, ServiceCaller),
            notify: notify.clone(),
            mailbox: Mutex::new(None),
        })))
    }

    /// The linked mailbox that sends as `[notify] from`.
    async fn mailbox(&self) -> Result<String, NotifyError> {
        if let Some(id) = self.mailbox.lock().await.clone() {
            return Ok(id);
        }
        let listed = self
            .client
            .clone()
            .list_connectors(ListConnectorsRequest { party_ids: vec![] })
            .await?
            .into_inner()
            .connectors;
        let wanted = self.notify.from.trim().to_ascii_lowercase();
        let found = listed
            .into_iter()
            .find(|c| {
                c.status == "linked" && c.can_send && c.external_id.to_ascii_lowercase() == wanted
            })
            .map(|c| c.id)
            .ok_or_else(|| NotifyError::NoMailbox(self.notify.from.clone()))?;
        *self.mailbox.lock().await = Some(found.clone());
        Ok(found)
    }

    async fn send(&self, to: &str, subject: String, body: String) -> Result<(), NotifyError> {
        let connector_id = self.mailbox().await?;
        let sent = self
            .client
            .clone()
            .send_mail(SendMailRequest {
                connector_id: connector_id.clone(),
                to: vec![to.to_owned()],
                subject,
                body,
                ..SendMailRequest::default()
            })
            .await;
        let mail = match sent {
            Ok(r) => r.into_inner().mail.unwrap_or_default(),
            Err(status) if status.code() == tonic::Code::NotFound => {
                // The mailbox went away (relinked, removed): forget it, so
                // the next attempt looks again.
                *self.mailbox.lock().await = None;
                return Err(NotifyError::Upstream(status));
            }
            Err(status) => return Err(NotifyError::Upstream(status)),
        };
        if mail.status == "sent" {
            Ok(())
        } else {
            Err(NotifyError::Failed(mail.error))
        }
    }

    /// Tell the owner someone asked.
    ///
    /// # Errors
    /// The mail did not go out; the caller logs and moves on.
    pub async fn owner_requested(&self, req: &RequestRow) -> Result<(), NotifyError> {
        let who = if req.name.is_empty() {
            req.email.clone()
        } else {
            format!("{} <{}>", req.name, req.email)
        };
        let subject = format!("CV access request: {who}");
        let note = if req.note.trim().is_empty() {
            "(no note)".to_owned()
        } else {
            req.note.trim().to_owned()
        };
        let body = format!(
            "{who} asked for the full CV on {}.\n\nTheir note:\n{note}\n\nDecide here: {}\n\n\
             This message was sent by the cv service.",
            req.requested_at.format("%Y-%m-%d %H:%M UTC"),
            self.notify.admin_url()
        );
        let outcome = self.send(&self.notify.to, subject, body).await;
        count("owner", &outcome);
        outcome
    }

    /// Tell the person they may download now. Best effort: an environment
    /// that limits recipients refuses strangers, and that is fine.
    ///
    /// # Errors
    /// The mail did not go out.
    pub async fn requester_approved(&self, req: &RequestRow) -> Result<(), NotifyError> {
        let subject = "Your request for the full CV was approved".to_owned();
        let body = format!(
            "Hello {},\n\nyour request for the full CV was approved. Sign in and download it \
             here: {}\n\nThe document is prepared for you and carries your name; please do \
             not pass it on.\n\nThis message was sent by the cv service.",
            if req.name.is_empty() {
                "there"
            } else {
                &req.name
            },
            self.notify.site_url()
        );
        let outcome = self.send(&req.email, subject, body).await;
        count("requester", &outcome);
        outcome
    }
}

fn count(kind: &'static str, outcome: &Result<(), NotifyError>) {
    let result = match outcome {
        Ok(()) => "sent",
        Err(NotifyError::NoMailbox(_)) => "no_mailbox",
        Err(NotifyError::Failed(_)) => "failed",
        Err(NotifyError::Upstream(s)) if s.code() == tonic::Code::FailedPrecondition => "refused",
        Err(NotifyError::Upstream(_)) => "unreachable",
    };
    metrics::counter!(names::CV_NOTIFICATIONS_TOTAL, "kind" => kind, "outcome" => result)
        .increment(1);
}
