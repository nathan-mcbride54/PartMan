//! Tests for the redaction boundary (WP-040 increment 3).

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};

use super::redaction::{FIELD_RULES, FieldClass};
use super::stream::{RESUME_TOKEN_SCHEMA, RESUME_TOKEN_SCHEMA_VERSION, ResumeToken};
use super::{
    DecodeRefusal, ENVELOPE_SCHEMA, ENVELOPE_SCHEMA_VERSION, Envelope, HANDSHAKE_SCHEMA,
    HANDSHAKE_SCHEMA_VERSION, Handshake, PROTOCOL_VERSION, is_build_version,
};

/// One raw exemplar per SEC-006 identifier class. Each is the class as
/// it actually presents — not adversarially shaped to pass a grammar,
/// because the boundary's stated reach is raw values and the schema
/// doc names deliberate shaping as the peer's violation.
const EXEMPLARS: [(&str, &str); 6] = [
    ("device serial", "WD-WCC4N5PZ3RKE"),
    ("path", "/dev/disk/by-id/ata-WDC_WD40EFRX-68N32N0"),
    ("path", "C:\\Users\\nate\\backup.img"),
    ("label", "Backup Disk 2"),
    ("username", "nate"),
    ("key", "-----BEGIN OPENSSH PRIVATE KEY-----"),
];

/// A file name is SEC-006's sixth class; it shares the token alphabet
/// with versions, so it gets its own exemplar to prove the grammar's
/// leading `digits.digits.digits` still refuses it.
const FILE_NAME: &str = "backup.img";

fn small_body() -> Vec<u8> {
    canonical::encode(&Value::Text("body".into())).expect("encodable")
}

fn valid_envelope_map(channel: &str, sequence: Option<u64>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    map.insert("schema".to_owned(), Value::Text(ENVELOPE_SCHEMA.into()));
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(ENVELOPE_SCHEMA_VERSION),
    );
    map.insert("channel".to_owned(), Value::Text(channel.into()));
    map.insert("body".to_owned(), Value::Bytes(small_body()));
    if let Some(sequence) = sequence {
        map.insert("sequence".to_owned(), Value::Unsigned(sequence));
    }
    map
}

fn valid_handshake_map() -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    map.insert("schema".to_owned(), Value::Text(HANDSHAKE_SCHEMA.into()));
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(HANDSHAKE_SCHEMA_VERSION),
    );
    map.insert("protocol_version".to_owned(), Value::Unsigned(1));
    map.insert("build".to_owned(), Value::Text("0.1.0".into()));
    map
}

fn valid_token_map() -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    map.insert("schema".to_owned(), Value::Text(RESUME_TOKEN_SCHEMA.into()));
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(RESUME_TOKEN_SCHEMA_VERSION),
    );
    map.insert("execution".to_owned(), Value::Bytes(b"exec-1".to_vec()));
    map.insert("last_sequence".to_owned(), Value::Unsigned(1));
    map
}

fn encoded(map: BTreeMap<String, Value>) -> Vec<u8> {
    canonical::encode(&Value::Map(map)).expect("encodable")
}

