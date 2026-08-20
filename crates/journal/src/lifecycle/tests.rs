//! Increment 5's suite: imported obligations 2 (the ordering half), 4,
//! 5, 6, 7, and 8 — the apply lifecycle enforced over decoded
//! journals, every admission a pure function of the bytes.

use partman_statemachine::{Effect, State, Transition};

use super::{
    AdmissionRefused, ChainBroken, FreshRediscovery, LogicalTime, ReEntry, ReEntryRefused,
    ValidityWindow, admit_apply, re_enter, trace,
};
use crate::records::{
    AuthorizationAct, AuthorizationTier, DisposalLinkage, PlanHashRef, Record, RecordedInstant,
    TransitionRecord,
};
use crate::retention::{DecodedJournal, decode_journal};
use crate::{DurabilityRefused, DurabilitySeam, Journal};

struct AcknowledgingSeam;

impl DurabilitySeam for AcknowledgingSeam {
    fn make_durable(&mut self, _new_bytes: &[u8]) -> Result<(), DurabilityRefused> {
        Ok(())
    }
}

fn plan(tag: u8) -> PlanHashRef {
    PlanHashRef::from_bytes([tag; 32])
}

fn act(target: PlanHashRef) -> Record {
    Record::AuthorizationAct(AuthorizationAct::new(target, AuthorizationTier::FloorAct))
}

/// The fixed instant this suite records transitions at; the lifecycle
/// rules under test read the chain, not the clock.
fn instant_t() -> RecordedInstant {
    RecordedInstant::from_secs(1_700_000_000)
}

fn non_terminal(target: PlanHashRef, transition: Transition) -> Record {
    Record::Transition(
        TransitionRecord::non_terminal(target, transition, instant_t()).expect("row"),
    )
}

fn terminal(target: PlanHashRef, transition: Transition, effect: Effect) -> Record {
    Record::Transition(
        TransitionRecord::terminal(target, transition, effect, None, instant_t()).expect("row"),
    )
}

