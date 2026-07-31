use crate::identifier_display::{DisplayLimit, RawIdentifier};

use super::{
    SelectionId, SelectionLookupError, SelectionRegistry, SelectionRegistryError,
    SelectionWireError,
};

fn id(value: u64) -> SelectionId {
    SelectionId::new(value).expect("test IDs are nonzero")
}

// Requirements: UI-008, Section 12
//   A caller-supplied nonzero selection value crosses the future renderer boundary through one fixed uppercase-hex string without numeric loss or alternate spellings; allocation provenance remains a future view-model obligation
// Evidence: selection_wire_encoding_is_exact_and_strict
#[test]
fn selection_wire_encoding_is_exact_and_strict() {
    assert_eq!(id(1).to_wire().as_str(), "sid:0000000000000001");
    assert_eq!(id(u64::MAX).to_wire().as_str(), "sid:FFFFFFFFFFFFFFFF");
    for value in [1, 2, 0x00ff_1234, u64::MAX] {
        let original = id(value);
        assert_eq!(
            SelectionId::from_wire(original.to_wire().as_str()),
            Ok(original)
        );
    }

    assert_eq!(
        SelectionId::from_wire("sid:000000000000001"),
        Err(SelectionWireError::InvalidLength)
    );
    assert_eq!(
        SelectionId::from_wire("sid:10000000000000000"),
        Err(SelectionWireError::InvalidLength)
    );
    assert_eq!(
        SelectionId::from_wire("SID:0000000000000001"),
        Err(SelectionWireError::InvalidPrefix)
    );
    assert_eq!(
        SelectionId::from_wire("sid:000000000000000a"),
        Err(SelectionWireError::InvalidDigit)
    );
    assert_eq!(
        SelectionId::from_wire("sid:000000000000000G"),
        Err(SelectionWireError::InvalidDigit)
    );
    assert_eq!(
        SelectionId::from_wire("sid:000000000000000 "),
        Err(SelectionWireError::InvalidDigit)
    );
    assert_eq!(
        SelectionId::from_wire("sid:0000000000000000"),
        Err(SelectionWireError::Zero)
    );
    assert!(SelectionId::new(0).is_err());
}

// Requirements: UI-008, Section 12
//   Duplicate IDs, malformed callback values, and canonical but unknown IDs fail closed at the Rust-owned selection registry
// Evidence: selection_registry_rejects_duplicate_invalid_and_unknown_ids
#[test]
fn selection_registry_rejects_duplicate_invalid_and_unknown_ids() {
    assert_eq!(
        SelectionRegistry::new([id(1), id(2), id(1)]),
        Err(SelectionRegistryError::Duplicate(id(1)))
    );

    let registry = SelectionRegistry::new([id(1), id(2)]).expect("IDs are unique");
    assert_eq!(registry.len(), 2);
    assert!(!registry.is_empty());
    assert_eq!(registry.resolve_wire("sid:0000000000000002"), Ok(id(2)));
    assert_eq!(
        registry.resolve_wire("sid:0000000000000003"),
        Err(SelectionLookupError::Unknown(id(3)))
    );
    assert_eq!(
        registry.resolve_wire("not-a-selection"),
        Err(SelectionLookupError::InvalidWire(
            SelectionWireError::InvalidLength
        ))
    );
}

// Requirements: UI-008, Section 12
//   Registry membership resolves an explicitly supplied opaque ID independently of item order even when unrelated raw identifiers share presentation-only truncated text; correct item-to-ID association remains a future view-model obligation
// Evidence: registry_membership_is_order_independent_despite_a_presentation_collision
#[test]
fn registry_membership_is_order_independent_despite_a_presentation_collision() {
    let first = RawIdentifier::Bytes(b"AAAAAAAAAAXZZZZZZZZZZ".to_vec().into_boxed_slice());
    let second = RawIdentifier::Bytes(b"AAAAAAAAAAYZZZZZZZZZZ".to_vec().into_boxed_slice());
    let limit = DisplayLimit::new(12).expect("test limit is valid");
    let first_visual = first.bounded_display(limit);
    let second_visual = second.bounded_display(limit);
    assert!(first_visual.is_truncated());
    assert_eq!(first_visual, second_visual);
    assert_ne!(first.full_display(), second.full_display());

    // There is intentionally no identifier-to-selection-ID association here.
    // The later view model must create and test that binding; this registry can
    // establish only that lookup is independent of input iteration order.
    let before = SelectionRegistry::new([id(41), id(42)]).expect("IDs are unique");
    let selected = before
        .resolve_wire(id(42).to_wire().as_str())
        .expect("selected ID is present");

    let after = SelectionRegistry::new([id(42), id(41)]).expect("reordered IDs stay unique");
    assert_eq!(
        after.resolve_wire(selected.to_wire().as_str()),
        Ok(selected)
    );
}
