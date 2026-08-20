//! Increment 4's suite: obligations 9, 10, 11's derivation half, 12,
//! and 13's fixture — the liveness-scoped exemption with its linkage
//! closure, fail-closed budget exhaustion, covered ranges derived from
//! durable compaction records alone, monotonicity across compaction,
//! and the ADR-0028-shaped chain trace over a journal compacted around
//! the live apply.

use partman_statemachine::{Effect, Transition};

use super::{
    BudgetExhausted, CompactedRefused, compact, decode_journal, ledger, over_budget,
    over_budget_against,
};
use crate::records::{
    AuthorizationAct, AuthorizationTier, Checkpoint, DisposalLinkage, PlanHashRef, Record,
    RecordedInstant, TransitionRecord,
};
use crate::{CoveredRanges, Journal, MIN_FRAME_LEN, ReplayRefused, SeqNo, replay};

fn plan(tag: u8) -> PlanHashRef {
    PlanHashRef::from_bytes([tag; 32])
}

fn act(target: PlanHashRef) -> Record {
    Record::AuthorizationAct(AuthorizationAct::new(target, AuthorizationTier::FloorAct))
}

/// The fixed instant this suite records transitions at; retention reads
/// liveness from the chain, not the clock.
fn instant_t() -> RecordedInstant {
    RecordedInstant::from_secs(1_700_000_000)
}

fn completed(target: PlanHashRef) -> Record {
    Record::Transition(
        TransitionRecord::terminal(
            target,
            Transition::PostconditionsPass,
            Effect::Complete,
            None,
            instant_t(),
        )
        .expect("terminal row"),
    )
}

fn failed_with_disposal(target: PlanHashRef, recovery: PlanHashRef) -> Record {
    Record::Transition(
        TransitionRecord::terminal(
            target,
            Transition::FailureAccepted,
            Effect::Partial,
            Some(DisposalLinkage::new(recovery)),
            instant_t(),
        )
        .expect("the disposal arm"),
    )
}

fn checkpoint(target: PlanHashRef, index: u64) -> Record {
    Record::Checkpoint(Checkpoint::new(target, index))
}

fn journal_of(records: &[Record]) -> Journal {
    let mut journal = Journal::new();
    for record in records {
        journal
            .append(&record.encode().expect("encodes"))
            .expect("bounded");
    }
    journal
}

/// Which plans own the records of a decoded journal, in order, with
/// `None` for infrastructure.
fn owners(decoded: &super::DecodedJournal) -> Vec<Option<PlanHashRef>> {
    decoded
        .records()
        .iter()
        .map(|(_, record)| super::record_plan(record))
        .collect()
}

