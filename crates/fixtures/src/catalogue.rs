//! The fixture set, and writing it to disk.
//!
//! Each entry exists because a requirement needs it, and the reason is recorded
//! next to it. A fixture nobody can name a requirement for is a fixture nobody
//! will maintain.

use std::fs;
use std::io;
use std::path::Path;

use crate::layout::{
    self, EFI_SYSTEM, GptPartition, Guid, Image, LINUX_FILESYSTEM, MICROSOFT_BASIC_DATA,
    MbrPartition, SectorSize,
};
use crate::manifest::{MANIFEST_FILE, Manifest};
use crate::signature;

/// Four mebibytes of 512-byte sectors.
const SECTORS_4MIB_512: u64 = 8192;
/// Four mebibytes of 4096-byte sectors.
const SECTORS_4MIB_4KN: u64 = 1024;

/// A named fixture and why it exists.
pub struct Fixture {
    /// File name, without a directory.
    pub name: &'static str,
    /// The requirement this fixture serves.
    pub rationale: &'static str,
    /// Produce the bytes.
    pub build: fn() -> Image,
}

/// Every fixture this repository generates.
#[must_use]
pub fn catalogue() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "blank-512.img",
            rationale: "PART-001 initializes blank media; ADR-C3 makes a positively observed \
                        absent table a determined state, distinct from an unreadable one",
            build: || Image::blank(SectorSize::B512, SECTORS_4MIB_512),
        },
        Fixture {
            name: "gpt-basic-512.img",
            rationale: "INV-003 GPT detection, and the ordinary case every other test compares to",
            build: || gpt_basic(SectorSize::B512, SECTORS_4MIB_512),
        },
        Fixture {
            name: "gpt-basic-4kn.img",
            rationale: "IMG-011 cross-sector-size work: a 4Kn table is not a 512-byte table with \
                        different numbers, and a parser that assumes 512 passes every other fixture",
            build: || gpt_basic(SectorSize::B4096, SECTORS_4MIB_4KN),
        },
        Fixture {
            name: "mbr-basic-512.img",
            rationale: "INV-003 MBR detection; PART-010 converts between the two schemes",
            build: mbr_basic,
        },
        Fixture {
            name: "gpt-invalid-primary-valid-backup-512.img",
            rationale: "INV-003 damaged metadata and REC-001 restore: the primary header no longer \
                        checksums while the backup does, so the table remains positively \
                        determinable. Recoverable, and NOT ADR-C3 Indeterminate — an earlier \
                        version of this catalogue claimed it was",
            build: gpt_invalid_primary,
        },
        Fixture {
            name: "gpt-conflicting-tables-512.img",
            rationale: "ADR-C3 Indeterminate, properly: primary and backup are each independently \
                        valid and describe different partitions, so the table parses ambiguously \
                        and nothing about it can be positively determined. SAFE-005 must fail \
                        closed, and this must never be confused with blank media",
            build: gpt_conflicting_tables,
        },
        Fixture {
            name: "gpt-missing-backup-512.img",
            rationale: "INV-003 inconsistent tables: a valid primary with no backup, which \
                        PART-013 backup and REC-001 restore both have to reason about",
            build: gpt_missing_backup,
        },
        Fixture {
            name: "gpt-both-copies-invalid-512.img",
            rationale: "ADR-C3 Indeterminate on the unreadable arm: both copies still claim \
                        to be tables and neither checksums, while the protective MBR keeps \
                        asserting a GPT exists. The SI-35 resolution round found this row of \
                        the classification untestable without a fixture — added so the \
                        both-copies-invalid case is measured, never prose",
            build: gpt_both_copies_invalid,
        },
        Fixture {
            name: "hybrid-mbr-gpt-512.img",
            rationale: "INV-003 hybrid tables: one disk described twice, by two schemes that can \
                        disagree. SI-27 records this as a node-naming collision family",
            build: hybrid_mbr_gpt,
        },
        Fixture {
            name: "apm-basic-512.img",
            rationale: "INV-003 Apple Partition Map. Every field is big-endian, so a parser that \
                        assumes little-endian everywhere passes every other fixture here",
            build: apm_basic,
        },
        Fixture {
            name: "luks2-whole-disk-512.img",
            rationale: "FS-004 LUKS detection and LIN-003 LUKS2 support, whole-disk: MODEL-002 \
                        permits an encryption layer with no intervening partition table",
            build: luks2_whole_disk,
        },
        Fixture {
            name: "lvm2-pv-orphan-512.img",
            rationale: "FS-004 LVM PV detection, and ADR-C5's `consumer: Null`: a member whose \
                        aggregate is not observed must be represented, not discarded (INV-008)",
            build: lvm2_pv_orphan,
        },
        Fixture {
            name: "mdraid-1.2-member-512.img",
            rationale: "FS-004 Linux RAID detection and LIN-005 mdraid support; also an orphaned \
                        member, since no array assembles from one fixture",
            build: mdraid_12_member,
        },
        Fixture {
            name: "ext4-with-stale-mdraid-090-512.img",
            rationale: "The multi-signature case: a current ext4 and an obsolete 0.90 array \
                        membership on one device. A 0.90 superblock lives near the END, so \
                        formatting the start never removes it. SI-27 files this as a collision \
                        family and round three's narrowing of it rests on this fixture existing",
            build: ext4_with_stale_mdraid,
        },
    ]
}

