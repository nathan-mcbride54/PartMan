//! The WP-070 protection-artifact store (ADR-0030's store class).
//!
//! REC-011 names the object this crate exists for: a backup of
//! protection-relevant metadata — encryption headers (REC-011), and
//! PART-013's parse-level table backups and raw region captures under
//! ADR-0024's arms — created and verified before the write it insures,
//! and dangerous in a specific, well-known way: it freezes state at
//! backup time, so what was revoked afterward remains usable with it.
//! ADR-0030 gave the class four rules; this crate is their
//! pure-library half:
//!
//! - **Home (Rule 1).** A dedicated helper-owned store, sibling to and
//!   never inside the journal, inheriting JRN-004's admin-protected
//!   documented-location clause. This crate owns the store's layout
//!   and discipline over an injected [`StoreSeam`]; each helper's
//!   per-OS on-disk path and seam implementation land under that
//!   helper's own grant, and `schemas/artifact-store.md` keeps the
//!   table.
//! - **Reference by identity (Rule 2).** An artifact is addressed by
//!   the SHA-256 of its exact bytes. Every surface outside the store
//!   carries [`ProtectionArtifactRef`] — content hash plus store
//!   identity, the journal crate's own reference vocabulary — and the
//!   bytes have no exit but [`Store::fetch`], which re-verifies them
//!   against the reference before returning anything.
//! - **Retention by liveness (Rule 3).** [`Store::retention_pass`]
//!   computes reclaimability from the journal's decoded records and
//!   from nothing else: an artifact is exempt while any apply whose
//!   protection record references it is exempt under ADR-0029's
//!   linkage closure, an unreferenced object fails closed as an
//!   orphan, and a corrupt object is never reclaimable.
//! - **Consequence-stated end of life (Rule 4).** Reclaiming a
//!   terminated artifact demands an explicit [`DeleteDecision`];
//!   silence retains. The two consequence sentences the deciding
//!   surface must render are pinned here ([`RETAIN_CONSEQUENCE`],
//!   [`DELETE_CONSEQUENCE`]) and in `schemas/artifact-store.md`, so a
//!   consumer cannot drift its own wording. The deciding surface
//!   itself — SEC-009-shaped, displayed, changeable — is the surface
//!   package's obligation, recorded in WP-070's assignment; this crate
//!   makes silent deletion unrepresentable and no more.
//!
//! What this crate deliberately is not:
//!
//! - **No on-disk path policy.** The per-OS store root — Linux's
//!   `/var/lib/partman`-sibling directory among them — is each
//!   helper's, under its own grant, exactly as the journal's on-disk
//!   home was WP-L110 increment 4a's. This crate never names a path.
//! - **No platform durability.** [`StoreSeam::put`] is required to be
//!   durable-on-return by contract; the truth of that contract on a
//!   real platform is the helper packages' acceptance obligation,
//!   exactly as JRN-002's seam records.
//! - **No metadata.** The store holds bytes by hash — no index, no
//!   sidecar, no kind tag. Which plan an artifact insures, which
//!   PART-013 arm produced it, and which regions a raw capture covers
//!   are the journal's protection records' facts; a second copy here
//!   could disagree with the first.
//! - **No ordering enforcement.** PART-013's discharge order — the
//!   artifact durable and verified before the protection record that
//!   references it is journaled — is the depositing helper's, stated
//!   in `schemas/artifact-store.md`; a pure library cannot see the
//!   journal append it must precede.
//! - **No user policy surface.** Rule 4's deciding surface belongs to
//!   the package that builds retention management; this crate pins the
//!   vocabulary and refuses the silent path.

use std::collections::BTreeMap;
use std::fmt;

use partman_journal::records::{
    ArtifactHashRef, ArtifactStore, PlanHashRef, ProtectionArm, ProtectionArtifactRef, Record,
};
use partman_journal::retention::{CompactedRefused, DecodedJournal, ledger};
use sha2::{Digest as _, Sha256};

#[cfg(test)]
mod tests;

/// An object's name in the store: the SHA-256 of its bytes, rendered
/// as 64 lowercase hexadecimal characters. The name *is* the
/// reference's content hash — one namespace, no index, no second
/// identity to fall out of agreement with the first (ADR-0030 Rule 2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectName([u8; 32]);

