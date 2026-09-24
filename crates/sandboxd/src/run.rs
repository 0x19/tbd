//! One run: start a sandbox, write the source, compile, run, remove the
//! sandbox. Each step is a `docker exec` with its own deadline and output caps,
//! so its output and exit status stay apart from the others'. A deadline or an
//! output cap is enforced by removing the whole container: killing the `docker`
//! client would leave the program running inside it.

use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tbd_common::metrics::names;
use tokio::{
    io::{AsyncRead, AsyncReadExt as _, AsyncWriteExt as _},
    process::Command,
};

use crate::{
    config::Config,
    recipe::{self, Language},
};

/// What a caller sends.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    /// `go` or `rust`.
    pub language: Language,
    /// The whole program, one file.
    pub source: String,
    /// What the program reads on its standard input.
    #[serde(default)]
    pub stdin: String,
}

/// Why a step was stopped from outside, if it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Killed {
    /// It ended by itself.
    None,
    /// It passed its deadline.
    Timeout,
    /// It was killed by a signal it did not send itself; with the memory limit,
    /// that is the memory limit.
    Memory,
    /// Its output passed the cap.
    Output,
    /// It wrote a file past the file-size cap (the kernel's `SIGXFSZ`).
    FileSize,
    /// The whole sandbox was ended at one of its limits (memory counted for the
    /// sandbox as a whole, or processes), from outside it.
    Limit,
}

/// What the step's exit says about how it ended, when nothing here stopped it.
/// `docker exec` reports 128 with no status of the program's own when the
/// sandbox around it was ended; 137 and 153 are `SIGKILL` and `SIGXFSZ`.
fn classify(exit_code: Option<i32>) -> Killed {
    match exit_code {
        Some(128) => Killed::Limit,
        Some(137) => Killed::Memory,
        Some(153) => Killed::FileSize,
        _ => Killed::None,
    }
}

/// One step's result.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Step {
    /// The exit status; absent when the step was stopped from outside.
    pub exit_code: Option<i32>,
    /// Standard output, at most the cap, lossily decoded.
    pub stdout: String,
    /// Standard error, likewise.
    pub stderr: String,
    /// A stream passed the cap and was cut.
    pub truncated: bool,
    /// Wall time, milliseconds.
    pub wall_ms: u64,
    /// Why it was stopped from outside.
    pub killed: Killed,
}

/// How the run ended, in one word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Compiled, ran, exited 0.
    Ok,
    /// Compiled, ran, exited non-zero.
    Exit,
    /// Did not compile.
    CompileError,
    /// A step was stopped from outside (see its `killed`).
    Killed,
}

impl Outcome {
    /// The metric label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Exit => "exit",
            Self::CompileError => "compile_error",
            Self::Killed => "killed",
        }
    }
}

/// What the daemon answers.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunResponse {
    /// The run's id; the sandbox's name ends with it.
    pub id: String,
    /// The language it ran.
    pub language: Language,
    /// How it ended.
    pub outcome: Outcome,
    /// The compiler.
    pub compile: Step,
    /// The program; absent when it did not compile.
    pub run: Option<Step>,
    /// Everything, container start and removal included, milliseconds.
    pub total_ms: u64,
}

/// Why a run could not be made at all.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The request breaks a bound.
    #[error("{0}")]
    Invalid(String),
    /// The sandbox could not be started or reached.
    #[error("sandbox unavailable: {0}")]
    Unavailable(String),
}

/// Check a request against the bounds, before anything starts.
///
/// # Errors
/// The source is empty or too big, or the input is too big.
pub fn validate(config: &Config, req: &RunRequest) -> Result<(), RunError> {
    let l = &config.limits;
    if req.source.trim().is_empty() {
        return Err(RunError::Invalid("source: empty".into()));
    }
    if req.source.len() > l.max_source_bytes {
        return Err(RunError::Invalid(format!(
            "source: longer than {} bytes",
            l.max_source_bytes
        )));
    }
    if req.stdin.len() > l.max_stdin_bytes {
        return Err(RunError::Invalid(format!(
            "stdin: longer than {} bytes",
            l.max_stdin_bytes
        )));
    }
    Ok(())
}

/// Removes the sandbox when dropped, whatever happened: a caller who goes away
/// mid-run leaves nothing behind.
struct Sandbox {
    name: String,
    removed: bool,
}

impl Sandbox {
    async fn remove(&mut self) {
        if !self.removed {
            self.removed = true;
            remove(&self.name).await;
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if !self.removed
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            let name = std::mem::take(&mut self.name);
            handle.spawn(async move { remove(&name).await });
        }
    }
}

async fn remove(name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "--force", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
}

fn millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

