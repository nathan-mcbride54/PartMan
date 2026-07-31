//! Rust half of the schema-level canonical-set parity proof.
//!
//! Both implementations read `schemas/domain/canonical-set-vectors.json`.
//! Crucially, every producer vector arrives out of order: the test exercises
//! the sort rather than merely encoding a fixture already arranged as expected.

use std::collections::BTreeMap;
use std::path::PathBuf;

use partman_domain::canonical::set::{self, Error as SetError};
use partman_domain::canonical::{MAX_DEPTH, Value, decode, encode, hash_encoded};
use serde_json::Value as Json;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/domain/canonical-set-vectors.json")
}

fn document() -> Json {
    let path = fixture_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
    let document: Json = serde_json::from_str(&raw).expect("fixture is valid JSON");
    assert_eq!(
        document["schema"].as_str(),
        Some("partman.canonical-set-vectors")
    );
    assert_eq!(document["schema_version"].as_u64(), Some(1));
    assert_eq!(
        document["rule"].as_str(),
        Some("unsigned-lexicographic-full-pce-element-bytes")
    );
    document
}

fn from_hex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "hex needs an even length");
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).expect("valid hex"))
        .collect()
}

fn build(json: &Json) -> Value {
    let object = json.as_object().expect("value is an object");
    assert_eq!(object.len(), 1, "value has exactly one tag");
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
                entries.insert(
                    pair[0].as_str().expect("key is a string").to_owned(),
                    build(&pair[1]),
                );
            }
            Value::Map(entries)
        }
        "bool" => Value::Bool(payload.as_bool().expect("bool is a boolean")),
        "null" => Value::Null,
        other => panic!("unrecognized value representation {other:?}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

// Requirements: MODEL-005, MODEL-006
//   Both languages actively sort one shared set fixture and reproduce the exact same canonical bytes and SHA-256 digests
// Evidence: shared_set_vectors_exercise_sorting_and_match_recorded_bytes_and_hashes
#[test]
fn shared_set_vectors_exercise_sorting_and_match_recorded_bytes_and_hashes() {
    let document = document();
    let vectors = document["ordering_vectors"]
        .as_array()
        .expect("ordering vectors are an array");
    assert!(vectors.len() >= 3, "the ordering fixture looks truncated");

    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let set_depth = usize::try_from(vector["set_depth"].as_u64().expect("set depth"))
            .expect("set depth fits usize");
        let input: Vec<Value> = vector["input"]
            .as_array()
            .expect("input is an array")
            .iter()
            .map(build)
            .collect();
        let expected = vector["canonical"].as_str().expect("canonical");

        let semantic_array = encode(&Value::Array(input.clone())).expect("plain array encodes");
        assert_ne!(
            hex(&semantic_array),
            expected,
            "{name}: fixture input is already sorted, so it proves no sorting"
        );

        let actual =
            set::encode_array(&input, set_depth).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(hex(&actual), expected, "{name}");
        assert_eq!(
            hash_encoded(&actual)
                .expect("set bytes are canonical pce")
                .to_hex(),
            vector["sha256"].as_str().expect("sha256"),
            "{name}"
        );

        let Value::Array(decoded) = decode(&actual).expect("encoded set decodes") else {
            panic!("{name}: set encoding did not produce an array");
        };
        set::validate_array(&decoded, 0).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

// Requirements: MODEL-006
//   The committed extent case makes bytewise and length-first comparators disagree, so choosing the wrong convention fails a test
// Evidence: the_extent_vector_distinguishes_bytewise_from_length_first_order
#[test]
fn the_extent_vector_distinguishes_bytewise_from_length_first_order() {
    let document = document();
    let vector = &document["ordering_vectors"][0];
    let input: Vec<Value> = vector["input"]
        .as_array()
        .expect("input")
        .iter()
        .map(build)
        .collect();
    let first = encode(&input[0]).expect("first extent encodes");
    let second = encode(&input[1]).expect("second extent encodes");

    assert!(
        first.len() < second.len(),
        "length-first would keep the first input first"
    );
    let actual = set::encode_array(&input, 0).expect("set encodes");
    assert!(
        actual[1..].starts_with(&second),
        "plain bytewise must move the longer second input first"
    );
}

// Requirements: MODEL-006, SAFE-005
//   Schema-set validation accepts only strict byte order and fails closed on descending or duplicate elements without repairing them
// Evidence: shared_set_validation_vectors_enforce_strict_order_without_repair
#[test]
fn shared_set_validation_vectors_enforce_strict_order_without_repair() {
    let document = document();
    let vectors = document["validation_vectors"]
        .as_array()
        .expect("validation vectors are an array");
    assert!(vectors.len() >= 3, "the validation fixture looks truncated");

    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let set_depth = usize::try_from(vector["set_depth"].as_u64().expect("set depth"))
            .expect("set depth fits usize");
        let observed: Vec<Value> = vector["observed"]
            .as_array()
            .expect("observed is an array")
            .iter()
            .map(build)
            .collect();
        let result = set::validate_array(&observed, set_depth);
        if vector["accepted"].as_bool().expect("accepted") {
            result.unwrap_or_else(|error| panic!("{name}: {error}"));
            continue;
        }

        match vector["error"].as_str().expect("error") {
            "duplicate" => assert!(
                matches!(result, Err(SetError::DuplicateElement { .. })),
                "{name}: {result:?}"
            ),
            "not-strictly-increasing" => assert!(
                matches!(result, Err(SetError::NotStrictlyIncreasing { .. })),
                "{name}: {result:?}"
            ),
            other => panic!("{name}: unknown expected error {other}"),
        }
    }
}

// Requirements: MODEL-005, MODEL-006, SAFE-005
//   Both set producers and validators consume the enclosing artifact's remaining depth budget; a standalone-valid element is refused exactly when the combined depth exceeds the shared limit
// Evidence: set_element_encoding_inherits_the_enclosing_depth_budget
#[test]
fn set_element_encoding_inherits_the_enclosing_depth_budget() {
    let document = document();
    let vectors = document["depth_vectors"]
        .as_array()
        .expect("depth vectors are an array");
    assert!(vectors.len() >= 2, "the depth fixture looks truncated");

    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let set_depth = usize::try_from(vector["set_depth"].as_u64().expect("set depth"))
            .expect("set depth fits usize");
        let element_array_depth = vector["element_array_depth"]
            .as_u64()
            .expect("element array depth");
        let mut element = Value::Unsigned(0);
        for _ in 0..element_array_depth {
            element = Value::Array(vec![element]);
        }

        encode(&element).unwrap_or_else(|error| panic!("{name}: standalone element: {error}"));
        let elements = [element];
        let produced = set::encode_array(&elements, set_depth);
        let validated = set::validate_array(&elements, set_depth);
        if vector["accepted"].as_bool().expect("accepted") {
            produced.unwrap_or_else(|error| panic!("{name}: producer inherited boundary: {error}"));
            validated
                .unwrap_or_else(|error| panic!("{name}: validator inherited boundary: {error}"));
        } else {
            assert!(
                matches!(
                    produced,
                    Err(SetError::ElementNotEncodable {
                        source: partman_domain::canonical::Error::DepthLimitExceeded,
                        ..
                    })
                ),
                "{name}: producer: {produced:?}"
            );
            assert!(
                matches!(
                    validated,
                    Err(SetError::ElementNotEncodable {
                        source: partman_domain::canonical::Error::DepthLimitExceeded,
                        ..
                    })
                ),
                "{name}: validator: {validated:?}"
            );
        }
    }
}

