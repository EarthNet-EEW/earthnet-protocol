//! Regression lock for the committed cross-language vectors.
//!
//! Rebuilds the fixed-key messages used by `examples/gen_vectors.rs` and asserts
//! their canonical encoding + signature match the locked constants. Any change to
//! the wire format or signing scheme breaks this test, forcing an intentional
//! regeneration (`cargo run --example gen_vectors`) and a `protocol_version` bump.

use earthnet_protocol::{
    sign, verify, ConfirmedEvent, EvidenceKind, Location, Observation, SourceType,
    PROTOCOL_VERSION,
};
use ed25519_dalek::SigningKey;
use prost::Message;

const OBS_CANONICAL: &str = "08011210abababababababababababababababab1a20ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c20023080808c9eede2cfee173814420a0a0536366a643210e0124d0000f04050015da69b443c";
const OBS_SIG: &str = "b43747d690bb7c40c630e859feb80dc99db835b41521d6fb17ae68edbfd5c874c47af94163eff1e37e2ef5f95059526cda7bdb5d6b0aa4a128632eca9602aa09";
const EVT_CANONICAL: &str = "08011210cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd1a2066be7e332c7a453332bd9d0a7f7db055f5c5ef1a06ada66d98b39fb6810c473a2080808c9eede2cfee172880c692adeee2cfee17320b0a0636366a64326b10d8043d00000c42456666c6404d9a99993e500158036210abababababababababababababababab621001010101010101010101010101010101";
const EVT_SIG: &str = "46e81eaa7ac8d589d2159523f53c41e5da5ea0aac9463fd44d924b18b0a1a7ad1e0716af3e681bbfd45282c32c5fdeb4bd2e74c17f3d900a1fed7587a0caef0c";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn canonical<T: earthnet_protocol::Signed>(msg: &T) -> String {
    let mut bare = msg.clone();
    bare.set_signature(Vec::new());
    hex(&bare.encode_to_vec())
}

fn observation() -> (SigningKey, Observation) {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let mut obs = Observation {
        protocol_version: PROTOCOL_VERSION,
        observation_id: vec![0xAB; 16],
        pubkey: key.verifying_key().to_bytes().to_vec(),
        source_type: SourceType::Phone as i32,
        source_id: String::new(),
        captured_at_ns: 1_719_600_000_000_000_000,
        clock_uncert_ms: 20,
        location: Some(Location {
            geohash: "66jd2".into(),
            precision_m: 2400,
        }),
        sta_lta_ratio: 7.5,
        p_wave_detected: true,
        estimated_pga: 0.012,
        reported_magnitude: 0.0,
        signature: Vec::new(),
    };
    sign(&key, &mut obs);
    (key, obs)
}

fn confirmed_event() -> (SigningKey, ConfirmedEvent) {
    let key = SigningKey::from_bytes(&[11u8; 32]);
    let mut evt = ConfirmedEvent {
        protocol_version: PROTOCOL_VERSION,
        event_id: vec![0xCD; 16],
        pubkey: key.verifying_key().to_bytes().to_vec(),
        origin_time_ns: 1_719_600_000_000_000_000,
        issued_at_ns: 1_719_600_000_300_000_000,
        epicenter: Some(Location {
            geohash: "66jd2k".into(),
            precision_m: 600,
        }),
        depth_km: 35.0,
        magnitude: 6.2,
        magnitude_uncert: 0.3,
        evidence: EvidenceKind::Official as i32,
        num_observations: 3,
        obs_ids: vec![vec![0xAB; 16], vec![0x01; 16]],
        supersedes: Vec::new(),
        signature: Vec::new(),
    };
    sign(&key, &mut evt);
    (key, evt)
}

#[test]
fn observation_vector_locked() {
    let (_k, obs) = observation();
    assert_eq!(canonical(&obs), OBS_CANONICAL, "Observation canonical encoding drifted");
    assert_eq!(hex(&obs.signature), OBS_SIG, "Observation signature drifted");
    assert!(verify(&obs).is_ok());
}

#[test]
fn confirmed_event_vector_locked() {
    let (_k, evt) = confirmed_event();
    assert_eq!(canonical(&evt), EVT_CANONICAL, "ConfirmedEvent canonical encoding drifted");
    assert_eq!(hex(&evt.signature), EVT_SIG, "ConfirmedEvent signature drifted");
    assert!(verify(&evt).is_ok());
}

#[test]
fn vectors_file_matches_locked_constants() {
    let json = std::fs::read_to_string("tests/vectors/v0_1.json")
        .expect("run `cargo run --example gen_vectors` first");
    for c in [OBS_CANONICAL, OBS_SIG, EVT_CANONICAL, EVT_SIG] {
        assert!(json.contains(c), "committed vectors file out of sync with locked constants");
    }
}
