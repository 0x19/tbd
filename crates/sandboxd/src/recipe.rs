//! The recipe: the only place a container's options and a step's command are
//! written. Nothing from a request reaches either; a language picks a row of
//! [`Language`], and the source and input travel on standard input only.

use serde::{Deserialize, Serialize};

use crate::config::{Images, Limits};

/// Where the program's files live: the in-memory scratch space, the one
/// writable place in the container.
pub const WORK: &str = "/work";

/// The label every sandbox carries, so a leftover can be found and removed.
pub const LABEL: &str = "tbd.sandbox";

/// How long a sandbox's idle process lives at most, seconds: past every
/// deadline, so a container the daemon lost track of still ends itself.
pub const LIFETIME_SECS: u32 = 300;

/// A language the sandbox runs: one file, the standard library only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Go, `CGO_ENABLED=0`, no module downloads.
    Go,
    /// Rust, `rustc` directly: no Cargo, so no build scripts and no crates.
    Rust,
}

impl Language {
    /// Every language, in the order the API lists them.
    pub const ALL: [Self; 2] = [Self::Go, Self::Rust];

    /// The label value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::Rust => "rust",
        }
    }

    /// The source file's name in the scratch space.
    #[must_use]
    pub fn file(self) -> &'static str {
        match self {
            Self::Go => "main.go",
            Self::Rust => "main.rs",
        }
    }

    /// The toolchain image.
    #[must_use]
    pub fn image(self, images: &Images) -> String {
        match self {
            Self::Go => images.go.clone(),
            Self::Rust => images.rust.clone(),
        }
    }

    /// The compiler's command, run in the scratch space; it writes `prog`.
    #[must_use]
    pub fn build(self) -> Vec<&'static str> {
        match self {
            Self::Go => vec!["go", "build", "-o", "prog", "main.go"],
            Self::Rust => vec![
                "rustc",
                "--edition",
                "2024",
                "-C",
                "opt-level=2",
                "-o",
                "prog",
                "main.rs",
            ],
        }
    }

    /// The environment every step of this language sees.
    #[must_use]
    pub fn env(self) -> Vec<(&'static str, &'static str)> {
        let mut env = vec![("HOME", WORK), ("TMPDIR", "/work/tmp")];
        if self == Self::Go {
            env.extend([
                ("CGO_ENABLED", "0"),
                ("GOTOOLCHAIN", "local"),
                ("GOPROXY", "off"),
                ("GOFLAGS", "-trimpath"),
                ("GOENV", "off"),
                ("GOPATH", "/work/go"),
                // Read-only, built into the image: the standard library once,
                // so a build is a second, not twenty-four.
                ("GOCACHE", "/opt/gocache"),
            ]);
        }
        env
    }
}

