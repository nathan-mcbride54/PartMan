//! These run everywhere, including on Windows where no prober exists.
//!
//! They cannot check what `libblkid` says — that is `cargo xtask probe`, and it
//! needs Linux. What they can check is that the recorded table stays exhaustive,
//! that the parsers handle the real output shapes, and that the comparison is
//! capable of failing. The last is the point: an expectation table compared by a
//! function that never disagrees would be the same defect `evidence` was written
//! to end, one layer out.

use std::collections::BTreeSet;

use super::{Observation, compare, expectations, parse_udev, parse_wipefs};
use crate::catalogue::catalogue;

#[test]
fn every_fixture_has_a_recorded_prober_expectation() {
    // Exhaustive in both directions, for the same reason the claims are: a
    // fixture added without one would silently stop being checked against any
    // real tool, and that is how a fixture nothing recognizes gets shipped.
    let fixtures: BTreeSet<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
    let recorded: BTreeSet<&str> = expectations()
        .iter()
        .map(|expectation| expectation.fixture)
        .collect();

    let unrecorded: Vec<&&str> = fixtures.difference(&recorded).collect();
    assert!(
        unrecorded.is_empty(),
        "these fixtures have no recorded prober output: {unrecorded:?}"
    );
    let orphaned: Vec<&&str> = recorded.difference(&fixtures).collect();
    assert!(
        orphaned.is_empty(),
        "these expectations name no fixture in the catalogue: {orphaned:?}"
    );
}

#[test]
fn every_expectation_records_why_it_is_what_it_is() {
    // Three of these rows are not what a reader would predict from the
    // fixture's name — the 4Kn image reported as PMBR, the conflicting tables
    // reported as an ordinary GPT, the stale mdraid winning over the live ext4.
    // A row without a reason invites someone to "fix" the fixture.
    for expectation in expectations() {
        assert!(
            expectation.note.len() > 40,
            "{} needs a real note",
            expectation.fixture
        );
    }
}

#[test]
fn the_conflicting_and_healthy_tables_are_recorded_as_indistinguishable() {
    // SI-35 rests on this equality, so state it as an assertion rather than
    // leaving it implicit in two rows that happen to match. If libblkid ever
    // separates them, this fails and SI-35 narrows.
    let all = expectations();
    let healthy = all
        .iter()
        .find(|e| e.fixture == "gpt-basic-512.img")
        .expect("the baseline must be recorded");
    let conflicting = all
        .iter()
        .find(|e| e.fixture == "gpt-conflicting-tables-512.img")
        .expect("the ambiguous fixture must be recorded");

    assert_eq!(healthy.part_table_type, conflicting.part_table_type);
    assert_eq!(healthy.fs_type, conflicting.fs_type);
    assert_eq!(
        healthy.signatures, conflicting.signatures,
        "ADR-C3's Present and Indeterminate are recorded as producing identical prober output; \
         if that stops being true, SI-35 changes"
    );
}

#[test]
fn the_stale_signature_is_the_one_the_single_answer_interface_reports() {
    // The asymmetry the protection model turns on, asserted rather than left in
    // prose: wipefs enumerates both, and the udev-shaped answer is the stale
    // array membership, not the live file system.
    let all = expectations();
    let stale = all
        .iter()
        .find(|e| e.fixture == "ext4-with-stale-mdraid-090-512.img")
        .expect("the multi-signature fixture must be recorded");

    assert_eq!(
        stale.fs_type,
        Some("linux_raid_member"),
        "the single-answer interface must report the stale signature"
    );
    let kinds: Vec<&str> = stale.signatures.iter().map(|(_, kind)| *kind).collect();
    assert!(
        kinds.contains(&"ext4") && kinds.contains(&"linux_raid_member"),
        "the enumerating interface must report both: {kinds:?}"
    );
}

