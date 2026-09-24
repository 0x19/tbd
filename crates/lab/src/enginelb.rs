//! A small gRPC load balancer: what Envoy is in a deployed environment.
//!
//! Every deployed environment routes service-to-service traffic through Envoy,
//! which balances across replicas, health checks them and ejects the ones that
//! start failing. An in-process lab has no Envoy, so a stack built here has
//! historically pointed one service straight at one instance of another — which
//! is fine for a check but makes a stack with two replicas a lie, because
//! nothing ever uses the second one.
//!
//! This is the missing piece, small enough to read in one sitting:
//!
//! - **Per request, not per connection.** gRPC multiplexes every call onto one
//!   HTTP/2 connection, so a layer-4 proxy would pin a client to one replica
//!   for its lifetime and only ever deliver failover. Balancing has to happen
//!   above the connection, which is why this forwards whole HTTP requests.
//! - **Health checks** take an instance out of rotation when it stops serving,
//!   and put it back when it recovers.
//! - **Outlier ejection** takes one out when it *keeps serving but starts
//!   failing*. A backend injected with an error rate stays perfectly healthy by
//!   every health check it answers, so this is the only thing that catches it.
//!
//! What it deliberately is not: no retries (a caller's timeout is its own), no
//! load metric beyond round-robin, no TLS (a lab stack is loopback), and no
//! request buffering. Envoy is the real thing; this is the shape of it.

use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{Router, extract::State, response::Response};
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint as TonicEndpoint};
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};
use tower::{Service as _, ServiceExt as _};

use crate::service::TaskHandle;

/// How the balancer decides an endpoint is misbehaving.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    /// How often each endpoint's health is probed.
    pub probe_interval: Duration,
    /// How much recent history an ejection decision looks at.
    pub window: Duration,
    /// Requests needed in the window before the ratio means anything. Without
    /// it, the first failed request of a quiet minute is a 100% error rate.
    pub min_requests: u32,
    /// The failure ratio, 0 to 1, at which an endpoint leaves the rotation.
    pub error_ratio: f64,
    /// How long an ejected endpoint stays out the first time. Each consecutive
    /// ejection doubles it, up to [`Policy::max_cooldown`] — an endpoint that
    /// keeps failing its probation is asked back less and less often, so a
    /// fault that outlives one cooldown does not cost a fresh blip every time
    /// it lapses.
    pub cooldown: Duration,
    /// The ceiling on that doubling.
    pub max_cooldown: Duration,
    /// How many times slower than its fastest peer an endpoint may be before it
    /// is ejected for being slow rather than for failing. `None` turns the rule
    /// off; with one endpoint it never applies, because there is no peer to be
    /// slower than.
    pub slow_multiple: Option<f64>,
    /// The floor under which slowness is not worth acting on, however bad the
    /// ratio looks. Two endpoints at 1 ms and 4 ms are not an outage.
    pub slow_floor: Duration,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            probe_interval: Duration::from_secs(1),
            window: Duration::from_secs(5),
            min_requests: 10,
            error_ratio: 0.2,
            cooldown: Duration::from_secs(10),
            max_cooldown: Duration::from_secs(80),
            slow_multiple: Some(4.0),
            slow_floor: Duration::from_millis(100),
        }
    }
}

/// One endpoint, as a caller sees it.
#[derive(Debug, Clone)]
pub struct EndpointStatus {
    /// The instance name this endpoint was built from.
    pub name: String,
    /// Whether its last health probe said it was serving.
    pub healthy: bool,
    /// Whether outlier ejection has it out of the rotation.
    pub ejected: bool,
    /// Whether it is currently eligible to receive requests.
    pub in_rotation: bool,
    /// Requests it served in the ejection window.
    pub requests: u32,
    /// How many of those failed.
    pub failures: u32,
}

/// The mutable half of an endpoint.
#[derive(Debug)]
struct Health {
    healthy: bool,
    ejected_until: Option<Instant>,
    /// Consecutive ejections. Sets how long the next one lasts, and how little
    /// evidence a re-ejection needs; cleared once it serves a clean window.
    ejections: u32,
    /// One entry per recent request: when it happened, whether it failed, and
    /// how long it took.
    recent: VecDeque<Seen>,
}