impl ObjectName {
    /// The name of the object a reference's content hash addresses.
    #[must_use]
    pub const fn from_content_hash(content: ArtifactHashRef) -> Self {
        ObjectName(*content.as_bytes())
    }

    /// The content hash this name spells.
    #[must_use]
    pub const fn content_hash(&self) -> ArtifactHashRef {
        ArtifactHashRef::from_bytes(self.0)
    }

    /// The canonical rendering: 64 lowercase hexadecimal characters.
    /// This is the store's one on-disk spelling; seam implementations
    /// use it verbatim as the object's file name.
    #[must_use]
    pub fn as_hex(&self) -> String {
        const DIGITS: [u8; 16] = *b"0123456789abcdef";
        let mut hex = String::with_capacity(64);
        for byte in self.0 {
            hex.push(char::from(DIGITS[usize::from(byte >> 4)]));
            hex.push(char::from(DIGITS[usize::from(byte & 0x0F)]));
        }
        hex
    }

    /// Parse a canonical rendering back to a name — the inverse of
    /// [`ObjectName::as_hex`], for seam implementations listing a real
    /// store. Exactly 64 characters, each `0-9a-f`; anything else is a
    /// refusal, never a repair, so a stray file in a store directory
    /// cannot masquerade as an object.
    ///
    /// # Errors
    ///
    /// [`NameInvalid`] naming the defect.
    pub fn from_hex(name: &str) -> Result<Self, NameInvalid> {
        if name.len() != 64 {
            return Err(NameInvalid::Length { length: name.len() });
        }
        let mut bytes = [0_u8; 32];
        for (index, chunk) in name.as_bytes().chunks_exact(2).enumerate() {
            let hi =
                hex_digit(chunk[0]).ok_or(NameInvalid::NotLowercaseHex { index: index * 2 })?;
            let lo = hex_digit(chunk[1]).ok_or(NameInvalid::NotLowercaseHex {
                index: index * 2 + 1,
            })?;
            bytes[index] = (hi << 4) | lo;
        }
        Ok(ObjectName(bytes))
    }
}

/// The value of one lowercase hexadecimal digit, or `None`. Uppercase
/// is deliberately refused: the store has one canonical spelling, and
/// accepting two spellings of one name would let a real directory hold
/// two objects with one identity.
const fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

impl fmt::Debug for ObjectName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The hex spelling is a hash — safe on every rendering surface,
        // and more legible in a refusal than a 32-entry byte array.
        write!(formatter, "ObjectName({})", self.as_hex())
    }
}

/// A rejected object-name spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameInvalid {
    /// The spelling is not exactly 64 characters.
    Length {
        /// The offered length.
        length: usize,
    },
    /// A character outside `0-9a-f` (uppercase included — one
    /// canonical spelling, deliberately).
    NotLowercaseHex {
        /// The first offending character's byte index.
        index: usize,
    },
}

/// The store's byte-level backend: the boundary between this crate's
/// discipline and a platform's directory. Implementations are each
/// helper's, under its own grant, against the per-OS root
/// `schemas/artifact-store.md` documents; tests inject fakes.
///
/// The contract every implementation owes:
///
/// - `put` is **durable on return** — an acknowledged object survives
///   a crash. The platform truth of that sentence is the helper's
///   acceptance obligation, exactly as JRN-002's seam records; this
///   crate verifies content by re-read but cannot see an fsync.
/// - `list` returns each held object exactly once.
/// - A refusal's `reason` names the seam's own condition and never
///   embeds object bytes.
pub trait StoreSeam {
    /// Durably store `bytes` under `name`, overwriting any existing
    /// object of that name.
    ///
    /// # Errors
    ///
    /// [`SeamRefused`] when the object cannot be durably stored.
    fn put(&mut self, name: &ObjectName, bytes: &[u8]) -> Result<(), SeamRefused>;

    /// The named object's bytes, or `None` if no such object is held.
    ///
    /// # Errors
    ///
    /// [`SeamRefused`] when presence cannot be determined or the held
    /// object cannot be read.
    fn read(&self, name: &ObjectName) -> Result<Option<Vec<u8>>, SeamRefused>;

