//! Increment 4a's Tier-1 suite: the journal-borne apply to the
//! authorization boundary — the journal open over injected seams, the
//! backward-clock bound over its high-water instant, the durable
//! validation store and its consumption, S2's two phases, CONC-003 on
//! its published edge, CONC-004's computed flag, journal-query, and the
//! torn-tail recovery of a lifecycle — all pure, all platforms (the
//! evidence rule: structural properties over authored inputs; the real
//! file seam's platform truth is the Tier-2 acceptance's).

use std::collections::BTreeMap;

use partman_capability::engine::{RuntimeFacts, TechnologyLimits};
use partman_domain::canonical::Value;
use partman_domain::model::capability::Operation as CapOp;
use partman_domain::model::identity::TableState;
use partman_domain::model::naming::{NamingFields, NodeId, derive_id};
use partman_domain::model::protection::{Facts, HostRange, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_journal::records::{
    AuthorizationTier, PlanHashRef, Record, RecordedInstant, TransitionRecord,
};
use partman_journal::retention::decode_journal;
use partman_journal::{DurabilityRefused, DurabilitySeam, Journal};
use partman_statemachine::Transition;

use crate::apply::{
    APPLY_SUBMITTED, ApplyAnswer, ApplyCore, DECLINED_OR_EXPIRED, EDIT_OR_INVALIDATION, apply_plan,
    clock_bound, high_water_instant, journal_query, note_validation, transitional_now,
};
use crate::authorize::{Ceremony, CeremonyCompleted, CeremonyUnavailable, RefusingCeremony};
use crate::validate::{ValidateRequest, ValidatedPlan, validate_plan};
use crate::{Operation, Request, RequestRefusal, Response};

const NOW: u64 = 1_700_000_000;
const UID: u32 = 1000;

/// The Tier-1 "disk": collects what the seam made durable, so a restart
/// recovers from exactly what would have survived a crash — never from
/// the core's in-memory bytes.
#[derive(Default)]
struct DiskSeam {
    disk: Vec<u8>,
    refuse: bool,
}

impl DurabilitySeam for DiskSeam {
    fn make_durable(&mut self, new_bytes: &[u8]) -> Result<(), DurabilityRefused> {
        if self.refuse {
            return Err(DurabilityRefused {
                reason: "the test seam refuses".to_owned(),
            });
        }
        self.disk.extend_from_slice(new_bytes);
        Ok(())
    }
}

/// A capture carrying one plannable device, and its address. The table
/// checksum parameter lets a test author "the world moved" (a different
/// snapshot hash) without touching anything else.
fn device_snapshot(serial: &[u8], checksum: u8) -> (TopologySnapshot, NodeId) {
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
        .insert(id, TableState::present([checksum; 32]));
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![device], vec![], facts)
            .expect("assembles");
    (snapshot, id)
}