/// One request, as the balancer saw it.
#[derive(Debug, Clone, Copy)]
struct Seen {
    at: Instant,
    failed: bool,
    took: Duration,
}

#[derive(Debug)]
struct Endpoint {
    name: String,
    channel: Channel,
    health: Mutex<Health>,
}

impl Endpoint {
    /// Drop history that has left the window, then count what is left.
    fn tally(health: &mut Health, now: Instant, window: Duration) -> (u32, u32) {
        while health
            .recent
            .front()
            .is_some_and(|seen| now.saturating_duration_since(seen.at) > window)
        {
            health.recent.pop_front();
        }
        let requests = u32::try_from(health.recent.len()).unwrap_or(u32::MAX);
        let failures = u32::try_from(health.recent.iter().filter(|seen| seen.failed).count())
            .unwrap_or(u32::MAX);
        (requests, failures)
    }

    /// Mean time of the requests still in the window, or `None` with none.
    fn mean(health: &Health) -> Option<Duration> {
        let n = u32::try_from(health.recent.len()).ok().filter(|n| *n > 0)?;
        let total: Duration = health.recent.iter().map(|seen| seen.took).sum();
        Some(total / n)
    }
}

/// The balancer. Cheap to clone behind an [`Arc`]; one per backend kind.
#[derive(Debug)]
pub struct Balancer {
    endpoints: Vec<Endpoint>,
    policy: Policy,
    next: AtomicUsize,
}