// Requirements: JRN-004
//   Imported obligation 9, the exemption as a property rather than a
//   filter: a retention pass over a journal holding non-terminal
//   applies reclaims nothing of any live apply or its linkage closure
//   — a live apply's records survive whole, a terminal original whose
//   recovery plan is still live survives whole (the closure over
//   ADR-0027's linkage), a terminal whose disposal names a plan that
//   never started survives (an unstarted recovery is not a terminated
//   one, fail-closed toward retention), and a fully terminated chain
//   is reclaimed end to end. The reclaimable set is computed by the
//   pass itself from the journal's records; no API accepts a
//   caller-named range, and a double-terminal journal refuses rather
//   than guesses.
// Evidence: a_retention_pass_reclaims_nothing_live_or_in_the_linkage_closure
#[test]
fn a_retention_pass_reclaims_nothing_live_or_in_the_linkage_closure() {
    let done = plan(0xA1);
    let live = plan(0xB2);
    let orig_live = plan(0xC3);
    let rec_live = plan(0xD4);
    let orig_done = plan(0xE5);
    let rec_done = plan(0xF6);
    let orphaned = plan(0x97);
    // done: terminated, unlinked — reclaimable.
    // live: live — exempt.
    // orig_live: failed, recovered by rec_live; rec_live live — both
    //   exempt (the closure).
    // orig_done: failed, recovered by rec_done; rec_done terminated —
    //   chain over, both reclaimable.
    // orphaned: failed, recovered by a plan that never started — exempt.
    let journal = journal_of(&[
        act(done),
        completed(done),
        act(live),
        act(orig_live),
        failed_with_disposal(orig_live, rec_live),
        act(rec_live),
        act(orig_done),
        failed_with_disposal(orig_done, rec_done),
        act(rec_done),
        completed(rec_done),
        act(orphaned),
        failed_with_disposal(orphaned, plan(0x08)),
    ]);

    let decoded = decode_journal(journal.bytes()).expect("intact");
    let built = ledger(&decoded).expect("consistent");
    for exempt_plan in [live, orig_live, rec_live, orphaned] {
        assert!(built.exempt(exempt_plan), "{exempt_plan:?} must be exempt");
    }
    for reclaimable_plan in [done, orig_done, rec_done] {
        assert!(
            !built.exempt(reclaimable_plan),
            "{reclaimable_plan:?} has wholly terminated"
        );
    }

    let compacted = compact(&journal).expect("compactable");
    let after = decode_journal(compacted.journal.bytes()).expect("compacted journal decodes");
    let survivors = owners(&after);
    for kept in [live, orig_live, rec_live, orphaned] {
        assert!(
            survivors.iter().filter(|o| **o == Some(kept)).count() > 0,
            "every record of {kept:?} survives"
        );
    }
    for gone in [done, orig_done, rec_done] {
        assert_eq!(
            survivors.iter().filter(|o| **o == Some(gone)).count(),
            0,
            "no record of {gone:?} survives"
        );
    }
    // The exempt applies' record counts are unchanged, not merely
    // nonzero.
    assert_eq!(
        survivors.iter().filter(|o| o.is_some()).count(),
        6,
        "live(1) + orig_live(2) + rec_live(1) + orphaned(2) records survive; the six of done/orig_done/rec_done are gone"
    );

    let doubled = journal_of(&[act(done), completed(done), completed(done)]);
    assert!(
        matches!(
            compact(&doubled),
            Err(CompactedRefused::TerminalTwice { plan: p, .. }) if p == done
        ),
        "two terminals for one plan refuse rather than guess"
    );
}

// Requirements: JRN-004
//   Imported obligation 10: budget exhaustion is a journaled failure
//   through an existing Section 8 edge, and no code path reclaims a
//   live record, structurally. The per-apply spend equals the encoded
//   frame bytes attributed to each plan (infrastructure charged to
//   none), exhaustion against a stated bound names the plan, its
//   spend, and the bound, the only resolution the exhausted type
//   offers is the published Executing → RecoveryRequired row — an
//   existing edge, verified against the row's own endpoints — and a
//   retention pass over the journal that holds the exhausted (still
//   live) apply reclaims none of its records: the writer is stopped,
//   the recoverer is never blinded.
// Evidence: budget_exhaustion_fails_closed_through_an_existing_edge
#[test]
fn budget_exhaustion_fails_closed_through_an_existing_edge() {
    let (hungry, quiet) = (plan(0x11), plan(0x22));
    let records = [
        act(hungry),
        checkpoint(hungry, 0),
        checkpoint(hungry, 1),
        act(quiet),
    ];
    let journal = journal_of(&records);
    let decoded = decode_journal(journal.bytes()).expect("intact");

    let expected_hungry: u64 = records[..3]
        .iter()
        .map(|record| (MIN_FRAME_LEN + record.encode().expect("encodes").len()) as u64)
        .sum();
    let spend = decoded.spend();
    assert_eq!(
        spend.iter().find(|(p, _)| *p == hungry).map(|(_, s)| *s),
        Some(expected_hungry),
        "spend is the encoded frame bytes, attributed per apply"
    );

    assert!(
        over_budget(&decoded).is_empty(),
        "the shipped budget is generous"
    );
    let exhausted = over_budget_against(&decoded, expected_hungry);
    assert_eq!(
        exhausted,
        [BudgetExhausted {
            plan: hungry,
            spent: expected_hungry,
            budget: expected_hungry,
        }],
        "exhaustion names the plan, the spend, and the bound"
    );

    let failure = exhausted[0].journaled_failure(RecordedInstant::from_secs(1_700_000_777));
    assert_eq!(failure.plan(), hungry);
    assert_eq!(
        failure.instant(),
        RecordedInstant::from_secs(1_700_000_777),
        "the journaled failure carries the caller's own clock reading (schema v2)"
    );
    assert_eq!(
        failure.transition(),
        Transition::StepFailureOrInterruption,
        "the exhaustion routes through an existing published row"
    );
    assert_eq!(
        (
            failure.transition().from().name(),
            failure.transition().to().name()
        ),
        ("Executing", "RecoveryRequired"),
        "the row's endpoints are Section 8's, unchanged"
    );

    // Fail-closed direction: the exhausted apply is live, so the
    // retention pass reclaims nothing of it.
    let compacted = compact(&journal).expect("compactable");
    assert_eq!(
        compacted.journal.bytes(),
        journal.bytes(),
        "nothing was reclaimable: every apply is live"
    );
    assert!(compacted.compaction_records.is_empty());
}

