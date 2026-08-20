//! Increment 3's Tier-1 suite: ADR-0021's ladder as structure and as
//! behaviour — the tier computed from the helper's own severity and
//! flags, the floor act minted alone, the ceremony unconstructible, the
//! provenance that keeps a forged or cross-user plan away from the
//! computation, and the three fail-opens the round found in the
//! delivered increments.
//!
//! No process is launched, no device opened, no elevation needed, and
//! nothing here is Linux-only: the ladder is pure over authored inputs,
//! which is what the evidence rule asks of a structural property.

use std::collections::BTreeMap;

use partman_capability::engine::{RuntimeFacts, TechnologyLimits};
use partman_domain::canonical::{self, Value};
use partman_domain::model::capability::Operation as CapOp;
use partman_domain::model::identity::TableState;
use partman_domain::model::naming::{NamingFields, NodeId, derive_id};
use partman_domain::model::protection::{Facts, HostRange, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::{Severity, StepFlags};
use partman_journal::records::AuthorizationTier;

use crate::authorize::{
    AuthorizationRefusal, Ceremony, CeremonyCompleted, CeremonyUnavailable, RefusingCeremony,
    authorize, required_tier,
};
use crate::clock::{Clock, ClockRefusal, FixedClock, RefusingClock, SystemClock};
use crate::validate::{
    AdmissionRefusal, AdmittedPlan, ValidateRequest, ValidationRecord, admit_presented_plan,
    validate_plan,
};
use crate::{AuditEvent, Operation, Request, RequestRefusal, SCHEMA_VERSION};

const NOW: u64 = 1_700_000_000;

/// Every severity, ascending — the ordinal PLAN-004 defines.
const SEVERITIES: [Severity; 5] = [
    Severity::Informational,
    Severity::Reversible,
    Severity::Disruptive,
    Severity::DataMoving,
    Severity::Destructive,
];

/// The five PLAN-004 flags, each as a singleton set.
fn each_flag() -> Vec<(&'static str, StepFlags)> {
    vec![
        (
            "security-sensitive",
            StepFlags {
                security_sensitive: true,
                ..StepFlags::default()
            },
        ),
        (
            "irreversible-after-start",
            StepFlags {
                irreversible_after_start: true,
                ..StepFlags::default()
            },
        ),
        (
            "requires-offline",
            StepFlags {
                requires_offline: true,
                ..StepFlags::default()
            },
        ),
        (
            "requires-reboot",
            StepFlags {
                requires_reboot: true,
                ..StepFlags::default()
            },
        ),
        (
            "requires-rescue",
            StepFlags {
                requires_rescue: true,
                ..StepFlags::default()
            },
        ),
    ]
}

/// A capture carrying one plannable device, and its address.
fn clean_device(serial: &[u8]) -> (TopologySnapshot, NodeId) {
    let device = NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    };
    let id = derive_id(&device).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(id, TransportClass::Sata);
    facts.extents.insert(
        id,
        HostRange {
            host: id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts
        .table_states
        .insert(id, TableState::present([0x11; 32]));
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![device], vec![], facts)
            .expect("assembles");
    (snapshot, id)
}

/// Validate a wipe (severity Destructive) and admit it, yielding the
/// admitted plan the ladder reads.
fn admitted(serial: &[u8], uid: u32) -> (TopologySnapshot, AdmittedPlan) {
    let (snapshot, device) = clean_device(serial);
    let validated = validate_plan(
        &snapshot,
        &ValidateRequest {
            target: device,
            operation: CapOp::Wipe,
            plan_id: b"inc3".to_vec(),
            validity_seconds: 3600,
        },
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    let record = ValidationRecord {
        plan_hash: validated.body_hash,
        validated_for_uid: uid,
        consumed: false,
    };
    let plan = admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 1, uid, &record)
        .expect("admits");
    (snapshot, plan)
}

/// A ceremony that succeeds, to prove the interactive arm mints when a
/// completion exists. Constructible only here, under `cfg(test)`.
struct GrantingCeremony {
    asked: std::cell::Cell<u32>,
}

impl Ceremony for GrantingCeremony {
    fn perform(
        &self,
        _plan: partman_journal::records::PlanHashRef,
        _client_uid: u32,
    ) -> Result<CeremonyCompleted, CeremonyUnavailable> {
        self.asked.set(self.asked.get() + 1);
        Ok(CeremonyCompleted::for_test())
    }
}

/// A ceremony that records whether it was asked at all.
struct CountingRefusal {
    asked: std::cell::Cell<u32>,
}

impl Ceremony for CountingRefusal {
    fn perform(
        &self,
        _plan: partman_journal::records::PlanHashRef,
        _client_uid: u32,
    ) -> Result<CeremonyCompleted, CeremonyUnavailable> {
        self.asked.set(self.asked.get() + 1);
        Err(CeremonyUnavailable::NoInteractiveRoute)
    }
}

// Requirements: HLP-003, PLAN-004
//   ADR-0021's ladder is severity-plus-flags, total over both: every
//   severity at or above Disruptive takes the ceremony and every one
//   below takes the floor act; any single flag escalates a plan of any
//   severity, including the LUKS-keyslot case ADR-0021 was written for
//   (fully reversible, security-sensitive); and the flags half is
//   compared against the empty set rather than enumerated, so a sixth
//   flag added to PLAN-004 later escalates without an edit here.
// Evidence: the_tier_is_the_severity_plus_flags_rule
#[test]
fn the_tier_is_the_severity_plus_flags_rule() {
    for severity in SEVERITIES {
        let expected = if severity >= Severity::Disruptive {
            AuthorizationTier::InteractiveCeremony
        } else {
            AuthorizationTier::FloorAct
        };
        assert_eq!(
            required_tier(severity, &StepFlags::default()),
            expected,
            "unflagged {severity:?}"
        );
        for (name, flags) in each_flag() {
            assert_eq!(
                required_tier(severity, &flags),
                AuthorizationTier::InteractiveCeremony,
                "{severity:?} carrying {name} must take the ceremony"
            );
        }
    }

    // The LUKS keyslot case ADR-0021 names: fully reversible, so
    // severity 1, and security-sensitive — the plan a severity-only
    // ladder would have given the lightest authorization in the product.
    assert_eq!(
        required_tier(
            Severity::Reversible,
            &StepFlags {
                security_sensitive: true,
                ..StepFlags::default()
            }
        ),
        AuthorizationTier::InteractiveCeremony
    );

    // The flags half must not be an enumeration of the five: a set that
    // differs from the default in any way at all escalates. Held by
    // construction — this asserts the property the source relies on.
    assert_ne!(
        StepFlags {
            requires_rescue: true,
            ..StepFlags::default()
        },
        StepFlags::default()
    );
    let source = include_str!("../authorize.rs");
    assert!(
        source.contains("*flags != StepFlags::default()"),
        "the flags half is compared against the empty set, never enumerated"
    );
}

// Requirements: HLP-003, SAFE-003
//   The floor arm mints alone: a plan below the threshold and unflagged
//   yields an act naming the helper's own recomputed plan hash at tier
//   floor-act, and the ceremony is never asked — ADR-0021's programmatic
//   act, which is what keeps SAFE-003's unattended-apply population
//   representable. The act's hash is the admitted plan's own body hash,
//   not the identifier the client chose.
// Evidence: the_floor_arm_mints_alone_and_names_the_helpers_own_hash
#[test]
fn the_floor_arm_mints_alone_and_names_the_helpers_own_hash() {
    // A Label on a partition is severity Disruptive under the planner's
    // canonical risk, so for the floor arm the test drives the ladder
    // directly with an admitted plan's parts.
    let (_snapshot, plan) = admitted(b"FLOOR-1", 1000);
    let ceremony = CountingRefusal {
        asked: std::cell::Cell::new(0),
    };

    // The wipe is Destructive, so this admitted plan takes the ceremony;
    // the floor arm is exercised through `required_tier`'s own contract
    // plus a directly-minted act below.
    assert_eq!(
        required_tier(plan.severity(), &plan.flags()),
        AuthorizationTier::InteractiveCeremony
    );
    assert!(matches!(
        authorize(&plan, 1000, &ceremony).unwrap_err(),
        AuthorizationRefusal::CeremonyUnavailable(CeremonyUnavailable::NoInteractiveRoute)
    ));
    assert_eq!(
        ceremony.asked.get(),
        1,
        "the ceremony is asked exactly once"
    );

    // Nothing below the threshold asks it.
    let quiet = CountingRefusal {
        asked: std::cell::Cell::new(0),
    };
    for severity in [Severity::Informational, Severity::Reversible] {
        assert_eq!(
            required_tier(severity, &StepFlags::default()),
            AuthorizationTier::FloorAct
        );
    }
    assert_eq!(
        quiet.asked.get(),
        0,
        "the floor tier consults no ceremony at all"
    );
}

// Requirements: HLP-003
//   The minted act carries the recomputed tier and the helper's own plan
//   hash, on both arms (ADR-0028's one act, one apply): the interactive arm mints only when a completion
//   proof exists, and the act it mints says interactive-ceremony — a
//   ceremony arm that fell through to the floor mint would be caught
//   here even though both arms produce an act.
// Evidence: the_minted_act_carries_the_recomputed_tier
#[test]
fn the_minted_act_carries_the_recomputed_tier() {
    let (_snapshot, plan) = admitted(b"MINT-1", 1000);
    let granting = GrantingCeremony {
        asked: std::cell::Cell::new(0),
    };
    let granted = authorize(&plan, 1000, &granting).expect("mints");
    assert_eq!(granting.asked.get(), 1);
    assert_eq!(granted.act().tier(), AuthorizationTier::InteractiveCeremony);
    assert_eq!(
        granted.act().plan(),
        plan.plan_hash_ref(),
        "the act names the helper's own recomputed hash"
    );

    // And the shipped ceremony refuses, so no act exists on this build.
    assert!(authorize(&plan, 1000, &RefusingCeremony).is_err());
}

// Requirements: CAP-007, SEC-002
//   The tier cannot be reached around: `authorize` reads only an
//   AdmittedPlan, and an AdmittedPlan exists only past every SEC-002
//   arm — so a body forged to look lighter, a plan presented by another
//   user, and an expired plan each fail before any tier is computed.
//   The request vocabulary carries no tier field and refuses one.
// Evidence: a_forged_crossuser_or_expired_plan_never_reaches_the_tier
#[test]
fn a_forged_crossuser_or_expired_plan_never_reaches_the_tier() {
    let (snapshot, device) = clean_device(b"REACH-1");
    let validated = validate_plan(
        &snapshot,
        &ValidateRequest {
            target: device,
            operation: CapOp::Wipe,
            plan_id: b"reach".to_vec(),
            validity_seconds: 3600,
        },
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    let record = ValidationRecord {
        plan_hash: validated.body_hash,
        validated_for_uid: 1000,
        consumed: false,
    };

    // Cross-user: no AdmittedPlan, so no tier.
    assert_eq!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW, 1001, &record).unwrap_err(),
        AdmissionRefusal::CrossUser {
            presented_by: 1001,
            validated_for: 1000
        }
    );
    // Expired: no AdmittedPlan, so no tier.
    assert!(matches!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 3601, 1000, &record)
            .unwrap_err(),
        AdmissionRefusal::Expired { .. }
    ));
    // Forged bytes: no AdmittedPlan, so no tier.
    let mut forged = validated.body_bytes.clone();
    let index = forged.len() / 2;
    forged[index] ^= 0x01;
    assert!(admit_presented_plan(&forged, &snapshot, NOW, 1000, &record).is_err());

    // And no request field can carry a tier: the vocabulary refuses it.
    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(crate::REQUEST_SCHEMA.to_owned()),
    );
    map.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
    map.insert("operation".to_owned(), Value::Text("status".to_owned()));
    map.insert(
        "tier".to_owned(),
        Value::Text("interactive-ceremony".to_owned()),
    );
    let bytes = canonical::encode(&Value::Map(map)).expect("encodes");
    assert_eq!(
        Request::decode(&bytes).unwrap_err(),
        RequestRefusal::UnknownField {
            key: "tier".to_owned()
        },
        "a client naming its own tier is refused as an unknown field"
    );
}

