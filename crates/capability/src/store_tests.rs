//! The CAP-006 store's CI gate (WP-050 increment 3): the Tier-1 test
//! that holds `docs/capabilities/` to its schema, in the
//! `shared_vectors` pattern — repository data validated by a test that
//! fails the build.

use std::path::PathBuf;

use serde_json::Value as Json;

use super::store::{
    FLOORS_SCHEMA, FLOORS_SCHEMA_VERSION, OPERATION_NAMES, PLATFORM_LABELS, STORE_SCHEMA,
    STORE_SCHEMA_VERSION,
};

fn capabilities_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/capabilities")
}

fn load(name: &str) -> Json {
    let path = capabilities_dir().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} must parse as JSON: {error}", path.display()))
}

fn fs_kind_tags() -> &'static [&'static str] {
    // The §3a tags of schemas/domain/node-entry-format.md, the store's
    // declared vocabulary for `file_system`.
    &[
        "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "fat12", "fat16", "fat32", "exfat", "ntfs",
        "refs", "hfsplus", "apfs", "udf", "swap",
    ]
}

// Requirements: CAP-006, MODEL-003
//   The qualification store parses, declares exactly its schema and
//   version, carries no unknown top-level field, and every advertised
//   row is well-formed over the declared vocabularies: a Section 9
//   platform label, a §3a file-system tag, a CAP-002 operation name,
//   and a state from the closed pair.
// Evidence: the_qualification_store_satisfies_its_schema
#[test]
fn the_qualification_store_satisfies_its_schema() {
    let store = load("qualifications.json");
    let object = store.as_object().expect("store is an object");
    assert_eq!(
        object.get("schema").and_then(Json::as_str),
        Some(STORE_SCHEMA)
    );
    assert_eq!(
        object.get("schema_version").and_then(Json::as_u64),
        Some(STORE_SCHEMA_VERSION)
    );
    for key in object.keys() {
        assert!(
            matches!(key.as_str(), "schema" | "schema_version" | "advertised"),
            "unknown top-level field {key}"
        );
    }
    let rows = object
        .get("advertised")
        .and_then(Json::as_array)
        .expect("advertised is a list");
    for row in rows {
        let row = row.as_object().expect("row is an object");
        for key in row.keys() {
            assert!(
                matches!(
                    key.as_str(),
                    "platform" | "file_system" | "operation" | "state" | "evidence"
                ),
                "unknown row field {key}"
            );
        }
        let platform = row
            .get("platform")
            .and_then(Json::as_str)
            .expect("platform");
        assert!(
            PLATFORM_LABELS.contains(&platform),
            "unknown platform {platform}"
        );
        let file_system = row
            .get("file_system")
            .and_then(Json::as_str)
            .expect("file_system");
        assert!(
            fs_kind_tags().contains(&file_system),
            "unknown file system {file_system}"
        );
        let operation = row
            .get("operation")
            .and_then(Json::as_str)
            .expect("operation");
        assert!(
            OPERATION_NAMES.contains(&operation),
            "unknown operation {operation}"
        );
        match row.get("state").and_then(Json::as_str).expect("state") {
            "unqualified" => {
                assert!(
                    !row.contains_key("evidence"),
                    "an unqualified row carries no evidence fields"
                );
            }
            "qualified" => {
                let evidence = row
                    .get("evidence")
                    .and_then(Json::as_object)
                    .expect("a qualified row carries an evidence object");
                for field in ["fixture", "run", "date", "transcript_sha256"] {
                    assert!(
                        evidence.get(field).and_then(Json::as_str).is_some(),
                        "qualified evidence must carry {field}"
                    );
                }
            }
            other => panic!("state must be unqualified or qualified, not {other}"),
        }
    }
}

// Requirements: CAP-006, CAP-003
//   Qualification cannot arrive silently: the number of qualified rows
//   is pinned here, at zero — the truthful count while no apply path
//   exists anywhere in the product — so the diff that first qualifies a
//   combination must also move this expectation, under review. This is
//   the store half of the increment-1 rule that `supported` is
//   unreachable without evidence.
// Evidence: no_row_is_silently_qualified
#[test]
fn no_row_is_silently_qualified() {
    let store = load("qualifications.json");
    let qualified = store["advertised"]
        .as_array()
        .expect("advertised is a list")
        .iter()
        .filter(|row| row["state"].as_str() == Some("qualified"))
        .count();
    assert_eq!(
        qualified, 0,
        "a qualification is a reviewed act that moves this pinned count"
    );
}

// Requirements: CAP-006, MODEL-003
//   The floors document parses, declares exactly its schema and
//   version, and every floor entry names a tool, a floor, and its
//   basis. The list is empty and that is the truthful state: no
//   storage tool is invoked anywhere in the product, and a floor for a
//   tool nobody calls would be an assertion nobody can test.
// Evidence: the_floors_document_satisfies_its_schema
#[test]
fn the_floors_document_satisfies_its_schema() {
    let floors = load("tool-version-floors.json");
    let object = floors.as_object().expect("floors is an object");
    assert_eq!(
        object.get("schema").and_then(Json::as_str),
        Some(FLOORS_SCHEMA)
    );
    assert_eq!(
        object.get("schema_version").and_then(Json::as_u64),
        Some(FLOORS_SCHEMA_VERSION)
    );
    for key in object.keys() {
        assert!(
            matches!(key.as_str(), "schema" | "schema_version" | "floors"),
            "unknown top-level field {key}"
        );
    }
    for entry in object
        .get("floors")
        .and_then(Json::as_array)
        .expect("floors is a list")
    {
        let entry = entry.as_object().expect("floor entry is an object");
        for field in ["tool", "floor", "basis"] {
            assert!(
                entry.get(field).and_then(Json::as_str).is_some(),
                "a floor entry must carry {field}"
            );
        }
        for key in entry.keys() {
            assert!(
                matches!(key.as_str(), "tool" | "floor" | "basis"),
                "unknown floor field {key}"
            );
        }
    }
}
