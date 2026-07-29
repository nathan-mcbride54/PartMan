//! The accessibility policy, held **outside** the file it judges.
//!
//! This module exists because of a defect found by the 2026-07-29 project
//! audit. Every floor and every required role used to be read from
//! `schemas/design-tokens.json` — the same file the harness audits. Lowering
//! the file's own `text` threshold from 4.5 to 3.0 and dimming a colour to
//! match let normal-size text pass the entire Tier-1 gate at **3.33:1**, and
//! deleting a semantic role from every table at once dropped the audit from 234
//! checks to 228 while still reporting success.
//!
//! Both are the failure `AGENTS.md` already names for the canonical vectors:
//! an implementation checked against a table it also owns proves only
//! self-consistency. The harness was written to enforce that rule on colours
//! and broke it on the policy governing them.
//!
//! So the numbers below are **not** configuration. Two of them are external
//! standards that this project does not get to choose, and the third is a
//! project decision recorded as one. `schemas/design-tokens.json` may restate
//! them for a front end to read, but [`crate::audit`] requires the restatement
//! to *agree* rather than treating it as authority.

/// WCAG 2.2 Success Criterion 1.4.3 Contrast (Minimum), level AA, normal text.
///
/// <https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum>
///
/// Not negotiable and not a project setting: it is the published threshold. A
/// palette that cannot meet it changes its colours.
pub const WCAG_AA_NORMAL_TEXT: f64 = 4.5;

/// WCAG 2.2 Success Criterion 1.4.11 Non-text Contrast, level AA.
///
/// <https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast>
///
/// Applies to user-interface components and meaningful graphical objects. A
/// role carrying this threshold MUST NOT be used to render normal-size text,
/// and that restriction is the entire reason its floor is lower.
pub const WCAG_AA_NON_TEXT: f64 = 3.0;

/// CIE76 floor for role pairs whose confusion would mislead about risk.
///
/// **A project decision, not a standard.** WCAG sets no such threshold. It was
/// chosen to sit comfortably above the delta-E 10.1 collapse WP-030 increment 1
/// found between `severity.reversible` and `severity.destructive` under
/// deuteranopia, and comfortably below the 21.9 the corrected palette achieves.
///
/// Changing it is a policy change and needs an ADR with evidence, not a palette
/// edit. It lives here rather than in the token file so that lowering it is a
/// reviewed code change rather than a data edit.
pub const COLOR_SEPARATION_FLOOR: f64 = 12.0;

/// The specification version this token vocabulary was derived from.
pub const REQUIRED_SPEC_VERSION: &str = "4.0.0";

/// Threshold names a pairing may declare, and the floor each one means.
///
/// A pairing naming anything else is refused rather than skipped: a skipped
/// pairing is a promise that silently stopped being checked.
#[must_use]
pub fn threshold_for(kind: &str) -> Option<f64> {
    match kind {
        "text" => Some(WCAG_AA_NORMAL_TEXT),
        "ui" => Some(WCAG_AA_NON_TEXT),
        _ => None,
    }
}

/// Themes the product must define, per UI-001.
///
/// UI-001 requires a dark charcoal default, system theme support, and an
/// accessible high-contrast theme. All three must exist and must define the
/// same roles, so a component cannot render in the default theme and fall back
/// to nothing in the theme whose entire purpose is legibility.
pub const REQUIRED_THEMES: [&str; 3] = ["dark", "high-contrast", "light"];

/// The theme whose roster the others are compared against, and the UI-001
/// default.
pub const DEFAULT_THEME: &str = "dark";

/// Storage entity roles, from UI-003.
///
/// UI-003 requires physical devices, partitions, containers, volumes,
/// encryption, file systems, mounts, and free space to be distinguished
/// visually *and* textually. Each needs a role; deleting one means the product
/// cannot represent that concept, which is a specification violation and not a
/// smaller product.
pub const REQUIRED_ENTITY_ROLES: [&str; 8] = [
    "entity.device",
    "entity.partition",
    "entity.container",
    "entity.volume",
    "entity.encryption",
    "entity.filesystem",
    "entity.mount",
    "entity.freeSpace",
];

/// Severity roles, from PLAN-004's ordinal scale.
///
/// Exactly the five classes the specification defines: 0 Informational,
/// 1 Reversible, 2 Disruptive, 3 Data-moving, 4 Destructive. Not four, and not
/// six.
pub const REQUIRED_SEVERITY_ROLES: [&str; 5] = [
    "severity.informational",
    "severity.reversible",
    "severity.disruptive",
    "severity.dataMoving",
    "severity.destructive",
];

/// Progress roles, from UI-011.
///
/// UI-011 requires progress UI to distinguish planning, waiting for
/// authorization, executing, verifying, reboot pending, recovering, failed, and
/// complete.
pub const REQUIRED_PROGRESS_ROLES: [&str; 8] = [
    "progress.planning",
    "progress.awaitingAuthorization",
    "progress.executing",
    "progress.verifying",
    "progress.rebootPending",
    "progress.recovering",
    "progress.failed",
    "progress.complete",
];

/// Role pairs whose confusion would mislead a user about risk or outcome.
///
/// Held here rather than in the token file for the same reason as the floors:
/// the audit demonstrated that a pair could be deleted from the file and the
/// harness would simply check one fewer thing and still report success. The
/// most important entry is the first — PLAN-004 severity 1, "fully undoable via
/// an emitted reversal plan", against severity 4, "data is intentionally
/// destroyed".
pub const REQUIRED_DISTINCT_PAIRS: [(&str, &str); 7] = [
    ("severity.reversible", "severity.destructive"),
    ("severity.reversible", "severity.dataMoving"),
    ("severity.disruptive", "severity.destructive"),
    ("severity.informational", "severity.destructive"),
    ("progress.complete", "progress.failed"),
    ("progress.executing", "progress.failed"),
    ("progress.complete", "progress.recovering"),
];

/// Every role the product's vocabulary requires, in one iterator.
///
/// No `#[must_use]`: `impl Iterator` already carries it.
pub fn required_meaning_bearing_roles() -> impl Iterator<Item = &'static str> {
    REQUIRED_ENTITY_ROLES
        .into_iter()
        .chain(REQUIRED_SEVERITY_ROLES)
        .chain(REQUIRED_PROGRESS_ROLES)
}

/// Whether a role name carries meaning UI-007 protects.
///
/// Surfaces, text and borders do not: requiring an icon for `surface.base`
/// would be noise that trains a reader to ignore the rule.
#[must_use]
pub fn carries_meaning(role: &str) -> bool {
    role.starts_with("entity.") || role.starts_with("severity.") || role.starts_with("progress.")
}

#[cfg(test)]
mod tests;