/// Validate a wipe (severity Destructive — the interactive tier) over
/// the given capture.
fn validated_wipe(snapshot: &TopologySnapshot, device: NodeId) -> ValidatedPlan {
    validate_plan(
        snapshot,
        &ValidateRequest {
            target: device,
            operation: CapOp::Wipe,
            plan_id: b"inc4a".to_vec(),
            validity_seconds: 3600,
        },
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates")
}

/// One validated-and-noted plan over a fresh core: the state every
/// phase-one test starts from. Returns the pieces the tests drive.
fn noted(
    serial: &[u8],
) -> (
    ApplyCore,
    DiskSeam,
    DiskSeam,
    TopologySnapshot,
    ValidatedPlan,
) {
    let (snapshot, device) = device_snapshot(serial, 0x11);
    let validated = validated_wipe(&snapshot, device);
    let mut core = ApplyCore::new();
    let mut journal_seam = DiskSeam::default();
    let mut store_seam = DiskSeam::default();
    let journaled = note_validation(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &validated.body_hash,
        UID,
        AuthorizationTier::InteractiveCeremony,
        validated.not_after,
        &validated.body_bytes,
        NOW,
    )
    .expect("records durably");
    assert!(journaled, "a fresh lifecycle journals ValidatorPasses");
    (core, journal_seam, store_seam, snapshot, validated)
}

fn hash_of(validated: &ValidatedPlan) -> [u8; 32] {
    *validated.body_hash.as_bytes()
}

/// A ceremony that completes — constructible only under `cfg(test)` —
/// so the no-grant boundary is proven even past a completion.
struct CompletingCeremony;

impl Ceremony for CompletingCeremony {
    fn perform(
        &self,
        _plan: PlanHashRef,
        _client_uid: u32,
    ) -> Result<CeremonyCompleted, CeremonyUnavailable> {
        Ok(CeremonyCompleted::for_test())
    }
}

// Requirements: JRN-002, HLP-004, PLAN-007
//   The validation is journaled at validation — ValidatorPasses with
//   schema v2's recorded instant, durable before the answer — and the
//   backward-clock bound stands on it: a presentation whose clock reads
//   below the journal's high-water instant refuses, which is exactly
//   the validation-to-presentation window clock.rs named as this
//   increment's debt. A re-validation of an already-Validated plan
//   journals no second row: the chain's own from-discipline governs.
// Evidence: validation_is_journaled_with_its_instant_and_the_clock_bound_holds
#[test]
fn validation_is_journaled_with_its_instant_and_the_clock_bound_holds() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"CLOCK-4A");
    let decoded = core.decoded().expect("decodes");
    let transitions: Vec<_> = decoded
        .records()
        .iter()
        .filter_map(|(_, record)| match record {
            Record::Transition(t) => Some(t),
            _ => None,
        })
        .collect();
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].transition(), Transition::ValidatorPasses);
    assert_eq!(transitions[0].instant(), RecordedInstant::from_secs(NOW));
    assert_eq!(high_water_instant(&decoded), Some(NOW));
    assert_eq!(clock_bound(NOW, Some(NOW)), Ok(()));
    assert_eq!(clock_bound(NOW - 1, Some(NOW)), Err((NOW - 1, NOW)));

    // The bound refuses the presentation itself: a clock stepped back
    // between validation and presentation dates nothing.
    let decision = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash_of(&validated),
        &snapshot,
        NOW - 1,
        UID,
        &RefusingCeremony,
    );
    assert!(decision.journaled.is_empty());
    match decision.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(arm, "clock-behind-journal"),
        answer @ ApplyAnswer::Awaiting { .. } => panic!("a backward clock must refuse: {answer:?}"),
    }

    // Idempotent against the chain: re-noting the same validated plan
    // appends no second ValidatorPasses row.
    let again = note_validation(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &validated.body_hash,
        UID,
        AuthorizationTier::InteractiveCeremony,
        validated.not_after,
        &validated.body_bytes,
        NOW,
    )
    .expect("records");
    assert!(!again, "an already-Validated plan gets no second row");
}

// Requirements: SEC-002, JRN-002, ADR-0028
//   Phase one: the presentation passes SEC-002's arms against the fresh
//   capture, the validation's consumption is committed to the store
//   before ApplySubmitted is committed to the journal (the fail-closed
//   order), and the answer names the helper-computed tier. Both
//   consumption and submission survive a restart from the seams' disks
//   — the durable store, not the in-memory flag.
// Evidence: phase_one_consumes_durably_then_journals_apply_submitted
#[test]
fn phase_one_consumes_durably_then_journals_apply_submitted() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"PHASE-1");
    let decision = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash_of(&validated),
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    assert_eq!(decision.journaled, vec![APPLY_SUBMITTED]);
    match decision.answer {
        ApplyAnswer::Awaiting {
            plan_hash,
            tier,
            not_after,
        } => {
            assert_eq!(plan_hash, hash_of(&validated));
            assert_eq!(tier, AuthorizationTier::InteractiveCeremony);
            assert_eq!(not_after, validated.not_after);
        }
        answer @ ApplyAnswer::Refused { .. } => {
            panic!("phase one answers awaiting-authorization: {answer:?}")
        }
    }

    // The restart reads what the seams made durable — a crash after the
    // answer finds the journal ahead of nothing.
    let restarted = ApplyCore::recover(&journal_seam.disk, &store_seam.disk).expect("recovers");
    let record = restarted
        .validation(&hash_of(&validated))
        .expect("the store survives");
    assert!(record.consumed, "consumption is a store entry, durable");
    let decoded = restarted.decoded().expect("decodes");
    let last = decoded.records().last().expect("records");
    match &last.1 {
        Record::Transition(t) => assert_eq!(t.transition(), Transition::ApplySubmitted),
        other => panic!("the last record is the submission: {other:?}"),
    }
}

