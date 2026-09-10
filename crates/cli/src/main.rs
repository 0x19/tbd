//! `tbd` binary. Stdout is the product here, so the print lints are allowed in
//! this file only.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use owo_colors::{OwoColorize, Stream};
use tbd_cli::{
    check,
    edit::{Edit, Status},
    list,
    ports::Ports,
    registry,
    repo::Workspace,
    scaffold,
    service::{Kind, Service, ServiceName},
    template::{self, Vars},
};

#[derive(Parser)]
#[command(name = "tbd", version = tbd_cli::VERSION, about = "Scaffold and register services in this repository")]
struct Cli {
    /// Repository root. Default: the workspace above the current directory.
    #[arg(long, env = "TBD_REPO", global = true)]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create something.
    #[command(subcommand)]
    New(New),
    /// Inspect or repair scaffolded services.
    #[command(subcommand)]
    Service(ServiceCmd),
}

#[derive(Subcommand)]
enum New {
    /// Scaffold a service and register it in every shared file.
    Service(NewService),
}

#[derive(Args)]
struct NewService {
    /// Service name: 2 to 24 lowercase letters and digits, starting with a letter.
    name: String,
    /// Listen port. Default: one above the highest service port in use.
    #[arg(long)]
    port: Option<u16>,
    /// Prometheus port on the developer's machine (containers always use 9464).
    /// Default: one above the highest in use.
    #[arg(long)]
    metrics_port: Option<u16>,
    /// Template kind.
    #[arg(long, value_enum, default_value_t = Kind::Grpc)]
    kind: Kind,
    /// bacon keybinding. Default: the name's first letter.
    #[arg(long)]
    bacon_key: Option<char>,
    /// Show what would change and write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Overwrite generated files that exist with different content.
    #[arg(long)]
    force: bool,
}

#[derive(Subcommand)]
enum ServiceCmd {
    /// Verify every registration of a scaffolded service is present.
    Check {
        /// Service name.
        name: String,
        /// Apply the missing registrations (never creates files).
        #[arg(long)]
        fix: bool,
    },
    /// List binary crates and their registration state.
    List,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!(
                "{} {error:#}",
                "error".if_supports_color(Stream::Stderr, |t| t.red())
            );
            ExitCode::from(1)
        }
    }
}

fn workspace(repo: Option<&Path>) -> anyhow::Result<Workspace> {
    if let Some(root) = repo {
        return Ok(Workspace::open(root));
    }
    let cwd = std::env::current_dir().context("current directory")?;
    Ok(Workspace::discover(&cwd)?)
}

fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    let mut ws = workspace(cli.repo.as_deref())?;
    match cli.command {
        Command::New(New::Service(args)) => new_service(&mut ws, &args),
        Command::Service(ServiceCmd::Check { name, fix }) => service_check(&mut ws, &name, fix),
        Command::Service(ServiceCmd::List) => service_list(&mut ws),
    }
}

fn label(word: &str, colour: fn(&str) -> String) -> String {
    format!("{:<8}", colour(word))
}

fn green(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |t| t.green())
        .to_string()
}
fn yellow(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |t| t.yellow())
        .to_string()
}
fn dim(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |t| t.dimmed())
        .to_string()
}
fn red(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |t| t.red()).to_string()
}

fn new_service(ws: &mut Workspace, args: &NewService) -> anyhow::Result<ExitCode> {
    let name = ServiceName::parse(&args.name)?;
    let ports = Ports::scan(ws)?;
    let port = args.port.unwrap_or_else(|| ports.next_listen());
    let metrics_port = args.metrics_port.unwrap_or_else(|| ports.next_metrics());
    let bacon_key = args
        .bacon_key
        .or_else(|| name.as_str().chars().next())
        .context("empty name")?;
    let service = Service {
        name,
        kind: args.kind,
        port,
        metrics_port,
        bacon_key,
    };

    let existing = check::service_from_marker(ws, service.name.as_str())?;
    let already = existing.is_some();
    if let Some(prior) = &existing
        && (prior.port != service.port || prior.metrics_port != service.metrics_port)
        && args.port.is_none()
    {
        // Re-running without flags: keep the ports the crate was scaffolded with.
        return new_service_with(ws, prior, args.dry_run, args.force, true);
    }
    if !already && ports.listen_in_use(service.port) {
        anyhow::bail!(
            "port {} is already used by another service; pass --port",
            service.port
        );
    }
    if !already && bacon_key_taken(ws, bacon_key)? {
        anyhow::bail!("bacon key {bacon_key:?} is already bound; pass --bacon-key");
    }
    new_service_with(ws, &service, args.dry_run, args.force, already)
}

fn bacon_key_taken(ws: &mut Workspace, key: char) -> anyhow::Result<bool> {
    let Some(text) = ws.read(Path::new("bacon.toml"))? else {
        return Ok(false);
    };
    let needle = format!("{key} = \"job:");
    Ok(text.lines().any(|l| l.starts_with(&needle)))
}

