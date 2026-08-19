//! Increment 5a: the capability seam — CAP-004's `RuntimeFacts` for the
//! operations this adapter serves, produced in WP-050's own vocabulary,
//! from probes a launcher-owning caller supplies. Nothing here launches.
//!
//! **What this is, on the plan's finding F1**
//! (`docs/reviews/WP-L100_INCREMENT_5_PLAN_2026-08-19.md`). No read-only
//! operation needs a tool: every operation this adapter serves is a
//! source-class read of sysfs, the udev database and procfs files; INV-006
//! forbids repair tools during discovery; ACC-009 gates the *write* step;
//! and `docs/capabilities/format.md` §2 decides that a tool's floor arrives
//! "with the first package that invokes it". So the honest tool roster of
//! this adapter is **empty for every operation it serves**, stated per
//! operation and pinned by test, and this module's work is the **seam**: the
//! ACC-009 mapping from a probe result to the engine's [`ToolState`] — the
//! mapping WP-035's doctor deliberately left to "the capability engine, not
//! here" — and the assembly into [`RuntimeFacts`]. When WP-L110 invokes
//! storage tools, its roster and floors arrive through this seam; the launch
//! discipline itself (SAFE-004's structured argv, trusted absolute paths,
//! bounded output, a time limit, a sanitized environment, and the identity
//! clause the doctor carves out) stays with the package that launches, which
//! this one does not — the SAFE-002 structural guard over every shipped
//! module holds for this file too.
//!
//! **What this is not.** A mutating operation is not served by a read-only
//! adapter, and its tool needs are WP-L110's to state; asking here answers
//! a typed [`NotServed`], never an empty roster that a reader would take
//! for "no tool needed". The Section 9 floor determination is 5b's, on
//! rows DR16–DR18 and WP-050's `Undetermined` arm; until then a caller
//! supplies the [`PlatformFact`] it can stand behind, and this module
//! carries it unchanged.

use partman_capability::engine::{PlatformFact, RuntimeFacts, ToolFact, ToolState};
use partman_domain::model::capability::{Operation, OperationClass};

/// One tool an operation would need, named for remediation text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolRequirement {
    /// The tool's base name.
    pub tool: &'static str,
    /// Why the operation wants it, in one clause.
    pub role: &'static str,
}

/// The tools each served operation requires. **Empty for every source-class
/// operation, by finding F1** — a row here against a read-only operation
/// is exactly what the requirements test refuses.
pub const REQUIREMENTS: &[(Operation, &[ToolRequirement])] = &[
    (Operation::Detect, &[]),
    (Operation::Read, &[]),
    (Operation::Check, &[]),
    (Operation::Copy, &[]),
];

/// Tools INV-006 forbids during discovery — mount, unlock and repair —
/// which no source-class requirement may name. The list is the test's
/// vocabulary for "every mount, unlock, and repair-tool call", held against
/// [`REQUIREMENTS`] rather than asserted in prose.
pub const FORBIDDEN_DURING_DISCOVERY: &[&str] = &[
    "mount",
    "umount",
    "cryptsetup",
    "fsck",
    "e2fsck",
    "fsck.ext4",
    "xfs_repair",
    "btrfs",
    "ntfsfix",
    "fsck.fat",
    "mdadm",
    "vgchange",
    "lvchange",
];

/// Why runtime facts cannot be produced for an operation here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotServed {
    /// A mutating operation: its tool needs are the write helper's
    /// (WP-L110) to state, and an empty roster from a read-only adapter
    /// would read as "no tool needed".
    Mutating {
        /// The operation asked about.
        operation: Operation,
    },
}

/// The tools an operation requires, or why this adapter cannot say.
///
/// # Errors
///
/// [`NotServed`] for every mutating operation.
pub fn required_tools(operation: Operation) -> Result<&'static [ToolRequirement], NotServed> {
    if operation.class() == OperationClass::Mutating {
        return Err(NotServed::Mutating { operation });
    }
    Ok(REQUIREMENTS
        .iter()
        .find(|(served, _)| *served == operation)
        .map_or(&[][..], |(_, tools)| tools))
}

/// A tool version as the probing caller parsed it. Ordered by
/// `(major, minor, patch)`, which is what a floor comparison needs and all
/// it needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major.
    pub major: u32,
    /// Minor.
    pub minor: u32,
    /// Patch; `0` where the banner carried none.
    pub patch: u32,
}

/// One tool's probe result, as the launcher-owning caller established it
/// (WP-035's doctor today: existence at a compiled absolute path, a
/// bounded `--version`, a parsed banner). Structured, so this module
/// parses no banner and reads no path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolProbe {
    /// A regular file at a trusted absolute path answered the probe.
    Present {
        /// The absolute path that answered.
        path: String,
        /// The parsed version, or `None` where the banner did not parse.
        version: Option<Version>,
    },
    /// No candidate path held a regular file.
    Absent {
        /// The compiled candidates checked, in probe order.
        checked: Vec<String>,
    },
    /// A candidate existed and the probe failed — non-zero exit, time
    /// limit, output limit, or a launch failure.
    Failed {
        /// One sentence of detail.
        reason: String,
    },
}

/// A tool's version floor, as read by the caller from
/// `docs/capabilities/tool-version-floors.json` — never authored here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolFloor {
    /// The tool the floor is for.
    pub tool: &'static str,
    /// The floor.
    pub floor: Version,
}

/// ACC-009's mapping, fail-closed on every arm the text leaves open.
///
/// Present at or above a known floor is [`ToolState::PresentInRange`].
/// Absent is [`ToolState::Missing`]. Everything else is
/// [`ToolState::OutOfRange`]: present below the floor, present with **no
/// floor known** (no tested range exists, so no version is inside it — the
/// floors store is empty until a package invokes the tool and sets one),
/// present with an unparsed version, or a failed probe. `None` for the
/// probe — the caller did not look — is [`ToolState::Missing`]: the tool
/// is not established present, which is the same fail-closed answer as
/// absent, and the caller's report is what says where it did not look.
#[must_use]
pub fn tool_state(probe: Option<&ToolProbe>, floor: Option<&ToolFloor>) -> ToolState {
    match probe {
        None | Some(ToolProbe::Absent { .. }) => ToolState::Missing,
        Some(ToolProbe::Failed { .. }) => ToolState::OutOfRange,
        Some(ToolProbe::Present { version, .. }) => match (version, floor) {
            (Some(version), Some(floor)) if *version >= floor.floor => ToolState::PresentInRange,
            _ => ToolState::OutOfRange,
        },
    }
}

/// Assemble CAP-004's runtime facts for one served operation from the
/// caller's probes and floors, carrying the caller's platform determination
/// unchanged (5b's to produce).
///
/// # Errors
///
/// [`NotServed`] for a mutating operation.
pub fn runtime_facts(
    operation: Operation,
    probes: &[(&str, ToolProbe)],
    floors: &[ToolFloor],
    platform: PlatformFact,
) -> Result<RuntimeFacts, NotServed> {
    let tools = required_tools(operation)?
        .iter()
        .map(|requirement| {
            let probe = probes
                .iter()
                .find(|(name, _)| *name == requirement.tool)
                .map(|(_, probe)| probe);
            let floor = floors.iter().find(|floor| floor.tool == requirement.tool);
            ToolFact {
                tool: requirement.tool.to_owned(),
                state: tool_state(probe, floor),
            }
        })
        .collect();
    Ok(RuntimeFacts { tools, platform })
}
