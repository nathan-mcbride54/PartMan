//! Increment 3's suite: the JRN-006 record vocabulary — round-trips
//! pinned against the documented vectors, MODEL-003's explicit
//! rejection, the record-write-time effect constraints increment 1
//! deferred here, the three-variant hash-only protection record, the
//! JRN-005 redaction gate in the WP-035/WP-040 shape, and the
//! ADR-0027 disposal chain reconstructed from journal bytes alone.

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};
use partman_statemachine::{Effect, Transition};

use super::{
    ArtifactHashRef, ArtifactStore, AuthorizationAct, AuthorizationTier, Checkpoint,
    CompactionAuthority, CompactionRecord, DecodeRefused, DisposalLinkage, DryRunRefusal,
    PER_APPLY_JOURNAL_BUDGET_BYTES, PlanHashRef, ProtectionArm, ProtectionArtifactRef,
    ProtectionRecord, RECORD_SCHEMA, RECORD_SCHEMA_VERSION, Record, RecordInvalid, RecordedInstant,
    Region, TransitionRecord,
};
use crate::{CoveredRanges, Journal, MAX_PAYLOAD_LEN, SeqNo, replay};

const DOC: &str = include_str!("../../../../schemas/journal/records.md");

/// The representative instant the documented vectors pin: a plain
/// epoch-seconds reading, distinct from zero so a dropped or defaulted
/// instant cannot round-trip unnoticed.
fn instant_t() -> RecordedInstant {
    RecordedInstant::from_secs(1_700_000_000)
}

fn plan_a() -> PlanHashRef {
    PlanHashRef::from_bytes([0x11; 32])
}

fn plan_b() -> PlanHashRef {
    PlanHashRef::from_bytes([0x22; 32])
}

fn artifact() -> ProtectionArtifactRef {
    ProtectionArtifactRef::new(
        ArtifactHashRef::from_bytes([0x33; 32]),
        ArtifactStore::HelperProtectionStore,
    )
}

/// One representative record per wire kind and protection arm — the
/// set the documented vectors pin and the sweeps walk.
fn representatives() -> Vec<Record> {
    vec![
        Record::AuthorizationAct(AuthorizationAct::new(plan_a(), AuthorizationTier::FloorAct)),
        Record::Transition(
            TransitionRecord::non_terminal(plan_a(), Transition::ValidatorPasses, instant_t())
                .expect("non-terminal row"),
        ),
        Record::Transition(
            TransitionRecord::terminal(
                plan_a(),
                Transition::FailureAccepted,
                Effect::Partial,
                Some(DisposalLinkage::new(plan_b())),
                instant_t(),
            )
            .expect("the disposal arm"),
        ),
        Record::Checkpoint(Checkpoint::new(plan_a(), 3)),
        Record::Protection(
            ProtectionRecord::new(
                plan_a(),
                ProtectionArm::ParseBackupVerified {
                    artifact: artifact(),
                },
            )
            .expect("present arm"),
        ),
        Record::Protection(
            ProtectionRecord::new(plan_a(), ProtectionArm::AbsenceDetermined).expect("absent arm"),
        ),
        Record::Protection(
            ProtectionRecord::new(
                plan_a(),
                ProtectionArm::RawCaptureVerified {
                    artifact: artifact(),
                    regions: vec![
                        Region {
                            start: 512,
                            length: 8,
                        },
                        Region {
                            start: 4_193_792,
                            length: 8,
                        },
                    ],
                },
            )
            .expect("capture arm"),
        ),
        Record::Compaction(
            CompactionRecord::new(
                SeqNo::from_raw(1),
                SeqNo::from_raw(2),
                CompactionAuthority::TerminalHistoryRetention,
            )
            .expect("forward range"),
        ),
    ]
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String");
    }
    out
}

/// Decode a record's canonical bytes back to its `pce/1` map for
/// tampering.
fn decoded_map(record: &Record) -> BTreeMap<String, Value> {
    let bytes = record.encode().expect("encodable");
    match canonical::decode(&bytes).expect("canonical") {
        Value::Map(map) => map,
        other => panic!("a record must encode as a map, not {other:?}"),
    }
}