// Requirements: MODEL-005, MODEL-006
//   Ordinary arrays preserve semantic order and remain generically hashable, while schema-declared sets sort and reject duplicates at their typed boundary
// Evidence: semantic_arrays_keep_order_and_set_duplicates_are_not_removed
#[test]
fn semantic_arrays_keep_order_and_set_duplicates_are_not_removed() {
    let descending = vec![Value::Unsigned(1), Value::Unsigned(0)];
    assert_eq!(
        encode(&Value::Array(descending.clone())).expect("array encodes"),
        from_hex("820100")
    );
    assert_eq!(
        set::encode_array(&descending, 0).expect("set encodes"),
        from_hex("820001")
    );
    assert_eq!(
        decode(&from_hex("820100")),
        Ok(Value::Array(descending)),
        "pce/1 still accepts a descending semantic array"
    );
    assert!(
        hash_encoded(&from_hex("820100")).is_ok(),
        "generic hashing proves pce/1 bytes, not a field's schema declaration"
    );

    let duplicate = vec![
        Value::Text("same".to_owned()),
        Value::Text("same".to_owned()),
    ];
    assert!(matches!(
        set::encode_array(&duplicate, 0),
        Err(SetError::DuplicateElement { .. })
    ));
    assert!(matches!(
        set::validate_array(&duplicate, 0),
        Err(SetError::DuplicateElement { .. })
    ));
}

// Requirements: MODEL-006, SAFE-005
//   The schema layer has its own fail-closed error boundary and treats the set array's actual depth as mandatory data
// Evidence: schema_set_errors_are_distinct_and_depth_is_not_optional
#[test]
fn schema_set_errors_are_distinct_and_depth_is_not_optional() {
    assert_eq!(
        set::encode_array(&[], MAX_DEPTH + 1),
        Err(SetError::SetDepthLimitExceeded {
            depth: MAX_DEPTH + 1
        })
    );
    assert_eq!(
        set::validate_array(&[], MAX_DEPTH + 1),
        Err(SetError::SetDepthLimitExceeded {
            depth: MAX_DEPTH + 1
        })
    );
    assert!(set::encode_array(&[], MAX_DEPTH).is_ok());
    assert!(matches!(
        set::encode_array(&[Value::Negative(0)], 0),
        Err(SetError::ElementNotEncodable {
            source: partman_domain::canonical::Error::NegativeOutOfRange(0),
            ..
        })
    ));
}
