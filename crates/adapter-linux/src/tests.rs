//! Increment 1's suite: the bounded read seam's two refusals, ADR-C4's three
//! answers kept apart above the seam, the interface-to-method mapping that
//! makes MODEL-004's derived confidence honest, and the INV-003 reach
//! declaration's completeness, ordering, basis coupling, and independence
//! from any reading surface.
//!
//! Every test drives a fake over a synthesized tree. None is platform-gated:
//! the adapter is pure over the injected seam, so this suite runs on all
//! three CI legs rather than only where a defect would be least convenient to
//! find.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use partman_domain::model::provenance::{Confidence, Method, Outcome, PropertyObservations};

use crate::contract::{
    AttributeRead, ContractSource, InterfaceAnswered, Listing, VALUE_LIMIT, list_bounded,
    read_attribute,
};
use crate::observation::{Interface, observe};

/// The Tier-1 fake. A path absent from either map yields `NotFound`, which is
/// how the positively-absent case is exercised; keys are normalized to
/// forward slashes so the suite runs on a Windows host.
struct FakeSource {
    dirs: BTreeMap<String, Result<Vec<String>, std::io::ErrorKind>>,
    files: BTreeMap<String, Result<Vec<u8>, std::io::ErrorKind>>,
}

fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl ContractSource for FakeSource {
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        match self.dirs.get(&key(path)) {
            Some(Ok(entries)) => Ok(entries.clone()),
            Some(Err(kind)) => Err(std::io::Error::new(*kind, "listing refused")),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no such directory",
            )),
        }
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        match self.files.get(&key(path)) {
            Some(Ok(bytes)) => Ok(bytes.clone()),
            Some(Err(kind)) => Err(std::io::Error::new(*kind, "read refused")),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no such attribute",
            )),
        }
    }
}

impl FakeSource {
    /// One interface directory holding every attribute shape at once: a
    /// present value, a padded value, an empty value, an unreadable one, an
    /// oversize one, and non-UTF-8 bytes. The absent case needs no entry —
    /// that is the point of it.
    fn one_interface() -> Self {
        let mut dirs = BTreeMap::new();
        dirs.insert(
            "/sys/class/block".to_owned(),
            Ok(vec!["sda".to_owned(), "sda1".to_owned()]),
        );
        let mut files = BTreeMap::new();
        files.insert(
            "/sys/class/block/sda/serial".to_owned(),
            Ok(b"S3Z9NB0K\n".to_vec()),
        );
        files.insert(
            "/sys/class/block/sda/vendor".to_owned(),
            Ok(b"ATA     \n".to_vec()),
        );
        files.insert("/sys/class/block/sda/wwid".to_owned(), Ok(b"\n".to_vec()));
        files.insert(
            "/sys/class/block/sda/model".to_owned(),
            Err(std::io::ErrorKind::PermissionDenied),
        );
        files.insert(
            "/sys/class/block/sda/oversize".to_owned(),
            Ok(vec![b'x'; VALUE_LIMIT + 1]),
        );
        files.insert(
            "/sys/class/block/sda/mangled".to_owned(),
            Ok(vec![0xff, 0xfe, 0x00]),
        );
        Self { dirs, files }
    }

    fn empty() -> Self {
        Self {
            dirs: BTreeMap::new(),
            files: BTreeMap::new(),
        }
    }
}

/// The block-class path both roots' tests share.
fn class_path() -> PathBuf {
    PathBuf::from("/sys/class/block")
}

/// List the fake's one interface and hand back the evidence token, which is
/// the only way this suite obtains one — the same path production code takes.
fn answered(source: &FakeSource) -> InterfaceAnswered {
    match list_bounded(source, &class_path()) {
        Listing::Listed { answered, .. } => answered,
        _ => panic!("the fake's interface must answer"),
    }
}

fn read(source: &FakeSource, name: &str, token: &InterfaceAnswered) -> AttributeRead {
    read_attribute(
        source,
        &PathBuf::from(format!("/sys/class/block/sda/{name}")),
        token,
    )
}