/// `docker run` arguments for a sandbox, after `run`: every restriction, then
/// the image and its idle process. The scratch space is the only writable
/// place; the network is none (and the runtime is configured to refuse one).
#[must_use]
pub fn container_args(
    language: Language,
    name: &str,
    limits: &Limits,
    images: &Images,
) -> Vec<String> {
    let mut args: Vec<String> = [
        "--detach",
        "--rm",
        "--name",
        name,
        "--label",
        LABEL,
        "--runtime",
        images.runtime.as_str(),
        "--network",
        "none",
        "--ipc",
        "none",
        "--hostname",
        "sandbox",
        "--read-only",
        "--user",
        "65534:65534",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--log-driver",
        "none",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    args.extend([
        "--tmpfs".to_owned(),
        format!(
            "{WORK}:rw,exec,nosuid,nodev,size={},mode=1777",
            limits.scratch
        ),
        "--tmpfs".to_owned(),
        "/tmp:rw,noexec,nosuid,nodev,size=16m,mode=1777".to_owned(),
        "--memory".to_owned(),
        limits.memory.clone(),
        "--memory-swap".to_owned(),
        limits.memory.clone(),
        "--cpus".to_owned(),
        limits.cpus.clone(),
        "--pids-limit".to_owned(),
        limits.pids.to_string(),
        "--ulimit".to_owned(),
        format!("fsize={0}:{0}", limits.max_file_bytes),
        "--ulimit".to_owned(),
        "nofile=256:256".to_owned(),
        "--ulimit".to_owned(),
        "core=0:0".to_owned(),
        "--workdir".to_owned(),
        WORK.to_owned(),
    ]);
    for (k, v) in language.env() {
        args.extend(["--env".to_owned(), format!("{k}={v}")]);
    }
    args.extend([
        language.image(images),
        "sleep".to_owned(),
        LIFETIME_SECS.to_string(),
    ]);
    args
}

/// `docker exec` arguments that write the source (read from standard input)
/// into the scratch space. The shell line is fixed; the source never is part
/// of it.
#[must_use]
pub fn write_source_args(language: Language, name: &str) -> Vec<String> {
    vec![
        "exec".to_owned(),
        "--interactive".to_owned(),
        name.to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        format!("mkdir -p /work/tmp && cat > {WORK}/{}", language.file()),
    ]
}

/// `docker exec` arguments for the compiler.
#[must_use]
pub fn build_args(language: Language, name: &str) -> Vec<String> {
    let mut args = vec!["exec".to_owned(), name.to_owned()];
    args.extend(language.build().into_iter().map(str::to_owned));
    args
}

/// `docker exec` arguments for the program, reading the input on standard input.
#[must_use]
pub fn run_args(name: &str) -> Vec<String> {
    vec![
        "exec".to_owned(),
        "--interactive".to_owned(),
        name.to_owned(),
        format!("{WORK}/prog"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            max_source_bytes: 65536,
            max_stdin_bytes: 65536,
            max_output_bytes: 65536,
            compile_timeout: std::time::Duration::from_secs(20),
            run_timeout: std::time::Duration::from_secs(5),
            max_concurrent: 4,
            memory: "512m".into(),
            cpus: "1".into(),
            pids: 128,
            scratch: "256m".into(),
            max_file_bytes: 67_108_864,
        }
    }

    fn images() -> Images {
        Images {
            go: "tbd-sandbox-go:dev".into(),
            rust: "tbd-sandbox-rust:dev".into(),
            runtime: "runsc".into(),
        }
    }

    fn has_pair(args: &[String], flag: &str, value: &str) -> bool {
        args.windows(2).any(|w| w[0] == flag && w[1] == value)
    }

    #[test]
    fn every_restriction_is_in_the_container_args() {
        for lang in Language::ALL {
            let a = container_args(lang, "tbd-sandbox-x", &limits(), &images());
            for (flag, value) in [
                ("--runtime", "runsc"),
                ("--network", "none"),
                ("--ipc", "none"),
                ("--user", "65534:65534"),
                ("--cap-drop", "ALL"),
                ("--security-opt", "no-new-privileges"),
                ("--log-driver", "none"),
                ("--memory", "512m"),
                ("--memory-swap", "512m"),
                ("--cpus", "1"),
                ("--pids-limit", "128"),
            ] {
                assert!(
                    has_pair(&a, flag, value),
                    "{lang:?}: {flag} {value} missing from {a:?}"
                );
            }
            for flag in ["--read-only", "--rm", "--detach"] {
                assert!(a.iter().any(|x| x == flag), "{lang:?}: {flag}");
            }
            assert!(
                !a.iter()
                    .any(|x| x.contains("--privileged") || x.contains("--volume") || x == "-v")
            );
            assert!(
                !a.iter()
                    .any(|x| x.starts_with("--cap-add") || x.starts_with("--device"))
            );
        }
    }

    #[test]
    fn the_source_is_never_part_of_a_command() {
        // The commands are built from the language and the container's name
        // alone; there is no parameter a request's text could reach.
        let w = write_source_args(Language::Go, "tbd-sandbox-x");
        assert_eq!(w[w.len() - 1], "mkdir -p /work/tmp && cat > /work/main.go");
        assert_eq!(build_args(Language::Rust, "n")[2], "rustc");
        assert_eq!(run_args("n"), ["exec", "--interactive", "n", "/work/prog"]);
    }

    #[test]
    fn go_builds_offline_without_c() {
        let env = Language::Go.env();
        for pair in [
            ("CGO_ENABLED", "0"),
            ("GOPROXY", "off"),
            ("GOTOOLCHAIN", "local"),
        ] {
            assert!(env.contains(&pair), "{pair:?}");
        }
    }
}
