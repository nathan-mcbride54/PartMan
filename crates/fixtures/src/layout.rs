//! Byte-level synthesis of partition-table layouts.
//!
//! Everything here is a pure function of its arguments. No clock, no random
//! source, no environment: two runs on two machines produce identical bytes, so
//! a fixture can be pinned by digest and regenerated rather than committed
//! (Section 11.3, Section 16).

use sha2::{Digest as _, Sha256};

/// Bytes in a fixture's logical sector.
///
/// Only the two sizes the product must handle are offered. IMG-011 makes the
/// 512-versus-4096 distinction a first-class concern, so a fixture declares it
/// rather than assuming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorSize {
    /// 512-byte logical sectors, including 512e media.
    B512,
    /// 4096-byte logical sectors (4Kn).
    B4096,
}

impl SectorSize {
    /// The size in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        match self {
            Self::B512 => 512,
            Self::B4096 => 4096,
        }
    }

    fn usize_bytes(self) -> usize {
        // Both variants are small constants, so this cannot truncate on any
        // target this workspace supports (WP-010 records the 64-bit `usize`
        // assumption).
        usize::try_from(self.bytes()).expect("a sector size fits in usize")
    }
}

/// A disk image under construction.
///
/// The whole image is held in memory. Fixtures are deliberately small — a
/// parser needs a valid table, not a realistic capacity — so this stays well
/// inside a test process's budget and keeps the writer honest about size.
#[derive(Clone, Debug)]
pub struct Image {
    bytes: Vec<u8>,
    sector: SectorSize,
}

impl Image {
    /// Create an all-zero image of `sectors` logical sectors.
    ///
    /// # Panics
    ///
    /// Panics if the requested image does not fit in memory on this target.
    #[must_use]
    pub fn blank(sector: SectorSize, sectors: u64) -> Self {
        let total = sector
            .bytes()
            .checked_mul(sectors)
            .expect("fixture size overflows u64");
        let total = usize::try_from(total).expect("fixture size fits in usize");
        Self {
            bytes: vec![0; total],
            sector,
        }
    }

    /// The image's logical sector size.
    #[must_use]
    pub const fn sector(&self) -> SectorSize {
        self.sector
    }

    /// The number of logical sectors.
    ///
    /// # Panics
    ///
    /// Panics if the image length does not fit in `u64`, which no target this
    /// workspace supports can produce.
    #[must_use]
    pub fn sectors(&self) -> u64 {
        u64::try_from(self.bytes.len()).expect("image length fits in u64") / self.sector.bytes()
    }

    /// The finished bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Borrow the bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Byte offset of a logical sector.
    fn offset(&self, lba: u64) -> usize {
        let offset = lba
            .checked_mul(self.sector.bytes())
            .expect("LBA offset overflows u64");
        usize::try_from(offset).expect("LBA offset fits in usize")
    }

    /// Write `data` at a byte offset inside logical sector `lba`.
    ///
    /// # Panics
    ///
    /// Panics if the write would run past the end of the image. A fixture that
    /// silently truncated its own table would be worse than useless.
    pub fn write_at(&mut self, lba: u64, offset_in_sector: usize, data: &[u8]) {
        let start = self.offset(lba) + offset_in_sector;
        let end = start
            .checked_add(data.len())
            .expect("fixture write overflows usize");
        assert!(
            end <= self.bytes.len(),
            "fixture write past end of image: {end} > {}",
            self.bytes.len()
        );
        self.bytes[start..end].copy_from_slice(data);
    }

    /// Overwrite one logical sector with zeros.
    pub fn zero_sector(&mut self, lba: u64) {
        let zeros = vec![0_u8; self.sector.usize_bytes()];
        self.write_at(lba, 0, &zeros);
    }

    /// Read a slice for checksum recomputation inside this module's tests.
    fn read(&self, lba: u64, offset_in_sector: usize, length: usize) -> &[u8] {
        let start = self.offset(lba) + offset_in_sector;
        &self.bytes[start..start + length]
    }
}

/// A globally unique identifier, stored in the mixed-endian form GPT uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guid([u8; 16]);