// Requirements: SAFE-005
//   A listing above the entry bound refuses with the count it saw rather
//   than returning a truncated list, so a partial listing is never
//   mistaken for a complete one. The bound is applied above the seam, so
//   this fake can drive it — WP-035's per-value bound sits inside its
//   production implementation and has no test for exactly that reason.
// Evidence: a_listing_over_the_entry_limit_refuses_rather_than_truncating
#[test]
fn a_listing_over_the_entry_limit_refuses_rather_than_truncating() {
    let mut source = FakeSource::empty();
    let entries: Vec<String> = (0..=crate::contract::ENTRY_LIMIT)
        .map(|index| format!("sd{index}"))
        .collect();
    let seen = entries.len();
    source.dirs.insert(key(&class_path()), Ok(entries));

    match list_bounded(&source, &class_path()) {
        Listing::OverLimit { seen: reported } => assert_eq!(
            reported, seen,
            "the refusal must report the count seen, not the bound"
        ),
        _ => panic!("a listing over the bound must refuse, never truncate"),
    }
}

// Requirements: SAFE-005
//   An interface the platform does not expose answers `unavailable`, never
//   an empty listing: "there is no such interface here" is not "this host
//   has nothing", and rendering the first as the second is the fail-closed
//   violation SAFE-005 exists to prevent.
// Evidence: an_absent_interface_is_unavailable_never_an_empty_listing
#[test]
fn an_absent_interface_is_unavailable_never_an_empty_listing() {
    match list_bounded(&FakeSource::empty(), &class_path()) {
        Listing::Unavailable { .. } => {}
        _ => panic!("an absent interface must be unavailable, never an empty listing"),
    }
}

// Requirements: SAFE-005
//   An attribute over the byte bound refuses with the count it saw and is
//   never truncated: a prefix is byte-for-byte indistinguishable from a
//   complete read of that length, so truncation would hand a caller a
//   partial answer wearing a whole answer's shape.
// Evidence: an_attribute_over_the_value_limit_refuses_rather_than_truncating
#[test]
fn an_attribute_over_the_value_limit_refuses_rather_than_truncating() {
    let source = FakeSource::one_interface();
    let token = answered(&source);
    match read(&source, "oversize", &token) {
        AttributeRead::OverLimit { seen } => assert_eq!(
            seen,
            VALUE_LIMIT + 1,
            "the refusal must report the byte count seen"
        ),
        _ => panic!("an attribute over the bound must refuse, never truncate"),
    }
}

// Requirements: MODEL-004
//   Exactly one trailing newline is stripped and nothing else, so a padded
//   value survives as the value the interface reported. Trimming all
//   trailing whitespace turns a padded vendor into an empty string, which
//   then reads as a positively determined absence of a vendor — ADR-C4's
//   conflation, and one WP-035 records having made.
// Evidence: one_trailing_newline_is_stripped_and_padding_is_kept
#[test]
fn one_trailing_newline_is_stripped_and_padding_is_kept() {
    let source = FakeSource::one_interface();
    let token = answered(&source);
    match read(&source, "serial", &token) {
        AttributeRead::Text(text) => assert_eq!(text, "S3Z9NB0K"),
        _ => panic!("a present attribute must read as text"),
    }
    match read(&source, "vendor", &token) {
        AttributeRead::Text(text) => assert_eq!(
            text, "ATA     ",
            "the padding is part of the value the interface reported"
        ),
        _ => panic!("a padded attribute is a value, not an absence"),
    }
}

// Requirements: MODEL-004
//   ADR-C4's three answers stay apart at this boundary: an attribute that
//   is not present under an interface that answered, and one that exists
//   and is empty, are both positively determined absences and become
//   `ObservedAbsent` — a value; an unreadable attribute becomes `Failed`
//   and is never rendered as an absence.
// Evidence: a_missing_attribute_is_an_absence_and_an_unreadable_one_is_a_failure
#[test]
fn a_missing_attribute_is_an_absence_and_an_unreadable_one_is_a_failure() {
    let source = FakeSource::one_interface();
    let token = answered(&source);

    for name in ["absent-entirely", "wwid"] {
        let read = read(&source, name, &token);
        assert!(
            matches!(
                observe(Interface::Sysfs, &read).outcome,
                Outcome::ObservedAbsent
            ),
            "{name} is a positively determined absence, which is a value"
        );
    }

    assert!(
        matches!(
            observe(Interface::Sysfs, &read(&source, "model", &token)).outcome,
            Outcome::Failed { .. }
        ),
        "an unreadable attribute is a failed read, never an absence"
    );
}

