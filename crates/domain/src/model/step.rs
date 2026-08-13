//! The mutating plan step and its sole constructor (WP-010 increment 3h;
//! ADR-0012's axis discharged, ADR-0018's acknowledgment vocabulary).
//!
//! A [`PlanStep`] for a target the closure refuses **cannot exist as a
//! value**: the fields are private, the only constructor runs the closure,
//! and a refused reach returns a typed error instead of a step. That is
//! ADR-0012's unrepresentability commitment at the type layer — a mutating
//! sentence naming a Section 2.1 non-goal node has no spelling — with the
//! helper's independent recomputation retained as the unweakened second
//! layer at validation (a later slice's boundary).
//!
//! The construction-refusal proof, in the compile-fail pattern the CLI
//! chassis set:
//!
//! ```compile_fail
//! use partman_domain::model::step::PlanStep;
//! use partman_domain::model::protection::StepRanges;
//!
//! // A step cannot be forged by literal: the fields are private, and no
//! // constructor exists that skips the closure.
//! let forged = PlanStep {
//!     target: todo!(),
//!     ranges: StepRanges::default(),
//!     affected: todo!(),
//! };
//! ```
//!
//! The acknowledgment vocabulary is ADR-0018's, closed: an acknowledgment
//! converts exactly one *indeterminate* arm on exactly one named node, and
//! **a refused node has no acknowledgment** — the consumed-member case is
//! deliberately unrepresentable, which is what separates this from
//! PART-014's bypassable "without an explicit supported plan" gloss.

use std::collections::BTreeSet;
use std::fmt;

use super::naming::NodeId;
use super::protection::{
    HostRange, IndeterminateGround, StepRanges, Verdict, affected_set, node_verdict,
};
use super::snapshot::TopologySnapshot;

/// PLAN-004's ordinal severity scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// 0 — no change to storage.
    Informational,
    /// 1 — fully undoable via an emitted reversal plan.
    Reversible,
    /// 2 — interrupts service with no expected data loss.
    Disruptive,
    /// 3 — data is relocated or transformed; loss possible on failure.
    DataMoving,
    /// 4 — data is intentionally destroyed.
    Destructive,
}

/// PLAN-004's orthogonal step flags — five booleans deliberately,
/// mirroring the requirement's own enumeration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct StepFlags {
    /// Touches encryption, keys, or authorization state.
    pub security_sensitive: bool,
    /// Cannot be undone once started.
    pub irreversible_after_start: bool,
    /// Requires the target offline.
    pub requires_offline: bool,
    /// Requires a reboot.
    pub requires_reboot: bool,
    /// Requires the rescue environment.
    pub requires_rescue: bool,
}

/// A step's declared risk (PLAN-004): severity plus orthogonal flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepRisk {
    /// The ordinal severity.
    pub severity: Severity,
    /// The orthogonal flags.
    pub flags: StepFlags,
}

/// ADR-0024's typed step class: the REC-001 repair family is a step
/// class, never an intent flag an ordinary operation can wear — the
/// state-selected protection arms attach to the type, per the
/// safety-is-computed-never-declared discipline (ADR-0012/0018).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepClass {
    /// An ordinary mutating step.
    #[default]
    Ordinary,
    /// A step of the typed REC-001 table-repair family: the only class
    /// whose `Indeterminate`-table protection arm is the verified raw
    /// capture, and the only class that can carry the capture-impossible
    /// acknowledgment (ADR-0024).
    TableRepair,
}

