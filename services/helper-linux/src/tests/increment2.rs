//! Increment 2's Tier-1 suite: the byte layer's windowing, the capture's
//! authorship over authored trees and catalogue bytes, the validate-plan
//! arms, SEC-002's admission function, the strict v2 decode, the wire
//! round, and the helper's reach declaration — all pure, all platforms
//! (the Evidence-sourcing rule: structural properties over authored
//! inputs; every host claim rests on the DR21 row and the increment's
//! Tier-2 acceptance, not on anything asserted here).

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

use partman_adapter_linux::contract::ContractSource;
use partman_capability::engine::{RuntimeFacts, TechnologyLimits};
use partman_domain::canonical::{self, Value};
use partman_domain::model::capability::Operation as CapOp;
use partman_domain::model::identity::{DeviceIdentity, IndeterminateCause, TableState};
use partman_domain::model::naming::{AggregateTechnology, NamingFields, NodeEntry, derive_id};
use partman_domain::model::protection::{Facts, HostRange, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::Severity;
use partman_planner::PlanRefusal;
use partman_table_parser::{Geometry, classify};

use crate::bytes::{ByteRefusal, DeviceReader, Windows, read_windows};
use crate::capture::{CaptureOutcome, DeviceCapture, capture};
use crate::validate::{
    AdmissionRefusal, ValidateRefusal, ValidateRequest, ValidationRecord, admit_presented_plan,
    parse_operation, validate_plan,
};
use crate::{Operation, Request, RequestRefusal, SCHEMA_VERSION, TargetSpelling, ValidateWire};

// ---------------------------------------------------------------- fakes

/// The Tier-1 contract fake, the adapter suite's shape: an absent path is
/// `NotFound`, which is the positively-absent case.
#[derive(Default)]
struct FakeSource {
    dirs: BTreeMap<String, Vec<String>>,
    files: BTreeMap<String, Vec<u8>>,
}

fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl ContractSource for FakeSource {
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        self.dirs
            .get(&key(path))
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such directory"))
    }
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        self.files
            .get(&key(path))
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such attribute"))
    }
}

/// The Tier-1 device reader: catalogue bytes by entry name, windowed by
/// the same pure function the Linux reader uses.
#[derive(Default)]
struct ImageReader {
    images: BTreeMap<String, Vec<u8>>,
}

impl DeviceReader for ImageReader {
    fn windows(
        &self,
        entry: &str,
        _device_number: &str,
        geometry: Geometry,
    ) -> Result<Windows, ByteRefusal> {
        let bytes = self.images.get(entry).ok_or(ByteRefusal::Open {
            kind: "NotFound".to_owned(),
        })?;
        read_windows(&mut Cursor::new(bytes), geometry)
    }
}

fn image(name: &str) -> Vec<u8> {
    let fixture = partman_fixtures::catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name.trim_end_matches(".img") == name)
        .unwrap_or_else(|| panic!("{name} is not in the catalogue"));
    (fixture.build)().into_bytes()
}

/// One fake host: devices with distinct USB-designated serials carrying
/// catalogue images, plus whatever a test adds.
struct Host {
    source: FakeSource,
    reader: ImageReader,
}

impl Host {
    fn new() -> Self {
        let mut source = FakeSource::default();
        source.dirs.insert("/sys/class/block".to_owned(), vec![]);
        Self {
            source,
            reader: ImageReader::default(),
        }
    }

    /// Add one whole device: `serial` designates it (via the fake's USB
    /// ancestor); `bytes` is its medium.
    fn device(&mut self, entry: &str, serial: Option<&str>, bytes: Vec<u8>) {
        let dir = format!("/sys/class/block/{entry}");
        self.source
            .dirs
            .get_mut("/sys/class/block")
            .unwrap()
            .push(entry.to_owned());
        let sectors = bytes.len() as u64 / 512;
        self.source
            .files
            .insert(format!("{dir}/size"), format!("{sectors}\n").into_bytes());
        self.source.files.insert(
            format!("{dir}/dev"),
            format!("8:{entry_len}", entry_len = 16 * self.reader.images.len()).into_bytes(),
        );
        self.source
            .files
            .insert(format!("{dir}/queue/logical_block_size"), b"512\n".to_vec());
        self.source.files.insert(
            format!("{dir}/queue/physical_block_size"),
            b"512\n".to_vec(),
        );
        if let Some(serial) = serial {
            for (marker, value) in [
                ("idVendor", "0781"),
                ("idProduct", "5583"),
                ("serial", serial),
            ] {
                self.source.files.insert(
                    format!("{dir}/device/../{marker}"),
                    value.as_bytes().to_vec(),
                );
            }
        }
        self.reader.images.insert(entry.to_owned(), bytes);
    }

