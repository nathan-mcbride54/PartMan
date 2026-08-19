//! The helper's own reach declaration (increment 2): ADR-0013's fourth
//! clause applied to the privileged contract — for each INV-003 state,
//! whether **this** contract can distinguish it, declared per contract
//! and platform, never derived per device, and never omitted when the
//! answer is no.
//!
//! The adapter's declaration says the client contract distinguishes no
//! table state, and that stays true at every privilege. What changed with
//! this increment is that a second contract exists: the byte layer —
//! head and tail windows through a read-only handle, classified by
//! `crates/table-parser`. Its separation of the six states is the SI-35
//! resolution's Tier-1 property, held against the catalogue in that
//! crate's own suite; **that the windows a root process reads are the
//! medium's bytes at the geometry sysfs states is the DR21 row**, which
//! every cell below cites. A cell here is a claim about the contract on
//! real Linux hosts, so the citation is the measured half; the parser's
//! half travels in each cell's basis note.

/// The declaration's schema identifier, versioned independently
/// (MODEL-003; provisional within major 0).
pub const REACH_SCHEMA: &str = "partman.helper-linux.reach/0";

/// The INV-003 state vocabulary, in INV-003's own order — the same six
/// the adapter's declaration names, deliberately.
pub const STATES: [&str; 6] = [
    "gpt",
    "mbr",
    "apple-partition-map",
    "missing-table",
    "hybrid-or-inconsistent",
    "corrupt-metadata",
];

/// What a cell's answer rests on. Closed at two.
pub mod basis {
    /// An observability row establishes this answer's host half.
    pub const MEASURED: &str = "measured";
    /// No row establishes it; the answer is `no` by default.
    pub const NOT_MEASURED: &str = "not-measured";
}

/// One INV-003 state's answer for the helper's byte-layer contract.
pub struct ReachCell {
    /// The state, from [`STATES`].
    pub state: &'static str,
    /// Whether this contract can distinguish it.
    pub distinguished: bool,
    /// [`basis::MEASURED`] or [`basis::NOT_MEASURED`].
    pub basis: &'static str,
    /// The `docs/quality/observability.md` heading the cell's host half
    /// rests on. `None` exactly when the basis is not-measured.
    pub citation: Option<&'static str>,
}

/// The contract's own statement.
pub struct ContractStatement {
    /// The contract's implementation state.
    pub state: &'static str,
    /// What would change it.
    pub reference: &'static str,
    /// One actionable sentence.
    pub detail: &'static str,
}

/// The published declaration: never partial — the array is fixed-size,
/// so a missing cell is a compile error rather than an omitted `no`.
pub struct ReachDeclaration {
    /// The contract statement.
    pub contract: ContractStatement,
    /// One cell per INV-003 state, in [`STATES`] order.
    pub cells: [ReachCell; STATES.len()],
}

/// The DR21 heading every measured cell cites — the host half: that the
/// bytes read through a whole-device node at head and tail are the
/// medium's, at the geometry sysfs states, identity-bracketed by device
/// number.
const DR21: &str = "The whole-device byte-window cell DR21";

const fn cell(state: &'static str) -> ReachCell {
    ReachCell {
        state,
        distinguished: true,
        basis: basis::MEASURED,
        citation: Some(DR21),
    }
}

/// The helper contract's published reach: every INV-003 state is
/// distinguished, because the byte layer reads and classifies the table
/// bytes themselves — GPT, MBR and APM as `Present` per scheme, a
/// positive `Absent` where every defined location was examined and none
/// claims a table, hybrid as the GPT state beside its `HybridMbr`
/// condition and view node, and corrupt metadata as `Indeterminate`
/// (unreadable or ambiguous) with per-copy conditions. The separation is
/// `crates/table-parser`'s Tier-1 property over the catalogue (the SI-35
/// resolution); the cited row is the host half.
pub const REACH: ReachDeclaration = ReachDeclaration {
    contract: ContractStatement {
        state: "implemented",
        reference: "a change to the byte layer's window shape, the parser's scheme coverage, \
                    or the DR21 row",
        detail: "the helper reads the head and tail windows of each uniquely-addressed whole \
                 device through a read-only handle bracketed by device number and classifies \
                 them with its own parser; the six states are separated by that parser over \
                 the fixture catalogue, and the cited row establishes that the windows are \
                 the medium's bytes at the stated geometry on real Linux block devices",
    },
    cells: [
        cell(STATES[0]),
        cell(STATES[1]),
        cell(STATES[2]),
        cell(STATES[3]),
        cell(STATES[4]),
        cell(STATES[5]),
    ],
};
