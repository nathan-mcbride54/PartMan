//! The WP-070 execution state machine (increment 1).
//!
//! Section 8's thirteen plan states and its transition table, as types
//! whose shape *is* the specification's claim: every declared transition
//! is a [`Transition`] variant carrying its endpoints and trigger, and an
//! undeclared transition has no variant to be — unrepresentable at
//! construction, not rejected at validation (Section 11.6's obligation,
//! imported by the assignment as obligation 1 from ADR-0027).
//!
//! The machine-readable table Section 8 requires under `schemas/` is
//! rendered by [`published_markdown`] from the same variants the property
//! tests check, and `schemas/state-machine.md` is held byte-fresh by
//! test — one source, three views (types, tests, document), no drift.
//!
//! Two decided readings are carried as documentation where they live:
//!
//! - **ADR-0027 (SI-20):** the two `RecoveryRequired` exits are the two
//!   arms — [`Transition::RollForwardSelected`] continues the *original*
//!   plan (same hash, same journal, state derived per JRN-003), and
//!   [`Transition::FailureAccepted`] is the disposal arm, whose trigger
//!   a distinct recovery plan's selection *is*, with the journaled
//!   linkage and disposal-before-apply ordering landing with the journal
//!   increments.
//! - **ADR-0028 (SI-21):** interruption suspends an apply; only
//!   [`State::is_terminal`] states end it. The re-entry edges
//!   ([`Transition::UserResumes`], [`Transition::RebootResume`],
//!   [`Transition::RollForwardSelected`]) continue the same apply under
//!   the same journaled act — enforcement is increment 5's; the machine
//!   carries the shape.
//!
//! This crate has no dependencies, performs no I/O, and holds no state:
//! it is vocabulary. The journal (increment 2) builds on it.

/// The thirteen top-level plan states, exactly as Section 8 lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// Being edited; not yet validated.
    Draft,
    /// Validator passed; edits return it to [`State::Draft`].
    Validated,
    /// Apply submitted; awaiting the HLP-003 authorization act.
    AwaitingAuthorization,
    /// Helper revalidation in progress (HLP-002, PLAN-006).
    Revalidating,
    /// Metadata/encryption backups being taken (PART-013, REC-011).
    Protecting,
    /// Steps executing.
    Executing,
    /// Postcondition verification (UI-012).
    Verifying,
    /// Terminal: postconditions passed.
    Completed,
    /// Suspended at a cancellable or checkpoint boundary.
    Paused,
    /// A declared reboot step was reached; resume is WIN-009's.
    RebootPending,
    /// Interrupted with recovery actions; persists until the user acts.
    RecoveryRequired,
    /// Terminal: failed, with a full report.
    Failed,
    /// Terminal: cancelled after journaled unwind.
    Cancelled,
}

impl State {
    /// Every state, in Section 8's listing order.
    pub const ALL: [State; 13] = [
        State::Draft,
        State::Validated,
        State::AwaitingAuthorization,
        State::Revalidating,
        State::Protecting,
        State::Executing,
        State::Verifying,
        State::Completed,
        State::Paused,
        State::RebootPending,
        State::RecoveryRequired,
        State::Failed,
        State::Cancelled,
    ];

    /// Section 8's terminal states: `Completed`, `Failed`, `Cancelled`.
    /// Every terminal record includes an effect summary ([`Effect`]),
    /// which [`TerminalRecord`] makes structural.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, State::Completed | State::Failed | State::Cancelled)
    }

    /// The state's name as Section 8 spells it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            State::Draft => "Draft",
            State::Validated => "Validated",
            State::AwaitingAuthorization => "AwaitingAuthorization",
            State::Revalidating => "Revalidating",
            State::Protecting => "Protecting",
            State::Executing => "Executing",
            State::Verifying => "Verifying",
            State::Completed => "Completed",
            State::Paused => "Paused",
            State::RebootPending => "RebootPending",
            State::RecoveryRequired => "RecoveryRequired",
            State::Failed => "Failed",
            State::Cancelled => "Cancelled",
        }
    }
}

/// The effect summary every terminal record carries (Section 8):
/// `no-writes`, `partial`, or `complete`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    /// No storage write occurred.
    NoWrites,
    /// Some declared writes occurred; the journal says which.
    Partial,
    /// Every declared write occurred and verified.
    Complete,
}

impl Effect {
    /// The effect's name as Section 8 spells it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Effect::NoWrites => "no-writes",
            Effect::Partial => "partial",
            Effect::Complete => "complete",
        }
    }
}