impl Balancer {
    /// Build a balancer over named `http://host:port` endpoints.
    ///
    /// # Errors
    /// An address does not parse as a URI.
    pub fn new(
        endpoints: impl IntoIterator<Item = (String, String)>,
        policy: Policy,
    ) -> anyhow::Result<Arc<Self>> {
        let endpoints = endpoints
            .into_iter()
            .map(|(name, url)| {
                let channel = TonicEndpoint::from_shared(url.clone())?
                    .connect_timeout(Duration::from_secs(2))
                    .connect_lazy();
                Ok(Endpoint {
                    name,
                    channel,
                    health: Mutex::new(Health {
                        // Optimistic: the first probe corrects it within a
                        // second, and refusing traffic before then would make
                        // a cold start look like an outage.
                        healthy: true,
                        ejected_until: None,
                        ejections: 0,
                        recent: VecDeque::new(),
                    }),
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        anyhow::ensure!(!endpoints.is_empty(), "a balancer needs an endpoint");
        Ok(Arc::new(Self {
            endpoints,
            policy,
            next: AtomicUsize::new(0),
        }))
    }

    /// Every endpoint, as it stands.
    pub async fn status(&self) -> Vec<EndpointStatus> {
        let now = Instant::now();
        let mut out = Vec::with_capacity(self.endpoints.len());
        for endpoint in &self.endpoints {
            let mut health = endpoint.health.lock().await;
            let (requests, failures) = Endpoint::tally(&mut health, now, self.policy.window);
            let ejected = health.ejected_until.is_some_and(|until| until > now);
            out.push(EndpointStatus {
                name: endpoint.name.clone(),
                healthy: health.healthy,
                ejected,
                in_rotation: health.healthy && !ejected,
                requests,
                failures,
            });
        }
        out
    }

    /// Pick the next endpoint in rotation.
    ///
    /// Falls back to the whole set when everything is out: a request to a
    /// failing backend beats a request to nothing at all, and the caller's own
    /// error is more useful than one this invented.
    async fn pick(&self) -> &Endpoint {
        let now = Instant::now();
        let start = self.next.fetch_add(1, Ordering::Relaxed);
        for offset in 0..self.endpoints.len() {
            let endpoint = &self.endpoints[(start + offset) % self.endpoints.len()];
            let health = endpoint.health.lock().await;
            let ejected = health.ejected_until.is_some_and(|until| until > now);
            if health.healthy && !ejected {
                drop(health);
                return endpoint;
            }
        }
        &self.endpoints[start % self.endpoints.len()]
    }

    /// Record how a request went and eject the endpoint if it has gone bad.
    async fn record(&self, endpoint: &Endpoint, failed: bool, took: Duration) {
        let now = Instant::now();
        let slow = self.slowest_peer_baseline(endpoint).await;

        let mut health = endpoint.health.lock().await;
        health.recent.push_back(Seen {
            at: now,
            failed,
            took,
        });
        let (requests, failures) = Endpoint::tally(&mut health, now, self.policy.window);

        if health.ejected_until.is_some_and(|until| until > now) {
            return;
        }

        // An endpoint on probation has to prove itself on far less evidence
        // than a fresh one: it already misbehaved, and making it earn a full
        // window again is what turns one fault into a blip per cooldown.
        let needed = if health.ejections > 0 {
            self.policy.min_requests.div_ceil(3).max(2)
        } else {
            self.policy.min_requests
        };
        if requests < needed {
            return;
        }

        let ratio = f64::from(failures) / f64::from(requests);
        let reason = if ratio >= self.policy.error_ratio {
            Some("failing too much to keep taking traffic")
        } else if slow.is_some_and(|limit| Endpoint::mean(&health).is_some_and(|m| m > limit)) {
            // Slow is not failing, but a replica dragging the percentile up is
            // still doing damage a healthy peer would not.
            Some("far slower than its peers")
        } else {
            None
        };

        match reason {
            Some(reason) => {
                health.ejections = health.ejections.saturating_add(1);
                let out = self
                    .policy
                    .cooldown
                    .saturating_mul(1u32 << health.ejections.saturating_sub(1).min(16))
                    .min(self.policy.max_cooldown);
                health.ejected_until = Some(now + out);
                // Start the probation clean, so one bad window does not eject
                // it again the instant it is readmitted.
                health.recent.clear();
                tracing::info!(
                    endpoint = %endpoint.name,
                    failures,
                    requests,
                    ejections = health.ejections,
                    out_for = ?out,
                    "ejected: {reason}"
                );
            }
            // A full clean window is what forgives the record: the next fault
            // to hit this endpoint gets the short cooldown again.
            None if health.ejections > 0
                && failures == 0
                && requests >= self.policy.min_requests =>
            {
                tracing::info!(endpoint = %endpoint.name, "back to normal after a clean window");
                health.ejections = 0;
            }
            None => {}
        }
    }

    /// How slow `endpoint` may be before it counts as an outlier: the fastest
    /// peer's mean times the policy's multiple, never below its floor.
    ///
    /// `None` when the rule is off, or when there is no peer to compare with —
    /// the last endpoint standing is as fast as things get, whatever it costs.
    async fn slowest_peer_baseline(&self, endpoint: &Endpoint) -> Option<Duration> {
        let multiple = self.policy.slow_multiple?;
        let now = Instant::now();
        let mut fastest: Option<Duration> = None;
        for peer in &self.endpoints {
            if std::ptr::eq(peer, endpoint) {
                continue;
            }
            let mut health = peer.health.lock().await;
            let ejected = health.ejected_until.is_some_and(|until| until > now);
            if ejected || !health.healthy {
                continue;
            }
            Endpoint::tally(&mut health, now, self.policy.window);
            if let Some(mean) = Endpoint::mean(&health) {
                fastest = Some(fastest.map_or(mean, |best: Duration| best.min(mean)));
            }
        }
        let fastest = fastest?;
        Some(fastest.mul_f64(multiple).max(self.policy.slow_floor))
    }

    /// Probe every endpoint's health until `cancel` fires.
    async fn probe(self: Arc<Self>, cancel: tokio_util::sync::CancellationToken) {
        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(self.policy.probe_interval) => {}
            }
            for endpoint in &self.endpoints {
                let serving = HealthClient::new(endpoint.channel.clone())
                    .check(HealthCheckRequest {
                        service: String::new(),
                    })
                    .await
                    .is_ok_and(|r| {
                        r.into_inner().status == (tonic_health::ServingStatus::Serving as i32)
                    });
                let mut health = endpoint.health.lock().await;
                if health.healthy != serving {
                    tracing::info!(endpoint = %endpoint.name, serving, "health changed");
                }
                health.healthy = serving;
            }
        }
    }
}

/// A running balancer: where it listens, and what it is doing.
#[derive(Debug)]
pub struct Running {
    /// The address to point a client at.
    pub addr: SocketAddr,
    /// The balancer itself, for [`Balancer::status`].
    pub balancer: Arc<Balancer>,
    task: TaskHandle,
    cancel: tokio_util::sync::CancellationToken,
}

impl Running {
    /// Stop serving and stop probing.
    pub async fn stop(self) {
        self.cancel.cancel();
        self.task.stop().await;
    }
}

/// Start a balancer on `listen` (port 0 for an ephemeral one).
///
/// # Errors
/// The address is taken, or an endpoint URL does not parse.
pub async fn start(
    listen: SocketAddr,
    endpoints: impl IntoIterator<Item = (String, String)>,
    policy: Policy,
) -> anyhow::Result<Running> {
    let balancer = Balancer::new(endpoints, policy)?;
    let listener = tokio::net::TcpListener::bind(listen).await?;
    let addr = listener.local_addr()?;

    let cancel = tokio_util::sync::CancellationToken::new();
    tokio::spawn(Arc::clone(&balancer).probe(cancel.clone()));

    let app = Router::new()
        .fallback(forward)
        .with_state(Arc::clone(&balancer));

    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let served = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = stopped.await;
        });
        if let Err(error) = served.await {
            tracing::error!(%error, "balancer exited with error");
        }
    });

    tracing::info!(%addr, endpoints = balancer.endpoints.len(), "balancer ready");
    Ok(Running {
        addr,
        balancer,
        task: TaskHandle::new(stop, task),
        cancel,
    })
}