/// Compile and run one program in one sandbox.
///
/// # Errors
/// The request breaks a bound, or the sandbox could not be started.
pub async fn run(config: &Config, req: RunRequest) -> Result<RunResponse, RunError> {
    validate(config, &req)?;
    let started = Instant::now();
    let id = uuid::Uuid::now_v7().to_string();
    let name = format!("tbd-sandbox-{id}");
    let lang = req.language;

    let args = recipe::container_args(lang, &name, &config.limits, &config.images);
    let out = Command::new("docker")
        .arg("run")
        .args(&args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| RunError::Unavailable(format!("docker: {e}")))?;
    let mut sandbox = Sandbox {
        name: name.clone(),
        removed: false,
    };
    if !out.status.success() {
        sandbox.removed = true;
        return Err(RunError::Unavailable(
            String::from_utf8_lossy(&out.stderr)
                .trim()
                .chars()
                .take(300)
                .collect(),
        ));
    }

    let limits = &config.limits;
    let write = step(
        &recipe::write_source_args(lang, &name),
        req.source.as_bytes(),
        Duration::from_secs(10),
        limits.max_output_bytes,
        &name,
    )
    .await;
    if write.exit_code != Some(0) {
        sandbox.remove().await;
        return Err(RunError::Unavailable(format!(
            "writing the source: {}",
            write.stderr.trim()
        )));
    }

    let compile = step(
        &recipe::build_args(lang, &name),
        b"",
        limits.compile_timeout,
        limits.max_output_bytes,
        &name,
    )
    .await;
    metrics::histogram!(names::SANDBOX_DURATION, "language" => lang.as_str(), "step" => "compile")
        .record(Duration::from_millis(compile.wall_ms).as_secs_f64());

    let (run, outcome) = if compile.killed != Killed::None {
        (None, Outcome::Killed)
    } else if compile.exit_code != Some(0) {
        (None, Outcome::CompileError)
    } else {
        let run = step(
            &recipe::run_args(&name),
            req.stdin.as_bytes(),
            limits.run_timeout,
            limits.max_output_bytes,
            &name,
        )
        .await;
        metrics::histogram!(names::SANDBOX_DURATION, "language" => lang.as_str(), "step" => "run")
            .record(Duration::from_millis(run.wall_ms).as_secs_f64());
        let outcome = match (run.killed, run.exit_code) {
            (Killed::None, Some(0)) => Outcome::Ok,
            (Killed::None, _) => Outcome::Exit,
            _ => Outcome::Killed,
        };
        (Some(run), outcome)
    };

    sandbox.remove().await;
    metrics::counter!(names::SANDBOX_RUNS_TOTAL, "language" => lang.as_str(), "outcome" => outcome.as_str())
        .increment(1);
    Ok(RunResponse {
        id,
        language: lang,
        outcome,
        compile,
        run,
        total_ms: millis(started.elapsed()),
    })
}

/// Read at most `cap` bytes; past it, say so and stop reading.
async fn capped<R: AsyncRead + Unpin>(mut r: R, cap: usize) -> (Vec<u8>, bool) {
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match r.read(&mut buf).await {
            Ok(0) | Err(_) => return (out, false),
            Ok(n) => {
                if out.len() + n > cap {
                    out.extend_from_slice(&buf[..cap - out.len()]);
                    return (out, true);
                }
                out.extend_from_slice(&buf[..n]);
            }
        }
    }
}

/// One `docker exec`: `input` on its standard input, both streams read up to
/// `cap`, `deadline` on the whole. Past the deadline or the cap, the sandbox is
/// removed, which is the only way to stop what runs inside it.
async fn step(
    args: &[String],
    input: &[u8],
    deadline: Duration,
    cap: usize,
    sandbox: &str,
) -> Step {
    let started = Instant::now();
    let child = Command::new("docker")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return Step {
                exit_code: None,
                stdout: String::new(),
                stderr: format!("docker: {e}"),
                truncated: false,
                wall_ms: 0,
                killed: Killed::None,
            };
        }
    };
    let (Some(mut stdin), Some(stdout), Some(stderr)) =
        (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        return Step {
            exit_code: None,
            stdout: String::new(),
            stderr: "docker: no pipes".into(),
            truncated: false,
            wall_ms: 0,
            killed: Killed::None,
        };
    };
    let input = input.to_vec();
    let feed = tokio::spawn(async move {
        let _ = stdin.write_all(&input).await;
        let _ = stdin.shutdown().await;
    });
    let out = tokio::spawn(capped(stdout, cap));
    let err = tokio::spawn(capped(stderr, cap));

    // Whichever comes first: both streams read to the end (or cut), or the deadline.
    let reading = async {
        let o = out.await.unwrap_or_default();
        let e = err.await.unwrap_or_default();
        (o, e)
    };
    let (stdout, stderr, cut, timed_out) = match tokio::time::timeout(deadline, reading).await {
        Ok(((o, out_cut), (e, err_cut))) => (o, e, out_cut || err_cut, false),
        Err(_) => (Vec::new(), Vec::new(), false, true),
    };
    let killed = if timed_out {
        Killed::Timeout
    } else if cut {
        Killed::Output
    } else {
        Killed::None
    };
    if killed != Killed::None {
        remove(sandbox).await;
        let _ = child.kill().await;
    }
    feed.abort();
    let status = if killed == Killed::None {
        tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .ok()
            .and_then(Result::ok)
    } else {
        None
    };
    let exit_code = status.and_then(|s| s.code());
    let killed = if killed == Killed::None {
        classify(exit_code)
    } else {
        killed
    };
    // When the sandbox itself was ended, what the client printed is the
    // runtime's own message, with its internal identifiers: say it plainly.
    let stderr = if killed == Killed::Limit {
        "the sandbox reached one of its limits (memory or processes) and was ended".to_owned()
    } else {
        String::from_utf8_lossy(&stderr).into_owned()
    };
    Step {
        exit_code: if killed == Killed::None {
            exit_code
        } else {
            None
        },
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr,
        truncated: cut,
        wall_ms: millis(started.elapsed()),
        killed,
    }
}
