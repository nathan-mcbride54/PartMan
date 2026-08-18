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
fn shipped_sources() -> [(&'static str, &'static str); 7] {
    [
        ("lib.rs", include_str!("lib.rs")),
        ("contract.rs", include_str!("contract.rs")),
        ("derivation.rs", include_str!("derivation.rs")),
        ("devices.rs", include_str!("devices.rs")),
        ("naming.rs", include_str!("naming.rs")),
        ("observation.rs", include_str!("observation.rs")),
        ("reach.rs", include_str!("reach.rs")),
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
    for wanted in crate::devices::UDEV_KEYS {
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