/// A whole-disk LUKS2 container.
fn luks2_whole_disk() -> Image {
    signature::whole_disk(SECTORS_4MIB_512, |image| {
        signature::luks2(image, "5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f");
    })
}

/// An LVM2 physical volume whose volume group is not observed.
fn lvm2_pv_orphan() -> Image {
    signature::whole_disk(SECTORS_4MIB_512, |image| {
        signature::lvm2_pv(image, "pvuuid000000000000000000000000000");
    })
}

/// An mdraid 1.2 member with no assembled array.
fn mdraid_12_member() -> Image {
    signature::whole_disk(SECTORS_4MIB_512, |image| {
        signature::mdraid_12(image, "md-array-1.2");
    })
}

/// A live ext4 file system over an obsolete 0.90 array membership.
fn ext4_with_stale_mdraid() -> Image {
    signature::whole_disk(SECTORS_4MIB_512, |image| {
        signature::ext4(image, "stale-pair/fs");
        signature::mdraid_090_at_end(image, "stale-pair/array");
    })
}

/// Two primary partitions, one active, in an ordinary MBR.
fn mbr_basic() -> Image {
    layout::mbr(
        SectorSize::B512,
        SECTORS_4MIB_512,
        &[
            MbrPartition {
                kind: 0x0c,
                active: true,
                first_lba: 2048,
                sectors: 2048,
            },
            MbrPartition {
                kind: 0x83,
                active: false,
                first_lba: 4096,
                sectors: 4000,
            },
        ],
    )
}

/// A GPT whose primary header no longer checksums, with an intact backup.
fn gpt_invalid_primary() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    layout::corrupt_primary_header_crc(&mut image);
    image
}

/// A GPT where neither copy checksums: both headers still claim "EFI PART"
/// and the protective MBR still asserts a GPT exists, so the table is
/// unreadable — positively distinct from a disk that never had one.
fn gpt_both_copies_invalid() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    layout::corrupt_primary_header_crc(&mut image);
    layout::corrupt_backup_header_crc(&mut image);
    image
}

/// Two independently valid GPTs on one disk, describing different partitions.
fn gpt_conflicting_tables() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    // A different, equally well-formed partition set in the backup copy. Same
    // disk GUID, so this is one disk described twice rather than two disks.
    layout::write_conflicting_backup(
        &mut image,
        "gpt-basic",
        &[GptPartition {
            type_guid: MICROSOFT_BASIC_DATA,
            unique_guid: Guid::derived("conflicting/only"),
            first_lba: 2048,
            last_lba: 8158,
            name: "Disagreeing",
        }],
    );
    image
}

