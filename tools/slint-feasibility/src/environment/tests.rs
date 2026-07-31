use super::{
    Access, ENVIRONMENT_INVENTORY, discover_accesses, parse_inventory, rust_string_literals,
};

// Requirements: SEC-010
//   The source-derived inventory is closed, rationale-bearing, rerun-complete, and distinguishes upstream-created state from rejected ambient input
// Evidence: committed_environment_inventory_is_closed_and_classified
#[test]
fn committed_environment_inventory_is_closed_and_classified() {
    let inventory = parse_inventory(ENVIRONMENT_INVENTORY.as_bytes()).expect("inventory parses");
    assert!(inventory.entries.iter().any(|entry| {
        entry.name == "SLINT_WIDGETS_LIBRARY"
            && entry.classification == "upstream-controlled"
            && !entry.rerun_input
    }));
    assert!(inventory.entries.iter().any(|entry| {
        entry.name == "DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES" && entry.rerun_input
    }));
    assert!(
        inventory
            .entries
            .iter()
            .all(|entry| !entry.rationale.is_empty())
    );
    let unknown = ENVIRONMENT_INVENTORY.replacen("runtime-rejected", "implementation-says-safe", 1);
    assert!(parse_inventory(unknown.as_bytes()).is_err());
}

// Requirements: SEC-010
//   The source scanner recognizes actual runtime, compile-time, and Cargo-written environment access while ignoring comments and unrelated diagnostic strings
// Evidence: source_scanner_distinguishes_environment_access_from_mentions
#[test]
fn source_scanner_distinguishes_environment_access_from_mentions() {
    let source = r##"
        // std::env::var("SLINT_COMMENT")
        let _ = std::env::var("SLINT_RUNTIME");
        let _ = option_env!("SLINT_COMPILE");
        let quote = '"';
        let apostrophe = '\'';
        let _ = std::env::var("SLINT_AFTER_CHARS");
        println!("cargo:rustc-env=SLINT_UPSTREAM={}", output);
        let _ = "Set SLINT_DIAGNOSTIC=1 to debug";
        let _ = r#"std::env::var(\"SLINT_RAW_TEXT\")"#;
    "##;
    let actual = discover_accesses(source).expect("source scans");
    assert!(actual.contains(&("SLINT_RUNTIME".to_owned(), Access::RuntimeRead)));
    assert!(actual.contains(&("SLINT_COMPILE".to_owned(), Access::CompileTimeRead)));
    assert!(actual.contains(&("SLINT_UPSTREAM".to_owned(), Access::CargoRustcEnvWrite)));
    assert!(actual.contains(&("SLINT_AFTER_CHARS".to_owned(), Access::RuntimeRead)));
    assert_eq!(actual.len(), 4);
}

// Requirements: SEC-010
//   Malformed comments and strings are rejected rather than causing a partial environment inventory
// Evidence: source_scanner_fails_closed_on_lexical_truncation
#[test]
fn source_scanner_fails_closed_on_lexical_truncation() {
    assert!(rust_string_literals("/* unterminated").is_err());
    assert!(rust_string_literals("let x = \"unterminated").is_err());
    assert!(rust_string_literals("let x = r###\"unterminated").is_err());
}
