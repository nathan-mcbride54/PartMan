//! What each fixture must *contain* for its stated reason to be true.
//!
//! A project review found the systemic weakness in this crate: tests whose names
//! are the safety claim. The root cause it named was that no test bound a
//! catalogue fixture's bytes to its rationale — every layout and signature test
//! rebuilt its own image from its own literals, so the catalogue could produce
//! something else entirely and the suite stayed green.
//!
//! That was measured rather than assumed. With [`crate::catalogue`]'s LUKS2
//! builder emptied to a blank image, and the multi-signature builder stripped of
//! its stale mdraid half, **all 64 tests passed.** `luks2-whole-disk-512.img`
//! would have been 4 MiB of zeros while its rationale claimed FS-004 LUKS
//! detection, and the traceability record would have gone on citing it.
//!
//! This module closes that. Every catalogue entry has a [`Claim`] here, the set
//! is exhaustive in both directions, and [`crate::catalogue::generate`] refuses
//! to write an image that does not satisfy its own. A fixture's purpose is
//! computed from its bytes, exactly as [`crate::interlock`] computes a target's
//! disposability rather than accepting an assertion about it.
//!
//! # Independence
//!
//! A check that calls the writer's own checksum function proves only that the
//! writer agrees with itself. So the three checksums that matter are
//! reimplemented here by a different method:
//!
//! * CRC-32 by table lookup, where [`crate::layout::crc32`] iterates bits.
//! * LVM2's CRC bitwise, where [`crate::signature::lvm2_crc`] uses a nibble
//!   table.
//! * mdraid's folded word sum with an iterative fold.
//!
//! Reimplementation alone would still be only two opinions, so each is anchored
//! outside this repository: CRC-32 against the published IEEE check value, and
//! the other two against `libblkid` 2.41, which validates both before it will
//! name a format at all.
//!
//! # Every check begins by fixing the length
//!
//! Each check calls [`expect_length`] first, so every fixed offset below it is
//! already known to be in range. That is what keeps a malformed input a
//! *refusal* rather than a panic — this runs inside `generate`, where a panic
//! and a refusal are very different failures.

use crate::catalogue;

/// A property a fixture's bytes must have for its rationale to hold.
pub struct Claim {
    /// The fixture this constrains, by catalogue name.
    pub fixture: &'static str,
    /// The property, phrased so a failure names what was lost rather than which
    /// assertion fired.
    pub property: &'static str,
    /// Decide it from the bytes alone.
    pub check: fn(&[u8]) -> Result<(), String>,
}

/// Why a fixture's bytes do not support its rationale.
#[derive(Debug)]
pub enum Missing {
    /// No claim is registered for this fixture, so nothing constrains it.
    ///
    /// This is a refusal rather than a pass. A fixture with no registered claim
    /// is exactly the state this module exists to prevent, and treating an
    /// unknown name as "nothing to check" would reintroduce it silently, one
    /// new fixture at a time.
    NoClaim {
        /// The fixture name.
        fixture: String,
    },
    /// The bytes do not have the property.
    Unsatisfied {
        /// The fixture name.
        fixture: String,
        /// The property that was supposed to hold.
        property: &'static str,
        /// What was found instead.
        detail: String,
    },
}

impl core::fmt::Display for Missing {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoClaim { fixture } => write!(
                formatter,
                "{fixture} has no registered claim in `evidence`; a fixture nothing constrains is \
                 a fixture that can quietly stop being what it is named for"
            ),
            Self::Unsatisfied {
                fixture,
                property,
                detail,
            } => write!(
                formatter,
                "{fixture} no longer satisfies its stated purpose ({property}): {detail}"
            ),
        }
    }
}

impl std::error::Error for Missing {}

/// Check one fixture's bytes against its registered claim.
///
/// # Errors
///
/// Returns [`Missing::NoClaim`] if nothing is registered for `fixture`, and
/// [`Missing::Unsatisfied`] if the bytes do not have the property.
pub fn verify(fixture: &str, bytes: &[u8]) -> Result<(), Missing> {
    let claim = claims()
        .into_iter()
        .find(|claim| claim.fixture == fixture)
        .ok_or_else(|| Missing::NoClaim {
            fixture: fixture.to_owned(),
        })?;
    (claim.check)(bytes).map_err(|detail| Missing::Unsatisfied {
        fixture: fixture.to_owned(),
        property: claim.property,
        detail,
    })
}

/// Check every catalogue fixture against its claim, building each from source.
///
/// # Errors
///
/// Returns the first fixture whose bytes do not support its rationale.
pub fn verify_catalogue() -> Result<(), Missing> {
    let mut images = Vec::new();
    for fixture in catalogue::catalogue() {
        let bytes = (fixture.build)().into_bytes();
        verify(fixture.name, &bytes)?;
        images.push((fixture.name, bytes));
    }
    expect_distinct_identities(&images)
}

