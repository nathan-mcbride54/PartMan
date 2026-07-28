use std::collections::BTreeSet;
use std::fs;

use super::{catalogue, generate, load_manifest};
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

#[test]
fn the_catalogue_covers_the_states_adr_c3_distinguishes() {
    // Blank, present, and unreadable are three states, not two, and ADR-C3 turns
    // on telling them apart. If a fixture for one disappears, this fails.
    let names: Vec<&str> = catalogue().iter().map(|fixture| fixture.name).collect();
    assert!(names.contains(&"blank-512.img"), "positively absent table");
    assert!(names.contains(&"gpt-basic-512.img"), "present table");
    assert!(
        names.contains(&"gpt-corrupt-header-512.img"),
        "indeterminate table"
    );
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
