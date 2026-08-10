//! Tests for SAFE-003's identity record (WP-010 increment 3d).

use crate::canonical::{self, Hash, Value};

use super::identity::{
    ContinuityWitness, DeviceIdentity, IdentityParseError, IdentityStrength, IndeterminateCause,
    TableState, WitnessOutcome, compare_witness, identity_from_map,
};

fn record(table: TableState) -> DeviceIdentity {
    DeviceIdentity {
        serial: Some(b"S1".to_vec()),
        wwn: None,
        os_instance_id: Some(b"instance".to_vec()),
        connection_path: Some(b"pci-0000:00".to_vec()),
        total_bytes: 1 << 30,
        logical_sector_size: Some(512),
        physical_sector_size: Some(4096),
        table,
        witness: None,
    }
}

fn present() -> TableState {
    TableState::Present {
        checksum: canonical::hash(&Value::Unsigned(0)).expect("hashable"),
    }
}

// Requirements: SAFE-003
//   Strong requires a stable hardware identifier, both sector sizes, and a
//   positively determined table state; a device whose table failed to
//   parse cannot be Strong even with a serial — ADR-C3's tightening.
// Evidence: strength_requires_identifier_geometry_and_a_determined_table
#[test]
fn strength_requires_identifier_geometry_and_a_determined_table() {
    assert_eq!(record(present()).strength(), IdentityStrength::Strong);

    let mut no_identifier = record(present());
    no_identifier.serial = None;
    assert_eq!(no_identifier.strength(), IdentityStrength::Weak);

    let unreadable = record(TableState::Indeterminate {
        cause: IndeterminateCause::Unreadable,
    });
    assert_eq!(unreadable.strength(), IdentityStrength::Weak);

    let mut no_geometry = record(present());
    no_geometry.physical_sector_size = None;
    assert_eq!(no_geometry.strength(), IdentityStrength::Weak);
}

// Requirements: SAFE-003
//   A blank device with a positively determined absent table can be
//   Strong — ADR-C3's resolution of SI-02, where the observing contract
//   positively determines absence (the helper's raw read).
// Evidence: a_positively_absent_table_supports_strong
#[test]
fn a_positively_absent_table_supports_strong() {
    assert_eq!(
        record(TableState::Absent).strength(),
        IdentityStrength::Strong
    );
}

// Requirements: SAFE-003, MODEL-005
//   ADR-C4's guard, held in bytes: a positively absent table and an
//   unreadable one — and a present one — produce three pairwise distinct
//   body values.
// Evidence: the_three_table_states_are_three_distinct_body_values
#[test]
fn the_three_table_states_are_three_distinct_body_values() {
    let encode =
        |table: TableState| canonical::encode(&record(table).body_value()).expect("encodable");
    let present_bytes = encode(present());
    let absent_bytes = encode(TableState::Absent);
    let indeterminate_bytes = encode(TableState::Indeterminate {
        cause: IndeterminateCause::Unreadable,
    });
    assert_ne!(present_bytes, absent_bytes);
    assert_ne!(absent_bytes, indeterminate_bytes);
    assert_ne!(present_bytes, indeterminate_bytes);
}

// Requirements: SAFE-003
//   The witness comparison follows the measurements: comparable only
//   within an unchanged epoch, never on a decrease; movement is exchange;
//   absence on either side is unavailability.
// Evidence: witness_comparison_follows_the_measured_semantics
#[test]
fn witness_comparison_follows_the_measured_semantics() {
    let bound = ContinuityWitness {
        epoch_token: b"pdo-A".to_vec(),
        counter: 7,
    };
    let same = ContinuityWitness {
        epoch_token: b"pdo-A".to_vec(),
        counter: 7,
    };
    let moved = ContinuityWitness {
        epoch_token: b"pdo-A".to_vec(),
        counter: 9,
    };
    let decreased = ContinuityWitness {
        epoch_token: b"pdo-A".to_vec(),
        counter: 3,
    };
    let re_arrived = ContinuityWitness {
        epoch_token: b"pdo-B".to_vec(),
        counter: 7,
    };
    assert_eq!(
        compare_witness(Some(&bound), Some(&same)),
        WitnessOutcome::NoExchangeObserved
    );
    assert_eq!(
        compare_witness(Some(&bound), Some(&moved)),
        WitnessOutcome::ExchangeObserved
    );
    assert_eq!(
        compare_witness(Some(&bound), Some(&decreased)),
        WitnessOutcome::Incomparable,
        "a decrease is a reset the token failed to witness"
    );
    assert_eq!(
        compare_witness(Some(&bound), Some(&re_arrived)),
        WitnessOutcome::Incomparable
    );
    assert_eq!(
        compare_witness(None, Some(&bound)),
        WitnessOutcome::Unavailable
    );
    assert_eq!(
        compare_witness(Some(&bound), None),
        WitnessOutcome::Unavailable
    );
}