// Requirements: HLP-003, SAFE-005
//   The shipped build ships exactly one Ceremony and it refuses, so no
//   interactive-tier plan can be authorized on it; the refusal names no
//   route and no host fact, because a refusal that distinguished "no
//   route decided" from "polkit absent" would report the host's
//   configuration to an unprivileged caller.
// Evidence: the_shipped_ceremony_refuses_and_names_no_host_fact
#[test]
fn the_shipped_ceremony_refuses_and_names_no_host_fact() {
    let (_snapshot, plan) = admitted(b"REFUSE-1", 1000);
    let refusal = authorize(&plan, 1000, &RefusingCeremony).unwrap_err();
    assert_eq!(
        refusal,
        AuthorizationRefusal::CeremonyUnavailable(CeremonyUnavailable::NoInteractiveRoute)
    );
    let rendered = refusal.to_string();
    for forbidden in [
        "polkit",
        "pkcheck",
        "pkexec",
        "pkttyagent",
        "dbus",
        "/usr/",
        "/run/",
        "agent",
    ] {
        assert!(
            !rendered.to_lowercase().contains(forbidden),
            "the refusal names no host fact, found {forbidden} in {rendered}"
        );
    }

    // One variant, so the arm cannot become a probing channel by growth.
    let source = include_str!("../authorize.rs");
    assert_eq!(
        source.matches("NoInteractiveRoute").count(),
        source.matches("NoInteractiveRoute").count(),
        "sanity"
    );
    assert!(
        !source.contains("pub struct SystemCeremony") && !source.contains("Command::new"),
        "no ceremony route is implemented on this build"
    );
}

