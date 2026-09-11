//! `chaos` binary. Stdout is the product here, so the print lints are allowed
//! in this file only.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use tbd_chaos::{
    config::{self, ChaosConfig, Source},
    kinds,
};
use tbd_common::telemetry::TelemetryArgs;

#[derive(Parser)]
#[command(name = "chaos", version = tbd_common::VERSION, about = "Validate, load-test and fault-test the stack")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    telemetry: TelemetryArgs,
    /// Environment: picks `<config-dir>/<env>.toml` to merge over `base.toml`.
    #[arg(long, env = config::ENV_VAR, default_value = config::DEFAULT_ENV, global = true)]
    env: String,
    /// Directory holding `base.toml` and one file per environment.
    #[arg(long, env = "CHAOS_CONFIG_DIR", default_value = config::DEFAULT_DIR, global = true)]
    config_dir: PathBuf,
    /// Public base domain the UI links to (`grafana.<domain>`, `logs.<domain>`, ...).
    /// Default: `[links] domain`.
    #[arg(long, env = "CHAOS_PUBLIC_DOMAIN", global = true)]
    public_domain: Option<String>,
}

/// Flags that override `[targets]` in the config. Precedence, lowest first:
/// the config, `CHAOS_<KIND>_URL` per kind, `--target` (or `CHAOS_TARGETS`),
/// the per-kind aliases.
#[derive(Args, Debug, Clone)]
struct TargetArgs {
    /// Target URL for a kind, as `KIND=URL`; repeatable. `CHAOS_<KIND>_URL`
    /// sets one kind from the environment. Kinds: see `chaos kinds`.
    #[arg(
        long = "target",
        value_name = "KIND=URL",
        value_parser = parse_target,
        env = "CHAOS_TARGETS",
        value_delimiter = ','
    )]
    targets: Vec<(String, String)>,
    /// Alias for `--target protocol=URL`.
    #[arg(long, value_name = "URL")]
    protocol: Option<String>,
    /// Alias for `--target engine=URL`.
    #[arg(long, value_name = "URL")]
    engine: Option<String>,
    /// Alias for `--target ledger=URL`.
    #[arg(long, value_name = "URL")]
    ledger: Option<String>,
}

/// `KIND=URL` where `KIND` is a registered kind with a validate target.
fn parse_target(s: &str) -> Result<(String, String), String> {
    let (kind, url) = s
        .split_once('=')
        .ok_or_else(|| format!("{s:?}: expected KIND=URL"))?;
    let known = kinds::with_target()
        .map(|(k, _)| k.name)
        .collect::<Vec<_>>()
        .join(", ");
    if kinds::by_name(kind).is_none_or(|k| k.target.is_none()) {
        return Err(format!(
            "{kind:?}: not a kind with a validate target; one of {known}"
        ));
    }
    url::Url::parse(url).map_err(|e| format!("{kind}: {e}"))?;
    Ok((kind.to_owned(), url.to_owned()))
}

/// Bearer token for a deployed stack (Envoy requires one on the API).
#[derive(Args, Debug)]
struct AuthArgs {
    /// A fixed bearer token. Default: `[auth] token`.
    #[arg(long, env = "CHAOS_TOKEN", hide_env_values = true)]
    token: Option<String>,
    /// `OAuth2` token endpoint for the client-credentials grant. Default: `[auth] token_url`.
    #[arg(long, env = "CHAOS_AUTH_TOKEN_URL")]
    auth_token_url: Option<String>,
    /// Client id. Default: `[auth] client_id`.
    #[arg(long, env = "CHAOS_AUTH_CLIENT_ID")]
    auth_client_id: Option<String>,
    /// Client secret. Default: `[auth] client_secret`.
    #[arg(long, env = "CHAOS_AUTH_CLIENT_SECRET", hide_env_values = true)]
    auth_client_secret: Option<String>,
}