/// A valid primary GPT with the whole backup copy erased.
///
/// The backup *entry array* is erased along with the header. An earlier version
/// zeroed only the last sector, so 16 KiB of byte-identical entry array survived
/// at LBAs 8159 to 8190 and any recovery tool that scans rather than seeking to
/// the last LBA would have found a backup on a fixture named for having none.
/// An adversarial pass found it; the fixture is now what its name says.
fn gpt_missing_backup() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    let last = image.sectors() - 1;
    for lba in backup_region(&image)..=last {
        image.zero_sector(lba);
    }
    image
}

/// First LBA of the backup GPT copy: the entry array, then the header.
fn backup_region(image: &Image) -> u64 {
    let entry_bytes = 128 * 128;
    let array_sectors = entry_bytes / image.sector().bytes();
    image.sectors() - 1 - array_sectors
}

/// A GPT disk whose MBR describes the same extents a second time.
fn hybrid_mbr_gpt() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    // Replace the protective MBR with one that aliases the first GPT partition,
    // which is what makes a hybrid disk ambiguous rather than merely unusual.
    image.write_at(0, 446, &[0_u8; 64]);
    layout::write_hybrid_mbr(
        &mut image,
        &[
            MbrPartition {
                kind: 0xee,
                active: false,
                first_lba: 1,
                sectors: 2047,
            },
            MbrPartition {
                kind: 0x0c,
                active: false,
                first_lba: 2048,
                sectors: 2048,
            },
        ],
    );
    image
}

/// An Apple Partition Map with a map entry and one HFS partition.
fn apm_basic() -> Image {
    layout::apm(
        SectorSize::B512,
        SECTORS_4MIB_512,
        &[
            ("Apple", "Apple_partition_map", 1, 63),
            ("Untitled", "Apple_HFS", 64, 8000),
        ],
    )
}

/// The fixture set **this build of the binary** produces, computed with no I/O.
///
/// This is the interlock's root of trust, and it is deliberately not read from
/// disk. Generation is deterministic, so the digests here are a pure function of
/// the compiled catalogue: an attacker who can write to the fixture directory
/// cannot change what the interlock expects, only what is on disk to compare
/// against it.
///
/// The earlier design derived expectations from `tests/generated/MANIFEST`,
/// which is a user-writable file in the directory being verified. That reduced
/// the whole check to an assertion — the exact failure this crate's own
/// documentation warns against — and let a hand-written manifest authorize any
/// file at all.
#[must_use]
pub fn expected() -> Manifest {
    let images: Vec<(String, Vec<u8>)> = catalogue()
        .into_iter()
        .map(|fixture| (fixture.name.to_owned(), (fixture.build)().into_bytes()))
        .collect();
    Manifest::build(&images)
}

/// Build the standard two-partition GPT used as the baseline.
///
/// The label differs by sector size, and that is not cosmetic: it is what gives
/// the two images distinct disk GUIDs. Both used `"gpt-basic"` until the
/// catalogue-wide identity check found it, so a 512-byte disk and a 4Kn disk —
/// different media, different partitions — carried one identity. The fixtures
/// *derived* from the 512-byte baseline keep its GUID deliberately, because
/// they are the same disk in different states.
fn gpt_basic(sector: SectorSize, sectors: u64) -> Image {
    let label = match sector {
        SectorSize::B512 => "gpt-basic",
        SectorSize::B4096 => "gpt-basic-4kn",
    };
    let (esp_first, esp_last, data_first, data_last) = match sector {
        SectorSize::B512 => (2048, 4095, 4096, sectors - 34),
        SectorSize::B4096 => (256, 511, 512, sectors - 34),
    };
    layout::gpt(
        sector,
        sectors,
        label,
        &[
            GptPartition {
                type_guid: EFI_SYSTEM,
                unique_guid: Guid::derived("gpt-basic/esp"),
                first_lba: esp_first,
                last_lba: esp_last,
                name: "EFI System",
            },
            GptPartition {
                type_guid: if matches!(sector, SectorSize::B512) {
                    LINUX_FILESYSTEM
                } else {
                    MICROSOFT_BASIC_DATA
                },
                unique_guid: Guid::derived("gpt-basic/data"),
                first_lba: data_first,
                last_lba: data_last,
                name: "Data",
            },
        ],
    )
}