#[test]
fn the_parsers_read_the_shapes_the_tools_actually_emit() {
    // Captured verbatim from libblkid 2.41 rather than invented, including the
    // escaped space in an LVM version string and the header row wipefs prints
    // when it has anything to say.
    let udev = parse_udev(
        "ID_FS_UUID=pvuuid-0000-0000-0000-0000-0000-000000\n\
         ID_FS_VERSION=LVM2\\x20001\n\
         ID_FS_TYPE=LVM2_member\n\
         ID_FS_USAGE=raid\n",
    );
    assert_eq!(
        udev.get("ID_FS_TYPE").map(String::as_str),
        Some("LVM2_member")
    );
    assert_eq!(udev.len(), 4);

    let wipefs = parse_wipefs(
        "OFFSET   TYPE\n\
         0x3f0000 linux_raid_member\n\
         0x438    ext4\n",
    );
    assert_eq!(
        wipefs,
        [
            (0x438, "ext4".to_owned()),
            (0x003f_0000, "linux_raid_member".to_owned())
        ]
        .into_iter()
        .collect()
    );

    // Nothing detected: blkid prints nothing and exits 2, wipefs prints nothing
    // at all — not even a header. Both must parse to empty rather than panic.
    assert!(parse_udev("").is_empty());
    assert!(parse_wipefs("").is_empty());
}

#[test]
fn a_matching_observation_produces_no_disagreements() {
    let all = expectations();
    let expected = all
        .iter()
        .find(|e| e.fixture == "luks2-whole-disk-512.img")
        .expect("recorded");
    assert!(compare(expected, &observation_of(expected)).is_empty());
}

#[test]
fn the_comparison_is_capable_of_failing_in_every_direction() {
    // The test that makes the table mean something. A comparison that never
    // disagrees would leave `cargo xtask probe` reporting success over any
    // output at all, which is the fake-success path Section 12 forbids.
    let all = expectations();
    let expected = all
        .iter()
        .find(|e| e.fixture == "ext4-with-stale-mdraid-090-512.img")
        .expect("recorded");

    // A format that is no longer detected.
    let mut lost = observation_of(expected);
    lost.udev.remove("ID_FS_TYPE");
    assert!(
        compare(expected, &lost)
            .iter()
            .any(|reason| reason.contains("reported nothing")),
        "a fixture becoming undetectable must be a disagreement"
    );

    // The live file system winning instead of the stale membership, which is
    // the specific reversal Part 5's conclusion would turn on.
    let mut reversed = observation_of(expected);
    reversed
        .udev
        .insert("ID_FS_TYPE".to_owned(), "ext4".to_owned());
    assert!(
        compare(expected, &reversed)
            .iter()
            .any(|reason| reason.contains("prober said \"ext4\"")),
        "a changed answer must be a disagreement"
    );

    // A signature that vanished.
    let mut missing = observation_of(expected);
    missing
        .signatures
        .retain(|(_, kind)| kind != "linux_raid_member");
    assert!(
        compare(expected, &missing)
            .iter()
            .any(|reason| reason.contains("is not there")),
        "a lost signature must be a disagreement"
    );

    // And one that appeared. A fixture gaining a signature is as much a
    // regression as losing one, and only checking one direction would miss it.
    let mut extra = observation_of(expected);
    extra.signatures.insert((0x1000, "swap".to_owned()));
    assert!(
        compare(expected, &extra)
            .iter()
            .any(|reason| reason.contains("unexpected swap")),
        "an added signature must be a disagreement"
    );
}