fn encoded(map: BTreeMap<String, Value>) -> Vec<u8> {
    canonical::encode(&Value::Map(map)).expect("encodable")
}

/// The complete closed text vocabulary of schema v2 — every `Text`
/// value any encoded record may carry, transcribed from the schema
/// document's own listing rather than from the encoder.
fn closed_vocabulary() -> Vec<&'static str> {
    let mut vocabulary = vec![
        "partman.journal.record",
        "authorization-act",
        "transition",
        "checkpoint",
        "protection",
        "compaction",
        "floor-act",
        "interactive-ceremony",
        "no-writes",
        "partial",
        "complete",
        "absence-determined",
        "parse-backup-verified",
        "raw-capture-verified",
        "helper-protection-store",
        "terminal-history-retention",
    ];
    vocabulary.extend(TRANSITION_TAGS);
    vocabulary
}

/// The 23 transition tags, transcribed independently in the published
/// table's row order.
const TRANSITION_TAGS: [&str; 23] = [
    "validator-passes",
    "edit-or-invalidation",
    "apply-submitted",
    "authorization-granted",
    "declined-or-expired",
    "revalidation-passes",
    "identity-mismatch",
    "backups-verified",
    "backup-failure",
    "final-step-complete",
    "user-pauses",
    "reboot-step-reached",
    "step-failure-or-interruption",
    "cancel-honored",
    "user-resumes",
    "cancel-while-paused",
    "topology-changed-while-paused",
    "reboot-resume",
    "resume-impossible",
    "postconditions-pass",
    "postcondition-failure",
    "roll-forward-selected",
    "failure-accepted",
];

/// The declared field-name set, transcribed from the schema document.
const FIELD_NAMES: [&str; 14] = [
    "schema",
    "schema_version",
    "kind",
    "plan",
    "tier",
    "transition",
    "instant",
    "effect",
    "recovery_plan",
    "step_index",
    "arm",
    "artifact",
    "store",
    "regions",
];

const REGION_FIELD_NAMES: [&str; 2] = ["start", "length"];
const COMPACTION_FIELD_NAMES: [&str; 3] = ["first", "last", "authority"];

/// One raw exemplar per SEC-006 identifier class, as WP-040's gate
/// spells them.
const EXEMPLARS: [&str; 7] = [
    "WD-WCC4N5PZ3RKE",
    "/dev/disk/by-id/ata-WDC_WD40EFRX-68N32N0",
    "C:\\Users\\nate\\backup.img",
    "Backup Disk 2",
    "nate",
    "-----BEGIN OPENSSH PRIVATE KEY-----",
    "backup.img",
];

// Requirements: JRN-006, MODEL-003
//   The versioned journal record schema: every v2 record class —
//   authorization act, non-terminal and terminal-with-linkage
//   transitions, checkpoint, all three protection arms, compaction —
//   encodes through WP-010's pce/1 codec carrying the pinned schema
//   identifier and version, decodes back to an equal value, and is
//   pinned byte-for-byte against the golden vectors published in
//   schemas/journal/records.md, so the document and the encoder cannot
//   drift apart silently; ADR-0029's per-apply budget constant lands
//   with the schema and is pinned in the same document.
// Evidence: every_record_class_round_trips_and_matches_the_documented_vectors
#[test]
fn every_record_class_round_trips_and_matches_the_documented_vectors() {
    assert!(DOC.contains(RECORD_SCHEMA), "the doc pins the schema id");
    assert!(
        DOC.contains(&format!(
            "`schema_version` | Unsigned | always `{RECORD_SCHEMA_VERSION}`"
        )),
        "the doc pins the version"
    );
    assert!(
        DOC.contains(&PER_APPLY_JOURNAL_BUDGET_BYTES.to_string()),
        "ADR-0029's budget constant lands with the schema document"
    );
    assert!(
        u64::try_from(MAX_PAYLOAD_LEN).expect("small") < PER_APPLY_JOURNAL_BUDGET_BYTES,
        "the budget admits many maximum-size frames"
    );

    for record in representatives() {
        let bytes = record.encode().expect("every representative encodes");
        assert!(
            bytes.len() <= MAX_PAYLOAD_LEN,
            "every record fits a frame payload"
        );
        assert_eq!(
            Record::decode(&bytes).expect("every representative decodes"),
            record,
            "round-trip must be identity"
        );
        let vector = hex(&bytes);
        assert!(
            DOC.contains(&vector),
            "schemas/journal/records.md must publish this vector for {}: {vector}",
            record.kind()
        );
    }

    let _ = DryRunRefusal::PendingQualification;
}

