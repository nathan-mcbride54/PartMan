use super::{
    EFI_SYSTEM, GptPartition, Guid, Image, LINUX_FILESYSTEM, MbrPartition, SectorSize, apm,
    corrupt_primary_header_crc, crc32, gpt, mbr,
};

fn sample_partitions() -> Vec<GptPartition> {
    vec![
        GptPartition {
            type_guid: EFI_SYSTEM,
            unique_guid: Guid::derived("test/esp"),
            first_lba: 2048,
            last_lba: 4095,
            name: "EFI System",
        },
        GptPartition {
            type_guid: LINUX_FILESYSTEM,
            unique_guid: Guid::derived("test/root"),
            first_lba: 4096,
            last_lba: 8158,
            name: "root",
        },
    ]
}

// Requirements: INV-003
//   The fixture CRC-32 implementation reproduces the published IEEE check value instead of validating GPT checksums only against itself
// Evidence: crc32_matches_the_published_check_value
#[test]
fn crc32_matches_the_published_check_value() {
    // The IEEE check value every CRC-32 implementation is expected to reproduce.
    // Getting this wrong would make every GPT fixture subtly invalid while still
    // looking plausible.
    assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    assert_eq!(crc32(b""), 0);
}

// Requirements: Section 11.3
//   Rebuilding one GPT fixture from the same inputs produces byte-identical output that can be generated and cached instead of committed
// Evidence: generation_is_deterministic
#[test]
fn generation_is_deterministic() {
    // The property that lets a fixture be pinned by digest instead of committed
    // (Section 11.3, Section 16).
    let first = gpt(SectorSize::B512, 8192, "det", &sample_partitions()).into_bytes();
    let second = gpt(SectorSize::B512, 8192, "det", &sample_partitions()).into_bytes();
    assert_eq!(first, second);
}

// Requirements: Section 11.3
//   Derived fixture identifiers are stable, input-sensitive, and carry the expected UUID version and variant bits
// Evidence: a_derived_guid_is_stable_and_well_formed
#[test]
fn a_derived_guid_is_stable_and_well_formed() {
    let a = Guid::derived("label");
    let b = Guid::derived("label");
    assert_eq!(a, b);
    assert_ne!(a, Guid::derived("other"));

    let raw = a.as_bytes();
    assert_eq!(raw[7] & 0xf0, 0x40, "version nibble must say 4");
    assert_eq!(raw[8] & 0xc0, 0x80, "variant bits must be RFC 4122");
}

// Requirements: INV-003
//   A GPT fixture contains a protective MBR plus primary and backup GPT headers at their required locations
// Evidence: a_gpt_image_carries_both_headers_and_a_protective_mbr
#[test]
fn a_gpt_image_carries_both_headers_and_a_protective_mbr() {
    let image = gpt(SectorSize::B512, 8192, "basic", &sample_partitions());
    let bytes = image.bytes();

    assert_eq!(&bytes[510..512], &[0x55, 0xaa], "protective MBR signature");
    assert_eq!(bytes[450], 0xee, "protective MBR partition type");
    assert_eq!(&bytes[512..520], b"EFI PART", "primary header signature");

    let last = 8191 * 512;
    assert_eq!(
        &bytes[last..last + 8],
        b"EFI PART",
        "backup header signature"
    );
}

// Requirements: INV-003
//   A conforming independent recomputation accepts the primary GPT header checksum emitted by the fixture writer
// Evidence: the_gpt_header_crc_validates
#[test]
fn the_gpt_header_crc_validates() {
    let image = gpt(SectorSize::B512, 8192, "crc", &sample_partitions());
    assert!(
        header_crc_is_valid(image.bytes()),
        "primary header must verify"
    );
}

// Requirements: INV-003
//   The damaged-primary fixture retains GPT magic while invalidating its primary checksum, representing recoverable damage rather than blank or ambiguous media
// Evidence: the_corrupt_fixture_keeps_its_signature_but_fails_its_crc
#[test]
fn the_corrupt_fixture_keeps_its_signature_but_fails_its_crc() {
    // A damaged table has to stay distinguishable from blank media, so the
    // signature must survive while the checksum does not. This is the
    // *recoverable* case, not ADR-C3's `Indeterminate` one — the comment here
    // said Indeterminate until an audit caught it, which is the same mislabel
    // the catalogue was corrected for.
    let mut image = gpt(SectorSize::B512, 8192, "corrupt", &sample_partitions());
    assert!(header_crc_is_valid(image.bytes()));

    corrupt_primary_header_crc(&mut image);
    assert_eq!(
        &image.bytes()[512..520],
        b"EFI PART",
        "a corrupt table still claims to be a table"
    );
    assert!(
        !header_crc_is_valid(image.bytes()),
        "the checksum must no longer verify"
    );
}