// Requirements: SEC-002, ADR-0028, Section 8
//   One validation, one submission — held across the published terminal
//   and across a restart: the window closing while awaiting terminates
//   on DeclinedOrExpired → Cancelled (NoWrites, the row's constraint),
//   a third presentation refuses replayed from the durable store, a
//   restart changes none of it, and a fresh validation supersedes the
//   spent record for a fresh lifecycle.
// Evidence: one_validation_admits_one_submission_and_replay_survives_restart
#[test]
#[allow(clippy::too_many_lines)]
fn one_validation_admits_one_submission_and_replay_survives_restart() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"REPLAY-4A");
    let hash = hash_of(&validated);
    let submit = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    assert!(matches!(submit.answer, ApplyAnswer::Awaiting { .. }));

    // The window closes while awaiting: the published terminal edge.
    let expired = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        validated.not_after + 1,
        UID,
        &RefusingCeremony,
    );
    assert_eq!(expired.journaled, vec![DECLINED_OR_EXPIRED]);
    match &expired.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "declined-or-expired"),
        answer @ ApplyAnswer::Awaiting { .. } => {
            panic!("expiry terminates on the published edge: {answer:?}")
        }
    }
    let decoded = core.decoded().expect("decodes");
    let last = decoded.records().last().expect("records");
    match &last.1 {
        Record::Transition(t) => {
            assert_eq!(t.transition(), Transition::DeclinedOrExpired);
            assert_eq!(
                t.effect(),
                Some(partman_statemachine::Effect::NoWrites),
                "the row's constraint"
            );
        }
        other => panic!("the terminal is journaled: {other:?}"),
    }

    // A third presentation is a replay — the validation's one
    // submission is spent, and says so.
    let replay = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        validated.not_after + 2,
        UID,
        &RefusingCeremony,
    );
    match &replay.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "replayed"),
        answer @ ApplyAnswer::Awaiting { .. } => panic!("a spent validation replays: {answer:?}"),
    }

    // Across a restart, from the durable bytes alone.
    let mut restarted = ApplyCore::recover(&journal_seam.disk, &store_seam.disk).expect("recovers");
    let replay = apply_plan(
        &mut restarted,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        validated.not_after + 3,
        UID,
        &RefusingCeremony,
    );
    match &replay.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "replayed"),
        answer @ ApplyAnswer::Awaiting { .. } => panic!("the replay arm is durable: {answer:?}"),
    }

    // The route back is a re-validation — which re-plans, so the fresh
    // body carries a fresh window and a fresh hash, and its own
    // lifecycle submits. The spent record stays spent; nothing revives
    // it.
    let (snapshot_again, device) = device_snapshot(b"REPLAY-4A", 0x11);
    let later = validated.not_after + 4;
    let fresh_validated = validate_plan(
        &snapshot_again,
        &ValidateRequest {
            target: device,
            operation: CapOp::Wipe,
            plan_id: b"inc4a".to_vec(),
            validity_seconds: 3600,
        },
        later,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    assert_ne!(
        hash_of(&fresh_validated),
        hash,
        "a re-validation is a fresh plan with a fresh hash"
    );
    let noted_again = note_validation(
        &mut restarted,
        &mut journal_seam,
        &mut store_seam,
        &fresh_validated.body_hash,
        UID,
        AuthorizationTier::InteractiveCeremony,
        fresh_validated.not_after,
        &fresh_validated.body_bytes,
        later,
    )
    .expect("records");
    assert!(noted_again, "the fresh plan opens its own lifecycle");
    let fresh = apply_plan(
        &mut restarted,
        &mut journal_seam,
        &mut store_seam,
        &hash_of(&fresh_validated),
        &snapshot_again,
        later + 1,
        UID,
        &RefusingCeremony,
    );
    assert!(
        matches!(fresh.answer, ApplyAnswer::Awaiting { .. }),
        "the fresh validation submits: {fresh:?}"
    );
}