// Requirements: MODEL-004
//   Bytes that are not UTF-8 refuse rather than being lossily converted,
//   and the refusal is a failed read: a mangled identifier is not the
//   value the interface reported, and reporting it as one would put a
//   value into provenance that no interface ever produced.
// Evidence: non_utf8_attribute_bytes_refuse_rather_than_being_lossily_converted
#[test]
fn non_utf8_attribute_bytes_refuse_rather_than_being_lossily_converted() {
    let source = FakeSource::one_interface();
    let token = answered(&source);
    let read = read(&source, "mangled", &token);
    assert!(matches!(read, AttributeRead::NotText));
    match observe(Interface::Sysfs, &read).outcome {
        Outcome::Failed { error } => assert!(
            !error.contains('\u{fffd}'),
            "the refusal must not echo a lossily converted value"
        ),
        _ => panic!("non-UTF-8 bytes are a failed read"),
    }
}

// Requirements: MODEL-004
//   The interface decides the method, and the method decides the derived
//   confidence: a directly read `sysfs` attribute is `Direct` and derives
//   `authoritative`, while a `udev` database value — computed by root's
//   udevd at device-add time and read here from its cache — is
//   `Heuristic` and derives `inferred`. The conservative direction is the
//   decision: calling a cached third-party computation authoritative would
//   let one stale record outrank nothing.
// Evidence: udev_values_are_inferred_and_sysfs_attributes_authoritative
#[test]
fn udev_values_are_inferred_and_sysfs_attributes_authoritative() {
    let source = FakeSource::one_interface();
    let token = answered(&source);
    let read = read(&source, "serial", &token);

    let direct = observe(Interface::Sysfs, &read);
    assert_eq!(direct.method, Method::Direct);
    assert_eq!(direct.adapter, "partman-adapter-linux/linux-sysfs");
    assert_eq!(
        PropertyObservations {
            observations: vec![direct],
        }
        .derive_confidence()
        .expect("a text value encodes"),
        Confidence::Authoritative
    );

    let cached = observe(Interface::UdevDatabase, &read);
    assert_eq!(cached.method, Method::Heuristic);
    assert_eq!(cached.adapter, "partman-adapter-linux/linux-udev-db");
    assert_eq!(
        PropertyObservations {
            observations: vec![cached],
        }
        .derive_confidence()
        .expect("a text value encodes"),
        Confidence::Inferred,
        "a cached third-party computation is inferred, never authoritative"
    );
}

// Requirements: MODEL-004
//   An absence reading is unavailable to a caller that cannot show the
//   interface answered: reading an attribute requires the evidence token,
//   and the token is produced only by a listing that succeeded. The
//   compile-fail doctest on the type is the proof that it cannot be
//   asserted; this test records the obligation's positive half, that the
//   one reachable producer is a successful listing.
// Evidence: an_absence_needs_evidence_that_the_interface_answered
#[test]
fn an_absence_needs_evidence_that_the_interface_answered() {
    assert!(
        matches!(
            list_bounded(&FakeSource::empty(), &class_path()),
            Listing::Unavailable { .. }
        ),
        "an interface that did not answer yields no evidence token"
    );
    let source = FakeSource::one_interface();
    let token = answered(&source);
    assert!(matches!(
        read(&source, "absent-entirely", &token),
        AttributeRead::NotPresent
    ));
}

// Requirements: INV-003
//   The reach declaration carries one cell per INV-003 state, in INV-003's
//   own order. The array is fixed-size, so a missing cell is a compile
//   error rather than an omitted `no` — which INV-003 forbids.
// Evidence: the_linux_reach_declaration_is_complete_and_ordered
#[test]
fn the_linux_reach_declaration_is_complete_and_ordered() {
    let declared: Vec<&str> = crate::reach::REACH
        .cells
        .iter()
        .map(|cell| cell.state)
        .collect();
    assert_eq!(
        declared,
        crate::reach::STATES.to_vec(),
        "one cell per INV-003 state, in INV-003's order — a missing cell is an omitted `no`"
    );
}

// Requirements: INV-003
//   Every cell is negative on the not-measured basis with no citation, and
//   the coupling holds in both directions: a citation exists exactly when
//   the basis is measured. The contract statement says this contract
//   reaches no surface yet and names the increment that changes it, so the
//   declaration is derived from the contract rather than from a device.
// Evidence: the_linux_reach_declaration_claims_no_state_and_cites_nothing_yet
#[test]
fn the_linux_reach_declaration_claims_no_state_and_cites_nothing_yet() {
    for cell in &crate::reach::REACH.cells {
        assert!(
            !cell.distinguished,
            "{}: no state is reached yet",
            cell.state
        );
        assert_eq!(cell.basis, crate::reach::basis::NOT_MEASURED);
        assert_eq!(
            cell.citation.is_some(),
            cell.basis == crate::reach::basis::MEASURED,
            "{}: a citation exists exactly when the basis is measured",
            cell.state
        );
    }
    assert_eq!(crate::reach::REACH.contract.state, "not-implemented");
    assert!(
        crate::reach::REACH
            .contract
            .reference
            .contains("increment 2"),
        "the statement must name what changes it"
    );
    assert!(crate::reach::reach_json().contains(crate::reach::REACH_SCHEMA));
}