/// Generate every fixture into `root`, replacing whatever was there.
///
/// Returns the manifest, which carries the SAFE-007 disposable-test token.
///
/// # Errors
///
/// Returns any I/O error from creating the directory or writing a file.
pub fn generate(root: &Path) -> io::Result<Manifest> {
    generate_from(root, &catalogue())
}

/// Generate a caller-supplied fixture set.
///
/// Split out from [`generate`] for one reason: the evidence gate below could not
/// be tested otherwise. An adversarial pass deleted the gate and all 74 tests
/// stayed green, because every test fed `generate` the real catalogue — which
/// satisfies its claims — so nothing ever exercised the refusal. A safety check
/// no test can reach is the defect `evidence` exists to end, and it was sitting
/// inside the fix for it.
pub(crate) fn generate_from(root: &Path, fixtures: &[Fixture]) -> io::Result<Manifest> {
    let existing = directory_is_ours(root);
    fs::create_dir_all(root)?;

    // Remove anything this build does not produce, so a fixture withdrawn from
    // the catalogue does not linger and get consumed by a later test that
    // enumerates the directory instead of the manifest. Withdrawing the ZFS
    // fixture left exactly such a file behind.
    //
    // Only ever prune a directory this project can *prove* it owns. The proof
    // used to be `root.join(MANIFEST_FILE).is_file()`, which establishes
    // nothing: any directory holding an unrelated file — or a symlink, since
    // `is_file` follows one — named `MANIFEST` was treated as ours and lost its
    // other regular files. See [`directory_is_ours`].
    if existing {
        let expected_names: Vec<&str> = fixtures.iter().map(|fixture| fixture.name).collect();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name != MANIFEST_FILE && !expected_names.contains(&name) {
                fs::remove_file(entry.path())?;
            }
        }
    }

    // Verify every image *before* writing any of them. Verifying inside the
    // write loop left a half-written directory behind on a refusal, which is a
    // worse state than either outcome: a caller that ignored the error would
    // find a fixture set that looks partial rather than absent.
    let mut images = Vec::new();
    for fixture in fixtures {
        let bytes = (fixture.build)().into_bytes();
        // Refuse an image that no longer supports its own rationale. The
        // rationale beside each entry is a claim, and a claim nothing computes
        // is what let the LUKS2 fixture become 4 MiB of zeros while the
        // traceability record still cited it for FS-004.
        crate::evidence::verify(fixture.name, &bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        images.push((fixture.name.to_owned(), bytes));
    }

    for (name, bytes) in &images {
        fs::write(root.join(name), bytes)?;
    }

    let manifest = Manifest::build(&images);
    fs::write(root.join(MANIFEST_FILE), manifest.render())?;
    Ok(manifest)
}

/// Can this project prove it owns `root`, and may therefore delete from it?
///
/// Ownership is *computed*, not inferred from a filename — the same rule the
/// interlock applies to a destructive target. The directory must hold a regular
/// file named `MANIFEST`, reached without following a symlink, whose contents
/// parse as one of our manifests. `Manifest::parse` recomputes the token from
/// the parsed entries and rejects a mismatch, so a hand-written file cannot
/// claim ownership either.
///
/// Every failure is a refusal to prune. A directory this function cannot prove
/// is ours keeps its files; generation still writes, because writing named files
/// into a root the caller chose is what it was asked to do, but nothing is
/// removed. Deleting on an unproven claim is the failure worth avoiding.
fn directory_is_ours(root: &Path) -> bool {
    let path = root.join(MANIFEST_FILE);
    // `symlink_metadata` does not follow links, so a symlink named `MANIFEST`
    // pointing at some real file elsewhere cannot authorize deletion here.
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    let Ok(text) = fs::read_to_string(&path) else {
        return false;
    };
    Manifest::parse(&text).is_ok()
}

/// Read a manifest previously written by [`generate`].
///
/// # Errors
///
/// Returns an I/O error if the file cannot be read, or a parse error rendered
/// as an [`io::Error`] if it is not a well-formed manifest. Both are refusals
/// as far as the interlock is concerned.
pub fn load_manifest(root: &Path) -> io::Result<Manifest> {
    let text = fs::read_to_string(root.join(MANIFEST_FILE))?;
    Manifest::parse(&text).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests;
