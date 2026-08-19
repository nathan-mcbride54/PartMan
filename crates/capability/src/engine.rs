//! The engine core (WP-050 increment 2): CAP-001's conditioning as one
//! entry point, composing decided arms in refusal-precedence order.
//!
//! [`capability`] computes the CAP-003 answer for one operation on one
//! exact target over a decoded snapshot, caller-supplied technology
//! limits, and CAP-004-shaped runtime facts. The composition order is
//! decided text arm by arm: ADR-0011's multipath detection-only rule
//! first (LIN-006 — added in increment 4, whose all-reasons coverage
//! test caught its absence; it precedes protection because LIN-006
//! names the reason this population reports, while the closure refuses
//! the same population anyway — reporting moves, permission never
//! does), then the domain's protection gate, then immutable technology
//! limits (FS-007, statused per ADR-0020), then Section 9's platform
//! floor, then ACC-009's runtime tool preconditions; a pair no
//! arm refuses is `preview` — implemented for planning, apply refused
//! pending CAP-006 qualification evidence, which is CAP-003's own
//! definition of the state this product is in while no apply path
//! exists.
//!
//! Two properties the tests hold rather than this comment:
//!
//! - **CAP-005 agreement.** The protection arm calls the same
//!   `protection_gate` the plan constructor's closure backs, so what the
//!   engine calls plannable, [`PlanStep::mutating`] constructs, and what
//!   it refuses for protection, the constructor refuses on the same
//!   ground — enumerated over every operation/target pair of a fixture
//!   snapshot carrying a permitted device, a refused technology, and an
//!   indeterminate orphan.
//! - **Source classes are never suppressed** (3g's rule, preserved
//!   through composition): a source operation takes no protection
//!   refusal from any verdict, on any target.
//!
//! CAP-001 also names mount state, boot role, and OS identity as
//! conditioning inputs. No decided text consumes them yet, and this
//! module deliberately does not carry them as dead fields — a field the
//! engine reads nothing from would represent conditioning that does not
//! happen. Each arrives with the text that decides its rule (the
//! WP-035 vacuous-state discipline: name the absence, do not ship its
//! shell).

use partman_domain::model::capability::{Operation, OperationClass, protection_gate};
use partman_domain::model::naming::{FileSystemKind, NamingFields, NodeEntry, NodeId};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_domain::model::topology::EdgeKind;

use super::{Capability, Reason, Remediation};

/// The caller-supplied immutable technology limits (FS-007's facts, as
/// WP-035's `facts` surface states them): pairs of file-system kind and
/// the operation that kind can never perform. A limit matches when the
/// **target itself** is a file-system node of the limited kind — the
/// capability question is about the named target, and reach onto a
/// contained file system from a host mutation is the plan layer's
/// affected-set business, not a capability-time inference.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TechnologyLimits {
    limits: Vec<(FileSystemKind, Operation)>,
}

impl TechnologyLimits {
    /// A limit set from the caller's immutable facts.
    #[must_use]
    pub fn new(limits: Vec<(FileSystemKind, Operation)>) -> Self {
        Self { limits }
    }

    fn forbids(&self, kind: &FileSystemKind, operation: Operation) -> bool {
        self.limits
            .iter()
            .any(|(limited, op)| limited == kind && *op == operation)
    }
}

/// One tool the requested operation needs, in the doctor's vocabulary
/// (CAP-004): the caller resolves which tools apply; this engine judges
/// only the state it is handed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolFact {
    /// The tool's name, for remediation text only — never resolved,
    /// launched, or path-searched here.
    pub tool: String,
    /// The doctor-established state.
    pub state: ToolState,
}

/// A required tool's runtime state (ACC-009's two failure classes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolState {
    /// Present at its trusted absolute path, inside the tested range.
    PresentInRange,
    /// Absent.
    Missing,
    /// Present but outside the tested version range.
    OutOfRange,
}

