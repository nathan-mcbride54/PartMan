//! The apply lifecycle, enforced at the library layer (increment 5):
//! ADR-0028's definition — an apply is one execution lifecycle of one
//! plan, from its authorization act to a terminal state, identified by
//! the plan hash and an unbroken journal chain — held over decoded
//! journals, with imported obligations 2 (the ordering half), 4, 5, 6,
//! 7, and 8.
//!
//! What is enforced here, and how:
//!
//! - **One act, one apply** (ADR-0021/0028, obligation 7):
//!   [`admit_apply`] admits an apply only against an unconsumed
//!   authorization act for exactly this plan — an act is plan-bound by
//!   its record, consumed by the apply it starts, and never usable for
//!   a second apply or another plan.
//! - **Disposal before recovery** (ADR-0027, obligation 2's ordering
//!   half): a recovery plan named by a disposal linkage is inadmissible
//!   while the original's Failed record sits above the journal's
//!   durable watermark. The HLP-005 structural half — one plan per
//!   bound device set — is the platform packages', re-recorded there.
//! - **Re-entry continues the same apply** (obligation 5): each of the
//!   three re-entry edges traces to the original act through an
//!   unbroken chain — connected Section 8 transitions, the act
//!   preceding the grant — and a broken chain refuses with the break
//!   named.
//! - **The window bounds every re-entry** (PLAN-007, obligation 6): a
//!   re-entry past the plan's validity window is rejected, and a fresh
//!   act journaled after the suspension readmits the same apply — two
//!   acts, one apply, exactly as ADR-0028 reads PLAN-007's re-approval
//!   sentence. Time is an injected seam ([`LogicalTime`]); the truth
//!   of "now" is the caller's, the comparison is this module's.
//! - **Roll-forward takes fresh re-discovery by type** (obligation 4):
//!   [`ReEntry::RollForwardSelected`] carries the
//!   [`FreshRediscovery`] token in its variant, so the JRN-003 rule —
//!   state from journal plus fresh re-discovery — is demanded by the
//!   signature on exactly the edge ADR-0027 names. The token attests
//!   an input this pure crate cannot itself perform; platform truth is
//!   the helper packages' acceptance obligation.
//! - **No process state** (obligation 8): every admission derives from
//!   the decoded journal and nothing else. [`ApplyAdmitted`] and
//!   [`ReEntryAdmitted`] have no public constructor — a grant held
//!   only in a process's memory has no journal record, and a journal
//!   without the act refuses by name.

use partman_statemachine::{State, Transition};

use crate::Journal;
use crate::records::{PlanHashRef, Record};
use crate::retention::DecodedJournal;
use crate::{SeqNo, records::TransitionRecord};

/// A caller-supplied logical instant for PLAN-007 window checks. This
/// pure crate holds no clock: the helper supplies "now" from its own
/// authority, and this module enforces the comparison — the same seam
/// posture as the durability boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalTime(pub u64);

/// PLAN-007's validity window as the validated plan body states it:
/// the last instant at which entry to the apply path is permitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidityWindow {
    /// The window's inclusive expiry.
    pub expires: LogicalTime,
}

/// The attestation that a fresh re-discovery was performed for a
/// roll-forward (JRN-003's second input). Constructing one asserts the
/// caller ran its probe; this pure crate cannot verify that, and says
/// so — the type exists so the roll-forward edge *demands* the input,
/// and asserting platform truth is the helper packages' acceptance
/// work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreshRediscovery {
    _attested: (),
}

impl FreshRediscovery {
    /// Attest that a fresh re-discovery has just been performed.
    #[must_use]
    pub const fn attested() -> Self {
        FreshRediscovery { _attested: () }
    }
}

/// The three ADR-0028 re-entry edges. Roll-forward carries the
/// fresh-re-discovery attestation in its variant: the edge cannot be
/// named without the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReEntry {
    /// Paused → Executing (topology re-verified first, per the row).
    UserResumes,
    /// `RebootPending` → Revalidating (WIN-009's continuity).
    RebootResume,
    /// `RecoveryRequired` → Executing, ADR-0027's first arm — the
    /// original plan continues, state derived from journal plus the
    /// fresh re-discovery this variant demands.
    RollForwardSelected(FreshRediscovery),
}

impl ReEntry {
    /// The published transition this re-entry takes.
    #[must_use]
    pub const fn transition(self) -> Transition {
        match self {
            ReEntry::UserResumes => Transition::UserResumes,
            ReEntry::RebootResume => Transition::RebootResume,
            ReEntry::RollForwardSelected(_) => Transition::RollForwardSelected,
        }
    }

    /// The suspension state this re-entry leaves.
    #[must_use]
    pub const fn suspension(self) -> State {
        self.transition().from()
    }
}

/// One plan's traced chain: its current lifecycle's acts, transitions,
/// and standing, derived from the decoded journal alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyChain {
    plan: PlanHashRef,
    acts: Vec<SeqNo>,
    consumed_act: Option<SeqNo>,
    current: Option<State>,
    last_transition_seq: Option<SeqNo>,
    in_flight: bool,
    terminal: bool,
}