/// The journaled path from validation to Executing: the act precedes
/// the grant, exactly as admission consumed it.
fn path_to_executing(target: PlanHashRef) -> Vec<Record> {
    vec![
        non_terminal(target, Transition::ValidatorPasses),
        non_terminal(target, Transition::ApplySubmitted),
        act(target),
        non_terminal(target, Transition::AuthorizationGranted),
        non_terminal(target, Transition::RevalidationPasses),
        non_terminal(target, Transition::BackupsVerified),
    ]
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

fn decoded_of(journal: &Journal) -> DecodedJournal {
    decode_journal(journal.bytes()).expect("intact")
}

const IN_WINDOW: LogicalTime = LogicalTime(10);
const EXPIRED: LogicalTime = LogicalTime(99);
const WINDOW: ValidityWindow = ValidityWindow {
    expires: LogicalTime(50),
};

// Requirements: Section 8
//   Imported obligation 7 (ADR-0021/0028): one act can never admit a
//   second apply of the same plan or any apply of another plan. An
//   admission requires an unconsumed act for exactly the plan offered
//   — no act refuses by name, another plan's act is no act for this
//   one, an in-flight apply refuses a second admission citing the act
//   it consumed, a terminated lifecycle's consumed act admits nothing
//   further, and a fresh act after the terminal admits exactly one new
//   apply.
// Evidence: an_act_admits_exactly_one_apply_of_exactly_its_plan
#[test]
fn an_act_admits_exactly_one_apply_of_exactly_its_plan() {
    let (p, q) = (plan(0x10), plan(0x20));

    let empty = journal_of(&[]);
    assert_eq!(
        admit_apply(&empty, &decoded_of(&empty), p),
        Err(AdmissionRefused::NoAct { plan: p })
    );

    let others_act = journal_of(&[act(q)]);
    assert_eq!(
        admit_apply(&others_act, &decoded_of(&others_act), p),
        Err(AdmissionRefused::NoAct { plan: p }),
        "another plan's act is no act for this one"
    );

    let ready = journal_of(&[
        non_terminal(p, Transition::ValidatorPasses),
        non_terminal(p, Transition::ApplySubmitted),
        act(p),
    ]);
    let admitted = admit_apply(&ready, &decoded_of(&ready), p).expect("one unconsumed act");
    assert_eq!(admitted.plan(), p);
    assert_eq!(admitted.act().get(), 3);

    let in_flight = journal_of(&path_to_executing(p));
    assert!(
        matches!(
            admit_apply(&in_flight, &decoded_of(&in_flight), p),
            Err(AdmissionRefused::ApplyInFlight { act }) if act.get() == 3
        ),
        "an in-flight apply refuses a second admission, citing its act"
    );

    let mut done = path_to_executing(p);
    done.push(non_terminal(p, Transition::FinalStepComplete));
    done.push(terminal(
        p,
        Transition::PostconditionsPass,
        Effect::Complete,
    ));
    let done_journal = journal_of(&done);
    assert_eq!(
        admit_apply(&done_journal, &decoded_of(&done_journal), p),
        Err(AdmissionRefused::NoAct { plan: p }),
        "a consumed act admits nothing after its apply terminates"
    );

    let mut second = done.clone();
    second.push(act(p));
    let second_journal = journal_of(&second);
    let readmitted =
        admit_apply(&second_journal, &decoded_of(&second_journal), p).expect("a fresh act");
    assert_eq!(
        readmitted.act().get(),
        9,
        "the fresh act, not the consumed one"
    );

    // The sharpest form: one act, two grants. A second lifecycle whose
    // grant arrives with no second act must refuse at the grant —
    // otherwise one act has admitted two applies.
    let mut reused = done;
    reused.push(non_terminal(p, Transition::ValidatorPasses));
    reused.push(non_terminal(p, Transition::ApplySubmitted));
    reused.push(non_terminal(p, Transition::AuthorizationGranted));
    let reused_journal = journal_of(&reused);
    assert_eq!(
        admit_apply(&reused_journal, &decoded_of(&reused_journal), p),
        Err(AdmissionRefused::Broken(ChainBroken::ActMissing {
            grant_seq: crate::SeqNo::from_raw(11),
        })),
        "one act can never carry a second grant"
    );
}

// Requirements: Section 8, JRN-002
//   Imported obligation 2's ordering half (ADR-0027): a recovery
//   plan's apply is unreachable while the original's Failed record is
//   not durable. Admission consults the journal's durable watermark:
//   with the Failed-with-linkage terminal appended but uncommitted the
//   recovery plan refuses by name, and after one commit through the
//   seam the same admission succeeds — the disposal is durable before
//   the recovery may apply. The HLP-005 structural half on a shared
//   device set is the platform packages' obligation, re-recorded
//   there.
// Evidence: a_recovery_apply_is_unreachable_until_the_disposal_is_durable
#[test]
fn a_recovery_apply_is_unreachable_until_the_disposal_is_durable() {
    let (original, recovery) = (plan(0x30), plan(0x40));
    let mut journal = journal_of(&[
        non_terminal(original, Transition::ValidatorPasses),
        non_terminal(original, Transition::ApplySubmitted),
        act(original),
        non_terminal(original, Transition::AuthorizationGranted),
        non_terminal(original, Transition::RevalidationPasses),
        non_terminal(original, Transition::BackupsVerified),
        non_terminal(original, Transition::StepFailureOrInterruption),
    ]);
    let mut seam = AcknowledgingSeam;
    journal.commit(&mut seam).expect("durable so far");

    journal
        .append(
            &Record::Transition(
                TransitionRecord::terminal(
                    original,
                    Transition::FailureAccepted,
                    Effect::Partial,
                    Some(DisposalLinkage::new(recovery)),
                    instant_t(),
                )
                .expect("the disposal arm"),
            )
            .encode()
            .expect("encodes"),
        )
        .expect("bounded");
    journal
        .append(&act(recovery).encode().expect("encodes"))
        .expect("bounded");

    let decoded = decoded_of(&journal);
    assert!(
        matches!(
            admit_apply(&journal, &decoded, recovery),
            Err(AdmissionRefused::DisposalNotDurable { original: o, terminal_seq })
                if o == original && terminal_seq.get() == 8
        ),
        "the recovery is unreachable while the Failed record is pending"
    );

    journal.commit(&mut seam).expect("now durable");
    let admitted = admit_apply(&journal, &decoded, recovery).expect("disposal durable");
    assert_eq!(admitted.plan(), recovery);
}

// Requirements: Section 8, JRN-003
//   Imported obligation 5 (ADR-0028): a resume on each of the three
//   re-entry edges traces to the original act through an unbroken
//   journal chain — the same apply, the same consumed act — and a
//   broken chain refuses naming the break: a grant with no act behind
//   it (the in-memory-grant shape), and a path whose transition leaves
//   a state the chain never reached (records missing between).
// Evidence: re_entry_traces_an_unbroken_chain_and_a_broken_chain_refuses
#[test]
fn re_entry_traces_an_unbroken_chain_and_a_broken_chain_refuses() {
    let p = plan(0x50);
    let suspensions: [(Transition, ReEntry); 3] = [
        (Transition::UserPauses, ReEntry::UserResumes),
        (Transition::RebootStepReached, ReEntry::RebootResume),
        (
            Transition::StepFailureOrInterruption,
            ReEntry::RollForwardSelected(FreshRediscovery::attested()),
        ),
    ];
    for (suspend, edge) in suspensions {
        let mut records = path_to_executing(p);
        records.push(non_terminal(p, suspend));
        let journal = journal_of(&records);
        let admitted = re_enter(&decoded_of(&journal), p, edge, IN_WINDOW, WINDOW)
            .expect("an unbroken chain re-enters");
        assert_eq!(admitted.transition(), edge.transition());
        assert_eq!(
            admitted.acts(),
            [crate::SeqNo::from_raw(3)],
            "the continuation cites the original act, nothing else"
        );
    }

    // Break 1: the grant with no act — the chain refuses at the grant.
    let mut no_act: Vec<Record> = path_to_executing(p);
    no_act.remove(2);
    no_act.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&no_act);
    assert_eq!(
        re_enter(
            &decoded_of(&journal),
            p,
            ReEntry::UserResumes,
            IN_WINDOW,
            WINDOW
        ),
        Err(ReEntryRefused::Broken(ChainBroken::ActMissing {
            grant_seq: crate::SeqNo::from_raw(3),
        }))
    );

    // Break 2: a missing transition — Paused claimed from a state the
    // journal never reached.
    let mut gap: Vec<Record> = path_to_executing(p);
    gap.remove(5); // BackupsVerified: Protecting → Executing
    gap.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&gap);
    assert_eq!(
        re_enter(
            &decoded_of(&journal),
            p,
            ReEntry::UserResumes,
            IN_WINDOW,
            WINDOW
        ),
        Err(ReEntryRefused::Broken(ChainBroken::Disconnected {
            at: crate::SeqNo::from_raw(6),
            expected_from: State::Protecting,
            found_from: State::Executing,
        }))
    );

    // A re-entry on the wrong edge refuses against the journal's own
    // state.
    let mut paused = path_to_executing(p);
    paused.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&paused);
    assert_eq!(
        re_enter(
            &decoded_of(&journal),
            p,
            ReEntry::RebootResume,
            IN_WINDOW,
            WINDOW
        ),
        Err(ReEntryRefused::WrongState {
            current: Some(State::Paused),
            required: State::RebootPending,
        })
    );
}

