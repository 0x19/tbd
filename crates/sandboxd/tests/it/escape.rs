//! The escape suite (RFC 0010, study 0003): hostile programs, each asserted to
//! be contained. The numbers in the comments are the RFC's threat list. Every
//! run prints one line (`escape <case> <outcome> <killed> <ms>`) for the study.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stderr)]

use std::time::Instant;

use serde_json::{Value, json};

use crate::support::{Daemon, run};

async fn case(d: &Daemon, name: &str, language: &str, source: &str) -> (Value, u128) {
    let t = Instant::now();
    let r = run(d, json!({"language": language, "source": source})).await;
    let ms = t.elapsed().as_millis();
    let killed = r["run"]["killed"]
        .as_str()
        .or(r["compile"]["killed"].as_str())
        .unwrap_or("-");
    eprintln!(
        "escape {name} {} {killed} {ms}",
        r["outcome"].as_str().unwrap_or("?")
    );
    (r, ms)
}

fn out(r: &Value) -> String {
    format!(
        "{}{}",
        r["run"]["stdout"].as_str().unwrap_or_default(),
        r["run"]["stderr"].as_str().unwrap_or_default()
    )
}

const GO_DIAL: &str = r#"package main
import ("fmt";"net";"time")
func main() {
	for _, a := range []string{"1.1.1.1:80", "172.21.0.1:7788", "10.43.0.10:53", "127.0.0.1:22"} {
		c, err := net.DialTimeout("tcp", a, 2*time.Second)
		if err == nil { fmt.Println("CONNECTED", a); c.Close() } else { fmt.Println("refused", a, err) }
	}
	_, err := net.LookupHost("example.org")
	fmt.Println("dns", err != nil)
}
"#;

/// 1: no network of any kind: the internet, the machine, the cluster, loopback.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn it_reaches_no_network() {
    let d = Daemon::start().await;
    let (r, _) = case(&d, "network", "go", GO_DIAL).await;
    let o = out(&r);
    assert!(!o.contains("CONNECTED"), "{r}");
    assert!(o.contains("dns true"), "name resolution must fail: {r}");
}

const GO_HOST: &str = r#"package main
import ("fmt";"os";"path/filepath")
func main() {
	h, _ := os.ReadFile("/etc/hostname")
	fmt.Printf("hostname=%s", h)
	procs, _ := filepath.Glob("/proc/[0-9]*")
	fmt.Println("procs", len(procs))
	for _, p := range []string{"/etc/x", "/usr/bin/x", "/opt/gocache/x"} {
		fmt.Println("write", p, os.WriteFile(p, []byte("x"), 0o644) == nil)
	}
	_, err := os.Stat("/var/run/docker.sock")
	fmt.Println("docker socket visible", err == nil)
	u, _ := os.ReadFile("/proc/self/status")
	fmt.Println(len(u) > 0)
}
"#;

/// 2: the machine is not there: its own hostname, a handful of processes, a
/// read-only filesystem outside the scratch space, no Docker socket.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn it_cannot_see_or_write_the_machine() {
    let d = Daemon::start().await;
    let (r, _) = case(&d, "machine", "go", GO_HOST).await;
    let o = out(&r);
    assert!(o.contains("hostname=sandbox"), "{r}");
    let procs: usize = o
        .lines()
        .find_map(|l| l.strip_prefix("procs "))
        .and_then(|n| n.trim().parse().ok())
        .unwrap();
    assert!(procs < 10, "only the sandbox's own processes: {procs}");
    for p in ["/etc/x", "/usr/bin/x", "/opt/gocache/x"] {
        assert!(
            o.contains(&format!("write {p} false")),
            "{p} must be read-only: {r}"
        );
    }
    assert!(o.contains("docker socket visible false"), "{r}");
}

/// 2: no privileged call works: mount, a new namespace, tracing, eBPF.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn privileged_calls_fail() {
    let d = Daemon::start().await;
    let src = r#"package main