// Requirements: MODEL-003, JRN-006
//   Explicit rejection, never silent acceptance or repair: a bumped or
//   missing schema version, a wrong schema identifier, a non-map, a
//   non-canonical byte stream, an unknown kind, an unknown field, a
//   mistyped field, a wrong-length hash, a sequence zero, and a
//   backwards compaction range each return their own typed refusal —
//   and the wire cannot smuggle a shape the constructors refuse,
//   because decode routes through the same validation.
// Evidence: unknown_versions_kinds_fields_and_tags_refuse_rather_than_repair
#[test]
fn unknown_versions_kinds_fields_and_tags_refuse_rather_than_repair() {
    let act = &representatives()[0];

    // The retired v1 refuses by version before any field is read —
    // MODEL-003's explicit rejection, with nothing to migrate: no v1
    // byte ever reached disk, because no journal on-disk home existed
    // while v1 was current.
    let mut map = decoded_map(act);
    map.insert("schema_version".to_owned(), Value::Unsigned(1));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongVersion)
    );

    let mut map = decoded_map(act);
    map.insert("schema_version".to_owned(), Value::Unsigned(3));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongVersion)
    );

    let mut map = decoded_map(act);
    map.remove("schema_version");
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongVersion)
    );

    let mut map = decoded_map(act);
    map.insert("schema".to_owned(), Value::Text("partman.plan".into()));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongSchema)
    );

    assert_eq!(
        Record::decode(&canonical::encode(&Value::Unsigned(7)).expect("encodable")),
        Err(DecodeRefused::NotAMap)
    );
    assert!(matches!(
        Record::decode(&[0xff, 0x00]),
        Err(DecodeRefused::NotCanonical(_))
    ));

    let mut map = decoded_map(act);
    map.insert("kind".to_owned(), Value::Text("journal-open".into()));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::UnknownTag { field: "kind" })
    );

    let mut map = decoded_map(act);
    map.insert("surplus".to_owned(), Value::Unsigned(1));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::UnknownField)
    );

    let mut map = decoded_map(act);
    map.remove("tier");
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::MissingField { field: "tier" })
    );

    let mut map = decoded_map(act);
    map.insert("plan".to_owned(), Value::Unsigned(4));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongType { field: "plan" })
    );

    let mut map = decoded_map(act);
    map.insert("plan".to_owned(), Value::Bytes(vec![0x11; 31]));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::WrongHashLength { field: "plan" })
    );

    let compaction = representatives().pop().expect("nonempty");
    let mut map = decoded_map(&compaction);
    map.insert("first".to_owned(), Value::Unsigned(0));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::Invalid(RecordInvalid::SequenceZero))
    );

    let mut map = decoded_map(&compaction);
    map.insert("first".to_owned(), Value::Unsigned(9));
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::Invalid(
            RecordInvalid::CompactionRangeBackwards {
                first: SeqNo::from_raw(9),
                last: SeqNo::from_raw(2),
            }
        ))
    );

    assert_eq!(
        CompactionRecord::new(
            SeqNo::from_raw(9),
            SeqNo::from_raw(2),
            CompactionAuthority::TerminalHistoryRetention
        ),
        Err(RecordInvalid::CompactionRangeBackwards {
            first: SeqNo::from_raw(9),
            last: SeqNo::from_raw(2),
        }),
        "the constructor refuses what the wire refuses"
    );
}

