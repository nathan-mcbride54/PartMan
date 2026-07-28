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
            name: "gpt-corrupt-header-512.img",
            rationale: "ADR-C3 Indeterminate: a device that plainly claims a table whose table \
                        cannot be trusted. SAFE-005 must fail closed here, and this must never be \
                        confused with blank media",
            build: gpt_corrupt_header,
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
    ]
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

/// A valid GPT whose primary header no longer checksums.
fn gpt_corrupt_header() -> Image {
    let mut image = gpt_basic(SectorSize::B512, SECTORS_4MIB_512);
    layout::corrupt_primary_header_crc(&mut image);
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
    fs::create_dir_all(root)?;

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