    /// Every held object's name, each exactly once.
    ///
    /// # Errors
    ///
    /// [`SeamRefused`] when the store cannot be enumerated.
    fn list(&self) -> Result<Vec<ObjectName>, SeamRefused>;

    /// Remove the named object.
    ///
    /// # Errors
    ///
    /// [`SeamRefused`] when the object cannot be removed.
    fn remove(&mut self, name: &ObjectName) -> Result<(), SeamRefused>;
}

/// A seam's refusal. Carries the seam's own stated reason; the store
/// adds nothing to it and treats every refusal the same way — the
/// operation did not happen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeamRefused {
    /// The seam's stated reason.
    pub reason: String,
}

/// The protection-artifact store: ADR-0030's discipline over an
/// injected [`StoreSeam`]. One value serves one store — today the one
/// [`ArtifactStore::HelperProtectionStore`] names.
#[derive(Debug)]
pub struct Store<S: StoreSeam> {
    seam: S,
}

impl<S: StoreSeam> Store<S> {
    /// A store over the injected seam.
    #[must_use]
    pub fn new(seam: S) -> Self {
        Store { seam }
    }

    /// Deposit an artifact: hash it, store it, **verify it by re-read**
    /// — REC-011's own word — and only then hand back the hash-only
    /// reference every other surface carries. The reference exists
    /// only past the verification, so a caller holding one holds proof
    /// the store could reproduce the exact bytes at deposit time.
    ///
    /// Depositing bytes the store already holds is one artifact, not
    /// two: the store is content-addressed, and the verified re-read
    /// is performed either way.
    ///
    /// The discharge order — this verified return *before* the journal
    /// record that references the artifact is appended — is the
    /// depositing helper's obligation, stated in
    /// `schemas/artifact-store.md`.
    ///
    /// # Errors
    ///
    /// [`DepositRefused`]: empty bytes (a zero-length artifact
    /// witnesses a failed capture, never a backup), a seam refusal, or
    /// a failed verification — each fail-closed, none returning a
    /// reference.
    pub fn deposit(&mut self, bytes: &[u8]) -> Result<ProtectionArtifactRef, DepositRefused> {
        if bytes.is_empty() {
            return Err(DepositRefused::Empty);
        }
        let content = content_hash(bytes);
        let name = ObjectName::from_content_hash(content);
        self.seam.put(&name, bytes).map_err(DepositRefused::Seam)?;
        let held = self
            .seam
            .read(&name)
            .map_err(DepositRefused::Seam)?
            .ok_or(DepositRefused::VerifyMissing { name })?;
        let computed = content_hash(&held);
        if computed != content {
            return Err(DepositRefused::VerifyMismatch { name, computed });
        }
        Ok(ProtectionArtifactRef::new(
            content,
            ArtifactStore::HelperProtectionStore,
        ))
    }

    /// The referenced artifact's bytes, re-verified against the
    /// reference before anything is returned — the store never serves
    /// bytes whose hash is not the name they were asked for by.
    ///
    /// # Errors
    ///
    /// [`FetchRefused`]: the object is missing, its bytes no longer
    /// hash to the reference, or the seam refused.
    pub fn fetch(&self, reference: &ProtectionArtifactRef) -> Result<Vec<u8>, FetchRefused> {
        // One store exists today; a second [`ArtifactStore`] variant
        // makes this match — and the routing question it stands for —
        // a compile error rather than a silent misroute.
        match reference.store() {
            ArtifactStore::HelperProtectionStore => {}
        }
        let name = ObjectName::from_content_hash(reference.content());
        let held = self
            .seam
            .read(&name)
            .map_err(FetchRefused::Seam)?
            .ok_or(FetchRefused::Missing { name })?;
        let computed = content_hash(&held);
        if computed != reference.content() {
            return Err(FetchRefused::Corrupt { name, computed });
        }
        Ok(held)
    }

