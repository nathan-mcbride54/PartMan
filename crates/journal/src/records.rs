//! The JRN-006 record vocabulary (increment 3): `partman.journal.record`
//! version 1, the versioned public journal schema MODEL-003 requires,
//! encoded through WP-010's `pce/1` canonical codec (MODEL-005) —
//! consumed, never re-derived.
//!
//! Every record class here was decided before this module existed, and
//! each carries its authority:
//!
//! - **The authorization-act record** (ADR-0021, ADR-0028): the floor
//!   act as a journal fact — the exact plan hash and the
//!   helper-computed [`AuthorizationTier`]. The journal is the act's
//!   only home; a helper process never holds authorization state the
//!   journal does not (ADR-0028), and the enforcement of that property
//!   is increment 5's.
//! - **Transition and checkpoint records** (Section 8, JRN-002): one
//!   record per taken transition, with the terminal rows carrying their
//!   effect summaries under the published per-row constraints — the
//!   check increment 1's `TerminalRecord` documentation deferred to
//!   record-write time lives in [`TransitionRecord::terminal`].
//! - **The disposal linkage** (ADR-0027): a Failed-by-recovery-selection
//!   terminal carries the recovery plan's identity, so "Failed,
//!   recovered by plan X" is one reconstructable chain from the journal
//!   alone. The linkage rides only the [`Transition::FailureAccepted`]
//!   row — every other row refuses it.
//! - **The three-variant protection record** (ADR-0024, ADR-0030): a
//!   verified parse-level backup, a positively determined absence, or a
//!   verified raw capture of the write-target regions. No arm is
//!   silent, and artifact references are content hash plus store
//!   identity only — the artifact's bytes have no field to occupy,
//!   which is ADR-0030's "never its bytes" held structurally.
//! - **The compaction record and the per-apply budget** (ADR-0029): the
//!   durable declaration that legitimizes a reclaimed sequence range,
//!   naming its authority; and [`PER_APPLY_JOURNAL_BUDGET_BYTES`],
//!   landing with the schema exactly as the ADR requires. Deriving
//!   [`crate::CoveredRanges`] from compaction records — and enforcing
//!   the budget — is increment 4's.
//! - **The dry-run refusal class** (ADR-0026): [`DryRunRefusal`],
//!   response-data vocabulary consumed from validate-plan surfaces,
//!   distinguishable by type from every validation-failure class. The
//!   enforcement is the helper packages'; the class is this package's.
//!
//! **The WP-010 joint sequencing each ADR names is discharged
//! hash-only:** every reference a record carries — plan, recovery plan,
//! protection artifact — is a 32-byte content hash consumed from the
//! domain crate's MODEL-005 identity, so no WP-010 body schema changed
//! to admit this vocabulary, and nothing here can drift from the plan
//! encoding it references.
//!
//! **JRN-005 by construction:** no record class has a free-text
//! position. Every field is a pinned constant, a member of a closed tag
//! vocabulary, an unsigned integer, or a 32-byte hash; the strict
//! decoder refuses unknown fields, unknown tags, and mistyped
//! positions without echoing their content. The redaction sweep in the
//! test module plants every SEC-006 identifier class in every position
//! and proves each refusal — the WP-035/WP-040 gate shape.
//!
//! The wire layout is specified in `schemas/journal/records.md` and
//! pinned by golden vectors held in agreement with that document by
//! test.

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};
use partman_statemachine::{Effect, Transition};

use crate::SeqNo;

/// The record schema's identifier, carried by every record (MODEL-003;
/// domain separation per `schemas/canonical-encoding.md` §5).
pub const RECORD_SCHEMA: &str = "partman.journal.record";

/// The record schema's version. Any other version refuses at decode —
/// MODEL-003's explicit-rejection arm; migration is a future reviewed
/// increment's, not a silent acceptance.
pub const RECORD_SCHEMA_VERSION: u64 = 1;

/// ADR-0029's per-apply journal budget, landing with the JRN-006 schema
/// as that ADR requires: 256 MiB of encoded frames per apply. Chosen
/// generously — over two hundred maximum-size frames
/// ([`crate::MAX_PAYLOAD_LEN`]), millions of typical records — because
/// the failure direction is what was decided, not the magnitude:
/// exhaustion is a journaled failure through Section 8's existing
/// edges, never a reclamation of live records. Enforcement is
/// increment 4's.
pub const PER_APPLY_JOURNAL_BUDGET_BYTES: u64 = 256 * 1024 * 1024;