/// Forward one request to a chosen endpoint and grade the answer.
async fn forward(
    State(balancer): State<Arc<Balancer>>,
    request: axum::extract::Request,
) -> Response {
    let endpoint = balancer.pick().await;
    let (parts, body) = request.into_parts();
    let request = http::Request::from_parts(parts, tonic::body::Body::new(body));

    // A tonic channel is buffer-backed: it must be polled ready before it is
    // called, or the buffer panics on an unreserved send.
    let mut channel = endpoint.channel.clone();
    let started = Instant::now();
    let answer = match channel.ready().await {
        Ok(channel) => channel.call(request).await,
        Err(error) => Err(error),
    };
    let took = started.elapsed();

    let (failed, response) = match answer {
        Ok(response) => {
            // A gRPC failure that happens before any message is sent arrives as
            // a `grpc-status` header, which is every fault this lab injects. One
            // that only ever appears in trailers is not graded: reading it would
            // mean buffering every streaming response, which is worse than
            // missing it.
            let failed = response
                .headers()
                .get("grpc-status")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i32>().ok())
                .is_some_and(|code| code != 0);
            let (parts, body) = response.into_parts();
            (
                failed,
                Response::from_parts(parts, axum::body::Body::new(body)),
            )
        }
        Err(error) => {
            tracing::debug!(endpoint = %endpoint.name, %error, "forwarding failed");
            (true, unavailable())
        }
    };

    balancer.record(endpoint, failed, took).await;
    response
}