    fn capture(&self) -> CaptureOutcome {
        capture(
            &self.source,
            Path::new("/sys"),
            Path::new("/run/udev/data"),
            &self.reader,
            1_700_000_000,
            false,
        )
        .expect("captures")
    }
}

// ------------------------------------------------- authored snapshots

fn clean_device(serial: &[u8]) -> (TopologySnapshot, partman_domain::model::naming::NodeId) {
    let device = NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    };
    let id = derive_id(&device).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(id, TransportClass::Sata);
    facts.extents.insert(
        id,
        HostRange {
            host: id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts
        .table_states
        .insert(id, TableState::present([0x42; 32]));
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![device], vec![], facts)
            .expect("assembles");
    (snapshot, id)
}

/// Increment 4a's CONC-004 probe: one clean device captured through the
/// real `capture()` with the caller's transitional flag — the same
/// fixture under both flag values, so the flag's journey from the
/// parameter through `assemble` into the body hash is what the test
/// exercises (a hard-coded value inside `capture` cannot survive it).
pub(super) fn capture_with_flag(transitional: bool) -> CaptureOutcome {
    let mut host = Host::new();
    host.device("sda", Some("C4-PROBE"), image("gpt-basic-512"));
    capture(
        &host.source,
        Path::new("/sys"),
        Path::new("/run/udev/data"),
        &host.reader,
        1_700_000_000,
        transitional,
    )
    .expect("captures")
}

fn wipe_request(target: partman_domain::model::naming::NodeId) -> ValidateRequest {
    ValidateRequest {
        target,
        operation: CapOp::Wipe,
        plan_id: b"inc2-test".to_vec(),
        validity_seconds: 3600,
    }
}

const NOW: u64 = 1_700_000_000;