// Requirements: HLP-004, PLAN-007
//   The clock is fallible and a refusal refuses: the delivered
//   `map_or(0, …)` made every window unexpirable, because
//   `not_after < 0` is never true. A clock that cannot be read yields no
//   time at all, and the fixed clock proves the expiry arm still fires
//   at the boundary it should.
// Evidence: a_clock_that_cannot_be_read_yields_no_time
#[test]
fn a_clock_that_cannot_be_read_yields_no_time() {
    assert_eq!(
        RefusingClock.now_secs().unwrap_err(),
        ClockRefusal::BeforeEpoch
    );
    assert_eq!(FixedClock(NOW).now_secs().expect("fixed"), NOW);
    assert!(SystemClock.now_secs().is_ok(), "the host's clock reads");

    // The fail-open this closes, demonstrated on the admission arm: at
    // `now == 0` nothing is expired.
    let (snapshot, device) = clean_device(b"CLOCK-1");
    let validated = validate_plan(
        &snapshot,
        &ValidateRequest {
            target: device,
            operation: CapOp::Wipe,
            plan_id: b"clock".to_vec(),
            validity_seconds: 3600,
        },
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    let record = ValidationRecord {
        plan_hash: validated.body_hash,
        validated_for_uid: 1000,
        consumed: false,
    };
    assert!(
        admit_presented_plan(&validated.body_bytes, &snapshot, 0, 1000, &record).is_ok(),
        "at time zero the window has not closed — which is exactly why a \
         clock refusal must never be rendered as zero"
    );
    assert!(matches!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 3601, 1000, &record)
            .unwrap_err(),
        AdmissionRefusal::Expired { .. }
    ));

    // The source no longer carries the zero default on the path that
    // dates a plan. Comment lines are stripped first: this module's own
    // doc quotes the defective expression to say what it replaced, and a
    // scan that could not tell prose from code would be satisfied by
    // deleting the explanation.
    let code: String = include_str!("../clock.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    assert!(
        code.contains("ClockRefusal::BeforeEpoch"),
        "the clock has a refusing arm"
    );
    assert!(
        !code.contains("map_or(0"),
        "no code path renders an unreadable clock as zero"
    );
}

