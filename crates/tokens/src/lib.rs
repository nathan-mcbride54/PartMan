//! Design tokens and the WP-030 accessibility harness.
//!
//! `schemas/design-tokens.json` is the single source of truth for PartMan's
//! visual language. This crate reads it and *computes* the properties the
//! specification requires of it, rather than recording that someone once
//! checked:
//!
//! - **UI-001** — a dark charcoal default, a system theme, and an accessible
//!   high-contrast theme, all three defining the same set of roles.
//! - **UI-007** — colour is never the only carrier of identity, selection, file
//!   system, health, or risk. Every role that means something declares an icon,
//!   a label and a shape, and no two roles share an icon *and* a label.
//! - **UI-008** — every declared foreground/background pairing meets its WCAG
//!   2.2 AA threshold, in every theme.
//!
//! The token file lives in `schemas/` for the same reason
//! `canonical-encoding-vectors.json` does: when the Tauri front end arrives it
//! must read *this* file. An implementation checked against a table it also
//! owns proves only self-consistency, and `AGENTS.md` already records that
//! lesson for the canonical codec.
//!
//! # What this crate does not establish
//!
//! It renders nothing and opens no window. UI-008 also requires keyboard-only
//! operation, screen-reader semantics, 200% zoom and reduced motion; none of
//! those can be satisfied by a token file and none of them are inspected here.
//! [`audit::Report::caveats`] carries that list into the harness output so a
//! green run is never read as more than it is.
//!
//! This crate performs no privileged operation, opens no device, and launches
//! no process. It reads one JSON file and computes numbers from it.

pub mod audit;
pub mod color;
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