impl AuthArgs {
    fn apply(self, config: &mut ChaosConfig) {
        if let Some(v) = self.token {
            config.auth.token = v;
        }
        if let Some(v) = self.auth_token_url {
            config.auth.token_url = v;
        }
        if let Some(v) = self.auth_client_id {
            config.auth.client_id = v;
        }
        if let Some(v) = self.auth_client_secret {
            config.auth.client_secret = v;
        }
    }
}

/// Flags of `chaos serve`; each overrides one config field.
#[derive(Args, Debug)]
struct ServeArgs {
    /// Listen address. Default: `[serve] listen`.
    #[arg(long, env = "CHAOS_LISTEN_ADDR")]
    listen: Option<SocketAddr>,
    /// API prefix. Default: `[serve] base_path`.
    #[arg(long, env = "CHAOS_BASE_PATH")]
    base_path: Option<String>,
    /// Built UI directory. Default: `[serve] ui_dir`.
    #[arg(long, env = "CHAOS_UI_DIR")]
    ui_dir: Option<String>,
    /// Topology to run in-process. Default: `[paths] topology`.
    #[arg(long, env = "CHAOS_TOPOLOGY")]
    topology: Option<PathBuf>,
    /// Scenario directory. Default: `[paths] scenarios`.
    #[arg(long, env = "CHAOS_SCENARIOS_DIR")]
    scenarios: Option<PathBuf>,
    /// Its `*.toml` files are copied into the scenario directory on start when
    /// missing there; existing files are left alone.
    /// Default: `[paths] scenarios_seed`.
    #[arg(long, env = "CHAOS_SCENARIOS_SEED")]
    scenarios_seed: Option<PathBuf>,
    /// Run records directory. Default: `[paths] results`.
    #[arg(long, env = "CHAOS_RESULTS_DIR")]
    results: Option<PathBuf>,
    /// Schedules file. Default: `[paths] schedules`.
    #[arg(long, env = "CHAOS_SCHEDULES_FILE")]
    schedules: Option<PathBuf>,
    /// File of instances added to the stack at runtime. Default: `[paths] stack`.
    #[arg(long, env = "CHAOS_STACK_FILE")]
    stack_file: Option<PathBuf>,
    /// Slack incoming webhook for run notifications. Default: `[notify.slack] webhook`, none.
    #[arg(long, env = "CHAOS_SLACK_WEBHOOK", hide_env_values = true)]
    slack_webhook: Option<String>,
    /// Do not start the topology stack; API only.
    #[arg(long)]
    no_stack: bool,
    #[command(flatten)]
    targets: TargetArgs,
    #[command(flatten)]
    auth: AuthArgs,
}

impl ServeArgs {
    fn apply(self, config: &mut ChaosConfig) {
        self.targets.apply(config);
        self.auth.apply(config);
        if let Some(v) = self.listen {
            config.serve.listen = v;
        }
        if let Some(v) = self.base_path {
            config.serve.base_path = v;
        }
        if let Some(v) = self.ui_dir {
            config.serve.ui_dir = v;
        }
        if let Some(v) = self.topology {
            config.paths.topology = v;
        }
        if let Some(v) = self.scenarios {
            config.paths.scenarios = v;
        }
        if let Some(v) = self.scenarios_seed {
            config.paths.scenarios_seed = v;
        }
        if let Some(v) = self.results {
            config.paths.results = v;
        }
        if let Some(v) = self.schedules {
            config.paths.schedules = v;
        }
        if let Some(v) = self.stack_file {
            config.paths.stack = v;
        }
        if let Some(v) = self.slack_webhook {
            config.notify.slack.webhook = v;
        }
        if self.no_stack {
            config.serve.start_stack = false;
        }
    }
}