// Requirements: JRN-002, SEC-002
//   The answer never runs ahead of the record: a journal seam that
//   cannot establish durability refuses phase one, the durable disk
//   holds no ApplySubmitted, and the consumed store entry (committed
//   first, the fail-closed order) costs the client exactly a
//   re-validation — never a second submission from one validation.
// Evidence: a_refusing_seam_refuses_the_answer_and_the_disk_stays_behind
#[test]
fn a_refusing_seam_refuses_the_answer_and_the_disk_stays_behind() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"SEAM-4A");
    journal_seam.refuse = true;
    let decision = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash_of(&validated),
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    match &decision.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "durability"),
        answer @ ApplyAnswer::Awaiting { .. } => panic!("no durability, no answer: {answer:?}"),
    }
    // What a crash would find: consumption durable, submission absent.
    let restarted = ApplyCore::recover(&journal_seam.disk, &store_seam.disk).expect("recovers");
    assert!(
        restarted
            .validation(&hash_of(&validated))
            .expect("stored")
            .consumed,
        "the consume-first order"
    );
    let decoded = restarted.decoded().expect("decodes");
    assert!(
        decoded.records().iter().all(|(_, record)| !matches!(
            record,
            Record::Transition(t) if t.transition() == Transition::ApplySubmitted
        )),
        "the refused submission never reached the disk"
    );
}

// Requirements: HLP-003, ADR-0021
//   Phase two refuses exactly where increment 3 refuses: the interactive
//   tier dies on the ceremony's own arm with its own sentence, and no
//   grant exists past it on this build — a completed ceremony (test-only
//   constructible) and the floor tier alike land on grant-not-served,
//   because AuthorizationGranted and everything after it are 4b's.
// Evidence: phase_two_refuses_where_increment_3_refuses_and_no_grant_exists
#[test]
fn phase_two_refuses_where_increment_3_refuses_and_no_grant_exists() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"PHASE-2");
    let hash = hash_of(&validated);
    let submit = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    assert!(matches!(submit.answer, ApplyAnswer::Awaiting { .. }));

    // The shipped ceremony refuses — increment 3's arm, verbatim.
    let second = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 20,
        UID,
        &RefusingCeremony,
    );
    assert!(second.journaled.is_empty(), "a refusal journals nothing");
    match &second.answer {
        ApplyAnswer::Refused { arm, detail } => {
            assert_eq!(*arm, "ceremony-unavailable");
            assert_eq!(*detail, CeremonyUnavailable::NoInteractiveRoute.to_string());
        }
        answer @ ApplyAnswer::Awaiting { .. } => panic!("the ceremony's own arm: {answer:?}"),
    }

    // Even a completed ceremony reaches no grant: the edge is 4b's.
    let completed = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 30,
        UID,
        &CompletingCeremony,
    );
    match &completed.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "grant-not-served"),
        answer @ ApplyAnswer::Awaiting { .. } => {
            panic!("no grant exists on this build: {answer:?}")
        }
    }

    // The floor tier lands on the same boundary. (Unreachable over this
    // build's wire — the sized-create pin — so the store is authored.)
    let (floor_core_snapshot, floor_device) = device_snapshot(b"FLOOR-4A", 0x11);
    let floor_validated = validated_wipe(&floor_core_snapshot, floor_device);
    let mut floor_core = ApplyCore::new();
    let mut floor_journal = DiskSeam::default();
    let mut floor_store = DiskSeam::default();
    note_validation(
        &mut floor_core,
        &mut floor_journal,
        &mut floor_store,
        &floor_validated.body_hash,
        UID,
        AuthorizationTier::FloorAct,
        floor_validated.not_after,
        &floor_validated.body_bytes,
        NOW,
    )
    .expect("records");
    let floor_hash = hash_of(&floor_validated);
    let submit = apply_plan(
        &mut floor_core,
        &mut floor_journal,
        &mut floor_store,
        &floor_hash,
        &floor_core_snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    match submit.answer {
        ApplyAnswer::Awaiting { tier, .. } => {
            // The tier on the answer is recomputed from the admitted
            // plan (a wipe is interactive), not read from the store —
            // the store's word feeds phase two only.
            assert_eq!(tier, AuthorizationTier::InteractiveCeremony);
        }
        answer @ ApplyAnswer::Refused { .. } => panic!("submits: {answer:?}"),
    }
    let floor_phase_two = apply_plan(
        &mut floor_core,
        &mut floor_journal,
        &mut floor_store,
        &floor_hash,
        &floor_core_snapshot,
        NOW + 20,
        UID,
        &RefusingCeremony,
    );
    match &floor_phase_two.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "grant-not-served"),
        answer @ ApplyAnswer::Awaiting { .. } => {
            panic!("the floor tier consumes nothing on this build: {answer:?}")
        }
    }
}

