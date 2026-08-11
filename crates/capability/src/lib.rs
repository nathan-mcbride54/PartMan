//! The WP-050 capability engine's vocabulary (increment 1).
//!
//! CAP-003 requires every capability answer to carry one of four statuses
//! plus a reason and remediation. This increment delivers that vocabulary
//! as types whose construction rules hold the requirement's definitions:
//!
//! - [`Status::Supported`] is reachable only through
//!   [`QualificationEvidence`], which has **no constructor in this
//!   increment** — CAP-003 defines `supported` as apply permitted *backed
//!   by matrix evidence* (CAP-006), the evidence store does not exist
//!   until increment 3, and no apply path exists anywhere in the product.
//!   Unreachable is therefore the correct answer, held by the compiler
//!   rather than by review (the ADR-0012 pattern; see the `compile_fail`
//!   proof on [`QualificationEvidence`]).
//! - The reason vocabulary is a **closed, versioned enum**
//!   ([`REASON_SCHEMA`], MODEL-003) — the home for gated-state reporting
//!   the register's round-three record fixes, carrying at birth exactly
//!   the reasons decided texts name. The protection grounds re-enumerate
//!   `crates/domain`'s closed enums through exhaustive `From` impls, so a
//!   domain arm added later fails compilation here and the vocabulary
//!   version bump becomes a reviewed decision instead of silent drift.
//! - Remediation is structured text carried beside the reason, never
//!   synthesized from it: the caller who knows the precondition states
//!   the remedy, and a reason with no remedy says so explicitly.
//!
//! ## Consumers (CAP-005: one engine, every surface)
//!
//! Three consumer classes take this crate's answers, each under its own
//! package's grant, none with authority over them:
//!
//! - **The CLI (WP-080)** renders answers as advisory UX — CAP-007's
//!   rule: a client cannot upgrade a capability by asserting it, and the
//!   helper trusts only its own recomputation (HLP-002). The chassis's
//!   `capabilities` typed refusal names this package as the engine
//!   deliverer; replacing that refusal with real payloads is WP-080's
//!   work, serializing [`Reason`] under [`REASON_SCHEMA`].
//! - **The planner (WP-060)** conditions planning on answers: `preview`
//!   permits planning and simulation; `unsupported` and `blocked` refuse
//!   the affected write step (ACC-009's planner half). Answers never
//!   construct steps — `PlanStep::mutating` is the sole constructor and
//!   re-runs the same closure, which is why the CAP-005 agreement is
//!   enumerable rather than asserted.
//! - **The platform adapters (WP-W100/WP-L100/WP-M100)** produce what
//!   the engine consumes — decoded snapshots and CAP-004-shaped
//!   [`engine::RuntimeFacts`] — and render per-platform answers.
//!   Adapters never compute a verdict of their own.
//!
//! What this increment deliberately does not do: no engine computation
//! (increment 2), no evidence store (increment 3), no status coupling for
//! [`Reason::TechnologyLimit`] — FS-007's words ("as explicit blocked
//! reasons") and CAP-003's `blocked` definition ("implemented, but a
//! runtime precondition fails") assign that case different statuses, the
//! Section 1.11 shape, and the conflict is to be filed on the register
//! rather than decided silently here. The decided couplings — 3g's
//! refusals-to-`unsupported` and indeterminacies-to-`blocked`, LIN-006's
//! multipath `unsupported`, ACC-009's tool cases as `blocked` — are
//! carried by [`Capability::from_protection_gate`] and the per-status
//! constructors' documented grounds.

use partman_domain::model::capability::ProtectionGate;
use partman_domain::model::protection::{IndeterminateGround, RefusalGround};

pub mod engine;
pub mod store;

#[cfg(test)]
mod engine_tests;
#[cfg(test)]
mod store_tests;
#[cfg(test)]
mod tests;

/// The reason vocabulary's schema identity (MODEL-003).
pub const REASON_SCHEMA: &str = "partman.capability.reason";
/// The current reason vocabulary version. Adding, removing, or re-meaning
/// a variant is a version bump decided in review, never a silent edit.
pub const REASON_SCHEMA_VERSION: u64 = 1;

/// CAP-003's four statuses. The bare enum carries no authority: a
/// [`Capability`] is constructible only through the per-status
/// constructors, and `Supported` only through evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Apply permitted, backed by CAP-006 matrix evidence.
    Supported,
    /// Planning and simulation permitted; apply refused pending
    /// qualification evidence. Labeled as such by every surface.
    Preview,
    /// The product does not implement the operation for this target.
    Unsupported,
    /// Implemented, but a runtime precondition fails.
    Blocked,
}

