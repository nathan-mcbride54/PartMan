//! Pure, bounded classification of partition-table bytes into ADR-C3's
//! three states — the raw-sector contract ADR-0014 names, and nothing
//! wider.
//!
//! The axis decision (`docs/adr/0014-si35-table-state-axis.md`) makes the
//! privileged helper the sole author of partition-table state, computed
//! from its own parser over raw sectors, because the measurement campaign
//! established that nothing else separates the decisive fixtures — not
//! any client projection on any platform, and not the privileged
//! `blkid`/`wipefs` probes either. This crate is that parser. It is
//! **pure over caller-supplied windows**: the first and last stretch of a
//! medium plus its geometry, the exact shape M10 measured as separating.
//! It performs no I/O, launches nothing, opens nothing, and holds no
//! Section 5 domain type — a [`Classification`] of bytes is upstream
//! evidence, not an inventory node, snapshot, identity record, or plan.
//!
//! **Refusal over generality**, the recorded house pattern: the caller's
//! contract violations (impossible geometry, undersized or oversized
//! windows) refuse with a typed [`ParseRefusal`]; everything a hostile
//! *medium* can do lands in the classification itself, fail-closed — a
//! header whose entry array cannot be verified within the supplied window
//! is an **invalid copy**, never a trusted one, and a scheme that is
//! claimed but unreadable is [`TableState::Indeterminate`], never
//! guessed.
//!
//! This is a parser of on-disk metadata under Section 11.4: `unsafe` is
//! denied by the workspace lint, and its fuzz target lands in `fuzz/`
//! beside the codec and plist targets.
//!
//! The `Present` checksum is computed over **copy-invariant content** —
//! never raw header sectors, whose copy-position fields differ between
//! the two GPT copies by design. The byte recipe is stated at
//! [`gpt_content_checksum`] and lands normatively in
//! `schemas/table-checksum.md` with the SI-35 resolution's specification
//! change.

use sha2::{Digest as _, Sha256};

/// The largest window this parser will consider, per window. Real callers
/// supply 64 KiB; a mebibyte is headroom, not generality.
pub const WINDOW_LIMIT: usize = 1024 * 1024;

/// The most partition entries a GPT header may declare before the copy is
/// treated as unverifiable. UEFI's floor is 128; four times that is
/// generous, and a header demanding more is asking this parser to walk
/// megabytes on its say-so.
pub const GPT_ENTRY_COUNT_LIMIT: u32 = 512;

/// The most Apple Partition Map entries accepted, per the format's own
/// practical bound (the map must fit before the first partition).
pub const APM_ENTRY_LIMIT: u32 = 63;

/// Why the *caller's* input was refused. A refusal is about the call, not
/// the medium: hostile media classify, they do not error.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseRefusal {
    /// The sector size is not one this parser's offsets are defined for.
    SectorSizeUnsupported {
        /// The size the caller stated.
        stated: u32,
    },
    /// A window is not a whole number of sectors.
    WindowNotSectorMultiple,
    /// A window exceeds [`WINDOW_LIMIT`].
    WindowOverLimit,
    /// The head window is too small to contain LBA 0 and LBA 1.
    HeadWindowTooSmall,
    /// The tail window is too small to contain the last sector.
    TailWindowTooSmall,
    /// The stated medium cannot hold the structures the windows imply.
    GeometryImpossible,
}

impl ParseRefusal {
    /// One human-actionable sentence.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::SectorSizeUnsupported { stated } => {
                format!("sector size {stated} is not 512 or 4096; refused rather than guessed")
            }
            Self::WindowNotSectorMultiple => {
                "a window is not a whole number of sectors; refused".to_owned()
            }
            Self::WindowOverLimit => {
                format!("a window exceeds {WINDOW_LIMIT} bytes; refused rather than truncated")
            }
            Self::HeadWindowTooSmall => {
                "the head window does not cover LBA 0 and LBA 1; refused".to_owned()
            }
            Self::TailWindowTooSmall => {
                "the tail window does not cover the last sector; refused".to_owned()
            }
            Self::GeometryImpossible => {
                "the stated geometry cannot hold the supplied windows; refused".to_owned()
            }
        }
    }
}