/// A plan's identity as a journal record carries it: the MODEL-005
/// body hash, by value. This is a *reference* — constructing one
/// asserts nothing about the plan's existence or validity, which is
/// why a public constructor is honest here where the domain crate's
/// [`canonical::Hash`] deliberately has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanHashRef([u8; 32]);

impl PlanHashRef {
    /// Build a reference from recorded digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        PlanHashRef(bytes)
    }

    /// The digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<&canonical::Hash> for PlanHashRef {
    fn from(hash: &canonical::Hash) -> Self {
        PlanHashRef(*hash.as_bytes())
    }
}

/// A protection artifact's identity: the content hash of its bytes in
/// the helper-owned store (ADR-0030). A distinct type from
/// [`PlanHashRef`] so a plan hash and an artifact hash cannot be
/// cross-wired.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArtifactHashRef([u8; 32]);

impl ArtifactHashRef {
    /// Build a reference from recorded digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        ArtifactHashRef(bytes)
    }

    /// The digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<&canonical::Hash> for ArtifactHashRef {
    fn from(hash: &canonical::Hash) -> Self {
        ArtifactHashRef(*hash.as_bytes())
    }
}

/// The helper-computed authorization tier (ADR-0021): derived by the
/// helper from its own recomputed severity and flags, never from a
/// client claim. Vocabulary here; the computation and enforcement are
/// the helper packages'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorizationTier {
    /// The floor act: fresh, explicit, plan-hash-bound, single-use,
    /// journaled — every apply requires at least this, severity 0
    /// included. May be programmatic (a scripted CLI apply is a fresh
    /// explicit act).
    FloorAct,
    /// Fresh interactive OS-mediated ceremony: severity ≥ Disruptive,
    /// or any plan carrying a step flag — the flags-nonempty closed
    /// rule.
    InteractiveCeremony,
}

impl AuthorizationTier {
    const fn wire_name(self) -> &'static str {
        match self {
            AuthorizationTier::FloorAct => "floor-act",
            AuthorizationTier::InteractiveCeremony => "interactive-ceremony",
        }
    }
}

/// The dry-run refusal class (ADR-0026): a dry run of a preview-backed
/// plan runs to the helper's own recomputed capability gate and
/// refuses there with this typed reason — never a success, so
/// PLAN-009's guarantee stays absolute. Its own type, deliberately:
/// "the combination is unqualified" must be distinguishable by type
/// from every validation-failure class, so "your plan is fine" and
/// "your plan is broken" can never conflate. Response-data vocabulary
/// consumed from validate-plan surfaces; the enforcement and the
/// remediation naming the CAP-006 evidence gap are the helper
/// packages'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DryRunRefusal {
    /// The plan is valid and the capability combination is
    /// preview-backed: apply would refuse pending CAP-006
    /// qualification evidence, and so does the rehearsal, here.
    PendingQualification,
}

/// The store a protection artifact lives in (ADR-0030 Rule 1): the
/// dedicated helper-owned protection-artifact store, admin-protected
/// and documented per OS under JRN-004's location clause. A closed
/// one-member vocabulary in v1 because the ADR fixes exactly one store
/// per helper; the per-OS location is the helper package's documented
/// deployment fact, never journal content — a path here would be a
/// SEC-006 identifier position, and this schema has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactStore {
    /// The helper's dedicated protection-artifact store.
    HelperProtectionStore,
}

impl ArtifactStore {
    const fn wire_name(self) -> &'static str {
        match self {
            ArtifactStore::HelperProtectionStore => "helper-protection-store",
        }
    }
}

/// A hash-only reference to a protection artifact: content hash plus
/// store identity, exactly ADR-0030's Rule 2 — and nothing else. The
/// artifact's bytes have no field to occupy, structurally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectionArtifactRef {
    content: ArtifactHashRef,
    store: ArtifactStore,
}

impl ProtectionArtifactRef {
    /// Build a reference.
    #[must_use]
    pub const fn new(content: ArtifactHashRef, store: ArtifactStore) -> Self {
        ProtectionArtifactRef { content, store }
    }

    /// The artifact's content hash.
    #[must_use]
    pub const fn content(&self) -> ArtifactHashRef {
        self.content
    }

    /// The store holding the artifact.
    #[must_use]
    pub const fn store(&self) -> ArtifactStore {
        self.store
    }
}

/// One byte region of a raw protection capture: `start` and `length`
/// in bytes on the captured target. Validated by
/// [`ProtectionRecord::new`]: nonzero length, strictly ascending,
/// non-overlapping, no arithmetic overflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    /// The region's first byte offset.
    pub start: u64,
    /// The region's length in bytes.
    pub length: u64,
}

