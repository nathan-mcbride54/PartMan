use std::collections::BTreeSet;
use std::fs;

use super::{Fixture, catalogue, generate, generate_from, load_manifest};
use crate::layout::{Image, SectorSize};
use crate::manifest::MANIFEST_FILE;

struct Sandbox(std::path::PathBuf);

impl Sandbox {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("partman-catalogue-{tag}"));
        let _ = fs::remove_dir_all(&root);
        Self(root)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn every_fixture_has_a_unique_name_and_a_stated_reason() {
    // Section 11.7 fails a work package that claims a requirement without linked
    // evidence. A fixture nobody can name a requirement for is unmaintainable.
    let names: BTreeSet<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
    assert_eq!(
        names.len(),
        catalogue().len(),
        "fixture names must be unique"
    );

    for fixture in catalogue() {
        assert!(
            fixture.rationale.len() > 20,
            "{} needs a real rationale",
            fixture.name
        );
        assert_eq!(
            std::path::Path::new(fixture.name).extension(),
            Some(std::ffi::OsStr::new("img")),
            "{} must be named as an image",
            fixture.name
        );
    }
}

/// The partition-table state ADR-C3 distinguishes.
#[derive(Debug, PartialEq, Eq)]
enum TableState {
    /// Positively observed to have no table.
    Absent,
    /// A table was read and can be trusted.
    Present,
    /// Unreadable, or it parses ambiguously.
    Indeterminate,
}

/// Classify a GPT image's table state by reading its bytes.
///
/// A deliberately independent oracle: it recomputes both header checksums from
/// the image rather than asking the code that wrote them, so a writer bug cannot
/// make this agree with itself. The previous version of this test asserted only
/// that three *filenames* existed, which is why a fixture that was not
/// indeterminate went unnoticed while its traceability record claimed it was.
fn classify(bytes: &[u8]) -> TableState {
    let last = bytes.len() - 512;
    let primary_ok = header_is_valid(bytes, 512);
    let backup_ok = header_is_valid(bytes, last);
    let primary_present = &bytes[512..520] == b"EFI PART";
    let backup_present = &bytes[last..last + 8] == b"EFI PART";

    match (primary_present, backup_present) {
        (false, false) => TableState::Absent,
        _ if !primary_ok && !backup_ok => TableState::Indeterminate,
        (true, true) if primary_ok && backup_ok => {
            // Both trustworthy: they must agree, or nothing can be determined.
            let primary_array_crc = u32::from_le_bytes([
                bytes[512 + 88],
                bytes[512 + 89],
                bytes[512 + 90],
                bytes[512 + 91],
            ]);
            let backup_array_crc = u32::from_le_bytes([
                bytes[last + 88],
                bytes[last + 89],
                bytes[last + 90],
                bytes[last + 91],
            ]);
            if primary_array_crc == backup_array_crc {
                TableState::Present
            } else {
                TableState::Indeterminate
            }
        }
        // Exactly one copy is trustworthy, so the table is still determinable.
        _ => TableState::Present,
    }
}

fn header_is_valid(bytes: &[u8], offset: usize) -> bool {
    if offset + 92 > bytes.len() || &bytes[offset..offset + 8] != b"EFI PART" {
        return false;
    }
    let mut header = bytes[offset..offset + 92].to_vec();
    let stored = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
    header[16..20].copy_from_slice(&0_u32.to_le_bytes());
    crate::layout::crc32(&header) == stored
}

#[test]
fn the_fixtures_actually_classify_as_the_states_adr_c3_distinguishes() {
    // Blank, present, and indeterminate are three states, not two, and ADR-C3
    // turns on telling them apart. This classifies the bytes; it does not check
    // that some filenames exist.
    let cases = [
        ("blank-512.img", TableState::Absent),
        ("gpt-basic-512.img", TableState::Present),
        // Damaged but recoverable: the backup still checksums, so the table is
        // positively determinable. This is the correction — it was previously
        // claimed to be the indeterminate case.
        (
            "gpt-invalid-primary-valid-backup-512.img",
            TableState::Present,
        ),
        ("gpt-missing-backup-512.img", TableState::Present),
        // Genuinely ambiguous: two independently valid, disagreeing tables.
        ("gpt-conflicting-tables-512.img", TableState::Indeterminate),
    ];

    for (name, expected) in cases {
        let fixture = catalogue()
            .into_iter()
            .find(|fixture| fixture.name == name)
            .unwrap_or_else(|| panic!("{name} must be in the catalogue"));
        let bytes = (fixture.build)().into_bytes();
        assert_eq!(classify(&bytes), expected, "{name}");
    }
}

#[test]
fn the_recoverable_and_indeterminate_fixtures_are_genuinely_different() {
    // The distinction the previous test could not see, stated directly: one has
    // a trustworthy copy left and the other does not.
    let recoverable = build("gpt-invalid-primary-valid-backup-512.img");
    let ambiguous = build("gpt-conflicting-tables-512.img");

    assert!(
        !header_is_valid(&recoverable, 512),
        "the recoverable fixture's primary must be damaged"
    );
    assert!(
        header_is_valid(&recoverable, recoverable.len() - 512),
        "and its backup must still checksum, which is what makes it recoverable"
    );

    assert!(
        header_is_valid(&ambiguous, 512) && header_is_valid(&ambiguous, ambiguous.len() - 512),
        "the ambiguous fixture's copies must both checksum"
    );
    assert_ne!(
        &ambiguous[512 + 88..512 + 92],
        &ambiguous[ambiguous.len() - 512 + 88..ambiguous.len() - 512 + 92],
        "and must describe different partition arrays, or they are not in conflict"
    );
}