/// A verbatim capture of `blkid -p -o udev` and `wipefs -n --output OFFSET,TYPE`
/// over the whole catalogue.
///
/// Recorded 2026-07-28 from `libblkid` 2.41.0 (util-linux 2.41, 18 March 2025)
/// on Debian, probing the generated images as regular files. Pasted rather than
/// summarized, escapes and column padding included, so the parsers are tested
/// against what the tools emit rather than against a tidied version of it.
///
/// This is what lets the table below be checked on a machine with no prober.
/// `cargo xtask probe` runs the tools live, and CI runs that on Linux; this
/// keeps the recorded expectations honest everywhere else, and would catch a
/// transcription error between the capture and the table.
const CAPTURE: &str = "\
FIXTURE apm-basic-512.img
UDEV ID_PART_TABLE_TYPE=mac
WIPE 0x0    mac
FIXTURE blank-512.img
FIXTURE ext4-with-stale-mdraid-090-512.img
UDEV ID_FS_VERSION=0.90.0
UDEV ID_FS_UUID=fb2871eb-405c-788b-e2c6-fb8cfe3b5444
UDEV ID_FS_UUID_ENC=fb2871eb-405c-788b-e2c6-fb8cfe3b5444
UDEV ID_FS_TYPE=linux_raid_member
UDEV ID_FS_USAGE=raid
WIPE 0x3f0000 linux_raid_member
WIPE 0x438    ext4
FIXTURE gpt-basic-4kn.img
UDEV ID_PART_TABLE_TYPE=PMBR
WIPE 0x1fe  PMBR
FIXTURE gpt-basic-512.img
UDEV ID_PART_TABLE_UUID=7a1e9153-bef6-4752-9460-8c23898f2cbf
UDEV ID_PART_TABLE_TYPE=gpt
WIPE 0x200    gpt
WIPE 0x3ffe00 gpt
WIPE 0x1fe    PMBR
FIXTURE gpt-conflicting-tables-512.img
UDEV ID_PART_TABLE_UUID=7a1e9153-bef6-4752-9460-8c23898f2cbf
UDEV ID_PART_TABLE_TYPE=gpt
WIPE 0x200    gpt
WIPE 0x3ffe00 gpt
WIPE 0x1fe    PMBR
FIXTURE gpt-invalid-primary-valid-backup-512.img
UDEV ID_PART_TABLE_UUID=7a1e9153-bef6-4752-9460-8c23898f2cbf
UDEV ID_PART_TABLE_TYPE=gpt
WIPE 0x3ffe00 gpt
WIPE 0x1fe    PMBR
FIXTURE gpt-missing-backup-512.img
UDEV ID_PART_TABLE_UUID=7a1e9153-bef6-4752-9460-8c23898f2cbf
UDEV ID_PART_TABLE_TYPE=gpt
WIPE 0x200  gpt
WIPE 0x1fe  PMBR
FIXTURE hybrid-mbr-gpt-512.img
UDEV ID_PART_TABLE_UUID=7a1e9153-bef6-4752-9460-8c23898f2cbf
UDEV ID_PART_TABLE_TYPE=gpt
WIPE 0x200    gpt
WIPE 0x3ffe00 gpt
WIPE 0x1fe    PMBR
FIXTURE luks2-whole-disk-512.img
UDEV ID_FS_VERSION=2
UDEV ID_FS_UUID=5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f
UDEV ID_FS_UUID_ENC=5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f
UDEV ID_FS_TYPE=crypto_LUKS
UDEV ID_FS_USAGE=crypto
WIPE 0x0    crypto_LUKS
FIXTURE lvm2-pv-orphan-512.img
UDEV ID_FS_UUID=pvuuid-0000-0000-0000-0000-0000-000000
UDEV ID_FS_UUID_ENC=pvuuid-0000-0000-0000-0000-0000-000000
UDEV ID_FS_VERSION=LVM2\\x20001
UDEV ID_FS_TYPE=LVM2_member
UDEV ID_FS_USAGE=raid
WIPE 0x218  LVM2_member
FIXTURE mbr-basic-512.img
UDEV ID_PART_TABLE_TYPE=dos
WIPE 0x1fe  dos
FIXTURE mdraid-1.2-member-512.img
UDEV ID_FS_UUID=62fc041a-4333-f945-a326-3e563a464412
UDEV ID_FS_UUID_ENC=62fc041a-4333-f945-a326-3e563a464412
UDEV ID_FS_LABEL=pm:0
UDEV ID_FS_LABEL_ENC=pm:0
UDEV ID_FS_VERSION=1.2
UDEV ID_FS_TYPE=linux_raid_member
UDEV ID_FS_USAGE=raid
WIPE 0x1000 linux_raid_member
";

