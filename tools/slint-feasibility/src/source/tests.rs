use std::fs;

use super::{
    SOURCE_INVENTORY, hex_digest, normalized_relative_path, parse_inventory, published_tree_hash,
};
use sha2::{Digest, Sha256};

fn temporary_directory(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "partman-slint-feasibility-{name}-{}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("stale test directory is removable");
    }
    fs::create_dir_all(&path).expect("test directory is creatable");
    path
}

// Requirements: SEC-010
//   The compiler source inventory is strict, exact, carries the reviewed nine-file compiler boundary, and enumerates every reachable runtime licence package rather than accepting implementation-owned additions
// Evidence: committed_source_inventory_is_strict_and_complete
#[test]
fn committed_source_inventory_is_strict_and_complete() {
    let inventory = parse_inventory(SOURCE_INVENTORY.as_bytes()).expect("inventory is valid");
    assert_eq!(inventory.critical_files.len(), 9);
    assert_eq!(inventory.license_packages.len(), 10);
    assert_eq!(
        inventory.tree_hash,
        "85107306da880f388216602768b62c92de8b705ff49d436e82d233235630499c"
    );
    let forged = SOURCE_INVENTORY.replacen(
        "\"schemaVersion\": 1",
        "\"schemaVersion\": 1, \"pass\": true",
        1,
    );
    assert!(parse_inventory(forged.as_bytes()).is_err());
    let forged_license = SOURCE_INVENTORY.replacen(
        "GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0",
        "GPL-3.0-only",
        1,
    );
    assert!(parse_inventory(forged_license.as_bytes()).is_err());
}

// Requirements: SEC-010
//   Published-tree hashing uses the reviewed u64-big-endian length framing, ordinal path ordering, raw bytes, and excludes only root .cargo-ok
// Evidence: tree_hash_framing_is_explicit_and_order_independent
#[test]
fn tree_hash_framing_is_explicit_and_order_independent() {
    let root = temporary_directory("framing");
    fs::create_dir(root.join("nested")).expect("nested directory is creatable");
    fs::write(root.join("z"), b"last").expect("fixture is writable");
    fs::write(root.join("nested").join("a"), [0_u8, 0xff]).expect("fixture is writable");
    fs::write(root.join(".cargo-ok"), b"ignored only at root").expect("fixture is writable");

    let (actual, count) = published_tree_hash(&root).expect("fixture tree hashes");
    let mut expected = Sha256::new();
    for (path, content) in [("nested/a", &[0_u8, 0xff][..]), ("z", &b"last"[..])] {
        expected.update(
            u64::try_from(path.len())
                .expect("path length fits")
                .to_be_bytes(),
        );
        expected.update(path.as_bytes());
        expected.update(
            u64::try_from(content.len())
                .expect("content length fits")
                .to_be_bytes(),
        );
        expected.update(content);
    }
    assert_eq!(actual, hex_digest(expected.finalize()));
    assert_eq!(count, 2);
    fs::remove_dir_all(root).expect("test directory is removable");
}

// Requirements: SEC-010
//   Registry source paths are normalized beneath the selected package root and refuse parent or non-component ambiguity
// Evidence: source_path_normalization_is_contained
#[test]
fn source_path_normalization_is_contained() {
    let root = std::path::Path::new("root");
    assert_eq!(
        normalized_relative_path(root, &root.join("a").join("b")).expect("child path"),
        "a/b"
    );
    assert!(normalized_relative_path(root, std::path::Path::new("outside")).is_err());
}