import ("fmt";"syscall")
func main() {
	fmt.Println("mount", syscall.Mount("none", "/work", "tmpfs", 0, "") == nil)
	fmt.Println("unshare", syscall.Unshare(syscall.CLONE_NEWUSER|syscall.CLONE_NEWNS) == nil)
	fmt.Println("ptrace", syscall.PtraceAttach(1) == nil)
	_, _, e := syscall.Syscall(321, 0, 0, 0) // bpf
	fmt.Println("bpf", e == 0)
	fmt.Println("setuid", syscall.Setuid(0) == nil)
}
"#;
    let (r, _) = case(&d, "privileged", "go", src).await;
    let o = out(&r);
    for call in ["mount", "unshare", "ptrace", "bpf", "setuid"] {
        assert!(
            o.contains(&format!("{call} false")),
            "{call} must fail: {r}"
        );
    }
}

/// 3: a fork bomb meets the process limit and the deadline.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn a_fork_bomb_is_contained() {
    let d = Daemon::start().await;
    let src = r#"use std::process::Command;
fn main() {
    let mut n = 0;
    loop {
        match Command::new("/work/prog").spawn() {
            Ok(_) => n += 1,
            Err(e) => { println!("stopped after {n}: {e}"); std::thread::sleep(std::time::Duration::from_secs(60)); }
        }
    }
}
"#;
    // Contained either way: the deadline kills it, or the process limit ends
    // the sandbox (gVisor then reports the program's end as an error exit).
    let (r, ms) = case(&d, "fork_bomb", "rust", src).await;
    assert_eq!(r["outcome"], "killed", "{r}");
    assert!(ms < 15_000, "{ms} ms");
    let (after, _) = case(
        &d,
        "after_fork_bomb",
        "go",
        "package main\nfunc main(){ println(\"alive\") }\n",
    )
    .await;
    assert_eq!(
        after["outcome"], "ok",
        "the daemon still runs the next program: {after}"
    );
}

/// 3: memory past the limit is killed.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn memory_past_the_limit_is_killed() {
    let d = Daemon::start().await;
    let src =
        "fn main() { let mut v: Vec<Vec<u8>> = Vec::new(); loop { v.push(vec![7u8; 64 << 20]); } }";
    let (r, _) = case(&d, "memory", "rust", src).await;
    assert_eq!(r["outcome"], "killed", "{r}");
    let why = r["run"]["killed"].as_str().unwrap();
    assert!(why == "memory" || why == "limit", "{r}");
    assert!(
        !r["run"]["stderr"].as_str().unwrap().contains("urpc"),
        "no runtime internals: {r}"
    );
}

/// 3: the disk: the scratch space is small and in memory, a file has a cap.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn filling_the_disk_fails_inside() {
    let d = Daemon::start().await;
    let src = r#"use std::io::Write;
fn main() {
    let mut f = std::fs::File::create("/work/fill").unwrap();
    let chunk = vec![0u8; 1 << 20];
    let mut mb = 0;
    loop {
        if let Err(e) = f.write_all(&chunk) { println!("stopped at {mb} MB: {e}"); return; }
        mb += 1;
    }
}
"#;
    // Either the write fails inside, or the file-size cap ends the program.
    let (r, _) = case(&d, "disk", "rust", src).await;
    if r["run"]["killed"] == "file_size" {
        return;
    }
    let o = out(&r);
    assert!(o.contains("stopped at"), "{r}");
    let mb: u64 = o
        .split("stopped at ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(u64::MAX);
    assert!(mb <= 256, "{mb} MB written");
}