/// The medium's stated shape. The caller asserts it; the parser's answers
/// are relative to it, which is what keeps the 4Kn trap honest — probing
/// a 4Kn medium with 512-byte geometry answers for that contract, exactly
/// as libblkid's file probe does, and the answer is `Indeterminate`
/// rather than a fabricated table.
#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    /// Logical sector size in bytes: 512 or 4096.
    pub sector_size: u32,
    /// Total sectors on the medium.
    pub total_sectors: u64,
}

/// ADR-C3's three states, exactly, with `Present` carrying the checksum
/// its definition ("read and hashed") requires.
///
/// This enum is deliberately closed and carries **no proceed-enabling
/// reading**: no `Default`, no `is_safe`, no ordering. A consumer must
/// match all three arms, and what a write path may do with each is the
/// specification's to say (SAFE-005 disables affected writes on the
/// conditions below; PART-001's categorical invariant arrives with the
/// SI-35 resolution's spec change), never this crate's.
#[derive(Debug, PartialEq, Eq)]
pub enum TableState {
    /// A table was read and its copy-invariant content hashed.
    Present {
        /// SHA-256 over the scheme's copy-invariant content; see
        /// [`gpt_content_checksum`] for the GPT recipe.
        checksum: [u8; 32],
    },
    /// Positively observed to have no table: every location where a
    /// supported scheme lives was examined and none claims one. Never a
    /// statement about *data* — an absent table does not mean an absent
    /// file system, and the signature facts are another surface's to
    /// report.
    Absent,
    /// A table is claimed and cannot be trusted, with the basis stated.
    Indeterminate {
        /// Which arm of "unreadable or ambiguous" fired.
        basis: IndeterminateBasis,
    },
}

/// The two arms of ADR-C3's `Indeterminate`, kept distinct because their
/// remedies differ: ambiguity needs an authority decision, unreadability
/// needs recovery.
#[derive(Debug, PartialEq, Eq)]
pub enum IndeterminateBasis {
    /// Two independently valid copies describe different contents, and
    /// nothing in the format names a winner.
    Ambiguous,
    /// A table is claimed — by a protective MBR, or by a copy's own magic
    /// — and no copy verifies.
    Unreadable,
}

/// A condition detected beside the state — body facts, not extra states.
/// Each is a SAFE-005 hook ("corrupt metadata" disables the affected
/// writes) without collapsing a determinable table into `Indeterminate`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Condition {
    /// The primary GPT copy claims a table and does not verify; the
    /// backup carried the content.
    PrimaryInvalid,
    /// The backup GPT copy claims a table and does not verify.
    BackupInvalid,
    /// No backup GPT copy exists where the format requires one.
    BackupMissing,
    /// The MBR beside a valid GPT carries non-protective entries: one
    /// disk described twice. Detection only (INV-003); what a plan may do
    /// under it is PART-014/SI-11 material, deliberately not decided
    /// here.
    HybridMbr,
}

/// Which scheme the state was derived from, when one was claimed.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Scheme {
    /// GUID Partition Table.
    Gpt,
    /// Master Boot Record, standalone.
    Mbr,
    /// Apple Partition Map.
    Apm,
}

/// One classification: the state, the scheme it came from (absent tables
/// have none), and the detected conditions, in a fixed order.
#[derive(Debug, PartialEq, Eq)]
pub struct Classification {
    /// ADR-C3's state.
    pub state: TableState,
    /// The scheme examined, where any was claimed.
    pub scheme: Option<Scheme>,
    /// Detected conditions, ordered as declared in [`Condition`].
    pub conditions: Vec<Condition>,
}

/// One parsed-and-verified GPT copy's content, copy-position fields
/// excluded.
struct GptContent {
    disk_guid: [u8; 16],
    first_usable: u64,
    last_usable: u64,
    entry_count: u32,
    entry_size: u32,
    entries: Vec<u8>,
}

impl GptContent {
    fn agrees_with(&self, other: &Self) -> bool {
        self.disk_guid == other.disk_guid
            && self.first_usable == other.first_usable
            && self.last_usable == other.last_usable
            && self.entry_count == other.entry_count
            && self.entry_size == other.entry_size
            && self.entries == other.entries
    }
}