// Requirements: CONC-003, Section 8
//   External changes invalidate drafts on the published edge: a
//   presentation whose fresh capture contradicts the plan's bound
//   snapshot journals EditOrInvalidation (Validated → Draft) rather
//   than merely refusing, and a re-validation then opens a fresh row
//   from Draft.
// Evidence: a_stale_presentation_invalidates_on_the_published_edge
#[test]
fn a_stale_presentation_invalidates_on_the_published_edge() {
    let (mut core, mut journal_seam, mut store_seam, _snapshot, validated) = noted(b"STALE-4A");
    // The world moved: same device, different table checksum.
    let (moved, _) = device_snapshot(b"STALE-4A", 0x77);
    let decision = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash_of(&validated),
        &moved,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    assert_eq!(decision.journaled, vec![EDIT_OR_INVALIDATION]);
    match &decision.answer {
        ApplyAnswer::Refused { arm, .. } => assert_eq!(*arm, "stale"),
        answer @ ApplyAnswer::Awaiting { .. } => panic!("a stale presentation refuses: {answer:?}"),
    }
    let decoded = core.decoded().expect("decodes");
    let last = decoded.records().last().expect("records");
    match &last.1 {
        Record::Transition(t) => {
            assert_eq!(t.transition(), Transition::EditOrInvalidation);
        }
        other => panic!("the invalidation is journaled: {other:?}"),
    }
    // The validation was not consumed by a stale presentation; the
    // record stays available to a fresh lifecycle after re-validation.
    assert!(!core.validation(&hash_of(&validated)).unwrap().consumed);
}