fn new_service_with(
    ws: &mut Workspace,
    service: &Service,
    dry_run: bool,
    force: bool,
    rerun: bool,
) -> anyhow::Result<ExitCode> {
    let regs = registry::registrations(service)?;
    let plan = scaffold::plan(ws, regs)?;
    let blockers = plan.blockers(force);
    if !blockers.is_empty() {
        for b in &blockers {
            println!("{}{b}", label("error", red));
        }
        println!(
            "{} registration(s) blocked; nothing written",
            blockers.len()
        );
        return Ok(ExitCode::from(1));
    }
    if plan.all_present() {
        println!(
            "{}{} is already scaffolded ({} registrations present)",
            label("ok", green),
            service.name,
            plan.items.len()
        );
        return Ok(ExitCode::SUCCESS);
    }
    if dry_run {
        for item in &plan.items {
            let reg = &item.registration;
            match &item.status {
                Status::Present => println!("{}{}", label("skip", dim), reg.path.display()),
                Status::Missing | Status::Conflict(_) => {
                    let verb = if matches!(reg.edit, Edit::Create { .. }) {
                        "create"
                    } else {
                        "edit"
                    };
                    println!(
                        "{}{}  [{}]",
                        label(verb, yellow),
                        reg.path.display(),
                        reg.id
                    );
                    for line in reg.edit.added() {
                        println!("        + {line}");
                    }
                }
                Status::Unresolvable(_) => {}
            }
        }
        println!("dry run: nothing written");
        return Ok(ExitCode::SUCCESS);
    }
    let outcome = scaffold::apply(ws, &plan, force, true)?;
    scaffold::verify_toml(ws)?;
    ws.commit()
        .context("writing files; `git checkout -- .` rolls back")?;
    for path in &outcome.created {
        println!("{}{}", label("create", green), path.display());
    }
    for id in &outcome.edited {
        println!("{}{id}", label("edit", yellow));
    }
    if rerun {
        println!(
            "{}{} present, {} applied",
            label("ok", green),
            outcome.skipped.len(),
            outcome.edited.len() + outcome.created.len()
        );
    }
    if !outcome.created.is_empty() {
        println!();
        println!("next:");
        let vars = Vars::for_service(service);
        for step in registry::CHECKLIST {
            println!("  {}", template::render(step, &vars)?);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn service_check(ws: &mut Workspace, name: &str, fix: bool) -> anyhow::Result<ExitCode> {
    ServiceName::parse(name)?;
    let Some(service) = check::service_from_marker(ws, name)? else {
        println!(
            "{}crates/{name}/CLAUDE.md carries no `tbd new service` marker",
            label("error", red)
        );
        return Ok(ExitCode::from(2));
    };
    if fix {
        let regs = registry::registrations(&service)?;
        let plan = scaffold::plan(ws, regs)?;
        let outcome = scaffold::apply(ws, &plan, false, false)?;
        scaffold::verify_toml(ws)?;
        ws.commit()?;
        for id in &outcome.edited {
            println!("{}{id}", label("edit", yellow));
        }
    }
    let report = check::check(ws, &service)?;
    let mut problems = 0;
    for line in report.problems() {
        problems += 1;
        let (word, why) = match &line.status {
            Status::Missing => ("missing", String::new()),
            Status::Conflict(w) | Status::Unresolvable(w) => ("blocked", w.clone()),
            Status::Present => unreachable!(),
        };
        println!("{}{}  {}  {why}", label(word, red), line.id, line.path);
    }
    if problems == 0 {
        println!(
            "{}{} registrations present for {}",
            label("ok", green),
            report.lines.len(),
            service.name
        );
        Ok(ExitCode::SUCCESS)
    } else {
        println!("{problems} of {} registrations missing", report.lines.len());
        Ok(ExitCode::from(1))
    }
}

fn service_list(ws: &mut Workspace) -> anyhow::Result<ExitCode> {
    let rows = list::rows(ws)?;
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(["name", "package", "kind", "port", "managed", "registered"]);
    for row in rows {
        let (kind, port, managed, registered) = match (&row.managed, row.registered) {
            (Some(s), Some((present, total))) => (
                s.kind.to_string(),
                s.port.to_string(),
                "yes".to_owned(),
                format!("{present}/{total}"),
            ),
            (Some(s), None) => (
                s.kind.to_string(),
                s.port.to_string(),
                "yes".into(),
                "?".into(),
            ),
            (None, _) => ("-".into(), "-".into(), "no".into(), "-".into()),
        };
        table.add_row([row.name, row.package, kind, port, managed, registered]);
    }
    println!("{table}");
    Ok(ExitCode::SUCCESS)
}
