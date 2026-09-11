//! Generated protobuf and gRPC code. Regenerated on every build from `/proto`.
//!
//! Do not add logic here; wrap generated types in the crate that owns the
//! behaviour instead.

#![allow(missing_docs, clippy::pedantic, clippy::all, unused_qualifications)]

/// Every compiled file, imports included (`google/api/http.proto`,
/// `google/api/annotations.proto`, `google/protobuf/*`), as one encoded
/// `FileDescriptorSet`. The protocol's transcoder reads the `google.api.http`
/// method options from it; the per-package sets below serve reflection.
pub const DESCRIPTOR_SET_ALL: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/all_descriptor.bin"));

pub mod engine {
    pub mod v1 {
        tonic::include_proto!("tbd.engine.v1");
        /// Encoded `FileDescriptorSet` for this package, for gRPC reflection.
        pub const DESCRIPTOR_SET: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/engine_descriptor.bin"));
    }
}

pub mod protocol {
    pub mod v1 {
        tonic::include_proto!("tbd.protocol.v1");
        /// Encoded `FileDescriptorSet` for this package, for gRPC reflection.
        pub const DESCRIPTOR_SET: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/protocol_descriptor.bin"));
    }
}

pub mod ledger {
    pub mod v1 {
        tonic::include_proto!("tbd.ledger.v1");
        /// Encoded `FileDescriptorSet` for this package, for gRPC reflection.
        pub const DESCRIPTOR_SET: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/ledger_descriptor.bin"));
    }
}

pub mod humans {
    pub mod v1 {
        tonic::include_proto!("tbd.humans.v1");
        /// Encoded `FileDescriptorSet` for this package, for gRPC reflection.
        pub const DESCRIPTOR_SET: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/humans_descriptor.bin"));
    }
}