/// A terminal record's summary: the terminal state and its effect,
/// inseparably. Section 8's "every terminal record includes an effect
/// summary" held by construction — there is no terminal without an
/// effect, and no way to name a non-terminal state here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalRecord {
    terminal: State,
    effect: Effect,
}

/// The one refusal this crate owns: a terminal record was requested for
/// a state Section 8 does not list as terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotTerminal {
    /// The non-terminal state that was offered.
    pub state: State,
}

impl TerminalRecord {
    /// Construct a terminal record; refuses for non-terminal states.
    /// Where the arriving [`Transition`] constrains the effect
    /// ([`Transition::effect_constraint`]), the journal increment holds
    /// that check at record-write time — this type holds the
    /// terminal-with-effect shape.
    ///
    /// # Errors
    ///
    /// [`NotTerminal`], naming the offered state, when it is not one of
    /// Section 8's three terminal states.
    pub const fn new(terminal: State, effect: Effect) -> Result<Self, NotTerminal> {
        if terminal.is_terminal() {
            Ok(TerminalRecord { terminal, effect })
        } else {
            Err(NotTerminal { state: terminal })
        }
    }

    /// The terminal state.
    #[must_use]
    pub const fn terminal(self) -> State {
        self.terminal
    }

    /// The effect summary.
    #[must_use]
    pub const fn effect(self) -> Effect {
        self.effect
    }
}

/// One variant per published transition-table row — twenty-three, no
/// more, no fewer. A `(from, to)` pair outside the table has no variant
/// and therefore no representation: Section 8's "no other transitions
/// exist" is the type's shape, and the Section 11.6 property test proves
/// the variant set equals the published table exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Transition {
    /// Draft → Validated: validator passes.
    ValidatorPasses,
    /// Validated → Draft: user edit, or invalidation (CONC-003).
    EditOrInvalidation,
    /// Validated → `AwaitingAuthorization`: user/CLI submits apply.
    ApplySubmitted,
    /// `AwaitingAuthorization` → Revalidating: authorization granted
    /// (HLP-003).
    AuthorizationGranted,
    /// `AwaitingAuthorization` → Cancelled: user declines, or validity
    /// window expires (PLAN-007) — effect `no-writes`.
    DeclinedOrExpired,
    /// Revalidating → Protecting: helper revalidation passes (HLP-002,
    /// PLAN-006).
    RevalidationPasses,
    /// Revalidating → Failed: identity/topology mismatch — effect
    /// `no-writes` (ACC-007).
    IdentityMismatch,
    /// Protecting → Executing: metadata/encryption backups complete and
    /// verified (PART-013, REC-011).
    BackupsVerified,
    /// Protecting → Failed: backup failure (SAFE-005) — effect
    /// `no-writes`.
    BackupFailure,
    /// Executing → Verifying: final step complete.
    FinalStepComplete,
    /// Executing → Paused: user pause at a cancellable or checkpoint
    /// boundary.
    UserPauses,
    /// Executing → `RebootPending`: declared reboot step reached.
    RebootStepReached,
    /// Executing → `RecoveryRequired`: step failure with recovery actions,
    /// or interruption detected on restart.
    StepFailureOrInterruption,
    /// Executing → Cancelled: cancel honored at a safe point (PLAN-005)
    /// after journaled unwind — effect `no-writes` or `partial`.
    CancelHonored,
    /// Paused → Executing: user resumes; topology re-verified first.
    /// An ADR-0028 re-entry edge: the same apply continues under the
    /// same journaled act, within the PLAN-007 window.
    UserResumes,
    /// Paused → Cancelled: user cancels — effect per journal.
    CancelWhilePaused,
    /// Paused → `RecoveryRequired`: topology changed while paused.
    TopologyChangedWhilePaused,
    /// `RebootPending` → Revalidating: same plan hash resumes after boot
    /// (WIN-009). An ADR-0028 re-entry edge: continuity, not a retained
    /// grant.
    RebootResume,
    /// `RebootPending` → `RecoveryRequired`: resume impossible or state
    /// divergent.
    ResumeImpossible,
    /// Verifying → Completed: postconditions pass (UI-012).
    PostconditionsPass,
    /// Verifying → `RecoveryRequired`: postcondition failure.
    PostconditionFailure,
    /// `RecoveryRequired` → Executing: user selects a valid roll-forward
    /// action (REC-009). ADR-0027's first arm: the *original* plan
    /// continues — same hash, same journal, state derived from journal
    /// plus fresh re-discovery (JRN-003) — the one recovery act that is
    /// not its own plan.
    RollForwardSelected,
    /// `RecoveryRequired` → Failed: user accepts failure; full report
    /// generated. ADR-0027's second arm: selecting a distinct recovery
    /// action — its own `OperationPlan` — *is* this acceptance, and the
    /// terminal record carries the journaled linkage naming the
    /// recovery plan, durable before that plan may apply.
    FailureAccepted,
}