/// The retention authority a compaction record declares (ADR-0029):
/// the policy under which the range was reclaimed. Closed at one
/// member in v1 — terminal-history retention is the only reclamation
/// the liveness rule permits; a new authority is a schema version's
/// reviewed change, never a silent addition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionAuthority {
    /// SEC-009-shaped retention over terminal applies' history — the
    /// only population JRN-004's liveness scoping makes reclaimable.
    TerminalHistoryRetention,
}

impl CompactionAuthority {
    const fn wire_name(self) -> &'static str {
        match self {
            CompactionAuthority::TerminalHistoryRetention => "terminal-history-retention",
        }
    }
}

/// The authorization-act record (ADR-0021, ADR-0028): the floor act as
/// a journal fact — plan hash and helper-computed tier. One act
/// authorizes one apply; the single-use consumption and the
/// no-process-state property are increment 5's to enforce over this
/// record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorizationAct {
    plan: PlanHashRef,
    tier: AuthorizationTier,
}

impl AuthorizationAct {
    /// Build the record.
    #[must_use]
    pub const fn new(plan: PlanHashRef, tier: AuthorizationTier) -> Self {
        AuthorizationAct { plan, tier }
    }

    /// The plan the act authorizes, by exact hash.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The helper-computed tier the act was performed at.
    #[must_use]
    pub const fn tier(&self) -> AuthorizationTier {
        self.tier
    }
}

/// The disposal linkage (ADR-0027): the recovery plan's identity,
/// carried by a Failed-by-recovery-selection terminal so the chain
/// "Failed, recovered by plan X" reconstructs from the journal alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisposalLinkage {
    recovery_plan: PlanHashRef,
}

impl DisposalLinkage {
    /// Build the linkage.
    #[must_use]
    pub const fn new(recovery_plan: PlanHashRef) -> Self {
        DisposalLinkage { recovery_plan }
    }

    /// The recovery plan the disposal names.
    #[must_use]
    pub const fn recovery_plan(&self) -> PlanHashRef {
        self.recovery_plan
    }
}

/// A Section 8 transition record: the plan, the published transition
/// taken, and — exactly on terminal rows — the effect summary under
/// the row's published constraint, with the disposal linkage riding
/// only the [`Transition::FailureAccepted`] row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionRecord {
    plan: PlanHashRef,
    transition: Transition,
    effect: Option<Effect>,
    disposal: Option<DisposalLinkage>,
}

impl TransitionRecord {
    /// Build a record for a non-terminal transition.
    ///
    /// # Errors
    ///
    /// [`RecordInvalid::TerminalEffectMissing`] when the transition
    /// enters a terminal state — terminal rows must use
    /// [`TransitionRecord::terminal`] and carry their effect.
    pub const fn non_terminal(
        plan: PlanHashRef,
        transition: Transition,
    ) -> Result<Self, RecordInvalid> {
        if transition.to().is_terminal() {
            return Err(RecordInvalid::TerminalEffectMissing { transition });
        }
        Ok(TransitionRecord {
            plan,
            transition,
            effect: None,
            disposal: None,
        })
    }

    /// Build a record for a terminal transition, carrying the effect
    /// summary Section 8 requires of every terminal record, checked
    /// against the published per-row constraint — the record-write-time
    /// check increment 1 deferred here — and the disposal linkage
    /// where, and only where, ADR-0027 defines one.
    ///
    /// # Errors
    ///
    /// - [`RecordInvalid::EffectOnNonTerminal`] when the transition
    ///   does not enter a terminal state.
    /// - [`RecordInvalid::EffectOutsideConstraint`] when the published
    ///   row states an effect constraint and `effect` is not in it.
    /// - [`RecordInvalid::LinkageOutsideDisposalArm`] when a disposal
    ///   linkage is offered on any row but `FailureAccepted`.
    pub fn terminal(
        plan: PlanHashRef,
        transition: Transition,
        effect: Effect,
        disposal: Option<DisposalLinkage>,
    ) -> Result<Self, RecordInvalid> {
        if !transition.to().is_terminal() {
            return Err(RecordInvalid::EffectOnNonTerminal { transition });
        }
        if let Some(allowed) = transition.effect_constraint()
            && !allowed.contains(&effect)
        {
            return Err(RecordInvalid::EffectOutsideConstraint { transition, effect });
        }
        if disposal.is_some() && !matches!(transition, Transition::FailureAccepted) {
            return Err(RecordInvalid::LinkageOutsideDisposalArm { transition });
        }
        Ok(TransitionRecord {
            plan,
            transition,
            effect: Some(effect),
            disposal,
        })
    }