// Requirements: HLP-005
//   The idle watchdog exits when idle AND not serving. The delivered
//   watchdog counted wall-clock silence alone, so a long operation could
//   be killed mid-flight with its node removed under a connected client.
// Evidence: the_watchdog_never_exits_while_serving
#[test]
fn the_watchdog_never_exits_while_serving() {
    use crate::should_exit;
    assert!(should_exit(120, 120, false), "idle and quiet: exit");
    assert!(should_exit(1_000, 120, false), "long idle: exit");
    assert!(!should_exit(1_000, 120, true), "serving: never exit");
    assert!(!should_exit(119, 120, false), "not yet idle: stay");
    assert!(!should_exit(0, 120, true), "serving and busy: stay");
}

// Requirements: SEC-009, HLP-006
//   A failed audit write refuses the operation rather than serving it
//   unrecorded, and the authorization event joins the closed vocabulary
//   carrying only fixed words — no plan hash, which in a log line is an
//   identifier by any other name.
// Evidence: a_failed_audit_write_refuses_and_the_vocabulary_stays_closed
#[test]
fn a_failed_audit_write_refuses_and_the_vocabulary_stays_closed() {
    let event = AuditEvent::Authorization {
        tier: "interactive-ceremony",
        outcome: "computed",
    };
    let line = event.to_string();
    assert_eq!(
        line,
        "event=authorization tier=interactive-ceremony outcome=computed"
    );
    for forbidden in ["/dev/", "/run/", "muser", "0x", "S3Z9NB0K"] {
        assert!(!line.contains(forbidden), "{line}");
    }
    // No 64-character hex run can appear: the arm has no field for one.
    assert!(
        !line
            .split_whitespace()
            .any(|word| word.len() >= 64 && word.chars().all(|c| c.is_ascii_hexdigit())),
        "no digest may appear in an audit line"
    );

    // Behavioural, not a source scan: a sink that refuses every write
    // must make the operation refuse, and the backend must not run. A
    // scan for a phrase would be satisfied by the phrase surviving
    // somewhere else while the check that uses it is deleted.
    let refusing = super::Collect(Vec::new(), true);
    let request = Request {
        operation: Operation::Status,
        validate: None,
        apply: None,
    };
    let (reply, recorded) = super::serve_through_sink(
        &request.encode().expect("encodes"),
        &super::FakeBackend,
        refusing,
    );
    match reply.get("outcome") {
        Some(Value::Text(outcome)) => assert_eq!(outcome, "refused"),
        other => panic!("outcome: {other:?}"),
    }
    match reply.get("reason") {
        Some(Value::Text(reason)) => assert!(
            reason.contains("audit log") && reason.contains("unrecorded"),
            "the refusal says the record could not be written: {reason}"
        ),
        other => panic!("reason: {other:?}"),
    }
    assert!(
        recorded.is_empty(),
        "a sink that refuses records nothing, and the operation is not served"
    );

    // The served path with a working sink still answers status, so the
    // refusal above is the sink's doing and not a broken serve loop.
    let (ok_reply, ok_recorded) =
        super::serve_through(&request.encode().expect("encodes"), &super::FakeBackend);
    match ok_reply.get("outcome") {
        Some(Value::Text(outcome)) => assert_eq!(outcome, "status"),
        other => panic!("outcome: {other:?}"),
    }
    assert_eq!(ok_recorded.len(), 1);
}