impl ApplyChain {
    /// The current lifecycle's journaled state, if any transition has
    /// been recorded.
    #[must_use]
    pub const fn current(&self) -> Option<State> {
        self.current
    }

    /// Whether an admitted apply is neither terminal nor unstarted.
    #[must_use]
    pub const fn in_flight(&self) -> bool {
        self.in_flight
    }

    /// The act that started the current apply, once consumed.
    #[must_use]
    pub const fn consumed_act(&self) -> Option<SeqNo> {
        self.consumed_act
    }
}

/// Where and how a chain broke — obligation 5's refusal side. Every
/// variant names its position from the journal, because the journal is
/// the only witness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainBroken {
    /// An `AuthorizationGranted` transition with no unconsumed act
    /// before it — the grant existed nowhere but process memory.
    ActMissing {
        /// The grant transition's sequence number.
        grant_seq: SeqNo,
    },
    /// A transition whose `from` state is not the chain's prior `to`
    /// state: records are missing or forged between them.
    Disconnected {
        /// The disconnected transition's sequence number.
        at: SeqNo,
        /// The state the chain stood in.
        expected_from: State,
        /// The state the record claims to leave.
        found_from: State,
    },
}

/// Trace one plan's chain through a decoded journal.
///
/// # Errors
///
/// [`ChainBroken`], naming the break.
pub fn trace(decoded: &DecodedJournal, plan: PlanHashRef) -> Result<ApplyChain, ChainBroken> {
    let mut chain = ApplyChain {
        plan,
        acts: Vec::new(),
        consumed_act: None,
        current: None,
        last_transition_seq: None,
        in_flight: false,
        terminal: false,
    };
    let mut available: Vec<SeqNo> = Vec::new();
    for (seq, record) in decoded.records() {
        match record {
            Record::AuthorizationAct(act) if act.plan() == plan => {
                available.push(*seq);
                chain.acts.push(*seq);
            }
            Record::Transition(transition) if transition.plan() == plan => {
                step(&mut chain, &mut available, *seq, transition)?;
            }
            _ => {}
        }
    }
    Ok(chain)
}

fn step(
    chain: &mut ApplyChain,
    available: &mut Vec<SeqNo>,
    seq: SeqNo,
    transition: &TransitionRecord,
) -> Result<(), ChainBroken> {
    let taken = transition.transition();
    if chain.terminal {
        // A terminal ended the previous lifecycle; a fresh one begins.
        chain.terminal = false;
        chain.in_flight = false;
        chain.consumed_act = None;
        chain.current = None;
    }
    if let Some(prior) = chain.current
        && taken.from() != prior
    {
        return Err(ChainBroken::Disconnected {
            at: seq,
            expected_from: prior,
            found_from: taken.from(),
        });
    }
    if matches!(taken, Transition::AuthorizationGranted) {
        let Some(act_seq) = available.pop() else {
            return Err(ChainBroken::ActMissing { grant_seq: seq });
        };
        chain.consumed_act = Some(act_seq);
        chain.in_flight = true;
    }
    chain.current = Some(taken.to());
    chain.last_transition_seq = Some(seq);
    if taken.to().is_terminal() {
        chain.terminal = true;
        chain.in_flight = false;
    }
    Ok(())
}

/// An admitted apply: the plan and the act that authorizes it, both
/// journal facts. No public constructor exists — admission derives
/// from the journal or not at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyAdmitted {
    plan: PlanHashRef,
    act: SeqNo,
}

impl ApplyAdmitted {
    /// The plan admitted.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The journaled act consumed by this admission.
    #[must_use]
    pub const fn act(&self) -> SeqNo {
        self.act
    }
}

/// Why an apply was not admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionRefused {
    /// No unconsumed authorization act for this plan exists in the
    /// journal. An act for another plan is no act for this one, and a
    /// grant held only in process memory is no act at all.
    NoAct {
        /// The plan that sought admission.
        plan: PlanHashRef,
    },
    /// This plan's apply is already in flight; one act admits one
    /// apply, and a second apply needs its own lifecycle and act.
    ApplyInFlight {
        /// The act backing the in-flight apply.
        act: SeqNo,
    },
    /// The chain is broken; nothing is admitted over a journal that
    /// cannot witness the apply.
    Broken(ChainBroken),
    /// This plan is a recovery plan named by a disposal linkage whose
    /// Failed terminal is not yet durable — ADR-0027's ordering: the
    /// disposal is durable before the recovery plan may apply.
    DisposalNotDurable {
        /// The original plan being disposed.
        original: PlanHashRef,
        /// The Failed terminal's sequence number, above the durable
        /// watermark.
        terminal_seq: SeqNo,
    },
}