fn build(name: &str) -> Vec<u8> {
    let fixture = catalogue()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("{name} must be in the catalogue"));
    (fixture.build)().into_bytes()
}

#[test]
fn generation_writes_every_fixture_and_a_manifest() {
    let sandbox = Sandbox::new("write");
    let manifest = generate(&sandbox.0).expect("generation must succeed");

    for fixture in catalogue() {
        let path = sandbox.0.join(fixture.name);
        let bytes = fs::read(&path).expect("fixture must be written");
        assert!(!bytes.is_empty(), "{} must not be empty", fixture.name);
        let entry = manifest
            .entry(fixture.name)
            .expect("manifest must record every fixture");
        assert_eq!(
            entry.length,
            u64::try_from(bytes.len()).expect("length fits"),
            "{} length must match the manifest",
            fixture.name
        );
    }

    assert!(sandbox.0.join(MANIFEST_FILE).is_file());
    let reloaded = load_manifest(&sandbox.0).expect("the manifest must reload");
    assert_eq!(reloaded.token(), manifest.token());
}

#[test]
fn regenerating_reproduces_identical_bytes() {
    // Section 11.3 requires fixtures be deterministic and cached, and Section 16
    // forbids committing them. Both depend on this.
    let first = Sandbox::new("determinism-a");
    let second = Sandbox::new("determinism-b");
    let one = generate(&first.0).expect("generation must succeed");
    let two = generate(&second.0).expect("generation must succeed");

    assert_eq!(one.token(), two.token(), "the token must be reproducible");
    for fixture in catalogue() {
        let a = fs::read(first.0.join(fixture.name)).expect("readable");
        let b = fs::read(second.0.join(fixture.name)).expect("readable");
        assert_eq!(a, b, "{} must regenerate byte-for-byte", fixture.name);
    }
}

#[test]
fn a_missing_manifest_is_an_error_not_an_empty_manifest() {
    let sandbox = Sandbox::new("no-manifest");
    fs::create_dir_all(&sandbox.0).expect("directory must be creatable");
    assert!(
        load_manifest(&sandbox.0).is_err(),
        "an absent manifest must fail closed"
    );
}

#[test]
fn a_corrupt_manifest_is_rejected() {
    let sandbox = Sandbox::new("bad-manifest");
    generate(&sandbox.0).expect("generation must succeed");
    fs::write(sandbox.0.join(MANIFEST_FILE), "garbage").expect("writing must succeed");
    assert!(load_manifest(&sandbox.0).is_err());
}

#[test]
fn regeneration_removes_a_withdrawn_fixture() {
    // Withdrawing the ZFS fixture left its image behind in the working tree,
    // where a future test that enumerated the directory instead of the manifest
    // could have consumed it and claimed coverage that no longer exists.
    let sandbox = Sandbox::new("prune");
    generate(&sandbox.0).expect("first generation must succeed");

    let stale = sandbox.0.join("withdrawn-fixture-512.img");
    fs::write(&stale, b"left over from an earlier catalogue").expect("writing must succeed");
    assert!(stale.is_file());

    generate(&sandbox.0).expect("second generation must succeed");
    assert!(
        !stale.exists(),
        "a withdrawn fixture must not survive regeneration"
    );

    // And what remains is exactly the manifest plus the current catalogue.
    let present: BTreeSet<String> = fs::read_dir(&sandbox.0)
        .expect("readable")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let mut expected: BTreeSet<String> = catalogue().iter().map(|f| f.name.to_owned()).collect();
    expected.insert(MANIFEST_FILE.to_owned());
    assert_eq!(present, expected);
}

#[test]
fn generation_refuses_a_fixture_that_no_longer_supports_its_rationale() {
    // Until this existed, the evidence gate inside `generate` was load-bearing
    // on nothing: an adversarial pass deleted it and all 74 tests stayed green,
    // because every test fed `generate` the real catalogue, which satisfies its
    // claims. A safety check no test can reach is the defect `evidence` was
    // written to end, sitting inside the fix for it.
    //
    // So this hands `generate` a fixture that keeps a catalogue name and loses
    // what the name promises — the LUKS2 image reduced to zeros, which is the
    // mutation that was actually applied to the catalogue on this branch.
    let sandbox = Sandbox::new("refuse");
    let hollow = vec![Fixture {
        name: "luks2-whole-disk-512.img",
        rationale: "the real entry's rationale, now unsupported by the bytes",
        build: || Image::blank(SectorSize::B512, 8192),
    }];

    let error = generate_from(&sandbox.0, &hollow).expect_err("generation must refuse");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let message = error.to_string();
    assert!(
        message.contains("no LUKS magic"),
        "the refusal must name what was lost: {message}"
    );

    // And nothing was written. Verifying every image before writing any is what
    // keeps a refusal from leaving a half-generated directory behind.
    assert!(
        !sandbox.0.join("luks2-whole-disk-512.img").exists(),
        "a refused fixture must not reach disk"
    );
    assert!(!sandbox.0.join(MANIFEST_FILE).exists());
}

#[test]
fn generation_does_not_prune_a_directory_that_is_not_ours() {
    // The guard on the pruning above: a mistyped root must not delete a user's
    // files. Only a directory already holding one of our manifests is pruned.
    let sandbox = Sandbox::new("not-ours");
    fs::create_dir_all(&sandbox.0).expect("directory must be creatable");
    let bystander = sandbox.0.join("important.txt");
    fs::write(&bystander, b"not a fixture").expect("writing must succeed");

    generate(&sandbox.0).expect("generation must succeed");
    assert!(
        bystander.is_file(),
        "a directory with no manifest of ours must not be pruned"
    );
}