impl Transition {
    /// Every declared transition, in the published table's row order.
    pub const ALL: [Transition; 23] = [
        Transition::ValidatorPasses,
        Transition::EditOrInvalidation,
        Transition::ApplySubmitted,
        Transition::AuthorizationGranted,
        Transition::DeclinedOrExpired,
        Transition::RevalidationPasses,
        Transition::IdentityMismatch,
        Transition::BackupsVerified,
        Transition::BackupFailure,
        Transition::FinalStepComplete,
        Transition::UserPauses,
        Transition::RebootStepReached,
        Transition::StepFailureOrInterruption,
        Transition::CancelHonored,
        Transition::UserResumes,
        Transition::CancelWhilePaused,
        Transition::TopologyChangedWhilePaused,
        Transition::RebootResume,
        Transition::ResumeImpossible,
        Transition::PostconditionsPass,
        Transition::PostconditionFailure,
        Transition::RollForwardSelected,
        Transition::FailureAccepted,
    ];

    /// The state this transition leaves.
    #[must_use]
    pub const fn from(self) -> State {
        self.endpoints().0
    }

    /// The state this transition enters.
    #[must_use]
    pub const fn to(self) -> State {
        self.endpoints().1
    }

    /// Both endpoints, exactly as the published row states them.
    #[must_use]
    pub const fn endpoints(self) -> (State, State) {
        match self {
            Transition::ValidatorPasses => (State::Draft, State::Validated),
            Transition::EditOrInvalidation => (State::Validated, State::Draft),
            Transition::ApplySubmitted => (State::Validated, State::AwaitingAuthorization),
            Transition::AuthorizationGranted => (State::AwaitingAuthorization, State::Revalidating),
            Transition::DeclinedOrExpired => (State::AwaitingAuthorization, State::Cancelled),
            Transition::RevalidationPasses => (State::Revalidating, State::Protecting),
            Transition::IdentityMismatch => (State::Revalidating, State::Failed),
            Transition::BackupsVerified => (State::Protecting, State::Executing),
            Transition::BackupFailure => (State::Protecting, State::Failed),
            Transition::FinalStepComplete => (State::Executing, State::Verifying),
            Transition::UserPauses => (State::Executing, State::Paused),
            Transition::RebootStepReached => (State::Executing, State::RebootPending),
            Transition::StepFailureOrInterruption => (State::Executing, State::RecoveryRequired),
            Transition::CancelHonored => (State::Executing, State::Cancelled),
            Transition::UserResumes => (State::Paused, State::Executing),
            Transition::CancelWhilePaused => (State::Paused, State::Cancelled),
            Transition::TopologyChangedWhilePaused => (State::Paused, State::RecoveryRequired),
            Transition::RebootResume => (State::RebootPending, State::Revalidating),
            Transition::ResumeImpossible => (State::RebootPending, State::RecoveryRequired),
            Transition::PostconditionsPass => (State::Verifying, State::Completed),
            Transition::PostconditionFailure => (State::Verifying, State::RecoveryRequired),
            Transition::RollForwardSelected => (State::RecoveryRequired, State::Executing),
            Transition::FailureAccepted => (State::RecoveryRequired, State::Failed),
        }
    }