/// The GPT `Present` checksum recipe, stated once: SHA-256 over the
/// copy-invariant content, in this exact order and encoding —
/// `DiskGUID` (16 raw bytes) ∥ `FirstUsableLBA` (8 bytes little-endian) ∥
/// `LastUsableLBA` (8 LE) ∥ `NumberOfPartitionEntries` (4 LE) ∥
/// `SizeOfPartitionEntry` (4 LE) ∥ the partition entry array bytes.
/// Copy-position fields (`MyLBA`, `AlternateLBA`, `PartitionEntryLBA`,
/// both CRCs) are excluded because the two copies differ in them by
/// design; two agreeing copies therefore produce one checksum, from
/// either copy.
fn gpt_content_checksum(content: &GptContent) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content.disk_guid);
    hasher.update(content.first_usable.to_le_bytes());
    hasher.update(content.last_usable.to_le_bytes());
    hasher.update(content.entry_count.to_le_bytes());
    hasher.update(content.entry_size.to_le_bytes());
    hasher.update(&content.entries);
    hasher.finalize().into()
}

/// Classify one medium from its head and tail windows.
///
/// # Errors
///
/// Refuses only the caller's contract violations; see [`ParseRefusal`].
/// Everything a medium's bytes can do is a [`Classification`].
pub fn classify(
    head: &[u8],
    tail: &[u8],
    geometry: Geometry,
) -> Result<Classification, ParseRefusal> {
    let sector = validated_sector(geometry.sector_size)?;
    if head.len() > WINDOW_LIMIT || tail.len() > WINDOW_LIMIT {
        return Err(ParseRefusal::WindowOverLimit);
    }
    if !head.len().is_multiple_of(sector) || !tail.len().is_multiple_of(sector) {
        return Err(ParseRefusal::WindowNotSectorMultiple);
    }
    if head.len() < sector * 2 {
        return Err(ParseRefusal::HeadWindowTooSmall);
    }
    if tail.is_empty() {
        return Err(ParseRefusal::TailWindowTooSmall);
    }
    let head_sectors = (head.len() / sector) as u64;
    let tail_sectors = (tail.len() / sector) as u64;
    if geometry.total_sectors < head_sectors
        || geometry.total_sectors < tail_sectors
        || geometry.total_sectors < 4
    {
        return Err(ParseRefusal::GeometryImpossible);
    }

    let mbr = read_mbr(head, sector);
    let gpt_claimed_by_magic =
        claims_gpt(head, sector, 1) || claims_gpt(tail, sector, tail_sectors.saturating_sub(1));
    let gpt_claimed = mbr.protective || mbr.hybrid || gpt_claimed_by_magic;

    if gpt_claimed {
        return Ok(classify_gpt(
            head,
            tail,
            sector,
            tail_sectors,
            geometry,
            &mbr,
        ));
    }
    if mbr.standalone {
        return Ok(Classification {
            state: TableState::Present {
                checksum: mbr_checksum(head),
            },
            scheme: Some(Scheme::Mbr),
            conditions: Vec::new(),
        });
    }
    if let Some(state) = classify_apm(head, sector) {
        return Ok(Classification {
            state,
            scheme: Some(Scheme::Apm),
            conditions: Vec::new(),
        });
    }
    // Every location where a supported scheme lives was examined above
    // and none claims a table: ADR-C3's positive absence. Nothing here is
    // a statement about data — signature reporting is FS-004's surface.
    Ok(Classification {
        state: TableState::Absent,
        scheme: None,
        conditions: Vec::new(),
    })
}

fn validated_sector(stated: u32) -> Result<usize, ParseRefusal> {
    match stated {
        512 | 4096 => Ok(stated as usize),
        _ => Err(ParseRefusal::SectorSizeUnsupported { stated }),
    }
}

/// What LBA 0's MBR area asserts.
struct MbrReading {
    /// A lone 0xEE entry: the disk claims a GPT.
    protective: bool,
    /// 0xEE beside real entries: one disk described twice.
    hybrid: bool,
    /// Real entries, no 0xEE: a standalone MBR.
    standalone: bool,
}