// Requirements: INV-003
//   The reach declaration is a property of the contract and the platform,
//   never of a device: its module cannot name a reading surface at all —
//   not the seam, not its two primitives, not a file operation. A text
//   scan, and its exact reach is the needle list in the test body: it
//   catches those direct spellings, a glob import would evade it and is
//   refused by clippy's `wildcard_imports`, and the module's own imports
//   sit in the same file a reviewer reads.
// Evidence: the_linux_reach_declaration_names_no_read_surface
#[test]
fn the_linux_reach_declaration_names_no_read_surface() {
    let source = include_str!("reach.rs");
    for needle in [
        "std::fs",
        "File::",
        "read_to_string",
        "std::process",
        "ContractSource",
        "list_bounded",
        "read_attribute",
    ] {
        assert!(
            !source.contains(needle),
            "reach.rs contains `{needle}`: the declaration must stay a property of the \
             contract, declared independently of any device"
        );
    }
}

// Requirements: INV-003, MODEL-003
//   The published document and this crate state one vocabulary, not two:
//   the schema identifier, all six INV-003 states, both basis words, and
//   both contract state words appear in `schemas/adapter-linux/reach.md`
//   verbatim, so the format cannot drift from the module that produces it
//   without failing here.
// Evidence: the_published_reach_document_pins_this_crates_vocabulary
#[test]
fn the_published_reach_document_pins_this_crates_vocabulary() {
    const DOC: &str = include_str!("../../../schemas/adapter-linux/reach.md");

    assert!(
        DOC.contains(crate::reach::REACH_SCHEMA),
        "the document must publish the schema identifier"
    );
    for state in crate::reach::STATES {
        assert!(
            DOC.contains(state),
            "the document must publish the `{state}` state"
        );
    }
    for word in [
        crate::reach::basis::MEASURED,
        crate::reach::basis::NOT_MEASURED,
        crate::reach::REACH.contract.state,
        "implemented-reaches-no-table-state",
    ] {
        assert!(
            DOC.contains(word),
            "the document must publish the `{word}` vocabulary word"
        );
    }
}

// Requirements: SAFE-002
//   No shipped module consults privilege. There is no branch on user,
//   group, or a permission error, so running as root produces the same
//   answer as running as anyone else — a contract that widened with
//   privilege would make the published INV-003 reach a per-user
//   statement, which INV-003 forbids. A behavioural version of this test
//   would compare two runs of one fake and could never fail, since
//   nothing varies between them; the scan is structural for that reason.
// Evidence: the_adapter_names_no_privilege_conditional_branch
#[test]
fn the_adapter_names_no_privilege_conditional_branch() {
    for (name, source) in shipped_sources() {
        for needle in [
            "geteuid",
            "getuid",
            "getgid",
            "is_root",
            "effective_uid",
            "PermissionDenied =>",
        ] {
            assert!(
                !source.contains(needle),
                "{name} contains `{needle}`: this contract has no privilege-conditional branch"
            );
        }
    }
}

// Requirements: SAFE-002
//   No shipped module opens a device node or launches a process. The
//   contract is file reads of attribute and database paths the caller
//   supplies as roots, which is what keeps this package's whole suite
//   unprivileged at Tier 1.
// Evidence: the_adapter_opens_no_device_node_and_launches_no_process
#[test]
fn the_adapter_opens_no_device_node_and_launches_no_process() {
    for (name, source) in shipped_sources() {
        for needle in ["/dev/", "std::process", "Command::new", "std::env"] {
            assert!(
                !source.contains(needle),
                "{name} contains `{needle}`: this adapter opens no device and launches nothing"
            );
        }
    }
}

/// Every shipped module, by name, so a new one must enter this list before a
/// structural guard can silently stop covering the crate.
fn shipped_sources() -> [(&'static str, &'static str); 4] {
    [
        ("lib.rs", include_str!("lib.rs")),
        ("contract.rs", include_str!("contract.rs")),
        ("observation.rs", include_str!("observation.rs")),
        ("reach.rs", include_str!("reach.rs")),
    ]
}