/// No two fixtures may claim the same array, volume, or disk identity.
///
/// Every [`Claim`] above looks at one image alone, so none of them can see this.
/// An adversarial pass pointed out the consequence: reseeding
/// `mdraid-1.2-member-512.img` with the seed the stale-mdraid fixture uses gives
/// two catalogue images the same 16-byte array UUID, so both declare membership
/// in one array — and "no array assembles from one fixture" becomes false
/// against the catalogue it lives in, while manufacturing exactly the kind of
/// identifier collision SI-27 is trying to reason about.
fn expect_distinct_identities(images: &[(&'static str, Vec<u8>)]) -> Result<(), Missing> {
    let mut seen: Vec<(&str, &'static str, Vec<u8>)> = Vec::new();
    for (name, bytes) in images {
        for (kind, identity) in identities(name, bytes) {
            if let Some((_, other, _)) = seen
                .iter()
                .find(|(other_kind, _, value)| *other_kind == kind && value == &identity)
            {
                return Err(Missing::Unsatisfied {
                    fixture: (*name).to_owned(),
                    property: "every fixture's identity is its own",
                    detail: format!(
                        "its {kind} is the same as {other}'s, so two fixtures claim one identity"
                    ),
                });
            }
            seen.push((kind, name, identity));
        }
    }
    Ok(())
}

/// The identities a fixture publishes, by kind, for the uniqueness check.
fn identities(name: &str, bytes: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut found = Vec::new();
    match name {
        "gpt-basic-512.img" | "gpt-basic-4kn.img" => {
            let sector = if name.ends_with("4kn.img") { 4096 } else { 512 };
            if let Some(header) = gpt_header(bytes, sector) {
                found.push(("disk GUID", header.disk_guid.to_vec()));
            }
        }
        "luks2-whole-disk-512.img" => {
            found.push(("LUKS UUID", ascii_field(bytes, 168, 40).into_bytes()));
        }
        "lvm2-pv-orphan-512.img" => {
            found.push(("PV UUID", ascii_field(bytes, 544, 32).into_bytes()));
        }
        "mdraid-1.2-member-512.img" => {
            found.push(("array UUID", bytes[4112..4128].to_vec()));
        }
        "ext4-with-stale-mdraid-090-512.img" => {
            found.push(("file-system UUID", bytes[1128..1144].to_vec()));
            let at = mdraid_090_offset(bytes.len());
            // The 0.90 set UUID is four non-adjacent words.
            let mut uuid = bytes[at + 20..at + 24].to_vec();
            uuid.extend_from_slice(&bytes[at + 52..at + 64]);
            found.push(("array UUID", uuid));
        }
        _ => {}
    }
    found
}

/// Every claim this crate makes about a fixture's contents.
///
/// Exhaustive over [`catalogue::catalogue`] in both directions, and a test
/// enforces it: a fixture added without a claim fails, and a claim naming no
/// fixture fails.
#[must_use]
pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            fixture: "blank-512.img",
            property: "positively observed to have no table, and not merely unread",
            check: check_blank,
        },
        Claim {
            fixture: "gpt-basic-512.img",
            property: "a complete, internally consistent GPT at 512-byte sectors",
            check: check_gpt_basic,
        },
        Claim {
            fixture: "gpt-basic-4kn.img",
            property: "a 4Kn table, which is not a 512-byte table with different numbers",
            check: check_gpt_4kn,
        },
        Claim {
            fixture: "mbr-basic-512.img",
            property: "an MBR that a reader must not mistake for a GPT's protective one",
            check: check_mbr_basic,
        },
        Claim {
            fixture: "gpt-invalid-primary-valid-backup-512.img",
            property: "damaged primary, intact backup — recoverable, and NOT ADR-C3 Indeterminate",
            check: check_gpt_invalid_primary,
        },
        Claim {
            fixture: "gpt-conflicting-tables-512.img",
            property: "two independently valid tables that disagree — ADR-C3 Indeterminate",
            check: check_gpt_conflicting,
        },
        Claim {
            fixture: "gpt-missing-backup-512.img",
            property: "a valid primary with no backup at all",
            check: check_gpt_missing_backup,
        },
        Claim {
            fixture: "hybrid-mbr-gpt-512.img",
            property: "one disk described twice, by two schemes that can disagree",
            check: check_hybrid,
        },
        Claim {
            fixture: "apm-basic-512.img",
            property: "an Apple Partition Map whose fields are big-endian",
            check: check_apm,
        },
        Claim {
            fixture: "luks2-whole-disk-512.img",
            property: "a LUKS2 header a prober will name, whole-disk, with an identity",
            check: check_luks2,
        },
        Claim {
            fixture: "lvm2-pv-orphan-512.img",
            property: "an LVM2 PV label whose checksum `libblkid` will accept",
            check: check_lvm2,
        },
        Claim {
            fixture: "mdraid-1.2-member-512.img",
            property: "an mdraid 1.2 superblock `blkid` will validate, not merely match",
            check: check_mdraid_12,
        },
        Claim {
            fixture: "ext4-with-stale-mdraid-090-512.img",
            property: "a live ext4 AND an obsolete 0.90 array membership, on one device",
            check: check_ext4_with_stale_mdraid,
        },
    ]
}

/// Every fixture in the catalogue is this size, which fixes every offset below.
const FIXTURE_BYTES: usize = 4 * 1024 * 1024;

/// `EXT4_FEATURE_INCOMPAT_EXTENTS`, which is part of what makes a superblock
/// ext4 rather than ext2 as far as `libblkid` is concerned.
const INCOMPAT_EXTENTS: u32 = 0x40;
/// `EXT4_FEATURE_COMPAT_HAS_JOURNAL`, the other half of that distinction.
const COMPAT_HAS_JOURNAL: u32 = 0x04;

fn check_blank(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    if let Some(at) = bytes.iter().position(|byte| *byte != 0) {
        return Err(format!("byte {at} is non-zero, so the medium is not blank"));
    }
    Ok(())
}

fn check_gpt_basic(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    expect_protective_mbr(bytes)?;
    let primary = gpt_header(bytes, 512).ok_or("no valid primary header at LBA 1")?;
    let backup =
        gpt_header(bytes, bytes.len() - 512).ok_or("no valid backup header at the last LBA")?;
    expect_entry_array(bytes, &primary, 512)?;
    expect_entry_array(bytes, &backup, 512)?;
    if primary.entry_array_crc != backup.entry_array_crc {
        return Err("the two copies describe different partitions".to_owned());
    }
    if primary.disk_guid != backup.disk_guid {
        return Err("the two copies name different disks".to_owned());
    }
    expect_partitions_usable(bytes, &primary, 512, 2)
}