// Requirements: SAFE-003, MODEL-005
//   The record round-trips through its body value, and the witness field
//   is body content whose absence is representable and hash-distinct —
//   ADR-0017's placement.
// Evidence: the_record_round_trips_and_the_witness_is_body_content
#[test]
fn the_record_round_trips_and_the_witness_is_body_content() {
    let mut with_witness = record(present());
    with_witness.witness = Some(ContinuityWitness {
        epoch_token: b"pdo-A".to_vec(),
        counter: 41,
    });
    let Value::Map(map) = with_witness.body_value() else {
        panic!("body is a map");
    };
    let rebuilt = identity_from_map(&map).expect("parses");
    assert_eq!(rebuilt, with_witness);

    let without_witness = record(present());
    assert_ne!(
        canonical::encode(&with_witness.body_value()).expect("encodable"),
        canonical::encode(&without_witness.body_value()).expect("encodable")
    );
}

// Requirements: SAFE-003, MODEL-005
//   Strength is derived, never stored: no `strength` key exists in the
//   body, and a forged one refuses as an undeclared field — the
//   ADR-C4-style anti-assertion discipline applied to strength.
// Evidence: a_forged_strength_field_is_refused
#[test]
fn a_forged_strength_field_is_refused() {
    let Value::Map(mut map) = record(present()).body_value() else {
        panic!("body is a map");
    };
    assert!(!map.contains_key("strength"), "strength is never stored");
    map.insert("strength".to_owned(), Value::Text("strong".to_owned()));
    assert_eq!(
        identity_from_map(&map),
        Err(IdentityParseError::UnknownField {
            key: "strength".to_owned()
        })
    );
}

// Requirements: SAFE-003
//   A malformed table value, a truncated checksum, and an unknown
//   indeterminate cause are typed refusals.
// Evidence: malformed_table_values_are_typed_refusals
#[test]
fn malformed_table_values_are_typed_refusals() {
    let Value::Map(good) = record(present()).body_value() else {
        panic!("body is a map");
    };

    let mut short_checksum = good.clone();
    let mut table = std::collections::BTreeMap::new();
    table.insert("state".to_owned(), Value::Text("present".to_owned()));
    table.insert("checksum".to_owned(), Value::Bytes(vec![0; 8]));
    short_checksum.insert("table".to_owned(), Value::Map(table));
    assert!(matches!(
        identity_from_map(&short_checksum),
        Err(IdentityParseError::BadField { key: "table" })
    ));

    let mut bad_cause = good.clone();
    let mut table = std::collections::BTreeMap::new();
    table.insert("state".to_owned(), Value::Text("indeterminate".to_owned()));
    table.insert("cause".to_owned(), Value::Text("mystery".to_owned()));
    bad_cause.insert("table".to_owned(), Value::Map(table));
    assert!(matches!(
        identity_from_map(&bad_cause),
        Err(IdentityParseError::BadField { key: "table" })
    ));

    let mut absent_with_checksum = good;
    let mut table = std::collections::BTreeMap::new();
    table.insert("state".to_owned(), Value::Text("absent".to_owned()));
    table.insert(
        "checksum".to_owned(),
        Value::Bytes(Hash::from_bytes([0; 32]).as_bytes().to_vec()),
    );
    absent_with_checksum.insert("table".to_owned(), Value::Map(table));
    assert!(matches!(
        identity_from_map(&absent_with_checksum),
        Err(IdentityParseError::UnknownField { .. })
    ));
}