// Requirements: Section 8
//   The record-write-time effect check increment 1 deferred here, taken
//   over every published row: a non-terminal row refuses an effect and
//   a terminal row demands one; where the published row states an
//   effect constraint the record accepts exactly the constrained
//   effects (no-writes alone on the three no-writes rows,
//   no-writes-or-partial on the honored cancel) and an unconstrained
//   terminal row accepts all three; and ADR-0027's disposal linkage is
//   accepted on the failure-accepted row alone — every other row,
//   terminal or not, refuses it by name, on construction and on the
//   wire alike.
// Evidence: terminal_effects_are_enforced_at_record_write_time_for_every_row
#[test]
fn terminal_effects_are_enforced_at_record_write_time_for_every_row() {
    let effects = [Effect::NoWrites, Effect::Partial, Effect::Complete];
    for transition in Transition::ALL {
        let terminal = transition.to().is_terminal();
        assert_eq!(
            TransitionRecord::non_terminal(plan_a(), transition, instant_t()).is_ok(),
            !terminal,
            "{transition:?}: non-terminal construction iff the row is non-terminal"
        );
        for effect in effects {
            let allowed = match transition.effect_constraint() {
                Some(list) => list.contains(&effect),
                None => true,
            };
            let record =
                TransitionRecord::terminal(plan_a(), transition, effect, None, instant_t());
            if !terminal {
                assert_eq!(
                    record,
                    Err(RecordInvalid::EffectOnNonTerminal { transition }),
                    "{transition:?} takes no effect"
                );
            } else if allowed {
                let record = record.expect("constrained effect accepted");
                let bytes = Record::Transition(record).encode().expect("encodes");
                assert_eq!(
                    Record::decode(&bytes).expect("decodes"),
                    Record::Transition(record),
                    "{transition:?} with {effect:?} round-trips"
                );
            } else {
                assert_eq!(
                    record,
                    Err(RecordInvalid::EffectOutsideConstraint { transition, effect }),
                    "{transition:?} refuses {effect:?} outside the published constraint"
                );
            }
        }

        let linkage = Some(DisposalLinkage::new(plan_b()));
        if terminal {
            // Probe the linkage rule with an effect the row itself
            // allows, so the refusal under test is the linkage's own.
            let legal_effect = transition
                .effect_constraint()
                .map_or(Effect::Partial, |list| list[0]);
            let with_linkage = TransitionRecord::terminal(
                plan_a(),
                transition,
                legal_effect,
                linkage,
                instant_t(),
            );
            if matches!(transition, Transition::FailureAccepted) {
                let record = with_linkage.expect("the disposal arm carries the linkage");
                assert_eq!(record.disposal(), linkage);
            } else {
                assert_eq!(
                    with_linkage,
                    Err(RecordInvalid::LinkageOutsideDisposalArm { transition }),
                    "{transition:?} is not the disposal arm"
                );
            }
        }
    }

    // The wire cannot smuggle a linkage onto a non-terminal row: the
    // decoder routes the same invariant.
    let non_terminal =
        TransitionRecord::non_terminal(plan_a(), Transition::ValidatorPasses, instant_t())
            .expect("non-terminal row");
    let mut map = decoded_map(&Record::Transition(non_terminal));
    map.insert(
        "recovery_plan".to_owned(),
        Value::Bytes(plan_b().as_bytes().to_vec()),
    );
    assert_eq!(
        Record::decode(&encoded(map)),
        Err(DecodeRefused::Invalid(
            RecordInvalid::LinkageOutsideDisposalArm {
                transition: Transition::ValidatorPasses,
            }
        ))
    );
}