// Requirements: SEC-006, RPC-003
//   The redaction gate: a raw exemplar of every SEC-006 identifier
//   class — serial, path, label, username, key, file name — planted in
//   every non-allowlisted position of every format this package owns
//   refuses, including as an unknown field's own key. The strict
//   validator is the mechanism, so the gate needs no knowledge of the
//   denied classes: a pinned constant, an unsigned number, a closed
//   tag, and the build-version grammar each refuse the plant on their
//   own terms.
// Evidence: raw_identifiers_refuse_in_every_non_allowlisted_position
#[test]
fn raw_identifiers_refuse_in_every_non_allowlisted_position() {
    let exemplars = EXEMPLARS
        .iter()
        .map(|&(_, raw)| raw)
        .chain([FILE_NAME])
        .collect::<Vec<_>>();
    for raw in exemplars {
        // Envelope: the pinned schema constant.
        let mut map = valid_envelope_map("request", None);
        map.insert("schema".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Envelope::decode(&encoded(map)),
            Err(DecodeRefusal::WrongSchema),
            "envelope schema must refuse the plant: {raw}"
        );

        // Envelope: the pinned version, mistyped to carry text at all.
        let mut map = valid_envelope_map("request", None);
        map.insert("schema_version".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Envelope::decode(&encoded(map)),
            Err(DecodeRefusal::WrongSchema),
            "envelope schema_version must refuse the plant: {raw}"
        );

        // Envelope: the closed channel vocabulary.
        let mut map = valid_envelope_map("request", None);
        map.insert("channel".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Envelope::decode(&encoded(map)),
            Err(DecodeRefusal::BadField { key: "channel" }),
            "envelope channel must refuse the plant: {raw}"
        );

        // Envelope: the event sequence, an unsigned number.
        let mut map = valid_envelope_map("event", None);
        map.insert("sequence".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Envelope::decode(&encoded(map)),
            Err(DecodeRefusal::BadField { key: "sequence" }),
            "envelope sequence must refuse the plant: {raw}"
        );

        // Envelope: there is no position to invent — an identifier as
        // an unknown field's own key refuses by name.
        let mut map = valid_envelope_map("request", None);
        map.insert(raw.to_owned(), Value::Bool(true));
        assert_eq!(
            Envelope::decode(&encoded(map)),
            Err(DecodeRefusal::UnknownField { key: raw.into() }),
            "an invented envelope field must refuse: {raw}"
        );

        // Handshake: the pinned schema constant.
        let mut map = valid_handshake_map();
        map.insert("schema".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Handshake::decode(&encoded(map)),
            Err(DecodeRefusal::WrongSchema),
            "handshake schema must refuse the plant: {raw}"
        );

        // Handshake: the protocol version, an unsigned number.
        let mut map = valid_handshake_map();
        map.insert("protocol_version".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Handshake::decode(&encoded(map)),
            Err(DecodeRefusal::BadField {
                key: "protocol_version"
            }),
            "handshake protocol_version must refuse the plant: {raw}"
        );

        // Handshake: the build-version grammar, decode direction.
        let mut map = valid_handshake_map();
        map.insert("build".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            Handshake::decode(&encoded(map)),
            Err(DecodeRefusal::NotABuildVersion),
            "handshake build must refuse the plant: {raw}"
        );

        // Handshake: the same grammar binds the encode direction —
        // this side cannot emit what the peer's decode would refuse.
        let outbound = Handshake {
            protocol_version: PROTOCOL_VERSION,
            build: raw.into(),
        };
        assert_eq!(
            outbound.encode(),
            Err(DecodeRefusal::NotABuildVersion),
            "handshake encode must refuse the plant: {raw}"
        );

        // Resume token: the pinned schema constant.
        let mut map = valid_token_map();
        map.insert("schema".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            ResumeToken::decode(&encoded(map)),
            Err(DecodeRefusal::WrongSchema),
            "token schema must refuse the plant: {raw}"
        );

        // Resume token: the last sequence, an unsigned number.
        let mut map = valid_token_map();
        map.insert("last_sequence".to_owned(), Value::Text(raw.into()));
        assert_eq!(
            ResumeToken::decode(&encoded(map)),
            Err(DecodeRefusal::BadField {
                key: "last_sequence"
            }),
            "token last_sequence must refuse the plant: {raw}"
        );
    }
}