/// Confirm the partitions a header points at are a table a reader could use.
///
/// Ordering, overlap, and — the part that was missing — containment inside the
/// header's own declared usable range. Without it a partition could run past the
/// end of the medium or over the table itself, and an adversarial pass confirmed
/// exactly that: `gpt-basic-512.img` accepted a partition from LBA 100 000 to
/// 200 000 on an 8192-sector image, and every fixture derived from it inherited
/// the defect.
fn expect_partitions_usable(
    bytes: &[u8],
    header: &GptHeader,
    sector: u64,
    minimum: usize,
) -> Result<(), String> {
    let partitions = gpt_partitions(bytes, header, sector);
    if partitions.len() < minimum {
        return Err(format!(
            "{} populated partition entries, fewer than the {minimum} this fixture needs",
            partitions.len()
        ));
    }
    let sectors = u64::try_from(bytes.len()).map_err(|_| "image too large")? / sector;
    for (index, partition) in partitions.iter().enumerate() {
        if partition.last_lba < partition.first_lba {
            return Err(format!("partition {index} ends before it starts"));
        }
        if partition.first_lba < header.first_usable_lba
            || partition.last_lba > header.last_usable_lba
        {
            return Err(format!(
                "partition {index} spans LBA {}..={}, outside the header's usable range \
                 {}..={}",
                partition.first_lba,
                partition.last_lba,
                header.first_usable_lba,
                header.last_usable_lba
            ));
        }
        if partition.last_lba >= sectors {
            return Err(format!(
                "partition {index} ends at LBA {} on a {sectors}-sector image",
                partition.last_lba
            ));
        }
        for other in &partitions[index + 1..] {
            if partition.first_lba <= other.last_lba && other.first_lba <= partition.last_lba {
                return Err("two partitions overlap, so this is not a valid table".to_owned());
            }
        }
    }
    Ok(())
}

fn check_gpt_4kn(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    expect_protective_mbr(bytes)?;
    // The header is one *logical* sector in, which at 4Kn is byte 4096. A
    // reader that assumed 512 would look at byte 512 and find the zero padding
    // of the protective-MBR sector.
    let primary = gpt_header(bytes, 4096).ok_or("no valid primary header at LBA 1")?;
    if bytes[512..520] == *b"EFI PART" {
        return Err(
            "a GPT signature sits at byte 512, so this fixture no longer distinguishes a 4Kn \
             reader from a 512-byte one"
                .to_owned(),
        );
    }
    let backup =
        gpt_header(bytes, bytes.len() - 4096).ok_or("no valid backup header at the last LBA")?;
    expect_entry_array(bytes, &primary, 4096)?;
    expect_entry_array(bytes, &backup, 4096)?;
    if primary.my_lba != 1 {
        return Err(format!(
            "the primary header says it lives at LBA {}, not 1",
            primary.my_lba
        ));
    }
    // Read at 4096 bytes the alternate LBA must be this image's last sector.
    // Read at 512 it would point a quarter of the way in, which is the mistake
    // IMG-011 exists to catch.
    let last_4kn = u64::try_from(bytes.len() / 4096).map_err(|_| "image too large")? - 1;
    if primary.alternate_lba != last_4kn {
        return Err(format!(
            "the alternate LBA is {}, not the last 4096-byte sector ({last_4kn})",
            primary.alternate_lba
        ));
    }
    // The partitions get the same scrutiny the 512-byte claim gives its own.
    // They did not, and the asymmetry was the defect: the 4Kn fixture accepted
    // the 512-byte arm's literal LBAs — addressing 8 MiB to 33 MiB on a 4 MiB
    // device — which is precisely the fixture becoming "a 512-byte table with
    // different numbers", the one thing its rationale says it is not.
    expect_partitions_usable(bytes, &primary, 4096, 2)
}

fn check_mbr_basic(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    expect_boot_signature(bytes)?;
    let entries = mbr_entries(bytes);
    if entries.len() < 2 {
        return Err(format!(
            "{} entries; PART-010 converts between schemes and needs two",
            entries.len()
        ));
    }
    if entries.iter().any(|entry| entry.kind == 0xee) {
        return Err(
            "an entry is type 0xEE, which makes this a protective MBR rather than an \
             MBR-partitioned disk"
                .to_owned(),
        );
    }
    if !entries.iter().any(|entry| entry.active) {
        return Err("no entry is active; the boot flag is part of INV-003".to_owned());
    }
    if bytes[512..520] == *b"EFI PART" {
        return Err("a GPT header follows, so this is not an MBR-only disk".to_owned());
    }
    // The bounds and overlap checks the GPT claim has, which this one lacked.
    // PART-010 converts between the two schemes, and a conversion fixture whose
    // entries overlap each other and run past the end of the device exercises
    // the conversion of nothing.
    let sectors = u32::try_from(bytes.len() / 512).map_err(|_| "image too large")?;
    for (index, entry) in entries.iter().enumerate() {
        let end = entry
            .first_lba
            .checked_add(entry.sectors)
            .ok_or_else(|| format!("entry {index} overflows the LBA space"))?;
        if entry.sectors == 0 {
            return Err(format!("entry {index} declares a type but zero sectors"));
        }
        if entry.first_lba == 0 {
            return Err(format!(
                "entry {index} starts at LBA 0, over the table itself"
            ));
        }
        if end > sectors {
            return Err(format!(
                "entry {index} ends at LBA {end} on a {sectors}-sector image"
            ));
        }
        for other in &entries[index + 1..] {
            let other_end = other.first_lba.saturating_add(other.sectors);
            if entry.first_lba < other_end && other.first_lba < end {
                return Err("two MBR entries overlap, so this is not a valid table".to_owned());
            }
        }
    }
    Ok(())
}