#[derive(Subcommand)]
// `Serve` carries every serve flag; the others are a few paths. Built once, never moved.
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Start the stack described in a topology file and keep it running until Ctrl-C.
    Up {
        /// Topology or scenario file; only its `[stack]` section is used.
        /// Default: `[paths] topology` from the config.
        file: Option<PathBuf>,
    },
    /// Run scenario files: start the stack, apply load, play the timeline, assert.
    Run {
        /// Scenario files or glob patterns.
        #[arg(required_unless_present = "dir")]
        files: Vec<String>,
        /// Run every `*.toml` under a directory, recursively.
        #[arg(long, short)]
        dir: Option<PathBuf>,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Parse and check scenario files without running them.
    Check {
        /// Scenario files.
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Hit every surface of a running stack and report per check.
    Validate {
        #[command(flatten)]
        targets: TargetArgs,
        #[command(flatten)]
        auth: AuthArgs,
        /// Per-check timeout. Default: `[validate] timeout` from the config.
        #[arg(long, value_parser = humantime::parse_duration)]
        timeout: Option<Duration>,
        /// Extra PEM root to trust for https/wss targets (a staging edge, Caddy's
        /// internal CA). Default: `[validate] ca_cert` from the config.
        #[arg(long, env = "CHAOS_CA_CERT")]
        ca_cert: Option<PathBuf>,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Serve the HTTP API (and the UI when built) until Ctrl-C.
    Serve(ServeArgs),
    /// Print the effective configuration for the environment as TOML.
    Config,
    /// Stress campaigns against the ledger: model-checking workers, findings.
    Stress {
        #[command(subcommand)]
        command: StressCmd,
    },
    /// List the service kinds and the validate checks.
    Kinds {
        /// Emit JSON: `{"kinds": [...], "checks": [...]}`.
        #[arg(long, conflicts_with = "md")]
        json: bool,
        /// Emit Markdown tables (`docs/chaos/kinds.md`).
        #[arg(long)]
        md: bool,
    },
}

/// `chaos stress ...`.
#[derive(Subcommand)]
// `Run` carries the target and auth flags; `Check` a list of paths. Built once.
#[allow(clippy::large_enum_variant)]
enum StressCmd {
    /// Run campaign files: boot the stack (or use `--target ledger=URL`), drive
    /// the workers, play the timeline, report the findings.
    Run {
        /// Campaign files or glob patterns.
        #[arg(required_unless_present = "dir")]
        files: Vec<String>,
        /// Run every `*.toml` under a directory, recursively.
        #[arg(long, short)]
        dir: Option<PathBuf>,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
        /// Override `[campaign] seed` (a nightly run over seeds).
        #[arg(long)]
        seed: Option<u64>,
        #[command(flatten)]
        targets: TargetArgs,
        #[command(flatten)]
        auth: AuthArgs,
        /// Extra PEM root to trust for an https target. Default: `[validate] ca_cert`.
        #[arg(long, env = "CHAOS_CA_CERT")]
        ca_cert: Option<PathBuf>,
        /// Where findings are written, one JSON file per finding.
        #[arg(long, env = "CHAOS_FINDINGS_DIR", default_value = ".chaos/findings")]
        findings_dir: PathBuf,
    },
    /// Parse and check campaign files without running them.
    Check {
        /// Campaign files.
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Replay a finding's trace against a ledger and say whether it reproduces.
    Replay {
        /// A finding id under the findings directory, or a path to its JSON file.
        finding: String,
        /// Replays to try before giving up (a race needs more than one).
        #[arg(long, default_value_t = 1)]
        attempts: u32,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        targets: TargetArgs,
        #[command(flatten)]
        auth: AuthArgs,
        /// Extra PEM root to trust for an https target. Default: `[validate] ca_cert`.
        #[arg(long, env = "CHAOS_CA_CERT")]
        ca_cert: Option<PathBuf>,
        /// Where findings live.
        #[arg(long, env = "CHAOS_FINDINGS_DIR", default_value = ".chaos/findings")]
        findings_dir: PathBuf,
    },
}

/// Services run in-process and log every injected failure at error level,
/// which is correct in production and noise here. Unless the caller set a
/// filter, keep the tool's own logs and silence the services' request logs.
const DEFAULT_FILTER: &str = "warn,tbd_chaos=info,tbd_protocol::error=off,tower_http=off";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
    // Needs neither config nor logging, and its stdout is piped into a file.
    if let Command::Kinds { json, md } = cli.command {
        return kinds_command(json, md);
    }
    if std::env::var_os("RUST_LOG").is_none() && cli.telemetry.filter == "info" {
        DEFAULT_FILTER.clone_into(&mut cli.telemetry.filter);
    }
    let _telemetry = tbd_common::telemetry::init(&cli.telemetry, "chaos")?;
    let (mut config, source) = ChaosConfig::load(&cli.config_dir, &cli.env)?;
    if let Some(domain) = cli.public_domain {
        config.links.domain = domain;
    }
    config.links = config.links.resolved();
    tracing::info!(env = %source.env, files = ?source.files, "config");

    match cli.command {
        Command::Up { file } => up(file.unwrap_or(config.paths.topology)).await,
        Command::Run { files, dir, json } => run(files, dir, json).await,
        Command::Check { files } => {
            let mut ok = true;
            for f in files {
                match tbd_chaos::scenario::ScenarioFile::from_path(&f) {
                    Ok(s) => println!("ok    {}  ({})", f.display(), s.scenario.name),
                    Err(e) => {
                        ok = false;
                        println!("error {e}");
                    }
                }
            }
            if ok { Ok(()) } else { std::process::exit(1) }
        }
        Command::Validate {
            targets,
            auth,
            timeout,
            ca_cert,
            json,
        } => {
            targets.apply(&mut config);
            auth.apply(&mut config);
            if let Some(v) = ca_cert {
                config.validate.ca_cert = v;
            }
            config.check()?;
            let targets =
                tbd_chaos::validate::Targets::from_config(&config, &BTreeMap::new(), timeout)?;
            let report = tbd_chaos::validate::run(targets).await;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print!("{}", report.render());
            }
            if report.ok() {
                Ok(())
            } else {
                std::process::exit(1)
            }
        }
        Command::Serve(args) => {
            args.apply(&mut config);
            config.check()?;
            serve(config, source).await
        }
        Command::Config => {
            println!("# env: {}", source.env);
            for f in &source.files {
                println!("# {}", f.display());
            }
            print!("{}", toml::to_string_pretty(&config)?);
            Ok(())
        }
        Command::Stress { command } => stress(command, config).await,
        Command::Kinds { .. } => Ok(()),
    }
}

/// Expand files and globs, plus a directory, into sorted unique paths.
fn expand_paths(files: Vec<String>, dir: Option<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    if let Some(dir) = dir {
        let pattern = dir.join("**/*.toml");
        for entry in glob::glob(&pattern.to_string_lossy())? {
            paths.push(entry?);
        }
    }
    for f in files {
        if f.contains('*') || f.contains('?') {
            for entry in glob::glob(&f)? {
                paths.push(entry?);
            }
        } else {
            paths.push(PathBuf::from(f));
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

async fn stress(command: StressCmd, mut config: ChaosConfig) -> anyhow::Result<()> {
    match command {
        StressCmd::Check { files } => {
            let mut ok = true;
            for f in files {
                match tbd_chaos::stress::load_campaign(&f) {
                    Ok(c) => println!("ok     {} ({})", f.display(), c.campaign.campaign.name),
                    Err(e) => {
                        ok = false;
                        println!("error  {}: {e}", f.display());
                    }
                }
            }
            if ok { Ok(()) } else { std::process::exit(1) }
        }
        StressCmd::Replay {
            finding,
            attempts,
            json,
            targets,
            auth,
            ca_cert,
            findings_dir,
        } => {
            targets.apply(&mut config);
            auth.apply(&mut config);
            if let Some(v) = ca_cert {
                config.validate.ca_cert = v;
            }
            config.check()?;
            stress_replay(&config, &findings_dir, &finding, attempts, json).await
        }
        StressCmd::Run {
            files,
            dir,
            json,
            seed,
            targets,
            auth,
            ca_cert,
            findings_dir,
        } => {
            // Explicit targets mean "no stack": the ledger URL from the flags
            // (or CHAOS_LEDGER_URL), never the config's default.
            let explicit = !targets.targets.is_empty()
                || targets.ledger.is_some()
                || kinds::by_name("ledger").is_some_and(|k| {
                    std::env::var(k.env_var()).is_ok_and(|v| !v.trim().is_empty())
                });
            targets.apply(&mut config);
            auth.apply(&mut config);
            if let Some(v) = ca_cert {
                config.validate.ca_cert = v;
            }
            config.check()?;
            let options = tbd_chaos::stress::RunOptions {
                targets: explicit.then(|| {
                    config
                        .targets
                        .url("ledger")
                        .map(|url| {
                            vec![tbd_chaos::load::Target {
                                name: "ledger".into(),
                                http_url: url,
                                kind: "ledger".into(),
                            }]
                        })
                        .unwrap_or_default()
                }),
                seed,
                trust: config.trust()?,
            };
            let paths = expand_paths(files, dir)?;
            anyhow::ensure!(!paths.is_empty(), "no campaign files found");
            let hooks = tbd_chaos::stress::Hooks::default();
            let mut results = Vec::with_capacity(paths.len());
            for path in &paths {
                let result = tbd_chaos::stress::run_file_with(path, &options, &hooks).await;
                let written = tbd_chaos::stress::write_findings(&findings_dir, &result)
                    .map_err(|e| anyhow::anyhow!(e))?;
                if !json {
                    print!("{}", tbd_stress::render(&result));
                    for w in &written {
                        println!("      written   {}", w.display());
                    }
                }
                results.push(result);
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                println!("{}", tbd_stress::summary(&results));
            }
            if results.iter().all(|r| r.passed) {
                Ok(())
            } else {
                std::process::exit(1)
            }
        }
    }
}

fn kinds_command(json: bool, md: bool) -> anyhow::Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "kinds": kinds::describe(),
                "checks": tbd_chaos::validate::catalogue(),
            }))?
        );
    } else if md {
        print!("{}", kinds::markdown());
    } else {
        for k in kinds::ALL {
            let target = k
                .target
                .as_ref()
                .map_or("no validate target".to_owned(), |t| {
                    format!("target {} ({})", t.default_url, k.env_var())
                });
            println!(
                "{:<10} {:<26} {:<22} {target}",
                k.name,
                format!("[stack.{}.<name>]", k.plural),
                k.surface
            );
        }
        println!();
        for (k, c) in tbd_chaos::validate::checks() {
            println!("{:<22} {:<8} {}", c.name, c.surface, k.name);
        }
    }
    Ok(())
}

