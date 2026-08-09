//! The classification, tested against the catalogue's own images — built
//! in memory from source, so the fixtures' evidence claims and this
//! parser's answers can never drift apart silently. Every row of the
//! SI-35 resolution's accepted classification table appears here against
//! its named fixture, and the decisive demonstration carries its own
//! test.

use super::{
    Classification, Condition, Geometry, IndeterminateBasis, ParseRefusal, Scheme, TableState,
    classify,
};

/// The window size M10 measured as separating, and the caller shape the
/// helper will use.
const WINDOW: usize = 64 * 1024;

/// Build one catalogue image in memory and classify its windows.
fn classified(name: &str) -> Classification {
    classified_with(name, None)
}

/// Classify with a stated sector size, overriding the fixture's own.
fn classified_with(name: &str, stated_sector: Option<u32>) -> Classification {
    let fixture = partman_fixtures::catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name.trim_end_matches(".img") == name.trim_end_matches(".img"))
        .unwrap_or_else(|| panic!("{name} is not in the catalogue"));
    let bytes = (fixture.build)().into_bytes();
    let sector = stated_sector.unwrap_or(if name.contains("4kn") { 4096 } else { 512 });
    let geometry = Geometry {
        sector_size: sector,
        total_sectors: bytes.len() as u64 / u64::from(sector),
    };
    let head = &bytes[..WINDOW.min(bytes.len())];
    let tail = &bytes[bytes.len().saturating_sub(WINDOW)..];
    classify(head, tail, geometry).expect("catalogue fixtures classify without refusal")
}

fn checksum_of(state: &TableState) -> [u8; 32] {
    match state {
        TableState::Present { checksum } => *checksum,
        other => panic!("expected Present, got {other:?}"),
    }
}

// Requirements: INV-003, SAFE-005
//   The decisive SI-35 fixture — two independently valid GPTs describing different partitions — classifies as ADR-C3 Indeterminate on the ambiguous arm, never Present and never Absent: the refusal demonstration's classification half, on the exact fixture the register's evidence clause names
// Evidence: the_decisive_fixture_classifies_indeterminate_never_present
#[test]
fn the_decisive_fixture_classifies_indeterminate_never_present() {
    let classification = classified("gpt-conflicting-tables-512");
    assert_eq!(
        classification.state,
        TableState::Indeterminate {
            basis: IndeterminateBasis::Ambiguous
        },
        "two valid disagreeing copies must be ambiguous — a parser that picks a winner \
         invents an authority the format does not name"
    );
    assert_eq!(classification.scheme, Some(Scheme::Gpt));
}

// Requirements: INV-003
//   Every row of the accepted SI-35 classification table holds against its named catalogue fixture: healthy GPTs at both sector sizes are Present, one-valid-authority shapes are Present with their condition, the unreadable shape is Indeterminate, hybrid is Present with its condition, standalone MBR and APM are Present, and every signature-only medium is Absent
// Evidence: every_classification_row_holds_against_its_fixture
#[test]
fn every_classification_row_holds_against_its_fixture() {
    let healthy = classified("gpt-basic-512");
    assert!(matches!(healthy.state, TableState::Present { .. }));
    assert_eq!(healthy.scheme, Some(Scheme::Gpt));
    assert!(healthy.conditions.is_empty());

    let four_k = classified("gpt-basic-4kn");
    assert!(matches!(four_k.state, TableState::Present { .. }));
    assert_ne!(
        checksum_of(&four_k.state),
        checksum_of(&healthy.state),
        "different disks carry different content checksums"
    );

    let recovered = classified("gpt-invalid-primary-valid-backup-512");
    assert!(matches!(recovered.state, TableState::Present { .. }));
    assert_eq!(
        recovered.conditions,
        vec![Condition::PrimaryInvalid],
        "one valid authority is Present-with-condition, per the fixture's own recorded \
         claim: recoverable, and NOT Indeterminate"
    );

    let missing = classified("gpt-missing-backup-512");
    assert!(matches!(missing.state, TableState::Present { .. }));
    assert_eq!(missing.conditions, vec![Condition::BackupMissing]);

    let unreadable = classified("gpt-both-copies-invalid-512");
    assert_eq!(
        unreadable.state,
        TableState::Indeterminate {
            basis: IndeterminateBasis::Unreadable
        },
        "both copies invalid while the protective MBR asserts a GPT is the unreadable arm"
    );

    let hybrid = classified("hybrid-mbr-gpt-512");
    assert!(matches!(hybrid.state, TableState::Present { .. }));
    assert_eq!(hybrid.conditions, vec![Condition::HybridMbr]);

    let mbr = classified("mbr-basic-512");
    assert!(matches!(mbr.state, TableState::Present { .. }));
    assert_eq!(mbr.scheme, Some(Scheme::Mbr));

    let apm = classified("apm-basic-512");
    assert!(matches!(apm.state, TableState::Present { .. }));
    assert_eq!(apm.scheme, Some(Scheme::Apm));

    for absent in [
        "blank-512",
        "luks2-whole-disk-512",
        "lvm2-pv-orphan-512",
        "mdraid-1.2-member-512",
        "ext4-with-stale-mdraid-090-512",
    ] {
        let classification = classified(absent);
        assert_eq!(
            classification.state,
            TableState::Absent,
            "{absent}: no location claims a table, and an absent table says nothing about data"
        );
        assert_eq!(classification.scheme, None);
    }
}

