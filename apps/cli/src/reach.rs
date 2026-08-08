//! The INV-003 reach declaration: for each partition-table state INV-003
//! names, whether **this package's own contract** on this platform can
//! distinguish it.
//!
//! INV-003 as ADR-0013 scoped it (spec 6.0.0) requires the unprivileged
//! discovery layer to "publish the reach of its platform contract ... one
//! answer per state, per platform, derived from the contract rather than
//! from any device, never omitted when the answer is `no`". This module is
//! that publication for the contract WP-035 reads.
//!
//! Four properties, each of which a reviewer should be able to check rather
//! than take on trust:
//!
//! - **It is a property of the contract and the platform, not of a device.**
//!   Nothing here is derived per-device, because nothing here reads anything.
//!   `reach.rs` may not name the device seam at all, and a Tier-1 source-text
//!   test enforces that.
//! - **Every answer is `no`, on every platform.** WP-035's Section 14 row
//!   grants the declaration "for the contract this package itself reads",
//!   and closes: "This package's reach declaration describes its own
//!   contract and is not a claim about interfaces that contract does not
//!   read." The Linux and macOS contracts read identity attributes and no
//!   table-state surface — deliberately, since the scheme fields are the
//!   register's material — and Windows is deferred by a recorded decision,
//!   so every cell stays negative. A measured `yes` could arrive only with
//!   an increment that reads a table-state interface, landing with the code
//!   that reads it, never ahead of it.
//! - **A `no` is never omitted**, and a cell no observability row establishes
//!   is `no` by the not-measured default. It is never `yes` by inference from
//!   a nearby platform, from an API's documented enum, or from a privileged
//!   leg: `docs/work-packages/WP-035.md` records that privileged comparison
//!   legs "must not be used to claim that an unprivileged product path can
//!   observe the same fact".
//! - **Naming these states is not classifying a device.** Saying "this
//!   contract cannot distinguish a hybrid table" reports the contract's
//!   reach, which INV-003 requires. The `partition-table-state` entry in
//!   `inspect`'s gated list stands unchanged beside it, and no answer here
//!   ever says what state a device is in.

/// The reach payload's own schema identifier, versioned independently of the
/// envelope. Provisional within major version 0, per CLI-008 and MODEL-003.
pub const REACH_SCHEMA: &str = "partman.cli.reach/0";

/// The INV-003 state vocabulary, in INV-003's own order. Identical on every
/// platform: only the answers differ, which is what lets completeness and
/// order be checked once rather than per platform.
pub const STATES: [&str; 6] = [
    "gpt",
    "mbr",
    "apple-partition-map",
    "missing-table",
    "hybrid-or-inconsistent",
    "corrupt-metadata",
];

/// Why a cell answers the way it does. A closed two-word vocabulary, so a
/// reader can tell "measured and negative" from "never measured".
pub mod basis {
    /// An observability row establishes this answer.
    pub const MEASURED: &str = "measured";
    /// No row establishes it, so the answer is `no` by default.
    pub const NOT_MEASURED: &str = "not-measured";
}

/// One INV-003 state, and whether this platform's contract separates it.
pub struct ReachCell {
    /// The state, in INV-003's own words.
    pub state: &'static str,
    /// Whether this package's contract can distinguish it on this platform.
    pub distinguished: bool,
    /// [`basis::MEASURED`] or [`basis::NOT_MEASURED`].
    pub basis: &'static str,
    /// The `docs/quality/observability.md` heading this cell rests on.
    /// `None` exactly when the basis is not-measured — which is every cell
    /// in this increment, because an empty contract has nothing to cite.
    pub citation: Option<&'static str>,
}

/// What this package's contract on this platform is, and whether this build
/// reads it yet.
pub struct ContractStatement {
    /// The state word, from the chassis's typed-refusal vocabulary.
    pub state: &'static str,
    /// What changes it.
    pub reference: &'static str,
    /// One sentence a human can act on.
    pub detail: &'static str,
}

/// One platform's complete declaration.
pub struct ReachDeclaration {
    /// The contract's implementation state on this platform.
    pub contract: ContractStatement,
    /// One cell per INV-003 state, in [`STATES`] order. Never partial —
    /// `declaration_is_complete_and_ordered` proves it.
    pub cells: [ReachCell; STATES.len()],
}

