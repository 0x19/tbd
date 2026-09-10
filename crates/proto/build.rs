//! Compiles every `.proto` under `/proto` with `protox` (pure Rust, no `protoc`)
//! and generates tonic clients and servers plus a file descriptor set for
//! reflection.

use std::{env, fs, path::PathBuf};

use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proto");
    let files = [
        "tbd/engine/v1/engine.proto",
        "tbd/protocol/v1/protocol.proto",
    ];
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let fds = protox::compile(files, [proto_root.as_path()])?;

    // `compile_fds` does not write descriptor sets itself; reflection needs them.
    // One set per package so each service advertises only what it serves.
    for (name, file) in [("engine", files[0]), ("protocol", files[1])] {
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
