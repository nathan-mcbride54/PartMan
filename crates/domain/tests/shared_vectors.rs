//! Rust half of the MODEL-005 cross-language parity proof.
//!
//! This reads `schemas/canonical-encoding-vectors.json`, the same file the
//! TypeScript suite in `packages/canonical` reads. Neither language keeps its
//! own copy on purpose: an implementation checked against a table it also owns
//! proves only self-consistency, and two tables can drift silently.

use std::collections::BTreeMap;
use std::path::PathBuf;

use partman_domain::canonical::{Value, encode, hash};
use serde_json::Value as Json;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/canonical-encoding-vectors.json")
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

fn from_hex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "hex needs an even length");
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).expect("valid hex"))
        .collect()
}

/// Build a value from the fixture's representation.
///
/// Integers arrive as decimal strings. The fixture for a profile that exists
/// because JSON numbers cannot carry `u64` must not itself rely on JSON
/// numbers, and a plain `1b0020000000000000` vector would be corrupted by any
/// JSON parser that rounded it.
fn build(json: &Json) -> Value {
    let object = json.as_object().expect("value is an object");
    let (tag, payload) = object.iter().next().expect("value has one tag");

    match tag.as_str() {
        "uint" => Value::Unsigned(
            payload
                .as_str()
                .expect("uint is a decimal string")
                .parse()
                .expect("uint parses as u64"),
        ),
        "neg" => Value::Negative(
            payload
                .as_str()
                .expect("neg is a decimal string")
                .parse()
                .expect("neg parses as i64"),
        ),
        "bytes" => Value::Bytes(from_hex(payload.as_str().expect("bytes is hex"))),
        "text" => Value::Text(payload.as_str().expect("text is a string").to_owned()),
        "array" => Value::Array(
            payload
                .as_array()
                .expect("array is an array")
                .iter()
                .map(build)
                .collect(),
        ),
        "map" => {
            let mut entries = BTreeMap::new();
            for pair in payload.as_array().expect("map is an array of pairs") {
                let pair = pair.as_array().expect("entry is a pair");
                let key = pair[0].as_str().expect("key is a string").to_owned();
                entries.insert(key, build(&pair[1]));
            }
            Value::Map(entries)
        }
        "bool" => Value::Bool(payload.as_bool().expect("bool is a boolean")),
        "null" => Value::Null,
        other => panic!("unrecognized value representation {other:?}"),
    }
}

struct Fixture {
    name: String,
    value: Value,
    canonical: String,
    sha256: String,
}

fn load() -> Vec<Fixture> {
    let path = fixture_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
    let document: Json = serde_json::from_str(&raw).expect("fixture is valid JSON");

    assert_eq!(
        document["profile"].as_str(),
        Some("pce/1"),
        "fixture declares a different profile"
    );

    document["vectors"]
        .as_array()
        .expect("fixture has a vectors array")
        .iter()
        .map(|entry| Fixture {
            name: entry["name"].as_str().expect("name").to_owned(),
            value: build(&entry["value"]),
            canonical: entry["canonical"].as_str().expect("canonical").to_owned(),
            sha256: entry["sha256"].as_str().expect("sha256").to_owned(),
        })
        .collect()
}

#[test]
fn the_shared_fixture_is_populated() {
    // A fixture that silently became empty would make every other test in this
    // file pass vacuously.
    assert!(load().len() >= 30, "the shared fixture looks truncated");
}

// Requirements: MODEL-005
//   Rust consumes the repository's shared cross-language fixture and reproduces every exact canonical byte string
// Evidence: every_shared_vector_encodes_to_exactly_the_recorded_bytes
#[test]
fn every_shared_vector_encodes_to_exactly_the_recorded_bytes() {
    for fixture in load() {
        let actual = encode(&fixture.value).expect("fixture values are encodable");
        assert_eq!(hex(&actual), fixture.canonical, "{}", fixture.name);
    }
}

// Requirements: MODEL-005
//   Rust reproduces every SHA-256 digest recorded in the fixture TypeScript also consumes
// Evidence: every_shared_vector_hashes_to_exactly_the_recorded_digest
#[test]
fn every_shared_vector_hashes_to_exactly_the_recorded_digest() {
    for fixture in load() {
        let digest = hash(&fixture.value).expect("fixture values are hashable");
        assert_eq!(digest.to_hex(), fixture.sha256, "{}", fixture.name);
    }
}

#[test]
fn every_shared_vector_round_trips_through_decode() {
    for fixture in load() {
        let bytes = from_hex(&fixture.canonical);
        let decoded = partman_domain::canonical::decode(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.name));
        assert_eq!(decoded, fixture.value, "{}", fixture.name);
    }
}
