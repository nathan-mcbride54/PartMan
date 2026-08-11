//! Tests for the streams and reattach vocabulary (WP-040 increment 2).

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};

use super::stream::{Arrival, EventSequencer, RESUME_TOKEN_SCHEMA, ResumeToken, classify};
use super::{Channel, DecodeRefusal, Envelope};

fn small_body() -> Vec<u8> {
    canonical::encode(&Value::Text("event".into())).expect("encodable")
}

// Requirements: RPC-004, RPC-006
//   The producer sequence is monotone and gap-free from 1, and the
//   consumer classification is total: in order processes, already-seen
//   discards (replay after reattach is expected and harmless), and a
//   gap names the missing closed range whose recovery is the journal.
// Evidence: sequencing_is_monotone_and_classification_is_total
#[test]
fn sequencing_is_monotone_and_classification_is_total() {
    let mut sequencer = EventSequencer::new();
    assert_eq!(sequencer.last(), 0);
    assert_eq!(sequencer.next(), 1);
    assert_eq!(sequencer.next(), 2);
    assert_eq!(sequencer.next(), 3);
    assert_eq!(sequencer.last(), 3);

    assert_eq!(classify(0, 1), Arrival::InOrder);
    assert_eq!(classify(3, 4), Arrival::InOrder);
    assert_eq!(classify(3, 3), Arrival::AlreadySeen);
    assert_eq!(classify(3, 1), Arrival::AlreadySeen);
    assert_eq!(classify(3, 7), Arrival::Gap { from: 4, to: 6 });
    assert_eq!(classify(0, 5), Arrival::Gap { from: 1, to: 4 });
}

// Requirements: RPC-004, RPC-003
//   The envelope's per-channel presence rules hold strictly both ways:
//   an event carries exactly one sequence number, a request or response
//   carries none, and a violation refuses naming the channel and the
//   presence it found.
// Evidence: sequence_presence_follows_the_channel
#[test]
fn sequence_presence_follows_the_channel() {
    let event = Envelope::event(7, small_body()).expect("wraps");
    let bytes = event.encode().expect("encodes");
    let decoded = Envelope::decode(&bytes).expect("decodes");
    assert_eq!(decoded.sequence(), Some(7));
    assert_eq!(decoded.channel(), Channel::Event);

    let request = Envelope::request(small_body()).expect("wraps");
    assert_eq!(request.sequence(), None);

    // A hand-built event without a sequence refuses.
    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(super::ENVELOPE_SCHEMA.into()),
    );
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(super::ENVELOPE_SCHEMA_VERSION),
    );
    map.insert("channel".to_owned(), Value::Text("event".into()));
    map.insert("body".to_owned(), Value::Bytes(small_body()));
    let bytes = canonical::encode(&Value::Map(map.clone())).expect("encodable");
    assert_eq!(
        Envelope::decode(&bytes),
        Err(DecodeRefusal::SequenceMisplaced {
            channel: Channel::Event,
            present: false,
        })
    );

    // A hand-built request with a sequence refuses too.
    map.insert("channel".to_owned(), Value::Text("request".into()));
    map.insert("sequence".to_owned(), Value::Unsigned(1));
    let bytes = canonical::encode(&Value::Map(map)).expect("encodable");
    assert_eq!(
        Envelope::decode(&bytes),
        Err(DecodeRefusal::SequenceMisplaced {
            channel: Channel::Request,
            present: true,
        })
    );
}

// Requirements: RPC-006, MODEL-003
//   The resume token round-trips strictly — the reattach anchor names
//   the execution and the last processed sequence, and a smuggled
//   field refuses by name, because the token is the protocol's
//   statement of where to anchor and nothing more.
// Evidence: the_resume_token_round_trips_strictly
#[test]
fn the_resume_token_round_trips_strictly() {
    let token = ResumeToken {
        execution: b"exec-1".to_vec(),
        last_sequence: 41,
    };
    let bytes = token.encode().expect("encodes");
    let decoded = ResumeToken::decode(&bytes).expect("decodes");
    assert_eq!(decoded, token);

    let mut smuggled = BTreeMap::new();
    smuggled.insert("schema".to_owned(), Value::Text(RESUME_TOKEN_SCHEMA.into()));
    smuggled.insert("schema_version".to_owned(), Value::Unsigned(1));
    smuggled.insert("execution".to_owned(), Value::Bytes(b"exec-1".to_vec()));
    smuggled.insert("last_sequence".to_owned(), Value::Unsigned(41));
    smuggled.insert("skip_journal".to_owned(), Value::Bool(true));
    let bytes = canonical::encode(&Value::Map(smuggled)).expect("encodable");
    assert!(matches!(
        ResumeToken::decode(&bytes),
        Err(DecodeRefusal::UnknownField { .. })
    ));
}
