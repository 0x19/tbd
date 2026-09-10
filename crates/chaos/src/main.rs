//! `chaos` binary. Stdout is the product here, so the print lints are allowed
//! in this file only.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;

#[derive(Parser)]
#[command(name = "chaos", version = tbd_common::VERSION, about = "Validate, load-test and fault-test the stack")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    telemetry: TelemetryArgs,
}

#[derive(Subcommand)]
enum Command {
    /// Start the stack described in a topology file and keep it running until Ctrl-C.
    Up {
        /// Topology or scenario file; only its `[stack]` section is used.
        #[arg(default_value = "topologies/dev.toml")]
        file: PathBuf,
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
        /// Protocol base URL.
        #[arg(
            long,
            env = "CHAOS_PROTOCOL_URL",
            default_value = "http://127.0.0.1:8080"
        )]
        protocol: String,
        /// Engine gRPC URL.
        #[arg(
            long,
            env = "CHAOS_ENGINE_URL",
            default_value = "http://127.0.0.1:50051"
        )]
        engine: String,
        /// Per-check timeout.
        #[arg(long, default_value = "5s", value_parser = humantime::parse_duration)]
        timeout: Duration,
        /// Extra PEM root to trust for https/wss targets (a staging edge, Caddy's internal CA).
        #[arg(long, env = "CHAOS_CA_CERT")]
        ca_cert: Option<PathBuf>,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
    },
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
    match cli.command {
        Command::Up { file } => up(file).await,
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
            protocol,
            engine,
            timeout,
            ca_cert,
            json,
        } => {
            let trust = match ca_cert {
                Some(path) => tbd_chaos::tls::Trust::from_pem_file(&path)?,
                None => tbd_chaos::tls::Trust::default(),
            };
            let report = tbd_chaos::validate::run(tbd_chaos::validate::Targets {
                protocol,
                engine,
                timeout,
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
    }
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