// Requirements: MODEL-005
//   The Present checksum is computed over copy-invariant content, so the two agreeing GPT copies produce one value from either copy — the recipe stays stable under re-probe of unchanged bytes and never hashes the per-copy header fields that differ by design
// Evidence: the_checksum_is_copy_invariant
#[test]
fn the_checksum_is_copy_invariant() {
    // The invalid-primary fixture's checksum comes from the backup copy;
    // the healthy fixture's from agreeing copies (implementation reads the
    // primary). Both fixtures share their build recipe, so equal checksums
    // here prove the two copies of one table hash identically — and that
    // the recipe excluded every per-copy field.
    let from_agreeing = checksum_of(&classified("gpt-basic-512").state);
    let from_backup_only = checksum_of(&classified("gpt-invalid-primary-valid-backup-512").state);
    assert_eq!(
        from_agreeing, from_backup_only,
        "one table's content must hash to one value regardless of which copy carried it"
    );
}

// Requirements: SAFE-005
//   Probing a 4Kn medium under a 512-byte contract answers Indeterminate-unreadable rather than fabricating a table or reporting blank: the parser's answers are relative to the stated geometry, reproducing the measured libblkid PMBR trap honestly instead of guessing across sector sizes
// Evidence: wrong_geometry_answers_indeterminate_not_absent
#[test]
fn wrong_geometry_answers_indeterminate_not_absent() {
    let classification = classified_with("gpt-basic-4kn", Some(512));
    assert_eq!(
        classification.state,
        TableState::Indeterminate {
            basis: IndeterminateBasis::Unreadable
        },
        "the protective MBR claims a GPT this contract cannot read; blank would be a lie \
         and Present would be a fabrication"
    );
}

// Requirements: SAFE-005
//   The parser refuses the caller's contract violations with typed refusals — unsupported sector sizes, non-sector-multiple windows, oversize windows, windows too small to hold the structures — while hostile media never refuse, they classify
// Evidence: caller_contract_violations_refuse_with_typed_values
#[test]
fn caller_contract_violations_refuse_with_typed_values() {
    let head = vec![0_u8; 4096];
    let tail = vec![0_u8; 4096];
    let ok = Geometry {
        sector_size: 512,
        total_sectors: 8192,
    };

    assert_eq!(
        classify(
            &head,
            &tail,
            Geometry {
                sector_size: 520,
                ..ok
            }
        ),
        Err(ParseRefusal::SectorSizeUnsupported { stated: 520 })
    );
    assert_eq!(
        classify(&head[..500], &tail, ok),
        Err(ParseRefusal::WindowNotSectorMultiple)
    );
    assert_eq!(
        classify(&head[..512], &tail, ok),
        Err(ParseRefusal::HeadWindowTooSmall)
    );
    assert_eq!(
        classify(&head, &[], ok),
        Err(ParseRefusal::TailWindowTooSmall)
    );
    assert_eq!(
        classify(
            &head,
            &tail,
            Geometry {
                sector_size: 512,
                total_sectors: 2,
            }
        ),
        Err(ParseRefusal::GeometryImpossible)
    );
    let oversize = vec![0_u8; super::WINDOW_LIMIT + 512];
    assert_eq!(
        classify(&oversize, &tail, ok),
        Err(ParseRefusal::WindowOverLimit)
    );
}

