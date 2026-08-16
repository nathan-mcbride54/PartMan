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

/// The canonical effect-table entry for an operation over a target
/// (ADR-0018's canonical-step rule): the operation's minimal invariant
/// ranges, derivable from the body's own facts with no plan in scope.
///
/// A destructive operation's canonical entry destroys the target's
/// extent; a create writes the host's table extents and consumes an
/// unspecified free range, which by the constructor's own rule hosts
/// nothing and therefore cannot hide a reach; content operations declare
/// no host-range effects at capability time — the plan step's declared
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
            destroyed_target(extent)
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