impl Guid {
    /// Build a GUID from the canonical textual field order.
    #[must_use]
    pub const fn from_fields(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Self {
        let a = d1.to_le_bytes();
        let b = d2.to_le_bytes();
        let c = d3.to_le_bytes();
        Self([
            a[0], a[1], a[2], a[3], b[0], b[1], c[0], c[1], d4[0], d4[1], d4[2], d4[3], d4[4],
            d4[5], d4[6], d4[7],
        ])
    }

    /// Derive a stable GUID from a label.
    ///
    /// Deterministic by construction: the same label always yields the same
    /// GUID, on every machine and every run. That is what lets a fixture's
    /// digest be pinned. The version and variant bits are set so the result is
    /// a well-formed RFC 4122 version-4 GUID rather than 16 arbitrary bytes,
    /// because a parser under test is entitled to reject a malformed one.
    #[must_use]
    pub fn derived(label: &str) -> Self {
        let digest = Sha256::digest(label.as_bytes());
        let mut raw = [0_u8; 16];
        raw.copy_from_slice(&digest[..16]);
        raw[7] = (raw[7] & 0x0f) | 0x40;
        raw[8] = (raw[8] & 0x3f) | 0x80;
        Self(raw)
    }

    /// The raw 16 bytes, ready to write.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// EFI System partition type.
pub const EFI_SYSTEM: Guid = Guid::from_fields(
    0xc12a_7328,
    0xf81f,
    0x11d2,
    [0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e, 0xc9, 0x3b],
);

/// Linux filesystem data partition type.
pub const LINUX_FILESYSTEM: Guid = Guid::from_fields(
    0x0fc6_3daf,
    0x8483,
    0x4772,
    [0x8e, 0x79, 0x3d, 0x69, 0xd8, 0x47, 0x7d, 0xe4],
);

/// Microsoft basic data partition type.
pub const MICROSOFT_BASIC_DATA: Guid = Guid::from_fields(
    0xebd0_a0a2,
    0xb9e5,
    0x4433,
    [0x87, 0xc0, 0x68, 0xb6, 0xb7, 0x26, 0x99, 0xc7],
);

/// One GPT partition to synthesize.
#[derive(Clone, Debug)]
pub struct GptPartition {
    /// Partition type GUID.
    pub type_guid: Guid,
    /// Unique partition GUID.
    pub unique_guid: Guid,
    /// First logical sector, inclusive.
    pub first_lba: u64,
    /// Last logical sector, inclusive.
    pub last_lba: u64,
    /// Human-readable name, encoded UTF-16LE into the 72-byte field.
    pub name: &'static str,
}

/// Number of partition entries a conforming GPT reserves.
const GPT_ENTRY_COUNT: u32 = 128;
/// Size of one GPT partition entry.
const GPT_ENTRY_SIZE: u32 = 128;
/// Bytes of the GPT header that participate in its CRC.
const GPT_HEADER_SIZE: u32 = 92;

/// CRC-32 as GPT uses it: IEEE 802.3, reflected, initial and final inversion.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

/// Write a protective MBR at LBA 0, as a GPT disk requires.
fn write_protective_mbr(image: &mut Image) {
    let sectors = image.sectors();
    let protective = u32::try_from(sectors - 1).unwrap_or(u32::MAX);
    let mut entry = [0_u8; 16];
    entry[0] = 0x00; // not bootable
    entry[1..4].copy_from_slice(&[0x00, 0x02, 0x00]); // CHS of LBA 1
    entry[4] = 0xee; // GPT protective
    entry[5..8].copy_from_slice(&[0xff, 0xff, 0xff]); // CHS beyond addressing
    entry[8..12].copy_from_slice(&1_u32.to_le_bytes());
    entry[12..16].copy_from_slice(&protective.to_le_bytes());

    image.write_at(0, 446, &entry);
    image.write_at(0, 510, &[0x55, 0xaa]);
}

/// Encode the partition entry array.
fn gpt_entry_array(partitions: &[GptPartition]) -> Vec<u8> {
    let size = usize::try_from(GPT_ENTRY_COUNT * GPT_ENTRY_SIZE).expect("entry array fits usize");
    let mut array = vec![0_u8; size];
    for (index, partition) in partitions.iter().enumerate() {
        let base = index * usize::try_from(GPT_ENTRY_SIZE).expect("entry size fits usize");
        array[base..base + 16].copy_from_slice(partition.type_guid.as_bytes());
        array[base + 16..base + 32].copy_from_slice(partition.unique_guid.as_bytes());
        array[base + 32..base + 40].copy_from_slice(&partition.first_lba.to_le_bytes());
        array[base + 40..base + 48].copy_from_slice(&partition.last_lba.to_le_bytes());
        array[base + 48..base + 56].copy_from_slice(&0_u64.to_le_bytes());
        for (offset, unit) in partition.name.encode_utf16().take(36).enumerate() {
            let at = base + 56 + offset * 2;
            array[at..at + 2].copy_from_slice(&unit.to_le_bytes());
        }
    }
    array
}

/// Encode one GPT header, with both CRC fields computed.
fn gpt_header(
    my_lba: u64,
    alternate_lba: u64,
    first_usable: u64,
    last_usable: u64,
    entry_lba: u64,
    disk_guid: Guid,
    entry_array_crc: u32,
) -> Vec<u8> {
    let mut header = vec![0_u8; usize::try_from(GPT_HEADER_SIZE).expect("header fits usize")];
    header[0..8].copy_from_slice(b"EFI PART");
    header[8..12].copy_from_slice(&0x0001_0000_u32.to_le_bytes());
    header[12..16].copy_from_slice(&GPT_HEADER_SIZE.to_le_bytes());
    // header[16..20] is the header CRC, left zero while it is computed.
    header[24..32].copy_from_slice(&my_lba.to_le_bytes());
    header[32..40].copy_from_slice(&alternate_lba.to_le_bytes());
    header[40..48].copy_from_slice(&first_usable.to_le_bytes());
    header[48..56].copy_from_slice(&last_usable.to_le_bytes());
    header[56..72].copy_from_slice(disk_guid.as_bytes());
    header[72..80].copy_from_slice(&entry_lba.to_le_bytes());
    header[80..84].copy_from_slice(&GPT_ENTRY_COUNT.to_le_bytes());
    header[84..88].copy_from_slice(&GPT_ENTRY_SIZE.to_le_bytes());
    header[88..92].copy_from_slice(&entry_array_crc.to_le_bytes());

    let crc = crc32(&header);
    header[16..20].copy_from_slice(&crc.to_le_bytes());
    header
}

/// Sectors occupied by the 128-entry partition array at this sector size.
fn entry_array_sectors(sector: SectorSize) -> u64 {
    let bytes = u64::from(GPT_ENTRY_COUNT) * u64::from(GPT_ENTRY_SIZE);
    bytes.div_ceil(sector.bytes())
}

/// Build a complete, valid GPT image.
///
/// # Panics
///
/// Panics if the image is too small to hold both copies of the table.
#[must_use]
pub fn gpt(sector: SectorSize, sectors: u64, label: &str, partitions: &[GptPartition]) -> Image {
    let mut image = Image::blank(sector, sectors);
    let array_sectors = entry_array_sectors(sector);
    assert!(
        sectors > 2 * array_sectors + 3,
        "image too small for a GPT with both copies"
    );

    let last = sectors - 1;
    let first_usable = 2 + array_sectors;
    let last_usable = last - array_sectors - 1;
    let backup_entry_lba = last - array_sectors;

    write_protective_mbr(&mut image);

    let array = gpt_entry_array(partitions);
    let array_crc = crc32(&array);
    let disk_guid = Guid::derived(&format!("{label}/disk"));

    image.write_at(2, 0, &array);
    image.write_at(backup_entry_lba, 0, &array);

    let primary = gpt_header(1, last, first_usable, last_usable, 2, disk_guid, array_crc);
    let backup = gpt_header(
        last,
        1,
        first_usable,
        last_usable,
        backup_entry_lba,
        disk_guid,
        array_crc,
    );
    image.write_at(1, 0, &primary);
    image.write_at(last, 0, &backup);

    image
}

/// One MBR primary partition.
#[derive(Clone, Copy, Debug)]
pub struct MbrPartition {
    /// Partition type byte, for example `0x83` for Linux or `0x07` for NTFS.
    pub kind: u8,
    /// Whether the active (boot) flag is set.
    pub active: bool,
    /// First logical sector.
    pub first_lba: u32,
    /// Length in logical sectors.
    pub sectors: u32,
}

/// Build an MBR image with up to four primary partitions.
///
/// # Panics
///
/// Panics if more than four partitions are supplied.
#[must_use]
pub fn mbr(sector: SectorSize, sectors: u64, partitions: &[MbrPartition]) -> Image {
    assert!(partitions.len() <= 4, "an MBR holds four primary entries");
    let mut image = Image::blank(sector, sectors);
    write_mbr_table(&mut image, partitions);
    image
}

/// Overwrite LBA 0 with an MBR table that is not merely protective.
///
/// This is what makes a hybrid disk hybrid: the same extents are described by
/// both an MBR and a GPT, and the two descriptions can disagree. INV-003
/// requires detecting it rather than picking whichever table is read first.
///
/// # Panics
///
/// Panics if more than four partitions are supplied.
pub fn write_hybrid_mbr(image: &mut Image, partitions: &[MbrPartition]) {
    assert!(partitions.len() <= 4, "an MBR holds four primary entries");
    write_mbr_table(image, partitions);
}

/// Write the four-entry table and boot signature into LBA 0.
fn write_mbr_table(image: &mut Image, partitions: &[MbrPartition]) {
    for (index, partition) in partitions.iter().enumerate() {
        let mut entry = [0_u8; 16];
        entry[0] = if partition.active { 0x80 } else { 0x00 };
        entry[1..4].copy_from_slice(&[0xfe, 0xff, 0xff]);
        entry[4] = partition.kind;
        entry[5..8].copy_from_slice(&[0xfe, 0xff, 0xff]);
        entry[8..12].copy_from_slice(&partition.first_lba.to_le_bytes());
        entry[12..16].copy_from_slice(&partition.sectors.to_le_bytes());
        image.write_at(0, 446 + index * 16, &entry);
    }
    image.write_at(0, 510, &[0x55, 0xaa]);
}

/// Build an Apple Partition Map image (INV-003).
///
/// All APM fields are big-endian, which is the point of including it: a parser
/// that assumes little-endian everywhere passes every other fixture here.
///
/// # Panics
///
/// Panics if the image cannot hold the map.
#[must_use]
pub fn apm(sector: SectorSize, sectors: u64, entries: &[(&str, &str, u32, u32)]) -> Image {
    let mut image = Image::blank(sector, sectors);
    let count = u32::try_from(entries.len()).expect("APM entry count fits u32");

    // Block 0: Driver Descriptor Record.
    let mut ddr = [0_u8; 20];
    ddr[0..2].copy_from_slice(&0x4552_u16.to_be_bytes()); // 'ER'
    let block_size = u16::try_from(sector.bytes()).unwrap_or(u16::MAX);
    ddr[2..4].copy_from_slice(&block_size.to_be_bytes());
    let block_count = u32::try_from(sectors).unwrap_or(u32::MAX);
    ddr[4..8].copy_from_slice(&block_count.to_be_bytes());
    image.write_at(0, 0, &ddr);

    // Blocks 1..=count: partition map entries.
    for (index, (name, kind, start, length)) in entries.iter().enumerate() {
        let lba = 1 + u64::try_from(index).expect("APM index fits u64");
        let mut entry = [0_u8; 136];
        entry[0..2].copy_from_slice(&0x504d_u16.to_be_bytes()); // 'PM'
        entry[4..8].copy_from_slice(&count.to_be_bytes());
        entry[8..12].copy_from_slice(&start.to_be_bytes());
        entry[12..16].copy_from_slice(&length.to_be_bytes());
        write_fixed_ascii(&mut entry[16..48], name);
        write_fixed_ascii(&mut entry[48..80], kind);
        image.write_at(lba, 0, &entry);
    }

    image
}

/// Copy `text` into a fixed-width, NUL-padded ASCII field.
fn write_fixed_ascii(field: &mut [u8], text: &str) {
    for (slot, byte) in field.iter_mut().zip(text.bytes()) {
        *slot = byte;
    }
}

/// Replace the backup table with an independently valid one that disagrees.
///
/// This is what ADR-C3 means by a table that "parses ambiguously", and it is a
/// different thing from a damaged one. Both copies checksum correctly and each
/// is internally consistent, so a reader has two trustworthy descriptions of the
/// same disk and no ground for preferring either. Nothing can be positively
/// determined, which is exactly the `Indeterminate` state.
///
/// Contrast [`corrupt_primary_header_crc`], which damages one copy and leaves
/// the other authoritative — recoverable, not indeterminate.
///
/// # Panics
///
/// Panics if the image is too small to hold both copies of the table.
pub fn write_conflicting_backup(image: &mut Image, label: &str, partitions: &[GptPartition]) {
    let sectors = image.sectors();
    let array_sectors = entry_array_sectors(image.sector());
    assert!(
        sectors > 2 * array_sectors + 3,
        "image too small for a GPT with both copies"
    );

    let last = sectors - 1;
    let first_usable = 2 + array_sectors;
    let last_usable = last - array_sectors - 1;
    let backup_entry_lba = last - array_sectors;

    let array = gpt_entry_array(partitions);
    let array_crc = crc32(&array);
    // The same disk GUID: two tables for one disk, not two disks.
    let disk_guid = Guid::derived(&format!("{label}/disk"));

    image.write_at(backup_entry_lba, 0, &array);
    let backup = gpt_header(
        last,
        1,
        first_usable,
        last_usable,
        backup_entry_lba,
        disk_guid,
        array_crc,
    );
    image.write_at(last, 0, &backup);
}

/// Corrupt a GPT primary header's CRC without disturbing its signature.
///
/// The result is **damaged but recoverable**, not ADR-C3 `Indeterminate`: the
/// backup header is untouched and still authoritative, so the table remains
/// positively determinable. This doc comment claimed the opposite until an
/// audit noticed it contradicted [`write_conflicting_backup`] three functions
/// below, which had already been corrected. The signature surviving while the
/// checksum does not is what keeps this distinct from a blank device — that
/// much was always true, and it is the part ADR-C3 forbids conflating.
pub fn corrupt_primary_header_crc(image: &mut Image) {
    let mut crc = [0_u8; 4];
    crc.copy_from_slice(image.read(1, 16, 4));
    crc[0] ^= 0xff;
    image.write_at(1, 16, &crc);
}

#[cfg(test)]
mod tests;