/// ADR-0018's closed acknowledgment vocabulary, extended by ADR-0024's
/// capture-impossible entry. Each entry names the exact node it covers;
/// an acknowledgment for one node covers no other, and no entry exists
/// that covers a refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Acknowledgment {
    /// The release acknowledgment: the user explicitly releases an
    /// orphan or released signature — an observed signature with no
    /// observed consumer — accepting that destroying it makes its
    /// technology's structure unimportable. Recorded at plan creation
    /// under UI-009 typed confirmation; re-derived at validation, where
    /// an acknowledgment whose object turns out consumed diverges and
    /// rejects.
    Release {
        /// The exact signature node released.
        signature: NodeId,
    },
    /// The opaque-destruction acknowledgment (FS-010 plus the opacity
    /// statement) for arms this crate does not yet model; carried in the
    /// vocabulary so the set stays closed, refusing at construction
    /// until its arm exists.
    OpaqueDestruction {
        /// The locked layer acknowledged.
        layer: NodeId,
    },
    /// The identity-bound-restore acknowledgment for `Indeterminate`
    /// tables (REC-001's family). Its arm exists since ADR-0024: lawful
    /// exactly on a [`StepClass::TableRepair`] step covering a device
    /// whose authored table state is `Indeterminate`.
    IdentityBoundRestore {
        /// The table restored.
        table: NodeId,
    },
    /// ADR-0024's capture-impossible acknowledgment: the user's
    /// separately supported recovery strategy (Section 12's clause),
    /// recorded at plan creation and naming the exact uncapturable
    /// regions. Lawful only on a [`StepClass::TableRepair`] step
    /// covering an `Indeterminate`-state device — unconstructible
    /// outside the typed repair family — and never a mid-flight prompt.
    UncapturableRegions {
        /// The device whose pre-state cannot be preserved.
        table: NodeId,
        /// The exact uncapturable regions: on the covered device,
        /// strictly ascending, non-overlapping, nonzero.
        regions: Vec<HostRange>,
    },
}

impl Acknowledgment {
    const fn covers(&self) -> NodeId {
        match self {
            Self::Release { signature } => *signature,
            Self::OpaqueDestruction { layer } => *layer,
            Self::IdentityBoundRestore { table } | Self::UncapturableRegions { table, .. } => {
                *table
            }
        }
    }
}

/// The region discipline ADR-0024's acknowledgment names share with the
/// journal's raw-capture record: on the covered device, strictly
/// ascending, non-overlapping, lengths nonzero, and at least one.
fn regions_well_formed(covered: NodeId, regions: &[HostRange]) -> bool {
    if regions.is_empty() {
        return false;
    }
    let mut previous_end: Option<u64> = None;
    for region in regions {
        if region.host != covered || region.length == 0 {
            return false;
        }
        if let Some(end) = previous_end
            && region.start < end
        {
            return false;
        }
        previous_end = Some(region.start.saturating_add(region.length));
    }
    true
}

/// A step precondition (Section 6's step-preconditions item, ADR-0022's
/// two-time truthfulness): body content a validation boundary re-checks
/// against the snapshot it binds, so a claim that was true at emission
/// refuses once the world has moved instead of silently becoming a
/// different plan. The vocabulary is closed; a kind exists because a
/// recorded decision needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precondition {
    /// The named region hosts nothing: no authenticated fact places any
    /// node (other than the host itself) on any of its bytes. The
    /// shrink-back shape: a grow's reversal is truthful exactly while
    /// nothing sits on the reclaimed tail of the target's own address
    /// space.
    RegionUnoccupied {
        /// The region that must be empty.
        region: HostRange,
    },
    /// The named node's entire address space hosts nothing. ADR-0022's
    /// named fixture — a reversal that deletes a created structure is
    /// metadata-only exactly while the structure holds no user data,
    /// and refuses once anything lands in it.
    HostUnoccupied {
        /// The node whose address space must be empty.
        host: NodeId,
    },
}

impl Precondition {
    /// The first node violating this precondition in the given
    /// snapshot's authenticated facts, or `None` where it holds.
    #[must_use]
    pub fn violated_by(&self, snapshot: &TopologySnapshot) -> Option<NodeId> {
        match self {
            Self::RegionUnoccupied { region } => snapshot
                .facts()
                .extents
                .iter()
                .find(|(node, extent)| {
                    **node != region.host
                        && extent.host == region.host
                        && extent.start < region.start.saturating_add(region.length)
                        && region.start < extent.start.saturating_add(extent.length)
                })
                .map(|(node, _)| *node),
            Self::HostUnoccupied { host } => snapshot
                .facts()
                .extents
                .iter()
                .find(|(node, extent)| **node != *host && extent.host == *host)
                .map(|(node, _)| *node),
        }
    }
}

