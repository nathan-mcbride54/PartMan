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
        Operation::Wipe | Operation::Encrypt => destroyed_target(extent),
        Operation::Move
        | Operation::Shrink
        | Operation::Grow
        | Operation::Create
        | Operation::Repair
        | Operation::Label
        | Operation::Uuid
        | Operation::Decrypt => StepRanges {
            written_table_extents: extent.into_iter().collect(),
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
