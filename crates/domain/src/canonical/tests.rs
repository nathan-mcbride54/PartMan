//! Conformance tests for the `pce/1` profile.
//!
//! The golden vectors are the load-bearing artifact of ADR-C1: they are what a
//! second implementation, in any language, is checked against. Changing one is
//! a profile change, never a test fix.

use std::collections::BTreeMap;

use super::{Error, MAX_DEPTH, Value, compare_keys, decode, encode, hash};

fn map(pairs: &[(&str, Value)]) -> Value {
    Value::Map(
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
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

/// Every vector is `(value, canonical hex)`.
///
/// These pin the profile. A cross-language implementation is conformant when it
/// reproduces this table exactly.
fn golden_vectors() -> Vec<(Value, &'static str)> {
    vec![
        // Integer width boundaries, each side of every argument form.
        (Value::Unsigned(0), "00"),
        (Value::Unsigned(23), "17"),
        (Value::Unsigned(24), "1818"),
        (Value::Unsigned(255), "18ff"),
        (Value::Unsigned(256), "190100"),
        (Value::Unsigned(65535), "19ffff"),
        (Value::Unsigned(65536), "1a00010000"),
        (Value::Unsigned(4_294_967_295), "1affffffff"),
        (Value::Unsigned(4_294_967_296), "1b0000000100000000"),
        // The JCS cliff: these are exactly the values RFC 8785 cannot carry as
        // numbers, and the reason ADR-C1 rejected canonical JSON.
        (Value::Unsigned(9_007_199_254_740_991), "1b001fffffffffffff"),
        (Value::Unsigned(9_007_199_254_740_992), "1b0020000000000000"),
        (Value::Unsigned(u64::MAX), "1bffffffffffffffff"),
        // Negative integers, including the extreme.
        (Value::Negative(-1), "20"),
        (Value::Negative(-24), "37"),
        (Value::Negative(-25), "3818"),
        (Value::Negative(i64::MIN), "3b7fffffffffffffff"),
        // Strings and byte strings, including empty and multi-byte.
        (Value::Text(String::new()), "60"),
        (Value::Text("a".to_owned()), "6161"),
        (Value::Text("\u{00e9}".to_owned()), "62c3a9"),
        // An astral-plane code point, four UTF-8 bytes.
        (Value::Text("\u{1f600}".to_owned()), "64f09f9880"),
        // Text may contain an embedded NUL; it is not a terminator.
        (Value::Text("a\u{0}b".to_owned()), "63610062"),
        (Value::Bytes(Vec::new()), "40"),
        (Value::Bytes(vec![0x01, 0x02]), "420102"),
        // Simple values.
        (Value::Bool(false), "f4"),
        (Value::Bool(true), "f5"),
        (Value::Null, "f6"),
        // Containers.
        (Value::Array(Vec::new()), "80"),
        (
            Value::Array(vec![Value::Unsigned(1), Value::Unsigned(2)]),
            "820102",
        ),
        (map(&[]), "a0"),
        (map(&[("a", Value::Unsigned(1))]), "a1616101"),
        // Length-first ordering: "z" precedes "aa" because it is shorter. Rust's
        // own String ordering would put "aa" first, so this vector is the one
        // that catches a BTreeMap iteration order leaking into the encoder.
        (
            map(&[("aa", Value::Unsigned(2)), ("z", Value::Unsigned(1))]),
            "a2617a0162616102",
        ),
        // A key that is a prefix of another key.
        (
            map(&[("a", Value::Unsigned(1)), ("ab", Value::Unsigned(2))]),
            "a261610162616202",
        ),
        // The boundary where a key length changes argument form: 23 vs 24 bytes.
        (
            map(&[
                ("k".repeat(23).as_str(), Value::Unsigned(1)),
                ("k".repeat(24).as_str(), Value::Unsigned(2)),
            ]),
            "a2776b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b01\
             78186b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b02",
        ),
    ]
}

#[test]
fn golden_vectors_encode_exactly() {
    for (value, expected) in golden_vectors() {
        let expected = expected.replace([' ', '\n'], "");
        let actual = encode(&value).expect("golden vectors are encodable");
        assert_eq!(hex(&actual), expected, "encoding {value:?}");
    }
}

#[test]
fn golden_vectors_round_trip() {
    for (value, expected) in golden_vectors() {
        let bytes = from_hex(&expected.replace([' ', '\n'], ""));
        let decoded = decode(&bytes).expect("golden vectors decode");
        assert_eq!(decoded, value, "decoding {expected}");
        assert_eq!(
            encode(&decoded).expect("re-encodable"),
            bytes,
            "re-encoding {expected}"
        );
    }
}

#[test]
fn hashing_is_sha256_over_canonical_bytes() {
    // SHA-256 of the single byte 0x00, which is the canonical encoding of 0.
    let digest = hash(&Value::Unsigned(0)).expect("hashable");
    assert_eq!(
        digest.to_hex(),
        "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
    );
    // The empty map is 0xa0, a different single byte, so a different digest.
    let empty_map = hash(&map(&[])).expect("hashable");
    assert_ne!(digest, empty_map);
}

#[test]
fn schema_fields_separate_domains() {
    // Two artifacts with identical payload shape but different schema fields
    // must not collide (schemas/canonical-encoding.md, section 5).
    let plan = map(&[
        ("schema", Value::Text("partman.plan".to_owned())),
        ("schema_version", Value::Unsigned(1)),
    ]);
    let snapshot = map(&[
        ("schema", Value::Text("partman.snapshot".to_owned())),
        ("schema_version", Value::Unsigned(1)),
    ]);
    assert_ne!(
        hash(&plan).expect("hashable"),
        hash(&snapshot).expect("hashable")
    );

    // A schema version bump changes the hash of the same logical content.
    let bumped = map(&[
        ("schema", Value::Text("partman.plan".to_owned())),
        ("schema_version", Value::Unsigned(2)),
    ]);
    assert_ne!(
        hash(&plan).expect("hashable"),
        hash(&bumped).expect("hashable")
    );
}

#[test]
fn key_ordering_is_length_first_not_bytewise() {
    use core::cmp::Ordering;
    // The distinguishing case: bytewise would order "aa" before "z".
    assert_eq!(compare_keys("z", "aa"), Ordering::Less);
    assert_eq!(
        "z".cmp("aa"),
        Ordering::Greater,
        "Rust orders these bytewise"
    );
    // Within one length, bytewise applies.
    assert_eq!(compare_keys("ab", "ac"), Ordering::Less);
    assert_eq!(compare_keys("a", "a"), Ordering::Equal);
}

#[test]
fn encoding_is_independent_of_insertion_order() {
    let forward = map(&[
        ("z", Value::Unsigned(1)),
        ("aa", Value::Unsigned(2)),
        ("b", Value::Unsigned(3)),
    ]);
    let mut reversed = BTreeMap::new();
    reversed.insert("b".to_owned(), Value::Unsigned(3));
    reversed.insert("aa".to_owned(), Value::Unsigned(2));
    reversed.insert("z".to_owned(), Value::Unsigned(1));
    assert_eq!(
        encode(&forward).expect("encodable"),
        encode(&Value::Map(reversed)).expect("encodable")
    );
}

#[test]
fn negative_variant_rejects_non_negative_payload() {
    assert_eq!(
        encode(&Value::Negative(0)),
        Err(Error::NegativeOutOfRange(0))
    );
}

/// Every input here is well-formed CBOR that this profile must still reject.
///
/// Accepting any of them would let bytes that hash one way decode to a value
/// that was authorized under a different hash.
#[test]
fn non_canonical_and_excluded_input_is_rejected() {
    let cases: &[(&str, &str, Error)] = &[
        (
            "non-shortest 1-byte argument",
            "1801",
            Error::NonShortestArgument,
        ),
        (
            "non-shortest 2-byte argument",
            "190017",
            Error::NonShortestArgument,
        ),
        (
            "non-shortest 4-byte argument",
            "1a00000017",
            Error::NonShortestArgument,
        ),
        (
            "non-shortest 8-byte argument",
            "1b0000000000000017",
            Error::NonShortestArgument,
        ),
        (
            "non-shortest text length",
            "78016161",
            Error::NonShortestArgument,
        ),
        ("half-precision float", "f93c00", Error::FloatNotAllowed),
        (
            "single-precision float",
            "fa3f800000",
            Error::FloatNotAllowed,
        ),
        (
            "double-precision float 0.0",
            "fb0000000000000000",
            Error::FloatNotAllowed,
        ),
        ("undefined", "f7", Error::SimpleValueNotAllowed(23)),
        ("tag 42", "d82a01", Error::TagNotAllowed(42)),
        ("tag 0", "c001", Error::TagNotAllowed(0)),
        ("indefinite-length array", "9f01ff", Error::IndefiniteLength),
        (
            "indefinite-length text",
            "7f6161ff",
            Error::IndefiniteLength,
        ),
        (
            "reserved additional information 28",
            "1c",
            Error::ReservedAdditionalInformation(28),
        ),
        ("integer map key", "a10101", Error::MapKeyNotText),
        (
            "negative argument above i64::MAX",
            "3b8000000000000000",
            Error::NegativeTooLarge(9_223_372_036_854_775_808),
        ),
        ("trailing byte", "0000", Error::TrailingBytes(1)),
        ("truncated head", "18", Error::UnexpectedEnd),
        // The declared length is checked against the remaining input before the
        // payload is read, so a truncated string reports the specific rule from
        // section 6 obligation 7 rather than a generic end-of-input.
        (
            "truncated text payload",
            "6261",
            Error::LengthExceedsInput {
                declared: 2,
                remaining: 1,
            },
        ),
    ];

    for (name, input, expected) in cases {
        let actual = decode(&from_hex(input));
        assert_eq!(actual, Err(expected.clone()), "{name} ({input})");
    }
}

#[test]
fn duplicate_and_misordered_map_keys_are_rejected() {
    // {"a": 1, "a": 2} -- a duplicate key.
    let duplicate = from_hex("a2616101616102");
    assert_eq!(
        decode(&duplicate),
        Err(Error::MapKeyNotStrictlyIncreasing {
            key: "a".to_owned()
        })
    );

    // {"aa": 2, "z": 1} -- bytewise order, which this profile forbids because
    // "z" is shorter and must come first.
    let misordered = from_hex("a262616102617a01");
    assert_eq!(
        decode(&misordered),
        Err(Error::MapKeyNotStrictlyIncreasing {
            key: "z".to_owned()
        })
    );
}

#[test]
fn ill_formed_utf8_is_rejected() {
    // 0x61 declares a 1-byte text string whose payload is a lone continuation
    // byte.
    assert_eq!(decode(&from_hex("6180")), Err(Error::InvalidUtf8));
    // An unpaired surrogate encoded in the CESU-8 style Rust must reject.
    assert_eq!(decode(&from_hex("63eda080")), Err(Error::InvalidUtf8));
}

#[test]
fn declared_length_beyond_input_is_rejected_before_allocating() {
    // A byte string claiming 2^32 bytes with none present. The check must fire
    // on the declared length rather than attempt the allocation.
    let hostile = from_hex("5affffffff");
    assert_eq!(
        decode(&hostile),
        Err(Error::LengthExceedsInput {
            declared: 4_294_967_295,
            remaining: 0
        })
    );

    // The same for an array element count.
    let hostile_array = from_hex("9bffffffffffffffff");
    assert!(matches!(
        decode(&hostile_array),
        Err(Error::LengthExceedsInput { .. })
    ));
}

#[test]
fn nesting_beyond_the_depth_limit_is_rejected_not_crashed() {
    // MAX_DEPTH + 2 nested single-element arrays: deep enough to exceed the
    // limit, and the point of the test is that this returns an error rather
    // than exhausting the stack.
    let depth = MAX_DEPTH + 2;
    let mut bytes = vec![0x81; depth];
    bytes.push(0x00);
    assert_eq!(decode(&bytes), Err(Error::DepthLimitExceeded));

    // One level inside the limit still decodes.
    let mut shallow = vec![0x81; MAX_DEPTH - 1];
    shallow.push(0x00);
    assert!(decode(&shallow).is_ok());
}

/// The encoder and decoder must agree on what is representable.
///
/// Before this was enforced, `encode` accepted arbitrary nesting and happily
/// produced bytes that `decode` then rejected with `DepthLimitExceeded`. A
/// producer would have computed and published a hash over an artifact no
/// conforming decoder could revalidate — and, since `encode` recursed without
/// bound, a deep enough value was a stack overflow rather than an error.
#[test]
fn the_encoder_enforces_the_same_depth_limit_as_the_decoder() {
    let mut deep = Value::Unsigned(0);
    for _ in 0..=MAX_DEPTH {
        deep = Value::Array(vec![deep]);
    }
    assert_eq!(encode(&deep), Err(Error::DepthLimitExceeded));

    // One level inside the limit encodes, and what it produces decodes.
    let mut permitted = Value::Unsigned(0);
    for _ in 0..(MAX_DEPTH - 1) {
        permitted = Value::Array(vec![permitted]);
    }
    let bytes = encode(&permitted).expect("within the limit");
    assert_eq!(decode(&bytes), Ok(permitted));
}

/// Anything the encoder emits, the decoder accepts.
#[test]
fn encoder_output_is_always_decodable() {
    for (value, _) in golden_vectors() {
        let bytes = encode(&value).expect("golden vectors encode");
        assert_eq!(decode(&bytes), Ok(value), "round trip");
    }
}

#[test]
fn empty_input_is_rejected() {
    assert_eq!(decode(&[]), Err(Error::UnexpectedEnd));
}

/// The exact hazard ADR-C1 recorded, pinned as a regression guard.
///
/// A `u64` that reaches a JavaScript encoder as `Number` rather than `BigInt`
/// is not encoded as an integer. Running the `cborg` oracle on `2**53` gives:
///
/// * `BigInt(2**53)` -> `1b0020000000000000`, the correct unsigned integer.
/// * `Number(2**53)` -> `fa5a000000`, a *single-precision float*.
///
/// The profile excludes floats, so the mistake is rejected here rather than
/// silently producing an artifact whose hash disagrees with the Rust side. If
/// floats were ever admitted to the value model, this defense disappears.
#[test]
fn a_javascript_number_encoded_as_float_is_rejected() {
    let number_2_pow_53 = from_hex("fa5a000000");
    assert_eq!(decode(&number_2_pow_53), Err(Error::FloatNotAllowed));

    // The BigInt form is the one that round-trips.
    let bigint_2_pow_53 = from_hex("1b0020000000000000");
    assert_eq!(
        decode(&bigint_2_pow_53),
        Ok(Value::Unsigned(9_007_199_254_740_992))
    );
}

#[test]
fn errors_describe_the_rule_they_enforce() {
    // The message names the violated rule, because operators and reviewers use
    // it to tell a corrupt artifact from a rejected attack.
    assert!(
        Error::NonShortestArgument.to_string().contains("shortest"),
        "message should name the shortest-form rule"
    );
    assert!(
        Error::FloatNotAllowed
            .to_string()
            .contains("floating-point")
    );
    assert!(
        Error::DepthLimitExceeded
            .to_string()
            .contains(&MAX_DEPTH.to_string())
    );
}

#[test]
fn hashing_bytes_requires_proving_they_are_canonical() {
    // The public surface used to include `hash_canonical_bytes(&[u8]) -> Hash`,
    // whose documentation asked callers to pass only canonical bytes. The plan
    // hash is an authorization boundary under HLP-001, HLP-003 and SEC-001, and
    // an instruction in a doc comment is not a guarantee — so the proof is now
    // `decode` itself, which accepts only the unique canonical encoding.
    for (what, bytes) in [
        // Decodes to 0, but is not the canonical encoding of 0. Hashing it
        // would put a second digest on one logical value, which is exactly the
        // malleability ADR-C1 exists to remove.
        ("a non-shortest argument", vec![0x18, 0x00]),
        ("trailing bytes", vec![0xf6, 0xf6]),
        ("a float", vec![0xfa, 0x00, 0x00, 0x00, 0x00]),
        ("a tag", vec![0xc0, 0x01]),
        ("an indefinite-length array", vec![0x9f, 0x01, 0xff]),
        (
            "a duplicate map key",
            vec![0xa2, 0x61, 0x61, 0x01, 0x61, 0x61, 0x02],
        ),
        ("empty input", Vec::new()),
    ] {
        assert!(
            super::hash_encoded(&bytes).is_err(),
            "{what} must not be hashable: {bytes:02x?}"
        );
    }
}

#[test]
fn hashing_canonical_bytes_agrees_with_hashing_the_value() {
    // Narrowing the API must not have changed any digest. Both routes to a hash
    // have to agree, or one of them is authorizing different bytes.
    for value in [
        Value::Null,
        Value::Bool(true),
        Value::Unsigned(0),
        Value::Unsigned(u64::MAX),
        Value::Negative(i64::MIN),
        Value::Text("é".to_owned()),
        Value::Bytes(vec![0, 1, 2]),
        Value::Array(vec![Value::Null, Value::Bool(false)]),
    ] {
        let encoded = encode(&value).expect("encodable");
        assert_eq!(
            super::hash_encoded(&encoded).expect("its own bytes must be canonical"),
            hash(&value).expect("hashable"),
            "{value:?}"
        );
    }
}