fn check_gpt_invalid_primary(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    if bytes[512..520] != *b"EFI PART" {
        return Err(
            "the primary no longer claims to be a table, which makes this indistinguishable \
             from blank media rather than damaged"
                .to_owned(),
        );
    }
    if gpt_header(bytes, 512).is_some() {
        return Err("the primary header still checksums, so nothing is damaged".to_owned());
    }
    let backup = gpt_header(bytes, bytes.len() - 512)
        .ok_or("the backup does not checksum either, so this is not recoverable")?;
    expect_entry_array(bytes, &backup, 512)?;
    // "Recoverable" has to mean there is something to recover. A backup whose
    // entry array is all zeros checksums perfectly and restores a disk with no
    // partitions at all, which REC-001 would have nothing to do with.
    expect_partitions_usable(bytes, &backup, 512, 2)
}

fn check_gpt_conflicting(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    let primary = gpt_header(bytes, 512)
        .ok_or("the primary must checksum, or this is damaged rather than ambiguous")?;
    let backup = gpt_header(bytes, bytes.len() - 512)
        .ok_or("the backup must checksum, or this is damaged rather than ambiguous")?;
    // Each copy must be independently *trustworthy*, not merely present: a
    // header that verifies while its entry array does not is a table that looks
    // authoritative and is not.
    expect_entry_array(bytes, &primary, 512)?;
    expect_entry_array(bytes, &backup, 512)?;
    // Both copies must be usable tables, or a reader would simply prefer the
    // one that parses and nothing would be ambiguous.
    expect_partitions_usable(bytes, &primary, 512, 1)?;
    expect_partitions_usable(bytes, &backup, 512, 1)?;

    // The disagreement must be in the *extents*, not merely in the CRC.
    //
    // Checking `entry_array_crc` inequality alone was not enough, and an
    // adversarial pass demonstrated it: changing one character of a partition
    // *name* in the backup copy makes the two CRCs differ while both tables
    // describe byte-identical extents. A reader gets the same layout whichever
    // copy it trusts, so everything is determinable — and the fixture would
    // still have been reported as ADR-C3 `Indeterminate`, which is the one
    // thing this fixture exists to be.
    let primary_extents = extents(&gpt_partitions(bytes, &primary, 512));
    let backup_extents = extents(&gpt_partitions(bytes, &backup, 512));
    if primary_extents == backup_extents {
        return Err(format!(
            "both copies describe the same extents ({primary_extents:?}), so a reader gets one \
             answer whichever it trusts and the table is determinable"
        ));
    }
    if primary.disk_guid != backup.disk_guid {
        return Err(
            "the copies name different disks, which is two disks rather than one disk \
             described twice"
                .to_owned(),
        );
    }
    Ok(())
}

fn check_gpt_missing_backup(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    let primary = gpt_header(bytes, 512).ok_or("the primary must still checksum")?;
    expect_entry_array(bytes, &primary, 512)?;
    expect_partitions_usable(bytes, &primary, 512, 2)?;
    // The *whole* backup copy, not just its header sector. Checking only the
    // last 512 bytes let 16 KiB of byte-identical entry array survive at LBAs
    // 8159 to 8190 — a backup any scanning recovery tool would find, on a
    // fixture whose property is "no backup at all".
    let array_sectors = 128 * 128 / 512;
    let backup_start = bytes.len() - (array_sectors + 1) * 512;
    if let Some(at) = bytes[backup_start..].iter().position(|byte| *byte != 0) {
        return Err(format!(
            "byte {} of the backup region is non-zero, so a backup survives",
            backup_start + at
        ));
    }
    Ok(())
}

fn check_hybrid(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    expect_boot_signature(bytes)?;
    let primary = gpt_header(bytes, 512).ok_or("the GPT half must be valid, or this is an MBR")?;
    expect_entry_array(bytes, &primary, 512)?;
    let entries = mbr_entries(bytes);
    if !entries.iter().any(|entry| entry.kind == 0xee) {
        return Err("no 0xEE entry, so the disk does not also claim to be GPT".to_owned());
    }
    // The hybrid property: an ordinary MBR entry covering the exact extent a
    // GPT partition claims. Without one the two schemes describe different
    // things and cannot conflict, which is what makes a hybrid disk ambiguous
    // rather than merely unusual.
    let partitions = gpt_partitions(bytes, &primary, 512);
    let aliased = entries
        .iter()
        .filter(|entry| entry.kind != 0xee)
        .any(|entry| {
            partitions.iter().any(|partition| {
                let first = u64::from(entry.first_lba);
                let last = first + u64::from(entry.sectors) - 1;
                first == partition.first_lba && last == partition.last_lba
            })
        });
    if !aliased {
        return Err(
            "no non-protective MBR entry aliases a GPT partition, so the two schemes describe \
             different things and cannot conflict"
                .to_owned(),
        );
    }
    Ok(())
}

