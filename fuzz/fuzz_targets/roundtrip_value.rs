//! Round-trip from the value side: `decode(encode(v)) == v`.
//!
//! The sibling target drives the decoder from bytes. This one drives the
//! encoder from structured values, which reaches shapes that random bytes
//! almost never produce — deeply nested containers, maps whose keys collide
//! under one ordering but not another, and integers at every argument-width
//! boundary.
//!
//! `Value` is built here from raw entropy rather than by deriving `Arbitrary`
//! on the domain type, so the domain crate keeps no fuzzing dependency.

#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use partman_domain::canonical::set::{self, Error as SetError};
use partman_domain::canonical::{Error, MAX_DEPTH, Value, decode, encode};
use std::collections::BTreeMap;

/// Build a value, bounding recursion so generation terminates.
///
/// The bound is deliberately a little above `MAX_DEPTH` so the encoder's own
/// depth rejection is exercised rather than avoided.
fn build(u: &mut Unstructured<'_>, depth: usize) -> Result<Value> {
    let leaf_only = depth >= MAX_DEPTH + 4;
    let choice = if leaf_only {
        u.int_in_range(0..=5)?
    } else {
        u.int_in_range(0..=7)?
    };

    Ok(match choice {
        0 => Value::Unsigned(u64::arbitrary(u)?),
        1 => {
            let raw = i64::arbitrary(u)?;
            // Value::Negative is only valid below zero.
            Value::Negative(if raw < 0 { raw } else { -1 })
        }
        2 => Value::Bytes(Vec::<u8>::arbitrary(u)?),
        3 => Value::Text(String::arbitrary(u)?),
        4 => Value::Bool(bool::arbitrary(u)?),
        5 => Value::Null,
        6 => {
            let count = u.int_in_range(0..=4)?;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(build(u, depth + 1)?);
            }
            Value::Array(items)
        }
        _ => {
            let count = u.int_in_range(0..=4)?;
            let mut entries = BTreeMap::new();
            for _ in 0..count {
                entries.insert(String::arbitrary(u)?, build(u, depth + 1)?);
            }
            Value::Map(entries)
        }
    })
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(value) = build(&mut u, 0) else { return };

    let bytes = match encode(&value) {
        Ok(bytes) => bytes,
        // The only permitted refusals. Anything else is a bug.
        Err(Error::DepthLimitExceeded | Error::NegativeOutOfRange(_)) => return,
        Err(other) => panic!("encoder refused a representable value: {other}"),
    };

    // Whatever the encoder emits, the decoder must accept, and it must be the
    // same value. This is the symmetry that was missing before the encoder
    // enforced the decoder's depth limit.
    assert_eq!(
        decode(&bytes).as_ref(),
        Ok(&value),
        "round trip changed the value"
    );

    // And the encoding must be canonical: re-encoding is a fixed point.
    let again = encode(&decode(&bytes).expect("just decoded")).expect("re-encodable");
    assert_eq!(again, bytes, "encoding is not a fixed point");

    // When the generated top-level value is an array, drive the schema-level
    // set producer too. The pce encoder above already proved every element is
    // encodable at this depth, so only duplicate logical elements may be
    // refused. A successful producer result must decode to an array that the
    // set validator accepts unchanged.
    if let Value::Array(elements) = &value {
        match set::encode_array(elements, 0) {
            Ok(set_bytes) => {
                let Value::Array(sorted) = decode(&set_bytes).expect("set bytes decode") else {
                    panic!("a set producer emitted something other than an array");
                };
                set::validate_array(&sorted, 0).expect("set producer emitted strict order");
                assert_eq!(
                    set::encode_array(&sorted, 0).expect("validated set re-encodes"),
                    set_bytes,
                    "schema-set encoding is not a fixed point"
                );
            }
            Err(SetError::DuplicateElement { .. }) => {}
            Err(other) => panic!("set producer refused encodable unique elements: {other}"),
        }
    }
});
