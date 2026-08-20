//! The store increment's suite: ADR-0030's four rules as behaviour —
//! verified deposits, verified fetches, the liveness retention pass
//! over the journal's own records (imported obligation 2 of the
//! WP-070 store grant), the fail-closed orphan/missing/corrupt arms,
//! the explicit end-of-life decision, and the bytes-off-every-surface
//! sweep (imported obligation 1's crate half).

use std::collections::BTreeMap;

use partman_journal::Journal;
use partman_journal::records::{
    ArtifactHashRef, ArtifactStore, AuthorizationAct, AuthorizationTier, DisposalLinkage,
    PlanHashRef, ProtectionArm, ProtectionArtifactRef, ProtectionRecord, Record, RecordedInstant,
    TransitionRecord,
};
use partman_journal::retention::{CompactedRefused, DecodedJournal, decode_journal};
use partman_statemachine::{Effect, Transition};

use super::{
    DELETE_CONSEQUENCE, DeleteDecision, DepositRefused, FetchRefused, NameInvalid, ObjectName,
    RETAIN_CONSEQUENCE, ReclaimRefused, RetentionClass, SeamRefused, Store, StoreSeam,
    content_hash,
};

// ---------------------------------------------------------------------
// The fake seam.

/// How the fake misbehaves on `put`, for the verification arms.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PutMode {
    /// Store exactly what was offered.
    Honest,
    /// Acknowledge and store nothing — the vanished-object shape.
    DropSilently,
    /// Acknowledge and store tampered bytes — the corrupted-write
    /// shape.
    CorruptStored,
    /// Refuse.
    Refuse,
}

/// An in-memory seam. Tests reach into `objects` directly to model
/// after-the-fact corruption a lying platform could produce.
struct MapSeam {
    objects: BTreeMap<ObjectName, Vec<u8>>,
    put_mode: PutMode,
}

impl MapSeam {
    fn honest() -> Self {
        MapSeam {
            objects: BTreeMap::new(),
            put_mode: PutMode::Honest,
        }
    }

    fn with_mode(put_mode: PutMode) -> Self {
        MapSeam {
            objects: BTreeMap::new(),
            put_mode,
        }
    }
}

impl StoreSeam for MapSeam {
    fn put(&mut self, name: &ObjectName, bytes: &[u8]) -> Result<(), SeamRefused> {
        match self.put_mode {
            PutMode::Honest => {
                self.objects.insert(*name, bytes.to_vec());
                Ok(())
            }
            PutMode::DropSilently => Ok(()),
            PutMode::CorruptStored => {
                let mut tampered = bytes.to_vec();
                tampered[0] ^= 0xFF;
                self.objects.insert(*name, tampered);
                Ok(())
            }
            PutMode::Refuse => Err(SeamRefused {
                reason: "the fake refuses puts".to_owned(),
            }),
        }
    }

    fn read(&self, name: &ObjectName) -> Result<Option<Vec<u8>>, SeamRefused> {
        Ok(self.objects.get(name).cloned())
    }

    fn list(&self) -> Result<Vec<ObjectName>, SeamRefused> {
        Ok(self.objects.keys().copied().collect())
    }

