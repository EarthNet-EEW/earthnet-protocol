//! earthnet-protocol — signed seismic event wire format for EarthNet.
//!
//! The spine of the network: a signed [`Observation`] (raw pick) and a signed
//! [`ConfirmedEvent`] (post-fusion, triggers the alarm). Both are Protobuf
//! messages signed with Ed25519 over a deterministic payload — see [`sign`].

/// Generated Protobuf types (package `earthnet.v1`).
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/earthnet.v1.rs"));
}

pub mod detect;
pub mod sign;

pub use proto::{ConfirmedEvent, EvidenceKind, Location, Observation, SourceType};
pub use sign::{sign, verify, SignError, Signed};

/// Wire protocol major version carried in every message's `protocol_version`.
pub const PROTOCOL_VERSION: u32 = 1;