/// Recompute the primary header CRC the way a conforming reader would.
fn header_crc_is_valid(bytes: &[u8]) -> bool {
    let header = &bytes[512..512 + 92];
    let mut recomputed = header.to_vec();
    let stored = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
    recomputed[16..20].copy_from_slice(&0_u32.to_le_bytes());
    crc32(&recomputed) == stored
}

// Requirements: IMG-011
//   The 4Kn fixture has 4096-byte sectors and places its GPT header at byte 4096 instead of reusing a 512-byte layout
// Evidence: a_4kn_image_has_4096_byte_sectors
#[test]
fn a_4kn_image_has_4096_byte_sectors() {
    let image = gpt(SectorSize::B4096, 1024, "4kn", &[]);
    assert_eq!(image.sector().bytes(), 4096);
    assert_eq!(image.sectors(), 1024);
    assert_eq!(image.bytes().len(), 1024 * 4096);
    // The header lives at LBA 1, which is 4096 bytes in rather than 512.
    assert_eq!(&image.bytes()[4096..4104], b"EFI PART");
}

// Requirements: INV-003
//   The MBR fixture records its active flag, partition kind, extent, and boot signature in the on-disk table
// Evidence: an_mbr_image_records_its_entries
#[test]
fn an_mbr_image_records_its_entries() {
    let image = mbr(
        SectorSize::B512,
        8192,
        &[MbrPartition {
            kind: 0x83,
            active: true,
            first_lba: 2048,
            sectors: 4096,
        }],
    );
    let bytes = image.bytes();
    assert_eq!(bytes[446], 0x80, "active flag");
    assert_eq!(bytes[450], 0x83, "partition type");
    assert_eq!(&bytes[454..458], &2048_u32.to_le_bytes());
    assert_eq!(&bytes[458..462], &4096_u32.to_le_bytes());
    assert_eq!(&bytes[510..512], &[0x55, 0xaa]);
}

// Requirements: INV-003
//   The APM fixture encodes its driver descriptor and partition-map fields in big-endian form
// Evidence: an_apm_image_is_big_endian
#[test]
fn an_apm_image_is_big_endian() {
    // The reason APM is in the catalogue at all: a reader that assumes
    // little-endian passes every other fixture and fails only here.
    let image = apm(SectorSize::B512, 8192, &[("Apple", "Apple_HFS", 1, 63)]);
    let bytes = image.bytes();
    assert_eq!(&bytes[0..2], &[0x45, 0x52], "'ER' driver descriptor");
    assert_eq!(
        &bytes[2..4],
        &512_u16.to_be_bytes(),
        "block size, big-endian"
    );
    assert_eq!(&bytes[512..514], &[0x50, 0x4d], "'PM' map entry");
    assert_eq!(&bytes[520..524], &1_u32.to_be_bytes(), "start, big-endian");
}

// Requirements: PART-001, INV-003
//   A blank-media fixture is positively empty across its full declared size rather than merely lacking a filename or table entry
// Evidence: a_blank_image_is_entirely_zero
#[test]
fn a_blank_image_is_entirely_zero() {
    // PART-001's target, and ADR-C3's positively-observed-absent state.
    let image = Image::blank(SectorSize::B512, 64);
    assert!(image.bytes().iter().all(|byte| *byte == 0));
    assert_eq!(image.bytes().len(), 64 * 512);
}

// Requirements: Section 11.3, Section 12
//   A fixture writer panics on an out-of-bounds write instead of silently truncating malformed evidence and reporting success
// Evidence: writing_past_the_end_panics_rather_than_truncating
#[test]
#[should_panic(expected = "fixture write past end of image")]
fn writing_past_the_end_panics_rather_than_truncating() {
    // A fixture that silently truncated its own table would be a test that
    // proves nothing while appearing to pass.
    let mut image = Image::blank(SectorSize::B512, 1);
    image.write_at(0, 500, &[0_u8; 64]);
}