// Requirements: HLP-002, INV-003, SAFE-005
//   The capture authors exactly what the parser says, byte for byte: the
//   authored table state on each catalogue medium equals an independent
//   classification of the same bytes (Present carrying the identical
//   copy-invariant checksum), the claimed scheme's table node exists and
//   a hybrid medium carries its second view node, a positive absence
//   authors no node, and the body hash is stable across two captures of
//   unchanged hardware — PLAN-006's comparison stays satisfiable because
//   the timestamp lives in the envelope.
// Evidence: the_capture_authors_what_the_parser_says_and_only_that
#[test]
#[allow(clippy::too_many_lines)]
fn the_capture_authors_what_the_parser_says_and_only_that() {
    let mut host = Host::new();
    host.device("sda", Some("SER-A"), image("gpt-basic-512"));
    host.device("sdb", Some("SER-B"), image("blank-512"));
    host.device("sdc", Some("SER-C"), image("gpt-conflicting-tables-512"));
    host.device("sdd", Some("SER-D"), image("hybrid-mbr-gpt-512"));
    let outcome = host.capture();

    let expected: Vec<(&str, &str, Option<&str>, bool)> = vec![
        ("device:0", "present", Some("gpt"), false),
        ("device:1", "absent", None, false),
        ("device:2", "indeterminate-ambiguous", Some("gpt"), false),
        ("device:3", "present", Some("gpt"), true),
    ];
    for (index, (selector, state, scheme, hybrid)) in expected.iter().enumerate() {
        match &outcome.devices[index] {
            DeviceCapture::Authored {
                selector: s,
                state: st,
                scheme: sc,
                hybrid: h,
                ..
            } => {
                assert_eq!(
                    (s.as_str(), *st, *sc, *h),
                    (*selector, *state, *scheme, *hybrid)
                );
            }
            other => panic!("{selector}: {other:?}"),
        }
    }

    // Byte-for-byte agreement with an independent classification.
    for (entry, name) in [
        ("sda", "gpt-basic-512"),
        ("sdb", "blank-512"),
        ("sdc", "gpt-conflicting-tables-512"),
        ("sdd", "hybrid-mbr-gpt-512"),
    ] {
        let bytes = image(name);
        let geometry = Geometry {
            sector_size: 512,
            total_sectors: bytes.len() as u64 / 512,
        };
        let head = &bytes[..65_536];
        let tail = &bytes[bytes.len() - 65_536..];
        let independent = classify(head, tail, geometry).expect("classifies");
        let device = NamingFields::PhysicalDevice {
            serial: Some(host.serial_of(entry)),
            wwn: None,
            total_bytes: bytes.len() as u64,
        };
        let id = derive_id(&device).expect("derivable");
        let authored = outcome.snapshot.facts().table_states.get(&id);
        let expected = match independent.state {
            partman_table_parser::TableState::Present { checksum } => TableState::present(checksum),
            partman_table_parser::TableState::Absent => TableState::Absent,
            partman_table_parser::TableState::Indeterminate { basis } => {
                TableState::Indeterminate {
                    cause: match basis {
                        partman_table_parser::IndeterminateBasis::Ambiguous => {
                            IndeterminateCause::Ambiguous
                        }
                        partman_table_parser::IndeterminateBasis::Unreadable => {
                            IndeterminateCause::Unreadable
                        }
                    },
                }
            }
        };
        assert_eq!(authored, Some(&expected), "{entry}");
    }

    // The blank device carries no table node; the hybrid carries two.
    let entries = outcome.snapshot.topology().entries();
    let tables = |parent| {
        entries
            .iter()
            .filter(move |entry| match entry {
                NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => {
                    matches!(fields, NamingFields::PartitionTable { parent: p, .. } if *p == parent)
                }
            })
            .count()
    };
    let id_of = |entry: &str| {
        let bytes = host.reader.images.get(entry).unwrap();
        derive_id(&NamingFields::PhysicalDevice {
            serial: Some(host.serial_of(entry)),
            wwn: None,
            total_bytes: bytes.len() as u64,
        })
        .unwrap()
    };
    assert_eq!(tables(id_of("sda")), 1);
    assert_eq!(
        tables(id_of("sdb")),
        0,
        "a positive absence authors no node"
    );
    assert_eq!(tables(id_of("sdc")), 1);
    assert_eq!(tables(id_of("sdd")), 2, "the hybrid's second view node");

    // Two captures of unchanged hardware hash equal (PLAN-006).
    let again = capture(
        &host.source,
        Path::new("/sys"),
        Path::new("/run/udev/data"),
        &host.reader,
        1_700_000_777,
        false,
    )
    .expect("captures");
    assert_eq!(
        outcome.snapshot_hash, again.snapshot_hash,
        "the timestamp is envelope content, so unchanged hardware re-probes equal"
    );
    assert_ne!(
        outcome.snapshot.envelope.capture_timestamp,
        again.snapshot.envelope.capture_timestamp
    );
}

impl Host {
    fn serial_of(&self, entry: &str) -> Vec<u8> {
        self.source
            .files
            .get(&format!("/sys/class/block/{entry}/device/../serial"))
            .cloned()
            .expect("the test gave this device a serial")
    }
}

