use super::{
    ext4, luks2, lvm2_pv, mdraid_12, mdraid_090_at_end, mdraid_090_offset, whole_disk, zfs_member,
};
use crate::layout::{Image, SectorSize};

#[test]
fn every_writer_is_deterministic() {
    let one = whole_disk(8192, |image| ext4(image, "seed")).into_bytes();
    let two = whole_disk(8192, |image| ext4(image, "seed")).into_bytes();
    assert_eq!(one, two);
}

#[test]
fn the_ext4_magic_lands_where_a_prober_looks() {
    let image = whole_disk(8192, |image| ext4(image, "seed"));
    let bytes = image.bytes();
    assert_eq!(&bytes[1024 + 56..1024 + 58], &0xef53_u16.to_le_bytes());
}

#[test]
fn the_luks2_version_is_big_endian() {
    // The one field in this module that is not little-endian.
    let image = whole_disk(8192, |image| luks2(image, "0f8b2c1e-luks"));
    assert_eq!(&image.bytes()[0..6], &[0x4c, 0x55, 0x4b, 0x53, 0xba, 0xbe]);
    assert_eq!(&image.bytes()[6..8], &2_u16.to_be_bytes());
}

#[test]
fn the_lvm2_label_carries_both_strings_a_prober_requires() {
    // Writing LABELONE alone detects nothing; the type string is required too.
    let image = whole_disk(8192, |image| lvm2_pv(image, "pv-uuid"));
    assert_eq!(&image.bytes()[512..520], b"LABELONE");
    assert_eq!(&image.bytes()[536..544], b"LVM2 001");
}

#[test]
fn the_090_superblock_sits_near_the_end_not_the_start() {
    // The whole point of the stale-signature fixture: formatting the start of a
    // device does not reach this.
    let image = Image::blank(SectorSize::B512, 8192);
    let offset = mdraid_090_offset(&image);
    let total = 8192 * 512;
    assert!(offset > total / 2, "0.90 lives near the end");
    assert!(offset + 4096 <= total, "and still inside the device");
    assert_eq!(offset % (64 * 1024), 0, "aligned to 64 KiB");
}

#[test]
fn a_device_can_carry_a_file_system_and_a_stale_array_membership() {
    let image = whole_disk(8192, |image| {
        ext4(image, "fs");
        mdraid_090_at_end(image, "array");
    });
    let bytes = image.bytes();
    assert_eq!(&bytes[1024 + 56..1024 + 58], &0xef53_u16.to_le_bytes());
    let offset = usize::try_from(mdraid_090_offset(&image)).expect("fits");
    assert_eq!(&bytes[offset..offset + 4], &0xa92b_4efc_u32.to_le_bytes());
}

#[test]
fn mdraid_12_sits_four_kibibytes_in() {
    let image = whole_disk(8192, |image| mdraid_12(image, "array"));
    assert_eq!(&image.bytes()[4096..4100], &0xa92b_4efc_u32.to_le_bytes());
}

#[test]
fn zfs_labels_are_written_at_both_ends() {
    // Round two's observation: repurposing clears the leading pair only.
    let image = whole_disk(8192, zfs_member);
    let bytes = image.bytes();
    let magic = 0x0000_0000_00ba_b10c_u64.to_le_bytes();
    let total = 8192 * 512;
    assert_eq!(&bytes[131_072..131_080], &magic, "label 0");
    assert_eq!(
        &bytes[total - 262_144 + 131_072..total - 262_144 + 131_080],
        &magic,
        "trailing label"
    );
}

#[test]
fn the_lvm2_checksum_is_not_an_ordinary_crc32() {
    // libblkid verifies this before reporting LVM2_member, so getting it wrong
    // leaves the fixture detected as nothing at all -- which is how it behaved
    // before the algorithm was reproduced correctly.
    assert_ne!(
        super::lvm2_crc(b"123456789"),
        crate::layout::crc32(b"123456789")
    );
    // Stable, so a fixture's bytes do not move between runs.
    assert_eq!(super::lvm2_crc(b"123456789"), super::lvm2_crc(b"123456789"));
}