/// Admit one apply of one plan, from the journal alone: an unconsumed
/// act for exactly this plan, no apply already in flight, and — where
/// a disposal linkage names this plan as a recovery — the original's
/// Failed record durable below the journal's watermark.
///
/// # Errors
///
/// [`AdmissionRefused`], naming the missing fact.
///
/// # Panics
///
/// Never: the one `expect` reads the consumed act of an in-flight
/// apply, which the trace only marks in-flight after consuming an act
/// — stated as a panic bound rather than hidden.
pub fn admit_apply(
    journal: &Journal,
    decoded: &DecodedJournal,
    plan: PlanHashRef,
) -> Result<ApplyAdmitted, AdmissionRefused> {
    let chain = trace(decoded, plan).map_err(AdmissionRefused::Broken)?;
    if chain.in_flight {
        return Err(AdmissionRefused::ApplyInFlight {
            act: chain
                .consumed_act
                .expect("an in-flight apply consumed its act"),
        });
    }
    // The unconsumed act: the latest act not consumed by a grant and
    // belonging to the current (post-terminal) lifecycle.
    let act = chain
        .acts
        .iter()
        .copied()
        .rfind(|seq| {
            Some(*seq) != chain.consumed_act
                && (chain.last_transition_seq.is_none_or(|t| *seq > t) || !chain.terminal)
        })
        .ok_or(AdmissionRefused::NoAct { plan })?;

    for (seq, record) in decoded.records() {
        if let Record::Transition(transition) = record
            && let Some(linkage) = transition.disposal()
            && linkage.recovery_plan() == plan
            && journal
                .durable_through()
                .is_none_or(|through| *seq > through)
        {
            return Err(AdmissionRefused::DisposalNotDurable {
                original: transition.plan(),
                terminal_seq: *seq,
            });
        }
    }
    Ok(ApplyAdmitted { plan, act })
}

/// An admitted re-entry: the same apply continuing under its journaled
/// act(s). No public constructor exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReEntryAdmitted {
    plan: PlanHashRef,
    transition: Transition,
    acts: Vec<SeqNo>,
}

impl ReEntryAdmitted {
    /// The plan whose apply continues.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The re-entry transition taken.
    #[must_use]
    pub const fn transition(&self) -> Transition {
        self.transition
    }

    /// The journaled acts this continuation traces to: the original,
    /// plus the fresh re-approval act when the window had expired —
    /// two acts, one apply, journaled as such.
    #[must_use]
    pub fn acts(&self) -> &[SeqNo] {
        &self.acts
    }
}

/// Why a re-entry was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReEntryRefused {
    /// The chain is broken; a resume that cannot trace to its act
    /// through an unbroken chain refuses (obligation 5).
    Broken(ChainBroken),
    /// The journal does not place the apply in this edge's suspension
    /// state.
    WrongState {
        /// The state the journal witnesses, if any.
        current: Option<State>,
        /// The suspension state the edge requires.
        required: State,
    },
    /// The apply never started or already terminated — there is
    /// nothing to re-enter.
    NotInFlight,
    /// The re-entry arrived past the plan's validity window and no
    /// fresh act has been journaled since the suspension — PLAN-007's
    /// rejection, with its re-approval sentence as the named route
    /// back.
    PastWindow {
        /// The window's expiry.
        expires: LogicalTime,
        /// The offered instant.
        now: LogicalTime,
    },
}

/// Admit a re-entry on one of the three edges: unbroken chain to the
/// act, the journal's state equal to the edge's suspension state, and
/// the PLAN-007 window honored — within it, the original act carries
/// the continuation; past it, only a fresh act journaled after the
/// suspension readmits, and the admission then cites both acts.
///
/// # Errors
///
/// [`ReEntryRefused`], naming the missing fact.
///
/// # Panics
///
/// Never: the `expect`s read the consumed act and last transition of
/// an apply the trace has already marked in-flight, which requires
/// both — stated as panic bounds rather than hidden.
pub fn re_enter(
    decoded: &DecodedJournal,
    plan: PlanHashRef,
    edge: ReEntry,
    now: LogicalTime,
    window: ValidityWindow,
) -> Result<ReEntryAdmitted, ReEntryRefused> {
    let chain = trace(decoded, plan).map_err(ReEntryRefused::Broken)?;
    if !chain.in_flight {
        return Err(ReEntryRefused::NotInFlight);
    }
    let required = edge.suspension();
    if chain.current != Some(required) {
        return Err(ReEntryRefused::WrongState {
            current: chain.current,
            required,
        });
    }
    let original = chain
        .consumed_act
        .expect("an in-flight apply consumed its act");
    if now <= window.expires {
        return Ok(ReEntryAdmitted {
            plan,
            transition: edge.transition(),
            acts: vec![original],
        });
    }
    let suspension_seq = chain
        .last_transition_seq
        .expect("an in-flight apply has transitions");
    let fresh = chain.acts.iter().copied().find(|seq| *seq > suspension_seq);
    match fresh {
        Some(fresh) => Ok(ReEntryAdmitted {
            plan,
            transition: edge.transition(),
            acts: vec![original, fresh],
        }),
        None => Err(ReEntryRefused::PastWindow {
            expires: window.expires,
            now,
        }),
    }
}

#[cfg(test)]
mod tests;