fn read_mbr(head: &[u8], sector: usize) -> MbrReading {
    // The boot signature lives at bytes 510..512 regardless of sector
    // size; a 4Kn medium still keeps the MBR structure in the first 512
    // bytes of LBA 0.
    let none = MbrReading {
        protective: false,
        hybrid: false,
        standalone: false,
    };
    if sector < 512 || head.len() < 512 || head[510..512] != [0x55, 0xaa] {
        return none;
    }
    let mut has_ee = false;
    let mut has_other = false;
    for entry in 0..4 {
        let kind = head[446 + entry * 16 + 4];
        match kind {
            0x00 => {}
            0xee => has_ee = true,
            _ => has_other = true,
        }
    }
    MbrReading {
        protective: has_ee && !has_other,
        hybrid: has_ee && has_other,
        // A 0x55AA sector with four zeroed entries is indistinguishable
        // from a file-system boot sector and is not read as a table
        // claim; a fixture must exist before that judgement is revisited.
        standalone: !has_ee && has_other,
    }
}

/// Whether a window's sector at `lba_in_window` claims the GPT magic.
fn claims_gpt(window: &[u8], sector: usize, lba_in_window: u64) -> bool {
    let Ok(lba) = usize::try_from(lba_in_window) else {
        return false;
    };
    let offset = lba.saturating_mul(sector);
    window.len() >= offset + 8 && window[offset..offset + 8] == *b"EFI PART"
}

fn classify_gpt(
    head: &[u8],
    tail: &[u8],
    sector: usize,
    tail_sectors: u64,
    geometry: Geometry,
    mbr: &MbrReading,
) -> Classification {
    let tail_start_lba = geometry.total_sectors - tail_sectors;
    let backup_lba_in_tail = tail_sectors - 1;
    let backup_claims = claims_gpt(tail, sector, backup_lba_in_tail);

    let primary = verified_copy(head, sector, 0, 1, geometry);
    let backup = verified_copy(
        tail,
        sector,
        tail_start_lba,
        geometry.total_sectors - 1,
        geometry,
    );

    let mut conditions = Vec::new();
    if mbr.hybrid {
        conditions.push(Condition::HybridMbr);
    }

    let state = match (primary, backup) {
        (Some(primary), Some(backup)) => {
            if primary.agrees_with(&backup) {
                TableState::Present {
                    checksum: gpt_content_checksum(&primary),
                }
            } else {
                // Two independently valid authorities, different content,
                // and the format names no winner: ADR-C3's ambiguous arm.
                conditions.clear();
                return Classification {
                    state: TableState::Indeterminate {
                        basis: IndeterminateBasis::Ambiguous,
                    },
                    scheme: Some(Scheme::Gpt),
                    conditions: hybrid_only(mbr),
                };
            }
        }
        (None, Some(backup)) => {
            conditions.push(Condition::PrimaryInvalid);
            TableState::Present {
                checksum: gpt_content_checksum(&backup),
            }
        }
        (Some(primary), None) => {
            conditions.push(if backup_claims {
                Condition::BackupInvalid
            } else {
                Condition::BackupMissing
            });
            TableState::Present {
                checksum: gpt_content_checksum(&primary),
            }
        }
        (None, None) => {
            // Claimed — by the protective MBR, a hybrid's 0xEE entry, or
            // a copy's surviving magic — and no copy verifies. The
            // unreadable arm, and deliberately not Absent: a disk that
            // asserts a table nobody can read must never look blank.
            return Classification {
                state: TableState::Indeterminate {
                    basis: IndeterminateBasis::Unreadable,
                },
                scheme: Some(Scheme::Gpt),
                conditions: hybrid_only(mbr),
            };
        }
    };

    Classification {
        state,
        scheme: Some(Scheme::Gpt),
        conditions,
    }
}

fn hybrid_only(mbr: &MbrReading) -> Vec<Condition> {
    if mbr.hybrid {
        vec![Condition::HybridMbr]
    } else {
        Vec::new()
    }
}

