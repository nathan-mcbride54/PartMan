use std::io::Cursor;

use super::{
    CARGO_BANNER, CARGO_COMMIT_DATE, CARGO_COMMIT_HASH, CARGO_RELEASE, STDERR_LIMIT, drain_bounded,
    verify_cargo_identity_output,
};

#[cfg(unix)]
use super::trusted_cargo_path_against;

// Requirements: SAFE-004, SEC-010
//   External metadata output is drained without unbounded retention and overflow is an explicit failure signal
// Work-Package: WP-030
// Evidence: bounded_reader_drains_and_reports_overflow
#[test]
fn bounded_reader_drains_and_reports_overflow() {
    let bytes = vec![b'x'; STDERR_LIMIT + 17];
    let (retained, overflow) =
        drain_bounded(Cursor::new(bytes), STDERR_LIMIT).expect("reader works");
    assert_eq!(retained.len(), STDERR_LIMIT);
    assert!(overflow);
}

// Requirements: SAFE-004, SEC-010
//   A rustup-style cargo symlink retains its cargo basename so proxy dispatch is not changed by validation
// Work-Package: WP-030
// Evidence: cargo_proxy_path_is_not_canonicalized
#[cfg(unix)]
#[test]
fn cargo_proxy_path_is_not_canonicalized() {
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "partman-slint-feasibility-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("fixture directory is created");
    let rustup = root.join("rustup");
    std::fs::write(&rustup, b"proxy").expect("proxy fixture is created");
    let cargo = root.join("cargo");
    symlink(&rustup, &cargo).expect("cargo proxy symlink is created");

    let validated =
        trusted_cargo_path_against(&cargo, &cargo).expect("selected cargo proxy is accepted");
    assert_eq!(validated, cargo);

    std::fs::remove_dir_all(root).expect("fixture directory is removed");
}

// Requirements: SAFE-004, SEC-010
//   Live metadata replay accepts only the pinned Cargo release and full commit identity, with no duplicate identity fields
// Work-Package: WP-030
// Evidence: cargo_identity_fails_closed_on_every_pinned_field
#[test]
fn cargo_identity_fails_closed_on_every_pinned_field() {
    let clean = format!(
        "{CARGO_BANNER}\nrelease: {CARGO_RELEASE}\ncommit-hash: {CARGO_COMMIT_HASH}\ncommit-date: {CARGO_COMMIT_DATE}\nhost: test-host\n"
    );
    verify_cargo_identity_output(clean.as_bytes()).expect("exact Cargo identity passes");

    for drifted in [
        clean.replacen(CARGO_BANNER, "cargo 1.96.1 (substituted)", 1),
        clean.replacen(CARGO_RELEASE, "1.96.1", 1),
        clean.replacen(
            CARGO_COMMIT_HASH,
            "0000000000000000000000000000000000000000",
            1,
        ),
        clean.replacen(CARGO_COMMIT_DATE, "2026-05-26", 1),
        clean.replace(
            &format!("release: {CARGO_RELEASE}\n"),
            &format!("release: {CARGO_RELEASE}\nrelease: {CARGO_RELEASE}\n"),
        ),
    ] {
        assert!(
            verify_cargo_identity_output(drifted.as_bytes()).is_err(),
            "mutated Cargo identity must fail"
        );
    }
}
