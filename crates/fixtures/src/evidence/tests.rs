//! These tests have to prove three things, and the last two were missing from
//! the first version of this module.
//!
//! 1. Every fixture satisfies its claim.
//! 2. Each claim is *capable of failing* — a check that accepts everything
//!    would pass (1) forever while proving nothing.
//! 3. Each claim fails **for the right reason**. A mutation that trips an
//!    unrelated assertion counts as proof of nothing, and an adversarial pass
//!    found two mutations here that did exactly that.
//!
//! So every mutation carries the phrase its refusal must contain. That is what
//! separates "the claim rejected this" from "something rejected this".

use std::collections::BTreeSet;

use super::{Missing, claims, verify, verify_catalogue};
use crate::catalogue::catalogue;

#[test]
fn every_fixture_has_a_claim_and_every_claim_has_a_fixture() {
    // Exhaustive in both directions. One direction alone would let a fixture be
    // added with nothing constraining it, which is how the catalogue came to
    // hold images nothing bound to their stated purpose.
    let fixtures: BTreeSet<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
    let claimed: BTreeSet<&str> = claims().iter().map(|claim| claim.fixture).collect();

    let unclaimed: Vec<&&str> = fixtures.difference(&claimed).collect();
    assert!(
        unclaimed.is_empty(),
        "these fixtures have no claim, so nothing stops them becoming something else: {unclaimed:?}"
    );
    let orphaned: Vec<&&str> = claimed.difference(&fixtures).collect();
    assert!(
        orphaned.is_empty(),
        "these claims name no fixture in the catalogue: {orphaned:?}"
    );
    assert_eq!(claims().len(), claimed.len(), "a fixture is claimed twice");
}

#[test]
fn every_fixture_satisfies_its_claim() {
    // Built from the catalogue's own builders, not from literals restated here.
    // Rebuilding the image inside the test is what made the previous layout and
    // signature suites unable to see a catalogue change at all.
    verify_catalogue().expect("every fixture must support its own rationale");
}

#[test]
fn a_fixture_with_no_registered_claim_is_refused_rather_than_passed() {
    // Treating an unknown name as "nothing to check" would reintroduce the gap
    // silently, one new fixture at a time.
    let refusal = verify("something-nobody-registered.img", &[0; 16])
        .expect_err("an unregistered fixture must not pass");
    assert!(matches!(refusal, Missing::NoClaim { .. }));
}

/// A mutation a claim must reject, and the reason it must give.
struct Mutation {
    fixture: &'static str,
    /// What the mutation does to the fixture's purpose, in plain terms.
    breaks: &'static str,
    /// A phrase the refusal must contain, so the *intended* check is what
    /// fired. Without this a mutation that merely changed the image's length
    /// would have counted as proof that a claim works.
    expect: &'static str,
    apply: fn(&mut [u8]),
}

/// At least one way each fixture can stop being what it is named for.
///
/// The two marked below are not hypothetical: they were applied to the
/// catalogue on this branch and the entire 64-test suite stayed green.
fn mutations() -> Vec<Mutation> {
    let mut all = table_mutations();
    all.extend(partition_mutations());
    all.extend(signature_mutations());
    all
}