// Requirements: JRN-006
//   The protection record is exactly three-variant (ADR-0024) and
//   hash-only (ADR-0030): a verified parse-level backup, a positively
//   determined absence, and a verified raw capture each construct and
//   round-trip; a raw capture's regions are validated — empty, zero
//   length, unordered, overlapping, and overflowing each a typed
//   refusal naming the offending index; and every byte-string position
//   in every encoded protection record is exactly 32 bytes, so the
//   artifact's bytes have no field to occupy — "never its bytes" held
//   structurally, not by policy.
// Evidence: the_protection_record_is_three_variant_and_carries_hashes_only
#[test]
fn the_protection_record_is_three_variant_and_carries_hashes_only() {
    let capture = |regions: Vec<Region>| {
        ProtectionRecord::new(
            plan_a(),
            ProtectionArm::RawCaptureVerified {
                artifact: artifact(),
                regions,
            },
        )
    };

    assert_eq!(capture(vec![]), Err(RecordInvalid::RegionsEmpty));
    assert_eq!(
        capture(vec![Region {
            start: 4,
            length: 0
        }]),
        Err(RecordInvalid::RegionZeroLength { index: 0 })
    );
    assert_eq!(
        capture(vec![
            Region {
                start: 512,
                length: 8
            },
            Region {
                start: 100,
                length: 8
            },
        ]),
        Err(RecordInvalid::RegionsNotAscendingOrOverlapping { index: 1 })
    );
    assert_eq!(
        capture(vec![
            Region {
                start: 512,
                length: 8
            },
            Region {
                start: 519,
                length: 1
            },
        ]),
        Err(RecordInvalid::RegionsNotAscendingOrOverlapping { index: 1 })
    );
    assert_eq!(
        capture(vec![Region {
            start: u64::MAX,
            length: 1
        }]),
        Err(RecordInvalid::RegionOverflow { index: 0 })
    );

    for record in representatives() {
        let Record::Protection(_) = &record else {
            continue;
        };
        let bytes = record.encode().expect("encodes");
        let value = canonical::decode(&bytes).expect("canonical");
        let mut byte_leaves = Vec::new();
        collect_bytes(&value, &mut byte_leaves);
        assert!(
            byte_leaves.iter().all(|leaf| leaf.len() == 32),
            "every byte-string position is a 32-byte hash; bulk has no field"
        );
    }
}

fn collect_bytes(value: &Value, out: &mut Vec<Vec<u8>>) {
    match value {
        Value::Bytes(bytes) => out.push(bytes.clone()),
        Value::Array(items) => {
            for item in items {
                collect_bytes(item, out);
            }
        }
        Value::Map(map) => {
            for item in map.values() {
                collect_bytes(item, out);
            }
        }
        _ => {}
    }
}