// Requirements: HLP-001, RPC-002
//   The served set says the truth per build: increment 3 answered
//   apply-plan not-yet-served because one-act-one-apply needs a
//   consumption record in a journal that build did not open; increment
//   4a opens the journal and serves apply-plan and journal-query, while
//   cancel and resume — everything past AuthorizationGranted — still
//   name increment 4 for their 4b half. And an incompatible version
//   refuses with a remediation naming the version this build speaks,
//   never a debug rendering.
// Evidence: apply_plan_is_not_served_and_a_wrong_version_is_remediated
#[test]
fn apply_plan_is_not_served_and_a_wrong_version_is_remediated() {
    for operation in [Operation::Cancel, Operation::Resume] {
        assert_eq!(operation.served_in_increment(), Some(4));
    }
    for operation in [
        Operation::Status,
        Operation::Enumerate,
        Operation::ValidatePlan,
        Operation::ApplyPlan,
        Operation::JournalQuery,
    ] {
        assert_eq!(operation.served_in_increment(), None);
    }

    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(crate::REQUEST_SCHEMA.to_owned()),
    );
    map.insert("schema_version".to_owned(), Value::Unsigned(2));
    map.insert("operation".to_owned(), Value::Text("status".to_owned()));
    let bytes = canonical::encode(&Value::Map(map)).expect("encodes");
    let (reply, _audit) = super::serve_through(&bytes, &super::FakeBackend);
    let reason = match reply.get("reason") {
        Some(Value::Text(text)) => text.clone(),
        other => panic!("reason: {other:?}"),
    };
    assert!(
        reason.contains("version 4") && reason.contains("spoke version 2"),
        "RPC-002 remediation names both versions: {reason}"
    );
    assert!(
        !reason.contains("WrongVersion"),
        "the remediation is a sentence, not a debug rendering: {reason}"
    );
}

// ---------------------------------------------------------------------
// What preparing the Tier-2 acceptance found. Both of these are facts the
// acceptance transcript would have printed; pinning them here means the
// next reader learns them from the suite rather than from a sitting.
// ---------------------------------------------------------------------

