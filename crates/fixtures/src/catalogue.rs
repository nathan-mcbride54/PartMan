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

/// A valid primary GPT with its backup header erased.
fn gpt_missing_backup() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    let last = image.sectors() - 1;
    image.zero_sector(last);
    image
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
fn gpt_basic(sector: SectorSize, sectors: u64) -> Image {
    let label = "gpt-basic";
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
    let existing = root.join(MANIFEST_FILE).is_file();
    fs::create_dir_all(root)?;

    // Remove anything this build does not produce, so a fixture withdrawn from
    // the catalogue does not linger and get consumed by a later test that
    // enumerates the directory instead of the manifest. Withdrawing the ZFS
    // fixture left exactly such a file behind.
    //
    // Only ever prune a directory that already holds one of our manifests, or
    // one we just created. Otherwise a mistyped root would delete a user's
    // files, and this function is not worth that risk.
    if existing {
        let expected_names: Vec<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
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

    let mut images = Vec::new();
    for fixture in catalogue() {
        let bytes = (fixture.build)().into_bytes();
        fs::write(root.join(fixture.name), &bytes)?;
        images.push((fixture.name.to_owned(), bytes));
    }

    let manifest = Manifest::build(&images);
    fs::write(root.join(MANIFEST_FILE), manifest.render())?;
    Ok(manifest)
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