// Requirements: JRN-005
//   Journals never contain secrets, held structurally at the schema
//   layer: no v1 record class has a free-text position — every Text
//   value in every encoded representative belongs to the closed
//   vocabulary transcribed from the schema document, and every map key
//   to the declared field set — and the WP-035/WP-040-shaped gate
//   plants a raw exemplar of every SEC-006 identifier class in every
//   text-capable position, as every unknown field's own key, and as a
//   mistyped hash, proving each refuses; no refusal echoes the planted
//   content back, asserted over the refusal's own rendering. Bounded
//   embedded tool output is the helper packages' surface, and no field
//   here can carry it.
// Evidence: no_record_position_carries_free_text_and_planted_identifiers_refuse
#[test]
fn no_record_position_carries_free_text_and_planted_identifiers_refuse() {
    let vocabulary = closed_vocabulary();
    for record in representatives() {
        let bytes = record.encode().expect("encodes");
        let value = canonical::decode(&bytes).expect("canonical");
        assert_text_closed(&value, &vocabulary);
    }

    let text_positions: [&str; 7] = [
        "schema",
        "kind",
        "tier",
        "transition",
        "effect",
        "store",
        "authority",
    ];
    for raw in EXEMPLARS {
        for record in representatives() {
            let map = decoded_map(&record);
            for position in text_positions {
                if !map.contains_key(position) {
                    continue;
                }
                let mut tampered = map.clone();
                tampered.insert(position.to_owned(), Value::Text(raw.into()));
                let refusal = Record::decode(&encoded(tampered))
                    .expect_err("a planted identifier must refuse");
                assert!(
                    !format!("{refusal:?}").contains(raw),
                    "the refusal must not echo the plant"
                );
            }

            let mut tampered = map.clone();
            tampered.insert(raw.to_owned(), Value::Null);
            assert_eq!(
                Record::decode(&encoded(tampered)),
                Err(DecodeRefused::UnknownField),
                "an exemplar as a field's own key refuses without echo"
            );

            if map.contains_key("plan") {
                let mut tampered = map.clone();
                tampered.insert("plan".to_owned(), Value::Text(raw.into()));
                assert_eq!(
                    Record::decode(&encoded(tampered)),
                    Err(DecodeRefused::WrongType { field: "plan" }),
                    "a hash position cannot be retyped to carry text"
                );
            }

            if map.contains_key("instant") {
                let mut tampered = map.clone();
                tampered.insert("instant".to_owned(), Value::Text(raw.into()));
                assert_eq!(
                    Record::decode(&encoded(tampered)),
                    Err(DecodeRefused::WrongType { field: "instant" }),
                    "the instant position cannot be retyped to carry text"
                );
            }
        }
    }
}

fn assert_text_closed(value: &Value, vocabulary: &[&str]) {
    match value {
        Value::Text(text) => {
            assert!(
                vocabulary.contains(&text.as_str()),
                "free text outside the closed vocabulary: {text:?}"
            );
        }
        Value::Array(items) => {
            for item in items {
                assert_text_closed(item, vocabulary);
            }
        }
        Value::Map(map) => {
            for (key, item) in map {
                assert!(
                    FIELD_NAMES.contains(&key.as_str())
                        || REGION_FIELD_NAMES.contains(&key.as_str())
                        || COMPACTION_FIELD_NAMES.contains(&key.as_str()),
                    "undeclared field name: {key:?}"
                );
                assert_text_closed(item, vocabulary);
            }
        }
        _ => {}
    }
}

// Requirements: JRN-006, JRN-003
//   ADR-0027's chain, this increment's half of imported obligation 3:
//   a journal holding plan A's authorization act, its transitions to a
//   Failed-by-recovery-selection terminal carrying the disposal
//   linkage, and plan B's authorization act — followed by a torn tail
//   from a crash mid-append — replays and decodes to the full chain
//   from the bytes alone: A's terminal names B, B's act is present,
//   and running the reconstruction twice yields identical chains, with
//   no input but the bytes. The compacted-journal half of the
//   obligation is increment 4's, as the assignment records.
// Evidence: the_disposal_chain_reconstructs_from_the_journal_alone
#[test]
fn the_disposal_chain_reconstructs_from_the_journal_alone() {
    let mut journal = Journal::new();
    let chain: Vec<Record> = vec![
        Record::AuthorizationAct(AuthorizationAct::new(plan_a(), AuthorizationTier::FloorAct)),
        Record::Transition(
            TransitionRecord::non_terminal(
                plan_a(),
                Transition::StepFailureOrInterruption,
                instant_t(),
            )
            .expect("row"),
        ),
        Record::Transition(
            TransitionRecord::terminal(
                plan_a(),
                Transition::FailureAccepted,
                Effect::Partial,
                Some(DisposalLinkage::new(plan_b())),
                instant_t(),
            )
            .expect("the disposal arm"),
        ),
        Record::AuthorizationAct(AuthorizationAct::new(
            plan_b(),
            AuthorizationTier::InteractiveCeremony,
        )),
    ];
    for record in &chain {
        journal
            .append(&record.encode().expect("encodes"))
            .expect("bounded");
    }
    journal
        .append(
            &Record::Checkpoint(Checkpoint::new(plan_b(), 0))
                .encode()
                .expect("encodes"),
        )
        .expect("bounded");
    let torn = &journal.bytes()[..journal.bytes().len() - 9];

    let reconstruct = |bytes: &[u8]| -> Vec<Record> {
        let replayed = replay(bytes, &CoveredRanges::none()).expect("torn tail truncates");
        replayed
            .records()
            .iter()
            .map(|frame| Record::decode(frame.payload()).expect("every surviving payload decodes"))
            .collect()
    };

    let first_pass = reconstruct(torn);
    assert_eq!(first_pass, chain, "the crash-cut record survives whole");
    assert_eq!(
        first_pass,
        reconstruct(torn),
        "reconstruction is idempotent"
    );

    let Record::Transition(terminal) = &first_pass[2] else {
        panic!("the third record is A's terminal");
    };
    assert_eq!(terminal.plan(), plan_a());
    let linkage = terminal
        .disposal()
        .expect("the terminal names its recovery plan");
    assert_eq!(linkage.recovery_plan(), plan_b());
    let recovery_act = first_pass
        .iter()
        .find_map(|record| match record {
            Record::AuthorizationAct(act) if act.plan() == linkage.recovery_plan() => Some(act),
            _ => None,
        })
        .expect("the recovery plan's act is in the journal");
    assert_eq!(recovery_act.tier(), AuthorizationTier::InteractiveCeremony);
}

