//! The property that makes the bounded plist reader's promise a promise.
//!
//! > For any input, `parse` either refuses with a typed error, or returns a
//! > value that sits inside every bound the module declares: container depth
//! > within `DEPTH_LIMIT`, total values within `NODE_LIMIT`, and every text
//! > run within `VALUE_LIMIT`.
//!
//! The reader consumes bytes a subprocess produced — externally supplied
//! input under Section 11.4 — and the macOS adapter builds `inspect` output
//! from what it returns. If a crafted document could smuggle an
//! over-limit value or an over-deep tree past the caps, the "bounded" in
//! "bounded reader" would be prose rather than a property. Hand-written
//! refusal cases in `apps/cli/src/tests.rs` prove the cases someone thought
//! of; this target searches for the ones nobody did.
//!
//! The two extraction entry points are driven on every input too: they must
//! never panic, and an input they accept must be an input `parse` accepts —
//! extraction is a view over the grammar, never a second grammar.
//!
//! Reachability under the engine's 4096-byte input cap
//! (`fuzz_engine_args`), stated exactly: the depth cap is reachable and
//! genuinely searched (seventeen nested containers fit in ~120 bytes); the
//! oversize, over-value, and over-node refusals all need larger inputs and
//! rest on the stable unit tests. The bound assertions below are kept for
//! all three anyway — they cost nothing per iteration and hold whichever
//! way the engine cap moves — but what this target searches beyond the
//! depth cap is panic-freedom and extractor consistency over the grammar.

#![no_main]

use libfuzzer_sys::fuzz_target;
use partman_cli::plist::{DEPTH_LIMIT, NODE_LIMIT, VALUE_LIMIT, Value, info_fields, parse, whole_disks};

/// Walk one parsed value, asserting every declared bound and returning the
/// value count under this node (itself included).
fn assert_bounds(value: &Value, depth: usize) -> usize {
    assert!(
        depth <= DEPTH_LIMIT,
        "a parsed tree exceeds the declared depth limit"
    );
    match value {
        Value::Dict(entries) => {
            let mut nodes = 1;
            for (key, child) in entries {
                assert!(
                    key.len() <= VALUE_LIMIT,
                    "a parsed key exceeds the declared value limit"
                );
                nodes += assert_bounds(child, depth + 1);
            }
            nodes
        }
        Value::Array(elements) => {
            let mut nodes = 1;
            for child in elements {
                nodes += assert_bounds(child, depth + 1);
            }
            nodes
        }
        Value::String(text) | Value::Integer(text) => {
            assert!(
                text.len() <= VALUE_LIMIT,
                "a parsed text run exceeds the declared value limit"
            );
            1
        }
        Value::Bool(_) => 1,
    }
}

fuzz_target!(|data: &[u8]| {
    match parse(data) {
        Ok(value) => {
            let nodes = assert_bounds(&value, 0);
            assert!(
                nodes <= NODE_LIMIT,
                "a parsed tree exceeds the declared node limit"
            );
        }
        Err(_) => {
            // Refusal is always correct: the accepted grammar is a narrow
            // subset of XML, so most byte strings are legitimately outside
            // it. The typed refusal is the answer, not a failure mode.
        }
    }

    // The extractors are views over the same grammar. They must never
    // panic, and anything they accept, `parse` must accept.
    let list = whole_disks(data);
    let info = info_fields(data);
    if list.is_ok() || info.is_ok() {
        assert!(
            parse(data).is_ok(),
            "an extractor accepted bytes the parser refuses"
        );
    }
});