    /// The trigger column, verbatim from the published table.
    #[must_use]
    pub const fn trigger(self) -> &'static str {
        match self {
            Transition::ValidatorPasses => "Validator passes",
            Transition::EditOrInvalidation => "User edit, or invalidation (CONC-003)",
            Transition::ApplySubmitted => "User/CLI submits apply",
            Transition::AuthorizationGranted => "Authorization granted (HLP-003)",
            Transition::DeclinedOrExpired => {
                "User declines, or validity window expires (PLAN-007) — effect `no-writes`"
            }
            Transition::RevalidationPasses => "Helper revalidation passes (HLP-002, PLAN-006)",
            Transition::IdentityMismatch => {
                "Identity/topology mismatch — effect `no-writes` (ACC-007)"
            }
            Transition::BackupsVerified => {
                "Metadata/encryption backups complete and verified (PART-013, REC-011)"
            }
            Transition::BackupFailure => "Backup failure (SAFE-005) — effect `no-writes`",
            Transition::FinalStepComplete => "Final step complete",
            Transition::UserPauses => "User pause at a cancellable or checkpoint boundary",
            Transition::RebootStepReached => "Declared reboot step reached",
            Transition::StepFailureOrInterruption => {
                "Step failure with recovery actions, or interruption detected on restart"
            }
            Transition::CancelHonored => {
                "Cancel honored at a safe point (PLAN-005) after journaled unwind — effect `no-writes` or `partial`"
            }
            Transition::UserResumes => "User resumes; topology re-verified first",
            Transition::CancelWhilePaused => "User cancels — effect per journal",
            Transition::TopologyChangedWhilePaused => "Topology changed while paused",
            Transition::RebootResume => "Same plan hash resumes after boot (WIN-009)",
            Transition::ResumeImpossible => "Resume impossible or state divergent",
            Transition::PostconditionsPass => "Postconditions pass (UI-012)",
            Transition::PostconditionFailure => "Postcondition failure",
            Transition::RollForwardSelected => "User selects a valid roll-forward action (REC-009)",
            Transition::FailureAccepted => "User accepts failure; full report generated",
        }
    }

    /// The effect constraint the published row states, where it states
    /// one. `None` means the row constrains nothing here — for
    /// [`Transition::CancelWhilePaused`] the row says "effect per
    /// journal", which is the journal increment's to determine at
    /// record-write time; rows entering `Completed` carry no stated
    /// constraint and the terminal record still carries its effect.
    #[must_use]
    pub const fn effect_constraint(self) -> Option<&'static [Effect]> {
        match self {
            Transition::DeclinedOrExpired
            | Transition::IdentityMismatch
            | Transition::BackupFailure => Some(&[Effect::NoWrites]),
            Transition::CancelHonored => Some(&[Effect::NoWrites, Effect::Partial]),
            _ => None,
        }
    }
}

/// Render the machine-readable transition table Section 8 requires under
/// `schemas/`, from the same variants the types check. The committed
/// `schemas/state-machine.md` is held byte-equal to this output by test;
/// to regenerate after a spec change, write this function's output over
/// that file and let the test arbitrate.
#[must_use]
pub fn published_markdown() -> String {
    let mut out = String::new();
    out.push_str(
        "# The execution state machine, machine-readably\n\
         \n\
         - Spec version: source of truth is `AGENT_BUILD_SPEC.md` Section 8\n\
         - Owner: WP-070 (`docs/work-packages/WP-070.md`)\n\
         - Generated from `crates/statemachine`'s `published_markdown()` and\n\
         \x20\x20held byte-fresh by the `the_published_table_is_byte_fresh` test:\n\
         \x20\x20one source — the `Transition` variants the property tests prove\n\
         \x20\x20equal to Section 8's table — three views (types, tests, this\n\
         \x20\x20document). To regenerate, write that function's output over this\n\
         \x20\x20file; the test arbitrates.\n\
         \n\
         This document records a delivered vocabulary. It decides nothing: a\n\
         row exists here because Section 8 publishes it and the crate encodes\n\
         it, never because this document says so.\n\
         \n\
         States: ",
    );
    for (i, state) in State::ALL.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('`');
        out.push_str(state.name());
        out.push('`');
    }
    out.push_str(
        ".\n\
         \n\
         Terminal states: `Completed`, `Failed`, `Cancelled` — every terminal\n\
         record carries an effect summary (`no-writes`, `partial`,\n\
         `complete`), held structurally by `TerminalRecord`.\n\
         \n\
         | From | To | Trigger |\n\
         | --- | --- | --- |\n",
    );
    for transition in Transition::ALL {
        out.push_str("| ");
        out.push_str(transition.from().name());
        out.push_str(" | ");
        out.push_str(transition.to().name());
        out.push_str(" | ");
        out.push_str(transition.trigger());
        out.push_str(" |\n");
    }
    out.push_str(
        "\n\
         No other transitions exist — structurally: an undeclared pair has no\n\
         `Transition` variant, and the Section 11.6 property test proves the\n\
         variant set equals this table exactly.\n\
         \n\
         The two `RecoveryRequired` exits are the two arms (ADR-0027):\n\
         roll-forward continues the *original* plan — same hash, same\n\
         journal, state derived from journal plus fresh re-discovery\n\
         (JRN-003) — and is the one recovery act that is not its own plan;\n\
         accepting failure is the disposal arm, which selecting a distinct\n\
         recovery action *is*, with the journaled linkage and the\n\
         disposal-durable-before-apply ordering landing with the journal\n\
         increments. Interruption suspends an apply and only terminals end\n\
         it (ADR-0028); the re-entry edges continue the same apply under the\n\
         same journaled act, within the PLAN-007 window.\n",
    );
    out
}

#[cfg(test)]
mod tests;
