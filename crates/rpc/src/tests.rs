//! Tests for the RPC message layer (WP-040 increment 1).

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};

use super::{
    Channel, DecodeRefusal, ENVELOPE_SCHEMA, Envelope, HANDSHAKE_SCHEMA, Handshake,
    MAX_MESSAGE_BYTES, PROTOCOL_VERSION,
};

fn small_body() -> Vec<u8> {
    canonical::encode(&Value::Text("body".into())).expect("encodable")
}

// Requirements: RPC-003, RPC-005, MODEL-003
//   The envelope round-trips through one strict validator: exact
//   schema, no unknown fields, every declared field well-typed, and
//   the body re-proved canonical — an envelope cannot launder bytes
//   the codec would refuse.
// Evidence: the_envelope_round_trips_strictly
#[test]
fn the_envelope_round_trips_strictly() {
    let envelope = Envelope::new(Channel::Request, small_body()).expect("wraps");
    let bytes = envelope.encode().expect("encodes");
    let decoded = Envelope::decode(&bytes).expect("decodes");
    assert_eq!(decoded, envelope);
    assert_eq!(decoded.channel(), Channel::Request);

    let non_canonical = Envelope::new(Channel::Event, vec![0xff, 0xff]);
    assert_eq!(non_canonical, Err(DecodeRefusal::BodyNotCanonical));
}

// Requirements: RPC-003
//   The strict arm rejects, both directions: an unknown field, a wrong
//   schema, and a mistyped channel each refuse by name — never a
//   silent skip, never a default.
// Evidence: unknown_and_mistyped_fields_refuse_by_name
#[test]
fn unknown_and_mistyped_fields_refuse_by_name() {
    let mut map = BTreeMap::new();
    map.insert("schema".to_owned(), Value::Text(ENVELOPE_SCHEMA.into()));
    map.insert("schema_version".to_owned(), Value::Unsigned(1));
    map.insert("channel".to_owned(), Value::Text("request".into()));
    map.insert("body".to_owned(), Value::Bytes(small_body()));
    map.insert("smuggled".to_owned(), Value::Bool(true));
    let bytes = canonical::encode(&Value::Map(map)).expect("encodable");
    assert_eq!(
        Envelope::decode(&bytes),
        Err(DecodeRefusal::UnknownField {
            key: "smuggled".into()
        })
    );

    let mut wrong = BTreeMap::new();
    wrong.insert("schema".to_owned(), Value::Text("partman.other".into()));
    wrong.insert("schema_version".to_owned(), Value::Unsigned(1));
    wrong.insert("channel".to_owned(), Value::Text("request".into()));
    wrong.insert("body".to_owned(), Value::Bytes(small_body()));
    let bytes = canonical::encode(&Value::Map(wrong)).expect("encodable");
    assert_eq!(Envelope::decode(&bytes), Err(DecodeRefusal::WrongSchema));

    let mut mistyped = BTreeMap::new();
    mistyped.insert("schema".to_owned(), Value::Text(ENVELOPE_SCHEMA.into()));
    mistyped.insert("schema_version".to_owned(), Value::Unsigned(1));
    mistyped.insert("channel".to_owned(), Value::Text("broadcast".into()));
    mistyped.insert("body".to_owned(), Value::Bytes(small_body()));
    let bytes = canonical::encode(&Value::Map(mistyped)).expect("encodable");
    assert_eq!(
        Envelope::decode(&bytes),
        Err(DecodeRefusal::BadField { key: "channel" })
    );
}

// Requirements: RPC-004
//   The size bound binds the wire before anything parses: an oversized
//   input refuses with both numbers named, at construction, at encode,
//   and at decode.
// Evidence: the_size_bound_binds_the_wire
#[test]
fn the_size_bound_binds_the_wire() {
    let oversized = vec![0_u8; MAX_MESSAGE_BYTES + 1];
    let refused = Envelope::decode(&oversized).expect_err("must refuse before parsing");
    assert_eq!(
        refused,
        DecodeRefusal::OversizedMessage {
            presented: MAX_MESSAGE_BYTES + 1,
            bound: MAX_MESSAGE_BYTES,
        }
    );

    let refused = Envelope::new(Channel::Request, oversized).expect_err("must refuse");
    assert!(matches!(refused, DecodeRefusal::OversizedMessage { .. }));
}

// Requirements: RPC-002, MODEL-003
//   The handshake round-trips strictly, and the compatibility rule is
//   total: equal versions are compatible; unequal versions refuse with
//   a remediation naming the older side — there is no downgrade arm.
// Evidence: the_handshake_refuses_and_never_degrades
#[test]
fn the_handshake_refuses_and_never_degrades() {
    let local = Handshake::local("build-a");
    let bytes = local.encode().expect("encodes");
    let decoded = Handshake::decode(&bytes).expect("decodes");
    assert_eq!(decoded, local);
    assert_eq!(decoded.protocol_version, PROTOCOL_VERSION);

    let peer = Handshake {
        protocol_version: PROTOCOL_VERSION,
        build: "build-b".into(),
    };
    local.compatible_with(&peer).expect("equal versions agree");

    let older = Handshake {
        protocol_version: PROTOCOL_VERSION + 1,
        build: "build-future".into(),
    };
    let refusal = local
        .compatible_with(&older)
        .expect_err("unequal versions refuse");
    assert_eq!(refusal.local, PROTOCOL_VERSION);
    assert_eq!(refusal.remote, PROTOCOL_VERSION + 1);
    assert!(
        refusal.remediation.contains("this side"),
        "the remediation names the older side: {}",
        refusal.remediation
    );

    let mut smuggled = BTreeMap::new();
    smuggled.insert("schema".to_owned(), Value::Text(HANDSHAKE_SCHEMA.into()));
    smuggled.insert("schema_version".to_owned(), Value::Unsigned(1));
    smuggled.insert("protocol_version".to_owned(), Value::Unsigned(1));
    smuggled.insert("build".to_owned(), Value::Text("b".into()));
    smuggled.insert("downgrade_ok".to_owned(), Value::Bool(true));
    let bytes = canonical::encode(&Value::Map(smuggled)).expect("encodable");
    assert!(matches!(
        Handshake::decode(&bytes),
        Err(DecodeRefusal::UnknownField { .. })
    ));
}