    /// Classify every held object against the journal's records —
    /// ADR-0030 Rule 3, computed from the decoded journal and from
    /// nothing else. An artifact is **exempt** while any apply whose
    /// protection record references it is exempt under ADR-0029's
    /// linkage closure; only an artifact whose every referencing apply
    /// has wholly terminated is reclaimable. An **unreferenced** object
    /// fails closed as an orphan — the journal cannot prove its
    /// closure terminated, so nothing reclaims it. A **corrupt** object
    /// is never reclaimable, whatever its liveness. A reference to an
    /// object the store does not hold is surfaced, because a journal
    /// record is promising bytes the store cannot produce.
    ///
    /// # Errors
    ///
    /// [`RetentionRefused`]: the journal refuses to ledger (its own
    /// typed grounds), or the seam refuses — either way the pass
    /// classifies nothing rather than guessing.
    pub fn retention_pass(
        &self,
        decoded: &DecodedJournal,
    ) -> Result<RetentionPass, RetentionRefused> {
        let ledger = ledger(decoded).map_err(RetentionRefused::Journal)?;
        let mut references: BTreeMap<ObjectName, Vec<PlanHashRef>> = BTreeMap::new();
        for (_, record) in decoded.records() {
            let Record::Protection(protection) = record else {
                continue;
            };
            let artifact = match protection.arm() {
                ProtectionArm::ParseBackupVerified { artifact }
                | ProtectionArm::RawCaptureVerified { artifact, .. } => artifact,
                ProtectionArm::AbsenceDetermined => continue,
            };
            match artifact.store() {
                ArtifactStore::HelperProtectionStore => {}
            }
            references
                .entry(ObjectName::from_content_hash(artifact.content()))
                .or_default()
                .push(protection.plan());
        }
        let mut held = self.seam.list().map_err(RetentionRefused::Seam)?;
        held.sort_unstable();
        let mut entries = Vec::new();
        for name in held {
            let bytes = self
                .seam
                .read(&name)
                .map_err(RetentionRefused::Seam)?
                .ok_or_else(|| {
                    RetentionRefused::Seam(SeamRefused {
                        reason: "a listed object could not be read back".to_owned(),
                    })
                })?;
            let computed = content_hash(&bytes);
            let class = if ObjectName::from_content_hash(computed) == name {
                match references.get(&name) {
                    None => RetentionClass::Orphan,
                    Some(plans) => {
                        let live: Vec<PlanHashRef> = plans
                            .iter()
                            .copied()
                            .filter(|plan| ledger.exempt(*plan))
                            .collect();
                        if live.is_empty() {
                            RetentionClass::TerminatedClosure {
                                plans: plans.clone(),
                            }
                        } else {
                            RetentionClass::Exempt { live_plans: live }
                        }
                    }
                }
            } else {
                RetentionClass::Corrupt { computed }
            };
            entries.push(RetentionEntry { name, class });
        }
        let missing = references
            .iter()
            .filter(|(name, _)| !entries.iter().any(|entry| entry.name == **name))
            .map(|(name, plans)| MissingReference {
                artifact: name.content_hash(),
                plans: plans.clone(),
            })
            .collect();
        Ok(RetentionPass { entries, missing })
    }

    /// Reclaim one artifact under an explicit end-of-life decision
    /// (ADR-0030 Rule 4). The store recomputes the retention pass
    /// itself — the ADR-0029 obligation-10 shape: no caller-computed
    /// liveness is accepted — and removes the object only when its
    /// every referencing apply has wholly terminated and its bytes
    /// still match its name. A live closure, an orphan, a corrupt
    /// object, and an object the store does not hold each refuse.
    ///
    /// Retention needs no call: silence retains, which is the
    /// fail-closed direction. The deciding surface's obligation to
    /// state both consequences is [`DeleteDecision`]'s documentation
    /// and the surface package's duty.
    ///
    /// # Errors
    ///
    /// [`ReclaimRefused`] naming the ground.
    pub fn reclaim(
        &mut self,
        decoded: &DecodedJournal,
        decision: &DeleteDecision,
    ) -> Result<ObjectName, ReclaimRefused> {
        let pass = self
            .retention_pass(decoded)
            .map_err(|refused| match refused {
                RetentionRefused::Journal(journal) => ReclaimRefused::Journal(journal),
                RetentionRefused::Seam(seam) => ReclaimRefused::Seam(seam),
            })?;
        let name = ObjectName::from_content_hash(decision.artifact());
        let Some(entry) = pass.entries.iter().find(|entry| entry.name == name) else {
            return Err(ReclaimRefused::NotHeld { name });
        };
        match &entry.class {
            RetentionClass::Exempt { live_plans } => Err(ReclaimRefused::StillLive {
                name,
                live_plans: live_plans.clone(),
            }),
            RetentionClass::Orphan => Err(ReclaimRefused::Orphan { name }),
            RetentionClass::Corrupt { computed } => Err(ReclaimRefused::Corrupt {
                name,
                computed: *computed,
            }),
            RetentionClass::TerminatedClosure { .. } => {
                self.seam.remove(&name).map_err(ReclaimRefused::Seam)?;
                Ok(name)
            }
        }
    }
}