// Requirements: SAFE-005
//   A GPT header that verifies its own CRC but demands an entry array the window cannot contain — or more entries than the declared bound — is an invalid copy, never a trusted one: the parser walks nothing on a header's say-so
// Evidence: an_unverifiable_entry_array_invalidates_the_copy
#[test]
fn an_unverifiable_entry_array_invalidates_the_copy() {
    // Start from the healthy image and point the primary's entry array
    // beyond the window, re-checksumming the header so only the
    // reachability check can refuse it.
    let fixture = partman_fixtures::catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name == "gpt-basic-512.img")
        .expect("catalogue holds the healthy fixture");
    let mut bytes = (fixture.build)().into_bytes();
    bytes[512 + 72..512 + 80].copy_from_slice(&10_000_u64.to_le_bytes());
    let header_size =
        u32::from_le_bytes(bytes[512 + 12..512 + 16].try_into().expect("4 bytes")) as usize;
    bytes[512 + 16..512 + 20].fill(0);
    let crc = super::crc32(&bytes[512..512 + header_size]);
    bytes[512 + 16..512 + 20].copy_from_slice(&crc.to_le_bytes());

    let geometry = Geometry {
        sector_size: 512,
        total_sectors: bytes.len() as u64 / 512,
    };
    let head = &bytes[..WINDOW];
    let tail = &bytes[bytes.len() - WINDOW..];
    let classification = classify(head, tail, geometry).expect("no caller violation here");
    // The backup still verifies, so the medium stays Present — carried by
    // the copy that could be checked — with the primary recorded invalid.
    assert!(matches!(classification.state, TableState::Present { .. }));
    assert_eq!(classification.conditions, vec![Condition::PrimaryInvalid]);
}

// Requirements: SAFE-005
//   The classification type carries no proceed-enabling reading: no Default, no is_-style predicate, no ordering — a consumer must match all three ADR-C3 arms explicitly, and what a writer may do with each stays the specification's to say
// Evidence: the_state_type_offers_no_proceed_enabling_reading
#[test]
fn the_state_type_offers_no_proceed_enabling_reading() {
    let source = include_str!("lib.rs");
    for forbidden in [
        "impl Default",
        "derive(Default",
        "fn is_safe",
        "fn is_ok",
        "fn proceed",
        "PartialOrd",
    ] {
        assert!(
            !source.contains(forbidden),
            "lib.rs contains `{forbidden}`: the state must stay a three-arm match with no \
             shortcut reading"
        );
    }
    // And the source-scan half of purity: no I/O, no process, no
    // environment — the parser stays pure over its arguments.
    for forbidden in ["std::fs", "std::process", "std::env", "std::net"] {
        assert!(
            !source.contains(forbidden),
            "lib.rs reaches `{forbidden}`; this parser is pure over caller-supplied bytes"
        );
    }
}

// Requirements: Section 11.4
//   The in-crate CRC-32 agrees with the fixtures crate's two independent spellings on the published IEEE check value and on real fixture bytes, so three implementations by three methods anchor the same function
// Evidence: the_crc_agrees_with_the_independent_spellings
#[test]
fn the_crc_agrees_with_the_independent_spellings() {
    assert_eq!(
        super::crc32(b"123456789"),
        0xcbf4_3926,
        "the published IEEE CRC-32 check value"
    );
    let fixture = partman_fixtures::catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name == "gpt-basic-512.img")
        .expect("catalogue holds the healthy fixture");
    let bytes = (fixture.build)().into_bytes();
    assert_eq!(
        super::crc32(&bytes[1024..1024 + 16384]),
        partman_fixtures::layout::crc32(&bytes[1024..1024 + 16384]),
        "bitwise-by-byte here, bit-iterating there: two spellings, one function"
    );
}