// Requirements: HLP-002, SAFE-003, FS-008
//   What the capture refuses to invent: two devices whose designated
//   sources derive one address absorb into the counted, flagged group
//   and no fact is authored under the shared address; a held device
//   stays a captured physical device with its standing recorded in the
//   envelope and no aggregate node or member edge emitted; a
//   host-assembled device is withdrawn exactly as the adapter withdraws
//   it.
// Evidence: collisions_holds_and_assemblies_author_nothing_they_cannot_stand_behind
#[test]
fn collisions_holds_and_assemblies_author_nothing_they_cannot_stand_behind() {
    let mut host = Host::new();
    host.device("sda", None, image("blank-512"));
    host.device("sdb", None, image("gpt-basic-512"));
    host.device("sdc", Some("SER-HELD"), image("blank-512"));
    // sdc is held by an md node.
    host.source.dirs.insert(
        "/sys/class/block/sdc/holders".to_owned(),
        vec!["md9".to_owned()],
    );
    host.source.files.insert(
        "/sys/class/block/md9/md/uuid".to_owned(),
        b"11111111:22222222:33333333:44444444".to_vec(),
    );
    // A dm device in the listing: withdrawn, not named.
    host.source
        .dirs
        .get_mut("/sys/class/block")
        .unwrap()
        .push("dm-0".to_owned());
    host.source
        .dirs
        .insert("/sys/class/block/dm-0/dm".to_owned(), vec![]);
    let outcome = host.capture();

    // sda and sdb: serial-less, equal size, one address — grouped.
    assert!(matches!(&outcome.devices[0], DeviceCapture::Grouped { .. }));
    assert!(matches!(&outcome.devices[1], DeviceCapture::Grouped { .. }));
    let shared = derive_id(&NamingFields::PhysicalDevice {
        serial: None,
        wwn: None,
        total_bytes: 4_194_304,
    })
    .unwrap();
    let facts = outcome.snapshot.facts();
    assert!(
        !facts.table_states.contains_key(&shared)
            && !facts.extents.contains_key(&shared)
            && !facts.transports.contains_key(&shared),
        "no fact under a shared address"
    );
    assert!(
        outcome
            .snapshot
            .topology()
            .entries()
            .iter()
            .any(|entry| matches!(entry, NodeEntry::Group { count: 2, .. })),
        "the group is counted"
    );

    // sdc: held, still captured, standing in the envelope.
    assert!(matches!(
        &outcome.devices[2],
        DeviceCapture::Authored {
            state: "absent",
            ..
        }
    ));
    assert!(
        outcome
            .snapshot
            .envelope
            .provenance
            .iter()
            .any(|(key, observations)| key == "device:2:holders"
                && observations.observations.iter().any(|observation| {
                    matches!(
                        &observation.outcome,
                        partman_domain::model::provenance::Outcome::Observed {
                            value: Value::Text(text)
                        } if text == "held by 1 holder(s)"
                    )
                })),
        "the held standing is recorded, reading (b)"
    );
    assert!(
        !outcome
            .snapshot
            .topology()
            .entries()
            .iter()
            .any(|entry| matches!(
                entry,
                NodeEntry::Single {
                    fields: NamingFields::Aggregate { .. },
                    ..
                } | NodeEntry::Group {
                    fields: NamingFields::Aggregate { .. },
                    ..
                }
            )),
        "no aggregate node is emitted from a hold"
    );

    // dm-0: withdrawn.
    assert!(matches!(
        &outcome.devices[3],
        DeviceCapture::NotNamed { why, .. } if why.contains("withdrawn")
    ));
}

// Requirements: HLP-002, SAFE-005
//   A refused window is a withheld state, never a guessed one: a device
//   whose read falls short of its stated geometry gets no table-state
//   fact — honest absence, failing closed at the closure — with the
//   refusal recorded, and the rest of the capture is untouched.
// Evidence: a_short_read_withholds_the_state_and_poisons_nothing_else
#[test]
fn a_short_read_withholds_the_state_and_poisons_nothing_else() {
    let mut host = Host::new();
    host.device("sda", Some("SER-A"), image("gpt-basic-512"));
    host.device("sdb", Some("SER-B"), image("mbr-basic-512"));
    // sda's medium answers 1 MiB short of its stated size.
    let truncated = {
        let mut bytes = host.reader.images.get("sda").unwrap().clone();
        bytes.truncate(bytes.len() - 1_048_576);
        bytes
    };
    host.reader.images.insert("sda".to_owned(), truncated);
    let outcome = host.capture();
    match &outcome.devices[0] {
        DeviceCapture::NamedOnly { withheld, node, .. } => {
            assert!(withheld.contains("window read"), "{withheld}");
            assert!(!outcome.snapshot.facts().table_states.contains_key(node));
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        &outcome.devices[1],
        DeviceCapture::Authored {
            state: "present",
            scheme: Some("mbr"),
            ..
        }
    ));
}

// Requirements: PLAN-006, HLP-002
//   The windowing is the parser's caller shape exactly: the cut windows
//   equal independent slices at both ends, a medium shorter than one
//   window is read whole, and an unusable geometry refuses before any
//   byte is read.
// Evidence: the_windowing_is_exact_at_both_ends
#[test]
fn the_windowing_is_exact_at_both_ends() {
    let bytes = image("gpt-invalid-primary-valid-backup-512");
    let geometry = Geometry {
        sector_size: 512,
        total_sectors: bytes.len() as u64 / 512,
    };
    let windows = read_windows(&mut Cursor::new(&bytes), geometry).expect("windows");
    assert_eq!(windows.head, bytes[..65_536].to_vec());
    assert_eq!(windows.tail, bytes[bytes.len() - 65_536..].to_vec());

    let tiny = vec![0_u8; 8192];
    let windows = read_windows(
        &mut Cursor::new(&tiny),
        Geometry {
            sector_size: 512,
            total_sectors: 16,
        },
    )
    .expect("windows");
    assert_eq!(windows.head.len(), 8192);
    assert_eq!(windows.tail.len(), 8192);

    assert_eq!(
        read_windows(
            &mut Cursor::new(&tiny),
            Geometry {
                sector_size: 0,
                total_sectors: 16,
            },
        )
        .unwrap_err(),
        ByteRefusal::GeometryUnusable
    );
}