    /// The plan whose apply took the transition.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The published transition taken.
    #[must_use]
    pub const fn transition(&self) -> Transition {
        self.transition
    }

    /// The effect summary — present exactly on terminal rows.
    #[must_use]
    pub const fn effect(&self) -> Option<Effect> {
        self.effect
    }

    /// The disposal linkage — present only on a
    /// Failed-by-recovery-selection terminal.
    #[must_use]
    pub const fn disposal(&self) -> Option<DisposalLinkage> {
        self.disposal
    }
}

/// A checkpoint record (JRN-002's second population): durable progress
/// inside Executing, by step index. The step vocabulary itself is the
/// plan body's (WP-010/WP-060); the journal carries the index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    plan: PlanHashRef,
    step_index: u64,
}

impl Checkpoint {
    /// Build the record.
    #[must_use]
    pub const fn new(plan: PlanHashRef, step_index: u64) -> Self {
        Checkpoint { plan, step_index }
    }

    /// The plan being executed.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The completed step's index in the plan body's step order.
    #[must_use]
    pub const fn step_index(&self) -> u64 {
        self.step_index
    }
}

/// The three protection arms (ADR-0024): every Protecting → Executing
/// transition carries exactly one, and no arm is silent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtectionArm {
    /// `Present`: primary and secondary metadata backed up at parse
    /// level and verified, referenced hash-only.
    ParseBackupVerified {
        /// The verified backup artifact.
        artifact: ProtectionArtifactRef,
    },
    /// `Absent`: the helper's fresh positively determined absence — a
    /// value, not a skip (ADR-C4's principle reaching the journal),
    /// with no user acknowledgement because the fact is the helper's
    /// own observation.
    AbsenceDetermined,
    /// `Indeterminate`, typed repair family: a verified raw capture of
    /// exactly the regions the plan will write, referenced hash-only,
    /// naming the captured regions.
    RawCaptureVerified {
        /// The verified capture artifact.
        artifact: ProtectionArtifactRef,
        /// The captured regions, strictly ascending and
        /// non-overlapping.
        regions: Vec<Region>,
    },
}

/// The protection record (ADR-0024, ADR-0030): the journaled outcome of
/// the Protecting state, three-variant, hash-only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtectionRecord {
    plan: PlanHashRef,
    arm: ProtectionArm,
}

impl ProtectionRecord {
    /// Build the record, validating a raw capture's region list:
    /// non-empty, every length nonzero, strictly ascending starts,
    /// non-overlapping, no arithmetic overflow.
    ///
    /// # Errors
    ///
    /// [`RecordInvalid::RegionsEmpty`],
    /// [`RecordInvalid::RegionZeroLength`],
    /// [`RecordInvalid::RegionsNotAscendingOrOverlapping`], or
    /// [`RecordInvalid::RegionOverflow`], each naming the offending
    /// index.
    pub fn new(plan: PlanHashRef, arm: ProtectionArm) -> Result<Self, RecordInvalid> {
        if let ProtectionArm::RawCaptureVerified { regions, .. } = &arm {
            if regions.is_empty() {
                return Err(RecordInvalid::RegionsEmpty);
            }
            let mut previous_end: Option<u64> = None;
            for (index, region) in regions.iter().enumerate() {
                if region.length == 0 {
                    return Err(RecordInvalid::RegionZeroLength { index });
                }
                let Some(end) = region.start.checked_add(region.length) else {
                    return Err(RecordInvalid::RegionOverflow { index });
                };
                if let Some(previous_end) = previous_end
                    && region.start < previous_end
                {
                    return Err(RecordInvalid::RegionsNotAscendingOrOverlapping { index });
                }
                previous_end = Some(end);
            }
        }
        Ok(ProtectionRecord { plan, arm })
    }

    /// The plan being protected for.
    #[must_use]
    pub const fn plan(&self) -> PlanHashRef {
        self.plan
    }

    /// The arm taken.
    #[must_use]
    pub const fn arm(&self) -> &ProtectionArm {
        &self.arm
    }
}

/// The compaction record (ADR-0029): the durable declaration that a
/// sequence range was reclaimed under a named authority, so replay
/// classifies the gap as policy rather than corruption. Deriving
/// [`crate::CoveredRanges`] from these records is increment 4's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionRecord {
    first: SeqNo,
    last: SeqNo,
    authority: CompactionAuthority,
}

