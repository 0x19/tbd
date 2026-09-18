//! A bank that behaves exactly as told.
//!
//! Everything the real one can do, including every way it can fail, and a
//! call counter -- because "the fourth scheduled call in a day is refused
//! without an HTTP request" is only a test if the test can count requests.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use serde_json::{Value, json};

use super::{
    Authorization, AuthorizationRequest, Provider, ProviderError, Psu, Session, SessionStatus,
};
use crate::import::{ProviderAccount, accounts_in};

/// What the next calls should do.
#[derive(Debug, Clone, Default)]
pub enum Mode {
    /// Answer from the fixtures.
    #[default]
    Ok,
    /// Every call is a 429, with this `Retry-After`.
    RateLimited(Option<Duration>),
    /// Every call says the consent is gone.
    ConsentInvalid,
    /// Every call fails before the bank sees it.
    Transport,
    /// Every call is a 401.
    Unauthorized,
}

/// Shared, so a test keeps a handle after moving the mock into the syncer.
#[derive(Debug, Default)]
pub struct Mock {
    inner: Arc<Mutex<Inner>>,
    calls: AtomicUsize,
}

#[derive(Debug, Default)]
struct Inner {
    mode: Mode,
    /// Pages per account uid.
    transactions: HashMap<String, Vec<Value>>,
    balances: HashMap<String, Value>,
    accounts: Vec<ProviderAccount>,
    /// Order `session()` reports, when it should differ from `accounts`.
    live_order: Option<Vec<String>>,
    session_status: String,
    valid_until: Option<chrono::DateTime<Utc>>,
    /// Calls per path, for assertions on what was and was not fetched.
    seen: Vec<String>,
    /// How many calls carried a PSU: attended ones.
    attended: usize,
}

impl Mock {
    /// An empty bank in a good mood.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Load the fixture responses the prototype saved, or any JSON shaped
    /// the same way: a session with `accounts`, and per-uid pages.
    pub fn with_session(self: &Arc<Self>, session: &Value) -> Arc<Self> {
        let mut g = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.accounts = accounts_in(session);
        g.session_status = "AUTHORIZED".into();
        g.valid_until = super::timestamp(session.pointer("/access/valid_until"));
        drop(g);
        Arc::clone(self)
    }

    /// The pages `transactions()` returns for one account.
    pub fn with_pages(self: &Arc<Self>, uid: &str, pages: Vec<Value>) -> Arc<Self> {
        self.lock().transactions.insert(uid.to_owned(), pages);
        Arc::clone(self)
    }

    /// The balances for one account.
    pub fn with_balances(self: &Arc<Self>, uid: &str, body: Value) -> Arc<Self> {
        self.lock().balances.insert(uid.to_owned(), body);
        Arc::clone(self)
    }

    /// Make `session()` list accounts in a different order than the
    /// session was created with. This is what Erste actually does.
    pub fn with_live_order(self: &Arc<Self>, uids: Vec<String>) -> Arc<Self> {
        self.lock().live_order = Some(uids);
        Arc::clone(self)
    }

    /// Change how the next calls behave.
    pub fn set_mode(&self, mode: Mode) {
        self.lock().mode = mode;
    }

    /// Mark the session expired on the provider's side.
    pub fn expire(&self) {
        let mut g = self.lock();
        g.session_status = "EXPIRED".into();
        g.mode = Mode::ConsentInvalid;
    }

    /// How many calls reached this bank, in total.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    /// The paths called, in order.
    pub fn seen(&self) -> Vec<String> {
        self.lock().seen.clone()
    }

    /// How many calls carried a PSU, i.e. were attended.
    pub fn attended(&self) -> usize {
        self.lock().attended
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn admit(&self, path: &str) -> Result<(), ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut g = self.lock();
        g.seen.push(path.to_owned());
        match g.mode.clone() {
            Mode::Ok => Ok(()),
            Mode::RateLimited(retry_after) => Err(ProviderError::RateLimited { retry_after }),
            Mode::ConsentInvalid => Err(ProviderError::ConsentInvalid("mock: consent gone".into())),
            Mode::Transport => Err(ProviderError::Transport("mock: connection reset".into())),
            Mode::Unauthorized => Err(ProviderError::Unauthorized("mock: 401".into())),
        }
    }
}

#[async_trait]
impl Provider for Mock {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn start_authorization(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<Authorization, ProviderError> {
        self.admit("/auth")?;
        Ok(Authorization {
            url: format!("https://bank.mock/login?state={}", request.state),
            authorization_id: "mock-auth".into(),
        })
    }

    async fn create_session(&self, code: &str) -> Result<Session, ProviderError> {
        self.admit("/sessions")?;
        if code.is_empty() {
            return Err(ProviderError::Http {
                status: 400,
                body: "empty code".into(),
            });
        }
        let g = self.lock();
        Ok(Session {
            session_id: format!("mock-session-{code}"),
            accounts: g.accounts.clone(),
            valid_until: g.valid_until,
            raw: json!({ "session_id": format!("mock-session-{code}") }),
        })
    }

    async fn session(&self, session_id: &str) -> Result<SessionStatus, ProviderError> {
        self.admit(&format!("/sessions/{session_id}"))?;
        let g = self.lock();
        Ok(SessionStatus {
            status: g.session_status.clone(),
            account_uids: g
                .live_order
                .clone()
                .unwrap_or_else(|| g.accounts.iter().map(|a| a.uid.clone()).collect()),
            valid_until: g.valid_until,
        })
    }

    async fn balances(&self, account_uid: &str, psu: Option<&Psu>) -> Result<Value, ProviderError> {
        self.admit(&format!("/accounts/{account_uid}/balances"))?;
        if psu.is_some() {
            self.lock().attended += 1;
        }
        Ok(self
            .lock()
            .balances
            .get(account_uid)
            .cloned()
            .unwrap_or_else(|| json!({ "balances": [] })))
    }

    async fn transactions(
        &self,
        account_uid: &str,
        _from: NaiveDate,
        _to: NaiveDate,
        psu: Option<&Psu>,
    ) -> Result<Vec<Value>, ProviderError> {
        self.admit(&format!("/accounts/{account_uid}/transactions"))?;
        if psu.is_some() {
            self.lock().attended += 1;
        }
        Ok(self
            .lock()
            .transactions
            .get(account_uid)
            .cloned()
            .unwrap_or_else(|| vec![json!({ "transactions": [] })]))
    }
}