// Requirements: Section 8
//   Imported obligation 6 (ADR-0028, PLAN-007): a re-entry past the
//   plan's validity window is rejected naming the window and the
//   instant, and a fresh act journaled after the suspension readmits
//   the same apply — the admission cites both acts, two acts, one
//   apply, journaled as such; a fresh act journaled before the
//   suspension does not count, and within the window the original act
//   alone carries the continuation.
// Evidence: re_entry_past_the_window_rejects_and_a_fresh_act_readmits
#[test]
fn re_entry_past_the_window_rejects_and_a_fresh_act_readmits() {
    let p = plan(0x60);
    let mut records = path_to_executing(p);
    records.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&records);
    assert_eq!(
        re_enter(
            &decoded_of(&journal),
            p,
            ReEntry::UserResumes,
            EXPIRED,
            WINDOW
        ),
        Err(ReEntryRefused::PastWindow {
            expires: WINDOW.expires,
            now: EXPIRED,
        }),
        "past the window with no fresh act, the re-entry rejects"
    );

    let mut with_fresh = records.clone();
    with_fresh.push(act(p));
    let journal = journal_of(&with_fresh);
    let readmitted = re_enter(
        &decoded_of(&journal),
        p,
        ReEntry::UserResumes,
        EXPIRED,
        WINDOW,
    )
    .expect("the fresh act readmits");
    assert_eq!(
        readmitted.acts(),
        [crate::SeqNo::from_raw(3), crate::SeqNo::from_raw(8)],
        "two acts, one apply, journaled as such"
    );

    let in_window = re_enter(
        &decoded_of(&journal),
        p,
        ReEntry::UserResumes,
        IN_WINDOW,
        WINDOW,
    )
    .expect("within the window");
    assert_eq!(
        in_window.acts(),
        [crate::SeqNo::from_raw(3)],
        "within the window the original act alone carries the continuation"
    );

    // An act journaled before the suspension is not a re-approval of
    // it: only a fresh act after the suspension readmits.
    let mut stale_fresh = path_to_executing(p);
    stale_fresh.push(act(p));
    stale_fresh.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&stale_fresh);
    assert_eq!(
        re_enter(
            &decoded_of(&journal),
            p,
            ReEntry::UserResumes,
            EXPIRED,
            WINDOW
        ),
        Err(ReEntryRefused::PastWindow {
            expires: WINDOW.expires,
            now: EXPIRED,
        }),
        "a pre-suspension act does not carry a past-window re-entry"
    );
}