// Requirements: HLP-002, CAP-007, PLAN-007
//   Validate-plan is the helper re-planning over its own capture: a
//   lawful request over an authored capture validates with the helper's
//   severity and the window it stamped; SI-13's structural interim
//   refuses an aggregate target before the planner runs; a validity
//   request over PLAN-007's maximum refuses; and over a capture whose
//   transport is unrecognized — every real capture today — the
//   capability gate's refusal travels verbatim, which is the fail-closed
//   answer until the transport rows exist.
// Evidence: validate_plan_replans_over_the_capture_and_refuses_on_typed_arms
#[test]
fn validate_plan_replans_over_the_capture_and_refuses_on_typed_arms() {
    let (snapshot, device) = clean_device(b"VAL-1");
    let validated = validate_plan(
        &snapshot,
        &wipe_request(device),
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    assert_eq!(validated.severity, Severity::Destructive);
    assert_eq!(validated.not_after, NOW + 3600);
    assert_eq!(validated.snapshot_hash, snapshot.body_hash().unwrap());
    assert_eq!(validated.body_hash.as_bytes().len(), 32);

    // SI-13: an aggregate target refuses structurally.
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"POOL".to_vec()),
    };
    let pool_id = derive_id(&pool).unwrap();
    let with_pool = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![pool],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    assert!(matches!(
        validate_plan(
            &with_pool,
            &wipe_request(pool_id),
            NOW,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
        )
        .unwrap_err(),
        ValidateRefusal::AggregateTarget { target } if target == pool_id
    ));

    // PLAN-007's maximum.
    let mut over = wipe_request(device);
    over.validity_seconds = 604_801;
    assert!(matches!(
        validate_plan(
            &snapshot,
            &over,
            NOW,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
        )
        .unwrap_err(),
        ValidateRefusal::ValidityOverMaximum { requested: 604_801 }
    ));

    // The unrecognized-transport capture refuses at the capability gate.
    let mut host = Host::new();
    host.device("sda", Some("SER-A"), image("gpt-basic-512"));
    let outcome = host.capture();
    let target = derive_id(&NamingFields::PhysicalDevice {
        serial: Some(host.serial_of("sda")),
        wwn: None,
        total_bytes: 4_194_304,
    })
    .unwrap();
    match validate_plan(
        &outcome.snapshot,
        &wipe_request(target),
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .unwrap_err()
    {
        ValidateRefusal::Planner(PlanRefusal::CapabilityRefused { .. }) => {}
        other => panic!("expected the capability gate's verbatim refusal, got {other:?}"),
    }
}