/// Mutations that damage a table's structure outright.
fn table_mutations() -> Vec<Mutation> {
    vec![
        Mutation {
            fixture: "blank-512.img",
            breaks: "the medium is no longer blank",
            expect: "not blank",
            apply: |bytes| bytes[1_000_000] = 1,
        },
        Mutation {
            fixture: "gpt-basic-512.img",
            breaks: "the primary header stops checksumming",
            expect: "no valid primary header",
            apply: |bytes| bytes[520] ^= 0xff,
        },
        Mutation {
            fixture: "gpt-basic-512.img",
            breaks: "the entry array no longer matches the CRC its header declares",
            expect: "entry array does not",
            apply: |bytes| bytes[1024] ^= 0xff,
        },
        Mutation {
            fixture: "gpt-basic-4kn.img",
            breaks: "a 512-byte reader would now find a table, so the fixture traps nobody",
            expect: "byte 512",
            apply: |bytes| {
                let header: Vec<u8> = bytes[4096..4096 + 92].to_vec();
                bytes[512..512 + 92].copy_from_slice(&header);
            },
        },
        Mutation {
            fixture: "gpt-basic-4kn.img",
            breaks: "the 4Kn header is gone",
            expect: "no valid primary header",
            apply: |bytes| bytes[4096..4100].fill(0),
        },
        Mutation {
            fixture: "mbr-basic-512.img",
            breaks: "the table becomes a GPT's protective MBR rather than a real one",
            expect: "protective MBR",
            apply: |bytes| bytes[450] = 0xee,
        },
        Mutation {
            fixture: "gpt-invalid-primary-valid-backup-512.img",
            breaks: "the primary is repaired, so nothing is damaged and the fixture is a duplicate",
            expect: "still checksums",
            apply: |bytes| repair_gpt(bytes, 512, 512),
        },
        Mutation {
            fixture: "gpt-conflicting-tables-512.img",
            breaks: "the backup stops checksumming, making this damaged rather than ambiguous",
            expect: "backup must checksum",
            apply: |bytes| {
                let last = bytes.len() - 512;
                bytes[last + 24] ^= 0xff;
            },
        },
        Mutation {
            fixture: "gpt-missing-backup-512.img",
            breaks: "something reappears in the last sector",
            expect: "backup survives",
            apply: |bytes| {
                let last = bytes.len() - 512;
                bytes[last] = 1;
            },
        },
        Mutation {
            // The gap the shipped fixture actually had: zeroing only the header
            // sector left 16 KiB of byte-identical backup entry array behind,
            // which any scanning recovery tool would find.
            fixture: "gpt-missing-backup-512.img",
            breaks: "the backup entry array survives, so the backup is not gone",
            expect: "backup survives",
            apply: |bytes| {
                let at = bytes.len() - 33 * 512;
                bytes[at] = 1;
            },
        },
        Mutation {
            fixture: "hybrid-mbr-gpt-512.img",
            breaks: "the second entry becomes protective too, so the schemes cannot disagree",
            expect: "aliases a GPT partition",
            apply: |bytes| bytes[466] = 0xee,
        },
        Mutation {
            fixture: "apm-basic-512.img",
            breaks: "the block size is written little-endian, so the endianness trap is gone",
            expect: "big-endian, not 512",
            apply: |bytes| bytes[2..4].copy_from_slice(&512_u16.to_le_bytes()),
        },
        Mutation {
            fixture: "apm-basic-512.img",
            breaks: "the second map entry's extent is wrong, and nothing used to read it",
            expect: "map entry 1 spans",
            apply: |bytes| bytes[1024 + 8..1024 + 12].copy_from_slice(&0_u32.to_be_bytes()),
        },
        Mutation {
            fixture: "apm-basic-512.img",
            breaks: "the HFS partition loses its type string, which is what a reader reports",
            expect: "not of type",
            apply: |bytes| bytes[1024 + 48..1024 + 80].fill(0),
        },
    ]
}