/// Why protection refuses a pair — `crates/domain`'s closed refusal
/// grounds, re-enumerated so this vocabulary's closure is its own and a
/// domain addition fails compilation here (see [`From<RefusalGround>`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectionRefusalReason {
    /// ZFS: detect pools and members; never mutate.
    Zfs,
    /// Windows Storage Spaces pool or space structure.
    StorageSpaces,
    /// Windows dynamic disks (LDM).
    Ldm,
    /// Apple Fusion: an APFS container self-reporting two or more members.
    Fusion,
    /// A recognized remote transport.
    RemoteTransport,
    /// Consumed by a refused consumer or produced by a refused producer.
    InheritedFromConsumerOrProducer,
    /// The node inherits its root device's device-scope refusal.
    InheritedDeviceScope,
}

impl From<RefusalGround> for ProtectionRefusalReason {
    fn from(ground: RefusalGround) -> Self {
        match ground {
            RefusalGround::Zfs => Self::Zfs,
            RefusalGround::StorageSpaces => Self::StorageSpaces,
            RefusalGround::Ldm => Self::Ldm,
            RefusalGround::Fusion => Self::Fusion,
            RefusalGround::RemoteTransport => Self::RemoteTransport,
            RefusalGround::InheritedFromConsumerOrProducer => Self::InheritedFromConsumerOrProducer,
            RefusalGround::InheritedDeviceScope => Self::InheritedDeviceScope,
        }
    }
}

/// Why protection is indeterminate — the domain's closed indeterminacy
/// grounds, re-enumerated for the same reason as the refusals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectionIndeterminacyReason {
    /// An unrecognized technology, kind, or discriminant.
    Unrecognized,
    /// A signature with no observed consumer — the orphan arm.
    OrphanSignature,
    /// A collision group: no member is individually addressable.
    CollisionGroup,
    /// A fact the deciding arm needs is absent.
    MissingFact,
    /// The node inherits its root device's device-scope indeterminacy.
    InheritedDeviceScope,
}

impl From<IndeterminateGround> for ProtectionIndeterminacyReason {
    fn from(ground: IndeterminateGround) -> Self {
        match ground {
            IndeterminateGround::Unrecognized => Self::Unrecognized,
            IndeterminateGround::OrphanSignature => Self::OrphanSignature,
            IndeterminateGround::CollisionGroup => Self::CollisionGroup,
            IndeterminateGround::MissingFact => Self::MissingFact,
            IndeterminateGround::InheritedDeviceScope => Self::InheritedDeviceScope,
        }
    }
}

/// The closed reason vocabulary ([`REASON_SCHEMA`] version
/// [`REASON_SCHEMA_VERSION`]). Every variant is a decided text's:
/// the two protection arms are ADR-0018's closure grounds; multipath is
/// ADR-0011's detection-only rule (LIN-006's `unsupported` reason); the
/// two tool arms are ACC-009's; the technology limit is FS-007's; the
/// platform floor is Section 9's; pending-evidence is CAP-003's own
/// `preview` ground. Specifics — which tool, which limit, which floor —
/// live in [`Remediation`] text, not in variant payloads, so the closure
/// versions cleanly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// A Section 2.1 protection refusal (ADR-0018).
    ProtectionRefused {
        /// The refusing arm's ground.
        ground: ProtectionRefusalReason,
    },
    /// Protection could not be determined (ADR-0018's residual).
    ProtectionIndeterminate {
        /// What could not be determined.
        cause: ProtectionIndeterminacyReason,
    },
    /// Multipath is detection-only in v1 (ADR-0011, LIN-006).
    MultipathDetectionOnly,
    /// A required tool is absent (ACC-009).
    ToolMissing,
    /// A required tool is outside its tested version range (ACC-009).
    ToolVersionOutOfRange,
    /// An immutable technology limit (FS-007). Its status coupling is
    /// deliberately unasserted in this increment: FS-007's wording and
    /// CAP-003's `blocked` definition disagree, and the conflict is the
    /// register's to decide, not this enum's to bury.
    TechnologyLimit,
    /// The environment is below a Section 9 platform floor.
    PlatformFloor,
    /// Implemented and planable, but no CAP-006 qualification evidence
    /// backs apply — CAP-003's `preview` ground.
    UnqualifiedPendingEvidence,
    /// Apply is backed by a stored CAP-006 qualification row. Appears
    /// only on `supported` answers, which are constructible only through
    /// [`QualificationEvidence`] — so this reason cannot be asserted, it
    /// can only be carried by an answer the evidence built.
    QualifiedByEvidence,
}

impl Reason {
    /// Every variant, for closure and coverage enumeration in tests and,
    /// later, in the CAP-006 store's all-reasons check. One entry per
    /// variant with representative protection grounds; the protection
    /// sub-enums enumerate exhaustively in their own tests.
    #[must_use]
    pub const fn all_variants() -> &'static [Self; 9] {
        &[
            Self::ProtectionRefused {
                ground: ProtectionRefusalReason::Zfs,
            },
            Self::ProtectionIndeterminate {
                cause: ProtectionIndeterminacyReason::Unrecognized,
            },
            Self::MultipathDetectionOnly,
            Self::ToolMissing,
            Self::ToolVersionOutOfRange,
            Self::TechnologyLimit,
            Self::PlatformFloor,
            Self::UnqualifiedPendingEvidence,
            Self::QualifiedByEvidence,
        ]
    }
}

