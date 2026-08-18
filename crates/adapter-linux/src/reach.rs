//! The INV-003 reach declaration: for each partition-table state INV-003
//! names, whether **this package's own contract** on Linux can distinguish it.
//!
//! INV-003 as ADR-0013 scoped it requires the unprivileged discovery layer to
//! publish the reach of its platform contract — one answer per state, per
//! platform, derived from the contract rather than from any device, never
//! omitted when the answer is `no`. This module is that publication for the
//! contract WP-L100 delivers.
//!
//! Four properties, each of which a reviewer should be able to check rather
//! than take on trust:
//!
//! - **It is a property of the contract and the platform, not of a device.**
//!   Nothing here is derived per-device, because nothing here consults any
//!   device at all. This module may not name the crate's own reading surface,
//!   and a Tier-1 source-text test enforces that.
//! - **Every answer is `no`, on the not-measured basis, and increment 2
//!   re-decided that basis rather than inheriting it.** The field roster is
//!   now fixed and contains no partition-table key at all, so the contract
//!   statement moves to the reaches-no-table-state spelling while every cell
//!   stays negative. The basis stays `not-measured` for a reason that is
//!   itself measured-adjacent rather than lazy: a citation's vocabulary is
//!   `docs/quality/observability.md` headings, and **no Linux heading exists
//!   for `mbr` or `apple-partition-map` at all** — those answers live only in
//!   the fixture prober — so "measured" is unexecutable for at least two of
//!   the six, and a declaration split between two bases would say more about
//!   this repository's records than about the contract.
//!
//!   **A correction, recorded rather than edited away.** Increment 1 wrote
//!   that "the `udev` database does carry `ID_PART_TABLE_TYPE`". That was
//!   wrong twice over: the token appears in the record under the
//!   direct-signature-probe column, which is measured **denied** to the
//!   unprivileged client, and those probes were run over **regular files,
//!   not devices**. The two rows that do enumerate what the client-readable
//!   database carries — the WSL2 identifier row and the real-hardware
//!   client-signature row — name no table-type key. The conclusion that
//!   sentence supported is unchanged and in fact better supported: if the
//!   readable database carries no table-type key, this contract reaches no
//!   table state more cleanly, not less.
//! - **A `no` is never omitted**, and a cell no observability row establishes
//!   is `no` by the not-measured default — never `yes` by inference from a
//!   nearby platform, from an interface's documented enum, or from a
//!   privileged leg.
//! - **Naming these states is not classifying a device.** Saying "this
//!   contract cannot distinguish a hybrid table" reports the contract's reach,
//!   which INV-003 requires. No answer here ever says what state a device is
//!   in — the state itself is authored by the privileged helper under
//!   ADR-0014, never by this client.

/// The reach payload's own schema identifier, versioned independently.
/// Provisional within major version 0, per MODEL-003.
pub const REACH_SCHEMA: &str = "partman.adapter-linux.reach/0";

/// The INV-003 state vocabulary, in INV-003's own order.
pub const STATES: [&str; 6] = [
    "gpt",
    "mbr",
    "apple-partition-map",
    "missing-table",
    "hybrid-or-inconsistent",
    "corrupt-metadata",
];

/// What a cell's answer rests on. Closed at two, so a reader can tell
/// "measured and negative" from "never measured".
pub mod basis {
    /// An observability row establishes this answer.
    pub const MEASURED: &str = "measured";
    /// No row establishes it, so the answer is `no` by default.
    pub const NOT_MEASURED: &str = "not-measured";
}

/// One INV-003 state's answer for this contract.
pub struct ReachCell {
    /// The state, from [`STATES`].
    pub state: &'static str,
    /// Whether this contract can distinguish it.
    pub distinguished: bool,
    /// [`basis::MEASURED`] or [`basis::NOT_MEASURED`].
    pub basis: &'static str,
    /// The `docs/quality/observability.md` heading this cell rests on.
    /// `None` exactly when the basis is not-measured.
    pub citation: Option<&'static str>,
}

/// What this package's contract does on this platform, in one closed word
/// plus a sentence a reader can act on.
pub struct ContractStatement {
    /// The contract's implementation state.
    pub state: &'static str,
    /// What changes it.
    pub reference: &'static str,
    /// One actionable sentence.
    pub detail: &'static str,
}

/// The published declaration: a contract statement and one cell per INV-003
/// state.
pub struct ReachDeclaration {
    /// The contract's implementation state on this platform.
    pub contract: ContractStatement,
    /// One cell per INV-003 state, in [`STATES`] order. Never partial — the
    /// array is fixed-size, so a missing cell is a compile error rather than
    /// an omitted `no`.
    pub cells: [ReachCell; STATES.len()],
}

/// A `const fn` rather than a literal table, because six hand-copied cells is
/// six chances to type `true`. There is exactly one place a `yes` could be
/// written, and it is not reachable from here.
const fn nothing_distinguished(contract: ContractStatement) -> ReachDeclaration {
    const fn cell(state: &'static str) -> ReachCell {
        ReachCell {
            state,
            distinguished: false,
            basis: basis::NOT_MEASURED,
            citation: None,
        }
    }
    ReachDeclaration {
        contract,
        cells: [
            cell(STATES[0]),
            cell(STATES[1]),
            cell(STATES[2]),
            cell(STATES[3]),
            cell(STATES[4]),
            cell(STATES[5]),
        ],
    }
}

/// This package's published reach.
///
/// One declaration, not a per-platform table: this package's contract is the
/// Linux one and has no answer to give about any other platform. A sibling
/// adapter publishes its own.
pub const REACH: ReachDeclaration = nothing_distinguished(ContractStatement {
    state: "implemented-reaches-no-table-state",
    reference: "a partition-table key entering this contract's field roster",
    detail: "this contract reads sysfs block-class attributes, udev database records, and \
             the kernel's procfs mount and swap tables, and its roster carries no \
             partition-table key at all — so no state above is distinguishable through it, \
             and the state itself is authored by the privileged helper rather than \
             determined here",
});

/// Render the declaration as JSON.
#[must_use]
pub fn reach_json() -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"");
    out.push_str(REACH_SCHEMA);
    out.push_str("\",\"contract\":{\"state\":\"");
    out.push_str(REACH.contract.state);
    out.push_str("\",\"reference\":\"");
    out.push_str(REACH.contract.reference);
    out.push_str("\",\"detail\":\"");
    out.push_str(REACH.contract.detail);
    out.push_str("\"},\"states\":[");
    for (index, cell) in REACH.cells.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"state\":\"");
        out.push_str(cell.state);
        out.push_str("\",\"distinguished\":");
        out.push_str(if cell.distinguished { "true" } else { "false" });
        out.push_str(",\"basis\":\"");
        out.push_str(cell.basis);
        out.push_str("\",\"citation\":");
        match cell.citation {
            Some(citation) => {
                out.push('"');
                out.push_str(citation);
                out.push('"');
            }
            None => out.push_str("null"),
        }
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// Append the declaration to a human answer, in a two-space block.
pub fn reach_human(out: &mut String) {
    out.push_str("  reach (INV-003, ");
    out.push_str(REACH_SCHEMA);
    out.push_str("):\n    contract: ");
    out.push_str(REACH.contract.state);
    out.push_str(" (");
    out.push_str(REACH.contract.reference);
    out.push_str(")\n");
    for cell in &REACH.cells {
        out.push_str("    ");
        out.push_str(cell.state);
        out.push_str(": ");
        out.push_str(if cell.distinguished { "yes" } else { "no" });
        out.push_str(" (");
        out.push_str(cell.basis);
        out.push_str(")\n");
    }
}
