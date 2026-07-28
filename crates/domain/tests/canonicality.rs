//! The canonicality property, checked exhaustively over mutated inputs.
//!
//! Section 11.4 requires a `cargo-fuzz` target for this decoder, and one exists
//! in `fuzz/`. That target needs a nightly toolchain and runs in CI. This file
//! asserts *the same property* on the pinned stable toolchain, over a bounded,
//! deterministic mutation space, so the property is verifiable on any developer
//! machine and a regression is caught by `cargo xtask ci` rather than only by a
//! scheduled fuzz run.
//!
//! The property is the one that makes the plan hash an authorization boundary:
//!
//! > For any input, `decode` either fails, or returns a value whose `encode`
//! > reproduces the input **byte for byte**.
//!
//! A decoder that accepted a non-canonical encoding would let an attacker
//! submit bytes that decode to an approved plan yet hash differently. Listing
//! rejection cases by hand can only prove the cases someone thought of; this
//! searches every single-bit and single-byte perturbation of every known-good
//! encoding, which is where a missed rule actually hides.

use partman_domain::canonical::{decode, encode, hash};

/// Canonical encodings drawn from the shared fixture, plus nested shapes.
///
/// Seeds are the interesting neighbourhoods: mutating a valid encoding reaches
/// the near-miss inputs that a decoder is most likely to accept by accident.
fn seeds() -> Vec<Vec<u8>> {
    let hexes = [
        "00",
        "17",
        "1818",
        "18ff",
        "190100",
        "19ffff",
        "1a00010000",
        "1affffffff",
        "1b0000000100000000",
        "1b001fffffffffffff",
        "1b0020000000000000",
        "1bffffffffffffffff",
        "20",
        "37",
        "3818",
        "3b7fffffffffffffff",
        "60",
        "6161",
        "62c3a9",
        "64f09f9880",
        "63610062",
        "40",
        "420102",
        "f4",
        "f5",
        "f6",
        "80",
        "820102",
        "a0",
        "a1616101",
        "a2617a0162616102",
        "a261610162616202",
        "a1617882a16179f6f5",
        "8281008101",
        "a2616101616202",
    ];
    hexes
        .iter()
        .map(|text| {
            (0..text.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("valid hex"))
                .collect()
        })
        .collect()
}

/// Assert the canonicality property for one input.
///
/// This is exactly what `fuzz/fuzz_targets/decode_is_canonical.rs` asserts.
fn check(input: &[u8]) {
    let Ok(value) = decode(input) else {
        // Rejection is always a correct outcome. The profile is a strict subset
        // of CBOR, so most byte strings are legitimately not in it.
        return;
    };

    let reencoded = encode(&value).expect("a decoded value must be re-encodable");
    assert_eq!(
        reencoded, input,
        "decode accepted a non-canonical encoding: {input:02x?} re-encodes to {reencoded:02x?}"
    );

    // Hashing an accepted value must not panic, and must agree with hashing the
    // bytes it came from.
    let by_value = hash(&value).expect("a decoded value must be hashable");
    let by_bytes = partman_domain::canonical::hash_canonical_bytes(input);
    assert_eq!(
        by_value, by_bytes,
        "hash disagreed with its own canonical bytes"
    );
}

#[test]
fn seeds_are_canonical() {
    for seed in seeds() {
        assert!(decode(&seed).is_ok(), "seed must decode: {seed:02x?}");
        check(&seed);
    }
}

#[test]
fn every_single_bit_flip_preserves_canonicality() {
    let mut checked = 0_usize;
    for seed in seeds() {
        for index in 0..seed.len() {
            for bit in 0..8 {
                let mut mutated = seed.clone();
                mutated[index] ^= 1 << bit;
                check(&mutated);
                checked += 1;
            }
        }
    }
    assert!(
        checked > 1000,
        "expected a broad mutation space, got {checked}"
    );
}

#[test]
fn every_truncation_preserves_canonicality() {
    for seed in seeds() {
        for length in 0..seed.len() {
            check(&seed[..length]);
        }
    }
}

#[test]
fn appended_bytes_preserve_canonicality() {
    // Trailing data must never be silently ignored, or two byte strings would
    // decode to one value and hash differently.
    for seed in seeds() {
        for extra in [0x00_u8, 0x01, 0x20, 0x60, 0x80, 0xa0, 0xf6, 0xff] {
            let mut mutated = seed.clone();
            mutated.push(extra);
            assert!(
                decode(&mutated).is_err(),
                "trailing byte {extra:#04x} was accepted after {seed:02x?}"
            );
        }
    }
}

#[test]
fn byte_substitutions_preserve_canonicality() {
    // Header bytes carry the major type and argument form, so substituting each
    // position with boundary values reaches most malformed-head cases.
    for seed in seeds() {
        for index in 0..seed.len() {
            for replacement in [
                0x00_u8, 0x18, 0x19, 0x1a, 0x1b, 0x1f, 0x5f, 0x7f, 0xc0, 0xf7, 0xff,
            ] {
                let mut mutated = seed.clone();
                mutated[index] = replacement;
                check(&mutated);
            }
        }
    }
}

#[test]
fn short_exhaustive_inputs_preserve_canonicality() {
    // Every one-byte and two-byte input. Small enough to enumerate, and it
    // covers every initial byte, which is where major type and argument form
    // are decided.
    for first in 0..=u8::MAX {
        check(&[first]);
        for second in 0..=u8::MAX {
            check(&[first, second]);
        }
    }
}