/// Parse and fully verify one GPT copy inside one window, or decline.
///
/// Every failure — bad magic, bad size, bad CRC, wrong self-position, an
/// entry array outside the window or over the bound, an entry-array CRC
/// mismatch — returns `None`: an unverifiable copy is an invalid copy,
/// never a trusted one.
fn verified_copy(
    window: &[u8],
    sector: usize,
    window_start_lba: u64,
    header_lba: u64,
    geometry: Geometry,
) -> Option<GptContent> {
    let header_offset = usize::try_from(header_lba.checked_sub(window_start_lba)?).ok()? * sector;
    let header = window.get(header_offset..header_offset + 92)?;
    if &header[0..8] != b"EFI PART" {
        return None;
    }
    let header_size = u32::from_le_bytes(header[12..16].try_into().ok()?) as usize;
    if !(92..=sector).contains(&header_size) {
        return None;
    }
    let full_header = window.get(header_offset..header_offset + header_size)?;
    let declared_crc = u32::from_le_bytes(header[16..20].try_into().ok()?);
    let mut zeroed = full_header.to_vec();
    zeroed[16..20].fill(0);
    if crc32(&zeroed) != declared_crc {
        return None;
    }
    let my_lba = u64::from_le_bytes(header[24..32].try_into().ok()?);
    if my_lba != header_lba {
        return None;
    }
    let entry_count = u32::from_le_bytes(header[80..84].try_into().ok()?);
    let entry_size = u32::from_le_bytes(header[84..88].try_into().ok()?);
    if entry_count == 0
        || entry_count > GPT_ENTRY_COUNT_LIMIT
        || !(128..=4096).contains(&entry_size)
        || !entry_size.is_multiple_of(128)
    {
        return None;
    }
    let entries_lba = u64::from_le_bytes(header[72..80].try_into().ok()?);
    let entries_offset = usize::try_from(entries_lba.checked_sub(window_start_lba)?)
        .ok()?
        .checked_mul(sector)?;
    let entries_len = (entry_count as usize).checked_mul(entry_size as usize)?;
    let entries = window.get(entries_offset..entries_offset.checked_add(entries_len)?)?;
    let declared_entries_crc = u32::from_le_bytes(header[88..92].try_into().ok()?);
    if crc32(entries) != declared_entries_crc {
        return None;
    }
    if entries_lba >= geometry.total_sectors || my_lba >= geometry.total_sectors {
        return None;
    }
    Some(GptContent {
        disk_guid: header[56..72].try_into().ok()?,
        first_usable: u64::from_le_bytes(header[40..48].try_into().ok()?),
        last_usable: u64::from_le_bytes(header[48..56].try_into().ok()?),
        entry_count,
        entry_size,
        entries: entries.to_vec(),
    })
}

/// The MBR `Present` checksum: SHA-256 over bytes 440..510 of LBA 0 —
/// the disk signature, the reserved pair, and the four entries — the
/// content a standalone MBR asserts, boot code excluded.
fn mbr_checksum(head: &[u8]) -> [u8; 32] {
    Sha256::digest(&head[440..510]).into()
}

/// Recognize and verify an Apple Partition Map, or decline.
fn classify_apm(head: &[u8], sector: usize) -> Option<TableState> {
    let first = head.get(sector..sector + 8)?;
    if first[0..2] != *b"PM" {
        return None;
    }
    let map_entries = u32::from_be_bytes(first[4..8].try_into().ok()?);
    if map_entries == 0 || map_entries > APM_ENTRY_LIMIT {
        // Claimed and not credible: unreadable, never absent.
        return Some(TableState::Indeterminate {
            basis: IndeterminateBasis::Unreadable,
        });
    }
    let map_len = (map_entries as usize).checked_mul(sector)?;
    let map = head.get(sector..sector.checked_add(map_len)?);
    let Some(map) = map else {
        return Some(TableState::Indeterminate {
            basis: IndeterminateBasis::Unreadable,
        });
    };
    for entry in 0..map_entries as usize {
        let offset = entry * sector;
        if map[offset..offset + 2] != *b"PM" {
            return Some(TableState::Indeterminate {
                basis: IndeterminateBasis::Unreadable,
            });
        }
        let declared = u32::from_be_bytes(map[offset + 4..offset + 8].try_into().ok()?);
        if declared != map_entries {
            return Some(TableState::Indeterminate {
                basis: IndeterminateBasis::Unreadable,
            });
        }
    }
    // The APM `Present` checksum: SHA-256 over the map's sectors — the
    // format keeps no redundant copy, so the map is the content.
    Some(TableState::Present {
        checksum: Sha256::digest(map).into(),
    })
}

/// CRC-32 (IEEE), bitwise. The fixtures crate iterates bits and its
/// evidence layer uses a lookup table; a third in-crate spelling keeps
/// this parser free of dev-only dependencies while staying comparable
/// against both in tests.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let low = crc & 1;
            crc >>= 1;
            if low == 1 {
                crc ^= 0xedb8_8320;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests;