/// Mutations that leave every checksum valid and change what the table *says*.
///
/// These are the ones the first version of this module could not catch. Each
/// repairs the CRCs it disturbs, so the table stays well-formed and only its
/// meaning changes — which is exactly how a fixture stops serving its purpose
/// without looking broken.
fn partition_mutations() -> Vec<Mutation> {
    vec![
        Mutation {
            fixture: "gpt-basic-512.img",
            breaks: "a partition runs ~100 MB past the end of a 4 MiB device",
            expect: "usable range",
            apply: |bytes| {
                edit_both_arrays(bytes, 512, |array| {
                    array[32..40].copy_from_slice(&100_000_u64.to_le_bytes());
                    array[40..48].copy_from_slice(&200_000_u64.to_le_bytes());
                });
            },
        },
        Mutation {
            fixture: "gpt-basic-4kn.img",
            breaks: "the 4Kn table carries the 512-byte arm's literal LBAs",
            expect: "usable range",
            apply: |bytes| {
                edit_both_arrays(bytes, 4096, |array| {
                    array[32..40].copy_from_slice(&2048_u64.to_le_bytes());
                    array[40..48].copy_from_slice(&4095_u64.to_le_bytes());
                });
            },
        },
        Mutation {
            // The finding that mattered most: "two tables that disagree" was
            // proven only by CRC inequality, and one character of a partition
            // *name* satisfies that while both copies describe identical
            // extents.
            fixture: "gpt-conflicting-tables-512.img",
            breaks: "both copies describe identical extents and differ only in a name",
            expect: "same extents",
            apply: |bytes| {
                let sector = 512_usize;
                let primary_array = array_start(bytes, sector, sector);
                let backup_array = array_start(bytes, bytes.len() - sector, sector);
                let length = 128 * 128;
                let copy: Vec<u8> = bytes[primary_array..primary_array + length].to_vec();
                bytes[backup_array..backup_array + length].copy_from_slice(&copy);
                // One character of the first partition's UTF-16 name.
                bytes[backup_array + 56] ^= 0x20;
                repair_gpt(bytes, bytes.len() - sector, sector);
            },
        },
        Mutation {
            fixture: "gpt-invalid-primary-valid-backup-512.img",
            breaks: "the surviving backup describes no partitions, so there is nothing to recover",
            expect: "populated partition entries",
            apply: |bytes| {
                let sector = 512_usize;
                let backup_array = array_start(bytes, bytes.len() - sector, sector);
                bytes[backup_array..backup_array + 128 * 128].fill(0);
                repair_gpt(bytes, bytes.len() - sector, sector);
            },
        },
        Mutation {
            fixture: "mbr-basic-512.img",
            breaks: "the two entries overlap and the second runs far past the device",
            expect: "overlap",
            apply: |bytes| {
                bytes[470..474].copy_from_slice(&2048_u32.to_le_bytes());
            },
        },
    ]
}

/// Mutations against the on-disk-signature fixtures.
fn signature_mutations() -> Vec<Mutation> {
    vec![
        Mutation {
            // Applied for real: the LUKS2 builder was replaced with a blank
            // whole-disk image and all 64 tests passed.
            fixture: "luks2-whole-disk-512.img",
            breaks: "the header is gone entirely, which is the defect that motivated this module",
            expect: "no LUKS magic",
            apply: |bytes| bytes[0..208].fill(0),
        },
        Mutation {
            fixture: "luks2-whole-disk-512.img",
            breaks: "the UUID moves back inside the label field, leaving no identity",
            expect: "label field is not empty",
            apply: |bytes| bytes[24..72].copy_from_slice(&[b'x'; 48]),
        },
        Mutation {
            fixture: "luks2-whole-disk-512.img",
            breaks: "the UUID becomes 36 characters that are not a UUID",
            expect: "not an identity",
            apply: |bytes| bytes[168..204].copy_from_slice(b"not-a-uuid-not-a-uuid-not-a-uuid-not"),
        },
        Mutation {
            fixture: "lvm2-pv-orphan-512.img",
            breaks: "the label checksum stops verifying, so libblkid detects nothing",
            expect: "label checksum is",
            apply: |bytes| bytes[544] ^= 0xff,
        },
        Mutation {
            // `sector_xl` lies outside the CRC span, so this leaves every
            // checksum valid while making the label undetectable.
            fixture: "lvm2-pv-orphan-512.img",
            breaks: "the label claims to live in a sector it is not in, so libblkid skips it",
            expect: "says it lives in sector",
            apply: |bytes| bytes[520..528].copy_from_slice(&7_u64.to_le_bytes()),
        },
        Mutation {
            fixture: "lvm2-pv-orphan-512.img",
            breaks: "the PV header pointer moves, so the UUID is not where the label says",
            expect: "PV header offset",
            apply: |bytes| {
                bytes[532..536].copy_from_slice(&200_u32.to_le_bytes());
                repair_lvm2(bytes);
            },
        },
        Mutation {
            fixture: "mdraid-1.2-member-512.img",
            breaks: "the superblock checksum stops verifying, so blkid -p reports nothing",
            expect: "superblock checksum is",
            apply: |bytes| bytes[4096 + 32] ^= 0xff,
        },
        Mutation {
            // The checksum has to be repaired, or it fires first and the orphan
            // check is never reached. An adversarial pass caught exactly that.
            fixture: "mdraid-1.2-member-512.img",
            breaks: "the array needs only one disk, so this member is no longer an orphan",
            expect: "raid_disks",
            apply: |bytes| {
                bytes[4096 + 92..4096 + 96].copy_from_slice(&1_u32.to_le_bytes());
                repair_mdraid_12(bytes);
            },
        },
        Mutation {
            fixture: "mdraid-1.2-member-512.img",
            breaks: "the member's data region begins past the end of the device",
            expect: "data region ends at",
            apply: |bytes| {
                bytes[4096 + 128..4096 + 136].copy_from_slice(&100_000_u64.to_le_bytes());
                repair_mdraid_12(bytes);
            },
        },
        Mutation {
            // Applied for real: the stale-mdraid half was removed from the
            // builder and all 64 tests passed.
            fixture: "ext4-with-stale-mdraid-090-512.img",
            breaks: "the stale array membership is gone, which is the entire point of the fixture",
            expect: "no mdraid magic",
            apply: |bytes| bytes[0x003f_0000..0x003f_1000].fill(0),
        },
        Mutation {
            fixture: "ext4-with-stale-mdraid-090-512.img",
            breaks: "the live file system is gone, leaving an ordinary orphaned member",
            expect: "no ext4 superblock magic",
            apply: |bytes| bytes[0x0438..0x043a].fill(0),
        },
        Mutation {
            fixture: "ext4-with-stale-mdraid-090-512.img",
            breaks: "the feature flags go, so a prober names it ext2 on a fixture called ext4",
            expect: "feature flags are",
            apply: |bytes| bytes[1024 + 92..1024 + 100].fill(0),
        },
        Mutation {
            fixture: "ext4-with-stale-mdraid-090-512.img",
            breaks: "the block size changes, so the declared count is 16 MiB on a 4 MiB device",
            expect: "s_log_block_size",
            apply: |bytes| bytes[1024 + 24..1024 + 28].copy_from_slice(&2_u32.to_le_bytes()),
        },
    ]
}