/// Whether the running environment satisfies Section 9's floor for this
/// platform. The floors change only via ADR; the engine may narrow on
/// this fact and may never widen below it.
///
/// **Three arms, deliberately.** A floor is a conjunction (Section 9's
/// Debian/Ubuntu row: distribution and version, kernel, `UDisks2`), and a
/// producer may be able to establish some conjuncts and not others — the
/// Linux read-only adapter reads two of the three from files it has rows
/// for and has no source for the third (WP-L100 increment 5b,
/// `docs/reviews/WP-L100_INCREMENT_5_PLAN_2026-08-19.md` F2). Mapping
/// "could not determine" to [`Self::BelowFloor`] would report a measured
/// shortfall nobody measured; mapping it to [`Self::MeetsFloor`] would
/// widen below the floor, which Section 9 forbids. So an undetermined
/// floor is its own value: the engine treats it exactly as below the
/// floor for status — `blocked` — and its remediation names the conjunct
/// that could not be established, so the answer is honest about what is
/// missing and never a guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformFact {
    /// At or above the floor.
    MeetsFloor,
    /// Below the floor.
    BelowFloor,
    /// The floor could not be determined: at least one conjunct has no
    /// established value. Not below the floor — no measurement said so —
    /// and never met.
    Undetermined {
        /// Which conjunct could not be established, and why, for the
        /// remediation text.
        conjunct: String,
    },
}

/// CAP-004-shaped runtime facts, supplied by the caller (WP-035's doctor
/// today; the platform adapters tomorrow). The engine performs no I/O to
/// gather these and cannot: judging supplied facts is the whole of its
/// runtime reach.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFacts {
    /// The tools this operation needs, with their doctor states.
    pub tools: Vec<ToolFact>,
    /// The Section 9 floor determination.
    pub platform: PlatformFact,
}

impl RuntimeFacts {
    /// Facts describing a clean environment needing no tools — the
    /// identity element of the runtime arms.
    #[must_use]
    pub const fn clean() -> Self {
        Self {
            tools: Vec::new(),
            platform: PlatformFact::MeetsFloor,
        }
    }
}

/// The one caller error this engine distinguishes from an answer: a
/// target the snapshot does not carry. CAP-001 is per **exact** target;
/// an address that resolves to nothing has no capability, honest or
/// otherwise, and inventing one would be an answer about nobody.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownTarget {
    /// The unresolvable address.
    pub target: NodeId,
}

fn entry_fields(snapshot: &TopologySnapshot, target: NodeId) -> Option<&NamingFields> {
    snapshot
        .topology()
        .entries()
        .iter()
        .find(|entry| entry.id() == target)
        .map(|entry| match entry {
            NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
        })
}

/// Whether ADR-0011's detection-only rule scopes this target: the target
/// is itself a multipath node, or a platform-membership edge names it as
/// a recognized member.
fn multipath_scoped(snapshot: &TopologySnapshot, target: NodeId, fields: &NamingFields) -> bool {
    matches!(fields, NamingFields::MultipathNode { .. })
        || snapshot
            .topology()
            .edges()
            .iter()
            .any(|edge| edge.kind == EdgeKind::PlatformMembership && edge.target == target)
}