// Requirements: JRN-004
//   Imported obligation 11's derivation half: the covered ranges that
//   let replay classify a gap as policy derive from the journal's own
//   durable compaction records and from nothing else — the compacted
//   journal decodes whole, while the same bytes replayed with no
//   covered ranges refuse as the named mid-chain-gap corruption case;
//   removing one retained frame beyond what any compaction record
//   covers refuses the same way; and a compaction record cannot hide
//   frame-level damage, because checksums run before gap
//   classification.
// Evidence: covered_ranges_derive_from_durable_compaction_records_alone
#[test]
fn covered_ranges_derive_from_durable_compaction_records_alone() {
    let (done, live) = (plan(0x31), plan(0x42));
    let journal = journal_of(&[act(done), completed(done), act(live), checkpoint(live, 0)]);
    let compacted = compact(&journal).expect("compactable").journal;

    let decoded = decode_journal(compacted.bytes()).expect("gap covered by its own record");
    assert_eq!(
        decoded.records().len(),
        3,
        "live apply's two records plus the compaction record"
    );

    assert!(
        matches!(
            replay(compacted.bytes(), &CoveredRanges::none()),
            Err(ReplayRefused::MidChainGap { .. })
        ),
        "without the derived ranges the same gap is the named corruption case"
    );

    // Splice out one retained frame (the live act, seq 3): no
    // compaction record covers it.
    let frames = decoded.replay().records();
    let removed_len = MIN_FRAME_LEN + frames[0].payload().len();
    let mut spliced = compacted.bytes().to_vec();
    spliced.drain(0..removed_len);
    assert!(
        matches!(
            decode_journal(&spliced),
            Err(CompactedRefused::Frames(ReplayRefused::MidChainGap { .. }))
        ),
        "an uncovered absence refuses even beside a compaction record"
    );

    let mut damaged = compacted.bytes().to_vec();
    damaged[MIN_FRAME_LEN + 2] ^= 0x01;
    assert!(
        matches!(
            decode_journal(&damaged),
            Err(CompactedRefused::Frames(
                ReplayRefused::InteriorChecksumMismatch { .. }
            ))
        ),
        "a compaction record cannot hide interior damage"
    );
}

// Requirements: JRN-001, JRN-004
//   Imported obligation 12: sequence monotonicity holds across
//   rotation and compaction — retained frames keep their sequence
//   numbers, the compaction record is appended at the continuing
//   position (never a reset, never a reuse), appends after compaction
//   continue from there, recover-and-continue over the compacted
//   bytes preserves the position (rotation in this pure crate), a
//   second retention round stays monotonic, and the first round's
//   compaction record — journal infrastructure — survives every later
//   round, because reclaiming it would orphan the gap it legitimizes.
// Evidence: sequence_monotonicity_holds_across_compaction_and_continued_appends
#[test]
fn sequence_monotonicity_holds_across_compaction_and_continued_appends() {
    let (first_done, second_done, live) = (plan(0x51), plan(0x62), plan(0x73));
    let mut journal = journal_of(&[
        act(first_done),       // 1
        completed(first_done), // 2
        act(live),             // 3
    ]);
    let pre_compaction_next = journal.next_seq();

    let compacted = compact(&journal).expect("compactable");
    let seqs: Vec<u64> = decode_journal(compacted.journal.bytes())
        .expect("decodes")
        .records()
        .iter()
        .map(|(seq, _)| seq.get())
        .collect();
    assert_eq!(
        seqs,
        [3, 4],
        "retained frame keeps its number; the compaction record continues at the next position"
    );
    assert!(
        seqs.windows(2).all(|pair| pair[0] < pair[1]),
        "strictly monotonic"
    );
    assert_eq!(
        compacted.journal.next_seq().get(),
        pre_compaction_next.get() + 1,
        "the compaction record consumed the continuing position — no reset, no reuse"
    );

    // Rotation as this pure crate has it: recover over the compacted
    // bytes and continue.
    journal = compacted.journal;
    let (recovered, _) = Journal::recover(
        journal.bytes(),
        &CoveredRanges::new([(SeqNo::from_raw(1), SeqNo::from_raw(2))]).expect("span"),
    )
    .expect("recover-and-continue");
    assert_eq!(recovered.next_seq(), journal.next_seq());

    // Terminate the live apply, append a second terminated apply, and
    // run a second round.
    journal
        .append(&completed(live).encode().expect("encodes"))
        .expect("bounded"); // 5
    journal
        .append(&act(second_done).encode().expect("encodes"))
        .expect("bounded"); // 6
    journal
        .append(&completed(second_done).encode().expect("encodes"))
        .expect("bounded"); // 7

    let second_round = compact(&journal).expect("compactable");
    let final_seqs: Vec<u64> = decode_journal(second_round.journal.bytes())
        .expect("decodes")
        .records()
        .iter()
        .map(|(seq, _)| seq.get())
        .collect();
    assert!(
        final_seqs.windows(2).all(|pair| pair[0] < pair[1]),
        "monotonic across the second round too: {final_seqs:?}"
    );
    assert!(
        final_seqs.contains(&4),
        "the first round's compaction record survives later rounds"
    );
    assert_eq!(
        second_round.journal.next_seq().get(),
        10,
        "seven appends and three compaction records across both rounds, no number ever reused"
    );
}