// Requirements: SEC-002, PLAN-006, PLAN-007, HLP-004
//   The admission arms, in order: a consumed record replays; a foreign
//   presenter is cross-user; bytes that are not the validated plan are a
//   hash mismatch; a plan bound to another capture is stale; an identity
//   claiming a table state the fresh capture's stamp contradicts is
//   cross-device; altered bytes die at the decode boundary; and a closed
//   window is expired — each a typed, explained refusal.
// Evidence: the_admission_arms_refuse_replayed_crossuser_altered_stale_crossdevice_expired
#[test]
#[allow(clippy::too_many_lines)]
fn the_admission_arms_refuse_replayed_crossuser_altered_stale_crossdevice_expired() {
    let (snapshot, device) = clean_device(b"ADM-1");
    let validated = validate_plan(
        &snapshot,
        &wipe_request(device),
        NOW,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
    )
    .expect("validates");
    let record = ValidationRecord {
        plan_hash: validated.body_hash,
        validated_for_uid: 1000,
        consumed: false,
    };

    // The happy path admits and returns the decoded plan.
    let plan = admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 10, 1000, &record)
        .expect("admits");
    assert_eq!(plan.severity(), Severity::Destructive);

    // Replayed.
    let consumed = ValidationRecord {
        consumed: true,
        ..record.clone()
    };
    assert_eq!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 10, 1000, &consumed)
            .unwrap_err(),
        AdmissionRefusal::Replayed
    );

    // Cross-user.
    assert_eq!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 10, 1001, &record)
            .unwrap_err(),
        AdmissionRefusal::CrossUser {
            presented_by: 1001,
            validated_for: 1000
        }
    );

    // Stale: the topology moved (a fresh capture of different hardware).
    let (other_snapshot, _) = clean_device(b"ADM-2");
    assert_eq!(
        admit_presented_plan(
            &validated.body_bytes,
            &other_snapshot,
            NOW + 10,
            1000,
            &record
        )
        .unwrap_err(),
        AdmissionRefusal::Stale
    );

    // Altered: one flipped byte fails the boundary, never validates.
    let mut altered = validated.body_bytes.clone();
    let index = altered.len() / 2;
    altered[index] ^= 0x01;
    assert!(matches!(
        admit_presented_plan(&altered, &snapshot, NOW + 10, 1000, &record).unwrap_err(),
        AdmissionRefusal::Altered { .. } | AdmissionRefusal::Stale | AdmissionRefusal::CrossDevice
    ));

    // Expired: the window closed.
    assert_eq!(
        admit_presented_plan(&validated.body_bytes, &snapshot, NOW + 3601, 1000, &record)
            .unwrap_err(),
        AdmissionRefusal::Expired {
            not_after: NOW + 3600,
            now: NOW + 3601
        }
    );

    // Cross-device: a bound identity claiming a table state the fresh
    // capture's stamp contradicts.
    let (snapshot_b, device_b) = clean_device(b"ADM-3");
    let mut identities = BTreeMap::new();
    identities.insert(
        device_b,
        DeviceIdentity {
            serial: Some(b"ADM-3".to_vec()),
            wwn: None,
            os_instance_id: None,
            connection_path: None,
            total_bytes: 1 << 30,
            logical_sector_size: Some(512),
            physical_sector_size: Some(512),
            table: TableState::Absent, // the stamp says Present
            witness: None,
        },
    );
    let planned = partman_planner::plan(
        partman_planner::PlanRequest {
            operation: CapOp::Wipe,
            target: device_b,
        },
        &snapshot_b,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &partman_planner::PlanIdentity {
            plan_id: b"adm-3".to_vec(),
            created_at: NOW,
            validity: partman_domain::model::plan::ValidityWindow {
                not_after: NOW + 3600,
            },
        },
    )
    .expect("plans");
    // Rebuild the same plan body with the divergent identity attached.
    let forged = partman_domain::model::plan::OperationPlan::assemble_linked(
        b"adm-3".to_vec(),
        NOW,
        &snapshot_b,
        partman_domain::model::plan::ValidityWindow {
            not_after: NOW + 3600,
        },
        identities,
        planned.plan.steps().to_vec(),
        planned.plan.reversal().unwrap().clone(),
    )
    .expect("assembles");
    let forged_bytes = canonical::encode(&forged.body_value().expect("body")).expect("encodes");
    let forged_record = ValidationRecord {
        plan_hash: forged.body_hash().unwrap(),
        validated_for_uid: 1000,
        consumed: false,
    };
    assert_eq!(
        admit_presented_plan(&forged_bytes, &snapshot_b, NOW + 10, 1000, &forged_record)
            .unwrap_err(),
        AdmissionRefusal::CrossDevice
    );
}