impl CompactionRecord {
    /// Build the record for the inclusive reclaimed range
    /// `first ..= last`.
    ///
    /// # Errors
    ///
    /// [`RecordInvalid::CompactionRangeBackwards`] when `last` is
    /// below `first`.
    pub fn new(
        first: SeqNo,
        last: SeqNo,
        authority: CompactionAuthority,
    ) -> Result<Self, RecordInvalid> {
        if last < first {
            return Err(RecordInvalid::CompactionRangeBackwards { first, last });
        }
        Ok(CompactionRecord {
            first,
            last,
            authority,
        })
    }

    /// The first reclaimed sequence number.
    #[must_use]
    pub const fn first(&self) -> SeqNo {
        self.first
    }

    /// The last reclaimed sequence number.
    #[must_use]
    pub const fn last(&self) -> SeqNo {
        self.last
    }

    /// The authority under which the range was reclaimed.
    #[must_use]
    pub const fn authority(&self) -> CompactionAuthority {
        self.authority
    }
}

/// One journal record: the closed v1 class set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    /// The authorization-act record.
    AuthorizationAct(AuthorizationAct),
    /// A Section 8 transition record.
    Transition(TransitionRecord),
    /// A checkpoint record.
    Checkpoint(Checkpoint),
    /// The protection record.
    Protection(ProtectionRecord),
    /// The compaction record.
    Compaction(CompactionRecord),
}

/// A construction invariant a record refused to violate. Decode
/// returns these through [`DecodeRefused::Invalid`], so a wire record
/// can never carry a shape the constructors refuse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordInvalid {
    /// A terminal-entering transition was offered with no effect.
    TerminalEffectMissing {
        /// The terminal-entering transition.
        transition: Transition,
    },
    /// An effect was offered on a transition that enters no terminal.
    EffectOnNonTerminal {
        /// The non-terminal transition.
        transition: Transition,
    },
    /// The effect is outside the published row's constraint.
    EffectOutsideConstraint {
        /// The transition whose row states the constraint.
        transition: Transition,
        /// The refused effect.
        effect: Effect,
    },
    /// A disposal linkage was offered on a row that is not the
    /// ADR-0027 disposal arm.
    LinkageOutsideDisposalArm {
        /// The transition the linkage was offered on.
        transition: Transition,
    },
    /// A raw capture with no regions.
    RegionsEmpty,
    /// A raw-capture region with zero length.
    RegionZeroLength {
        /// The offending region's index.
        index: usize,
    },
    /// Raw-capture regions out of ascending order or overlapping.
    RegionsNotAscendingOrOverlapping {
        /// The offending region's index.
        index: usize,
    },
    /// A raw-capture region whose end overflows.
    RegionOverflow {
        /// The offending region's index.
        index: usize,
    },
    /// A compaction range with `last` below `first`.
    CompactionRangeBackwards {
        /// The stated first sequence number.
        first: SeqNo,
        /// The stated last sequence number, below `first`.
        last: SeqNo,
    },
    /// A sequence number of zero, which no journal ever assigns.
    SequenceZero,
}

/// The strict decoder's refusals. Deliberately content-free where the
/// refused input was free-form: a planted identifier is never echoed
/// back through a refusal (SEC-006 posture, the WP-040 shape) — a
/// refusal names the *position*, by this crate's own static names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeRefused {
    /// The bytes are not a canonical `pce/1` encoding.
    NotCanonical(canonical::Error),
    /// The top-level value is not a map.
    NotAMap,
    /// The `schema` field is missing or is not the pinned identifier.
    WrongSchema,
    /// The `schema_version` field is missing or is not the supported
    /// version — MODEL-003's explicit rejection.
    WrongVersion,
    /// A field this schema does not declare (its content not echoed).
    UnknownField,
    /// A declared field is absent.
    MissingField {
        /// The missing field's name.
        field: &'static str,
    },
    /// A declared field carries the wrong `pce/1` type.
    WrongType {
        /// The mistyped field's name.
        field: &'static str,
    },
    /// A closed-vocabulary field carries an unknown tag (the tag not
    /// echoed).
    UnknownTag {
        /// The field whose vocabulary was missed.
        field: &'static str,
    },
    /// A hash field whose byte string is not exactly 32 bytes.
    WrongHashLength {
        /// The offending field's name.
        field: &'static str,
    },
    /// The wire shape decoded, but the record's own invariants refuse
    /// it.
    Invalid(RecordInvalid),
}

