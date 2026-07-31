//! The generated-fixture manifest.
//!
//! A line-oriented text format, deliberately not JSON: the manifest is read by
//! the SAFE-007 interlock before any destructive suite runs, so its parser is on
//! the safety path and is kept small enough to audit in one sitting.
//!
//! ```text
//! # partman-fixtures manifest v1
//! token <64 hex characters>
//! image <64 hex characters> <byte length> <name>
//! ```

use core::fmt;
use std::collections::BTreeMap;

use sha2::{Digest as _, Sha256};

/// Header line every manifest starts with.
pub const MANIFEST_HEADER: &str = "# partman-fixtures manifest v1";

/// File name the manifest is written under, inside the fixture root.
pub const MANIFEST_FILE: &str = "MANIFEST";

/// One generated image, as recorded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// SHA-256 of the image bytes, lowercase hex.
    pub digest: String,
    /// Length of the image in bytes.
    pub length: u64,
}

/// The parsed manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    token: String,
    entries: BTreeMap<String, Entry>,
}

impl Manifest {
    /// Build a manifest from generated images.
    ///
    /// The token is derived from the entries, so it is stable for a given
    /// fixture set and changes the moment that set does.
    ///
    /// **It is not a secret, and must not be described as one.** Since
    /// expectations moved to the compiled catalogue, the token is a pure
    /// function of source: it is identical on every machine building this
    /// commit, needs no I/O to compute, and `cargo xtask fixtures` prints it to
    /// stdout, where CI captures it into a log. Anyone who can read the
    /// repository can derive it.
    ///
    /// What it proves is narrower: the invocation supplied the exact value
    /// derived by this build. It does not prove who supplied it, whether they
    /// ran the generator, or whether they intended an operation. Its value is
    /// accident friction, and SAFE-007's strength here rests on target
    /// verification — which is computed from bytes and cannot be asserted —
    /// not on this. A factor with independent strength would require state that
    /// is not derivable from source or writable fixture-root contents.
    ///
    /// # Panics
    ///
    /// Panics if an image length does not fit in `u64`, which no target this
    /// workspace supports can produce.
    #[must_use]
    pub fn build(images: &[(String, Vec<u8>)]) -> Self {
        let mut entries = BTreeMap::new();
        for (name, bytes) in images {
            entries.insert(
                name.clone(),
                Entry {
                    digest: hex(&Sha256::digest(bytes)),
                    length: u64::try_from(bytes.len()).expect("image length fits u64"),
                },
            );
        }

        let token = derive_token(&entries);
        Self { token, entries }
    }

    /// The disposable-test token.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Look up an entry by fixture name.
    #[must_use]
    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.get(name)
    }

    /// Every recorded name, in sorted order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// Whether any recorded image has this digest.
    #[must_use]
    pub fn contains_digest(&self, digest: &str) -> bool {
        self.entries.values().any(|entry| entry.digest == digest)
    }

    /// Render the manifest.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from(MANIFEST_HEADER);
        out.push('\n');
        out.push_str("token ");
        out.push_str(&self.token);
        out.push('\n');
        for (name, entry) in &self.entries {
            out.push_str("image ");
            out.push_str(&entry.digest);
            out.push(' ');
            out.push_str(&entry.length.to_string());
            out.push(' ');
            out.push_str(name);
            out.push('\n');
        }
        out
    }

    /// Parse a manifest, rejecting anything that is not exactly the format above.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError`] for a missing header, a malformed line, a
    /// duplicate name, a digest that is not 64 lowercase hex characters, or a
    /// missing token. The interlock treats any error as a refusal, so a
    /// permissive parser here would be a hole in SAFE-007.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let mut lines = text.lines();
        if lines.next() != Some(MANIFEST_HEADER) {
            return Err(ManifestError::Header);
        }

        let mut token = None;
        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();

        for line in lines {
            if line.is_empty() {
                continue;
            }
            let mut fields = line.split(' ');
            match fields.next() {
                Some("token") => {
                    let value = fields.next().ok_or(ManifestError::Malformed)?;
                    if fields.next().is_some() || !is_sha256_hex(value) {
                        return Err(ManifestError::Malformed);
                    }
                    if token.replace(value.to_owned()).is_some() {
                        return Err(ManifestError::DuplicateToken);
                    }
                }
                Some("image") => {
                    let digest = fields.next().ok_or(ManifestError::Malformed)?;
                    let length = fields.next().ok_or(ManifestError::Malformed)?;
                    let name = fields.next().ok_or(ManifestError::Malformed)?;
                    if fields.next().is_some() || !is_sha256_hex(digest) || name.is_empty() {
                        return Err(ManifestError::Malformed);
                    }
                    let length = length
                        .parse::<u64>()
                        .map_err(|_| ManifestError::Malformed)?;
                    let entry = Entry {
                        digest: digest.to_owned(),
                        length,
                    };
                    if entries.insert(name.to_owned(), entry).is_some() {
                        return Err(ManifestError::DuplicateName(name.to_owned()));
                    }
                }
                _ => return Err(ManifestError::Malformed),
            }
        }

        let token = token.ok_or(ManifestError::MissingToken)?;

        // The token is a *function* of the entries, so a parsed manifest whose
        // token does not follow from its own entries is a forgery, not a
        // manifest. Accepting it was the defect that let a hand-written file
        // authorize an arbitrary target: the token proved only that someone had
        // written a token.
        if !constant_time_eq(token.as_bytes(), derive_token(&entries).as_bytes()) {
            return Err(ManifestError::TokenDoesNotFollowFromEntries);
        }

        Ok(Self { token, entries })
    }
}

/// Derive a manifest's token from its entries, in sorted order.
///
/// Sorted so the token is a function of the fixture set rather than of the order
/// the entries happened to be built or parsed in.
fn derive_token(entries: &BTreeMap<String, Entry>) -> String {
    let mut hasher = Sha256::new();
    for (name, entry) in entries {
        hasher.update(name.as_bytes());
        hasher.update(b" ");
        hasher.update(entry.digest.as_bytes());
        hasher.update(b"\n");
    }
    hex(&hasher.finalize())
}

/// Compare without an early exit.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

/// Why a manifest could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    /// The first line was not the expected header.
    Header,
    /// A line did not match the grammar.
    Malformed,
    /// More than one token line appeared.
    DuplicateToken,
    /// No token line appeared.
    MissingToken,
    /// The same fixture name appeared twice.
    DuplicateName(String),
    /// The declared token is not the one these entries derive.
    TokenDoesNotFollowFromEntries,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header => write!(
                formatter,
                "manifest does not start with {MANIFEST_HEADER:?}"
            ),
            Self::Malformed => formatter.write_str("manifest line does not match the grammar"),
            Self::DuplicateToken => formatter.write_str("manifest declares more than one token"),
            Self::MissingToken => formatter.write_str("manifest declares no token"),
            Self::DuplicateName(name) => write!(formatter, "manifest repeats the name {name:?}"),
            Self::TokenDoesNotFollowFromEntries => {
                formatter.write_str("manifest token does not follow from its own entries")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// Lowercase hex encoding.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    out
}

const fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod tests;