fn check_apm(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    if bytes[0..2] != [0x45, 0x52] {
        return Err("no 'ER' driver descriptor at block 0".to_owned());
    }
    let block_size = be_u16(bytes, 2);
    if block_size != 512 {
        return Err(format!("block size reads {block_size} big-endian, not 512"));
    }
    // There is deliberately no "and it reads wrong little-endian" check here.
    // For any non-palindromic value that is implied by the big-endian check
    // above and cannot fail, and an adversarial pass correctly called the
    // earlier version of it unreachable dead code. The endianness property *is*
    // the byte layout asserted throughout this function; a reader that assumes
    // little-endian gets 2 for the block size, 0x00200000 for the block count,
    // and 33 554 432 for the entry count.
    if be_u32(bytes, 4) != 8192 {
        return Err("the block count is not the device's 8192 blocks, big-endian".to_owned());
    }

    // Every map entry, not only the first. The second one carried the extents
    // and the type string that make this an Apple Partition Map rather than two
    // magic numbers, and nothing read it.
    let expected = [
        ("Apple", "Apple_partition_map", 1_u32, 63_u32),
        ("Untitled", "Apple_HFS", 64, 8000),
    ];
    let sectors = u32::try_from(bytes.len() / 512).map_err(|_| "image too large")?;
    for (index, (name, kind, start, length)) in expected.into_iter().enumerate() {
        let base = 512 * (index + 1);
        if bytes[base..base + 2] != [0x50, 0x4d] {
            return Err(format!("no 'PM' map entry at block {}", index + 1));
        }
        let count = be_u32(bytes, base + 4);
        if count != 2 {
            return Err(format!(
                "map entry {index} declares {count} entries in the map, not 2"
            ));
        }
        let actual_start = be_u32(bytes, base + 8);
        let actual_length = be_u32(bytes, base + 12);
        if actual_start != start || actual_length != length {
            return Err(format!(
                "map entry {index} spans {actual_start}..+{actual_length} big-endian, not \
                 {start}..+{length}"
            ));
        }
        if actual_start == 0 || actual_start.saturating_add(actual_length) > sectors {
            return Err(format!(
                "map entry {index} runs outside the {sectors}-block device"
            ));
        }
        if ascii_field(bytes, base + 16, 32) != name {
            return Err(format!("map entry {index} is not named {name:?}"));
        }
        if ascii_field(bytes, base + 48, 32) != kind {
            return Err(format!("map entry {index} is not of type {kind:?}"));
        }
    }
    Ok(())
}

fn check_luks2(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    if bytes[0..6] != [0x4c, 0x55, 0x4b, 0x53, 0xba, 0xbe] {
        return Err("no LUKS magic at offset 0".to_owned());
    }
    let version = be_u16(bytes, 6);
    if version != 2 {
        return Err(format!(
            "version reads {version} big-endian; LIN-003 is about LUKS2"
        ));
    }
    // The defect an audit found: `checksum_alg` and the UUID were written
    // inside the 48-byte `label`, leaving the fixture with no UUID at all.
    if bytes[24..72].iter().any(|byte| *byte != 0) {
        return Err(
            "the label field is not empty, which is how the UUID and checksum algorithm came \
             to be written inside it"
                .to_owned(),
        );
    }
    if &bytes[72..78] != b"sha256" {
        return Err("no checksum algorithm at offset 72".to_owned());
    }
    // An identity, not merely something non-blank: this fixture's whole purpose
    // is an encryption-layer node that ADR-C5 can name.
    let uuid = ascii_field(bytes, 168, 40);
    if !is_uuid_text(&uuid) {
        return Err(format!(
            "the UUID field holds {uuid:?}, which is not an identity a node can be named by"
        ));
    }
    // Whole-disk means exactly that: no partition table above it.
    if bytes[510..512] == [0x55, 0xaa] {
        return Err(
            "a boot signature is present, so this is no longer the whole-disk case MODEL-002 \
             permits"
                .to_owned(),
        );
    }
    Ok(())
}

fn check_lvm2(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    if &bytes[512..520] != b"LABELONE" {
        return Err("no LABELONE magic in sector 1".to_owned());
    }
    // Writing LABELONE alone is detected as nothing at all.
    if &bytes[536..544] != b"LVM2 001" {
        return Err("no LVM2 type string, without which nothing detects this".to_owned());
    }
    // `sector_xl` says which sector the label believes it is in, and `libblkid`
    // skips a label whose answer disagrees with where it found it. It sits at
    // bytes 520..528 — *outside* the CRC span — so a wrong value changes
    // neither the stored nor the recomputed checksum, and an adversarial pass
    // confirmed the fixture stayed green while becoming undetectable.
    let sector_xl = le_u64(bytes, 520);
    if sector_xl != 1 {
        return Err(format!(
            "the label says it lives in sector {sector_xl}, but it was written to sector 1; \
             `libblkid` skips a label that disagrees with where it is"
        ));
    }
    let stored = le_u32(bytes, 528);
    let computed = lvm2_crc_bitwise(&bytes[532..1024]);
    if stored != computed {
        return Err(format!(
            "label checksum is {stored:#010x} but the bytes give {computed:#010x}; `libblkid` \
             verifies this before reporting LVM2_member"
        ));
    }
    // `offset_xl` is the pointer a prober follows to reach the PV header. The
    // UUID must be read from where the label says it is, not from a constant —
    // reading it at a hard-coded 544 silently assumed the value being verified.
    let offset_xl = le_u32(bytes, 532);
    if offset_xl != 32 {
        return Err(format!(
            "the PV header offset is {offset_xl}, not the 32 this label's layout uses"
        ));
    }
    let uuid_at = 512 + usize::try_from(offset_xl).map_err(|_| "offset does not fit")?;
    // LVM PV UUIDs are 32 characters of a base-57 alphabet. Requiring the exact
    // length rather than merely "not blank" is what makes this an identity.
    let uuid = ascii_field(bytes, uuid_at, 32);
    if uuid.len() != 32 || !uuid.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "the PV UUID at the label's own offset holds {uuid:?}; ADR-C5's member node has no \
             identity without one"
        ));
    }
    Ok(())
}