const fn transition_wire_name(transition: Transition) -> &'static str {
    match transition {
        Transition::ValidatorPasses => "validator-passes",
        Transition::EditOrInvalidation => "edit-or-invalidation",
        Transition::ApplySubmitted => "apply-submitted",
        Transition::AuthorizationGranted => "authorization-granted",
        Transition::DeclinedOrExpired => "declined-or-expired",
        Transition::RevalidationPasses => "revalidation-passes",
        Transition::IdentityMismatch => "identity-mismatch",
        Transition::BackupsVerified => "backups-verified",
        Transition::BackupFailure => "backup-failure",
        Transition::FinalStepComplete => "final-step-complete",
        Transition::UserPauses => "user-pauses",
        Transition::RebootStepReached => "reboot-step-reached",
        Transition::StepFailureOrInterruption => "step-failure-or-interruption",
        Transition::CancelHonored => "cancel-honored",
        Transition::UserResumes => "user-resumes",
        Transition::CancelWhilePaused => "cancel-while-paused",
        Transition::TopologyChangedWhilePaused => "topology-changed-while-paused",
        Transition::RebootResume => "reboot-resume",
        Transition::ResumeImpossible => "resume-impossible",
        Transition::PostconditionsPass => "postconditions-pass",
        Transition::PostconditionFailure => "postcondition-failure",
        Transition::RollForwardSelected => "roll-forward-selected",
        Transition::FailureAccepted => "failure-accepted",
    }
}

fn transition_from_wire(name: &str) -> Option<Transition> {
    Transition::ALL
        .into_iter()
        .find(|&transition| transition_wire_name(transition) == name)
}

fn effect_from_wire(name: &str) -> Option<Effect> {
    [Effect::NoWrites, Effect::Partial, Effect::Complete]
        .into_iter()
        .find(|effect| effect.name() == name)
}

