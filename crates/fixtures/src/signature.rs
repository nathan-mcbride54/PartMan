//! On-disk signatures for the structures FS-004 requires be detected.
//!
//! FS-004 lists LVM PV, Linux RAID, LUKS, `BitLocker`, ZFS pool members, Storage
//! Spaces, and LDM metadata alongside file systems, and ADR-C5 resolves that by
//! materializing each as its own node rather than as a file-system kind. Those
//! nodes need fixtures, and a fixture that a real prober does not recognize
//! proves nothing.
//!
//! So every writer here is checked against `libblkid` and `wipefs` rather than
//! against this crate's own reading of a specification. The structures are
//! written far enough to be *detected*, not far enough to be *mounted*: enough
//! magic, version, and identity for a prober to name the format and move on.
//! Where a field is written only to satisfy a prober, the comment says so.

use crate::layout::{Image, SectorSize};

/// Offset of the ext2/3/4 superblock.
const EXT_SUPERBLOCK_OFFSET: u64 = 1024;
/// Offset of an mdraid 1.2 superblock, which sits 4 KiB into the device.
const MDRAID_12_OFFSET: u64 = 4096;
/// Magic shared by every mdraid superblock version.
const MDRAID_MAGIC: u32 = 0xa92b_4efc;

/// Write an ext4 superblock.
///
/// The feature flags matter: `libblkid` separates ext2, ext3, and ext4 by them,
/// so a superblock carrying only the magic is detected as the wrong file system.
pub fn ext4(image: &mut Image, uuid_seed: &str) {
    // Block count is derived from the device rather than hard-coded. The first
    // version of this writer declared 8192 blocks of 1 KiB — 8 MiB — on a 4 MiB
    // image, having reused a sector count as a block count. A file system that
    // claims to be twice the size of its device is not a fixture of anything.
    let device_bytes = image.sectors() * image.sector().bytes();
    let blocks = u32::try_from(device_bytes / 1024).unwrap_or(u32::MAX);

    let mut sb = [0_u8; 264];
    // s_inodes_count, s_blocks_count_lo.
    sb[0..4].copy_from_slice(&1024_u32.to_le_bytes());
    sb[4..8].copy_from_slice(&blocks.to_le_bytes());
    // s_first_data_block. Must be 1 at a 1 KiB block size, not 0: the
    // superblock itself occupies block 0.
    sb[20..24].copy_from_slice(&1_u32.to_le_bytes());
    // s_log_block_size = 0, meaning 1024-byte blocks.
    sb[24..28].copy_from_slice(&0_u32.to_le_bytes());
    // s_blocks_per_group and s_inodes_per_group, so the geometry is at least
    // self-consistent for a reader that looks past the magic.
    sb[32..36].copy_from_slice(&8192_u32.to_le_bytes());
    sb[40..44].copy_from_slice(&256_u32.to_le_bytes());
    // s_magic.
    sb[56..58].copy_from_slice(&0xef53_u16.to_le_bytes());
    // s_state: cleanly unmounted.
    sb[58..60].copy_from_slice(&1_u16.to_le_bytes());
    // s_feature_compat = HAS_JOURNAL, s_feature_incompat = EXTENTS. Together
    // these are what make this ext4 rather than ext2.
    sb[92..96].copy_from_slice(&0x0000_0004_u32.to_le_bytes());
    sb[96..100].copy_from_slice(&0x0000_0040_u32.to_le_bytes());
    // s_uuid.
    sb[104..120].copy_from_slice(crate::layout::Guid::derived(uuid_seed).as_bytes());

    write_bytes(image, EXT_SUPERBLOCK_OFFSET, &sb);
}

/// Write a LUKS2 header (LIN-003, FS-004, REC-011).
pub fn luks2(image: &mut Image, uuid_text: &str) {
    // Field offsets follow `luks2_hdr_disk`. The first version of this writer
    // put `checksum_alg` at 24 and the UUID at 32 — both inside the 48-byte
    // `label` field — so the fixture had no UUID, no checksum algorithm, and a
    // spurious label, on a fixture whose purpose is an encryption-layer node
    // identity.
    let mut header = [0_u8; 208];
    header[0..6].copy_from_slice(&[0x4c, 0x55, 0x4b, 0x53, 0xba, 0xbe]);
    // Version is big-endian, unlike almost everything else in this module.
    header[6..8].copy_from_slice(&2_u16.to_be_bytes());
    // hdr_size, big-endian: the 16 KiB default metadata area.
    header[8..16].copy_from_slice(&16384_u64.to_be_bytes());
    // seqid.
    header[16..24].copy_from_slice(&1_u64.to_be_bytes());
    // label occupies 24..72 and is deliberately left empty.
    // checksum_alg, 32 bytes of NUL-padded ASCII at 72.
    header[72..78].copy_from_slice(b"sha256");
    // salt occupies 104..168.
    // uuid, 40 bytes of NUL-padded ASCII at 168.
    let uuid = uuid_text.as_bytes();
    let len = uuid.len().min(40);
    header[168..168 + len].copy_from_slice(&uuid[..len]);

    write_bytes(image, 0, &header);
}

