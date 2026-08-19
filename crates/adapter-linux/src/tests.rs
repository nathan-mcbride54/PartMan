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

use partman_domain::model::naming::{NamingFields, NodeEntry};
use partman_domain::model::provenance::{
    Confidence, Method, Observation, Outcome, PropertyObservations,
};

use crate::contract::{
    AttributeRead, ContractSource, InterfaceAnswered, Listing, NamingRead, VALUE_LIMIT,
    list_bounded, read_attribute, read_naming_source,
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
    ///
    /// Two entries are "empty" in different senses, and the difference is
    /// exactly what ADR-0034's bytes path exists to preserve: `wwid` holds a
    /// lone newline, which the text path strips into an empty string and
    /// therefore reads as a positively determined absence, while `empty`
    /// holds no bytes at all. Through the naming path the first is a
    /// one-byte value and only the second is an absence.
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
        files.insert("/sys/class/block/sda/empty".to_owned(), Ok(Vec::new()));
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

/// The same attribute, through the naming path instead of the text one. The
/// suite reads several files both ways on purpose: the divergence is the
/// deliverable, not an accident to be papered over.
fn naming(source: &FakeSource, name: &str, token: &InterfaceAnswered) -> NamingRead {
    read_naming_source(
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

// Requirements: INV-002
//   ADR-0019 takes identifier bytes contract-source-verbatim, and ADR-0034
//   draws the consequence in terms: the delivered text path applies three
//   transformations — a UTF-8 requirement, a lossy refusal, and a
//   trailing-newline strip — and is therefore "not a lawful naming-input
//   path". It owes a bytes seam instead, and calls that increment 3's first
//   delivery obligation. Reading the same three files both ways is what
//   makes the divergence a measured fact rather than a claim: the newline
//   ADR-0034 requires kept, a lone newline a one-byte name where the text
//   path reports a positively determined absence, and non-UTF-8 bytes a
//   legal name where the text path refuses.
// Evidence: naming_bytes_are_verbatim_where_the_text_path_transforms
#[test]
fn naming_bytes_are_verbatim_where_the_text_path_transforms() {
    let source = FakeSource::one_interface();
    let token = answered(&source);

    // The trailing newline: stripped by the text path, and ADR-0034 requires
    // it kept, because stripping has an undecidable edge — a value may
    // legitimately end in `0x0a`.
    assert!(
        matches!(read(&source, "serial", &token), AttributeRead::Text(text) if text == "S3Z9NB0K")
    );
    assert!(
        matches!(naming(&source, "serial", &token), NamingRead::Bytes(bytes) if bytes == b"S3Z9NB0K\n"),
        "naming bytes are the read's bytes, trailing newline included"
    );

    // That edge, exercised: a file holding only a newline. The text path
    // strips it to an empty string and reports an absence; through the
    // naming path it is a value one byte long.
    assert!(matches!(
        read(&source, "wwid", &token),
        AttributeRead::Empty
    ));
    assert!(
        matches!(naming(&source, "wwid", &token), NamingRead::Bytes(bytes) if bytes == b"\n"),
        "a lone newline is a one-byte name, not an absence"
    );

    // Only a file with no bytes at all is an absence on this path.
    assert!(matches!(
        naming(&source, "empty", &token),
        NamingRead::Empty
    ));

    // Non-UTF-8 bytes: refused by the text path, and a legal name here.
    assert!(matches!(
        read(&source, "mangled", &token),
        AttributeRead::NotText
    ));
    assert!(
        matches!(naming(&source, "mangled", &token), NamingRead::Bytes(bytes) if bytes == [0xff, 0xfe, 0x00]),
        "ADR-0019 makes non-UTF-8 identifier bytes legal, so this path has no NotText arm"
    );
}

// Requirements: SAFE-005
//   ADR-0034 gave ADR-0019's one naming outcome two siblings, because the
//   delivered contract produces outcomes ADR-0019 had no rule for, and the
//   two must stay separable at the seam because their consequences differ in
//   direction. A measured absence leaves the field absent, the name weaker,
//   and the device an operand — a stable truth about the hardware is a
//   lawful weak name. A failed read is not absence: the device is marked
//   indeterminate and is not a plan operand. An over-limit read belongs with
//   the failures, since bytes that were never seen whole are no evidence
//   about the device.
// Evidence: a_measured_absence_and_a_failed_naming_read_stay_separable
#[test]
fn a_measured_absence_and_a_failed_naming_read_stay_separable() {
    let source = FakeSource::one_interface();
    let token = answered(&source);

    assert!(
        matches!(
            naming(&source, "absent-entirely", &token),
            NamingRead::NotPresent
        ),
        "a source not present under an interface that answered is a measured absence"
    );
    assert!(
        matches!(naming(&source, "model", &token), NamingRead::Failed { .. }),
        "an unreadable source is a failed read, never an absence"
    );
    assert!(
        matches!(
            naming(&source, "oversize", &token),
            NamingRead::OverLimit { seen } if seen == VALUE_LIMIT + 1
        ),
        "an over-limit naming read refuses with the count it saw rather than truncating"
    );
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
    assert_eq!(
        crate::reach::REACH.contract.state,
        "implemented-reaches-no-table-state",
        "a contract that reads a roster is implemented; describing it as absent would make \
         the declaration underived from the contract, which INV-003 forbids"
    );
    assert!(
        crate::reach::REACH.contract.reference.contains("roster"),
        "the statement must name what changes it"
    );
    assert!(crate::reach::reach_json().contains(crate::reach::REACH_SCHEMA));
}

// Requirements: INV-003
//   The declaration stays derived from the contract: the roster this crate
//   actually reads carries no partition-table key, which is why every cell
//   is negative. A key entering the roster without the declaration being
//   re-decided is the drift this pins — the reach would then describe a
//   contract that no longer exists.
// Evidence: no_partition_table_key_is_in_the_roster_the_reach_describes
#[test]
fn no_partition_table_key_is_in_the_roster_the_reach_describes() {
    for (property, relative) in crate::devices::SYSFS_FIELDS {
        for spelling in ["part", "table"] {
            assert!(
                !property.contains(spelling) && !relative.contains(spelling),
                "{property}: a partition-table surface in the roster contradicts the \
                 published reach, which says the contract carries none"
            );
        }
    }
    for wanted in crate::devices::UDEV_KEYS {
        assert!(
            !wanted.contains("PART_TABLE"),
            "{wanted}: a partition-table key in the roster contradicts the published reach"
        );
    }
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
///
/// The array is fixed-size and its length is pinned by
/// `every_shipped_module_is_covered_by_the_structural_guards`, because both
/// SAFE-002 scans iterate it: a module added without an entry here would be
/// exempt from both while leaving both tests green.
fn shipped_sources() -> [(&'static str, &'static str); 13] {
    [
        ("lib.rs", include_str!("lib.rs")),
        ("arrays.rs", include_str!("arrays.rs")),
        ("contract.rs", include_str!("contract.rs")),
        ("derivation.rs", include_str!("derivation.rs")),
        ("devices.rs", include_str!("devices.rs")),
        ("floor.rs", include_str!("floor.rs")),
        ("held.rs", include_str!("held.rs")),
        ("naming.rs", include_str!("naming.rs")),
        ("observation.rs", include_str!("observation.rs")),
        ("reach.rs", include_str!("reach.rs")),
        ("runtime.rs", include_str!("runtime.rs")),
        ("state.rs", include_str!("state.rs")),
        ("volumes.rs", include_str!("volumes.rs")),
    ]
}

/// A tree with one whole device, one partition child, one node whose
/// `partition` attribute is unreadable, and a database record for the device.
fn one_device_tree() -> FakeSource {
    let mut dirs = BTreeMap::new();
    dirs.insert(
        "/sys/class/block".to_owned(),
        Ok(vec![
            "sda".to_owned(),
            "sda1".to_owned(),
            "masked".to_owned(),
        ]),
    );
    let mut files = BTreeMap::new();
    // sda: no `partition` attribute at all — a whole device.
    files.insert("/sys/class/block/sda/dev".to_owned(), Ok(b"8:0\n".to_vec()));
    files.insert(
        "/sys/class/block/sda/size".to_owned(),
        Ok(b"1000215216\n".to_vec()),
    );
    files.insert(
        "/sys/class/block/sda/device/serial".to_owned(),
        Ok(b"S3Z9NB0K\n".to_vec()),
    );
    // sda1: carries `partition`, so it is not a whole device.
    files.insert(
        "/sys/class/block/sda1/partition".to_owned(),
        Ok(b"1\n".to_vec()),
    );
    // masked: the attribute cannot be read, so admission must fail closed.
    files.insert(
        "/sys/class/block/masked/partition".to_owned(),
        Err(std::io::ErrorKind::PermissionDenied),
    );
    files.insert(
        "/run/udev/data/b8:0".to_owned(),
        Ok(b"S:disk/by-id/ata-X\nE:ID_SERIAL=ata-Samsung_S3Z9NB0K\nE:ID_BUS=ata\n".to_vec()),
    );
    FakeSource { dirs, files }
}

fn enumerate_fake(source: &FakeSource) -> crate::devices::Enumeration {
    crate::devices::enumerate(
        source,
        &PathBuf::from("/sys"),
        &PathBuf::from("/run/udev/data"),
    )
}

fn devices_of(source: &FakeSource) -> Vec<crate::devices::Device> {
    match enumerate_fake(source) {
        crate::devices::Enumeration::Listed { devices } => devices,
        _ => panic!("the fake's block class must answer"),
    }
}

fn outcome_of<'a>(
    device: &'a crate::devices::Device,
    key: &str,
) -> &'a partman_domain::model::provenance::Outcome {
    &device
        .properties
        .iter()
        .find(|(name, _)| name == key)
        .unwrap_or_else(|| panic!("{key} must be reported"))
        .1
        .observations[0]
        .outcome
}

// Requirements: INV-001
//   A node is admitted as a whole device only on a positively determined
//   absence of the partition attribute. A partition carries it and is
//   excluded; a node whose attribute cannot be read is excluded too, which
//   is the fail-closed direction — a successful-read test would promote a
//   partition into the device list on any read error, and its sector count
//   would then be reported as a device capacity.
// Evidence: whole_devices_are_admitted_only_on_a_positively_absent_partition_attribute
#[test]
fn whole_devices_are_admitted_only_on_a_positively_absent_partition_attribute() {
    let devices = devices_of(&one_device_tree());
    assert_eq!(
        devices.len(),
        1,
        "exactly the one node without a readable partition attribute is a whole device"
    );
    assert_eq!(devices[0].selector, "device:0");
}

// Requirements: SAFE-005
//   An absent block class answers unavailable, never an empty device list.
// Evidence: an_absent_block_class_is_unavailable_never_an_empty_device_list
#[test]
fn an_absent_block_class_is_unavailable_never_an_empty_device_list() {
    assert!(
        matches!(
            enumerate_fake(&FakeSource::empty()),
            crate::devices::Enumeration::Unavailable { .. }
        ),
        "an absent interface is unavailable, never an empty device list"
    );
}

// Requirements: MODEL-004
//   ADR-C4's separation across the database half: a key missing from a
//   record that exists is a positively determined absence, while every key
//   of a device whose record does not exist is unavailable — calling those
//   absent would claim the database answered and said nothing.
// Evidence: a_missing_record_is_unavailable_while_a_missing_key_within_one_is_absent
#[test]
fn a_missing_record_is_unavailable_while_a_missing_key_within_one_is_absent() {
    use partman_domain::model::provenance::Outcome;

    let with_record = devices_of(&one_device_tree());
    assert!(
        matches!(
            outcome_of(&with_record[0], "linux-udev-db:ID_WWN"),
            Outcome::ObservedAbsent
        ),
        "a key missing from a record that exists is an absence"
    );

    let mut without = one_device_tree();
    without.files.remove("/run/udev/data/b8:0");
    let devices = devices_of(&without);
    for wanted in crate::devices::UDEV_KEYS {
        assert!(
            matches!(
                outcome_of(&devices[0], &format!("linux-udev-db:{wanted}")),
                Outcome::Unavailable { .. }
            ),
            "{wanted}: with no record, every key is unavailable and none is absent"
        );
    }
}

// Requirements: MODEL-004, INV-002
//   Nothing elects an identifier: the attribute layer's serial and the
//   database's serial-shaped key are two properties under two native names,
//   because they are two interfaces' different answers rather than one
//   fact — merging them would manufacture a conflicting confidence out of
//   values that were never in conflict.
// Evidence: two_interfaces_reporting_a_serial_produce_two_properties_and_elect_neither
#[test]
fn two_interfaces_reporting_a_serial_produce_two_properties_and_elect_neither() {
    use partman_domain::canonical::Value;
    use partman_domain::model::provenance::Outcome;

    let devices = devices_of(&one_device_tree());
    let attribute = outcome_of(&devices[0], "linux-sysfs:device/serial");
    let database = outcome_of(&devices[0], "linux-udev-db:ID_SERIAL");
    assert!(
        matches!(attribute, Outcome::Observed { value: Value::Text(text) } if text == "S3Z9NB0K")
    );
    assert!(
        matches!(database, Outcome::Observed { value: Value::Text(text) } if text == "ata-Samsung_S3Z9NB0K")
    );
    assert!(
        !devices[0]
            .properties
            .iter()
            .any(|(name, _)| name == "serial"),
        "no unqualified serial property exists: electing one is not this layer's act"
    );
}

// Requirements: INV-002
//   ADR-0018's transport answer is Unrecognized for every device, and no
//   other variant is constructible in this crate. What is missing is the
//   discrimination protocol, not the values: classifying values are now
//   recorded on Linux (ID_BUS=usb, two ID_PATH values), but a value names
//   no class until ADR-0018's fabric-versus-local rows say which classes
//   are local, and those are outstanding on every platform. A mapping from
//   interface strings to classes could therefore come only from vendor
//   documentation. Unrecognized resolves to Indeterminate at the closure,
//   never Permitted, which is the fail-closed direction.
// Evidence: every_device_answers_unrecognized_and_no_positive_class_is_constructible
#[test]
fn every_device_answers_unrecognized_and_no_positive_class_is_constructible() {
    use partman_domain::model::protection::TransportClass;

    for device in devices_of(&one_device_tree()) {
        assert_eq!(device.transport, TransportClass::Unrecognized);
    }
    let source = include_str!("devices.rs");
    for named in [
        "TransportClass::NvmePcie",
        "TransportClass::Sata",
        "TransportClass::SasDirect",
        "TransportClass::Usb",
        "TransportClass::SdMmc",
        "TransportClass::ParavirtualLocal",
        "TransportClass::RecognizedRemote",
    ] {
        assert!(
            !source.contains(named),
            "devices.rs names `{named}`: no positive transport class may be constructible \
             while the discrimination rows ADR-0018 owes remain outstanding"
        );
    }
}

// Requirements: INV-002, MODEL-004
//   The published roster and this crate's constants are one roster, not
//   two: every sysfs path and every database key this crate reads appears
//   in `schemas/adapter-linux/fields.md` verbatim, so a field cannot enter
//   the code without entering the document that records whether any
//   measurement supports reading it.
// Evidence: the_published_field_roster_matches_this_crates_constants
#[test]
fn the_published_field_roster_matches_this_crates_constants() {
    const DOC: &str = include_str!("../../../schemas/adapter-linux/fields.md");

    for (property, relative) in crate::devices::SYSFS_FIELDS {
        assert!(
            DOC.contains(&format!("`{relative}`")),
            "{property}: the document must carry this path and say what supports reading it"
        );
    }
    for wanted in crate::devices::UDEV_KEYS
        .iter()
        .chain(crate::devices::UDEV_SIGNATURE_KEYS)
    {
        assert!(
            DOC.contains(&format!("`{wanted}`")),
            "{wanted}: the document must carry this key and say what supports reading it"
        );
    }
    assert!(
        DOC.contains("**none**"),
        "the document must state plainly where a field has no measured row, or the roster \
         reads as though every field were evidenced"
    );
}

// Requirements: INV-002
//   No shipped module constructs an identity record or derives a strength
//   from one. SAFE-003's record carries a required table state whose every
//   variant is a determination INV-003 forbids this contract making, so an
//   honest client record is not constructible and the real one binds at
//   validation from the helper's own re-discovery. A text scan over every
//   shipped module, and its exact reach is the needle list in the test
//   body: it catches the construction, derivation, and import spellings,
//   while leaving the crate doc free to name the type in prose in order to
//   say why it is absent — which is the sentence this test backs.
// Evidence: no_shipped_module_constructs_an_identity_record
#[test]
fn no_shipped_module_constructs_an_identity_record() {
    for (name, source) in shipped_sources() {
        for needle in [
            "DeviceIdentity {",
            "DeviceIdentity::",
            "IdentityStrength",
            "model::identity",
            ".strength()",
        ] {
            assert!(
                !source.contains(needle),
                "{name} contains `{needle}`: this contract emits no identity record and derives \
                 no strength — the record binds at validation from the helper's re-discovery"
            );
        }
    }
}

/// A source that records every path read through it, so a test can assert
/// what was **not** read. ADR-0034's verification clause asks for exactly
/// that — "an undesignated class yields absent fields with no read attempted
/// against an undesignated source" — and no assertion over return values can
/// establish it.
struct RecordingSource {
    inner: FakeSource,
    reads: std::cell::RefCell<Vec<String>>,
}

impl RecordingSource {
    fn over(inner: FakeSource) -> Self {
        Self {
            inner,
            reads: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn read_any_named(&self, needle: &str) -> bool {
        self.reads
            .borrow()
            .iter()
            .any(|path| path.rsplit('/').next() == Some(needle))
    }
}

impl ContractSource for RecordingSource {
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        self.inner.list_dir(path)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        self.reads.borrow_mut().push(key(path));
        self.inner.read_bytes(path)
    }
}

/// A USB-attached device whose designated serial sits four ancestors above
/// the bus node — the depth the 2026-08-04 instrument measured — carrying
/// that sitting's own recorded values.
///
/// The tree plants a **decoy** `serial` on a nearer ancestor that carries no
/// USB markers. The decoy is the whole point of the fixture: ADR-0034's rule
/// is structural rather than a fixed traversal, so a search that stopped at
/// the first readable `serial` would take the wrong one, and a search that
/// counted to four would pass this fixture while failing every host whose
/// topology nests differently.
fn usb_device_tree(depth: usize) -> FakeSource {
    let mut dirs = BTreeMap::new();
    dirs.insert("/sys/class/block".to_owned(), Ok(vec!["sdb".to_owned()]));
    let mut files = BTreeMap::new();
    // FR5's measured sector count for the whole-device node.
    files.insert(
        "/sys/class/block/sdb/size".to_owned(),
        Ok(b"244457472\n".to_vec()),
    );
    // Two decoys, one on each side of the USB node, each answering `serial`
    // while carrying no USB markers. The nearer one defeats a search keyed
    // on the first readable `serial`; the farther one defeats a search that
    // walked to a fixed depth.
    for decoy in [1, depth + 1] {
        files.insert(
            format!("{}/serial", ancestor(decoy)),
            Ok(b"DECOY\n".to_vec()),
        );
    }
    // The USB device node, with both markers and R1's recorded serial.
    for (attribute, value) in [
        ("idVendor", &b"0781\n"[..]),
        ("idProduct", &b"5583\n"[..]),
        ("serial", &b"A20036CA8695D921\n"[..]),
    ] {
        files.insert(
            format!("{}/{attribute}", ancestor(depth)),
            Ok(value.to_vec()),
        );
    }
    FakeSource { dirs, files }
}

/// The path the walk reaches at one depth, spelled as the walk spells it:
/// parent components appended to the bus-node path, which is what the fake
/// keys on because the seam resolves no links.
fn ancestor(depth: usize) -> String {
    let mut path = "/sys/class/block/sdb/device".to_owned();
    for _ in 0..depth {
        path.push_str("/..");
    }
    path
}

/// The measured topology's depth: R1's instrument reached its USB ancestor
/// in four steps. Fixtures that need *a* USB device use this one; the test
/// that establishes the rule uses several.
fn one_usb_device_tree() -> FakeSource {
    usb_device_tree(4)
}

fn sdb() -> PathBuf {
    PathBuf::from("/sys/class/block/sdb")
}

// Requirements: INV-002
//   ADR-0034 designates "the `serial` attribute of the device's nearest
//   sysfs ancestor that is a USB device node", and says in terms that "the
//   resolution rule is structural, not a fixed traversal" — the measured
//   instrument's `device/../../../../serial` names the structure it
//   reached, not the rule. The decoy on a nearer non-USB ancestor is what
//   separates the two readings: a search keyed on the first readable
//   `serial` takes the decoy, and only one keyed on the USB-device-node
//   predicate reaches the designated source.
// Evidence: the_designated_serial_is_found_by_predicate_not_by_depth
#[test]
fn the_designated_serial_is_found_by_predicate_not_by_depth() {
    // Several depths, because one cannot separate the two readings: a search
    // hardcoded to the measured four steps passes a four-deep fixture. Each
    // tree also carries a nearer and a farther decoy `serial`, so the only
    // rule that answers all of these is the USB-device-node predicate.
    for depth in [1, 2, 4, 7] {
        let source = usb_device_tree(depth);
        let token = answered(&source);
        assert_eq!(
            crate::naming::designated_serial(&source, &sdb(), &token),
            crate::naming::DesignatedSource::Present(b"A20036CA8695D921\n".to_vec()),
            "at depth {depth} the designated source is the USB ancestor's, verbatim and \
             newline-included — never a decoy's"
        );
    }

    // Beyond the bound the walk refuses rather than running on, and refuses
    // to the undesignated arm: a device whose USB ancestor was never
    // identified has no designated source to have failed at.
    let source = usb_device_tree(crate::naming::ANCESTOR_LIMIT + 1);
    let token = answered(&source);
    assert_eq!(
        crate::naming::designated_serial(&source, &sdb(), &token),
        crate::naming::DesignatedSource::Undesignated
    );
}

// Requirements: SAFE-005
//   ADR-0034 leaves WWN undesignated on Linux for every attachment class,
//   and leaves serial undesignated for every class but USB. Its
//   verification clause makes the consequence observable rather than
//   merely stated: "an undesignated class yields absent fields with **no
//   read attempted** against an undesignated source". A test over returned
//   values cannot establish that, so this one records the reads and
//   asserts over what is missing from the record.
// Evidence: an_undesignated_cell_yields_an_absent_field_with_no_read_attempted
#[test]
fn an_undesignated_cell_yields_an_absent_field_with_no_read_attempted() {
    // A device with no USB ancestor anywhere: every attachment class but
    // USB, which is the designation table's catch-all row.
    let mut tree = one_usb_device_tree();
    tree.files.remove(&format!("{}/idVendor", ancestor(4)));
    let source = RecordingSource::over(tree);
    let token = answered(&source.inner);

    assert_eq!(
        crate::naming::designated_serial(&source, &sdb(), &token),
        crate::naming::DesignatedSource::Undesignated,
        "no USB ancestor means no designated serial source, not a failure"
    );
    assert!(
        !source.read_any_named("serial"),
        "no read may be attempted against a source the designation does not name"
    );

    // The WWN cell is undesignated on Linux unconditionally, so naming a
    // device must not read a WWN-shaped source either.
    let named = crate::naming::name_device(&source, &sdb(), &token, "device:0".to_owned());
    assert!(matches!(
        named,
        crate::naming::DeviceNaming::Addressed {
            fields: NamingFields::PhysicalDevice { wwn: None, .. },
            ..
        }
    ));
    for wwn_shaped in ["wwid", "wwn"] {
        assert!(
            !source.read_any_named(wwn_shaped),
            "WWN is undesignated on Linux, so `{wwn_shaped}` is never read for naming"
        );
    }
}

// Requirements: SAFE-005
//   ADR-0034's two outcome rules, at the layer that applies them rather
//   than the seam that reports them. A measured absence leaves the field
//   absent and the node an **operand** — a stable truth about the hardware
//   is a lawful weak name. A failed read is not absence: the node keeps
//   its remaining fields and stops being a plan operand. Reading the two
//   the same way would let a transient failure quietly widen what a plan
//   may target.
// Evidence: a_failed_designated_read_drops_operand_standing_and_an_absence_does_not
#[test]
fn a_failed_designated_read_drops_operand_standing_and_an_absence_does_not() {
    let mut absent = one_usb_device_tree();
    absent.files.remove(&format!("{}/serial", ancestor(4)));
    let token = answered(&absent);
    let named = crate::naming::name_device(&absent, &sdb(), &token, "device:0".to_owned());
    assert!(
        matches!(
            named,
            crate::naming::DeviceNaming::Addressed {
                operand_eligible: true,
                fields: NamingFields::PhysicalDevice { serial: None, .. },
            }
        ),
        "a measured absence is a lawful weak name on a node that stays an operand"
    );

    let mut failed = one_usb_device_tree();
    failed.files.insert(
        format!("{}/serial", ancestor(4)),
        Err(std::io::ErrorKind::PermissionDenied),
    );
    let token = answered(&failed);
    let named = crate::naming::name_device(&failed, &sdb(), &token, "device:0".to_owned());
    assert!(
        matches!(
            named,
            crate::naming::DeviceNaming::Addressed {
                operand_eligible: false,
                fields: NamingFields::PhysicalDevice { serial: None, .. },
            }
        ),
        "a failed read is not an absence: the node is indeterminate and not a plan operand"
    );
}

// Requirements: MODEL-001
//   `NamingFields::PhysicalDevice` carries a required `total_bytes`, so the
//   sector count is a prerequisite for addressing a device at all. The unit
//   is measured rather than conventional — FR5 read sysfs `size`
//   `244457472` against `blockdev --getsize64`'s `125162225664` on the
//   whole-device node — and every way the input can fail to be a sector
//   count refuses the address instead of guessing one. A device that
//   cannot be addressed is reported, never dropped: an enumerated device
//   silently omitted is the fail-open SAFE-005 puts on the refusing side.
// Evidence: a_device_is_addressed_from_the_measured_sector_unit_or_not_at_all
#[test]
fn a_device_is_addressed_from_the_measured_sector_unit_or_not_at_all() {
    let source = one_usb_device_tree();
    let token = answered(&source);
    match crate::naming::name_device(&source, &sdb(), &token, "device:0".to_owned()) {
        crate::naming::DeviceNaming::Addressed {
            fields: NamingFields::PhysicalDevice { total_bytes, .. },
            ..
        } => assert_eq!(
            total_bytes, 125_162_225_664,
            "FR5's measured pair: sysfs sectors times 512 equals the byte interface's answer"
        ),
        other => panic!("the fixture device must be addressable, got {other:?}"),
    }

    for (label, value) in [
        ("not decimal", &b"12x4\n"[..]),
        // The one arm the decimal guard catches that the integer parse does
        // not: Rust accepts a leading `+`. A sysfs `size` of `+123` is not a
        // sector count, and naming a device from it would derive an address
        // from a value the platform never reported.
        ("signed", &b"+123\n"[..]),
        ("empty", &b"\n"[..]),
        ("over the 64-bit range", &b"99999999999999999999\n"[..]),
        (
            "overflowing the byte product",
            format!("{}\n", u64::MAX / 8).as_bytes(),
        ),
    ] {
        let mut broken = one_usb_device_tree();
        broken
            .files
            .insert("/sys/class/block/sdb/size".to_owned(), Ok(value.to_vec()));
        let token = answered(&broken);
        assert!(
            matches!(
                crate::naming::name_device(&broken, &sdb(), &token, "device:0".to_owned()),
                crate::naming::DeviceNaming::Refused(_)
            ),
            "a sector count that is {label} refuses the address rather than guessing one"
        );
    }
}

// Requirements: MODEL-002
//   ADR-0019's collision group, over the population ADR-0034 names when it
//   prices this designation: "a same-model USB pair sharing a constant
//   descriptor serial (the S4-measured population) derives equal names and
//   groups — the design's intended representation of that ambiguity". Two
//   such devices must collapse into one counted, flagged, indeterminate
//   entry rather than into one device or into two addresses that a
//   consumer would read as distinct hardware.
// Evidence: a_same_serial_same_size_pair_collapses_into_a_counted_group
#[test]
fn a_same_serial_same_size_pair_collapses_into_a_counted_group() {
    let source = one_usb_device_tree();
    let token = answered(&source);
    let one = crate::naming::name_device(&source, &sdb(), &token, "device:0".to_owned());
    let other = crate::naming::name_device(&source, &sdb(), &token, "device:1".to_owned());

    let entries = crate::naming::absorb_devices(&[one, other]).expect("absorption is total");
    assert_eq!(entries.len(), 1, "two equal names are one entry, not two");
    match &entries[0] {
        NodeEntry::Group { count, .. } => assert_eq!(*count, 2),
        NodeEntry::Single { .. } => {
            panic!("a shared constant serial must group rather than address one device")
        }
    }
}

// Requirements: INV-002
//   ADR-0034's verification clause in its structural form: "no naming
//   input flows through `read_attribute`". The text path is preserved for
//   the observation rows it was built for, so the rule cannot be enforced
//   by deleting it — only by holding that the module which names nodes
//   does not call it. A source-text guard is the only check that stays
//   true as the module grows.
// Evidence: no_naming_input_flows_through_the_text_path
#[test]
fn no_naming_input_flows_through_the_text_path() {
    // Comment lines are stripped before the scan. The module documents why
    // it takes the bytes path, which means naming the path it does not take;
    // a guard that made that sentence unwritable would buy nothing and cost
    // the reader the reason.
    let code: String = include_str!("naming.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for needle in ["read_attribute", "AttributeRead"] {
        assert!(
            !code.contains(needle),
            "naming.rs calls `{needle}`: ADR-0019 takes identifier bytes verbatim, and the \
             text path transforms them"
        );
    }
}

fn sysfs_observation(method: Method, outcome: Outcome) -> Observation {
    Observation {
        adapter: Interface::Sysfs.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method,
        outcome,
    }
}

fn observed(text: &str) -> Outcome {
    Outcome::Observed {
        value: partman_domain::canonical::Value::Text(text.to_owned()),
    }
}

/// A geometry input set, keyed exactly as the device roster publishes it.
fn geometry(
    logical: Vec<Observation>,
    physical: Vec<Observation>,
) -> Vec<(String, PropertyObservations)> {
    vec![
        (
            format!(
                "{}:{}",
                Interface::Sysfs.label(),
                crate::derivation::LOGICAL_BLOCK_SIZE
            ),
            PropertyObservations {
                observations: logical,
            },
        ),
        (
            format!(
                "{}:{}",
                Interface::Sysfs.label(),
                crate::derivation::PHYSICAL_BLOCK_SIZE
            ),
            PropertyObservations {
                observations: physical,
            },
        ),
    ]
}

// Requirements: INV-004, MODEL-004
//   ADR-0033's imported obligation, with a fixture for each arm. A
//   derivation over an input whose set derives `unavailable` or
//   `conflicting` MUST NOT be presented as a value — the input's own state
//   is surfaced instead, so a guess never wears a computation's clothes —
//   while an `inferred` input IS fit, because the input's confidence
//   travels by reference rather than being copied onto the derivation. The
//   `conflicting` fixture is hand-built: this adapter keys each property by
//   the interface that answered, so production cannot currently produce a
//   plural set, and the arm would otherwise go untested rather than
//   unreachable.
// Evidence: the_alignment_derivation_is_presented_only_over_fit_inputs
#[test]
fn the_alignment_derivation_is_presented_only_over_fit_inputs() {
    let direct = |text: &str| vec![sysfs_observation(Method::Direct, observed(text))];

    // Authoritative inputs: presented. FR2's measured real-hardware pair.
    assert_eq!(
        crate::derivation::alignment(&geometry(direct("512"), direct("512"))),
        crate::derivation::Derived::Presented(crate::derivation::Alignment {
            logical_bytes: 512,
            granularity_bytes: 512,
        })
    );

    // An inferred input is fit, and the derivation carries no confidence of
    // its own to record that it was.
    assert!(matches!(
        crate::derivation::alignment(&geometry(
            vec![sysfs_observation(Method::Heuristic, observed("512"))],
            direct("4096"),
        )),
        crate::derivation::Derived::Presented(_)
    ));

    // Unavailable: the input's own state is what is surfaced.
    let unavailable = vec![sysfs_observation(
        Method::Direct,
        Outcome::Unavailable {
            reason: "the interface did not answer".to_owned(),
        },
    )];
    assert!(matches!(
        crate::derivation::alignment(&geometry(unavailable, direct("512"))),
        crate::derivation::Derived::Withheld(crate::derivation::Withheld::InputState {
            state: Confidence::Unavailable,
            ..
        })
    ));

    // Conflicting: two reads disagree, and the derivation must not pick one.
    // Both values parse, which is the point — a value-first implementation
    // would present a number here and say nothing disagreed.
    let conflicting = vec![
        sysfs_observation(Method::Direct, observed("512")),
        sysfs_observation(Method::Direct, observed("4096")),
    ];
    assert!(matches!(
        crate::derivation::alignment(&geometry(conflicting, direct("512"))),
        crate::derivation::Derived::Withheld(crate::derivation::Withheld::InputState {
            state: Confidence::Conflicting,
            ..
        })
    ));

    // Fit by confidence, and still nothing to compute with. A positively
    // determined absence is `authoritative`, which is exactly why this arm
    // cannot be folded into the one above.
    for unusable in [
        vec![sysfs_observation(Method::Direct, Outcome::ObservedAbsent)],
        direct("not-a-number"),
        direct("0"),
    ] {
        assert!(
            matches!(
                crate::derivation::alignment(&geometry(unusable, direct("512"))),
                crate::derivation::Derived::Withheld(
                    crate::derivation::Withheld::NoUsableValue { .. }
                )
            ),
            "an input with no usable value withholds rather than guessing"
        );
    }

    // A property the contract never reported at all.
    assert!(matches!(
        crate::derivation::alignment(&[]),
        crate::derivation::Derived::Withheld(crate::derivation::Withheld::NoUsableValue { .. })
    ));
}

// Requirements: INV-004
//   INV-004: the free-extent derivation "MUST NOT be presented at all where
//   the host declares a table scheme the build cannot name, or where a
//   partition the authenticated names place in the host's address space is
//   not one the derivation subtracts". Both grounds hold for this contract
//   and for one reason — it builds no partition-table node — which is the
//   branch ADR-0036's forward obligation puts to this increment. The
//   refusal is a surface rather than an omission, because an absent surface
//   and a refusing one are different things to a consumer.
// Evidence: the_free_extent_derivation_is_not_presented_on_this_contract
#[test]
fn the_free_extent_derivation_is_not_presented_on_this_contract() {
    assert!(matches!(
        crate::derivation::free_extents(),
        crate::derivation::Derived::Withheld(crate::derivation::Withheld::NotPresented { .. })
    ));
}

// Requirements: INV-004, MODEL-004
//   ADR-0033's other half: a derivation "is never stored, and carries no
//   observation set and no confidence of its own". Held structurally rather
//   than by review — the stored device shape names neither derivation, so
//   there is no field in which one could be written down, and the
//   derivation result type carries no provenance member to copy a
//   confidence into.
// Evidence: no_derivation_is_stored_and_none_carries_provenance
#[test]
fn no_derivation_is_stored_and_none_carries_provenance() {
    let devices = include_str!("devices.rs");
    for needle in ["alignment", "free_extent"] {
        assert!(
            !devices.contains(needle),
            "devices.rs names `{needle}`: a derivation recomputed at use has no stored home"
        );
    }
    let derivation: String = include_str!("derivation.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "PropertyObservations {",
        "Confidence::Authoritative",
        "Confidence::Inferred",
    ] {
        assert!(
            !derivation.contains(needle),
            "derivation.rs constructs `{needle}`: a derivation carries neither an observation \
             set nor a confidence of its own"
        );
    }
}

// Requirements: SAFE-002
//   Both structural guards iterate one fixed roster of shipped modules, so
//   a module added without an entry would be exempt from both while leaving
//   both tests green — two passing tests asserting nothing about the new
//   code. This pins the roster against the crate's own module declarations,
//   making that omission a failure rather than a silent hole.
// Evidence: every_shipped_module_is_covered_by_the_structural_guards
#[test]
fn every_shipped_module_is_covered_by_the_structural_guards() {
    let declared: Vec<String> = include_str!("lib.rs")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .filter_map(|rest| rest.strip_suffix(';'))
        .map(|name| format!("{name}.rs"))
        .collect();
    let covered: Vec<&str> = shipped_sources().iter().map(|(name, _)| *name).collect();

    for name in &declared {
        assert!(
            covered.contains(&name.as_str()),
            "{name} is a shipped module the structural guards do not scan"
        );
    }
    assert_eq!(
        covered.len(),
        declared.len() + 1,
        "the guarded roster is the declared modules plus lib.rs itself; a stale extra entry \
         hides a deleted module and a missing one hides a new module"
    );
}

// ---------------------------------------------------------------------------
// Increment 4a: the kind markers, and the state layer.
// ---------------------------------------------------------------------------

/// A tree of five whole devices — a plain disk, a loop device, a dm node, an
/// md array, and one whose `dm` marker cannot be listed — each with the
/// attributes the naming path needs, so the same tree serves the withdrawal
/// and the keying. Marker directories are DR3's; the values are shapes.
fn assembled_tree() -> FakeSource {
    let mut dirs = BTreeMap::new();
    dirs.insert(
        "/sys/class/block".to_owned(),
        Ok(vec![
            "dm-0".to_owned(),
            "loop6".to_owned(),
            "md127".to_owned(),
            "sdm".to_owned(),
            "shady".to_owned(),
        ]),
    );
    dirs.insert(
        "/sys/class/block/dm-0/dm".to_owned(),
        Ok(vec!["name".to_owned(), "uuid".to_owned()]),
    );
    dirs.insert(
        "/sys/class/block/loop6/loop".to_owned(),
        Ok(vec!["backing_file".to_owned()]),
    );
    dirs.insert(
        "/sys/class/block/md127/md".to_owned(),
        Ok(vec!["level".to_owned(), "raid_disks".to_owned()]),
    );
    dirs.insert(
        "/sys/class/block/shady/dm".to_owned(),
        Err(std::io::ErrorKind::PermissionDenied),
    );
    let mut files = BTreeMap::new();
    for (name, number) in [
        ("dm-0", "253:0"),
        ("loop6", "7:6"),
        ("md127", "9:127"),
        ("sdm", "8:192"),
        ("shady", "253:9"),
    ] {
        files.insert(
            format!("/sys/class/block/{name}/dev"),
            Ok(format!("{number}\n").into_bytes()),
        );
        files.insert(
            format!("/sys/class/block/{name}/size"),
            Ok(b"2097152\n".to_vec()),
        );
    }
    FakeSource { dirs, files }
}

fn named(source: &FakeSource, name: &str, selector: &str) -> crate::naming::DeviceNaming {
    let token = answered(source);
    crate::naming::name_device(
        source,
        &PathBuf::from(format!("/sys/class/block/{name}")),
        &token,
        selector.to_owned(),
    )
}

// Requirements: INV-001, LIN-006, SAFE-005
//   DR3 establishes that `dm/`, `md/`, and `loop/` positively mark a
//   device-mapper node, an mdraid array, and a loop device, and that a plain
//   disk carries none of them. Increment 3a admitted every one of those
//   nodes as an operand-eligible `PhysicalDevice`; this withdraws them: a
//   marker positively present reports the node as host-assembled and names
//   it nothing, every marker positively absent admits a plain disk to the
//   naming path, and a marker whose listing did not answer refuses — the
//   `partition` discipline again, because admitting on a failed read would
//   name a loop device a physical device on the strength of nothing.
// Evidence: a_host_assembled_node_is_withdrawn_and_an_undetermined_marker_refuses
#[test]
fn a_host_assembled_node_is_withdrawn_and_an_undetermined_marker_refuses() {
    use crate::devices::{DeviceKind, HostAssembledKind};
    use crate::naming::DeviceNaming;

    let source = assembled_tree();
    let devices = devices_of(&source);
    assert_eq!(
        devices.len(),
        5,
        "every node lacking `partition` is enumerated"
    );
    let kind_of = |name: &str| {
        devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(name))
            .map(|d| d.kind.clone())
            .expect("the device is enumerated")
    };
    assert_eq!(
        kind_of("253:0"),
        DeviceKind::HostAssembled(HostAssembledKind::DeviceMapper)
    );
    assert_eq!(
        kind_of("7:6"),
        DeviceKind::HostAssembled(HostAssembledKind::Loop)
    );
    assert_eq!(
        kind_of("9:127"),
        DeviceKind::HostAssembled(HostAssembledKind::Mdraid)
    );
    assert_eq!(kind_of("8:192"), DeviceKind::Plain);
    assert!(
        matches!(kind_of("253:9"), DeviceKind::Indeterminate { .. }),
        "a marker that did not answer leaves the kind undetermined, never plain"
    );

    // The naming path honours the verdict.
    assert!(matches!(
        named(&source, "loop6", "device:1"),
        DeviceNaming::Withdrawn {
            kind: HostAssembledKind::Loop,
            ..
        }
    ));
    assert!(matches!(
        named(&source, "dm-0", "device:0"),
        DeviceNaming::Withdrawn {
            kind: HostAssembledKind::DeviceMapper,
            ..
        }
    ));
    assert!(matches!(
        named(&source, "sdm", "device:3"),
        DeviceNaming::Addressed { .. }
    ));
    match named(&source, "shady", "device:4") {
        DeviceNaming::Refused(refusal) => assert!(
            refusal.reason.contains("undetermined"),
            "the refusal names the undetermined kind: {}",
            refusal.reason
        ),
        other => panic!("an undetermined marker must refuse, got {other:?}"),
    }

    // And absorption sees exactly the one plain disk.
    let all: Vec<_> = ["dm-0", "loop6", "md127", "sdm", "shady"]
        .iter()
        .enumerate()
        .map(|(i, name)| named(&source, name, &format!("device:{i}")))
        .collect();
    let entries = crate::naming::absorb_devices(&all).expect("absorption is total");
    assert_eq!(
        entries.len(),
        1,
        "withdrawn and refused devices derive no address; only the plain disk absorbs"
    );
}

/// DR1's recorded lines: the guest's root, a pseudo file system, the
/// whole-disk ext4, the loop ext4, the LVM ext4, and the Btrfs mount whose
/// `major:minor` is anonymous.
const MOUNTINFO: &str = "\
30 1 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw,discard,errors=remount-ro
26 30 0:24 / /proc rw,nosuid,nodev,noexec,relatime shared:13 - proc proc rw
298 30 8:192 / /mnt/dr-ext4 rw,relatime shared:240 - ext4 /dev/sdm rw
328 30 7:6 / /mnt/dr-loop rw,relatime shared:247 - ext4 /dev/loop6 rw
343 30 253:0 / /mnt/dr-lv rw,relatime shared:254 - ext4 /dev/mapper/vg_dr_a-lv_a rw
299 30 0:43 / /mnt/dr-btrfs rw,relatime shared:241 - btrfs /dev/sdk rw,space_cache=v2,subvolid=5,subvol=/
";

fn procfs(mountinfo: &[u8], swaps: &[u8]) -> FakeSource {
    let mut source = assembled_tree();
    source
        .files
        .insert("/proc/self/mountinfo".to_owned(), Ok(mountinfo.to_vec()));
    source
        .files
        .insert("/proc/swaps".to_owned(), Ok(swaps.to_vec()));
    source
}

// Requirements: INV-004, MODEL-004, LIN-006
//   DR1 establishes the mount table's shape and its keying field for an
//   ordinary client, and that a Btrfs mount's `major:minor` is an
//   anonymous device that names its member only in the source field. Every
//   line becomes one attributed procfs observation carrying the line
//   verbatim, parsed into its documented fields with no transformation;
//   keying to the admitted devices is by `major:minor` alone — a device
//   without a device number keys nothing, a mount whose source path names a
//   device but whose number is another's keys to the number — and the
//   anonymous, partition, and pseudo entries stay unkeyed rather than being
//   guessed at. Loop devices are withdrawn from naming and still key: the
//   mount is a state fact about a node the adapter admits, not a name.
// Evidence: the_mount_table_parses_in_the_recorded_shape_and_keys_by_major_minor
#[test]
fn the_mount_table_parses_in_the_recorded_shape_and_keys_by_major_minor() {
    use crate::state::{Table, key_mounts, read_mounts};
    use partman_domain::model::provenance::{Method, Outcome};

    let source = procfs(
        MOUNTINFO.as_bytes(),
        b"Filename\tType\tSize\tUsed\tPriority\n",
    );
    let devices = devices_of(&source);
    let Table::Listed { entries } = read_mounts(&source, &PathBuf::from("/proc")) else {
        panic!("the recorded table parses")
    };
    assert_eq!(entries.len(), 6);
    let ext4 = &entries[2];
    assert_eq!((ext4.mount_id, ext4.parent_id), (298, 30));
    assert_eq!((ext4.major, ext4.minor), (8, 192));
    assert_eq!(ext4.mount_point, "/mnt/dr-ext4");
    assert_eq!(ext4.optional_fields, vec!["shared:240".to_owned()]);
    assert_eq!(ext4.fs_type, "ext4");
    assert_eq!(ext4.source, "/dev/sdm");
    assert_eq!(ext4.super_options, "rw");
    assert_eq!(
        ext4.observation.adapter,
        "partman-adapter-linux/linux-procfs"
    );
    assert_eq!(ext4.observation.method, Method::Direct);
    assert!(matches!(
        &ext4.observation.outcome,
        Outcome::Observed { value: partman_domain::canonical::Value::Text(line) }
            if line == MOUNTINFO.lines().nth(2).unwrap()
    ));
    let btrfs = &entries[5];
    assert_eq!(
        (btrfs.major, btrfs.minor),
        (0, 43),
        "DR1: the Btrfs mount is anonymous"
    );
    assert_eq!(btrfs.source, "/dev/sdk");
    assert_eq!(btrfs.super_options, "rw,space_cache=v2,subvolid=5,subvol=/");

    let keyed = key_mounts(&entries, &devices);
    let selectors: Vec<&str> = keyed.by_device.iter().map(|(s, _)| *s).collect();
    let selector_of = |number: &str| {
        devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.selector.as_str())
            .unwrap()
    };
    assert_eq!(
        selectors,
        vec![
            selector_of("8:192"),
            selector_of("7:6"),
            selector_of("253:0")
        ],
        "the whole disk, the withdrawn loop, and the dm node each key their mount"
    );
    assert_eq!(keyed.by_device[0].1[0].mount_point, "/mnt/dr-ext4");
    let unkeyed: Vec<&str> = keyed
        .unkeyed
        .iter()
        .map(|e| e.mount_point.as_str())
        .collect();
    assert_eq!(
        unkeyed,
        vec!["/", "/proc", "/mnt/dr-btrfs"],
        "the partition-backed root, the pseudo file system, and the anonymous Btrfs mount key to nothing"
    );

    // Keying is by number, never by the source path: a device that lost its
    // `dev` attribute keys nothing even though a line's source names it, and
    // a line whose source names one device but carries another's number
    // keys to the number.
    let mut renamed = procfs(
        b"298 30 8:192 / /mnt/x rw shared:1 - ext4 /dev/loop6 rw\n",
        b"Filename\n",
    );
    renamed.files.remove("/sys/class/block/loop6/dev");
    let devices = devices_of(&renamed);
    let Table::Listed { entries } = read_mounts(&renamed, &PathBuf::from("/proc")) else {
        panic!("parses")
    };
    let keyed = key_mounts(&entries, &devices);
    assert_eq!(keyed.by_device.len(), 1);
    assert_eq!(
        devices
            .iter()
            .find(|d| d.selector == keyed.by_device[0].0)
            .and_then(|d| d.device_number.as_deref()),
        Some("8:192"),
        "the number wins over the name"
    );
    assert!(keyed.unkeyed.is_empty());
}

// Requirements: SAFE-005, MODEL-004
//   A line off the recorded shape — no separator, the wrong count after
//   it, a non-numeric id, a malformed `major:minor` — refuses the whole
//   table as a `failed` observation naming the line, never a partial list:
//   a partial mount set could present a mounted device as unmounted. An
//   absent table is `unavailable`, never an empty table; an over-limit read
//   refuses rather than truncating, under the table's own bound.
// Evidence: a_mount_line_off_the_recorded_shape_refuses_the_whole_table
#[test]
fn a_mount_line_off_the_recorded_shape_refuses_the_whole_table() {
    use crate::state::{Table, read_mounts};
    use partman_domain::model::provenance::Outcome;

    let good = "30 1 8:1 / / rw shared:1 - ext4 /dev/sda1 rw\n";
    for (bad, what) in [
        (
            "26 30 0:24 / /proc rw shared:13 proc proc rw\n",
            "no `-` separator",
        ),
        (
            "26 30 0:24 / /proc rw shared:13 - proc proc\n",
            "two fields after the separator",
        ),
        (
            "26 x 0:24 / /proc rw shared:13 - proc proc rw\n",
            "non-numeric parent id",
        ),
        (
            "26 30 0-24 / /proc rw shared:13 - proc proc rw\n",
            "malformed major:minor",
        ),
        (
            "26 30 0:24 / - proc proc rw\n",
            "fewer than six fields before the separator",
        ),
    ] {
        let source = procfs(format!("{good}{bad}").as_bytes(), b"Filename\n");
        match read_mounts(&source, &PathBuf::from("/proc")) {
            Table::Refused { observation } => match observation.outcome {
                Outcome::Failed { error } => assert!(
                    error.contains("line 2") && error.contains("refused, not read partially"),
                    "{what}: {error}"
                ),
                other => panic!("{what}: a malformed table is failed, got {other:?}"),
            },
            Table::Listed { .. } => panic!("{what}: a malformed line must refuse the table"),
        }
    }

    // Absent: unavailable, never empty.
    let absent = assembled_tree();
    assert!(matches!(
        read_mounts(&absent, &PathBuf::from("/proc")),
        Table::Refused {
            observation: partman_domain::model::provenance::Observation {
                outcome: Outcome::Unavailable { .. },
                ..
            }
        }
    ));

    // Over the table's own bound: refused, not truncated. The bound is the
    // table's, wider than a device record's.
    const { assert!(crate::contract::TABLE_LIMIT > crate::contract::RECORD_LIMIT) };
    let big = procfs(&vec![b'x'; crate::contract::TABLE_LIMIT + 1], b"Filename\n");
    match read_mounts(&big, &PathBuf::from("/proc")) {
        Table::Refused { observation } => assert!(matches!(
            observation.outcome,
            Outcome::Failed { ref error } if error.contains("not truncated")
        )),
        Table::Listed { .. } => panic!("an over-limit table must refuse"),
    }
}

// Requirements: LIN-006, MODEL-004, SAFE-005
//   DR2 establishes the swap table's shape for an ordinary client: the
//   header, then one row per active swap. A row parses into its five
//   fields as an attributed observation; a table not opening with the
//   header, or a row without exactly five fields, refuses.
// Evidence: the_swap_table_parses_and_a_missing_header_refuses
#[test]
fn the_swap_table_parses_and_a_missing_header_refuses() {
    use crate::state::{Table, read_swaps};
    use partman_domain::model::provenance::Outcome;

    let table = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n\
                 /dev/sdn                                partition\t1048572\t\t0\t\t-2\n";
    let source = procfs(b"", table.as_bytes());
    let Table::Listed { entries } = read_swaps(&source, &PathBuf::from("/proc")) else {
        panic!("DR2's table parses")
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, "/dev/sdn");
    assert_eq!(entries[0].kind, "partition");
    assert_eq!(
        (
            entries[0].size_kib,
            entries[0].used_kib,
            entries[0].priority
        ),
        (1_048_572, 0, -2)
    );
    assert_eq!(
        entries[0].observation.adapter,
        "partman-adapter-linux/linux-procfs"
    );

    // Header only: an empty, lawful table.
    let empty = procfs(b"", b"Filename\tType\tSize\tUsed\tPriority\n");
    assert!(matches!(
        read_swaps(&empty, &PathBuf::from("/proc")),
        Table::Listed { entries } if entries.is_empty()
    ));

    for (bad, what) in [
        ("/dev/sdn partition 1048572 0 -2\n", "no header"),
        (
            "Filename Type Size Used Priority\n/dev/sdn partition 1048572 0\n",
            "four fields",
        ),
        (
            "Filename Type Size Used Priority\n/dev/sdn partition big 0 -2\n",
            "non-numeric size",
        ),
    ] {
        let source = procfs(b"", bad.as_bytes());
        assert!(
            matches!(
                read_swaps(&source, &PathBuf::from("/proc")),
                Table::Refused {
                    observation: partman_domain::model::provenance::Observation {
                        outcome: Outcome::Failed { .. },
                        ..
                    }
                }
            ),
            "{what} must refuse"
        );
    }
}

// ---------------------------------------------------------------------------
// Increment 4b, first slice: mdraid arrays as designator-absent aggregates.
// ---------------------------------------------------------------------------

/// The assembled tree plus a second array, both with DR5's `md/raid_disks`
/// and DR4's `slaves/`, and one array whose count is not a count.
fn arrays_tree() -> FakeSource {
    let mut source = assembled_tree();
    source.dirs.insert(
        "/sys/class/block".to_owned(),
        Ok(vec![
            "dm-0".to_owned(),
            "loop6".to_owned(),
            "md126".to_owned(),
            "md127".to_owned(),
            "md99".to_owned(),
            "sdm".to_owned(),
            "shady".to_owned(),
        ]),
    );
    for (name, number, count, members) in [
        ("md127", "9:127", "2\n", vec!["sde", "sdf"]),
        ("md126", "9:126", "2\n", vec!["sdg", "sdh"]),
        ("md99", "9:99", "two\n", vec!["sdi"]),
    ] {
        source.dirs.insert(
            format!("/sys/class/block/{name}/md"),
            Ok(vec!["level".to_owned(), "raid_disks".to_owned()]),
        );
        source.dirs.insert(
            format!("/sys/class/block/{name}/slaves"),
            Ok(members.iter().map(|m| (*m).to_owned()).collect()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/dev"),
            Ok(format!("{number}\n").into_bytes()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/size"),
            Ok(b"2093056\n".to_vec()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/md/raid_disks"),
            Ok(count.as_bytes().to_vec()),
        );
    }
    source
}

// Requirements: LIN-006, INV-004, SAFE-005
//   The Linux host-assembled designation round found no source that may
//   name an mdraid array under ADR-0034's discipline, so this slice names
//   none: every array admitted by DR3's `md/` marker is reported as an
//   `Aggregate { Mdraid, designator: None }` — the fail-closed
//   representation ADR-0019 decides and slice 3q enforces — carrying its
//   self-reported member count read from DR5's `md/raid_disks` (a decimal
//   or a refusal, never a guess) and the kernel's `slaves/` listing as a
//   report, not an edge (DR4: per-mapping). Only marker-admitted arrays
//   are reported: the plain disk, the dm node, the loop, and the
//   undetermined device are not arrays whatever their entries say. Two
//   arrays absorb into one collision group; and the domain's closure gives
//   the lone designator-absent aggregate the missing-fact indeterminacy
//   slice 3q added — the arc's two halves meeting in one assertion.
// Evidence: mdraid_arrays_are_reported_as_designator_absent_aggregates_and_never_operands
#[test]
fn mdraid_arrays_are_reported_as_designator_absent_aggregates_and_never_operands() {
    use crate::arrays::{MemberCount, Members, absorb_arrays, report_arrays};
    use partman_domain::model::naming::{AggregateTechnology, NamingFields, NodeEntry, derive_id};
    use partman_domain::model::protection::{Facts, IndeterminateGround, Verdict, node_verdict};
    use partman_domain::model::topology::Topology;

    let source = arrays_tree();
    let devices = devices_of(&source);
    let reports = report_arrays(&source, &PathBuf::from("/sys"), &devices);
    assert_eq!(
        reports.len(),
        3,
        "exactly the marker-admitted arrays are reported"
    );
    for report in &reports {
        assert_eq!(
            report.fields,
            NamingFields::Aggregate {
                technology: AggregateTechnology::Mdraid,
                designator: None,
            },
            "this slice names nothing"
        );
    }
    let by_selector = |number: &str| {
        let selector = devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.selector.clone())
            .unwrap();
        reports.iter().find(|r| r.selector == selector).unwrap()
    };
    assert_eq!(by_selector("9:127").member_count, MemberCount::Reported(2));
    assert_eq!(
        by_selector("9:127").members,
        Members::Listed(vec!["sde".to_owned(), "sdf".to_owned()])
    );
    assert_eq!(by_selector("9:126").member_count, MemberCount::Reported(2));
    assert!(
        matches!(
            by_selector("9:99").member_count,
            MemberCount::Refused { .. }
        ),
        "a count that is not a count is refused, never guessed"
    );

    // Two or more absorb into one indeterminate group; alone, the closure
    // refuses through slice 3q's arm.
    let entries = absorb_arrays(&reports).expect("absorption is total");
    assert_eq!(entries.len(), 1);
    assert!(matches!(entries[0], NodeEntry::Group { count: 3, .. }));

    let lone = absorb_arrays(&reports[..1]).expect("absorption is total");
    let NodeEntry::Single { ref fields, .. } = lone[0] else {
        panic!("one array absorbs alone")
    };
    let id = derive_id(fields).expect("derivable");
    let topology = Topology::build(vec![fields.clone()], vec![]).expect("builds");
    assert!(
        matches!(
            node_verdict(&topology, &Facts::default(), id),
            Verdict::Indeterminate {
                cause: IndeterminateGround::MissingFact
            }
        ),
        "the designator-absent array is not an operand — the domain arm slice 3q added"
    );
}

// ---------------------------------------------------------------------------
// Increment 4b, second slice: ADR-0053's designations.
// ---------------------------------------------------------------------------

/// The arrays tree plus `md/uuid` on two arrays (one absent, one
/// unreadable elsewhere), three dm nodes — two LVM logical volumes in two
/// volume-group classes and one opened container — a dm node whose uuid
/// does not answer, and the loop's backing path. Values are DR11/DR12/DR13
/// shapes; the uuids are the sitting's own.
fn designated_tree() -> FakeSource {
    let mut source = arrays_tree();
    source.dirs.insert(
        "/sys/class/block".to_owned(),
        Ok(vec![
            "dm-0".to_owned(),
            "dm-1".to_owned(),
            "dm-2".to_owned(),
            "dm-3".to_owned(),
            "dm-9".to_owned(),
            "loop6".to_owned(),
            "md126".to_owned(),
            "md127".to_owned(),
            "md99".to_owned(),
            "sdm".to_owned(),
            "shady".to_owned(),
        ]),
    );
    // DR11: md/uuid present on both arrays; the third array's is unreadable.
    source.files.insert(
        "/sys/class/block/md127/md/uuid".to_owned(),
        Ok(b"54b95c15-7548-d8fb-52b0-5c2ff4f5d9f2\n".to_vec()),
    );
    source.files.insert(
        "/sys/class/block/md126/md/uuid".to_owned(),
        Ok(b"1bdd6d6c-70b6-a01f-48e0-9517d541c4db\n".to_vec()),
    );
    source.files.insert(
        "/sys/class/block/md99/md/uuid".to_owned(),
        Err(std::io::ErrorKind::PermissionDenied),
    );
    // dm nodes: two LVs in two VG classes, one container, one silent.
    for (name, number, uuid, dmname) in [
        (
            "dm-0",
            "253:0",
            "LVM-ek99dYwwU1KaulyX1bqr3RJC2pGrYoWOcbq0yOdyE6EodBHuixHfFrHBIqyXf8Zw\n",
            "vg_dr_a-lv_a\n",
        ),
        (
            "dm-1",
            "253:1",
            "LVM-38AMHrVVxZ2ceGKT6AOPeP27yUz45eXZCozrUTTbf6F2hPbPclDK6IxcMF7eiEVw\n",
            "vg_dr_b-lv_b\n",
        ),
        (
            "dm-2",
            "253:2",
            "CRYPT-LUKS2-de5df2cca1a841ed94d64ebafb2b45e4-cr_a\n",
            "cr_a\n",
        ),
        (
            "dm-3",
            "253:3",
            "LVM-ek99dYwwU1KaulyX1bqr3RJC2pGrYoWOcbq0yOdyE6EodBHuixHfFrHBIqyXf8Zw\n",
            "vg_dr_a-lv_c\n",
        ),
    ] {
        source.dirs.insert(
            format!("/sys/class/block/{name}/dm"),
            Ok(vec!["name".to_owned(), "uuid".to_owned()]),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/dev"),
            Ok(format!("{number}\n").into_bytes()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/size"),
            Ok(b"524288\n".to_vec()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/dm/uuid"),
            Ok(uuid.as_bytes().to_vec()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/dm/name"),
            Ok(dmname.as_bytes().to_vec()),
        );
    }
    // dm-3 is an LV in dm-0's class; dm-9 has a dm marker but no readable uuid.
    source.dirs.insert(
        "/sys/class/block/dm-9/dm".to_owned(),
        Ok(vec!["name".to_owned()]),
    );
    source.files.insert(
        "/sys/class/block/dm-9/dev".to_owned(),
        Ok(b"253:9\n".to_vec()),
    );
    source
        .files
        .insert("/sys/class/block/dm-9/size".to_owned(), Ok(b"8\n".to_vec()));
    source.files.insert(
        "/sys/class/block/dm-9/dm/name".to_owned(),
        Ok(b"mystery\n".to_vec()),
    );
    // DR13: the loop's backing path.
    source.files.insert(
        "/sys/class/block/loop6/loop/backing_file".to_owned(),
        Ok(b"/var/tmp/dr-loop.img\n".to_vec()),
    );
    source
}

// Requirements: LIN-006, INV-004, SAFE-005
//   ADR-0053's mdraid cell: the designator is the array's `md/uuid` bytes
//   verbatim, trailing newline included, read through the bytes-preserving
//   path (DR11); an array whose source is unreadable keeps the
//   designator-absent name and standing (ADR-0034's failed-read outcome),
//   and the udev cache's `MD_UUID` is not read for naming. Two named
//   arrays absorb as two nodes; the designator-absent one absorbs alone
//   as an indeterminate non-operand.
// Evidence: an_mdraid_array_names_from_md_uuid_verbatim_and_an_unreadable_one_stays_absent
#[test]
fn an_mdraid_array_names_from_md_uuid_verbatim_and_an_unreadable_one_stays_absent() {
    use crate::arrays::{DesignatorRead, absorb_arrays, report_arrays};
    use partman_domain::model::naming::{AggregateTechnology, NamingFields, NodeEntry};

    let source = designated_tree();
    let devices = devices_of(&source);
    let reports = report_arrays(&source, &PathBuf::from("/sys"), &devices);
    assert_eq!(reports.len(), 3);
    let by_number = |number: &str| {
        let selector = devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.selector.clone())
            .unwrap();
        reports.iter().find(|r| r.selector == selector).unwrap()
    };
    let a = by_number("9:127");
    assert_eq!(a.designator, DesignatorRead::Present);
    assert_eq!(
        a.fields,
        NamingFields::Aggregate {
            technology: AggregateTechnology::Mdraid,
            designator: Some(b"54b95c15-7548-d8fb-52b0-5c2ff4f5d9f2\n".to_vec()),
        },
        "the designator is the md/uuid bytes verbatim — newline included, no colon-quartet re-spelling"
    );
    assert!(matches!(
        by_number("9:99").designator,
        DesignatorRead::Unreadable { .. }
    ));
    assert!(matches!(
        by_number("9:99").fields,
        NamingFields::Aggregate {
            designator: None,
            ..
        }
    ));
    let entries = absorb_arrays(&reports).expect("absorption is total");
    assert_eq!(
        entries.len(),
        3,
        "two named arrays and one designator-absent: three addresses"
    );
    assert!(
        entries
            .iter()
            .all(|e| matches!(e, NodeEntry::Single { .. }))
    );
}

// Requirements: LIN-006, INV-004, SAFE-005
//   ADR-0053's dm cells: a device-mapper node is classified by its
//   `dm/uuid` prefix — `LVM-` a logical volume, `CRYPT-` a container,
//   anything else unrecognized, a silent uuid undetermined — never by its
//   entry name; a logical volume is a `Volume` named from `dm/name` bytes
//   verbatim under the designator-absent LVM2 aggregate as its producer
//   (role absent, no client-readable VG id); volume-group classes partition
//   the volumes and set the group count, and enter no name; a container
//   yields no `Volume`; the loop's `loop/backing_file` is reported and no
//   node is built (3b's host node). Absorbed, the two group classes collapse
//   into one collision group of count two, and each volume names under it;
//   under the closure the volumes inherit their group's indeterminacy.
// Evidence: dm_nodes_are_classified_by_uuid_prefix_and_only_lvm_volumes_are_named
#[test]
fn dm_nodes_are_classified_by_uuid_prefix_and_only_lvm_volumes_are_named() {
    use crate::volumes::{
        MappingKind, SourceRead, absorb_mappings, lvm_group_fields, report_mappings,
    };
    use partman_domain::model::naming::{NamingFields, NodeEntry, derive_id};
    use partman_domain::model::protection::{Facts, Verdict, node_verdict};
    use partman_domain::model::topology::{Edge, EdgeKind, Topology};

    let source = designated_tree();
    let devices = devices_of(&source);
    let mappings = report_mappings(&source, &PathBuf::from("/sys"), &devices);
    let selector_of = |number: &str| {
        devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.selector.clone())
            .unwrap()
    };
    let kind_of = |number: &str| {
        mappings
            .mappings
            .iter()
            .find(|m| m.selector == selector_of(number))
            .map(|m| m.kind.clone())
            .unwrap()
    };
    assert!(matches!(
        kind_of("253:0"),
        MappingKind::LvmLogicalVolume { .. }
    ));
    assert_eq!(kind_of("253:2"), MappingKind::CryptMapping);
    assert!(matches!(kind_of("253:9"), MappingKind::Undetermined { .. }));
    assert_eq!(
        mappings.mappings.len(),
        5,
        "every dm-marked node is classified and reported"
    );

    // Three volumes in two classes; the container is not a volume.
    assert_eq!(mappings.volumes.len(), 3);
    assert_eq!(mappings.groups.len(), 2, "two volume-group classes seen");
    let producer = derive_id(&lvm_group_fields()).expect("derivable");
    let lv_a = mappings
        .volumes
        .iter()
        .find(|v| v.selector == selector_of("253:0"))
        .unwrap();
    assert_eq!(
        lv_a.fields,
        NamingFields::Volume {
            producer,
            name: b"vg_dr_a-lv_a\n".to_vec(),
            role: None,
        },
        "the name is dm/name verbatim, newline included; the producer is the designator-absent LVM2 address"
    );
    assert!(
        !mappings
            .volumes
            .iter()
            .any(|v| v.selector == selector_of("253:2")),
        "a dm-crypt mapping yields no Volume — its name is the opener's (ADR-0053)"
    );
    // The loop is reported, not built.
    assert_eq!(mappings.loops.len(), 1);
    assert_eq!(
        mappings.loops[0].backing_path,
        SourceRead::Present(b"/var/tmp/dr-loop.img\n".to_vec())
    );

    // Absorption: one group of count two, three volumes under its address.
    let entries = absorb_mappings(&mappings).expect("absorption is total");
    let groups: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e, NodeEntry::Group { .. }))
        .collect();
    assert_eq!(groups.len(), 1);
    assert!(matches!(groups[0], NodeEntry::Group { count: 2, .. }));
    let volumes: Vec<_> = entries
        .iter()
        .filter(|e| {
            matches!(
                e,
                NodeEntry::Single {
                    fields: NamingFields::Volume { .. },
                    ..
                }
            )
        })
        .collect();
    assert_eq!(volumes.len(), 3, "three distinct names under one producer");

    // Under the closure, a volume produced by a designator-absent group is
    // indeterminate — never an operand.
    let group_fields = lvm_group_fields();
    let lv_fields = lv_a.fields.clone();
    let lv_id = derive_id(&lv_fields).expect("derivable");
    let topology = Topology::build(
        vec![group_fields, lv_fields],
        vec![Edge {
            kind: EdgeKind::Production,
            source: producer,
            target: lv_id,
        }],
    )
    .expect("builds");
    assert!(matches!(
        node_verdict(&topology, &Facts::default(), lv_id),
        Verdict::Indeterminate { .. }
    ));
}

// ---------------------------------------------------------------------------
// Increment 4b, third slice: the held standing, and the cached signature
// view reported and consulted by nothing.
// ---------------------------------------------------------------------------

/// The designated tree plus DR15's member shapes: a mapped PV held by its
/// LV mapping (`dm-0`), the same VG's unmapped PV unheld with a cached
/// `LVM2_member`, two md members held by `md127`, a LUKS disk held by its
/// container (`dm-2`), a plain disk, an unassembled member unheld with a
/// cached `linux_raid_member`, a device whose `holders/` refuses, and a
/// device held by a node with neither identity attribute. Every plain
/// device carries a `holders/` directory, as DR15 measured on every member
/// and control.
fn held_tree() -> FakeSource {
    let mut source = designated_tree();
    let mut entries = vec![
        "dm-0", "dm-1", "dm-2", "dm-3", "dm-9", "loop6", "md126", "md127", "md99", "sdb", "sdc",
        "sde", "sdf", "sdi", "sdm", "sdq", "sdr", "sdt", "shady", "weird",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    entries.sort();
    source
        .dirs
        .insert("/sys/class/block".to_owned(), Ok(entries));
    for (name, number, holders) in [
        ("sdb", "8:16", Ok(vec!["dm-0"])),
        ("sdc", "8:32", Ok(vec![])),
        ("sde", "8:64", Ok(vec!["md127"])),
        ("sdf", "8:80", Ok(vec!["md127"])),
        ("sdi", "8:128", Ok(vec!["dm-2"])),
        ("sdm", "8:192", Ok(vec![])),
        ("sdq", "8:256", Err(std::io::ErrorKind::PermissionDenied)),
        ("sdr", "8:272", Ok(vec![])),
        ("sdt", "8:304", Ok(vec!["weird"])),
    ] {
        source.files.insert(
            format!("/sys/class/block/{name}/dev"),
            Ok(format!("{number}\n").into_bytes()),
        );
        source.files.insert(
            format!("/sys/class/block/{name}/size"),
            Ok(b"2097152\n".to_vec()),
        );
        source.dirs.insert(
            format!("/sys/class/block/{name}/holders"),
            holders.map(|h: Vec<&str>| h.into_iter().map(str::to_owned).collect()),
        );
    }
    // The holder with neither identity attribute: a marker-less node whose
    // md/uuid and dm/uuid are both absent.
    source.files.insert(
        "/sys/class/block/weird/dev".to_owned(),
        Ok(b"259:0\n".to_vec()),
    );
    // DR6/DR14's cached signature view: the unmapped PV and the unassembled
    // member both carry a member type in the cache; the PV's record has no
    // version key (a positively determined absence).
    source.files.insert(
        "/run/udev/data/b8:32".to_owned(),
        Ok(b"E:ID_FS_TYPE=LVM2_member\nE:ID_FS_USAGE=raid\n".to_vec()),
    );
    source.files.insert(
        "/run/udev/data/b8:272".to_owned(),
        Ok(b"E:ID_FS_TYPE=linux_raid_member\nE:ID_FS_USAGE=raid\nE:ID_FS_VERSION=1.2\n".to_vec()),
    );
    source
}

// Requirements: LIN-006, INV-004, SAFE-005
//   The member-signature offset round decided that this adapter builds no
//   `BackingSignature`, no `Backing` edge and no `EncryptionLayer` — the
//   fields are the helper's byte layer's — and reports instead what the
//   client reads: a whole device's held standing from sysfs `holders/`.
//   DR15 measured the relation live from both ends and agreeing by
//   identity while entry names moved, so a holder is keyed by its own
//   `md/uuid` or `dm/uuid` and never by its entry; a member held by an
//   array agrees with that array's own `slaves/` report; the unmapped PV
//   of an active VG and a plain disk are unheld; a listing that did not
//   answer is undetermined, never unheld; a holder with no readable
//   identity leaves the device held and unkeyed rather than keyed by name.
//   Only plain devices are reported — the standing is a physical device's.
// Evidence: a_whole_device_is_held_by_its_holders_identity_and_an_unanswered_listing_is_undetermined
#[test]
#[allow(clippy::too_many_lines)]
fn a_whole_device_is_held_by_its_holders_identity_and_an_unanswered_listing_is_undetermined() {
    use crate::arrays::{Members, report_arrays};
    use crate::held::{HolderIdentity, Standing, report_held};
    use partman_domain::model::naming::NamingFields;
    use partman_domain::model::provenance::{Method, Outcome};

    let source = held_tree();
    let devices = devices_of(&source);
    let reports = report_held(&source, &PathBuf::from("/sys"), &devices);
    let plain = devices
        .iter()
        .filter(|d| d.kind == crate::devices::DeviceKind::Plain)
        .count();
    assert_eq!(reports.len(), plain, "every plain device and nothing else");
    assert_eq!(
        plain, 10,
        "the nine members and controls, plus the marker-less holder itself"
    );
    let by_number = |number: &str| {
        let selector = devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.selector.clone())
            .unwrap();
        reports.iter().find(|r| r.selector == selector).unwrap()
    };

    // The mapped PV: held by its mapping, keyed by the mapping's dm/uuid.
    let sdb = by_number("8:16");
    let Standing::Held { holders } = &sdb.standing else {
        panic!("the mapped PV is held")
    };
    assert_eq!(holders.len(), 1);
    assert_eq!(holders[0].entry, "dm-0");
    assert_eq!(
        holders[0].identity,
        HolderIdentity::DeviceMapper(
            "LVM-ek99dYwwU1KaulyX1bqr3RJC2pGrYoWOcbq0yOdyE6EodBHuixHfFrHBIqyXf8Zw".to_owned()
        )
    );
    assert_eq!(sdb.observations.len(), 1);
    assert_eq!(sdb.observations[0].method, Method::Direct);
    match &sdb.observations[0].outcome {
        Outcome::Observed { value } => {
            let text = format!("{value:?}");
            assert!(
                text.contains("LVM-ek99dYwwU1Kaul"),
                "the value is the identity"
            );
            assert!(!text.contains("dm-0"), "the value is never the entry name");
        }
        other => panic!("held is observed, got {other:?}"),
    }

    // The md members: held by the array, keyed by its md/uuid, and the
    // array's own slaves/ report names them back — both sides agree.
    let arrays = report_arrays(&source, &PathBuf::from("/sys"), &devices);
    for number in ["8:64", "8:80"] {
        let Standing::Held { holders } = &by_number(number).standing else {
            panic!("an md member is held")
        };
        let HolderIdentity::Mdraid(uuid) = &holders[0].identity else {
            panic!("held by an array")
        };
        assert_eq!(uuid, "54b95c15-7548-d8fb-52b0-5c2ff4f5d9f2");
        let array = arrays
            .iter()
            .find(|a| {
                matches!(&a.fields, NamingFields::Aggregate { designator: Some(d), .. }
                    if d == b"54b95c15-7548-d8fb-52b0-5c2ff4f5d9f2\n")
            })
            .expect("the array named from the same uuid");
        let entry = devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .map(|d| d.entry.clone())
            .unwrap();
        assert!(
            matches!(&array.members, Members::Listed(m) if m.contains(&entry)),
            "the array's slaves/ names the member its holders/ names — DR15's symmetry"
        );
    }

    // The LUKS disk: held by its container.
    assert!(matches!(
        &by_number("8:128").standing,
        Standing::Held { holders } if matches!(&holders[0].identity,
            HolderIdentity::DeviceMapper(u) if u.starts_with("CRYPT-LUKS2-"))
    ));

    // The unmapped PV, the plain disk, and the unassembled member: unheld,
    // each with one positively-absent observation.
    for number in ["8:32", "8:192", "8:272"] {
        let report = by_number(number);
        assert_eq!(report.standing, Standing::Unheld, "{number} is unheld");
        assert!(matches!(report.observations[..], [ref o] if o.outcome == Outcome::ObservedAbsent));
    }

    // A listing that did not answer: undetermined, never unheld.
    let sdq = by_number("8:256");
    assert!(matches!(sdq.standing, Standing::Undetermined { .. }));
    assert!(matches!(
        sdq.observations[0].outcome,
        Outcome::Failed { .. }
    ));

    // A holder with no readable identity: still held, and not keyed by name.
    let sdt = by_number("8:304");
    let Standing::Held { holders } = &sdt.standing else {
        panic!("a listed holder holds")
    };
    assert_eq!(holders[0].entry, "weird");
    assert!(matches!(
        holders[0].identity,
        HolderIdentity::Unidentified { .. }
    ));
    assert!(matches!(
        sdt.observations[0].outcome,
        Outcome::Failed { .. }
    ));

    // Nothing here names anything: no naming outcome changed.
    let named = named(&source, "sdb", "device:0");
    assert!(matches!(
        named,
        crate::naming::DeviceNaming::Addressed {
            fields: NamingFields::PhysicalDevice { .. },
            ..
        }
    ));
}

// Requirements: MODEL-004, SAFE-005
//   DR6 and DR14 measured the cache naming a member's technology and
//   family; L4/L10 measured it reporting exactly the stale signature on a
//   stale pair. So `ID_FS_TYPE`, `ID_FS_USAGE` and `ID_FS_VERSION` are
//   reported as `Heuristic`/`inferred` observations — an absent key a
//   positively determined absence, a missing record unavailable — and
//   consulted by nothing: an unassembled member with a cached
//   `linux_raid_member` and a PV with a cached `LVM2_member` stay unheld,
//   because demoting on the cache would be, at the draft, ADR-0018's
//   rejected "unconditionally refused orphan signatures". The structural
//   half scans the held module's code for any signature key.
// Evidence: the_cached_signature_view_is_reported_as_inferred_and_consulted_by_nothing
#[test]
fn the_cached_signature_view_is_reported_as_inferred_and_consulted_by_nothing() {
    use crate::held::{Standing, report_held};
    use partman_domain::canonical::Value;
    use partman_domain::model::provenance::{Method, Outcome};

    let source = held_tree();
    let devices = devices_of(&source);
    let device = |number: &str| {
        devices
            .iter()
            .find(|d| d.device_number.as_deref() == Some(number))
            .unwrap()
    };
    let sdr = device("8:272");
    for (key, value) in [
        ("linux-udev-db:ID_FS_TYPE", "linux_raid_member"),
        ("linux-udev-db:ID_FS_USAGE", "raid"),
        ("linux-udev-db:ID_FS_VERSION", "1.2"),
    ] {
        assert_eq!(
            outcome_of(sdr, key),
            &Outcome::Observed {
                value: Value::Text(value.to_owned())
            }
        );
        let observation = &sdr
            .properties
            .iter()
            .find(|(name, _)| name == key)
            .unwrap()
            .1
            .observations[0];
        assert_eq!(
            observation.method,
            Method::Heuristic,
            "{key} is inferred, not authoritative"
        );
    }
    // The PV's record has no version key: absent, never unavailable.
    assert_eq!(
        outcome_of(device("8:32"), "linux-udev-db:ID_FS_VERSION"),
        &Outcome::ObservedAbsent
    );
    // A device with no record: unavailable, never absent.
    assert!(matches!(
        outcome_of(device("8:192"), "linux-udev-db:ID_FS_TYPE"),
        Outcome::Unavailable { .. }
    ));

    // Consulted by nothing: both cached members stay unheld.
    let reports = report_held(&source, &PathBuf::from("/sys"), &devices);
    for number in ["8:32", "8:272"] {
        let selector = device(number).selector.clone();
        let report = reports.iter().find(|r| r.selector == selector).unwrap();
        assert_eq!(
            report.standing,
            Standing::Unheld,
            "{number}: the cache decides nothing"
        );
    }
    // Structurally: the held module's code names no signature key.
    for (n, line) in include_str!("held.rs").lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        assert!(
            !code.contains("ID_FS") && !code.contains("SIGNATURE_KEYS"),
            "held.rs:{}: the cached signature view must not be consulted for standing",
            n + 1
        );
    }
}

// ---------------------------------------------------------------------------
// Increment 5a: the capability seam.
// ---------------------------------------------------------------------------

// Requirements: INV-006, SAFE-005
//   No read-only operation needs a tool (the plan's finding F1: every
//   served operation is a source-class file read, ACC-009 gates write
//   steps, and a floor arrives with the first package that invokes the
//   tool), so the roster is empty for every source-class operation and
//   an entry against one is refused; a mutating operation is not served
//   by a read-only adapter and answers a typed refusal, never an empty
//   roster that would read as "no tool needed"; and INV-006's "never run
//   repair tools during discovery" is held in two forms — no requirement
//   names a mount, unlock or repair tool, and the no-process guard over
//   every shipped module still stands over this one.
// Evidence: no_served_operation_requires_a_tool_and_a_mutating_one_is_not_served
#[test]
fn no_served_operation_requires_a_tool_and_a_mutating_one_is_not_served() {
    use crate::runtime::{
        FORBIDDEN_DURING_DISCOVERY, NotServed, REQUIREMENTS, required_tools, runtime_facts,
    };
    use partman_capability::engine::{PlatformFact, RuntimeFacts};
    use partman_domain::model::capability::{Operation, OperationClass};

    let mut served = 0;
    for operation in Operation::all() {
        match operation.class() {
            OperationClass::Source => {
                let tools = required_tools(*operation).expect("a source-class operation is served");
                assert!(
                    tools.is_empty(),
                    "{operation:?}: no read-only operation needs a tool"
                );
                let facts =
                    runtime_facts(*operation, &[], &[], PlatformFact::MeetsFloor).expect("served");
                assert_eq!(
                    facts,
                    RuntimeFacts {
                        tools: Vec::new(),
                        platform: PlatformFact::MeetsFloor
                    }
                );
                served += 1;
            }
            OperationClass::Mutating => {
                assert_eq!(
                    required_tools(*operation),
                    Err(NotServed::Mutating {
                        operation: *operation
                    }),
                    "{operation:?}: a mutating operation's tools are WP-L110's to state"
                );
                assert!(runtime_facts(*operation, &[], &[], PlatformFact::MeetsFloor).is_err());
            }
        }
    }
    assert_eq!(served, 4, "every source-class operation is served");
    // The table lists exactly the source-class operations, each empty.
    assert_eq!(REQUIREMENTS.len(), 4);
    for (operation, tools) in REQUIREMENTS {
        assert_eq!(operation.class(), OperationClass::Source);
        for tool in *tools {
            assert!(
                !FORBIDDEN_DURING_DISCOVERY.contains(&tool.tool),
                "{}: INV-006 forbids it during discovery",
                tool.tool
            );
        }
    }
    // The seam launches nothing itself: the crate-wide guard's needles,
    // applied to this module by name so a reader can see it is covered.
    for needle in ["std::process", "Command::new", "std::env", "/dev/"] {
        assert!(!include_str!("runtime.rs").contains(needle));
    }
}

// Requirements: CAP-004, SAFE-005
//   ACC-009's two failure classes as the engine spells them, applied to
//   a caller's structured probe fail-closed on every arm the text leaves
//   open: present at or above a known floor is in range; absent, or not
//   probed at all, is missing; present below the floor, present with no
//   floor known (no tested range exists), present with an unparsed
//   version, or a failed probe is out of range. The assembly finds each
//   requirement's probe and floor by name and carries the caller's
//   platform fact unchanged.
// Evidence: tool_state_follows_acc_009_and_fails_closed_where_the_text_is_open
#[test]
fn tool_state_follows_acc_009_and_fails_closed_where_the_text_is_open() {
    use crate::runtime::{ToolFloor, ToolProbe, Version, tool_state};
    use partman_capability::engine::ToolState;

    let v = |major, minor, patch| Version {
        major,
        minor,
        patch,
    };
    let floor = ToolFloor {
        tool: "wipefs",
        floor: v(2, 37, 0),
    };
    let present = |version: Option<Version>| ToolProbe::Present {
        path: "/usr/sbin/wipefs".to_owned(),
        version,
    };
    assert_eq!(
        tool_state(Some(&present(Some(v(2, 37, 2)))), Some(&floor)),
        ToolState::PresentInRange
    );
    assert_eq!(
        tool_state(Some(&present(Some(v(2, 37, 0)))), Some(&floor)),
        ToolState::PresentInRange,
        "the floor itself is in range"
    );
    assert_eq!(
        tool_state(Some(&present(Some(v(2, 36, 9)))), Some(&floor)),
        ToolState::OutOfRange,
        "below the floor"
    );
    assert_eq!(
        tool_state(Some(&present(Some(v(2, 41, 0)))), None),
        ToolState::OutOfRange,
        "no floor known: no tested range exists, so no version is inside it"
    );
    assert_eq!(
        tool_state(Some(&present(None)), Some(&floor)),
        ToolState::OutOfRange,
        "an unparsed version is not inside any range"
    );
    assert_eq!(
        tool_state(
            Some(&ToolProbe::Failed {
                reason: "timed out".to_owned()
            }),
            Some(&floor)
        ),
        ToolState::OutOfRange
    );
    assert_eq!(
        tool_state(
            Some(&ToolProbe::Absent {
                checked: vec!["/usr/sbin/wipefs".to_owned()]
            }),
            Some(&floor)
        ),
        ToolState::Missing
    );
    assert_eq!(
        tool_state(None, Some(&floor)),
        ToolState::Missing,
        "not probed is not established present"
    );
    assert!(
        v(3, 0, 0) > v(2, 41, 7) && v(2, 41, 0) > v(2, 37, 9),
        "ordered by (major, minor, patch)"
    );
}

// ---------------------------------------------------------------------------
// Increment 5b: the Section 9 floor determination.
// ---------------------------------------------------------------------------

/// DR16/DR17's jammy shapes, DR18's Arch shapes and DR19's Debian 12
/// shapes, byte for byte as the transcripts carry them (the os-release
/// bodies are the measured files; the kernel strings are the measured
/// `osrelease` contents).
const JAMMY_OS_RELEASE: &str = "PRETTY_NAME=\"Ubuntu 22.04.5 LTS\"\nNAME=\"Ubuntu\"\nVERSION_ID=\"22.04\"\nVERSION=\"22.04.5 LTS (Jammy Jellyfish)\"\nVERSION_CODENAME=jammy\nID=ubuntu\nID_LIKE=debian\nHOME_URL=\"https://www.ubuntu.com/\"\nSUPPORT_URL=\"https://help.ubuntu.com/\"\nBUG_REPORT_URL=\"https://bugs.launchpad.net/ubuntu/\"\nPRIVACY_POLICY_URL=\"https://www.ubuntu.com/legal/terms-and-policies/privacy-policy\"\nUBUNTU_CODENAME=jammy\n";
const DEBIAN_OS_RELEASE: &str = "PRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"\nNAME=\"Debian GNU/Linux\"\nVERSION_ID=\"12\"\nVERSION=\"12 (bookworm)\"\nVERSION_CODENAME=bookworm\nID=debian\nHOME_URL=\"https://www.debian.org/\"\nSUPPORT_URL=\"https://www.debian.org/support\"\nBUG_REPORT_URL=\"https://bugs.debian.org/\"\n";
const ARCH_OS_RELEASE: &str = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\nANSI_COLOR=\"38;2;23;147;209\"\nHOME_URL=\"https://archlinux.org/\"\nDOCUMENTATION_URL=\"https://wiki.archlinux.org/\"\nSUPPORT_URL=\"https://bbs.archlinux.org/\"\nBUG_REPORT_URL=\"https://gitlab.archlinux.org/groups/archlinux/-/issues\"\nPRIVACY_POLICY_URL=\"https://terms.archlinux.org/docs/privacy-policy/\"\nLOGO=archlinux-logo\n";

fn floor_source(os_release: Option<&[u8]>, kernel: Option<&[u8]>) -> FakeSource {
    let mut source = FakeSource::empty();
    if let Some(bytes) = os_release {
        source
            .files
            .insert("/etc/os-release".to_owned(), Ok(bytes.to_vec()));
    }
    if let Some(bytes) = kernel {
        source
            .files
            .insert("/proc/sys/kernel/osrelease".to_owned(), Ok(bytes.to_vec()));
    }
    source
}

fn floor_of(source: &FakeSource) -> crate::floor::FloorReport {
    crate::floor::platform_floor(source, &PathBuf::from("/etc"), &PathBuf::from("/proc"))
}

// Requirements: CAP-004, INV-002, SAFE-005
//   Section 9's Debian/Ubuntu floor determined from the two files DR16 and
//   DR17 measured on the jammy guest, byte for byte: `ID=ubuntu` and a
//   double-quoted `VERSION_ID="22.04"` (the quotes stripped and nothing
//   else) meet the release row, `5.15.0-186-generic` parses to `5.15` and
//   meets the kernel row exactly, and the UDisks2 conjunct is undetermined
//   by construction — so the composed fact is `Undetermined`, naming
//   UDisks2, which is the honest answer for every measured acceptance
//   environment (they run without the daemon) and never `MeetsFloor` (a
//   widening Section 9 forbids) or `BelowFloor` (unmeasured). Each key
//   read is an observation on the fourth interface, direct; the kernel
//   on procfs.
// Evidence: the_ubuntu_floor_is_undetermined_on_udisks2_alone_with_release_and_kernel_met
#[test]
fn the_ubuntu_floor_is_undetermined_on_udisks2_alone_with_release_and_kernel_met() {
    use crate::floor::{Conjunct, Tier};
    use partman_capability::engine::PlatformFact;
    use partman_domain::model::provenance::{Method, Outcome};

    let source = floor_source(
        Some(JAMMY_OS_RELEASE.as_bytes()),
        Some(b"5.15.0-186-generic\n"),
    );
    let report = floor_of(&source);
    assert_eq!(report.tier, Tier::Ubuntu);
    assert_eq!(report.distribution, Conjunct::Met, "22.04 is the row");
    assert_eq!(report.kernel, Conjunct::Met, "5.15 is the row, exactly");
    assert!(matches!(report.udisks2, Conjunct::Undetermined { .. }));
    match &report.platform {
        PlatformFact::Undetermined { conjunct } => {
            assert!(
                conjunct.contains("UDisks2"),
                "names the conjunct: {conjunct}"
            );
        }
        other => panic!("undetermined on UDisks2, got {other:?}"),
    }
    // Observations: ID and VERSION_ID on the fourth interface, the kernel on procfs, all direct.
    let os_release: Vec<_> = report
        .observations
        .iter()
        .filter(|o| o.adapter == "partman-adapter-linux/linux-os-release")
        .collect();
    assert_eq!(os_release.len(), 2);
    assert!(os_release.iter().all(|o| o.method == Method::Direct));
    assert!(os_release.iter().any(|o| matches!(&o.outcome,
        Outcome::Observed { value: partman_domain::canonical::Value::Text(t) } if t == "VERSION_ID=22.04")),
        "the quotes are stripped, nothing else");
    assert!(report.observations.iter().any(|o| o.adapter == "partman-adapter-linux/linux-procfs"
        && matches!(&o.outcome, Outcome::Observed { value: partman_domain::canonical::Value::Text(t) } if t == "5.15.0-186-generic")));
}

// Requirements: CAP-004, SAFE-005
//   DR18's Arch shape: `ID=arch` with no `VERSION_ID`, on the row that
//   names no version, no kernel and no UDisks2 conjunct — the only tier
//   that reaches `MeetsFloor`, on `ID` alone, with the absent key a
//   positively determined absence. And the shapes nobody measured are
//   undetermined, never assumed: Debian (recognized, its release shape
//   unmeasured), an unlisted `ID`, a missing `ID`, an absent file.
// Evidence: arch_meets_its_row_on_id_alone_and_unmeasured_shapes_are_undetermined
#[test]
fn arch_meets_its_row_on_id_alone_and_unmeasured_shapes_are_undetermined() {
    use crate::floor::{Conjunct, Tier};
    use partman_capability::engine::PlatformFact;
    use partman_domain::model::provenance::Outcome;

    let arch = floor_of(&floor_source(
        Some(ARCH_OS_RELEASE.as_bytes()),
        Some(b"7.1.8-arch1-3\n"),
    ));
    assert_eq!(arch.tier, Tier::Arch);
    assert_eq!(arch.distribution, Conjunct::Met);
    assert_eq!(arch.kernel, Conjunct::NotInRow);
    assert_eq!(arch.udisks2, Conjunct::NotInRow);
    assert_eq!(arch.platform, PlatformFact::MeetsFloor);
    assert!(
        arch.observations
            .iter()
            .any(|o| o.adapter == "partman-adapter-linux/linux-os-release"
                && o.outcome == Outcome::ObservedAbsent),
        "VERSION_ID absent on Arch is a positively determined absence"
    );

    let other = floor_of(&floor_source(
        Some(b"ID=fedora\nVERSION_ID=40\n"),
        Some(b"6.8.0\n"),
    ));
    assert_eq!(
        other.tier,
        Tier::Unrecognized {
            id: "fedora".to_owned()
        }
    );
    assert!(matches!(other.platform, PlatformFact::Undetermined { .. }));

    let no_id = floor_of(&floor_source(
        Some(b"NAME=\"Something\"\n"),
        Some(b"6.8.0\n"),
    ));
    assert!(matches!(no_id.tier, Tier::Unknown { .. }));
    assert!(matches!(no_id.platform, PlatformFact::Undetermined { .. }));

    let absent = floor_of(&floor_source(None, Some(b"6.8.0\n")));
    assert!(matches!(absent.tier, Tier::Unknown { .. }));
    assert!(matches!(absent.platform, PlatformFact::Undetermined { .. }));
    assert!(
        absent
            .observations
            .iter()
            .any(|o| matches!(o.outcome, Outcome::Unavailable { .. }))
    );
}

// Requirements: CAP-004, INV-002, SAFE-005
//   The Debian arm on DR19, byte for byte: the first Debian guest's
//   `os-release` (267 bytes; `ID=debian` unquoted, `VERSION_ID="12"`
//   double-quoted with ONE numeric part and no minor, no `ID_LIKE`) and
//   its `osrelease` (`6.1.0-52-cloud-amd64`). The release conjunct parses
//   the leading integer and compares it against 12 — it must not demand
//   Ubuntu's `major.minor` shape, which would read the measured file as
//   unparsable — and the quotes are stripped exactly as DR16's; the
//   kernel meets 5.15; the UDisks2 conjunct is undetermined by
//   construction, so the composed fact is `Undetermined` naming UDisks2
//   (the image ships without the daemon, DR19), never `MeetsFloor` and
//   never `BelowFloor`. `11` is a measured shortfall (`BelowFloor` even
//   beside the undetermined conjunct), `13` is above the floor, a missing
//   or unparsable `VERSION_ID` is undetermined, never assumed. Each key
//   read is an observation on the fourth interface, direct.
// Evidence: the_debian_arm_compares_one_numeric_part_on_dr19_and_is_undetermined_on_udisks2_alone
#[test]
fn the_debian_arm_compares_one_numeric_part_on_dr19_and_is_undetermined_on_udisks2_alone() {
    use crate::floor::{Conjunct, Tier, parse_major, parse_major_minor};
    use partman_capability::engine::PlatformFact;
    use partman_domain::canonical::Value;
    use partman_domain::model::provenance::Outcome;

    assert_eq!(DEBIAN_OS_RELEASE.len(), 267, "DR19 measured 267 bytes");
    let debian = floor_of(&floor_source(
        Some(DEBIAN_OS_RELEASE.as_bytes()),
        Some(b"6.1.0-52-cloud-amd64\n"),
    ));
    assert_eq!(debian.tier, Tier::Debian);
    assert_eq!(
        debian.distribution,
        Conjunct::Met,
        "VERSION_ID=\"12\" meets the row's 12 on one numeric part"
    );
    assert_eq!(debian.kernel, Conjunct::Met, "6.1 meets 5.15");
    assert!(matches!(debian.udisks2, Conjunct::Undetermined { .. }));
    match &debian.platform {
        PlatformFact::Undetermined { conjunct } => {
            assert!(
                conjunct.contains("UDisks2"),
                "names the conjunct: {conjunct}"
            );
        }
        other => {
            panic!("Debian with release and kernel met is undetermined on UDisks2, got {other:?}")
        }
    }
    assert!(
        debian
            .observations
            .iter()
            .any(|o| o.adapter == "partman-adapter-linux/linux-os-release"
                && matches!(&o.outcome, Outcome::Observed { value: Value::Text(text) } if text == "VERSION_ID=12")),
        "VERSION_ID observed as 12 on the fourth interface, quotes stripped"
    );
    // The measured shape is exactly the one `major.minor` refuses: the arm
    // must parse one part.
    assert_eq!(parse_major_minor("12"), None);
    assert_eq!(parse_major("12"), Some(12));
    assert_eq!(parse_major("12.1"), Some(12));
    assert_eq!(parse_major("6.1.0-52-cloud-amd64"), Some(6));
    assert_eq!(parse_major("bookworm"), None);
    assert_eq!(parse_major(""), None);

    let old = floor_of(&floor_source(
        Some(b"ID=debian\nVERSION_ID=\"11\"\n"),
        Some(b"5.10.0-28-amd64\n"),
    ));
    assert!(matches!(old.distribution, Conjunct::Unmet { .. }));
    assert_eq!(
        old.platform,
        PlatformFact::BelowFloor,
        "a measured shortfall wins over the undetermined conjunct"
    );
    let newer = floor_of(&floor_source(
        Some(b"ID=debian\nVERSION_ID=\"13\"\n"),
        Some(b"6.12.0-1-amd64\n"),
    ));
    assert_eq!(
        newer.distribution,
        Conjunct::Met,
        "a later major is above the floor"
    );
    let no_version = floor_of(&floor_source(
        Some(b"ID=debian\nVERSION_CODENAME=trixie\n"),
        Some(b"6.12.0-1-amd64\n"),
    ));
    assert!(
        matches!(no_version.distribution, Conjunct::Undetermined { .. }),
        "no VERSION_ID: undetermined, never assumed"
    );
    assert!(matches!(
        no_version.platform,
        PlatformFact::Undetermined { .. }
    ));
    let unparsable = floor_of(&floor_source(
        Some(b"ID=debian\nVERSION_ID=\"bookworm\"\n"),
        Some(b"6.1.0-52-cloud-amd64\n"),
    ));
    assert!(matches!(
        unparsable.distribution,
        Conjunct::Undetermined { .. }
    ));
    assert!(matches!(
        unparsable.platform,
        PlatformFact::Undetermined { .. }
    ));
}

// Requirements: CAP-004, SAFE-005
//   The composition is fail-closed on every arm: a measured shortfall in
//   release or kernel is `BelowFloor` even beside undetermined conjuncts;
//   an unparsable release or kernel string is undetermined, never
//   compared; a release above the row is met (that much arithmetic the
//   word "floor" states); an absent kernel file is undetermined; and
//   `MeetsFloor` needs every conjunct met or not in the row.
// Evidence: the_floor_composes_fail_closed_and_a_measured_shortfall_is_below
#[test]
fn the_floor_composes_fail_closed_and_a_measured_shortfall_is_below() {
    use crate::floor::{Conjunct, compose, parse_major_minor};
    use partman_capability::engine::PlatformFact;

    let old_release = floor_of(&floor_source(
        Some(b"ID=ubuntu\nVERSION_ID=\"20.04\"\n"),
        Some(b"5.15.0-186-generic\n"),
    ));
    assert!(matches!(old_release.distribution, Conjunct::Unmet { .. }));
    assert_eq!(
        old_release.platform,
        PlatformFact::BelowFloor,
        "a measured shortfall wins over undetermined"
    );
    let old_kernel = floor_of(&floor_source(
        Some(JAMMY_OS_RELEASE.as_bytes()),
        Some(b"5.4.0-150-generic\n"),
    ));
    assert!(matches!(old_kernel.kernel, Conjunct::Unmet { .. }));
    assert_eq!(old_kernel.platform, PlatformFact::BelowFloor);
    let newer = floor_of(&floor_source(
        Some(b"ID=ubuntu\nVERSION_ID=\"24.04\"\n"),
        Some(b"6.8.0-45-generic\n"),
    ));
    assert_eq!(newer.distribution, Conjunct::Met);
    assert_eq!(newer.kernel, Conjunct::Met);
    let unparsable = floor_of(&floor_source(
        Some(b"ID=ubuntu\nVERSION_ID=\"jammy\"\n"),
        Some(b"custom\n"),
    ));
    assert!(matches!(
        unparsable.distribution,
        Conjunct::Undetermined { .. }
    ));
    assert!(matches!(unparsable.kernel, Conjunct::Undetermined { .. }));
    assert!(matches!(
        unparsable.platform,
        PlatformFact::Undetermined { .. }
    ));
    let no_kernel = floor_of(&floor_source(Some(JAMMY_OS_RELEASE.as_bytes()), None));
    assert!(matches!(no_kernel.kernel, Conjunct::Undetermined { .. }));

    assert_eq!(parse_major_minor("22.04"), Some((22, 4)));
    assert_eq!(parse_major_minor("5.15.0-186-generic"), Some((5, 15)));
    assert_eq!(parse_major_minor("7.1.8-arch1-3"), Some((7, 1)));
    assert_eq!(parse_major_minor("jammy"), None);
    assert_eq!(parse_major_minor("5"), None);
    assert_eq!(
        compose(&Conjunct::Met, &Conjunct::Met, &Conjunct::Met),
        PlatformFact::MeetsFloor
    );
    assert_eq!(
        compose(&Conjunct::Met, &Conjunct::NotInRow, &Conjunct::NotInRow),
        PlatformFact::MeetsFloor
    );
    assert!(matches!(
        compose(
            &Conjunct::Met,
            &Conjunct::Met,
            &Conjunct::Undetermined {
                reason: "u".to_owned()
            }
        ),
        PlatformFact::Undetermined { .. }
    ));
    assert_eq!(
        compose(
            &Conjunct::Unmet {
                measured: "x".to_owned()
            },
            &Conjunct::Undetermined {
                reason: "u".to_owned()
            },
            &Conjunct::Met
        ),
        PlatformFact::BelowFloor
    );
}