impl Record {
    /// The record's wire kind tag.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Record::AuthorizationAct(_) => "authorization-act",
            Record::Transition(_) => "transition",
            Record::Checkpoint(_) => "checkpoint",
            Record::Protection(_) => "protection",
            Record::Compaction(_) => "compaction",
        }
    }

    /// Encode to the canonical `pce/1` bytes the journal frames carry.
    ///
    /// # Errors
    ///
    /// Passes through [`canonical::Error`]; a record this module
    /// constructed encodes without error, and the passthrough exists so
    /// no panic path hides in a library the helper packages will trust.
    pub fn encode(&self) -> Result<Vec<u8>, canonical::Error> {
        let mut map = BTreeMap::new();
        map.insert("schema".to_owned(), Value::Text(RECORD_SCHEMA.to_owned()));
        map.insert(
            "schema_version".to_owned(),
            Value::Unsigned(RECORD_SCHEMA_VERSION),
        );
        map.insert("kind".to_owned(), Value::Text(self.kind().to_owned()));
        match self {
            Record::AuthorizationAct(act) => {
                map.insert("plan".to_owned(), Value::Bytes(act.plan.0.to_vec()));
                map.insert(
                    "tier".to_owned(),
                    Value::Text(act.tier.wire_name().to_owned()),
                );
            }
            Record::Transition(record) => {
                map.insert("plan".to_owned(), Value::Bytes(record.plan.0.to_vec()));
                map.insert(
                    "transition".to_owned(),
                    Value::Text(transition_wire_name(record.transition).to_owned()),
                );
                if let Some(effect) = record.effect {
                    map.insert("effect".to_owned(), Value::Text(effect.name().to_owned()));
                }
                if let Some(disposal) = record.disposal {
                    map.insert(
                        "recovery_plan".to_owned(),
                        Value::Bytes(disposal.recovery_plan.0.to_vec()),
                    );
                }
            }
            Record::Checkpoint(checkpoint) => {
                map.insert("plan".to_owned(), Value::Bytes(checkpoint.plan.0.to_vec()));
                map.insert(
                    "step_index".to_owned(),
                    Value::Unsigned(checkpoint.step_index),
                );
            }
            Record::Protection(record) => {
                map.insert("plan".to_owned(), Value::Bytes(record.plan.0.to_vec()));
                let arm_name = match &record.arm {
                    ProtectionArm::ParseBackupVerified { .. } => "parse-backup-verified",
                    ProtectionArm::AbsenceDetermined => "absence-determined",
                    ProtectionArm::RawCaptureVerified { .. } => "raw-capture-verified",
                };
                map.insert("arm".to_owned(), Value::Text(arm_name.to_owned()));
                match &record.arm {
                    ProtectionArm::AbsenceDetermined => {}
                    ProtectionArm::ParseBackupVerified { artifact } => {
                        insert_artifact(&mut map, artifact);
                    }
                    ProtectionArm::RawCaptureVerified { artifact, regions } => {
                        insert_artifact(&mut map, artifact);
                        let encoded_regions = regions
                            .iter()
                            .map(|region| {
                                let mut entry = BTreeMap::new();
                                entry.insert("start".to_owned(), Value::Unsigned(region.start));
                                entry.insert("length".to_owned(), Value::Unsigned(region.length));
                                Value::Map(entry)
                            })
                            .collect();
                        map.insert("regions".to_owned(), Value::Array(encoded_regions));
                    }
                }
            }
            Record::Compaction(record) => {
                map.insert("first".to_owned(), Value::Unsigned(record.first.get()));
                map.insert("last".to_owned(), Value::Unsigned(record.last.get()));
                map.insert(
                    "authority".to_owned(),
                    Value::Text(record.authority.wire_name().to_owned()),
                );
            }
        }
        canonical::encode(&Value::Map(map))
    }

    /// Decode and validate one record from canonical bytes. Strict in
    /// every direction: non-canonical bytes, an unknown schema or
    /// version, an unknown field, an unknown tag, a mistyped position,
    /// a wrong-length hash, and any shape the constructors would
    /// refuse each return a typed refusal — nothing is repaired, and
    /// refused content is never echoed.
    ///
    /// # Errors
    ///
    /// [`DecodeRefused`], naming the refusal and, where a field is
    /// involved, the field by this crate's static name.
    pub fn decode(bytes: &[u8]) -> Result<Record, DecodeRefused> {
        let value = canonical::decode(bytes).map_err(DecodeRefused::NotCanonical)?;
        let Value::Map(mut map) = value else {
            return Err(DecodeRefused::NotAMap);
        };
        match map.remove("schema") {
            Some(Value::Text(schema)) if schema == RECORD_SCHEMA => {}
            _ => return Err(DecodeRefused::WrongSchema),
        }
        match map.remove("schema_version") {
            Some(Value::Unsigned(RECORD_SCHEMA_VERSION)) => {}
            _ => return Err(DecodeRefused::WrongVersion),
        }
        let kind = match map.remove("kind") {
            Some(Value::Text(kind)) => kind,
            Some(_) => return Err(DecodeRefused::WrongType { field: "kind" }),
            None => return Err(DecodeRefused::MissingField { field: "kind" }),
        };
        let record = match kind.as_str() {
            "authorization-act" => {
                let plan = take_hash(&mut map, "plan")?;
                let tier = match take_text(&mut map, "tier")?.as_str() {
                    "floor-act" => AuthorizationTier::FloorAct,
                    "interactive-ceremony" => AuthorizationTier::InteractiveCeremony,
                    _ => return Err(DecodeRefused::UnknownTag { field: "tier" }),
                };
                Record::AuthorizationAct(AuthorizationAct::new(PlanHashRef(plan), tier))
            }
            "transition" => decode_transition(&mut map)?,
            "checkpoint" => {
                let plan = PlanHashRef(take_hash(&mut map, "plan")?);
                let step_index = take_unsigned(&mut map, "step_index")?;
                Record::Checkpoint(Checkpoint::new(plan, step_index))
            }
            "protection" => decode_protection(&mut map)?,
            "compaction" => {
                let first = seq_no(take_unsigned(&mut map, "first")?)?;
                let last = seq_no(take_unsigned(&mut map, "last")?)?;
                let authority = match take_text(&mut map, "authority")?.as_str() {
                    "terminal-history-retention" => CompactionAuthority::TerminalHistoryRetention,
                    _ => return Err(DecodeRefused::UnknownTag { field: "authority" }),
                };
                Record::Compaction(
                    CompactionRecord::new(first, last, authority)
                        .map_err(DecodeRefused::Invalid)?,
                )
            }
            _ => return Err(DecodeRefused::UnknownTag { field: "kind" }),
        };
        if map.is_empty() {
            Ok(record)
        } else {
            Err(DecodeRefused::UnknownField)
        }
    }
}

fn decode_transition(map: &mut BTreeMap<String, Value>) -> Result<Record, DecodeRefused> {
    let plan = PlanHashRef(take_hash(map, "plan")?);
    let transition =
        transition_from_wire(&take_text(map, "transition")?).ok_or(DecodeRefused::UnknownTag {
            field: "transition",
        })?;
    let effect = match map.remove("effect") {
        None => None,
        Some(Value::Text(name)) => {
            Some(effect_from_wire(&name).ok_or(DecodeRefused::UnknownTag { field: "effect" })?)
        }
        Some(_) => return Err(DecodeRefused::WrongType { field: "effect" }),
    };
    let disposal = match map.remove("recovery_plan") {
        None => None,
        Some(Value::Bytes(bytes)) => Some(DisposalLinkage::new(PlanHashRef(hash_bytes(
            &bytes,
            "recovery_plan",
        )?))),
        Some(_) => {
            return Err(DecodeRefused::WrongType {
                field: "recovery_plan",
            });
        }
    };
    let record = match effect {
        Some(effect) => TransitionRecord::terminal(plan, transition, effect, disposal),
        None if disposal.is_some() => Err(RecordInvalid::LinkageOutsideDisposalArm { transition }),
        None => TransitionRecord::non_terminal(plan, transition),
    }
    .map_err(DecodeRefused::Invalid)?;
    Ok(Record::Transition(record))
}

