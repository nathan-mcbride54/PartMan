use super::{MANIFEST_HEADER, Manifest, ManifestError, hex};

fn sample() -> Manifest {
    Manifest::build(&[
        ("b.img".to_owned(), vec![1, 2, 3]),
        ("a.img".to_owned(), vec![4, 5]),
    ])
}

#[test]
fn a_manifest_round_trips() {
    let original = sample();
    let parsed = Manifest::parse(&original.render()).expect("rendered output must parse");
    assert_eq!(parsed, original);
}

#[test]
fn entries_are_recorded_in_sorted_order() {
    // The token is derived from the entries, so iteration order must not be able
    // to change it.
    let manifest = sample();
    let names: Vec<&str> = manifest.names().collect();
    assert_eq!(names, vec!["a.img", "b.img"]);

    let reordered = Manifest::build(&[
        ("a.img".to_owned(), vec![4, 5]),
        ("b.img".to_owned(), vec![1, 2, 3]),
    ]);
    assert_eq!(reordered.token(), sample().token());
}

#[test]
fn the_token_changes_when_the_fixture_set_changes() {
    let base = sample();
    let extra = Manifest::build(&[
        ("b.img".to_owned(), vec![1, 2, 3]),
        ("a.img".to_owned(), vec![4, 5]),
        ("c.img".to_owned(), vec![6]),
    ]);
    assert_ne!(base.token(), extra.token());

    let altered = Manifest::build(&[
        ("b.img".to_owned(), vec![1, 2, 4]),
        ("a.img".to_owned(), vec![4, 5]),
    ]);
    assert_ne!(base.token(), altered.token());
}

#[test]
fn a_digest_identifies_its_bytes() {
    let manifest = sample();
    let expected = hex(&sha2::Sha256::digest(&[1, 2, 3][..]));
    assert!(manifest.contains_digest(&expected));
    assert!(!manifest.contains_digest(&"0".repeat(64)));
    assert_eq!(manifest.entry("b.img").expect("entry exists").length, 3);
}

#[test]
fn a_missing_or_wrong_header_is_rejected() {
    assert_eq!(Manifest::parse("").unwrap_err(), ManifestError::Header);
    assert_eq!(
        Manifest::parse("# something else\n").unwrap_err(),
        ManifestError::Header
    );
}

#[test]
fn malformed_lines_are_rejected_rather_than_skipped() {
    // The interlock reads this file before deciding whether a destructive suite
    // may run, so a lenient parser is a hole in SAFE-007.
    let digest = "a".repeat(64);
    let cases = [
        format!("{MANIFEST_HEADER}\ntoken {digest}\nimage\n"),
        format!("{MANIFEST_HEADER}\ntoken {digest}\nimage {digest} 3\n"),
        format!("{MANIFEST_HEADER}\ntoken {digest}\nimage {digest} 3 a.img extra\n"),
        format!("{MANIFEST_HEADER}\ntoken {digest}\nimage {digest} notanumber a.img\n"),
        format!("{MANIFEST_HEADER}\ntoken {digest}\nunknown-verb value\n"),
        format!("{MANIFEST_HEADER}\ntoken {digest}\nimage short 3 a.img\n"),
        format!(
            "{MANIFEST_HEADER}\ntoken {digest}\nimage {} 3 a.img\n",
            "A".repeat(64)
        ),
    ];
    for case in cases {
        assert!(Manifest::parse(&case).is_err(), "must reject:\n{case}");
    }
}

#[test]
fn a_missing_or_duplicated_token_is_rejected() {
    let digest = "a".repeat(64);
    assert_eq!(
        Manifest::parse(&format!("{MANIFEST_HEADER}\nimage {digest} 3 a.img\n")).unwrap_err(),
        ManifestError::MissingToken
    );
    assert_eq!(
        Manifest::parse(&format!(
            "{MANIFEST_HEADER}\ntoken {digest}\ntoken {digest}\n"
        ))
        .unwrap_err(),
        ManifestError::DuplicateToken
    );
}

#[test]
fn a_duplicate_name_is_rejected() {
    let digest = "a".repeat(64);
    let text = format!(
        "{MANIFEST_HEADER}\ntoken {digest}\nimage {digest} 1 a.img\nimage {digest} 2 a.img\n"
    );
    assert!(matches!(
        Manifest::parse(&text).unwrap_err(),
        ManifestError::DuplicateName(name) if name == "a.img"
    ));
}

#[test]
fn hex_is_lowercase_and_padded() {
    assert_eq!(hex(&[0x00, 0x0f, 0xff]), "000fff");
}

use sha2::Digest as _;