fn check_mdraid_12(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;
    expect_mdraid_12_at(bytes, 4096)?;
    // "Also an orphaned member, since no array assembles from one fixture."
    // That is only true if the superblock says the array needs more members
    // than this fixture provides. A one-disk array would assemble.
    let raid_disks = le_u32(bytes, 4096 + 92);
    if raid_disks < 2 {
        return Err(format!(
            "the superblock declares {raid_disks} raid_disks, so this member would assemble an \
             array by itself and is not the orphan the rationale claims"
        ));
    }
    Ok(())
}

fn check_ext4_with_stale_mdraid(bytes: &[u8]) -> Result<(), String> {
    expect_length(bytes, FIXTURE_BYTES)?;

    // Half one: a live file system at the start — and specifically an *ext4*
    // one. `signature.rs` says it outright: "libblkid separates ext2, ext3, and
    // ext4 by [the feature flags], so a superblock carrying only the magic is
    // detected as the wrong file system." Nothing here read them, and an
    // adversarial pass zeroed both words to leave an ext2 inside a fixture
    // named for ext4, green.
    if le_u16(bytes, 0x0438) != 0xef53 {
        return Err("no ext4 superblock magic at 0x438".to_owned());
    }
    let compat = le_u32(bytes, 1024 + 92);
    let incompat = le_u32(bytes, 1024 + 96);
    if incompat & INCOMPAT_EXTENTS == 0 || compat & COMPAT_HAS_JOURNAL == 0 {
        return Err(format!(
            "feature flags are compat={compat:#x} incompat={incompat:#x}; without EXTENTS and \
             HAS_JOURNAL a prober names this ext2, not the live ext4 the rationale claims"
        ));
    }
    // The block count only means anything alongside the block size it is
    // counted in. Reading the count while assuming the shift is the same class
    // of defect as the one already fixed here — a sector count used as a block
    // count — and it was reintroduced by not reading `s_log_block_size` at all.
    let shift = le_u32(bytes, 1024 + 24);
    if shift != 0 {
        return Err(format!(
            "s_log_block_size is {shift}, so blocks are {} bytes and the count below would be \
             read in the wrong unit",
            1024_u64 << shift
        ));
    }
    let declared_blocks = le_u32(bytes, 1024 + 4);
    let device_kib = u32::try_from(bytes.len() / 1024).map_err(|_| "image too large")?;
    if declared_blocks != device_kib {
        return Err(format!(
            "the file system declares {declared_blocks} blocks of 1 KiB on a {device_kib} KiB \
             device"
        ));
    }
    if bytes[1024 + 104..1024 + 120].iter().all(|byte| *byte == 0) {
        return Err("the file system has no UUID, so its node cannot be named".to_owned());
    }

    // Half two: an array membership near the end, which is the only reason this
    // fixture answers anything. Formatting the start of a device never reaches
    // a 0.90 superblock, and that is what makes a stale signature possible.
    let offset = mdraid_090_offset(bytes.len());
    if offset <= bytes.len() / 2 {
        return Err(format!(
            "the 0.90 superblock at {offset:#x} is not past the midpoint, so start-of-device \
             formatting would reach it"
        ));
    }
    if !offset.is_multiple_of(64 * 1024) {
        return Err(format!("{offset:#x} is not 64 KiB aligned"));
    }
    expect_mdraid_090_at(bytes, offset)
}

fn expect_length(bytes: &[u8], expected: usize) -> Result<(), String> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(format!("image is {} bytes, not {expected}", bytes.len()))
    }
}

fn expect_boot_signature(bytes: &[u8]) -> Result<(), String> {
    if bytes[510..512] == [0x55, 0xaa] {
        Ok(())
    } else {
        Err("no 0x55AA boot signature at byte 510".to_owned())
    }
}

fn expect_protective_mbr(bytes: &[u8]) -> Result<(), String> {
    expect_boot_signature(bytes)?;
    if bytes[450] == 0xee {
        Ok(())
    } else {
        Err(format!(
            "first MBR entry is type {:#04x}, not the 0xEE a GPT disk requires",
            bytes[450]
        ))
    }
}

/// The fields of a GPT header, read back only if its own CRC verifies.
struct GptHeader {
    my_lba: u64,
    alternate_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    entry_lba: u64,
    entry_count: u32,
    entry_size: u32,
    entry_array_crc: u32,
    disk_guid: [u8; 16],
}

/// Parse a GPT header at `offset`, returning `None` unless it checksums.
fn gpt_header(bytes: &[u8], offset: usize) -> Option<GptHeader> {
    if offset + 92 > bytes.len() || bytes[offset..offset + 8] != *b"EFI PART" {
        return None;
    }
    let stored = le_u32(bytes, offset + 16);
    let mut header = bytes[offset..offset + 92].to_vec();
    header[16..20].copy_from_slice(&0_u32.to_le_bytes());
    if crc32_table(&header) != stored {
        return None;
    }
    let mut disk_guid = [0_u8; 16];
    disk_guid.copy_from_slice(&bytes[offset + 56..offset + 72]);
    Some(GptHeader {
        my_lba: le_u64(bytes, offset + 24),
        alternate_lba: le_u64(bytes, offset + 32),
        first_usable_lba: le_u64(bytes, offset + 40),
        last_usable_lba: le_u64(bytes, offset + 48),
        entry_lba: le_u64(bytes, offset + 72),
        entry_count: le_u32(bytes, offset + 80),
        entry_size: le_u32(bytes, offset + 84),
        entry_array_crc: le_u32(bytes, offset + 88),
        disk_guid,
    })
}