impl TargetArgs {
    fn apply(self, config: &mut ChaosConfig) {
        for (kind, _) in kinds::with_target() {
            if let Ok(url) = std::env::var(kind.env_var())
                && !url.trim().is_empty()
            {
                config.targets.set(kind.name, url);
            }
        }
        for (kind, url) in self.targets {
            config.targets.set(&kind, url);
        }
        for (kind, url) in [
            ("protocol", self.protocol),
            ("engine", self.engine),
            ("ledger", self.ledger),
        ] {
            if let Some(url) = url {
                config.targets.set(kind, url);
            }
        }
    }
}

async fn serve(config: ChaosConfig, source: Source) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.serve.listen)
        .await
        .with_context(|| format!("bind {}", config.serve.listen))?;
    let addr = listener.local_addr()?;
    let state = tbd_chaos::api::state(config, source).await?;

    println!("\nchaos serve on http://{addr}");
    println!(
        "  api   http://{addr}{}/overview",
        state.config.serve.base_path
    );
    if state.config.serve.ui_dir.is_empty() {
        println!("  ui    not configured ([serve] ui_dir)");
    } else {
        println!("  ui    http://{addr}{}/", state.config.serve.ui_path);
    }
    match state.stack_info().await {
        Ok(instances) => {
            println!("  stack {}", state.config.paths.topology.display());
            for i in instances {
                println!("        {:<12} {:<9} {}", i.name, i.kind, i.addr);
            }
        }
        Err(_) => println!("  stack none (--no-stack)"),
    }
    println!("\nCtrl-C to stop.");
    tbd_chaos::api::serve(state, listener, tbd_common::shutdown::signal()).await
}

