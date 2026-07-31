//! Design tokens and the WP-030 accessibility harness.
//!
//! `schemas/design-tokens.json` is the single source of truth for PartMan's
//! visual language. This crate reads it and *computes* the properties the
//! specification requires of it, rather than recording that someone once
//! checked:
//!
//! - **UI-001** — a dark charcoal default, a canonical system-theme mapping,
//!   and a separate accessible high-contrast theme, all three defining the same
//!   set of roles.
//! - **UI-003** — the exact eight-role storage-entity vocabulary exists.
//! - **PLAN-004** — the exact five-role ordinal risk vocabulary exists.
//! - **UI-007** — each of the 21 declared entity, risk-severity, and
//!   progress-state roles has redundant icon, label-ID, and shape channels, and
//!   no two roles share an icon *and* a label ID. Selection and health remain
//!   separate shell-evidence obligations.
//! - **UI-008** — every independently pinned foreground/background use meets
//!   its exact text-or-UI WCAG 2.2 AA threshold in every theme; independent
//!   policy also pins measurement semantics and the token vocabularies later
//!   generated into typography, layout, and cursor bindings.
//! - **UI-011** — the exact eight-role progress-state vocabulary exists.
//! - **UI-013** — theme and semantic display text is represented by stable
//!   label IDs instead of English strings embedded in the token contract.
//!
//! The token file lives in `schemas/` for the same reason
//! `canonical-encoding-vectors.json` does: every front end must consume *this*
//! file. An implementation checked against a table it also owns proves only
//! self-consistency, and `AGENTS.md` already records that lesson for the
//! canonical codec.
//!
//! # What this crate does not establish
//!
//! It renders nothing and opens no window. A declared system-theme mapping does
//! not prove that an operating-system signal was detected. Typography, layout,
//! target-size, caret, and cursor tokens do not prove their rendered behavior.
//! UI-008 also requires keyboard-only operation, screen-reader semantics, 200%
//! zoom and reduced motion; none of those can be satisfied by a token file and
//! none of them are inspected here. Token evidence alone does not prove the
//! separately tested desktop catalogue resolves every label ID. Likewise, the
//! UI-003 and UI-011 vocabularies do not prove that entities are rendered
//! distinctly or that a live progress surface makes the required state
//! transitions, and UI-013's exact-byte inspector contract remains application
//! evidence.
//! [`audit::Report::caveats`] carries that list into the harness output so a
//! green run is never read as more than it is.
//!
//! The library and audit path perform no privileged operation, open no device,
//! launch no process, and write no file: they read one JSON source and compute
//! evidence from it. The developer-only `generate_slint_tokens` binary keeps
//! the same device, privilege, and subprocess boundaries. Its `--check` mode is
//! read-only; its explicit `--write` mode writes only the deterministic Slint
//! contract under `packages/design-tokens/generated/`.

pub mod audit;
pub mod color;
pub mod policy;
pub mod slint;
pub mod tokens;

pub use audit::{Report, audit};
pub use tokens::TokenSet;

/// Audit the token file this repository ships.
///
/// The entry point `cargo xtask tokens` calls.
///
/// # Errors
///
/// Propagates any [`tokens::TokenError`] from reading the file. A token set
/// that loads but violates a requirement is **not** an error here: it returns a
/// [`Report`] whose findings are non-empty, so the caller decides how to
/// present the difference between "unreadable" and "readable and wrong".
pub fn audit_repository_tokens() -> Result<Report, tokens::TokenError> {
    Ok(audit(&TokenSet::load_repository_tokens()?))
}