/// Build the all-negative declaration this increment publishes.
///
/// A `const fn` rather than a literal table, because six hand-copied cells
/// per platform is six chances to type `true`. There is exactly one place a
/// `yes` could be written, and it is not reachable from here.
const fn unread_contract(reference: &'static str) -> ReachDeclaration {
    const fn cell(state: &'static str) -> ReachCell {
        ReachCell {
            state,
            distinguished: false,
            basis: basis::NOT_MEASURED,
            citation: None,
        }
    }
    ReachDeclaration {
        contract: ContractStatement {
            state: "not-implemented",
            reference,
            detail: DETAIL,
        },
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

/// The sentence beside every cell where this package reads nothing at all.
const DETAIL: &str = "this package reads no device interface yet, so its contract distinguishes no \
     partition-table state; every answer below is negative by the not-measured \
     default, and a positive answer arrives only with the increment that reads \
     the interface establishing it";

/// The sentence where a contract exists but reads no table-state interface.
///
/// The distinction matters and an earlier version of this module collapsed it.
/// "Reads nothing" and "reads identity attributes but no table-state surface"
/// produce the same all-negative cells for different reasons, and INV-003
/// requires the declaration be *derived from the contract* — so a contract
/// that exists must not be described as absent.
const DETAIL_READS_NO_TABLE_STATE: &str = "this package's contract reads whole-device identity attributes and no \
     partition-table interface, so it distinguishes no state below; the answers are \
     negative because nothing in this contract reaches them, not because nothing \
     has been measured";

/// Build the declaration for a platform whose contract exists but reaches no
/// partition-table state.
const fn read_contract_distinguishing_nothing(reference: &'static str) -> ReachDeclaration {
    const fn cell(state: &'static str) -> ReachCell {
        ReachCell {
            state,
            distinguished: false,
            basis: basis::NOT_MEASURED,
            citation: None,
        }
    }
    ReachDeclaration {
        contract: ContractStatement {
            state: "implemented-reaches-no-table-state",
            reference,
            detail: DETAIL_READS_NO_TABLE_STATE,
        },
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

/// The recorded decision that defers the Windows enumeration adapter.
///
/// WP-035's grant opened increment 10 only after a recorded choice among its
/// three named routes, and the recorded choice is deferral: no route is
/// simultaneously dependency-free, `unsafe`-free, and clean against the
/// tool-invocation rules, so the interim Windows adapter is not built and
/// the duty stays where Section 14 always placed it. The record is in
/// `docs/work-packages/WP-035.md`; this constant is how the Windows answer
/// names a decision rather than a promise, which is the shape the M0.5 gate
/// was written to accept.
pub const WINDOWS_DEFERRAL: &str =
    "deferred to WP-W100 by the WP-035 increment 10 route decision (2026-08-08)";

/// This build's declaration.
///
/// The reference names what changes the platform's answer: the increment
/// that will read its interfaces where one is pending, or the recorded
/// decision that defers it where none is. Neither form claims another
/// package's contract — the product's own INV-003 duty stays with WP-W100,
/// WP-L100 and WP-M100, and this declaration makes no claim about theirs.
pub const REACH: ReachDeclaration = if cfg!(target_os = "linux") {
    read_contract_distinguishing_nothing("WP-035 increment 8")
} else if cfg!(target_os = "macos") {
    read_contract_distinguishing_nothing("WP-035 increment 9")
} else if cfg!(target_os = "windows") {
    unread_contract(WINDOWS_DEFERRAL)
} else {
    unread_contract("no increment is scheduled for this platform")
};

/// Render the reach declaration as JSON.
#[must_use]
pub fn reach_json() -> String {
    let cells: Vec<String> = REACH
        .cells
        .iter()
        .map(|cell| {
            format!(
                "{{\"state\":{state},\"distinguished\":{distinguished},\"basis\":{basis},\
                 \"citation\":{citation}}}",
                state = crate::json_escaped(cell.state),
                distinguished = cell.distinguished,
                basis = crate::json_escaped(cell.basis),
                citation = match cell.citation {
                    Some(citation) => crate::json_escaped(citation),
                    None => "null".to_owned(),
                },
            )
        })
        .collect();
    format!(
        "{{\"schema\":{schema},\"contract\":{{\"state\":{state},\"reference\":{reference},\
         \"detail\":{detail}}},\"states\":[{cells}]}}",
        schema = crate::json_escaped(REACH_SCHEMA),
        state = crate::json_escaped(REACH.contract.state),
        reference = crate::json_escaped(REACH.contract.reference),
        detail = crate::json_escaped(REACH.contract.detail),
        cells = cells.join(","),
    )
}

/// Append the reach declaration to a human answer, in the same two-space
/// block style as the gated list it sits beside.
pub fn reach_human(out: &mut String) {
    out.push_str("  reach (INV-003, ");
    out.push_str(REACH_SCHEMA);
    out.push_str(")\n    contract: ");
    out.push_str(REACH.contract.state);
    out.push_str(" (");
    out.push_str(REACH.contract.reference);
    out.push_str(")\n    ");
    out.push_str(REACH.contract.detail);
    out.push('\n');
    for cell in &REACH.cells {
        let answer = if cell.distinguished { "yes" } else { "no" };
        out.push_str("    ");
        out.push_str(cell.state);
        out.push_str(": ");
        out.push_str(answer);
        out.push_str(" (");
        out.push_str(cell.basis);
        if let Some(citation) = cell.citation {
            out.push_str(", ");
            out.push_str(citation);
        }
        out.push_str(")\n");
    }
}