/// Write an LVM2 physical-volume label (LIN-004, FS-004).
///
/// The label lives in one of the first four sectors; sector 1 is conventional.
/// `libblkid` requires both `LABELONE` and the `LVM2 001` type string, which is
/// why writing the magic alone detects nothing.
pub fn lvm2_pv(image: &mut Image, pv_uuid: &str) {
    let sector = image.sector().bytes();
    let label_offset = sector; // sector 1

    // The whole 512-byte label sector is built first, because the checksum
    // covers everything after the checksum field itself.
    let mut label = [0_u8; 512];
    label[0..8].copy_from_slice(b"LABELONE");
    label[8..16].copy_from_slice(&1_u64.to_le_bytes()); // sector_xl
    // label[16..20] is crc_xl, filled in below.
    label[20..24].copy_from_slice(&32_u32.to_le_bytes()); // offset_xl
    label[24..32].copy_from_slice(b"LVM2 001");

    // PV header, at offset_xl from the start of the label.
    let uuid = pv_uuid.as_bytes();
    let uuid_len = uuid.len().min(32);
    label[32..32 + uuid_len].copy_from_slice(&uuid[..uuid_len]);
    label[64..72].copy_from_slice(&(image.sectors() * sector).to_le_bytes());

    // `libblkid` verifies this checksum, so a label with a zero CRC is detected
    // as nothing at all. It covers from `offset_xl` to the end of the sector.
    let crc = lvm2_crc(&label[20..512]);
    label[16..20].copy_from_slice(&crc.to_le_bytes());

    write_bytes(image, label_offset, &label);
}

/// LVM2's own CRC, which is not CRC-32.
///
/// A nibble-at-a-time variant with a distinct initial value. It is reproduced
/// here rather than approximated because `libblkid` verifies it before reporting
/// `LVM2_member`, so an ordinary CRC-32 would leave the fixture undetectable.
///
/// # Panics
///
/// Panics if a nibble does not fit in `usize`, which cannot occur.
#[must_use]
pub fn lvm2_crc(data: &[u8]) -> u32 {
    const TABLE: [u32; 16] = [
        0x0000_0000,
        0x1db7_1064,
        0x3b6e_20c8,
        0x26d9_30ac,
        0x76dc_4190,
        0x6b6b_51f4,
        0x4db2_6158,
        0x5005_713c,
        0xedb8_8320,
        0xf00f_9344,
        0xd6d6_a3e8,
        0xcb61_b38c,
        0x9b64_c2b0,
        0x86d3_d2d4,
        0xa00a_e278,
        0xbdbd_f21c,
    ];
    let mut crc = 0xf597_a6cf_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        crc = (crc >> 4) ^ TABLE[usize::try_from(crc & 0xf).expect("nibble fits usize")];
        crc = (crc >> 4) ^ TABLE[usize::try_from(crc & 0xf).expect("nibble fits usize")];
    }
    crc
}

/// Write an mdraid 1.2 superblock (LIN-005, FS-004).
pub fn mdraid_12(image: &mut Image, array_uuid_seed: &str) {
    let mut sb = [0_u8; 256];
    sb[0..4].copy_from_slice(&MDRAID_MAGIC.to_le_bytes());
    sb[4..8].copy_from_slice(&1_u32.to_le_bytes()); // major_version
    sb[16..32].copy_from_slice(crate::layout::Guid::derived(array_uuid_seed).as_bytes());
    // set_name, a NUL-padded field a prober reports as the array name.
    sb[32..36].copy_from_slice(b"pm:0");
    sb[72..76].copy_from_slice(&1_u32.to_le_bytes()); // level: RAID 1
    sb[80..88].copy_from_slice(&4096_u64.to_le_bytes()); // size, in 512-byte sectors
    sb[92..96].copy_from_slice(&2_u32.to_le_bytes()); // raid_disks
    sb[128..136].copy_from_slice(&2048_u64.to_le_bytes()); // data_offset
    sb[136..144].copy_from_slice(&4096_u64.to_le_bytes()); // data_size
    // super_offset, in 512-byte sectors, must agree with where this is written
    // or the superblock describes a location it is not in.
    sb[144..152].copy_from_slice(&(MDRAID_12_OFFSET / 512).to_le_bytes());

    // sb_csum. Without it `wipefs` still lists the superblock, because it
    // enumerates magic matches, while `blkid -p` reports nothing, because it
    // validates. That difference is itself worth knowing, and a fixture only
    // half the tooling recognizes would be a trap for whoever used it next.
    let csum = mdraid_1x_checksum(&sb);
    sb[216..220].copy_from_slice(&csum.to_le_bytes());

    write_bytes(image, MDRAID_12_OFFSET, &sb);
}

