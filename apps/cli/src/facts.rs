//! Technology-limit facts: FS-007's inputs, and nothing more.
//!
//! FS-007 requires immutable technical limits — its own example is XFS not
//! shrinking — surfaced as explicit blocked reasons. The *blocked reason* is
//! CAP-003 status vocabulary, owned by WP-050's capability engine and
//! forbidden here; what this module ships is the input side: properties of
//! a **technology**, never a verdict about a **target**. Every fact names
//! the technology, the operation it limits, the limit itself, and the basis
//! a reader can check. A test refuses the CAP-003 status words in every
//! rendering, so the boundary is mechanical rather than reviewed.

use crate::json_escaped;

/// One immutable property of a storage technology.
pub struct TechnologyFact {
    /// The technology the fact is about — never a device or a volume.
    pub technology: &'static str,
    /// The operation the limit constrains.
    pub operation: &'static str,
    /// The limit, stated as the property it is.
    pub limit: &'static str,
    /// Where a reader verifies it.
    pub basis: &'static str,
}

/// The shipped facts. Each is a property of the technology everywhere it
/// exists, which is what makes it recordable before any device has ever
/// been observed; anything that depends on a particular machine belongs to
/// the capability engine, not here.
pub const FACTS: &[TechnologyFact] = &[
    TechnologyFact {
        technology: "xfs",
        operation: "shrink",
        limit: "XFS provides no shrink operation; the filesystem grows only",
        basis: "xfs_growfs(8), which offers growth and no reverse; FS-007's own example",
    },
    TechnologyFact {
        technology: "ext4",
        operation: "shrink while mounted",
        limit: "ext4 shrinks offline only; resize2fs refuses to shrink a mounted filesystem",
        basis: "resize2fs(8): online resizing grows only",
    },
    TechnologyFact {
        technology: "linux-swap",
        operation: "resize in place",
        limit: "swap space carries no filesystem to preserve; resizing is recreation",
        basis: "mkswap(8): the signature is written anew for the new size",
    },
    TechnologyFact {
        technology: "fat32",
        operation: "hold a file of 4 GiB or larger",
        limit: "a single file is at most one byte under 4 GiB",
        basis: "the on-disk format's 32-bit file-size field",
    },
    TechnologyFact {
        technology: "fat32",
        operation: "address a volume beyond 2 TiB with 512-byte sectors",
        limit: "32-bit sector counts cap a 512-byte-sector volume at 2 TiB",
        basis: "the on-disk format's 32-bit total-sector field",
    },
];

/// Render the facts as a JSON array.
#[must_use]
pub fn facts_json() -> String {
    let entries: Vec<String> = FACTS
        .iter()
        .map(|fact| {
            format!(
                "{{\"technology\":{technology},\"operation\":{operation},\"limit\":{limit},\
                 \"basis\":{basis}}}",
                technology = json_escaped(fact.technology),
                operation = json_escaped(fact.operation),
                limit = json_escaped(fact.limit),
                basis = json_escaped(fact.basis),
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Render the facts for humans.
#[must_use]
pub fn facts_human() -> String {
    use std::fmt::Write as _;
    let mut out = format!(
        "technology facts ({} entries; properties of technologies, never verdicts about \
         targets)\n",
        FACTS.len()
    );
    for fact in FACTS {
        let _ = writeln!(out, "  {} — {}", fact.technology, fact.operation);
        let _ = writeln!(out, "    limit: {}", fact.limit);
        let _ = writeln!(out, "    basis: {}", fact.basis);
    }
    out
}