/// 4: spinning and sleeping forever both end at the deadline.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn forever_ends_at_the_deadline() {
    let d = Daemon::start().await;
    for (name, src) in [
        (
            "spin",
            "fn main() { let mut x: u64 = 0; loop { x = x.wrapping_add(1); std::hint::black_box(x); } }",
        ),
        (
            "sleep",
            "fn main() { std::thread::sleep(std::time::Duration::from_secs(3600)); }",
        ),
    ] {
        let (r, ms) = case(&d, name, "rust", src).await;
        assert_eq!(r["run"]["killed"], "timeout", "{name}: {r}");
        assert!((4_000..12_000).contains(&ms), "{name}: {ms} ms");
    }
}

/// 7: endless output is cut at the cap and the run stopped.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn endless_output_is_cut() {
    let d = Daemon::start().await;
    let src = "fn main() { loop { println!(\"<script>alert(1)</script> flood flood flood\"); } }";
    let (r, ms) = case(&d, "output", "rust", src).await;
    assert_eq!(r["run"]["killed"], "output", "{r}");
    assert!(r["run"]["truncated"].as_bool().unwrap());
    assert!(r["run"]["stdout"].as_str().unwrap().len() <= 65536);
    assert!(ms < 8_000, "{ms} ms");
}

/// 5: one run leaves nothing for the next.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn a_run_leaves_nothing_for_the_next() {
    let d = Daemon::start().await;
    let write = r#"fn main() { std::fs::write("/work/secret", "left behind").unwrap(); std::fs::write("/tmp/secret", "x").ok(); println!("written"); }"#;
    let read = r#"fn main() { println!("work {} tmp {}", std::path::Path::new("/work/secret").exists(), std::path::Path::new("/tmp/secret").exists()); }"#;
    let (w, _) = case(&d, "leave", "rust", write).await;
    assert_eq!(w["outcome"], "ok", "{w}");
    let (r, _) = case(&d, "find", "rust", read).await;
    assert_eq!(out(&r).trim(), "work false tmp false", "{r}");
}

/// 6: the compiler: a build that asks for a generator runs none, a C import
/// fails, and compile-time work ends at the deadline or the compiler's limit.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn the_compiler_runs_nothing_else() {
    let d = Daemon::start().await;
    let generate = "package main\n//go:generate sh -c \"touch /work/generated\"\nimport (\"fmt\";\"os\")\nfunc main(){_, err := os.Stat(\"/work/generated\"); fmt.Println(\"generated\", err == nil)}\n";
    let (r, _) = case(&d, "go_generate", "go", generate).await;
    assert_eq!(out(&r).trim(), "generated false", "{r}");

    let cgo =
        "package main\n// int two() { return 2; }\nimport \"C\"\nfunc main(){ println(C.two()) }\n";
    let (r, _) = case(&d, "cgo", "go", cgo).await;
    assert_eq!(r["outcome"], "compile_error", "{r}");

    let const_loop = "const fn spin() -> u64 { let mut i: u64 = 0; while i < u64::MAX { i += 1; } i }\nconst X: u64 = spin();\nfn main() { println!(\"{X}\"); }\n";
    let (r, ms) = case(&d, "const_eval", "rust", const_loop).await;
    assert!(
        r["outcome"] == "compile_error" || r["compile"]["killed"] == "timeout",
        "{r}"
    );
    assert!(ms < 30_000, "{ms} ms");
}

/// Nothing is left on the machine: every sandbox of the suite is gone.
#[tokio::test]
#[ignore = "needs gVisor and the sandbox images on the machine"]
async fn no_sandbox_is_left_behind() {
    let d = Daemon::start().await;
    let (r, _) = case(&d, "cleanup", "go", "package main\nfunc main(){}\n").await;
    assert_eq!(r["outcome"], "ok", "{r}");
    // Removal after a kill is asynchronous in the daemon; give it a moment.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let ps = tokio::process::Command::new("docker")
        .args(["ps", "-aq", "--filter", "label=tbd.sandbox"])
        .output()
        .await
        .unwrap();
    let left = String::from_utf8_lossy(&ps.stdout).trim().to_owned();
    assert!(left.is_empty(), "left behind: {left}");
}
