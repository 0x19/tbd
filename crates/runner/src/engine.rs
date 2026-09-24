//! What runs the programs: the sandbox daemon on the machine, or the stub.
//! The service above never knows which; it asks an [`Engine`] and maps its
//! answer and its failures onto the contract.

use std::{fmt, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::{Config, EngineKind};

/// A language, as the engine names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    /// Go.
    Go,
    /// Rust.
    Rust,
}

impl Lang {
    /// The name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::Rust => "rust",
        }
    }
}

/// One run, as the engine takes it.
#[derive(Debug, Clone, Serialize)]
pub struct Request {
    /// The language.
    pub language: Lang,
    /// The program.
    pub source: String,
    /// Its input.
    pub stdin: String,
}

/// One step, as the sandbox daemon answers it.
#[derive(Debug, Clone, Deserialize)]
pub struct Step {
    /// Absent when stopped from outside.
    pub exit_code: Option<i32>,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// A stream was cut.
    pub truncated: bool,
    /// Wall time.
    pub wall_ms: u64,
    /// Why it was stopped from outside.
    pub killed: String,
}

/// One run's answer.
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    /// The run's id.
    pub id: String,
    /// How it ended.
    pub outcome: String,
    /// The compiler.
    pub compile: Step,
    /// The program, when it compiled.
    pub run: Option<Step>,
    /// Everything.
    pub total_ms: u64,
}

/// Why an engine could not answer.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The request breaks one of the engine's own bounds.
    #[error("{0}")]
    Invalid(String),
    /// The engine is at its limit of runs at once.
    #[error("{0}")]
    Busy(String),
    /// The engine could not be reached, refused the runner, or failed.
    #[error("sandbox unavailable: {0}")]
    Unavailable(String),
}

/// What runs the programs.
#[async_trait]
pub trait Engine: Send + Sync + fmt::Debug {
    /// Compile and run one program.
    async fn run(&self, req: Request) -> Result<Response, EngineError>;
    /// The engine runs nothing (the stub), and every answer says so.
    fn stub(&self) -> bool;
}

/// Build the engine a configuration names.
///
/// # Errors
/// The sandbox daemon is named but no token was given.
pub fn build(config: &Config, token: Option<String>) -> Result<Box<dyn Engine>, String> {
    match config.engine.kind {
        EngineKind::Stub => Ok(Box::new(Stub)),
        EngineKind::Sandboxd => {
            let token = token
                .filter(|t| !t.trim().is_empty())
                .ok_or("[engine] kind = \"sandboxd\" needs RUNNER_SANDBOX_TOKEN")?;
            let http = reqwest::Client::builder()
                .timeout(config.engine.timeout)
                .build()
                .map_err(|e| e.to_string())?;
            Ok(Box::new(Sandboxd {
                url: config.engine.url.trim_end_matches('/').to_owned(),
                token,
                http,
            }))
        }
    }
}

/// The sandbox daemon on the machine (`docs/sandbox/README.md`).
pub struct Sandboxd {
    url: String,
    token: String,
    http: reqwest::Client,
}

impl fmt::Debug for Sandboxd {
    // The token never reaches a log line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sandboxd")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct Problem {
    #[serde(default)]
    error: String,
}

#[async_trait]
impl Engine for Sandboxd {
    async fn run(&self, req: Request) -> Result<Response, EngineError> {
        let resp = self
            .http
            .post(format!("{}/run", self.url))
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                EngineError::Unavailable(if e.is_timeout() {
                    "no answer in time".into()
                } else {
                    e.to_string()
                })
            })?;
        let status = resp.status();
        if status.is_success() {
            return resp
                .json()
                .await
                .map_err(|e| EngineError::Unavailable(format!("unreadable answer: {e}")));
        }
        let why = resp
            .json::<Problem>()
            .await
            .map(|p| p.error)
            .unwrap_or_default();
        Err(match status.as_u16() {
            400 => EngineError::Invalid(why),
            429 => EngineError::Busy(why),
            // 401 means the two sides' tokens differ: an operator's problem, not
            // the caller's, and the caller is never told more than unavailable.
            _ => EngineError::Unavailable(format!("{status}")),
        })
    }

    fn stub(&self) -> bool {
        false
    }
}

/// In process, runs nothing. Deterministic, scripted by markers in the source:
/// `<<compile_error>>`, `<<timeout>>`, `<<busy>>`, `<<down>>`, `<<hang>>` (holds
/// its run for five seconds, for admission tests). Otherwise the program
/// "prints" what it would have run. Every answer is labelled a stub upstream.
#[derive(Debug, Clone, Copy)]
pub struct Stub;

fn step(exit: Option<i32>, stdout: &str, stderr: &str, killed: &str) -> Step {
    Step {
        exit_code: exit,
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
        truncated: false,
        wall_ms: 1,
        killed: killed.to_owned(),
    }
}

#[async_trait]
impl Engine for Stub {
    async fn run(&self, req: Request) -> Result<Response, EngineError> {
        let s = req.source.as_str();
        if s.contains("<<down>>") {
            return Err(EngineError::Unavailable("stub: down".into()));
        }
        if s.contains("<<busy>>") {
            return Err(EngineError::Busy("stub: busy".into()));
        }
        if s.contains("<<hang>>") {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        let id = "stub".to_owned();
        if s.contains("<<compile_error>>") {
            return Ok(Response {
                id,
                outcome: "compile_error".into(),
                compile: step(Some(1), "", "stub: does not compile", "none"),
                run: None,
                total_ms: 1,
            });
        }
        let (outcome, run) = if s.contains("<<timeout>>") {
            ("killed", step(None, "", "", "timeout"))
        } else {
            let out = format!(
                "stub: would run {} bytes of {}\n",
                s.len(),
                req.language.as_str()
            );
            ("ok", step(Some(0), &out, "", "none"))
        };
        Ok(Response {
            id,
            outcome: outcome.into(),
            compile: step(Some(0), "", "", "none"),
            run: Some(run),
            total_ms: 2,
        })
    }

    fn stub(&self) -> bool {
        true
    }
}