/// A mutating plan step: target, declared ranges, and the affected set
/// the closure computed. Fields are private; [`PlanStep::mutating`] is
/// the only way to obtain one (preconditions attach afterward — they
/// only ever make validation stricter, so they need no closure run).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanStep {
    target: NodeId,
    ranges: StepRanges,
    affected: BTreeSet<NodeId>,
    acknowledgments: Vec<Acknowledgment>,
    risk: StepRisk,
    preconditions: Vec<Precondition>,
    class: StepClass,
}

/// Whether one acknowledgment lawfully covers its node on a step of the
/// given class (ADR-0018's law, extended by ADR-0024's arms): `Release`
/// converts exactly the orphan-signature indeterminacy;
/// `IdentityBoundRestore` and `UncapturableRegions` attach exactly to
/// the typed table-repair family over a device whose authored table
/// state is `Indeterminate` — with the capture-impossible entry also
/// required to name well-formed regions on that device. No entry covers
/// a refusal, and `OpaqueDestruction`'s arm still does not exist.
fn acknowledgment_lawful(
    acknowledgment: &Acknowledgment,
    verdict: &Verdict,
    table_state: Option<&super::identity::TableState>,
    class: StepClass,
) -> bool {
    match acknowledgment {
        Acknowledgment::Release { .. } => matches!(
            verdict,
            Verdict::Indeterminate {
                cause: IndeterminateGround::OrphanSignature
            }
        ),
        Acknowledgment::OpaqueDestruction { .. } => false,
        Acknowledgment::IdentityBoundRestore { .. } => {
            class == StepClass::TableRepair
                && matches!(
                    table_state,
                    Some(super::identity::TableState::Indeterminate { .. })
                )
        }
        Acknowledgment::UncapturableRegions { table, regions } => {
            class == StepClass::TableRepair
                && matches!(
                    table_state,
                    Some(super::identity::TableState::Indeterminate { .. })
                )
                && regions_well_formed(*table, regions)
        }
    }
}

impl PlanStep {
    /// The sole constructor: run the closure over the snapshot's own
    /// authenticated facts and refuse any non-permitted reach that no
    /// acknowledgment lawfully covers. The step is
    /// [`StepClass::Ordinary`]; the typed repair family constructs
    /// through [`Self::mutating_classed`], which this delegates to.
    ///
    /// # Errors
    ///
    /// [`StepRefusal`] naming the first rule violated.
    pub fn mutating(
        snapshot: &TopologySnapshot,
        target: NodeId,
        ranges: StepRanges,
        acknowledgments: Vec<Acknowledgment>,
        risk: StepRisk,
    ) -> Result<Self, StepRefusal> {
        Self::mutating_classed(
            snapshot,
            target,
            ranges,
            acknowledgments,
            risk,
            StepClass::Ordinary,
        )
    }