    fn remove(&mut self, name: &ObjectName) -> Result<(), SeamRefused> {
        self.objects.remove(name);
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Journal-authoring helpers, in the retention suite's own shape.

fn plan(tag: u8) -> PlanHashRef {
    PlanHashRef::from_bytes([tag; 32])
}

fn act(target: PlanHashRef) -> Record {
    Record::AuthorizationAct(AuthorizationAct::new(target, AuthorizationTier::FloorAct))
}

/// The fixed instant this suite records transitions at; retention
/// reads liveness from the chain, not the clock.
fn instant_t() -> RecordedInstant {
    RecordedInstant::from_secs(1_700_000_000)
}

fn completed(target: PlanHashRef) -> Record {
    Record::Transition(
        TransitionRecord::terminal(
            target,
            Transition::PostconditionsPass,
            Effect::Complete,
            None,
            instant_t(),
        )
        .expect("terminal row"),
    )
}

fn failed_with_disposal(target: PlanHashRef, recovery: PlanHashRef) -> Record {
    Record::Transition(
        TransitionRecord::terminal(
            target,
            Transition::FailureAccepted,
            Effect::Partial,
            Some(DisposalLinkage::new(recovery)),
            instant_t(),
        )
        .expect("the disposal arm"),
    )
}

fn parse_backup(target: PlanHashRef, artifact: ProtectionArtifactRef) -> Record {
    Record::Protection(
        ProtectionRecord::new(target, ProtectionArm::ParseBackupVerified { artifact })
            .expect("a valid protection record"),
    )
}

fn decoded_of(records: &[Record]) -> DecodedJournal {
    let mut journal = Journal::new();
    for record in records {
        journal
            .append(&record.encode().expect("encodes"))
            .expect("bounded");
    }
    decode_journal(journal.bytes()).expect("intact")
}

/// A reference for bytes that were never deposited — for authoring
/// journal records about a given content.
fn reference_to(bytes: &[u8]) -> ProtectionArtifactRef {
    ProtectionArtifactRef::new(content_hash(bytes), ArtifactStore::HelperProtectionStore)
}

// ---------------------------------------------------------------------

// Requirements: REC-011, PART-013
//   A deposit is verified by re-read before any reference exists —
//   REC-011's "create and verify" and PART-013's "verified" as the
//   store's own protocol: the returned reference's content hash equals
//   an independent SHA-256 of the deposited bytes (pinned against the
//   NIST `abc` vector, not this crate's own hasher), the object is
//   held under the hash's canonical hex name, and a second deposit of
//   the same bytes is the same one artifact. No arm hands out a
//   reference past a failure: zero bytes refuse (a zero-length
//   artifact witnesses a failed capture, never a backup), a seam that
//   acknowledges and stores nothing is caught by the re-read, a seam
//   that stores tampered bytes is caught by the recomputed hash, and a
//   refusing seam's refusal passes through — each fail-closed.
// Evidence: a_deposit_is_verified_by_re_read_and_only_then_referenced
#[test]
fn a_deposit_is_verified_by_re_read_and_only_then_referenced() {
    // The NIST FIPS 180-2 `abc` vector: an independent transcription,
    // so the store's hasher is checked against the standard rather
    // than against itself.
    let abc_digest: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    let mut store = Store::new(MapSeam::honest());
    let reference = store.deposit(b"abc").expect("a verified deposit");
    assert_eq!(reference.content(), ArtifactHashRef::from_bytes(abc_digest));
    assert_eq!(reference.store(), ArtifactStore::HelperProtectionStore);
    let name = ObjectName::from_content_hash(reference.content());
    assert_eq!(
        name.as_hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(store.seam.objects.len(), 1);
    assert_eq!(store.seam.objects.get(&name), Some(&b"abc".to_vec()));

    // Content-addressed: the same bytes again are the same artifact.
    let again = store.deposit(b"abc").expect("idempotent");
    assert_eq!(again, reference);
    assert_eq!(store.seam.objects.len(), 1);

    // Zero bytes refuse.
    assert_eq!(store.deposit(b""), Err(DepositRefused::Empty));

    // A seam that acknowledges and stores nothing: the re-read
    // catches it, and no reference exists.
    let mut dropping = Store::new(MapSeam::with_mode(PutMode::DropSilently));
    assert_eq!(
        dropping.deposit(b"abc"),
        Err(DepositRefused::VerifyMissing { name })
    );

    // A seam that stores tampered bytes: the recomputed hash catches
    // it and names what the bytes actually are.
    let mut corrupting = Store::new(MapSeam::with_mode(PutMode::CorruptStored));
    let mut tampered = b"abc".to_vec();
    tampered[0] ^= 0xFF;
    assert_eq!(
        corrupting.deposit(b"abc"),
        Err(DepositRefused::VerifyMismatch {
            name,
            computed: content_hash(&tampered),
        })
    );

    // A refusing seam passes its refusal through.
    let mut refusing = Store::new(MapSeam::with_mode(PutMode::Refuse));
    assert_eq!(
        refusing.deposit(b"abc"),
        Err(DepositRefused::Seam(SeamRefused {
            reason: "the fake refuses puts".to_owned(),
        }))
    );
}

// Requirements: REC-011
//   A fetch returns only bytes that hash to the reference they were
//   asked for by: a held artifact round-trips exactly, an unknown
//   reference is a typed Missing, and an object corrupted after
//   deposit is a typed Corrupt naming what the bytes actually hash to
//   — the corrupted bytes themselves are never returned, so a restore
//   can never be fed a backup that no longer is one.
// Evidence: a_fetch_returns_only_bytes_that_hash_to_the_reference
#[test]
fn a_fetch_returns_only_bytes_that_hash_to_the_reference() {
    let mut store = Store::new(MapSeam::honest());
    let payload = b"a parse-level backup of both table copies".to_vec();
    let reference = store.deposit(&payload).expect("a verified deposit");
    assert_eq!(store.fetch(&reference), Ok(payload.clone()));

    let unknown = reference_to(b"bytes nothing deposited");
    assert_eq!(
        store.fetch(&unknown),
        Err(FetchRefused::Missing {
            name: ObjectName::from_content_hash(unknown.content()),
        })
    );

    // Corrupt the held object underneath the store: the fetch refuses
    // and no bytes come back.
    let name = ObjectName::from_content_hash(reference.content());
    let mut tampered = payload;
    tampered[0] ^= 0xFF;
    store.seam.objects.insert(name, tampered.clone());
    assert_eq!(
        store.fetch(&reference),
        Err(FetchRefused::Corrupt {
            name,
            computed: content_hash(&tampered),
        })
    );
}

// Requirements: JRN-004, REC-011
//   Imported obligation 2 of the WP-070 store grant (ADR-0030 Rule 3):
//   a retention pass reclaims no artifact whose creating apply or
//   referencing closure is non-terminal. Over one authored journal and
//   one store: an artifact of a live apply is exempt; an artifact of a
//   failed apply whose recovery plan is still live is exempt through
//   ADR-0027's linkage closure; an artifact referenced by both a
//   terminated and a live apply is exempt because one live reference
//   suffices (content addressing makes shared artifacts real); and
//   only the artifact whose every referencing apply has wholly
//   terminated is reclaimable — computed by the pass itself from the
//   journal's records, with no API accepting a caller-computed
//   liveness.
// Evidence: a_retention_pass_reclaims_no_artifact_with_a_live_closure
#[test]
fn a_retention_pass_reclaims_no_artifact_with_a_live_closure() {
    let live = plan(0xA1);
    let done = plan(0xB2);
    let orig_failed = plan(0xC3);
    let rec_live = plan(0xD4);

    let mut store = Store::new(MapSeam::honest());
    let of_live = store
        .deposit(b"backup for the live apply")
        .expect("deposits");
    let of_done = store
        .deposit(b"backup for the finished apply")
        .expect("deposits");
    let of_failed = store
        .deposit(b"backup for the recovered apply")
        .expect("deposits");
    let shared = store.deposit(b"one backup, two applies").expect("deposits");

    let decoded = decoded_of(&[
        act(live),
        parse_backup(live, of_live),
        parse_backup(live, shared),
        act(done),
        parse_backup(done, of_done),
        parse_backup(done, shared),
        completed(done),
        act(orig_failed),
        parse_backup(orig_failed, of_failed),
        failed_with_disposal(orig_failed, rec_live),
        act(rec_live),
    ]);

    let pass = store.retention_pass(&decoded).expect("the pass runs");
    assert_eq!(
        pass.reclaimable(),
        vec![ObjectName::from_content_hash(of_done.content())]
    );
    assert!(pass.missing_references().is_empty());

    let class_of = |reference: &ProtectionArtifactRef| {
        pass.entries()
            .iter()
            .find(|entry| entry.name() == ObjectName::from_content_hash(reference.content()))
            .expect("classified")
            .class()
            .clone()
    };
    assert_eq!(
        class_of(&of_live),
        RetentionClass::Exempt {
            live_plans: vec![live]
        }
    );
    assert_eq!(
        class_of(&of_done),
        RetentionClass::TerminatedClosure { plans: vec![done] }
    );
    assert_eq!(
        class_of(&of_failed),
        RetentionClass::Exempt {
            live_plans: vec![orig_failed]
        }
    );
    // The shared artifact: one live reference among its referencing
    // plans keeps it, whatever the other plans did.
    assert_eq!(
        class_of(&shared),
        RetentionClass::Exempt {
            live_plans: vec![live]
        }
    );
}

// Requirements: JRN-004, REC-011
//   The fail-closed arms of the retention pass: an object no journal
//   record references is an orphan — its closure cannot be proven
//   terminated, so it is never reclaimable and an end-of-life decision
//   naming it refuses; a journal reference to an object the store does
//   not hold is surfaced as a missing reference naming the artifact
//   and its promising plans (a record is promising bytes the store
//   cannot produce); and an object whose held bytes no longer hash to
//   its name is Corrupt — never reclaimable even with its closure
//   wholly terminated, because a corrupt recovery asset is a finding,
//   not garbage.
// Evidence: the_orphan_missing_and_corrupt_arms_fail_closed
#[test]
fn the_orphan_missing_and_corrupt_arms_fail_closed() {
    let done = plan(0xE5);
    let mut store = Store::new(MapSeam::honest());

    // An orphan: deposited, referenced by nothing.
    let orphan = store
        .deposit(b"an object no record references")
        .expect("deposits");
    let orphan_name = ObjectName::from_content_hash(orphan.content());

    // A referenced-but-terminated artifact, corrupted in place.
    let corrupt = store.deposit(b"a backup that will rot").expect("deposits");
    let corrupt_name = ObjectName::from_content_hash(corrupt.content());

    // A reference the store cannot fulfill.
    let promised = reference_to(b"bytes the store never held");

    let decoded = decoded_of(&[
        act(done),
        parse_backup(done, corrupt),
        parse_backup(done, promised),
        completed(done),
    ]);

    let mut rotted = b"a backup that will rot".to_vec();
    rotted[0] ^= 0xFF;
    store.seam.objects.insert(corrupt_name, rotted.clone());

    let pass = store.retention_pass(&decoded).expect("the pass runs");
    assert_eq!(pass.reclaimable(), Vec::<ObjectName>::new());

    let class_of = |name: ObjectName| {
        pass.entries()
            .iter()
            .find(|entry| entry.name() == name)
            .expect("classified")
            .class()
            .clone()
    };
    assert_eq!(class_of(orphan_name), RetentionClass::Orphan);
    assert_eq!(
        class_of(corrupt_name),
        RetentionClass::Corrupt {
            computed: content_hash(&rotted)
        }
    );
    assert_eq!(pass.missing_references().len(), 1);
    assert_eq!(pass.missing_references()[0].artifact(), promised.content());
    assert_eq!(pass.missing_references()[0].plans(), &[done]);

    // The refusing reclaim arms.
    assert_eq!(
        store.reclaim(&decoded, &DeleteDecision::new(orphan.content())),
        Err(ReclaimRefused::Orphan { name: orphan_name })
    );
    assert_eq!(
        store.reclaim(&decoded, &DeleteDecision::new(corrupt.content())),
        Err(ReclaimRefused::Corrupt {
            name: corrupt_name,
            computed: content_hash(&rotted),
        })
    );
    assert_eq!(
        store.reclaim(&decoded, &DeleteDecision::new(promised.content())),
        Err(ReclaimRefused::NotHeld {
            name: ObjectName::from_content_hash(promised.content()),
        })
    );
}

// Requirements: JRN-004, REC-011
//   Reclaim recomputes liveness itself — the ADR-0029 obligation-10
//   shape carried to the store: no caller-computed liveness is
//   accepted, so a decision naming a live artifact refuses citing the
//   live plans, a decision naming a wholly terminated artifact removes
//   exactly that object, the removal is visible (a second decision
//   refuses NotHeld), and a journal the ledger itself refuses — the
//   double-terminal shape — refuses the reclaim on the journal's own
//   typed ground rather than proceeding over a corrupt liveness
//   picture.
// Evidence: reclaim_recomputes_liveness_and_refuses_every_arm_but_terminated
#[test]
fn reclaim_recomputes_liveness_and_refuses_every_arm_but_terminated() {
    let live = plan(0x11);
    let done = plan(0x22);
    let mut store = Store::new(MapSeam::honest());
    let of_live = store
        .deposit(b"still insuring a live apply")
        .expect("deposits");
    let of_done = store
        .deposit(b"insured an apply that finished")
        .expect("deposits");
    let decoded = decoded_of(&[
        act(live),
        parse_backup(live, of_live),
        act(done),
        parse_backup(done, of_done),
        completed(done),
    ]);

    assert_eq!(
        store.reclaim(&decoded, &DeleteDecision::new(of_live.content())),
        Err(ReclaimRefused::StillLive {
            name: ObjectName::from_content_hash(of_live.content()),
            live_plans: vec![live],
        })
    );

    let removed = store
        .reclaim(&decoded, &DeleteDecision::new(of_done.content()))
        .expect("a terminated closure reclaims");
    assert_eq!(removed, ObjectName::from_content_hash(of_done.content()));
    assert_eq!(store.seam.objects.len(), 1);
    assert!(
        store
            .seam
            .objects
            .contains_key(&ObjectName::from_content_hash(of_live.content()))
    );
    assert_eq!(
        store.reclaim(&decoded, &DeleteDecision::new(of_done.content())),
        Err(ReclaimRefused::NotHeld {
            name: ObjectName::from_content_hash(of_done.content()),
        })
    );

    // A journal the ledger refuses: two terminals for one plan.
    let double = decoded_of(&[act(done), completed(done), completed(done)]);
    assert!(matches!(
        store.reclaim(&double, &DeleteDecision::new(of_live.content())),
        Err(ReclaimRefused::Journal(
            CompactedRefused::TerminalTwice { .. }
        ))
    ));
}

// Requirements: REC-011
//   The end-of-life vocabulary (ADR-0030 Rule 4): deletion demands an
//   explicit DeleteDecision — no other deletion path exists, so the
//   silent direction is unrepresentable at this layer — and the two
//   consequence sentences the deciding surface must render are pinned
//   in doc-code agreement with schemas/artifact-store.md: retention's
//   sentence states that revoked credentials remain usable with the
//   backup, deletion's sentence states that the disaster-recovery copy
//   is forfeited, and the schema document carries both verbatim so a
//   consumer reading either source renders the same words.
// Evidence: the_end_of_life_vocabulary_states_both_consequences_and_is_pinned
#[test]
fn the_end_of_life_vocabulary_states_both_consequences_and_is_pinned() {
    assert!(RETAIN_CONSEQUENCE.contains("revoked"));
    assert!(RETAIN_CONSEQUENCE.contains("remains usable"));
    assert!(DELETE_CONSEQUENCE.contains("forfeits"));
    assert!(DELETE_CONSEQUENCE.contains("still exists"));
    assert_ne!(RETAIN_CONSEQUENCE, DELETE_CONSEQUENCE);

    let schema = include_str!("../../../schemas/artifact-store.md");
    assert!(
        schema.contains(RETAIN_CONSEQUENCE),
        "the schema document must carry the retain consequence verbatim"
    );
    assert!(
        schema.contains(DELETE_CONSEQUENCE),
        "the schema document must carry the delete consequence verbatim"
    );

    let artifact = ArtifactHashRef::from_bytes([0x5A; 32]);
    assert_eq!(DeleteDecision::new(artifact).artifact(), artifact);
}

// Requirements: SAFE-006, REC-011
//   Imported obligation 1's crate half (ADR-0030): artifact bytes are
//   absent from every surface this crate produces, with the hash
//   reference present where the bytes are not. Artifacts whose content
//   embeds a raw exemplar of every SEC-006 identifier class are
//   deposited, fetched, classified, refused and reclaimed, and every
//   rendering the crate can emit along the way — references, object
//   names, retention passes, and each refusal, including the
//   verify-mismatch and corrupt arms whose inputs are the tampered
//   bytes themselves — is asserted free of every exemplar, while the
//   reference's own hex spelling is present on the reference's
//   rendering. Seam refusal text is the seam implementation's
//   obligation, recorded on the trait's contract.
// Evidence: no_surface_of_the_store_carries_artifact_bytes
#[test]
fn no_surface_of_the_store_carries_artifact_bytes() {
    // One raw exemplar per SEC-006 identifier class, as WP-040's gate
    // spells them.
    const EXEMPLARS: [&str; 7] = [
        "WD-WCC4N5PZ3RKE",
        "/dev/disk/by-id/ata-WDC_WD40EFRX-68N32N0",
        "C:\\Users\\nate\\backup.img",
        "Backup Disk 2",
        "nate",
        "-----BEGIN OPENSSH PRIVATE KEY-----",
        "backup.img",
    ];
    let payload = EXEMPLARS.join("\n").into_bytes();
    let assert_clean = |surface: &str| {
        for exemplar in EXEMPLARS {
            assert!(
                !surface.contains(exemplar),
                "a store surface carried artifact content: {surface}"
            );
        }
    };

    let mut store = Store::new(MapSeam::honest());
    let reference = store.deposit(&payload).expect("deposits");
    let name = ObjectName::from_content_hash(reference.content());
    assert_clean(&format!("{reference:?}"));
    let rendered_name = format!("{name:?}");
    assert!(
        rendered_name.contains(&name.as_hex()),
        "the hash reference must be present where the bytes are not"
    );
    assert_clean(&rendered_name);

    // The verify-mismatch arm renders without echoing the tampered
    // content it was computed from.
    let mut corrupting = Store::new(MapSeam::with_mode(PutMode::CorruptStored));
    let mismatch = corrupting.deposit(&payload).expect_err("mismatch");
    assert_clean(&format!("{mismatch:?}"));

    // A live-referenced store and its pass, then corruption underneath
    // and the corrupt arms.
    let target = plan(0x33);
    let decoded = decoded_of(&[act(target), parse_backup(target, reference)]);
    let pass = store.retention_pass(&decoded).expect("the pass runs");
    assert_clean(&format!("{pass:?}"));
    let refused = store
        .reclaim(&decoded, &DeleteDecision::new(reference.content()))
        .expect_err("still live");
    assert_clean(&format!("{refused:?}"));

    let mut rotted = payload.clone();
    rotted[0] ^= 0xFF;
    store.seam.objects.insert(name, rotted);
    let corrupt = store.fetch(&reference).expect_err("corrupt");
    assert_clean(&format!("{corrupt:?}"));
    let pass = store.retention_pass(&decoded).expect("the pass runs");
    assert_clean(&format!("{pass:?}"));
}

// Requirements: REC-011
//   The store's one on-disk spelling: an object name renders as
//   exactly 64 lowercase hexadecimal characters, the rendering parses
//   back to the same name, and every other spelling refuses with its
//   defect named — a wrong length, an uppercase digit (one canonical
//   spelling, so one directory cannot hold two objects with one
//   identity), and a non-hex character each a typed refusal — so a
//   stray file in a real store directory can never masquerade as an
//   object.
// Evidence: the_object_name_spelling_round_trips_and_refuses_impostors
#[test]
fn the_object_name_spelling_round_trips_and_refuses_impostors() {
    let name = ObjectName::from_content_hash(ArtifactHashRef::from_bytes([0xAB; 32]));
    let hex = name.as_hex();
    assert_eq!(hex.len(), 64);
    assert!(
        hex.bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    );
    assert_eq!(ObjectName::from_hex(&hex), Ok(name));
    assert_eq!(name.content_hash(), ArtifactHashRef::from_bytes([0xAB; 32]));

    assert_eq!(
        ObjectName::from_hex("abc123"),
        Err(NameInvalid::Length { length: 6 })
    );
    let uppercase = hex.to_uppercase();
    assert_eq!(
        ObjectName::from_hex(&uppercase),
        Err(NameInvalid::NotLowercaseHex { index: 0 })
    );
    let mut stray = hex.clone();
    stray.replace_range(10..11, "z");
    assert_eq!(
        ObjectName::from_hex(&stray),
        Err(NameInvalid::NotLowercaseHex { index: 10 })
    );
}