/// Confirm the entry array a header points at is the one it checksums.
///
/// A header that verifies while its entry array does not is a table that looks
/// authoritative and is not, so this is checked wherever a header is read. Every
/// bound comes from the header itself, so all of the arithmetic is checked.
fn expect_entry_array(bytes: &[u8], header: &GptHeader, sector: u64) -> Result<(), String> {
    let start = header
        .entry_lba
        .checked_mul(sector)
        .and_then(|offset| usize::try_from(offset).ok())
        .ok_or("the entry-array LBA does not address this image")?;
    let length = u64::from(header.entry_count)
        .checked_mul(u64::from(header.entry_size))
        .and_then(|size| usize::try_from(size).ok())
        .ok_or("the declared entry-array size does not fit this image")?;
    let end = start
        .checked_add(length)
        .ok_or("the entry array's extent overflows")?;
    if end > bytes.len() {
        return Err(format!(
            "the entry array at {start:#x} runs past the end of the image"
        ));
    }
    let actual = crc32_table(&bytes[start..end]);
    if actual == header.entry_array_crc {
        Ok(())
    } else {
        Err(format!(
            "the header at LBA {} checksums but its entry array does not: {:#010x} declared, \
             {actual:#010x} computed",
            header.my_lba, header.entry_array_crc
        ))
    }
}

/// A GPT partition entry that is actually populated.
struct GptPartitionEntry {
    first_lba: u64,
    last_lba: u64,
}

/// The extents a table describes, which is what two copies must differ in for
/// the table to be genuinely ambiguous.
fn extents(partitions: &[GptPartitionEntry]) -> Vec<(u64, u64)> {
    partitions
        .iter()
        .map(|partition| (partition.first_lba, partition.last_lba))
        .collect()
}

/// Read the populated entries of the array a header points at.
fn gpt_partitions(bytes: &[u8], header: &GptHeader, sector: u64) -> Vec<GptPartitionEntry> {
    let mut partitions = Vec::new();
    let Some(start) = header
        .entry_lba
        .checked_mul(sector)
        .and_then(|offset| usize::try_from(offset).ok())
    else {
        return partitions;
    };
    let Ok(size) = usize::try_from(header.entry_size) else {
        return partitions;
    };
    for index in 0..usize::try_from(header.entry_count).unwrap_or(0) {
        let Some(base) = index.checked_mul(size).and_then(|at| start.checked_add(at)) else {
            break;
        };
        if base + 56 > bytes.len() {
            break;
        }
        // An all-zero type GUID marks an unused entry.
        if bytes[base..base + 16].iter().all(|byte| *byte == 0) {
            continue;
        }
        partitions.push(GptPartitionEntry {
            first_lba: le_u64(bytes, base + 32),
            last_lba: le_u64(bytes, base + 40),
        });
    }
    partitions
}

/// One MBR primary entry, as read back from the table.
struct MbrEntry {
    active: bool,
    kind: u8,
    first_lba: u32,
    sectors: u32,
}

/// Read the four primary entries, skipping empty ones.
fn mbr_entries(bytes: &[u8]) -> Vec<MbrEntry> {
    let mut entries = Vec::new();
    for index in 0..4 {
        let base = 446 + index * 16;
        let kind = bytes[base + 4];
        let sectors = le_u32(bytes, base + 12);
        if kind == 0 || sectors == 0 {
            continue;
        }
        entries.push(MbrEntry {
            active: bytes[base] == 0x80,
            kind,
            first_lba: le_u32(bytes, base + 8),
            sectors,
        });
    }
    entries
}

/// Confirm a validated mdraid 1.2 superblock at `offset`.
fn expect_mdraid_12_at(bytes: &[u8], offset: usize) -> Result<(), String> {
    if le_u32(bytes, offset) != 0xa92b_4efc {
        return Err(format!("no mdraid magic at {offset:#x}"));
    }
    let major = le_u32(bytes, offset + 4);
    if major != 1 {
        return Err(format!("major version {major}, not 1"));
    }
    // `super_offset` must agree with where the superblock actually is, or it
    // describes a location it is not in. Multiplied with a check, not plainly:
    // an adversarial pass showed a plain `* 512` overflows and panics in debug
    // on a hostile value, and this runs inside `generate`, where a panic and a
    // refusal are very different failures.
    let declared = le_u64(bytes, offset + 144)
        .checked_mul(512)
        .ok_or("the declared superblock offset overflows the address space")?;
    let actual = u64::try_from(offset).map_err(|_| "offset does not fit")?;
    if declared != actual {
        return Err(format!(
            "the superblock says it lives at {declared:#x} but sits at {offset:#x}"
        ));
    }
    if bytes[offset + 16..offset + 32]
        .iter()
        .all(|byte| *byte == 0)
    {
        return Err("the array UUID is entirely zero".to_owned());
    }
    // The geometry, which was otherwise entirely unconstrained: a member could
    // declare its data region starting past the end of the device it is written
    // to, which is not a member of anything.
    let total = u64::try_from(bytes.len()).map_err(|_| "image too large")?;
    let data_end = le_u64(bytes, offset + 128)
        .checked_add(le_u64(bytes, offset + 136))
        .and_then(|sectors| sectors.checked_mul(512))
        .ok_or("the declared data region overflows")?;
    if data_end > total {
        return Err(format!(
            "the member's data region ends at {data_end:#x} on a {total:#x}-byte device"
        ));
    }
    if ascii_field(bytes, offset + 32, 32).is_empty() {
        return Err("the superblock has no set name, which is what a prober reports".to_owned());
    }
    // Without a valid checksum `wipefs` still lists it, because it enumerates
    // magic matches, while `blkid -p` reports nothing, because it validates.
    let mut superblock = [0_u8; 256];
    superblock.copy_from_slice(&bytes[offset..offset + 256]);
    let stored = le_u32(bytes, offset + 216);
    superblock[216..220].copy_from_slice(&0_u32.to_le_bytes());
    let computed = folded_word_sum(&superblock);
    if stored != computed {
        return Err(format!(
            "superblock checksum is {stored:#010x} but the bytes give {computed:#010x}; \
             `blkid -p` validates this and would report nothing"
        ));
    }
    Ok(())
}