// Requirements: SEC-006
//   The allowlist is exactly two authored positions, pinned as
//   literals so widening it is a visible reviewed edit: the envelope
//   body, whose bytes are governed by the `schemas/`-defined type they
//   encode, and the resume token's execution handle, whose opacity is
//   WP-070's minting obligation. The boundary table classifies every
//   field of every format this package owns — its per-format field
//   sets equal the wire's actual key sets — and identifier-class bytes
//   demonstrably cross at the two allowlisted positions and nowhere
//   else.
// Evidence: the_allowlist_is_exactly_the_two_authored_positions
#[test]
fn the_allowlist_is_exactly_the_two_authored_positions() {
    // The table covers every format, every field, as literals.
    let fields = |format: &str| {
        FIELD_RULES
            .iter()
            .filter(|rule| rule.format == format)
            .map(|rule| rule.field)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        fields(ENVELOPE_SCHEMA),
        ["schema", "schema_version", "channel", "body", "sequence"]
    );
    assert_eq!(
        fields(HANDSHAKE_SCHEMA),
        ["schema", "schema_version", "protocol_version", "build"]
    );
    assert_eq!(
        fields(RESUME_TOKEN_SCHEMA),
        ["schema", "schema_version", "execution", "last_sequence"]
    );
    assert_eq!(FIELD_RULES.len(), 5 + 4 + 4, "no format escapes the table");

    // The allowlist, as literals, with an authority on every entry.
    let allowlisted = FIELD_RULES
        .iter()
        .filter(|rule| rule.class == FieldClass::IdentifierCapable)
        .map(|rule| (rule.format, rule.field))
        .collect::<Vec<_>>();
    assert_eq!(
        allowlisted,
        [
            (ENVELOPE_SCHEMA, "body"),
            (RESUME_TOKEN_SCHEMA, "execution")
        ]
    );
    for rule in &FIELD_RULES {
        assert_eq!(
            rule.class == FieldClass::IdentifierCapable,
            !rule.authority.is_empty(),
            "exactly the allowlisted positions name a governing authority: {}.{}",
            rule.format,
            rule.field
        );
    }

    // The table matches the wire: each format's encoded key set equals
    // its table rows (the envelope's sequence present exactly on the
    // event channel, per its presence rule).
    let keys = |bytes: &[u8]| match canonical::decode(bytes).expect("canonical") {
        Value::Map(map) => map.keys().cloned().collect::<Vec<_>>(),
        _ => panic!("a message is a map"),
    };
    let event = Envelope::event(1, small_body()).expect("wraps");
    assert_eq!(
        keys(&event.encode().expect("encodes")),
        ["body", "channel", "schema", "schema_version", "sequence"]
    );
    let request = Envelope::request(small_body()).expect("wraps");
    assert_eq!(
        keys(&request.encode().expect("encodes")),
        ["body", "channel", "schema", "schema_version"]
    );
    let handshake = Handshake::local("0.1.0");
    assert_eq!(
        keys(&handshake.encode().expect("encodes")),
        ["build", "protocol_version", "schema", "schema_version"]
    );
    let token = ResumeToken {
        execution: b"exec-1".to_vec(),
        last_sequence: 1,
    };
    assert_eq!(
        keys(&token.encode().expect("encodes")),
        ["execution", "last_sequence", "schema", "schema_version"]
    );

    // Identifier-class bytes cross at the two allowlisted positions —
    // governed there by the named authorities, not silently admitted.
    for (class, raw) in EXEMPLARS {
        let body = canonical::encode(&Value::Text(raw.into())).expect("encodable");
        let envelope = Envelope::request(body).expect("the body position admits the class");
        let bytes = envelope.encode().expect("encodes");
        assert_eq!(
            Envelope::decode(&bytes).expect("decodes"),
            envelope,
            "the body must carry a {class} for its schemas to govern"
        );

        let token = ResumeToken {
            execution: raw.as_bytes().to_vec(),
            last_sequence: 1,
        };
        let bytes = token.encode().expect("encodes");
        assert_eq!(
            ResumeToken::decode(&bytes).expect("decodes"),
            token,
            "the execution handle is opaque here; its opacity is WP-070's obligation"
        );
    }
}

// Requirements: SEC-006, RPC-002, MODEL-003
//   The build field is held to RPC-002's own word for it — a version:
//   digits '.' digits '.' digits with an optional +/- suffix over the
//   token alphabet, ASCII, bounded, in both directions. The refusal
//   names the rule and never echoes the value. The handshake schema
//   moves to version 2 for the constraint — a reviewed bump taken
//   while no consumer exists — so a v1-stamped handshake refuses
//   rather than being read under rules it was not written to.
// Evidence: the_build_version_grammar_holds_both_directions
#[test]
fn the_build_version_grammar_holds_both_directions() {
    for accepted in ["0.1.0", "10.20.30", "1.2.3+g436b49f", "1.2.3-rc.1"] {
        assert!(is_build_version(accepted), "must admit: {accepted}");
        let handshake = Handshake::local(accepted);
        let bytes = handshake.encode().expect("encodes");
        assert_eq!(Handshake::decode(&bytes).expect("decodes"), handshake);
    }

    let oversize = format!("1.0.0-{}", "x".repeat(59));
    let at_bound = format!("1.0.0-{}", "x".repeat(58));
    assert!(is_build_version(&at_bound), "the bound itself is admitted");
    for refused in [
        "",
        "1.2",
        "1.2.3.4",
        "v1.2.3",
        "1.2.3-",
        "1.2.3 ",
        "1.2.3-caf\u{e9}",
        oversize.as_str(),
    ] {
        assert!(!is_build_version(refused), "must refuse: {refused:?}");
    }

    // The refusal carries no payload: quoting the bytes would itself
    // carry what the boundary exists to keep out.
    assert_eq!(
        Handshake {
            protocol_version: PROTOCOL_VERSION,
            build: "Backup Disk 2".into(),
        }
        .encode(),
        Err(DecodeRefusal::NotABuildVersion)
    );

    // The reviewed bump: a v1-stamped handshake refuses.
    let mut stale = valid_handshake_map();
    stale.insert("schema_version".to_owned(), Value::Unsigned(1));
    assert_eq!(
        Handshake::decode(&encoded(stale)),
        Err(DecodeRefusal::WrongSchema)
    );
}