#[test]
fn the_mdraid_checksum_zeroes_its_own_field_before_summing() {
    // Otherwise the checksum would depend on whatever happened to be in the
    // field already, and would not be reproducible.
    let mut sb = [0_u8; 256];
    sb[0..4].copy_from_slice(&0xa92b_4efc_u32.to_le_bytes());
    let first = super::mdraid_1x_checksum(&sb);
    sb[216..220].copy_from_slice(&0xdead_beef_u32.to_le_bytes());
    assert_eq!(super::mdraid_1x_checksum(&sb), first);
}

#[test]
fn the_multi_signature_fixture_carries_both_at_the_offsets_a_prober_reads() {
    // The fixture that answered a question two earlier attempts could not: a
    // live ext4 at the start and a stale 0.90 array membership near the end.
    // `wipefs` reports both; `blkid -p` reports only the RAID member.
    let image = whole_disk(8192, |image| {
        ext4(image, "stale-pair/fs");
        mdraid_090_at_end(image, "stale-pair/array");
    });
    let bytes = image.bytes();

    // ext4 superblock magic, at the offset `wipefs` reports as 0x438.
    assert_eq!(&bytes[0x0438..0x043a], &0xef53_u16.to_le_bytes());
    // mdraid magic, at the offset `wipefs` reports as 0x3f0000 for this size.
    assert_eq!(
        &bytes[0x003f_0000..0x003f_0004],
        &0xa92b_4efc_u32.to_le_bytes()
    );
}

#[test]
fn the_090_set_uuid_occupies_its_four_non_adjacent_words() {
    // The defect an audit found: words 13, 14 and 15 were written at bytes 128,
    // 132 and 136 -- words 32, 33 and 34, which are `utime`, `state` and
    // `active_disks`. Three quarters of the array identity was zero, and
    // `blkid` showed it as `fb2871eb-0000-0000-0000-000000000000`.
    let image = whole_disk(8192, |image| mdraid_090_at_end(image, "uuid-check"));
    let offset = usize::try_from(mdraid_090_offset(&image)).expect("fits");
    let sb = &image.bytes()[offset..offset + 256];
    let uuid = *crate::layout::Guid::derived("uuid-check").as_bytes();

    assert_eq!(&sb[20..24], &uuid[0..4], "set_uuid0, word 5");
    assert_eq!(&sb[52..56], &uuid[4..8], "set_uuid1, word 13");
    assert_eq!(&sb[56..60], &uuid[8..12], "set_uuid2, word 14");
    assert_eq!(&sb[60..64], &uuid[12..16], "set_uuid3, word 15");

    // No part of the identity may be zero-filled, which is what the bug looked
    // like from outside.
    assert!(
        sb[52..64].iter().any(|byte| *byte != 0),
        "the trailing three words must not be blank"
    );
}

#[test]
fn the_luks2_fields_do_not_land_inside_the_label() {
    // `checksum_alg` was written at 24 and the UUID at 32, both inside the
    // 48-byte `label` field, so the fixture had no UUID at all.
    let image = whole_disk(8192, |image| {
        luks2(image, "5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f");
    });
    let header = image.bytes();

    assert!(
        header[24..72].iter().all(|byte| *byte == 0),
        "label must be empty, not carrying other fields"
    );
    assert_eq!(&header[72..78], b"sha256", "checksum_alg at 72");
    assert_eq!(
        &header[168..204],
        b"5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f",
        "uuid at 168"
    );
}

#[test]
fn the_ext4_block_count_matches_the_device_it_is_written_to() {
    // The first version declared 8192 one-kibibyte blocks -- 8 MiB -- on a 4 MiB
    // image, having reused a sector count as a block count.
    for sectors in [8192_u64, 2048] {
        let image = whole_disk(sectors, |image| ext4(image, "size-check"));
        let declared = u32::from_le_bytes([
            image.bytes()[1024 + 4],
            image.bytes()[1024 + 5],
            image.bytes()[1024 + 6],
            image.bytes()[1024 + 7],
        ]);
        let device_kib = u32::try_from(sectors * 512 / 1024).expect("fits");
        assert_eq!(declared, device_kib, "blocks must not exceed the device");
        // s_first_data_block must be 1 at a 1 KiB block size.
        assert_eq!(&image.bytes()[1024 + 20..1024 + 24], &1_u32.to_le_bytes());
    }
}