/// CAP-006 qualification evidence: the value that makes `supported`
/// constructible, and nothing else does.
///
/// **No constructor exists in this increment.** The evidence store and
/// its loader are increment 3; until a qualifying row can be loaded from
/// `docs/capabilities/`, `supported` is unreachable — which is CAP-003's
/// own definition doing the gating, since no apply path exists anywhere
/// in the product yet. Nothing may synthesize, default, or test-inject
/// this value into the shipped path.
///
/// The unreachability is compiler-held (the ADR-0012 pattern):
///
/// ```compile_fail
/// // There is no way to name a value of this type into existence: the
/// // field is private and no function returns it.
/// let evidence = partman_capability::QualificationEvidence { _sealed: () };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct QualificationEvidence {
    _sealed: (),
}

/// Remediation: what the asker can do about the answer, stated by the
/// caller who knows the precondition — never synthesized from the reason.
/// A reason with no remedy states that explicitly rather than omitting
/// the field (CAP-003 requires the answer to carry remediation).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remediation {
    /// A concrete action the user can take.
    Action(String),
    /// No remedy exists: the answer is structural (a Section 2.1
    /// refusal, an immutable limit). Stating so is the remediation.
    NoneExists,
}

/// One CAP-003 capability answer: status plus reason plus remediation.
/// Fields are private; the per-status constructors are the only route in,
/// and the decided couplings are theirs to hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capability {
    status: Status,
    reason: Reason,
    remediation: Remediation,
}

impl Capability {
    /// `supported`: apply permitted, backed by matrix evidence. Uncallable
    /// until [`QualificationEvidence`] gains its store-loading constructor
    /// in increment 3 — see the type's `compile_fail` proof.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the evidence token is deliberately consumed: qualification                   backs one answer, and a reusable reference would let one                   loaded row dress arbitrarily many answers as supported"
    )]
    #[must_use]
    pub fn supported(evidence: QualificationEvidence, remediation: Remediation) -> Self {
        let QualificationEvidence { _sealed: () } = evidence;
        Self {
            status: Status::Supported,
            reason: Reason::QualifiedByEvidence,
            remediation,
        }
    }

    /// `preview`: planning permitted, apply refused pending qualification
    /// (CAP-003's own ground).
    #[must_use]
    pub fn preview(remediation: Remediation) -> Self {
        Self {
            status: Status::Preview,
            reason: Reason::UnqualifiedPendingEvidence,
            remediation,
        }
    }

    /// `unsupported` with the reason the deciding text assigns.
    ///
    /// # Panics
    ///
    /// If handed [`Reason::QualifiedByEvidence`] — that reason cannot be
    /// asserted, only carried by an answer the evidence built.
    #[must_use]
    pub fn unsupported(reason: Reason, remediation: Remediation) -> Self {
        assert_ne!(
            reason,
            Reason::QualifiedByEvidence,
            "QualifiedByEvidence is carried by evidence, never asserted"
        );
        Self {
            status: Status::Unsupported,
            reason,
            remediation,
        }
    }

    /// `blocked` with the reason the deciding text assigns.
    ///
    /// # Panics
    ///
    /// If handed [`Reason::QualifiedByEvidence`] — that reason cannot be
    /// asserted, only carried by an answer the evidence built.
    #[must_use]
    pub fn blocked(reason: Reason, remediation: Remediation) -> Self {
        assert_ne!(
            reason,
            Reason::QualifiedByEvidence,
            "QualifiedByEvidence is carried by evidence, never asserted"
        );
        Self {
            status: Status::Blocked,
            reason,
            remediation,
        }
    }

    /// The decided protection coupling, exactly 3g's rule: a refusal is
    /// CAP-003 `unsupported`, an indeterminacy is CAP-003 `blocked`, and
    /// `Clear` is no answer at all — protection not gating a pair says
    /// nothing about tools, evidence, floors, or limits, so the engine
    /// keeps composing and no `Capability` exists yet.
    #[must_use]
    pub fn from_protection_gate(gate: &ProtectionGate, remediation: Remediation) -> Option<Self> {
        match gate {
            ProtectionGate::Clear => None,
            ProtectionGate::Unsupported { ground } => Some(Self::unsupported(
                Reason::ProtectionRefused {
                    ground: (*ground).into(),
                },
                remediation,
            )),
            ProtectionGate::Blocked { cause } => Some(Self::blocked(
                Reason::ProtectionIndeterminate {
                    cause: (*cause).into(),
                },
                remediation,
            )),
        }
    }

    /// The answer's status.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// The answer's reason.
    #[must_use]
    pub const fn reason(&self) -> Reason {
        self.reason
    }

    /// The answer's remediation.
    #[must_use]
    pub const fn remediation(&self) -> &Remediation {
        &self.remediation
    }
}
