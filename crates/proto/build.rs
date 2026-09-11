//! Compiles every `.proto` under `/proto` with `protox` (pure Rust, no `protoc`)
//! and generates tonic clients and servers plus a file descriptor set for
//! reflection.
//!
//! The files are discovered at run time, and the proto root is read from the
//! environment rather than baked in with `env!`: `mise run tbd:selfcheck` builds
//! a copy of the tree into the shared target directory, and Cargo gives a path
//! package the same build-script binary wherever it lives. A script that knew
//! its file list or its path at compile time would then serve the wrong tree.

use std::{env, fs, path::PathBuf};

use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../proto");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // `tbd/<name>/v1/<name>.proto`, sorted so the generated code is stable.
    let mut files: Vec<String> = Vec::new();
    for pkg in fs::read_dir(proto_root.join("tbd"))? {
        let pkg = pkg?;
        if !pkg.file_type()?.is_dir() {
            continue;
        }
        let name = pkg.file_name().to_string_lossy().into_owned();
        for entry in fs::read_dir(pkg.path().join("v1"))? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "proto") {
                let file = entry.file_name().to_string_lossy().into_owned();
                files.push(format!("tbd/{name}/v1/{file}"));
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no .proto files under {}", proto_root.display()).into());
    }

    let fds = protox::compile(&files, [proto_root.as_path()])?;

    // The whole contract in one set, imports included (`google/api/*`,
    // `google/protobuf/*`): the input of the protocol's descriptor-driven
    // transcoder, which reads `google.api.http` method options at start.
    fs::write(out_dir.join("all_descriptor.bin"), fds.encode_to_vec())?;

    // `compile_fds` does not write descriptor sets itself; reflection needs them.
    // One set per package so each service advertises only what it serves. The
    // package name is the second path segment: `tbd/<name>/v1/<name>.proto`.
    for file in &files {
        let name = file
            .split('/')
            .nth(1)
            .ok_or_else(|| format!("{file}: expected tbd/<name>/v1/<name>.proto"))?;
        let set = protox::compile([file.as_str()], [proto_root.as_path()])?;
        fs::write(
            out_dir.join(format!("{name}_descriptor.bin")),
            set.encode_to_vec(),
        )?;
    }

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .emit_rerun_if_changed(false)
        .compile_fds(fds)?;

    println!("cargo:rerun-if-changed={}", proto_root.display());
    Ok(())
}