    /// The sole constructor's classed form (ADR-0024): the same closure
    /// and the same acknowledgment law, with the step's typed class
    /// deciding which acknowledgment arms exist.
    ///
    /// A `Refused` node in the affected set refuses **regardless of
    /// acknowledgments** — no parameter of this function can express
    /// permission for it. An `Indeterminate` node constructs only when
    /// an acknowledgment of the matching kind names exactly that node
    /// (the orphan-signature arm, [`Acknowledgment::Release`]). The
    /// table-state acknowledgments (`IdentityBoundRestore`,
    /// `UncapturableRegions`) are lawful exactly on the typed
    /// table-repair family over an `Indeterminate`-state device —
    /// unconstructible outside it.
    ///
    /// # Errors
    ///
    /// [`StepRefusal`] naming the first rule violated.
    pub fn mutating_classed(
        snapshot: &TopologySnapshot,
        target: NodeId,
        ranges: StepRanges,
        acknowledgments: Vec<Acknowledgment>,
        risk: StepRisk,
        class: StepClass,
    ) -> Result<Self, StepRefusal> {
        let topology = snapshot.topology();
        let facts = snapshot.facts();
        for acknowledgment in &acknowledgments {
            let covered = acknowledgment.covers();
            let verdict = node_verdict(topology, facts, covered);
            if !acknowledgment_lawful(
                acknowledgment,
                &verdict,
                facts.table_states.get(&covered),
                class,
            ) {
                return Err(StepRefusal::UnlawfulAcknowledgment {
                    node: covered,
                    verdict,
                });
            }
        }
        let affected = affected_set(topology, facts, target, &ranges);
        for node in &affected {
            let verdict = node_verdict(topology, facts, *node);
            match verdict {
                Verdict::Permitted => {}
                Verdict::Refused { .. } => {
                    return Err(StepRefusal::Reached {
                        node: *node,
                        verdict,
                    });
                }
                Verdict::Indeterminate { ref cause } => {
                    let covered = matches!(cause, IndeterminateGround::OrphanSignature)
                        && acknowledgments.iter().any(|acknowledgment| {
                            matches!(acknowledgment, Acknowledgment::Release { signature } if signature == node)
                        });
                    if !covered {
                        return Err(StepRefusal::Reached {
                            node: *node,
                            verdict,
                        });
                    }
                }
            }
        }
        Ok(Self {
            target,
            ranges,
            affected,
            acknowledgments,
            risk,
            preconditions: vec![],
            class,
        })
    }

    /// The step's typed class (ADR-0024).
    #[must_use]
    pub const fn class(&self) -> StepClass {
        self.class
    }

    /// Attach preconditions to a constructed step. Additive and
    /// closure-free deliberately: a precondition can only make a
    /// validation boundary refuse more, never reach further, so the
    /// sole-constructor law is untouched.
    #[must_use]
    pub fn with_preconditions(mut self, preconditions: Vec<Precondition>) -> Self {
        self.preconditions = preconditions;
        self
    }

    /// The step's preconditions, re-checked at every validation
    /// boundary against the snapshot it binds.
    #[must_use]
    pub fn preconditions(&self) -> &[Precondition] {
        &self.preconditions
    }

    /// The step's declared risk (PLAN-004).
    #[must_use]
    pub const fn risk(&self) -> StepRisk {
        self.risk
    }

    /// The step's target.
    #[must_use]
    pub const fn target(&self) -> NodeId {
        self.target
    }

    /// The declared range sets.
    #[must_use]
    pub const fn ranges(&self) -> &StepRanges {
        &self.ranges
    }

    /// The affected set the closure computed (Section 6's estimated
    /// affected ranges derive from this).
    #[must_use]
    pub const fn affected(&self) -> &BTreeSet<NodeId> {
        &self.affected
    }

    /// The acknowledgments the step carries, hash-bound with it when the
    /// plan body lands.
    #[must_use]
    pub fn acknowledgments(&self) -> &[Acknowledgment] {
        &self.acknowledgments
    }
}

/// A step-construction refusal — the typed artifact ADR-0012's axis
/// produces instead of a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepRefusal {
    /// The closure reached a node the step may not touch.
    Reached {
        /// The reached node.
        node: NodeId,
        /// Its verdict.
        verdict: Verdict,
    },
    /// An acknowledgment names a node whose verdict its kind does not
    /// lawfully cover — including every refused node, for which no kind
    /// exists.
    UnlawfulAcknowledgment {
        /// The named node.
        node: NodeId,
        /// Its actual verdict.
        verdict: Verdict,
    },
}

impl fmt::Display for StepRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reached { node, verdict } => {
                write!(formatter, "step reaches {node}, which is {verdict:?}")
            }
            Self::UnlawfulAcknowledgment { node, verdict } => write!(
                formatter,
                "acknowledgment does not lawfully cover {node} ({verdict:?})"
            ),
        }
    }
}

impl std::error::Error for StepRefusal {}