/// SHA-256 over the artifact's exact bytes — the store's one identity
/// function (ADR-0030 Rule 2).
fn content_hash(bytes: &[u8]) -> ArtifactHashRef {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    ArtifactHashRef::from_bytes(hasher.finalize().into())
}

/// A refused deposit. No arm returns a reference: a reference in a
/// caller's hands is proof of a verified deposit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DepositRefused {
    /// Zero bytes: a zero-length artifact witnesses a failed capture,
    /// never a backup — fail closed.
    Empty,
    /// The seam refused.
    Seam(SeamRefused),
    /// The verifying re-read found no object where one was just
    /// acknowledged.
    VerifyMissing {
        /// The object that vanished.
        name: ObjectName,
    },
    /// The verifying re-read returned bytes that do not hash to the
    /// deposit.
    VerifyMismatch {
        /// The object that failed verification.
        name: ObjectName,
        /// What the read-back bytes actually hash to.
        computed: ArtifactHashRef,
    },
}

/// A refused fetch. No arm returns bytes: the store never serves
/// content whose hash is not the reference it was asked for by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchRefused {
    /// No object of the referenced name is held.
    Missing {
        /// The absent object.
        name: ObjectName,
    },
    /// The held bytes no longer hash to the reference.
    Corrupt {
        /// The corrupt object.
        name: ObjectName,
        /// What the held bytes actually hash to.
        computed: ArtifactHashRef,
    },
    /// The seam refused.
    Seam(SeamRefused),
}

/// One held object's classification under a retention pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetentionEntry {
    name: ObjectName,
    class: RetentionClass,
}

impl RetentionEntry {
    /// The object.
    #[must_use]
    pub const fn name(&self) -> ObjectName {
        self.name
    }

    /// Its classification.
    #[must_use]
    pub const fn class(&self) -> &RetentionClass {
        &self.class
    }
}

/// Why an object may — or may not — be reclaimed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetentionClass {
    /// Referenced by at least one apply whose ADR-0029 closure is
    /// still live: exempt, untouchable.
    Exempt {
        /// The referencing plans whose closures are live.
        live_plans: Vec<PlanHashRef>,
    },
    /// Every referencing apply has wholly terminated: eligible for an
    /// explicit end-of-life decision, and for nothing automatic.
    TerminatedClosure {
        /// The referencing plans, all terminated.
        plans: Vec<PlanHashRef>,
    },
    /// No journal record references this object, so its closure cannot
    /// be proven terminated: fail closed, never reclaimed.
    Orphan,
    /// The held bytes no longer hash to the object's name: never
    /// reclaimable, whatever its liveness — a corrupt recovery asset
    /// is a finding, not garbage.
    Corrupt {
        /// What the held bytes actually hash to.
        computed: ArtifactHashRef,
    },
}

/// A journal reference to an artifact the store does not hold: a
/// record is promising bytes the store cannot produce. Surfaced by
/// every retention pass; deciding what to do about one is the
/// consuming helper's, because only it knows whether the store it
/// opened is the store the journal meant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingReference {
    /// The promised artifact.
    artifact: ArtifactHashRef,
    /// The plans whose records promise it.
    plans: Vec<PlanHashRef>,
}

impl MissingReference {
    /// The promised artifact.
    #[must_use]
    pub const fn artifact(&self) -> ArtifactHashRef {
        self.artifact
    }