/// The mdraid 1.x superblock checksum: a folded sum of little-endian words.
///
/// Not a CRC. The superblock is summed as 32-bit words with its own checksum
/// field treated as zero, then the 64-bit total is folded into 32 bits.
///
/// # Panics
///
/// Panics if the folded sum does not fit in `u32`, which the fold prevents.
#[must_use]
pub fn mdraid_1x_checksum(superblock: &[u8; 256]) -> u32 {
    let mut zeroed = *superblock;
    zeroed[216..220].copy_from_slice(&0_u32.to_le_bytes());
    fold_words(&zeroed)
}

/// The 0.90 superblock checksum, with `sb_csum` (word 27) treated as zero.
///
/// The same folded word sum as 1.x, over a different field offset and a
/// different span.
fn mdraid_folded_checksum(superblock: &[u8; MDRAID_090_BYTES]) -> u32 {
    let mut zeroed = *superblock;
    zeroed[108..112].copy_from_slice(&0_u32.to_le_bytes());
    fold_words(&zeroed)
}

/// Sum little-endian 32-bit words, then fold the 64-bit total into 32 bits.
fn fold_words(bytes: &[u8]) -> u32 {
    let mut total = 0_u64;
    for word in bytes.chunks_exact(4) {
        let value = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        total += u64::from(value);
    }
    let folded = (total & 0xffff_ffff) + (total >> 32);
    u32::try_from(folded & 0xffff_ffff).expect("folded sum fits u32")
}

/// Write a legacy mdraid 0.90 superblock near the **end** of the device.
///
/// This is the fixture the stale-signature question needs. A 0.90 superblock
/// sits in the last 64 KiB-aligned block, which is why formatting the start of a
/// device does not remove it, and why a device can carry a current file system
/// and an obsolete array membership at the same time.
///
/// # Panics
///
/// Panics if the image is too small to hold a 0.90 superblock.
pub fn mdraid_090_at_end(image: &mut Image, array_uuid_seed: &str) {
    let offset = mdraid_090_offset(image);
    let uuid = *crate::layout::Guid::derived(array_uuid_seed).as_bytes();

    // A full 0.90 superblock, so the checksum covers what it is defined over.
    let mut sb = [0_u8; MDRAID_090_BYTES];
    sb[0..4].copy_from_slice(&MDRAID_MAGIC.to_le_bytes());
    sb[4..8].copy_from_slice(&0_u32.to_le_bytes()); // major_version, word 1
    sb[8..12].copy_from_slice(&90_u32.to_le_bytes()); // minor_version, word 2
    sb[12..16].copy_from_slice(&0_u32.to_le_bytes()); // patch_version, word 3
    sb[16..20].copy_from_slice(&0_u32.to_le_bytes()); // gvalid_words, word 4
    sb[28..32].copy_from_slice(&1_u32.to_le_bytes()); // level, word 7
    sb[32..36].copy_from_slice(&2048_u32.to_le_bytes()); // size in KiB, word 8
    sb[36..40].copy_from_slice(&2_u32.to_le_bytes()); // nr_disks, word 9
    sb[40..44].copy_from_slice(&2_u32.to_le_bytes()); // raid_disks, word 10

    // The 0.90 set UUID is split across four *non-adjacent* words: 5, then 13,
    // 14 and 15. An earlier version of this writer put the last three at bytes
    // 128, 132 and 136 — words 32, 33 and 34, which are `utime`, `state` and
    // `active_disks`. The array identity was three-quarters zero and the state
    // fields held UUID fragments. `blkid` reported the UUID as
    // `…-0000-0000-0000-000000000000`, which was the visible symptom.
    sb[20..24].copy_from_slice(&uuid[0..4]); // set_uuid0, word 5
    sb[52..56].copy_from_slice(&uuid[4..8]); // set_uuid1, word 13
    sb[56..60].copy_from_slice(&uuid[8..12]); // set_uuid2, word 14
    sb[60..64].copy_from_slice(&uuid[12..16]); // set_uuid3, word 15

    // sb_csum, word 27, over the whole superblock with the field zeroed.
    let csum = mdraid_folded_checksum(&sb);
    sb[108..112].copy_from_slice(&csum.to_le_bytes());

    write_bytes(image, offset, &sb);
}