// Requirements: RPC-003, RPC-005, HLP-001
//   The version-2 decode is strict in both directions: validate-plan
//   arguments on any other operation refuse as out of place, a missing
//   required argument refuses by name, a role or kind outside the closed
//   vocabulary refuses — an `Aggregate` target has no spelling at all —
//   version 1 is refused (the explicit migration), and the operation
//   name vocabulary equals the domain's exactly.
// Evidence: the_v2_decode_is_strict_in_both_directions
#[test]
#[allow(clippy::too_many_lines)]
fn the_v2_decode_is_strict_in_both_directions() {
    // A validate field on a status request is out of place.
    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(crate::REQUEST_SCHEMA.to_owned()),
    );
    map.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
    map.insert("operation".to_owned(), Value::Text("status".to_owned()));
    map.insert("plan_id".to_owned(), Value::Bytes(b"x".to_vec()));
    let bytes = canonical::encode(&Value::Map(map.clone())).unwrap();
    assert_eq!(
        Request::decode(&bytes).unwrap_err(),
        RequestRefusal::FieldOutOfPlace { key: "plan_id" }
    );

    // Versions 1 and 2 are refused: the explicit migration, and each
    // names the version it spoke so the reply can remediate (RPC-002).
    for spoken in [1_u64, 2] {
        let mut old = map.clone();
        old.remove("plan_id");
        old.insert("schema_version".to_owned(), Value::Unsigned(spoken));
        let bytes = canonical::encode(&Value::Map(old)).unwrap();
        assert_eq!(
            Request::decode(&bytes).unwrap_err(),
            RequestRefusal::WrongVersion { spoken }
        );
    }

    // A validate-plan request without its arguments refuses by name.
    let mut incomplete = map.clone();
    incomplete.remove("plan_id");
    incomplete.insert(
        "operation".to_owned(),
        Value::Text("validate-plan".to_owned()),
    );
    let bytes = canonical::encode(&Value::Map(incomplete.clone())).unwrap();
    assert_eq!(
        Request::decode(&bytes).unwrap_err(),
        RequestRefusal::MissingField { key: "target_kind" }
    );

    // An aggregate has no spelling; an unknown role refuses.
    for (kind, role, expected) in [
        (
            "aggregate",
            None,
            RequestRefusal::BadField { key: "target_kind" },
        ),
        (
            "partition-table",
            Some("zfs"),
            RequestRefusal::BadField { key: "target_role" },
        ),
    ] {
        let mut request = incomplete.clone();
        request.insert("target_kind".to_owned(), Value::Text(kind.to_owned()));
        request.insert("target_total_bytes".to_owned(), Value::Unsigned(1 << 30));
        if let Some(role) = role {
            request.insert("target_role".to_owned(), Value::Text(role.to_owned()));
        }
        request.insert(
            "requested_operation".to_owned(),
            Value::Text("wipe".to_owned()),
        );
        request.insert("plan_id".to_owned(), Value::Bytes(b"x".to_vec()));
        request.insert("validity_seconds".to_owned(), Value::Unsigned(0));
        let bytes = canonical::encode(&Value::Map(request)).unwrap();
        assert_eq!(Request::decode(&bytes).unwrap_err(), expected, "{kind}");
    }

    // The operation-name vocabulary equals the domain's, both ways.
    for operation in CapOp::all() {
        let name = crate::operation_name(*operation);
        assert_eq!(parse_operation(name), Some(*operation));
        assert!(
            partman_capability::store::OPERATION_NAMES.contains(&name),
            "{name} is the store's spelling"
        );
    }
    assert_eq!(parse_operation("rm -rf /"), None);
    assert_eq!(parse_operation("/bin/sh"), None);
}