fn decode_protection(map: &mut BTreeMap<String, Value>) -> Result<Record, DecodeRefused> {
    let plan = PlanHashRef(take_hash(map, "plan")?);
    let arm = match take_text(map, "arm")?.as_str() {
        "absence-determined" => ProtectionArm::AbsenceDetermined,
        "parse-backup-verified" => ProtectionArm::ParseBackupVerified {
            artifact: take_artifact(map)?,
        },
        "raw-capture-verified" => {
            let artifact = take_artifact(map)?;
            let regions = match map.remove("regions") {
                Some(Value::Array(entries)) => {
                    let mut regions = Vec::with_capacity(entries.len());
                    for entry in entries {
                        let Value::Map(mut entry) = entry else {
                            return Err(DecodeRefused::WrongType { field: "regions" });
                        };
                        let start = take_unsigned(&mut entry, "start")?;
                        let length = take_unsigned(&mut entry, "length")?;
                        if !entry.is_empty() {
                            return Err(DecodeRefused::UnknownField);
                        }
                        regions.push(Region { start, length });
                    }
                    regions
                }
                Some(_) => return Err(DecodeRefused::WrongType { field: "regions" }),
                None => return Err(DecodeRefused::MissingField { field: "regions" }),
            };
            ProtectionArm::RawCaptureVerified { artifact, regions }
        }
        _ => return Err(DecodeRefused::UnknownTag { field: "arm" }),
    };
    Ok(Record::Protection(
        ProtectionRecord::new(plan, arm).map_err(DecodeRefused::Invalid)?,
    ))
}

fn insert_artifact(map: &mut BTreeMap<String, Value>, artifact: &ProtectionArtifactRef) {
    map.insert(
        "artifact".to_owned(),
        Value::Bytes(artifact.content.0.to_vec()),
    );
    map.insert(
        "store".to_owned(),
        Value::Text(artifact.store.wire_name().to_owned()),
    );
}

fn take_artifact(
    map: &mut BTreeMap<String, Value>,
) -> Result<ProtectionArtifactRef, DecodeRefused> {
    let content = ArtifactHashRef(take_hash(map, "artifact")?);
    let store = match take_text(map, "store")?.as_str() {
        "helper-protection-store" => ArtifactStore::HelperProtectionStore,
        _ => return Err(DecodeRefused::UnknownTag { field: "store" }),
    };
    Ok(ProtectionArtifactRef::new(content, store))
}

fn take_text(
    map: &mut BTreeMap<String, Value>,
    field: &'static str,
) -> Result<String, DecodeRefused> {
    match map.remove(field) {
        Some(Value::Text(text)) => Ok(text),
        Some(_) => Err(DecodeRefused::WrongType { field }),
        None => Err(DecodeRefused::MissingField { field }),
    }
}

fn take_unsigned(
    map: &mut BTreeMap<String, Value>,
    field: &'static str,
) -> Result<u64, DecodeRefused> {
    match map.remove(field) {
        Some(Value::Unsigned(value)) => Ok(value),
        Some(_) => Err(DecodeRefused::WrongType { field }),
        None => Err(DecodeRefused::MissingField { field }),
    }
}

fn take_hash(
    map: &mut BTreeMap<String, Value>,
    field: &'static str,
) -> Result<[u8; 32], DecodeRefused> {
    match map.remove(field) {
        Some(Value::Bytes(bytes)) => hash_bytes(&bytes, field),
        Some(_) => Err(DecodeRefused::WrongType { field }),
        None => Err(DecodeRefused::MissingField { field }),
    }
}

fn hash_bytes(bytes: &[u8], field: &'static str) -> Result<[u8; 32], DecodeRefused> {
    bytes
        .try_into()
        .map_err(|_| DecodeRefused::WrongHashLength { field })
}

fn seq_no(raw: u64) -> Result<SeqNo, DecodeRefused> {
    if raw == 0 {
        return Err(DecodeRefused::Invalid(RecordInvalid::SequenceZero));
    }
    Ok(SeqNo::from_raw(raw))
}

#[cfg(test)]
mod tests;