// Requirements: CONC-004
//   Discovery during execution is transitional, computed from the
//   journal: a journaled lifecycle standing past the authorization
//   boundary makes the predicate true, everything at or before the
//   boundary leaves it false — and the flag reaches the snapshot's body
//   hash through the real capture, so a capture taken with an apply in
//   flight is hash-distinct from the same topology captured with none.
// Evidence: a_capture_with_an_apply_in_flight_is_hash_distinct
#[test]
fn a_capture_with_an_apply_in_flight_is_hash_distinct() {
    // The predicate over authored journals. Records past the boundary
    // are authored directly — 4a's own functions cannot produce them,
    // which is the boundary held by construction.
    let plan = PlanHashRef::from_bytes([0x55; 32]);
    let instant = RecordedInstant::from_secs(NOW);
    let mut journal = Journal::new();
    let append = |journal: &mut Journal, transition: Transition| {
        let record = TransitionRecord::non_terminal(plan, transition, instant).expect("row");
        journal
            .append(&Record::Transition(record).encode().expect("encodes"))
            .expect("bounded");
    };
    append(&mut journal, Transition::ValidatorPasses);
    append(&mut journal, Transition::ApplySubmitted);
    let decoded = decode_journal(journal.bytes()).expect("decodes");
    assert!(
        !transitional_now(&decoded),
        "at the boundary nothing is mid-apply"
    );
    append(&mut journal, Transition::AuthorizationGranted);
    append(&mut journal, Transition::RevalidationPasses);
    append(&mut journal, Transition::BackupsVerified);
    let decoded = decode_journal(journal.bytes()).expect("decodes");
    assert!(
        transitional_now(&decoded),
        "an apply in Executing is mid-apply"
    );

    // The flag's journey through the real capture: same fixture, both
    // values, distinct body hashes — a hard-coded value inside capture
    // cannot pass this.
    let settled = super::increment2::capture_with_flag(false);
    let transitional = super::increment2::capture_with_flag(true);
    assert_ne!(
        settled.snapshot_hash, transitional.snapshot_hash,
        "CONC-004: a mid-apply capture asserts less, hash-distinctly"
    );
}

// Requirements: JRN-006, HLP-001
//   journal-query is served from the decoded journal: every plan's last
//   journaled state under Section 8's own names, the high-water
//   instant, and the record count — all helper-authored.
// Evidence: journal_query_reports_states_and_the_high_water_instant
#[test]
fn journal_query_reports_states_and_the_high_water_instant() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"QUERY-4A");
    let hash = hash_of(&validated);
    let report = journal_query(&core.decoded().expect("decodes"));
    assert_eq!(report.records, 1);
    assert_eq!(report.high_water_instant, Some(NOW));
    assert_eq!(report.plans.len(), 1);
    assert_eq!(report.plans[0].plan_hash, hash);
    assert_eq!(report.plans[0].state, "Validated");

    let _ = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    let report = journal_query(&core.decoded().expect("decodes"));
    assert_eq!(report.plans[0].state, "AwaitingAuthorization");
    assert_eq!(report.high_water_instant, Some(NOW + 10));

    let _ = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        validated.not_after + 1,
        UID,
        &RefusingCeremony,
    );
    let report = journal_query(&core.decoded().expect("decodes"));
    assert_eq!(report.plans[0].state, "Cancelled");
    assert_eq!(report.records, 3);
}

// Requirements: JRN-001, JRN-003
//   A torn tail — the crash shape — truncates and the lifecycle
//   continues from the durable prefix: the recovered journal still
//   places the apply where the surviving records say, and the next
//   append continues the sequence.
// Evidence: a_torn_tail_recovers_and_the_lifecycle_continues
#[test]
fn a_torn_tail_recovers_and_the_lifecycle_continues() {
    let (mut core, mut journal_seam, mut store_seam, snapshot, validated) = noted(b"TORN-4A");
    let hash = hash_of(&validated);
    let _ = apply_plan(
        &mut core,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        NOW + 10,
        UID,
        &RefusingCeremony,
    );
    // The crash: the disk holds both records plus nine garbage bytes of
    // a frame that never finished.
    let mut torn = journal_seam.disk.clone();
    torn.extend_from_slice(&[0xAA; 9]);
    let mut restarted = ApplyCore::recover(&torn, &store_seam.disk).expect("recovers");
    let report = journal_query(&restarted.decoded().expect("decodes"));
    assert_eq!(report.plans[0].state, "AwaitingAuthorization");
    assert_eq!(report.records, 2, "the torn tail is dropped, nothing else");

    // The lifecycle continues where the survivors end.
    let expired = apply_plan(
        &mut restarted,
        &mut journal_seam,
        &mut store_seam,
        &hash,
        &snapshot,
        validated.not_after + 1,
        UID,
        &RefusingCeremony,
    );
    assert_eq!(expired.journaled, vec![DECLINED_OR_EXPIRED]);
    let report = journal_query(&restarted.decoded().expect("decodes"));
    assert_eq!(report.plans[0].state, "Cancelled");
}