/// CAP-001's conditioning: the CAP-003 answer for one operation on one
/// exact target, composed in refusal-precedence order — multipath
/// detection-only (ADR-0011), protection, technology limits, platform
/// floor, tool preconditions — with `preview` as the answer no arm
/// refuses (apply stays refused pending CAP-006 evidence, per the
/// increment-1 construction rule).
///
/// # Errors
///
/// [`UnknownTarget`] if the snapshot does not carry the target. That is
/// a caller error, not a capability answer.
pub fn capability(
    operation: Operation,
    target: NodeId,
    snapshot: &TopologySnapshot,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
) -> Result<Capability, UnknownTarget> {
    let Some(fields) = entry_fields(snapshot, target) else {
        return Err(UnknownTarget { target });
    };

    // Arm 1: ADR-0011's detection-only rule (LIN-006). A mutating
    // operation on a multipath node, or on a recognized member — a device
    // some platform-membership edge targets — is CAP-003 `unsupported`
    // with the multipath reason. This arm precedes protection because
    // LIN-006 names the reason this population reports, and the closure
    // refuses the same population anyway — the device-scope transport
    // arm reaches a multipath node as not-positively-local — so
    // precedence here changes the reported reason, never a permission:
    // the plan constructor still refuses these targets on the closure's
    // own ground. Detection-only means source classes pass untouched,
    // and no remedy exists: v1 policy, not a precondition.
    if operation.class() == OperationClass::Mutating && multipath_scoped(snapshot, target, fields) {
        return Ok(Capability::unsupported(
            Reason::MultipathDetectionOnly,
            Remediation::NoneExists,
        ));
    }

    // Arm 2: protection, by the same closure the plan constructor runs.
    // Source classes are never suppressed — the gate returns Clear for
    // them by 3g's own rule, preserved here by calling it unconditionally.
    let gate = protection_gate(snapshot.topology(), snapshot.facts(), target, operation);
    let protection_remediation = match &gate {
        partman_domain::model::capability::ProtectionGate::Blocked { .. } => Remediation::Action(
            "establish the undetermined fact through the helper's evidence contract, then \
             recompute"
                .to_owned(),
        ),
        _ => Remediation::NoneExists,
    };
    if let Some(answer) = Capability::from_protection_gate(&gate, protection_remediation) {
        return Ok(answer);
    }

    // Arm 3: immutable technology limits (FS-007), statused per ADR-0020:
    // `unsupported`, the limit as its explicit reason, and no remedy —
    // exactly, because the limit is immutable.
    if let NamingFields::FileSystem { kind, .. } = fields
        && limits.forbids(kind, operation)
    {
        return Ok(Capability::unsupported(
            Reason::TechnologyLimit,
            Remediation::NoneExists,
        ));
    }

    // Arm 4: the Section 9 floor. Below and undetermined both block under
    // the one floor reason; the remediation says which — a shortfall
    // that was measured, or a conjunct that could not be established.
    match &runtime.platform {
        PlatformFact::MeetsFloor => {}
        PlatformFact::BelowFloor => {
            return Ok(Capability::blocked(
                Reason::PlatformFloor,
                Remediation::Action(
                    "bring the environment to this platform's Section 9 floor".to_owned(),
                ),
            ));
        }
        PlatformFact::Undetermined { conjunct } => {
            return Ok(Capability::blocked(
                Reason::PlatformFloor,
                Remediation::Action(format!(
                    "the Section 9 floor could not be determined ({conjunct}); establish it \
                     through the platform adapter's evidence contract, then recompute"
                )),
            ));
        }
    }

    // Arm 5: ACC-009's tool preconditions, missing before out-of-range so
    // the remediation names the larger obstacle first.
    if let Some(missing) = runtime
        .tools
        .iter()
        .find(|fact| fact.state == ToolState::Missing)
    {
        return Ok(Capability::blocked(
            Reason::ToolMissing,
            Remediation::Action(format!(
                "install {} from the platform's package source at its trusted absolute path",
                missing.tool
            )),
        ));
    }
    if let Some(out_of_range) = runtime
        .tools
        .iter()
        .find(|fact| fact.state == ToolState::OutOfRange)
    {
        return Ok(Capability::blocked(
            Reason::ToolVersionOutOfRange,
            Remediation::Action(format!(
                "update {} into the tested version range",
                out_of_range.tool
            )),
        ));
    }

    // No arm refuses: implemented for planning, apply refused pending
    // qualification evidence (CAP-003's `preview`, the increment-1 rule).
    Ok(Capability::preview(Remediation::Action(
        "qualification evidence for this platform and file-system combination lands in \
         docs/capabilities/ under its own review"
            .to_owned(),
    )))
}