#[test]
fn every_claim_rejects_a_mutation_that_breaks_it_and_says_why() {
    // The test that makes the others mean something. A claim that cannot fail
    // is indistinguishable from no claim at all, and a claim that fails for the
    // wrong reason is indistinguishable from one that noticed.
    for mutation in mutations() {
        let fixture = catalogue()
            .into_iter()
            .find(|fixture| fixture.name == mutation.fixture)
            .unwrap_or_else(|| panic!("{} must be in the catalogue", mutation.fixture));
        let mut bytes = (fixture.build)().into_bytes();

        verify(mutation.fixture, &bytes)
            .unwrap_or_else(|error| panic!("the unmutated fixture must pass first: {error}"));

        (mutation.apply)(&mut bytes);
        let refusal = verify(mutation.fixture, &bytes).err().unwrap_or_else(|| {
            panic!(
                "{} accepted a fixture where {}; the claim cannot fail and so proves nothing",
                mutation.fixture, mutation.breaks
            )
        });
        let detail = refusal.to_string();
        assert!(
            detail.contains(mutation.expect),
            "{} refused a fixture where {} — but for the wrong reason. Expected a refusal \
             mentioning {:?}, got: {detail}",
            mutation.fixture,
            mutation.breaks,
            mutation.expect
        );
    }
}

#[test]
fn every_fixture_has_at_least_one_mutation_that_must_be_caught() {
    // Otherwise a claim could be added with no proof it is capable of failing,
    // which is the same gap one level up. Together with the test above this is
    // what rules out a claim whose check is `|_| Ok(())`.
    let fixtures: BTreeSet<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
    let mutated: BTreeSet<&str> = mutations().iter().map(|m| m.fixture).collect();
    let unproven: Vec<&&str> = fixtures.difference(&mutated).collect();
    assert!(
        unproven.is_empty(),
        "these claims have no mutation proving they can fail: {unproven:?}"
    );
}