// Requirements: HLP-001, RPC-002
//   Validate-plan over the real serve loop: a wire request round-trips
//   into `validated` with the helper-planned body, its 32-byte hashes
//   and the helper-computed severity — and the spelled target derives
//   the same address the capture derives, which is what lets a client
//   name a target without the helper trusting anything else it says.
// Evidence: the_wire_round_trips_a_validation_end_to_end
#[test]
#[allow(clippy::too_many_lines)]
fn the_wire_round_trips_a_validation_end_to_end() {
    struct PlanningBackend {
        snapshot: TopologySnapshot,
    }
    impl crate::Backend for PlanningBackend {
        fn status(&self) -> crate::Response {
            crate::Response::Status {
                build: "0.0.0".to_owned(),
                authorizing_uid: 1000,
                served: vec![],
            }
        }
        fn enumerate(&self) -> crate::Response {
            crate::Response::Enumeration {
                proposal: true,
                outcome: "listed".to_owned(),
                devices: vec![],
            }
        }
        fn validate_plan(
            &self,
            request: &ValidateWire,
            audit: &mut dyn crate::AuditSink,
        ) -> crate::Response {
            if audit
                .record(crate::AuditEvent::Captured {
                    devices: 1,
                    classified: 1,
                })
                .is_err()
            {
                return crate::Response::ValidationRefused {
                    arm: "audit".to_owned(),
                    detail: "the audit log could not be written".to_owned(),
                };
            }
            let target = request.target.derive().expect("derives");
            match validate_plan(
                &self.snapshot,
                &ValidateRequest {
                    target,
                    operation: request.requested,
                    plan_id: request.plan_id.clone(),
                    validity_seconds: request.validity_seconds,
                },
                NOW,
                &TechnologyLimits::default(),
                &RuntimeFacts::clean(),
            ) {
                Ok(validated) => crate::Response::Validated {
                    plan: validated.body_bytes,
                    plan_hash: validated.body_hash.as_bytes().to_vec(),
                    snapshot_hash: validated.snapshot_hash.as_bytes().to_vec(),
                    severity: crate::validate::severity_name(validated.severity).to_owned(),
                    flags: vec![],
                    tier: crate::authorize::required_tier(validated.severity, &validated.flags)
                        .wire_name()
                        .to_owned(),
                    not_after: validated.not_after,
                },
                Err(refusal) => crate::Response::ValidationRefused {
                    arm: "planner".to_owned(),
                    detail: format!("{refusal:?}"),
                },
            }
        }
        fn apply_plan(
            &self,
            _request: &crate::ApplyWire,
            _audit: &mut dyn crate::AuditSink,
        ) -> crate::Response {
            crate::Response::ApplyRefused {
                arm: "not-validated".to_owned(),
                detail: "this fixture backend holds no journal".to_owned(),
            }
        }
        fn journal_query(&self, _audit: &mut dyn crate::AuditSink) -> crate::Response {
            crate::Response::JournalReport {
                high_water_instant: None,
                records: 0,
                plans: vec![],
            }
        }
    }

    let (snapshot, _) = clean_device(b"WIRE-1");
    let expected_snapshot_hash = snapshot.body_hash().unwrap();
    let backend = PlanningBackend { snapshot };
    let request = Request {
        operation: Operation::ValidatePlan,
        apply: None,
        validate: Some(ValidateWire {
            target: TargetSpelling::Device {
                serial: Some(b"WIRE-1".to_vec()),
                wwn: None,
                total_bytes: 1 << 30,
            },
            requested: CapOp::Wipe,
            plan_id: b"wire-1".to_vec(),
            validity_seconds: 3600,
        }),
    };
    let (reply, audit) = super::serve_through(&request.encode().unwrap(), &backend);
    let text = |key: &str| match reply.get(key) {
        Some(Value::Text(text)) => text.clone(),
        other => panic!("{key}: {other:?}"),
    };
    assert_eq!(text("outcome"), "validated");
    assert_eq!(text("severity"), "destructive");
    match (reply.get("plan_hash"), reply.get("snapshot_hash")) {
        (Some(Value::Bytes(plan_hash)), Some(Value::Bytes(snapshot_hash))) => {
            assert_eq!(plan_hash.len(), 32);
            assert_eq!(snapshot_hash, &expected_snapshot_hash.as_bytes().to_vec());
        }
        other => panic!("hashes: {other:?}"),
    }
    assert_eq!(reply.get("not_after"), Some(&Value::Unsigned(NOW + 3600)));
    assert_eq!(
        audit,
        vec![
            crate::AuditEvent::Operation {
                operation: Some(Operation::ValidatePlan),
                outcome: "served"
            },
            crate::AuditEvent::Captured {
                devices: 1,
                classified: 1
            }
        ]
    );
}

// Requirements: INV-003, HLP-002
//   The helper's reach declaration: one cell per INV-003 state in the
//   requirement's own order, every distinguished cell measured and
//   citing a heading that exists in the observability record — a `yes`
//   never rests on inference — and the vocabulary equals the adapter's,
//   deliberately.
// Evidence: the_reach_declaration_cites_a_recorded_row_for_every_yes
#[test]
fn the_reach_declaration_cites_a_recorded_row_for_every_yes() {
    use crate::reach::{REACH, STATES, basis};
    assert_eq!(STATES, partman_adapter_linux::reach::STATES);
    let record = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/quality/observability.md"),
    )
    .expect("the observability record is in the repository");
    for (index, cell) in REACH.cells.iter().enumerate() {
        assert_eq!(cell.state, STATES[index], "INV-003's own order");
        if cell.distinguished {
            assert_eq!(cell.basis, basis::MEASURED);
            let citation = cell.citation.expect("a yes cites its row");
            assert!(
                record.contains(citation),
                "the cited heading `{citation}` is not in the record"
            );
        } else {
            assert!(cell.citation.is_none());
        }
    }
}
