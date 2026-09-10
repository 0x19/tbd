//! Compiles every `.proto` under `/proto` with `protox` (pure Rust, no `protoc`)
//! and generates tonic clients and servers plus a file descriptor set for
//! reflection.

use std::{env, fs, path::PathBuf};

use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read at run time, not `env!`: the self-check builds a copy of the tree into
    // the shared target dir, and a baked-in path would point at the deleted copy.
    let proto_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../proto");
    let files = [
        "tbd/engine/v1/engine.proto",
        "tbd/protocol/v1/protocol.proto",
        "tbd/ledger/v1/ledger.proto",
    ];
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let fds = protox::compile(files, [proto_root.as_path()])?;

    // `compile_fds` does not write descriptor sets itself; reflection needs them.
    // One set per package so each service advertises only what it serves. The
    // package name is the second path segment: `tbd/<name>/v1/<name>.proto`.
    for file in files {
        let name = file
            .split('/')
            .nth(1)
            .ok_or_else(|| format!("{file}: expected tbd/<name>/v1/<name>.proto"))?;
        let set = protox::compile([file], [proto_root.as_path()])?;
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