    /// The plans whose records promise it.
    #[must_use]
    pub fn plans(&self) -> &[PlanHashRef] {
        &self.plans
    }
}

/// A completed retention pass: every held object classified, every
/// unfulfillable reference surfaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetentionPass {
    entries: Vec<RetentionEntry>,
    missing: Vec<MissingReference>,
}

impl RetentionPass {
    /// Every held object's classification, in name order.
    #[must_use]
    pub fn entries(&self) -> &[RetentionEntry] {
        &self.entries
    }

    /// The objects whose every referencing apply has wholly
    /// terminated — the *only* candidates an end-of-life decision may
    /// name, and still nothing reclaims them without one.
    #[must_use]
    pub fn reclaimable(&self) -> Vec<ObjectName> {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.class, RetentionClass::TerminatedClosure { .. }))
            .map(|entry| entry.name)
            .collect()
    }

    /// Journal references the store cannot fulfill.
    #[must_use]
    pub fn missing_references(&self) -> &[MissingReference] {
        &self.missing
    }
}

/// A refused retention pass: the pass classifies nothing rather than
/// guessing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetentionRefused {
    /// The journal refused to ledger, on its own typed grounds.
    Journal(CompactedRefused),
    /// The seam refused.
    Seam(SeamRefused),
}

/// A refused reclaim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReclaimRefused {
    /// The journal refused to ledger, on its own typed grounds.
    Journal(CompactedRefused),
    /// The seam refused.
    Seam(SeamRefused),
    /// The store holds no such object.
    NotHeld {
        /// The absent object.
        name: ObjectName,
    },
    /// At least one referencing apply's closure is still live —
    /// ADR-0030 Rule 3 makes this arm unconditional: no decision
    /// reclaims a live artifact.
    StillLive {
        /// The object.
        name: ObjectName,
        /// The referencing plans whose closures are live.
        live_plans: Vec<PlanHashRef>,
    },
    /// No journal record references the object, so its closure cannot
    /// be proven terminated.
    Orphan {
        /// The unreferenced object.
        name: ObjectName,
    },
    /// The held bytes no longer hash to the object's name.
    Corrupt {
        /// The corrupt object.
        name: ObjectName,
        /// What the held bytes actually hash to.
        computed: ArtifactHashRef,
    },
}

/// The consequence of **retaining** a protection artifact past its
/// closure's termination, in the words REC-011 fixes (ADR-0030
/// Rule 4). The deciding surface renders this sentence — never a
/// paraphrase — wherever retention is offered as a choice.
pub const RETAIN_CONSEQUENCE: &str = "Retaining this backup preserves the state it captured at \
     backup time: a passphrase or key revoked since then remains usable with the backup.";

/// The consequence of **deleting** a protection artifact, in the words
/// REC-011 fixes (ADR-0030 Rule 4). The deciding surface renders this
/// sentence — never a paraphrase — wherever deletion is offered as a
/// choice.
pub const DELETE_CONSEQUENCE: &str = "Deleting this backup forfeits the disaster-recovery copy: \
     metadata corrupted or lost later is restorable only from a backup that still exists.";

/// The explicit end-of-life decision ADR-0030 Rule 4 demands before
/// the store deletes anything. Constructing one asserts that the
/// deciding surface displayed both [`RETAIN_CONSEQUENCE`] and
/// [`DELETE_CONSEQUENCE`] to the user whose decision this records —
/// this crate cannot see a display, so the assertion is the
/// constructing package's obligation, exactly as recorded in WP-070's
/// assignment. What the crate enforces is the other half: without one
/// of these values no deletion path exists, so the silent direction is
/// unrepresentable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeleteDecision {
    artifact: ArtifactHashRef,
}

impl DeleteDecision {
    /// Record the user's consequence-informed decision to delete the
    /// named artifact.
    #[must_use]
    pub const fn new(artifact: ArtifactHashRef) -> Self {
        DeleteDecision { artifact }
    }

    /// The artifact the decision names.
    #[must_use]
    pub const fn artifact(&self) -> ArtifactHashRef {
        self.artifact
    }
}
