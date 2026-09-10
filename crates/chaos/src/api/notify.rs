//! Slack notifications for finished runs.
//!
//! One incoming webhook per environment (`[notify.slack]`, the URL from
//! `CHAOS_SLACK_WEBHOOK` only), a filter on outcome and kind, and a per-schedule
//! override. Posting is fire and forget: a failure is logged, never surfaced to
//! the run.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::runs::{RunKind, RunRecord, RunStatus};
use crate::{config::SlackConfig, tls::Trust};

/// Per-schedule override of the `[notify.slack] on` filter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMode {
    /// Post when the outcome is in `[notify.slack] on`.
    #[default]
    Failures,
    /// Post every outcome.
    Always,
    /// Never post.
    Off,
}

/// What a run needs to become a message: the environment and where the UI is.
#[derive(Clone)]
pub struct Notifier {
    slack: SlackConfig,
    env: String,
    ui_base: String,
    http: reqwest::Client,
}

impl Notifier {
    /// `ui_base` is the public admin UI root (`https://chaosadmin.<domain>`) or
    /// empty for no links.
    pub fn new(slack: SlackConfig, env: &str, ui_base: &str) -> Self {
        Self {
            slack,
            env: env.to_owned(),
            ui_base: ui_base.trim_end_matches('/').to_owned(),
            http: Trust::default().http(Some(Duration::from_secs(10))),
        }
    }

    /// A webhook is configured.
    pub fn enabled(&self) -> bool {
        !self.slack.webhook.trim().is_empty()
    }

    /// What the UI may show: everything but the webhook.
    pub fn describe(&self) -> Value {
        json!({
            "enabled": self.enabled(),
            "channel": self.slack.channel,
            "on": self.slack.on,
            "kinds": self.slack.kinds,
            "env": self.env,
        })
    }

    /// Whether this record posts under `mode`.
    pub fn wants(&self, record: &RunRecord, mode: NotifyMode) -> bool {
        if !self.enabled() || mode == NotifyMode::Off {
            return false;
        }
        if !self.slack.kinds.contains(&record.kind) {
            return false;
        }
        mode == NotifyMode::Always || self.slack.on.contains(&record.status)
    }

    /// Post a hello, for `POST /notify/test`. Returns the error text on failure.
    pub async fn test(&self) -> Result<(), String> {
        let text = format!(
            ":white_check_mark: chaos serve on *{}* can reach this channel{}",
            self.env,
            self.link_line()
        );
        let payload = json!({
            "text": format!("chaos serve on {} can reach this channel", self.env),
            "blocks": [{ "type": "section", "text": { "type": "mrkdwn", "text": text } }],
        });
        self.deliver(payload).await
    }

    /// Post a message built by [`Notifier::message`]. Errors are logged.
    pub async fn send_payload(&self, payload: Value, what: &str) {
        if let Err(error) = self.deliver(payload).await {
            tracing::warn!(error, run = what, "slack notification failed");
        }
    }