/// Size of a 0.90 superblock, which its checksum is defined over.
const MDRAID_090_BYTES: usize = 4096;

/// Byte offset of a 0.90 superblock, by the kernel's own formula.
///
/// # Panics
///
/// Panics if the image is smaller than the 64 KiB the calculation assumes.
#[must_use]
pub fn mdraid_090_offset(image: &Image) -> u64 {
    let bytes = image.sectors() * image.sector().bytes();
    let kib = bytes / 1024;
    assert!(
        kib > 64,
        "an image below 64 KiB cannot hold a 0.90 superblock"
    );
    ((kib - 64) & !63_u64) * 1024
}

/// Smallest image `libblkid` will consider for ZFS.
///
/// Measured, not guessed: `libblkid` declares a 64 MiB minimum size for the ZFS
/// prober and skips it entirely below that, so a 4 MiB fixture carrying perfect
/// labels is detected as nothing at all. Every other fixture here is 4 MiB; this
/// one has to be sixteen times larger to be worth having.
pub const ZFS_MINIMUM_BYTES: u64 = 64 * 1024 * 1024;

/// Write ZFS vdev labels at both ends of the device (Section 2.1, FS-004).
///
/// ZFS writes four labels: two at the front and two at the back. Round two
/// observed that ordinary repurposing clears only the leading pair, which is
/// exactly why the trailing pair is written here.
///
/// # This does not yet produce a detectable member, and is not in the catalogue
///
/// Measured against `libblkid` 2.41: with the image at the required 64 MiB and
/// the uberblock magic present at the documented offset in all four labels —
/// both verified by hexdump — `blkid -p` still reports nothing. Some further
/// condition is required and **what it is has not been established**.
///
/// The writer is kept because the label placement it encodes is a real fact this
/// project depends on, and because the negative result is worth not repeating.
/// It is deliberately absent from [`crate::catalogue`]: a fixture named for a
/// format that no prober recognizes would be the exact trap this module's own
/// documentation warns against.
///
/// # Panics
///
/// Panics if the image is too small for four 256 KiB labels.
pub fn zfs_member(image: &mut Image) {
    const LABEL: u64 = 256 * 1024;
    const UBERBLOCK_OFFSET: u64 = 128 * 1024;
    const UBERBLOCK_SIZE: u64 = 1024;
    let total = image.sectors() * image.sector().bytes();
    assert!(
        total >= 4 * LABEL,
        "an image below 1 MiB cannot hold four ZFS labels"
    );

    // `libblkid` counts uberblocks and wants several before it will call a
    // device a pool member, so one magic is not enough. The uberblock array is
    // 128 KiB of fixed-size slots; filling the first eight clears that bar
    // without pretending the pool has more history than it does.
    let magic = 0x0000_0000_00ba_b10c_u64.to_le_bytes();
    for label in [0, LABEL, total - 2 * LABEL, total - LABEL] {
        for slot in 0..8 {
            write_bytes(
                image,
                label + UBERBLOCK_OFFSET + slot * UBERBLOCK_SIZE,
                &magic,
            );
        }
        // A name/value pair list would follow; the pool name is not needed for
        // detection and inventing one would imply a pool this fixture does not
        // describe.
    }
}

/// Write bytes at an absolute offset, translating to the image's sector grid.
fn write_bytes(image: &mut Image, offset: u64, data: &[u8]) {
    let sector = image.sector().bytes();
    let lba = offset / sector;
    let within = usize::try_from(offset % sector).expect("sector remainder fits usize");
    image.write_at(lba, within, data);
}

/// A whole-disk image with no partition table, carrying `writer`'s signature.
///
/// FS-004's structures occur whole-disk as often as inside a partition, and
/// ADR-C3's rejected carve-out turned on exactly that: a device with no table is
/// not a device with no data.
#[must_use]
pub fn whole_disk(sectors: u64, writer: impl FnOnce(&mut Image)) -> Image {
    let mut image = Image::blank(SectorSize::B512, sectors);
    writer(&mut image);
    image
}

#[cfg(test)]
mod tests;
