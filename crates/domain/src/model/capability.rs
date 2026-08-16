//! The protection gate on capability (WP-010 increment 3g; ADR-0018's
//! canonical-step rule, CAP-001, CAP-002, CAP-005, CAP-007).
//!
//! CAP-001 computes capability per exact target with no plan in scope;
//! ADR-0018 resolves the round-three disagreement hazard by construction:
//! **the capability engine and the plan constructor run the same closure.**
//! Each mutating operation defines a canonical effect-table entry over its
//! target — the operation's minimal invariant ranges, derivable with no
//! plan — and the protection gate is the closure's answer over that entry.
//! A real plan step declares real ranges and re-runs the same closure
//! authoritatively, so the two surfaces cannot disagree on a
//! target/operation pair (CAP-005).
//!
//! This module computes the **protection contribution** to capability
//! only. The full CAP-003 status — tool presence, version gates, matrix
//! evidence, `preview` qualification — is WP-050's engine, which consumes
//! this gate; per CAP-007 every client-shown status is advisory and the
//! helper trusts only its own recomputation.
//!
//! Source-class operations are never suppressed by a verdict (ADR-0018:
//! `detect` on a Storage Spaces pool stays honest per WIN-003, WIN-004's
//! copy-off-LDM stays advertised), under the source-access predicate the
//! plan layer enforces: a refused or indeterminate node may be an operand
//! only of steps that are source steps in their entirety.

use super::naming::NodeId;
use super::protection::{
    self, Facts, HostRange, IndeterminateGround, RefusalGround, StepRanges, Verdict,
};
use super::topology::Topology;

/// CAP-002's operation list, modelled separately as required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Detect a structure. Source class.
    Detect,
    /// Read content. Source class.
    Read,
    /// Create a structure in free space.
    Create,
    /// Grow a structure.
    Grow,
    /// Shrink a structure.
    Shrink,
    /// Move a structure.
    Move,
    /// Copy a structure, as the source operand. The destination side is a
    /// distinct mutating operation on the destination target.
    Copy,
    /// Check health, in read-only mode. Source class.
    Check,
    /// Repair a structure.
    Repair,
    /// Change a label.
    Label,
    /// Change a UUID or equivalent identifier.
    Uuid,
    /// Encrypt in place.
    Encrypt,
    /// Decrypt in place.
    Decrypt,
    /// Destroy content (DIA-005 distinguishes the wipe family's members;
    /// the protection gate treats them alike).
    Wipe,
}

/// ADR-0018's operation classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationClass {
    /// Never suppressed by a protection verdict.
    Source,
    /// Gated by the closure over the canonical entry.
    Mutating,
}

impl Operation {
    /// The operation's class.
    #[must_use]
    pub const fn class(self) -> OperationClass {
        match self {
            Self::Detect | Self::Read | Self::Check | Self::Copy => OperationClass::Source,
            Self::Create
            | Self::Grow
            | Self::Shrink
            | Self::Move
            | Self::Repair
            | Self::Label
            | Self::Uuid
            | Self::Encrypt
            | Self::Decrypt
            | Self::Wipe => OperationClass::Mutating,
        }
    }

    /// Every operation, for exhaustive tests.
    #[must_use]
    pub const fn all() -> &'static [Self; 14] {
        &[
            Self::Detect,
            Self::Read,
            Self::Create,
            Self::Grow,
            Self::Shrink,
            Self::Move,
            Self::Copy,
            Self::Check,
            Self::Repair,
            Self::Label,
            Self::Uuid,
            Self::Encrypt,
            Self::Decrypt,
            Self::Wipe,
        ]
    }
}

/// The protection contribution to a target/operation capability.
///
/// `Clear` is not a CAP-003 `supported` claim — it says protection does
/// not gate the pair; WP-050's engine layers tool, version, and evidence
/// gates on top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtectionGate {
    /// Protection does not gate this pair.
    Clear,
    /// A Section 2.1 refusal: CAP-003 `unsupported`, the reason citing
    /// the refusing ground.
    Unsupported {
        /// The refusing arm's ground.
        ground: RefusalGround,
    },
    /// An indeterminacy: CAP-003 `blocked`, remediable.
    Blocked {
        /// What could not be determined.
        cause: IndeterminateGround,
    },
}

