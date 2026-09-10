//! Integration tests: render every template, and resolve every registration
//! against the real repository so a reformatted shared file is caught here,
//! not by the next person scaffolding a service.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use tbd_cli::{
    edit::{Edit, Status},
    registry,
    repo::Workspace,
    scaffold,
    service::{Kind, Service, ServiceName},
    template::{self, Vars},
    templates,
};

fn zeta() -> Service {
    Service {
        name: ServiceName::parse("zeta").unwrap(),
        kind: Kind::Grpc,
        port: 50099,
        metrics_port: 9499,
        bacon_key: 'z',
    }
}

fn repo() -> Workspace {
    Workspace::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

#[test]
fn every_template_renders_without_leftover_tokens() {
    let vars = Vars::for_service(&zeta());
    for t in templates::GRPC {
        let out = template::render(t.out, &vars).unwrap();
        let body = template::render(t.body, &vars).unwrap();
        assert!(!out.contains("@@"), "{}", t.out);
        assert!(!body.contains("@@"), "{}: leftover token", t.out);
        if Path::new(&out).extension().is_some_and(|e| e == "toml") {
            body.parse::<toml::Table>()
                .unwrap_or_else(|e| panic!("{out}: not TOML: {e}"));
        }
        if Path::new(&out).extension().is_some_and(|e| e == "proto") {
            assert!(body.contains("package tbd.zeta.v1;"));
            assert!(body.contains("service ZetaService {"));
        }
        if out.starts_with("crates/chaos/src/kinds/") {
            assert!(body.contains("pub static KIND: Kind"), "{out}");
            assert!(body.contains("name: \"zeta\""), "{out}");
            assert!(body.contains("plural: \"zetas\""), "{out}");
            assert!(body.contains("grpc_zeta_ping"), "{out}");
            assert!(
                body.contains("default_url: \"http://127.0.0.1:50099\""),
                "{out}"
            );
        }
    }
}

#[test]
fn every_registration_resolves_against_the_real_tree() {
    let mut ws = repo();
    let plan = scaffold::plan(&mut ws, registry::registrations(&zeta()).unwrap()).unwrap();
    let unresolvable: Vec<String> = plan
        .items
        .iter()
        .filter_map(|i| match &i.status {
            Status::Unresolvable(why) => Some(format!("{}: {why}", i.registration.id)),
            _ => None,
        })
        .collect();
    assert!(
        unresolvable.is_empty(),
        "anchors not found; a shared file changed shape:\n{}",
        unresolvable.join("\n")
    );
    let present: Vec<&str> = plan
        .items
        .iter()
        .filter(|i| i.status == Status::Present)
        .map(|i| i.registration.id.as_str())
        .collect();
    assert!(present.is_empty(), "zeta must not exist yet: {present:?}");
}

#[test]
fn applying_twice_is_a_no_op_and_needles_ignore_ports() {
    let mut ws = repo();
    let service = zeta();
    let plan = scaffold::plan(&mut ws, registry::registrations(&service).unwrap()).unwrap();
    let outcome = scaffold::apply(&mut ws, &plan, false, true).unwrap();
    assert!(!outcome.created.is_empty());
    assert!(outcome.skipped.is_empty());
    scaffold::verify_toml(&mut ws).unwrap();

    // Same service again: everything present.
    let again = scaffold::plan(&mut ws, registry::registrations(&service).unwrap()).unwrap();
    let not_present: Vec<String> = again
        .items
        .iter()
        .filter(|i| i.status != Status::Present)
        .map(|i| format!("{}: {:?}", i.registration.id, i.status))
        .collect();
    assert!(not_present.is_empty(), "{}", not_present.join("\n"));

    // A different port: every non-create registration is still recognised.
    let other = Service {
        port: 50123,
        metrics_port: 9555,
        ..service
    };
    let plan = scaffold::plan(&mut ws, registry::registrations(&other).unwrap()).unwrap();
    for item in &plan.items {
        if matches!(item.registration.edit, Edit::Create { .. }) {
            continue;
        }
        assert_eq!(
            item.status,
            Status::Present,
            "{} depends on the port",
            item.registration.id
        );
    }
    assert!(ws.changed().len() > 20);
}
