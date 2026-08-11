//! Tests for the CAP-003 vocabulary (WP-050 increment 1).

use partman_domain::model::capability::ProtectionGate;
use partman_domain::model::protection::{IndeterminateGround, RefusalGround};

use super::{
    Capability, ProtectionIndeterminacyReason, ProtectionRefusalReason, REASON_SCHEMA,
    REASON_SCHEMA_VERSION, Reason, Remediation, Status,
};

// Requirements: CAP-003, MODEL-003
//   The reason vocabulary is closed and versioned: the schema identity
//   and version are pinned literals, and the variant roster is exactly
//   the decided texts' — a variant added, removed, or re-represented
//   moves this test, which is the review point the version bump rides.
// Evidence: the_reason_vocabulary_is_closed_and_versioned
#[test]
fn the_reason_vocabulary_is_closed_and_versioned() {
    assert_eq!(REASON_SCHEMA, "partman.capability.reason");
    assert_eq!(REASON_SCHEMA_VERSION, 1);
    assert_eq!(Reason::all_variants().len(), 9);
    let mut seen = Vec::new();
    for variant in Reason::all_variants() {
        assert!(
            !seen.contains(variant),
            "all_variants must not repeat a variant: {variant:?}"
        );
        seen.push(*variant);
    }
}

// Requirements: CAP-003
//   Every domain protection ground maps into this vocabulary, both ways
//   exhaustive: a refusal ground added to the domain fails compilation
//   in the From impl, and this enumeration is the coverage record.
// Evidence: every_protection_ground_maps_into_the_vocabulary
#[test]
fn every_protection_ground_maps_into_the_vocabulary() {
    let refusals = [
        RefusalGround::Zfs,
        RefusalGround::StorageSpaces,
        RefusalGround::Ldm,
        RefusalGround::Fusion,
        RefusalGround::RemoteTransport,
        RefusalGround::InheritedFromConsumerOrProducer,
        RefusalGround::InheritedDeviceScope,
    ];
    let mapped: Vec<ProtectionRefusalReason> =
        refusals.iter().map(|ground| (*ground).into()).collect();
    for pair in mapped.windows(2) {
        assert_ne!(pair[0], pair[1], "distinct grounds must stay distinct");
    }
    assert_eq!(mapped.len(), 7);

    let causes = [
        IndeterminateGround::Unrecognized,
        IndeterminateGround::OrphanSignature,
        IndeterminateGround::CollisionGroup,
        IndeterminateGround::MissingFact,
        IndeterminateGround::InheritedDeviceScope,
    ];
    let mapped: Vec<ProtectionIndeterminacyReason> =
        causes.iter().map(|cause| (*cause).into()).collect();
    assert_eq!(mapped.len(), 5);
}

// Requirements: CAP-003, CAP-007
//   The decided protection coupling holds: a closure refusal is
//   `unsupported`, an indeterminacy is `blocked` — 3g's rule carried
//   into CAP-003's vocabulary — and `Clear` produces no answer at all,
//   because protection not gating a pair says nothing about tools,
//   evidence, floors, or limits. No constructor accepts a status
//   contradicting the gate, which is the CAP-007 no-upgrade rule at the
//   type layer.
// Evidence: the_protection_coupling_is_the_decided_one
#[test]
fn the_protection_coupling_is_the_decided_one() {
    let refused = Capability::from_protection_gate(
        &ProtectionGate::Unsupported {
            ground: RefusalGround::Zfs,
        },
        Remediation::NoneExists,
    )
    .expect("a refusal is an answer");
    assert_eq!(refused.status(), Status::Unsupported);
    assert_eq!(
        refused.reason(),
        Reason::ProtectionRefused {
            ground: ProtectionRefusalReason::Zfs
        }
    );

    let blocked = Capability::from_protection_gate(
        &ProtectionGate::Blocked {
            cause: IndeterminateGround::MissingFact,
        },
        Remediation::Action("establish the missing fact through the helper contract".into()),
    )
    .expect("an indeterminacy is an answer");
    assert_eq!(blocked.status(), Status::Blocked);
    assert_eq!(
        blocked.reason(),
        Reason::ProtectionIndeterminate {
            cause: ProtectionIndeterminacyReason::MissingFact
        }
    );

    assert_eq!(
        Capability::from_protection_gate(&ProtectionGate::Clear, Remediation::NoneExists),
        None,
        "Clear is not an answer; the engine keeps composing"
    );
}

// Requirements: CAP-003
//   Every answer carries all three CAP-003 parts, and remediation is
//   caller-stated: the no-remedy case is an explicit value, never an
//   omitted field.
// Evidence: every_answer_carries_status_reason_and_remediation
#[test]
fn every_answer_carries_status_reason_and_remediation() {
    let preview = Capability::preview(Remediation::Action(
        "qualification evidence for this combination lands in docs/capabilities/".into(),
    ));
    assert_eq!(preview.status(), Status::Preview);
    assert_eq!(preview.reason(), Reason::UnqualifiedPendingEvidence);
    assert!(matches!(preview.remediation(), Remediation::Action(_)));

    let unsupported =
        Capability::unsupported(Reason::MultipathDetectionOnly, Remediation::NoneExists);
    assert_eq!(unsupported.status(), Status::Unsupported);
    assert_eq!(unsupported.reason(), Reason::MultipathDetectionOnly);
    assert_eq!(*unsupported.remediation(), Remediation::NoneExists);

    let blocked = Capability::blocked(
        Reason::ToolMissing,
        Remediation::Action("install the named tool from the platform's package source".into()),
    );
    assert_eq!(blocked.status(), Status::Blocked);
    assert_eq!(blocked.reason(), Reason::ToolMissing);
}

// Requirements: CAP-006, CAP-007
//   `supported` is unreachable in this increment: QualificationEvidence
//   has no constructor, so no test and no caller can produce the status —
//   the compile_fail doctest on the type is the proof, and this test
//   records the obligation's positive half: the constructor that will
//   exist must come from the increment-3 store, nowhere else.
// Evidence: supported_is_unreachable_without_stored_evidence
#[test]
fn supported_is_unreachable_without_stored_evidence() {
    // What CAN be proven at runtime: every reachable constructor yields a
    // non-Supported status. The Supported arm exists in the type so the
    // vocabulary is CAP-003-complete; reaching it requires evidence this
    // crate cannot mint.
    let reachable = [
        Capability::preview(Remediation::NoneExists).status(),
        Capability::unsupported(Reason::MultipathDetectionOnly, Remediation::NoneExists).status(),
        Capability::blocked(Reason::ToolMissing, Remediation::NoneExists).status(),
    ];
    assert!(
        reachable.iter().all(|status| *status != Status::Supported),
        "no reachable constructor may produce Supported"
    );
}

// Requirements: CAP-006, CAP-007
//   The evidence-built reason cannot be asserted through any assertive
//   constructor: handing it to `unsupported` or `blocked` panics, so a
//   caller cannot dress an unqualified answer as a qualified one.
// Evidence: the_evidence_reason_cannot_be_asserted
#[test]
#[should_panic(expected = "never asserted")]
fn the_evidence_reason_cannot_be_asserted() {
    let _ = Capability::blocked(Reason::QualifiedByEvidence, Remediation::NoneExists);
}