/// The canonical effect-table entry for an operation over a target: the
/// ranges the operation has over it that are derivable from the body's
/// own facts, with no plan and no request in scope.
///
/// **Not minimal, and never was.** Earlier revisions of this comment
/// called these "the operation's minimal invariant ranges" and credited
/// the phrase to "ADR-0018's canonical-step rule". ADR-0018 has no such
/// rule and does not use the phrase; it is ADR-0042:23's gloss, repeated
/// here and inherited by issue #392's filing. The entry was already
/// non-minimal before that: ADR-0038 made `Shrink` and `Move` declare the
/// whole target extent rather than the freed tail, because this function
/// takes no request and cannot know a shrink's new length, and the
/// truthful entry was measured to be a safety regression. ADR-0048 makes
/// it less minimal again. The invariant that does hold is conservatism —
/// an entry may over-approximate what the operation touches, never
/// under-approximate it.
///
/// A destructive operation's canonical entry destroys the target's
/// extent, or — where the target declares none — its whole frame
/// (ADR-0048, issue #392). The six write operations declare the target's
/// own extent as written, filtered to exclude a frame root's self-extent,
/// which is what §2.1 forbids naming wholesale (ADR-0042); despite an
/// earlier revision of this comment, a create declares no consumed range
/// here at all, because the free range it takes needs the request. Content
/// operations declare no host-range effects — the plan step's declared
/// ranges are authoritative and re-run the same closure.
#[must_use]
pub fn canonical_ranges(operation: Operation, target: NodeId, facts: &Facts) -> StepRanges {
    let extent = facts.extents.get(&target).copied();
    let destroyed_target = |extent: Option<HostRange>| StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: extent.into_iter().collect(),
    };
    match operation {
        // ADR-0038: the release operations seed the closure. ADR-0018
        // defines the destroyed class as releases — "a deleted
        // partition's extent, a shrink's truncated tail, a move's
        // source extent at commit" — and of the mutating operations
        // only Shrink and Move are named there beside the two that
        // destroy outright. The entry is the whole target extent, not
        // the range actually freed: this function takes no request
        // parameters and so cannot know a shrink's new length, and the
        // truthful entry was measured to be a safety regression — it
        // leaves a pool unreached where the conservative one refuses.
        // The plan layer, which does know its geometry, still computes
        // the real freed range.
        Operation::Wipe | Operation::Encrypt | Operation::Move | Operation::Shrink => {
            // ADR-0048, issue #392. A target that declares no extent —
            // a volume, an aggregate, an encryption layer, a multipath
            // node, for which `may_carry_extent` is false — had no range
            // here at all, so the closure never saw it destroyed: the
            // table it carries was reached as content and never
            // destroyed, ADR-0043's release never fired, and
            // `Wipe(volume)` gated `Clear` over a live pool. Its entry
            // is its whole frame, which is the honest reading: wiping a
            // volume destroys everything expressed in the volume's own
            // address space, and nothing else — `HostRange::intersects`
            // is frame-equal, so this range cannot touch another frame,
            // and `u64::MAX` is safe under the saturating arithmetic
            // `intersects` and `contains` both use.
            destroyed_target(extent.or(Some(HostRange {
                host: target,
                start: 0,
                length: u64::MAX,
            })))
        }
        // Issue #353. §2.1: "Table writes target the table node's own
        // extents, never the parent device wholesale." These six write,
        // and for a target below a frame root the entry is the target's
        // own extent — an over-approximation for a partition, whose
        // grow writes one table entry and whose label writes inside its
        // own bytes, but bounded by the target and reach-equivalent to
        // ADR-0039's carried content, and what the plan layer's
        // touched-device derivation reads. For a frame root — a target
        // whose extent is expressed in its own address space, a device —
        // the delivered entry was the parent device wholesale, in as
        // many words what the sentence forbids, and the whole-disk gates
        // refused *because* of it, by byte scan alone. Such a target
        // declares no written range: it seeds the set by identity and
        // ADR-0039's descent reaches what it carries, the table and
        // whatever is hosted directly on it. What a truthful per-kind
        // entry would be — the host's table extents for a create, one
        // entry for a grow — needs the request or the topology, neither
        // of which this function has; the plan step's declared ranges,
        // which do, are authoritative and re-run the same closure.
        Operation::Grow
        | Operation::Create
        | Operation::Repair
        | Operation::Label
        | Operation::Uuid
        | Operation::Decrypt => StepRanges {
            written_table_extents: extent
                .into_iter()
                .filter(|extent| extent.host != target)
                .collect(),
            consumed: vec![],
            destroyed: vec![],
        },
        Operation::Detect | Operation::Read | Operation::Check | Operation::Copy => {
            StepRanges::default()
        }
    }
}

/// The protection gate for a target/operation pair, computed by the same
/// closure the plan constructor runs (CAP-005 agreement by construction).
#[must_use]
pub fn protection_gate(
    topology: &Topology,
    facts: &Facts,
    target: NodeId,
    operation: Operation,
) -> ProtectionGate {
    if operation.class() == OperationClass::Source {
        return ProtectionGate::Clear;
    }
    let ranges = canonical_ranges(operation, target, facts);
    match protection::step_constructs(topology, facts, target, &ranges) {
        Ok(_) => ProtectionGate::Clear,
        Err(refusal) => match refusal.verdict {
            Verdict::Refused { ground } => ProtectionGate::Unsupported { ground },
            Verdict::Indeterminate { cause } => ProtectionGate::Blocked { cause },
            Verdict::Permitted => ProtectionGate::Clear,
        },
    }
}