    async fn deliver(&self, mut payload: Value) -> Result<(), String> {
        if !self.enabled() {
            return Err(
                "no webhook configured ([notify.slack] webhook / CHAOS_SLACK_WEBHOOK)".into(),
            );
        }
        if !self.slack.channel.trim().is_empty() {
            payload["channel"] = Value::String(self.slack.channel.clone());
        }
        let res = self
            .http
            .post(self.slack.webhook.trim())
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            Ok(())
        } else {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            Err(format!("{status}: {}", body.trim()))
        }
    }

    fn link_line(&self) -> String {
        if self.ui_base.is_empty() {
            String::new()
        } else {
            format!(" · <{}/|open Chaos Admin>", self.ui_base)
        }
    }

    /// The Block Kit message for a record. Public for tests.
    pub fn message(&self, r: &RunRecord, schedule: Option<&str>) -> Value {
        let (icon, verb) = match r.status {
            RunStatus::Passed => (":white_check_mark:", "passed"),
            RunStatus::Completed => (":white_check_mark:", "completed"),
            RunStatus::Failed => (":x:", "failed"),
            RunStatus::Error => (":rotating_light:", "errored"),
            RunStatus::Cancelled => (":no_entry_sign:", "was cancelled"),
            RunStatus::Running => (":hourglass:", "is running"),
        };
        let kind = match r.kind {
            RunKind::Scenario => "Scenario",
            RunKind::Load => "Load run",
            RunKind::Validate => "Validate",
        };
        let summary = r.summary();
        let headline = format!("{icon} {kind} *{}* {verb} on *{}*", r.name, self.env);
        let mut fields: Vec<Value> = Vec::new();
        let mut field = |k: &str, v: String| {
            fields.push(json!({ "type": "mrkdwn", "text": format!("*{k}*\n{v}") }));
        };
        field("Took", format!("{:.1} s", r.duration_s));
        if let Some(n) = summary.requests_total {
            field("Requests", n.to_string());
        }
        if let Some(e) = summary.error_rate {
            field("Errors", format!("{:.2}%", e * 100.0));
        }
        if let Some(p) = summary.p99_ms {
            field("p99", format!("{p:.1} ms"));
        }
        if let Some((passed, total)) = summary.passed {
            field("Checks", format!("{passed}/{total}"));
        }
        if let Some(s) = schedule {
            field("Schedule", s.to_owned());
        }

        let mut lines: Vec<String> = Vec::new();
        if let Some(scenario) = &r.scenario {
            for a in scenario.assertions.iter().filter(|a| !a.passed) {
                lines.push(format!(
                    "• `{}` expected {} · actual {}",
                    a.name, a.expected, a.actual
                ));
            }
        }
        if let Some(report) = &r.validate {
            for c in report.checks.iter().filter(|c| !c.passed) {
                lines.push(format!("• `{}` ({}) {}", c.name, c.surface, c.detail));
            }
        }
        if let Some(e) = &r.error {
            lines.push(format!("• {e}"));
        }
        if lines.len() > 8 {
            let more = lines.len() - 8;
            lines.truncate(8);
            lines.push(format!("… and {more} more"));
        }

        let mut blocks = vec![json!({
            "type": "section",
            "text": { "type": "mrkdwn", "text": headline },
        })];
        if !fields.is_empty() {
            blocks.push(json!({ "type": "section", "fields": fields }));
        }
        if !lines.is_empty() {
            blocks.push(json!({
                "type": "section",
                "text": { "type": "mrkdwn", "text": lines.join("\n") },
            }));
        }
        let mut context = vec![format!(
            "run `{}` · {}",
            &r.id[..13.min(r.id.len())],
            r.started_at
        )];
        if !self.ui_base.is_empty() {
            context.push(format!(
                "<{}/runs/view/?id={}|open the run>",
                self.ui_base, r.id
            ));
        }
        blocks.push(json!({
            "type": "context",
            "elements": [{ "type": "mrkdwn", "text": context.join(" · ") }],
        }));

        json!({
            "text": format!("{kind} {} {verb} on {}", r.name, self.env),
            "blocks": blocks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{ScenarioResult, assertions::AssertionResult};

    fn slack(webhook: &str) -> SlackConfig {
        SlackConfig {
            webhook: webhook.into(),
            channel: "#chaos-test".into(),
            on: vec![RunStatus::Failed, RunStatus::Error],
            kinds: vec![RunKind::Scenario, RunKind::Validate],
        }
    }

    fn failed_scenario() -> RunRecord {
        let mut r = RunRecord::start(RunKind::Scenario, "latency");
        r.scenario = Some(ScenarioResult {
            name: "latency".into(),
            file: None,
            passed: false,
            skipped: false,
            duration_s: 3.0,
            load: None,
            services: std::collections::BTreeMap::default(),
            events: vec![],
            assertions: vec![
                AssertionResult {
                    name: "max_p99_ms".into(),
                    passed: false,
                    expected: "<= 250 ms".into(),
                    actual: "412.0 ms".into(),
                },
                AssertionResult {
                    name: "min_requests".into(),
                    passed: true,
                    expected: ">= 250".into(),
                    actual: "299".into(),
                },
            ],
            error: None,
        });
        r.finish(RunStatus::Failed, 3.0);
        r
    }

    #[test]
    fn filter_follows_config_and_mode() {
        let n = Notifier::new(slack("https://hooks.slack.invalid/x"), "local", "");
        let failed = failed_scenario();
        assert!(n.wants(&failed, NotifyMode::Failures));
        assert!(!n.wants(&failed, NotifyMode::Off));
        let mut passed = failed.clone();
        passed.status = RunStatus::Passed;
        assert!(!n.wants(&passed, NotifyMode::Failures));
        assert!(n.wants(&passed, NotifyMode::Always));
        let mut load = failed.clone();
        load.kind = RunKind::Load;
        assert!(!n.wants(&load, NotifyMode::Always), "kinds filter wins");
        let off = Notifier::new(slack(""), "local", "");
        assert!(!off.enabled());
        assert!(!off.wants(&failed, NotifyMode::Always));
    }

    #[test]
    fn message_names_the_failed_assertion_and_links_the_run() {
        let n = Notifier::new(
            slack("https://hooks.slack.invalid/x"),
            "production",
            "https://chaosadmin.example.test/",
        );
        let r = failed_scenario();
        let m = n.message(&r, Some("nightly"));
        let text = m.to_string();
        assert!(text.contains("*latency* failed on *production*"), "{text}");
        assert!(text.contains("`max_p99_ms` expected <= 250 ms · actual 412.0 ms"));
        assert!(!text.contains("min_requests"));
        assert!(text.contains(&format!(
            "https://chaosadmin.example.test/runs/view/?id={}",
            r.id
        )));
        assert!(text.contains("*Schedule*\\nnightly"));
        assert_eq!(m["text"], "Scenario latency failed on production");
    }
}