// Requirements: Section 8, JRN-003
//   Imported obligation 4 (ADR-0027): the roll-forward edge derives
//   its state from journal plus fresh re-discovery, tested on this
//   edge by name. The fresh-re-discovery input is a seam demanded by
//   the type — the RollForwardSelected variant cannot be named without
//   an attestation — and the continuation's state is the journal's:
//   the admitted transition is the published RecoveryRequired →
//   Executing row, admitted only when the journal itself witnesses
//   RecoveryRequired, whatever the caller believes.
// Evidence: roll_forward_takes_fresh_rediscovery_by_type_and_journal_state
#[test]
fn roll_forward_takes_fresh_rediscovery_by_type_and_journal_state() {
    let p = plan(0x70);
    let mut records = path_to_executing(p);
    records.push(non_terminal(p, Transition::StepFailureOrInterruption));
    let journal = journal_of(&records);

    let edge = ReEntry::RollForwardSelected(FreshRediscovery::attested());
    let admitted = re_enter(&decoded_of(&journal), p, edge, IN_WINDOW, WINDOW)
        .expect("roll-forward with the attested input");
    assert_eq!(admitted.transition(), Transition::RollForwardSelected);
    assert_eq!(
        (admitted.transition().from(), admitted.transition().to()),
        (State::RecoveryRequired, State::Executing),
        "the original plan continues through the published row"
    );

    // The same edge against a journal that does not witness
    // RecoveryRequired refuses on the journal's state, not the
    // caller's belief.
    let executing = journal_of(&path_to_executing(p));
    assert_eq!(
        re_enter(&decoded_of(&executing), p, edge, IN_WINDOW, WINDOW),
        Err(ReEntryRefused::WrongState {
            current: Some(State::Executing),
            required: State::RecoveryRequired,
        })
    );
}

// Requirements: Section 8, JRN-003
//   Imported obligation 8 (ADR-0028), the hand-forged in-memory-grant
//   test: a helper restart holds no authorization state the journal
//   does not. Every admission is a pure function of the journal's
//   bytes — recomputing over a fresh decode of the same bytes answers
//   identically, which is all a restarted helper has; and a journal
//   whose act record was never written (the grant lived only in
//   process memory) refuses both admission and every re-entry edge by
//   name. ApplyAdmitted and ReEntryAdmitted have no public
//   constructor, so a forged grant has no type to inhabit.
// Evidence: a_restart_holds_no_authorization_the_journal_does_not
#[test]
fn a_restart_holds_no_authorization_the_journal_does_not() {
    let p = plan(0x80);
    let mut records = path_to_executing(p);
    records.push(non_terminal(p, Transition::UserPauses));
    let journal = journal_of(&records);

    // The restart: nothing survives but bytes. Two independent decodes
    // answer identically.
    let before = re_enter(
        &decoded_of(&journal),
        p,
        ReEntry::UserResumes,
        IN_WINDOW,
        WINDOW,
    );
    let (recovered, _) =
        Journal::recover(journal.bytes(), &crate::CoveredRanges::none()).expect("recoverable");
    let after = re_enter(
        &decoded_of(&recovered),
        p,
        ReEntry::UserResumes,
        IN_WINDOW,
        WINDOW,
    );
    assert_eq!(before, after, "a restart recomputes, it does not remember");

    // The forged grant: the helper "remembered" an authorization it
    // never journaled. The journal refuses the grant transition itself
    // — and with it, every downstream admission.
    let forged = journal_of(&[
        non_terminal(p, Transition::ValidatorPasses),
        non_terminal(p, Transition::ApplySubmitted),
        // No act record: the grant existed only in memory.
        non_terminal(p, Transition::AuthorizationGranted),
        non_terminal(p, Transition::RevalidationPasses),
        non_terminal(p, Transition::BackupsVerified),
        non_terminal(p, Transition::UserPauses),
    ]);
    let decoded = decoded_of(&forged);
    let broken = ChainBroken::ActMissing {
        grant_seq: crate::SeqNo::from_raw(3),
    };
    assert_eq!(
        trace(&decoded, p),
        Err(broken),
        "the chain itself refuses the memory-only grant"
    );
    assert_eq!(
        admit_apply(&forged, &decoded, p),
        Err(AdmissionRefused::Broken(broken))
    );
    for edge in [
        ReEntry::UserResumes,
        ReEntry::RebootResume,
        ReEntry::RollForwardSelected(FreshRediscovery::attested()),
    ] {
        assert_eq!(
            re_enter(&decoded, p, edge, IN_WINDOW, WINDOW),
            Err(ReEntryRefused::Broken(broken)),
            "{edge:?}: no journal act, no re-entry"
        );
    }
}