/// A gRPC `UNAVAILABLE`, shaped the way a gRPC client expects to read it.
fn unavailable() -> Response {
    let mut response = Response::new(axum::body::Body::empty());
    *response.status_mut() = http::StatusCode::OK;
    response.headers_mut().insert(
        "content-type",
        http::HeaderValue::from_static("application/grpc"),
    );
    response
        .headers_mut()
        .insert("grpc-status", http::HeaderValue::from_static("14"));
    response.headers_mut().insert(
        "grpc-message",
        http::HeaderValue::from_static("no endpoint could serve this"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A request that took no meaningful time, for tests about failure rather
    /// than latency.
    const FAST: Duration = Duration::from_millis(1);

    fn policy() -> Policy {
        Policy {
            probe_interval: Duration::from_millis(50),
            window: Duration::from_secs(5),
            min_requests: 4,
            error_ratio: 0.5,
            cooldown: Duration::from_millis(200),
            max_cooldown: Duration::from_millis(800),
            slow_multiple: Some(4.0),
            slow_floor: Duration::from_millis(100),
        }
    }

    fn balancer() -> Arc<Balancer> {
        Balancer::new(
            [
                ("a".to_owned(), "http://127.0.0.1:1".to_owned()),
                ("b".to_owned(), "http://127.0.0.1:2".to_owned()),
            ],
            policy(),
        )
        .expect("two loopback endpoints parse")
    }

    #[tokio::test]
    async fn it_spreads_across_every_endpoint_in_rotation() {
        let lb = balancer();
        let mut names = Vec::new();
        for _ in 0..4 {
            names.push(lb.pick().await.name.clone());
        }
        assert!(names.contains(&"a".to_owned()), "{names:?}");
        assert!(names.contains(&"b".to_owned()), "{names:?}");
    }

    #[tokio::test]
    async fn an_unhealthy_endpoint_leaves_the_rotation_and_comes_back() {
        let lb = balancer();
        lb.endpoints[0].health.lock().await.healthy = false;
        for _ in 0..6 {
            assert_eq!(
                lb.pick().await.name,
                "b",
                "an unhealthy endpoint is skipped"
            );
        }
        lb.endpoints[0].health.lock().await.healthy = true;
        let mut names = Vec::new();
        for _ in 0..4 {
            names.push(lb.pick().await.name.clone());
        }
        assert!(names.contains(&"a".to_owned()), "it comes back: {names:?}");
    }

    #[tokio::test]
    async fn a_failing_endpoint_is_ejected_then_readmitted() {
        let lb = balancer();
        let bad = &lb.endpoints[0];

        // Under min_requests, a bad ratio means nothing yet.
        lb.record(bad, true, FAST).await;
        lb.record(bad, true, FAST).await;
        assert!(
            bad.health.lock().await.ejected_until.is_none(),
            "too few requests to judge an endpoint on"
        );

        lb.record(bad, true, FAST).await;
        lb.record(bad, true, FAST).await;
        assert!(
            bad.health.lock().await.ejected_until.is_some(),
            "4 failures of 4 is past the ratio"
        );
        for _ in 0..6 {
            assert_eq!(lb.pick().await.name, "b", "an ejected endpoint is skipped");
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
        let mut names = Vec::new();
        for _ in 0..4 {
            names.push(lb.pick().await.name.clone());
        }
        assert!(
            names.contains(&"a".to_owned()),
            "the cooldown lapses and it is tried again: {names:?}"
        );
    }

    #[tokio::test]
    async fn a_healthy_endpoint_is_never_ejected() {
        let lb = balancer();
        let good = &lb.endpoints[0];
        for _ in 0..20 {
            lb.record(good, false, FAST).await;
        }
        assert!(good.health.lock().await.ejected_until.is_none());
        let status = lb.status().await;
        assert_eq!(status[0].requests, 20);
        assert_eq!(status[0].failures, 0);
        assert!(status[0].in_rotation);
    }

    #[tokio::test]
    async fn with_everything_out_it_still_forwards_somewhere() {
        let lb = balancer();
        for endpoint in &lb.endpoints {
            endpoint.health.lock().await.healthy = false;
        }
        // A failing backend beats no backend: the caller gets the real error.
        let picked = lb.pick().await;
        assert!(["a", "b"].contains(&picked.name.as_str()));
    }

    #[tokio::test]
    async fn an_empty_balancer_is_refused() {
        let empty: Vec<(String, String)> = Vec::new();
        assert!(Balancer::new(empty, Policy::default()).is_err());
    }
}