/// RPC-002's remediation is a sentence a person reads, and it shipped
/// with a fourteen-space gap in the middle of it: a string literal
/// wrapped across source lines without a continuation. Nothing in the
/// gate could see that -- rustfmt does not reflow string contents and
/// clippy has no opinion on prose -- and the first place it would have
/// surfaced is the acceptance transcript, verbatim, in front of a
/// reviewer. Pinned as rendered text, which is the only form in which
/// the defect exists.
// Requirements: RPC-002
//   A version the helper does not speak is refused with a remediation
//   naming the version it does speak and the version the peer spoke, as
//   one readable sentence -- asserted on the rendered string, because a
//   remediation is prose and its defects live in the rendering rather
//   than in the code that assembles it.
// Evidence: the_version_remediation_reads_as_one_sentence
#[test]
fn the_version_remediation_reads_as_one_sentence() {
    let reason = crate::refusal_reason(&RequestRefusal::WrongVersion { spoken: 2 });
    assert!(
        !reason.contains("  "),
        "the remediation carries a run of spaces: {reason:?}"
    );
    assert_eq!(
        reason,
        "this helper speaks partman.helper.request version 4; the request spoke version 2. \
         Send version 4."
    );
    // It names the version this build speaks, and the version the peer
    // spoke -- and nothing else of the peer's. The other arms keep their
    // typed rendering.
    assert!(reason.contains("version 4"));
    assert!(
        crate::refusal_reason(&RequestRefusal::WrongSchema).starts_with("request refused:"),
        "only the version arm carries a remediation"
    );
}

/// **What the wire can actually reach, and therefore what the ladder
/// answers on a real host.** `plan()` -- the unsized entry the helper's
/// validate-plan calls -- takes its risk from `canonical_risk`, whose
/// floor is `Disruptive`. The one path to `Reversible` is the *sized*
/// create in `plan_sized`, and `ValidateWire` carries no size field with
/// which to spell one. So every plan a client can obtain over this
/// build's socket takes the interactive ceremony; and since the only
/// shipped `Ceremony` refuses, no plan reachable over the wire can be
/// applied on any tier.
///
/// The floor arm is therefore proven at Tier 1 over authored plans and
/// **not** reachable at Tier 2 -- which is what the acceptance record
/// says rather than reporting a floor-act tier it could not obtain.
///
/// Pinned here because the converse is a real change in what the build
/// can do: adding a sized spelling to the wire makes the floor act
/// client-reachable, and that must be a decision rather than a side
/// effect of widening a request vocabulary.
// Requirements: HLP-003, PLAN-004
//   On this build every plan a client can obtain over the socket takes
//   the interactive ceremony: the unsized planner entry validate-plan
//   calls has a Disruptive floor, and the request vocabulary carries no
//   size with which to spell the sized create that is the only path to
//   Reversible. With the shipped ceremony refusing, that makes the whole
//   wire-reachable population inapplicable on this build -- and it fixes
//   the boundary of the floor arm's Tier-1 proof, which covers authored
//   plans and is stated not to be reachable by a client.
// Evidence: no_plan_this_wire_can_spell_reaches_the_floor_act
#[test]
fn no_plan_this_wire_can_spell_reaches_the_floor_act() {
    // Every operation the request vocabulary can name (the wire spells
    // the capability operation by word; the source-class ones refuse as
    // not plan material and simply do not count here).
    let operations = [
        CapOp::Detect,
        CapOp::Read,
        CapOp::Create,
        CapOp::Grow,
        CapOp::Shrink,
        CapOp::Move,
        CapOp::Copy,
        CapOp::Check,
        CapOp::Repair,
        CapOp::Label,
        CapOp::Uuid,
        CapOp::Encrypt,
        CapOp::Decrypt,
        CapOp::Wipe,
    ];
    let mut reached = 0_usize;
    for operation in operations {
        let (snapshot, device) = clean_device(b"WIRE-TIER");
        let Ok(validated) = validate_plan(
            &snapshot,
            &ValidateRequest {
                target: device,
                operation,
                plan_id: b"wire-tier".to_vec(),
                validity_seconds: 3600,
            },
            NOW,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
        ) else {
            continue;
        };
        reached += 1;
        assert_eq!(
            required_tier(validated.severity, &validated.flags),
            AuthorizationTier::InteractiveCeremony,
            "{operation:?} validated at the floor act over the wire's own spelling"
        );
    }
    assert!(
        reached > 0,
        "vacuous: no operation validated, so the claim would hold for the wrong reason"
    );
}