// Requirements: HLP-001, RPC-003
//   The apply wire is strict in both directions: plan_hash travels with
//   exactly apply-plan, exactly 32 bytes, refused missing, mistyped,
//   wrong-sized or out of place; and the new outcomes render their
//   closed shapes.
// Evidence: the_apply_wire_is_strict_and_the_new_outcomes_encode
#[test]
fn the_apply_wire_is_strict_and_the_new_outcomes_encode() {
    use partman_domain::canonical;

    let good = Request {
        operation: Operation::ApplyPlan,
        validate: None,
        apply: Some(crate::ApplyWire {
            plan_hash: [0x11; 32],
        }),
    };
    let bytes = good.encode().expect("encodes");
    assert_eq!(Request::decode(&bytes).expect("decodes"), good);

    let tamper = |edit: &dyn Fn(&mut BTreeMap<String, Value>)| -> Result<Request, RequestRefusal> {
        let value = canonical::decode(&bytes).expect("canonical");
        let Value::Map(mut map) = value else {
            panic!("a request is a map");
        };
        edit(&mut map);
        Request::decode(&canonical::encode(&Value::Map(map)).expect("encodes"))
    };

    assert_eq!(
        tamper(&|map| {
            map.remove("plan_hash");
        }),
        Err(RequestRefusal::MissingField { key: "plan_hash" })
    );
    assert_eq!(
        tamper(&|map| {
            map.insert("plan_hash".to_owned(), Value::Bytes(vec![0x11; 31]));
        }),
        Err(RequestRefusal::BadField { key: "plan_hash" })
    );
    assert_eq!(
        tamper(&|map| {
            map.insert("plan_hash".to_owned(), Value::Text("11".repeat(32)));
        }),
        Err(RequestRefusal::BadField { key: "plan_hash" })
    );
    assert_eq!(
        tamper(&|map| {
            map.insert("operation".to_owned(), Value::Text("status".to_owned()));
        }),
        Err(RequestRefusal::FieldOutOfPlace { key: "plan_hash" }),
        "the apply argument travels with exactly apply-plan"
    );

    let awaiting = Response::AwaitingAuthorization {
        plan_hash: vec![0x11; 32],
        tier: "interactive-ceremony".to_owned(),
        not_after: NOW + 3600,
    };
    let encoded = awaiting.encode().expect("encodes");
    let Value::Map(map) = canonical::decode(&encoded).expect("canonical") else {
        panic!("a response is a map");
    };
    assert_eq!(
        map.get("outcome"),
        Some(&Value::Text("awaiting-authorization".to_owned()))
    );
    assert_eq!(
        map.get("tier"),
        Some(&Value::Text("interactive-ceremony".to_owned()))
    );
    assert_eq!(map.get("not_after"), Some(&Value::Unsigned(NOW + 3600)));

    let report = Response::JournalReport {
        high_water_instant: Some(NOW),
        records: 3,
        plans: vec![crate::JournalPlanWire {
            plan_hash: vec![0x11; 32],
            state: "Cancelled".to_owned(),
            instant: NOW,
        }],
    };
    let encoded = report.encode().expect("encodes");
    let Value::Map(map) = canonical::decode(&encoded).expect("canonical") else {
        panic!("a response is a map");
    };
    assert_eq!(map.get("outcome"), Some(&Value::Text("journal".to_owned())));
    assert_eq!(map.get("records"), Some(&Value::Unsigned(3)));
    assert_eq!(map.get("high_water_instant"), Some(&Value::Unsigned(NOW)));

    let refused = Response::ApplyRefused {
        arm: "replayed".to_owned(),
        detail: "spent".to_owned(),
    };
    let encoded = refused.encode().expect("encodes");
    let Value::Map(map) = canonical::decode(&encoded).expect("canonical") else {
        panic!("a response is a map");
    };
    assert_eq!(
        map.get("outcome"),
        Some(&Value::Text("apply-refused".to_owned()))
    );
}
