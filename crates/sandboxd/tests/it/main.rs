//! End to end on a machine with gVisor and the sandbox images
//! (`mise run sandbox:install`, `mise run sandbox:images`): the real daemon on
//! port 0, real sandboxes. Ignored by default (CI has neither); run with
//! `mise run sandbox:test`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod escape;
mod support;

use serde_json::json;

use crate::support::{Daemon, run};

#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn go_and_rust_hello_world_run_and_read_their_input() {
    let d = Daemon::start().await;
    let go = run(&d, json!({"language": "go", "stdin": "gopher", "source":
        "package main\nimport (\"bufio\";\"fmt\";\"os\")\nfunc main(){s:=bufio.NewScanner(os.Stdin);s.Scan();fmt.Println(\"hello\", s.Text())}\n"})).await;
    assert_eq!(go["outcome"], "ok", "{go}");
    assert_eq!(go["run"]["stdout"], "hello gopher\n", "{go}");
    let rust = run(&d, json!({"language": "rust", "stdin": "crab", "source":
        "use std::io::Read;\nfn main(){let mut s=String::new();std::io::stdin().read_to_string(&mut s).unwrap();println!(\"hello {}\", s.trim());}\n"})).await;
    assert_eq!(rust["outcome"], "ok", "{rust}");
    assert_eq!(rust["run"]["stdout"], "hello crab\n", "{rust}");
}

#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn a_compile_error_is_the_compile_step_and_nothing_runs() {
    let d = Daemon::start().await;
    let r = run(
        &d,
        json!({"language": "rust", "source": "fn main() { let x: i32 = \"no\"; }"}),
    )
    .await;
    assert_eq!(r["outcome"], "compile_error", "{r}");
    assert!(
        r["compile"]["stderr"]
            .as_str()
            .unwrap()
            .contains("mismatched types"),
        "{r}"
    );
    assert!(r["run"].is_null(), "{r}");
}

#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn a_wrong_token_is_refused_before_anything_starts() {
    let d = Daemon::start().await;
    let resp = reqwest::Client::new()
        .post(format!("{}/run", d.url))
        .header("authorization", "Bearer not-the-token-not-the-token-not-it")
        .json(&json!({"language": "go", "source": "package main\nfunc main(){}"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
