//! `chaos` binary. Stdout is the product here, so the print lints are allowed
//! in this file only.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use tbd_chaos::config::{self, ChaosConfig, Source};
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

/// Flags that override `[targets]` in the config.
#[derive(Args, Debug, Clone)]
struct TargetArgs {
    /// Protocol base URL.
    #[arg(long, env = "CHAOS_PROTOCOL_URL")]
    protocol: Option<String>,
    /// Engine gRPC URL.
    #[arg(long, env = "CHAOS_ENGINE_URL")]
    engine: Option<String>,
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
    /// Copied into the scenario directory when it is missing or empty.
    /// Default: `[paths] scenarios_seed`.
    #[arg(long, env = "CHAOS_SCENARIOS_SEED")]
    scenarios_seed: Option<PathBuf>,
    /// Run records directory. Default: `[paths] results`.
    #[arg(long, env = "CHAOS_RESULTS_DIR")]
    results: Option<PathBuf>,
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
        if self.no_stack {
            config.serve.start_stack = false;
        }
    }
}

#[derive(Subcommand)]
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
}

/// Services run in-process and log every injected failure at error level,
/// which is correct in production and noise here. Unless the caller set a
/// filter, keep the tool's own logs and silence the services' request logs.
const DEFAULT_FILTER: &str = "warn,tbd_chaos=info,tbd_protocol::error=off,tower_http=off";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
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
            let trust = config.trust()?;
            let report = tbd_chaos::validate::run(tbd_chaos::validate::Targets {
                protocol: config.targets.protocol,
                engine: config.targets.engine,
                timeout: timeout.unwrap_or(config.validate.timeout),
                trust,
            })
            .await;
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
    }
}

impl TargetArgs {
    fn apply(self, config: &mut ChaosConfig) {
        if let Some(p) = self.protocol {
            config.targets.protocol = p;
        }
        if let Some(e) = self.engine {
            config.targets.engine = e;
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
        let surface = match instance.kind {
            "engine" => "grpc",
            "protocol" => "http/ws/graphql/grpc",
            _ => "",
        };
        println!(
            "  {:<12} {:<9} {:<22} {surface}",
            name, instance.kind, instance.addr
        );
    }
    if let Some(p) = stack.of_kind("protocol").first() {
        println!(
            "\n  chaos validate --protocol {} --engine {}",
            p.http_url(),
            stack
                .of_kind("engine")
                .first()
                .map(|e| e.http_url())
                .unwrap_or_default()
        );
    }
    println!("\nCtrl-C to stop.");
    tbd_common::shutdown::signal().await;
    stack.shutdown().await;
    Ok(())
}

async fn run(files: Vec<String>, dir: Option<PathBuf>, json: bool) -> anyhow::Result<()> {
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