/// Confirm a validated mdraid 0.90 superblock at `offset`.
fn expect_mdraid_090_at(bytes: &[u8], offset: usize) -> Result<(), String> {
    if offset + 4096 > bytes.len() {
        return Err(format!(
            "a 0.90 superblock at {offset:#x} runs past the end"
        ));
    }
    if le_u32(bytes, offset) != 0xa92b_4efc {
        return Err(format!("no mdraid magic at {offset:#x}"));
    }
    if le_u32(bytes, offset + 8) != 90 {
        return Err("the minor version is not 90, so this is not a legacy superblock".to_owned());
    }
    // The defect an audit found: the set UUID is split across four
    // *non-adjacent* words — 5, then 13, 14 and 15. Writing the last three at
    // words 32 to 34 left the identity three-quarters zero, and `blkid` showed
    // `…-0000-0000-0000-000000000000`.
    for (index, at) in [offset + 20, offset + 52, offset + 56, offset + 60]
        .into_iter()
        .enumerate()
    {
        if bytes[at..at + 4].iter().all(|byte| *byte == 0) {
            return Err(format!(
                "word {index} of the set UUID is zero; the array identity must not be \
                 partially blank"
            ));
        }
    }
    let mut superblock = [0_u8; 4096];
    superblock.copy_from_slice(&bytes[offset..offset + 4096]);
    let stored = le_u32(bytes, offset + 108);
    superblock[108..112].copy_from_slice(&0_u32.to_le_bytes());
    let computed = folded_word_sum(&superblock);
    if stored != computed {
        return Err(format!(
            "0.90 checksum is {stored:#010x} but the bytes give {computed:#010x}"
        ));
    }
    Ok(())
}

/// Where a 0.90 superblock lives on a device of this size, by the kernel's
/// formula. Recomputed here rather than imported, since it is part of the claim.
fn mdraid_090_offset(length: usize) -> usize {
    let kib = length / 1024;
    ((kib - 64) & !63) * 1024
}

/// Is this text a canonical 8-4-4-4-12 UUID?
fn is_uuid_text(text: &str) -> bool {
    let groups: Vec<&str> = text.split('-').collect();
    if groups.len() != 5 {
        return false;
    }
    if [8, 4, 4, 4, 12] != groups.iter().map(|g| g.len()).collect::<Vec<_>>()[..] {
        return false;
    }
    // All-zero is well-formed and is not an identity.
    groups
        .iter()
        .all(|group| group.chars().all(|c| c.is_ascii_hexdigit()))
        && text.chars().any(|c| c != '0' && c != '-')
}

/// CRC-32 (IEEE, reflected) by table lookup.
///
/// [`crate::layout::crc32`] iterates bits. This one builds a byte table, so a
/// defect in either does not reproduce in the other, and both are pinned to the
/// published check value `0xcbf43926` for `"123456789"`.
fn crc32_table(data: &[u8]) -> u32 {
    let table = crc32_lookup();
    let mut crc = 0xffff_ffff_u32;
    for byte in data {
        let index = usize::try_from((crc ^ u32::from(*byte)) & 0xff).expect("a byte fits usize");
        crc = (crc >> 8) ^ table[index];
    }
    !crc
}

fn crc32_lookup() -> [u32; 256] {
    let mut table = [0_u32; 256];
    for (index, slot) in table.iter_mut().enumerate() {
        let mut value = u32::try_from(index).expect("an index below 256 fits u32");
        for _ in 0..8 {
            value = if value & 1 == 1 {
                (value >> 1) ^ 0xedb8_8320
            } else {
                value >> 1
            };
        }
        *slot = value;
    }
    table
}

/// LVM2's CRC, a bit at a time.
///
/// [`crate::signature::lvm2_crc`] consumes a nibble at a time through a
/// 16-entry table. Two nibble steps are eight bit steps over the same
/// polynomial, so these must agree — and `libblkid` verifies the result before
/// it will report `LVM2_member`, which anchors the pair outside this repository.
fn lvm2_crc_bitwise(data: &[u8]) -> u32 {
    let mut crc = 0xf597_a6cf_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc
}

/// mdraid's checksum: little-endian 32-bit words summed, then folded to 32 bits.
///
/// Folded iteratively here, where the writer folds once. Anchored by `blkid -p`,
/// which validates this and reports nothing when it is wrong.
fn folded_word_sum(bytes: &[u8]) -> u32 {
    let mut total = 0_u64;
    for word in bytes.chunks_exact(4) {
        total += u64::from(u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
    }
    while total > u64::from(u32::MAX) {
        total = (total & 0xffff_ffff) + (total >> 32);
    }
    u32::try_from(total).unwrap_or(u32::MAX)
}

/// Read a NUL-padded ASCII field, trimming the padding.
fn ascii_field(bytes: &[u8], offset: usize, length: usize) -> String {
    let field = &bytes[offset..offset + length];
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(length);
    String::from_utf8_lossy(&field[..end]).into_owned()
}

fn le_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn be_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([bytes[at], bytes[at + 1]])
}

fn le_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn be_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn le_u64(bytes: &[u8], at: usize) -> u64 {
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(raw)
}

#[cfg(test)]
mod tests;