async fn up(file: PathBuf) -> anyhow::Result<()> {
    let text =
        std::fs::read_to_string(&file).with_context(|| format!("read {}", file.display()))?;
    let topology: tbd_chaos::topology::TopologyFile =
        toml::from_str(&text).with_context(|| format!("parse {}", file.display()))?;
    topology
        .stack
        .check()
        .map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;
    let stack = topology.stack.start().await?;

    println!("\nstack up:");
    for (name, instance) in stack.instances() {
        let surface = kinds::by_name(instance.kind).map_or("", |k| k.surface);
        println!(
            "  {:<12} {:<9} {:<22} {surface}",
            name, instance.kind, instance.addr
        );
    }
    let hint: Vec<String> = kinds::with_target()
        .filter_map(|(k, _)| {
            stack
                .of_kind(k.name)
                .first()
                .map(|i| format!("--target {}={}", k.name, i.http_url()))
        })
        .collect();
    if !hint.is_empty() {
        println!("\n  chaos validate {}", hint.join(" "));
    }
    println!("\nCtrl-C to stop.");
    tbd_common::shutdown::signal().await;
    stack.shutdown().await;
    Ok(())
}

async fn run(files: Vec<String>, dir: Option<PathBuf>, json: bool) -> anyhow::Result<()> {
    let paths = expand_paths(files, dir)?;
    anyhow::ensure!(!paths.is_empty(), "no scenario files found");

    let mut results = Vec::with_capacity(paths.len());
    for path in &paths {
        let result = tbd_chaos::scenario::run_file(path).await;
        if !json {
            print!("{}", tbd_chaos::scenario::report::render(&result));
        }
        results.push(result);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("{}", tbd_chaos::scenario::report::summary(&results));
    }
    if results.iter().all(|r| r.passed) {
        Ok(())
    } else {
        std::process::exit(1)
    }
}