/// Split the capture into one observation per fixture.
fn captured() -> std::collections::BTreeMap<String, Observation> {
    let mut all = std::collections::BTreeMap::new();
    let mut name = String::new();
    let mut udev = String::new();
    let mut wipefs = String::new();

    let mut flush = |name: &mut String, udev: &mut String, wipefs: &mut String| {
        if !name.is_empty() {
            all.insert(
                std::mem::take(name),
                Observation {
                    udev: parse_udev(udev),
                    signatures: parse_wipefs(wipefs),
                },
            );
        }
        udev.clear();
        wipefs.clear();
    };

    for line in CAPTURE.lines() {
        if let Some(fixture) = line.strip_prefix("FIXTURE ") {
            flush(&mut name, &mut udev, &mut wipefs);
            name = fixture.to_owned();
        } else if let Some(rest) = line.strip_prefix("UDEV ") {
            udev.push_str(rest);
            udev.push('\n');
        } else if let Some(rest) = line.strip_prefix("WIPE ") {
            wipefs.push_str(rest);
            wipefs.push('\n');
        }
    }
    flush(&mut name, &mut udev, &mut wipefs);
    all
}

#[test]
fn the_recorded_table_matches_a_real_probe_run() {
    // The check that keeps the table tied to a tool rather than to an opinion.
    // Without it, `expectations()` would be a hand-written list that only a
    // Linux CI job ever compared against reality — and a transcription slip
    // between the capture and the table would look exactly like a passing test.
    let observed = captured();
    for expectation in expectations() {
        let observation = observed
            .get(expectation.fixture)
            .unwrap_or_else(|| panic!("{} is missing from the capture", expectation.fixture));
        let disagreements = compare(&expectation, observation);
        assert!(
            disagreements.is_empty(),
            "{} does not match the captured libblkid 2.41 output: {disagreements:?}",
            expectation.fixture
        );
    }
    assert_eq!(
        observed.len(),
        expectations().len(),
        "the capture and the table must cover the same fixtures"
    );
}

#[test]
fn the_capture_shows_both_signatures_where_only_one_is_reported() {
    // Stated against the raw capture rather than the table, so the asymmetry
    // rests on tool output and not on how it was transcribed.
    let observed = captured();
    let stale = &observed["ext4-with-stale-mdraid-090-512.img"];

    assert_eq!(
        stale.udev.get("ID_FS_TYPE").map(String::as_str),
        Some("linux_raid_member"),
        "blkid reports exactly one type, and it is the stale one"
    );
    assert!(
        !stale.udev.contains_key("ID_FS_AMBIVALENT"),
        "the client is not even told the device is ambiguous"
    );
    assert_eq!(stale.signatures.len(), 2, "wipefs enumerates both");
}

/// Build the observation an expectation describes, so the comparison can be
/// tested without a prober.
fn observation_of(expected: &super::ProberExpectation) -> Observation {
    let mut udev = std::collections::BTreeMap::new();
    for (key, value) in [
        ("ID_FS_TYPE", expected.fs_type),
        ("ID_FS_UUID", expected.fs_uuid),
        ("ID_FS_LABEL", expected.fs_label),
        ("ID_PART_TABLE_TYPE", expected.part_table_type),
    ] {
        if let Some(value) = value {
            udev.insert(key.to_owned(), value.to_owned());
        }
    }
    Observation {
        udev,
        signatures: expected
            .signatures
            .iter()
            .map(|(offset, kind)| (*offset, (*kind).to_owned()))
            .collect(),
    }
}