#[test]
fn two_fixtures_may_not_claim_one_identity() {
    // No single-image claim can see this, so `verify_catalogue` checks the set.
    // Reseeding the mdraid member with the stale fixture's seed would give two
    // catalogue images one array UUID, and "no array assembles from one
    // fixture" would be false against the catalogue it lives in.
    let mdraid = build("mdraid-1.2-member-512.img");
    let stale = build("ext4-with-stale-mdraid-090-512.img");
    let at = 0x003f_0000;
    let mut stale_uuid = stale[at + 20..at + 24].to_vec();
    stale_uuid.extend_from_slice(&stale[at + 52..at + 64]);
    assert_ne!(
        mdraid[4112..4128].to_vec(),
        stale_uuid,
        "the two array fixtures must not share a set UUID"
    );
    verify_catalogue().expect("the catalogue's identities must be distinct");
}

fn build(name: &str) -> Vec<u8> {
    let fixture = catalogue()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("{name} must be in the catalogue"));
    (fixture.build)().into_bytes()
}

// --- Anchors -------------------------------------------------------------
//
// The module claims each reimplemented checksum is "anchored outside this
// repository". For CRC-32 that was true and executable. For the other two it
// was not: an adversarial pass observed that changing the initial constant in
// *both* the writer and the oracle keeps every test green while making every
// fixture undetectable, because the only thing being compared was the two
// implementations to each other — the exact "two opinions" the module doc says
// would be insufficient.
//
// These pin the values a real prober accepted. Captured 2026-07-28 from
// `libblkid` 2.41.0 on Debian, where `blkid -p -o udev` reported
// `LVM2_member` and `linux_raid_member` for the fixtures below. Changing an
// algorithm now changes a stored field, and these fail.

/// The LVM2 label checksum in the fixture `libblkid` reported as `LVM2_member`.
const LVM2_LABEL_CRC: u32 = 0xd226_5cf5;
/// The mdraid 1.2 `sb_csum` in the fixture `blkid -p` validated.
const MDRAID_12_CSUM: u32 = 0xa1e6_80fb;
/// The mdraid 0.90 `sb_csum` in the fixture `blkid -p` validated.
const MDRAID_090_CSUM: u32 = 0xc5b2_91a4;

#[test]
fn the_lvm2_checksum_matches_the_value_libblkid_accepted() {
    let bytes = build("lvm2-pv-orphan-512.img");
    let stored = u32::from_le_bytes([bytes[528], bytes[529], bytes[530], bytes[531]]);
    assert_eq!(
        stored, LVM2_LABEL_CRC,
        "the label checksum changed; libblkid 2.41 accepted {LVM2_LABEL_CRC:#010x}"
    );
    assert_eq!(super::lvm2_crc_bitwise(&bytes[532..1024]), LVM2_LABEL_CRC);
}

#[test]
fn the_mdraid_checksums_match_the_values_blkid_validated() {
    let member = build("mdraid-1.2-member-512.img");
    let stored = u32::from_le_bytes([member[4312], member[4313], member[4314], member[4315]]);
    assert_eq!(
        stored, MDRAID_12_CSUM,
        "the 1.2 superblock checksum changed; blkid 2.41 validated {MDRAID_12_CSUM:#010x}"
    );

    let stale = build("ext4-with-stale-mdraid-090-512.img");
    let at = 0x003f_0000 + 108;
    let stored = u32::from_le_bytes([stale[at], stale[at + 1], stale[at + 2], stale[at + 3]]);
    assert_eq!(
        stored, MDRAID_090_CSUM,
        "the 0.90 superblock checksum changed; blkid 2.41 validated {MDRAID_090_CSUM:#010x}"
    );
}

#[test]
fn the_crc32_here_reproduces_the_published_check_value() {
    // The anchor outside this repository. Both this table-driven version and
    // `layout`'s bitwise one are pinned to it, so agreeing with each other is
    // evidence rather than coincidence.
    assert_eq!(super::crc32_table(b"123456789"), 0xcbf4_3926);
    assert_eq!(super::crc32_table(b""), 0);
}

#[test]
fn the_two_crc32_implementations_agree_without_sharing_code() {
    // One iterates bits, the other looks up bytes. A defect in either would
    // have to be reproduced independently in the other to go unnoticed.
    for length in 0..64_usize {
        let data: Vec<u8> = (0..length).map(|index| spread(index, 37, 11)).collect();
        assert_eq!(
            super::crc32_table(&data),
            crate::layout::crc32(&data),
            "disagreement at length {length}"
        );
    }
}