async fn stress_replay(
    config: &ChaosConfig,
    findings_dir: &std::path::Path,
    finding: &str,
    attempts: u32,
    json: bool,
) -> anyhow::Result<()> {
    let (path, mut f) =
        tbd_chaos::stress::read_finding(findings_dir, finding).map_err(|e| anyhow::anyhow!(e))?;
    let ledger = config
        .targets
        .url("ledger")
        .map(|url| tbd_chaos::load::Target {
            name: "ledger".into(),
            http_url: url,
            kind: "ledger".into(),
        })
        .into_iter()
        .collect::<Vec<_>>();
    let outcome = tbd_chaos::stress::replay_finding(
        &path,
        &mut f,
        &ledger,
        &config.trust()?,
        attempts,
        Duration::from_secs(5),
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!(
            "{}  {}  {}  ({} steps, {} attempt(s)){}",
            if outcome.reproduced {
                "REPRODUCED"
            } else {
                "not reproduced"
            },
            f.invariant,
            f.message,
            outcome.steps_run,
            attempts,
            outcome
                .message
                .as_deref()
                .filter(|_| outcome.reproduced)
                .map(|m| format!("\n      {m}"))
                .unwrap_or_default()
        );
    }
    if outcome.reproduced {
        std::process::exit(1)
    }
    Ok(())
}