// Requirements: JRN-006, MODEL-003
//   Schema v2's recorded instant (the WP-L110 increment-4 shape round,
//   transition-only as decided): every transition record carries the
//   caller's clock reading and returns it unchanged through the wire —
//   two records differing only in their instants encode differently
//   and each decodes to its own reading, so a dropped or defaulted
//   instant cannot round-trip. A transition record without the instant
//   refuses MissingField rather than defaulting: a defaulted instant of
//   zero would sit below every honest reading and fail the consumer's
//   backward-clock bound open. A mistyped instant refuses by position.
//   The retired v1's exact shape — version 1, no instant field — is
//   refused by version before any field is read, with nothing to
//   migrate, because no v1 byte ever reached disk.
// Evidence: the_recorded_instant_is_required_and_round_trips_and_v1_refuses
#[test]
fn the_recorded_instant_is_required_and_round_trips_and_v1_refuses() {
    let at = |secs: u64| {
        Record::Transition(
            TransitionRecord::non_terminal(
                plan_a(),
                Transition::ValidatorPasses,
                RecordedInstant::from_secs(secs),
            )
            .expect("non-terminal row"),
        )
    };

    let early = at(1_700_000_000);
    let late = at(1_700_000_001);
    let early_bytes = early.encode().expect("encodes");
    let late_bytes = late.encode().expect("encodes");
    assert_ne!(
        early_bytes, late_bytes,
        "the instant is on the wire: distinct readings encode distinctly"
    );
    let Record::Transition(decoded) = Record::decode(&late_bytes).expect("decodes") else {
        panic!("a transition record decodes to a transition record");
    };
    assert_eq!(
        decoded.instant(),
        RecordedInstant::from_secs(1_700_000_001),
        "the wire returns the caller's own reading, unjudged"
    );
    assert!(
        RecordedInstant::from_secs(1_700_000_000) < decoded.instant(),
        "instants order, so a high-water maximum is well-defined for the consumer"
    );

    let mut absent = decoded_map(&early);
    absent.remove("instant");
    assert_eq!(
        Record::decode(&encoded(absent)),
        Err(DecodeRefused::MissingField { field: "instant" }),
        "an instant-less transition record refuses; nothing defaults"
    );

    // The retired v1's exact shape: version 1 and no instant field.
    let mut v1_shape = decoded_map(&early);
    v1_shape.insert("schema_version".to_owned(), Value::Unsigned(1));
    v1_shape.remove("instant");
    assert_eq!(
        Record::decode(&encoded(v1_shape)),
        Err(DecodeRefused::WrongVersion),
        "a v1 record refuses by version, before any field is read"
    );
}