// Requirements: JRN-004, JRN-003
//   Imported obligation 13's fixture, the two decisions reconciled:
//   ADR-0028's chain trace passes over a journal compacted around the
//   live apply. An original plan Failed-by-recovery-selection, its
//   live recovery mid-apply, and an unrelated terminated apply
//   compacted away: over the compacted bytes alone, the recovery's
//   act, its transitions, and the original's terminal-with-linkage
//   all survive (the exemption closure), the chain reconstructs
//   identically before and after compaction, and the reconstruction
//   is a pure function of the bytes — nothing from writer memory. The
//   re-entry-edge enforcement over this fixture is increment 5's, as
//   the assignment records.
// Evidence: the_chain_traces_over_a_journal_compacted_around_the_live_apply
#[test]
fn the_chain_traces_over_a_journal_compacted_around_the_live_apply() {
    let (original, recovery, unrelated) = (plan(0x81), plan(0x92), plan(0xA3));
    let journal = journal_of(&[
        act(unrelated),
        act(original),
        failed_with_disposal(original, recovery),
        act(recovery),
        checkpoint(recovery, 0),
        completed(unrelated),
    ]);

    let trace = |bytes: &[u8]| -> (PlanHashRef, Vec<u64>) {
        let decoded = decode_journal(bytes).expect("decodes");
        let terminal = decoded
            .records()
            .iter()
            .find_map(|(_, record)| match record {
                Record::Transition(t) if t.disposal().is_some() => Some(t),
                _ => None,
            })
            .expect("the original's terminal survives");
        let named = terminal.disposal().expect("linkage").recovery_plan();
        let recovery_seqs: Vec<u64> = decoded
            .records()
            .iter()
            .filter(|(_, record)| super::record_plan(record) == Some(named))
            .map(|(seq, _)| seq.get())
            .collect();
        assert!(
            decoded
                .records()
                .iter()
                .any(|(_, r)| matches!(r, Record::AuthorizationAct(a) if a.plan() == named)),
            "the recovery's act is reachable from the journal alone"
        );
        (named, recovery_seqs)
    };

    let before = trace(journal.bytes());
    let compacted = compact(&journal).expect("compactable");
    let after = trace(compacted.journal.bytes());
    assert_eq!(
        before, after,
        "the chain reads identically across compaction"
    );
    assert_eq!(after.0, recovery);
    assert_eq!(
        after.1,
        [4, 5],
        "the recovery's records keep their positions"
    );

    // The unrelated terminated apply is what compaction removed.
    let survivors = owners(&decode_journal(compacted.journal.bytes()).expect("decodes"));
    assert_eq!(
        survivors.iter().filter(|o| **o == Some(unrelated)).count(),
        0,
        "compaction happened around the live chain, not instead of it"
    );
}