#[test]
fn the_two_lvm2_checksums_agree_without_sharing_code() {
    // A nibble table against a bit loop over the same polynomial. On its own
    // this proves only that they agree; the pinned value above is what ties the
    // pair to something outside this repository.
    for length in 0..64_usize {
        let data: Vec<u8> = (0..length).map(|index| spread(index, 91, 7)).collect();
        assert_eq!(
            super::lvm2_crc_bitwise(&data),
            crate::signature::lvm2_crc(&data),
            "disagreement at length {length}"
        );
    }
}

#[test]
fn the_two_mdraid_checksums_agree_without_sharing_code() {
    let mut superblock = [0_u8; 256];
    for (index, byte) in superblock.iter_mut().enumerate() {
        *byte = spread(index, 13, 5);
    }
    let expected = crate::signature::mdraid_1x_checksum(&superblock);
    superblock[216..220].copy_from_slice(&0_u32.to_le_bytes());
    assert_eq!(super::folded_word_sum(&superblock), expected);
}

/// Spread an index over the whole byte range, so the agreement tests above
/// exercise every bit position rather than a run of small values.
fn spread(index: usize, stride: usize, offset: usize) -> u8 {
    u8::try_from((index * stride + offset) % 256).expect("a value modulo 256 fits a byte")
}

// --- Mutation helpers ----------------------------------------------------
//
// A mutation that leaves a checksum wrong proves only that the checksum check
// works. These repair what they disturb, so what changes is the table's
// *meaning* — which is how a fixture stops serving its purpose while still
// looking well-formed.

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(raw)
}

/// Byte offset of the entry array a header at `header_at` points to.
fn array_start(bytes: &[u8], header_at: usize, sector: usize) -> usize {
    usize::try_from(read_u64(bytes, header_at + 72)).expect("entry LBA fits") * sector
}

/// Recompute a GPT header's entry-array CRC and then its own CRC.
fn repair_gpt(bytes: &mut [u8], header_at: usize, sector: usize) {
    let start = array_start(bytes, header_at, sector);
    let count = usize::try_from(read_u32(bytes, header_at + 80)).expect("count fits");
    let size = usize::try_from(read_u32(bytes, header_at + 84)).expect("size fits");
    let array_crc = super::crc32_table(&bytes[start..start + count * size]);
    bytes[header_at + 88..header_at + 92].copy_from_slice(&array_crc.to_le_bytes());

    let mut header = bytes[header_at..header_at + 92].to_vec();
    header[16..20].copy_from_slice(&0_u32.to_le_bytes());
    let crc = super::crc32_table(&header);
    bytes[header_at + 16..header_at + 20].copy_from_slice(&crc.to_le_bytes());
}

/// Edit both copies of the entry array, then repair both headers.
///
/// Editing one copy alone would make the two disagree, and the "copies describe
/// different partitions" check would fire instead of the one under test.
fn edit_both_arrays(bytes: &mut [u8], sector: usize, edit: impl Fn(&mut [u8])) {
    let headers = [sector, bytes.len() - sector];
    for header_at in headers {
        let start = array_start(bytes, header_at, sector);
        let count = usize::try_from(read_u32(bytes, header_at + 80)).expect("count fits");
        let size = usize::try_from(read_u32(bytes, header_at + 84)).expect("size fits");
        edit(&mut bytes[start..start + count * size]);
    }
    for header_at in headers {
        repair_gpt(bytes, header_at, sector);
    }
}

fn repair_lvm2(bytes: &mut [u8]) {
    let crc = super::lvm2_crc_bitwise(&bytes[532..1024]);
    bytes[528..532].copy_from_slice(&crc.to_le_bytes());
}

fn repair_mdraid_12(bytes: &mut [u8]) {
    let mut superblock = [0_u8; 256];
    superblock.copy_from_slice(&bytes[4096..4352]);
    superblock[216..220].copy_from_slice(&0_u32.to_le_bytes());
    let csum = super::folded_word_sum(&superblock);
    bytes[4312..4316].copy_from_slice(&csum.to_le_bytes());
}
