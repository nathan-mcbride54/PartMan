//! The property that makes the plan hash an authorization boundary.
//!
//! > For any input, `decode` either fails, or returns a value whose `encode`
//! > reproduces the input **byte for byte**.
//!
//! If the decoder ever accepted a non-canonical encoding, an attacker could
//! submit bytes that decode to an approved plan yet hash differently — so the
//! bytes a user authorized would not be the bytes describing what executes.
//! Hand-written rejection cases can only prove the cases someone thought of.
//!
//! `crates/domain/tests/canonicality.rs` asserts the same property on stable
//! over a bounded mutation space, so a regression fails `cargo xtask ci` and not
//! only a scheduled fuzz run.

#![no_main]

use libfuzzer_sys::fuzz_target;
use partman_domain::canonical::{decode, encode, hash, hash_canonical_bytes};

fuzz_target!(|data: &[u8]| {
    let Ok(value) = decode(data) else {
        // Rejection is always correct: the profile is a strict subset of CBOR,
        // so most byte strings are legitimately outside it.
        return;
    };

    let reencoded = encode(&value).expect("a decoded value must be re-encodable");
    assert_eq!(
        reencoded, data,
        "decode accepted a non-canonical encoding"
    );

    // An accepted value must hash, and must agree with hashing its own bytes.
    let by_value = hash(&value).expect("a decoded value must be hashable");
    assert_eq!(
        by_value,
        hash_canonical_bytes(data),
        "hash disagreed with its own canonical bytes"
    );

    // Decoding is idempotent on canonical bytes.
    assert_eq!(decode(&reencoded), Ok(value), "re-encoded bytes must decode back");
});
