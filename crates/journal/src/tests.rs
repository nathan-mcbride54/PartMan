//! Increment 2's suite: append-only checksummed monotonic frames, the
//! torn-tail rule swept over every byte cut, interior damage refusing
//! rather than truncating, the three-way gap classification with the
//! mid-chain-gap fixture, the JRN-002 durability seam as a typed
//! boundary, and idempotent replay.
//!
//! Frames used as fixtures are forged by [`forged_frame`] over an
//! independent bit-by-bit CRC-32 transcription — deliberately a second
//! spelling of the format, so the encoding tests compare the crate
//! against the format's definition rather than against itself, and so
//! corruption fixtures (sequence zero, over-bound claims, gap splices)
//! can exist without the writer being able to produce them.

use crate::{
    AppendRefused, CoveredRanges, DurabilityRefused, DurabilitySeam, InvalidSpan, Journal,
    MAX_PAYLOAD_LEN, ReplayRefused, ReplayedRecord, SeqNo, TornTail, replay,
};

/// CRC-32 (IEEE 802.3), transcribed independently bit-by-bit — no
/// table, unlike the crate's implementation.
fn reference_crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Encode one frame from the format's definition: sequence (u64 LE),
/// payload length (u32 LE), payload, CRC-32 over everything before it
/// (u32 LE). Forging is unrestricted where the writer is not: invalid
/// sequence numbers and over-bound lengths are expressible here so the
/// corruption fixtures exist.
fn forged_frame(seq: u64, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(&seq.to_le_bytes());
    frame.extend_from_slice(
        &u32::try_from(payload.len())
            .expect("test payload")
            .to_le_bytes(),
    );
    frame.extend_from_slice(payload);
    let crc = reference_crc32(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());
    frame
}

fn forged_log(entries: &[(u64, &[u8])]) -> Vec<u8> {
    let mut log = Vec::new();
    for &(seq, payload) in entries {
        log.extend_from_slice(&forged_frame(seq, payload));
    }
    log
}

fn journal_with(payloads: &[&[u8]]) -> Journal {
    let mut journal = Journal::new();
    for payload in payloads {
        journal.append(payload).expect("bounded test payload");
    }
    journal
}

fn seqs(records: &[ReplayedRecord]) -> Vec<u64> {
    records.iter().map(|record| record.seq().get()).collect()
}

struct FakeSeam {
    offered: Vec<Vec<u8>>,
    refuse: bool,
}

impl FakeSeam {
    fn acknowledging() -> Self {
        FakeSeam {
            offered: Vec::new(),
            refuse: false,
        }
    }

    fn refusing() -> Self {
        FakeSeam {
            offered: Vec::new(),
            refuse: true,
        }
    }
}

impl DurabilitySeam for FakeSeam {
    fn make_durable(&mut self, new_bytes: &[u8]) -> Result<(), DurabilityRefused> {
        self.offered.push(new_bytes.to_vec());
        if self.refuse {
            return Err(DurabilityRefused {
                reason: "injected refusal".to_owned(),
            });
        }
        Ok(())
    }
}

const PAYLOADS: [&[u8]; 3] = [b"first", b"second-longer", b"third"];

/// Cumulative frame-end offsets for a log of the given payloads,
/// starting with 0 — computed from the format's arithmetic (16
/// overhead bytes per frame), not from the crate.
fn frame_ends(payloads: &[&[u8]]) -> Vec<usize> {
    let mut ends = vec![0];
    let mut total = 0;
    for payload in payloads {
        total += 16 + payload.len();
        ends.push(total);
    }
    ends
}

// Requirements: JRN-001
//   Append-only with per-record checksums and monotonic sequence
//   numbers: sequence numbers advance by exactly one from SeqNo::FIRST,
//   every earlier byte snapshot is a prefix of every later one, the
//   encoded log is byte-equal to an independent transcription of the
//   frame format (sequence, length, payload, CRC-32/IEEE trailer —
//   the reference implementation pinned to the standard check value),
//   an over-bound payload is a typed refusal leaving the journal
//   unchanged, and replay round-trips every record in order.
// Evidence: appends_are_append_only_checksummed_and_monotonic
#[test]
fn appends_are_append_only_checksummed_and_monotonic() {
    assert_eq!(
        reference_crc32(b"123456789"),
        0xCBF4_3926,
        "the reference transcription must match CRC-32's standard check value"
    );

    let mut journal = Journal::new();
    let mut snapshots = vec![journal.bytes().to_vec()];
    for (index, payload) in PAYLOADS.iter().enumerate() {
        let appended = journal.append(payload).expect("bounded payload");
        assert_eq!(
            appended.seq().get(),
            u64::try_from(index).expect("small") + 1,
            "sequence numbers advance by exactly one from SeqNo::FIRST"
        );
        snapshots.push(journal.bytes().to_vec());
    }
    assert_eq!(journal.next_seq().get(), 4);
    for pair in snapshots.windows(2) {
        assert!(
            pair[1].starts_with(&pair[0]),
            "every earlier snapshot is a byte prefix of every later one"
        );
    }

    let forged = forged_log(&[(1, PAYLOADS[0]), (2, PAYLOADS[1]), (3, PAYLOADS[2])]);
    assert_eq!(
        journal.bytes(),
        forged.as_slice(),
        "the writer's encoding must equal the independent transcription"
    );

    let oversize = vec![0u8; MAX_PAYLOAD_LEN + 1];
    let before = journal.bytes().to_vec();
    assert_eq!(
        journal.append(&oversize),
        Err(AppendRefused::PayloadOverBound {
            len: MAX_PAYLOAD_LEN + 1,
            bound: MAX_PAYLOAD_LEN,
        })
    );
    assert_eq!(journal.bytes(), before, "a refused append changes nothing");
    assert_eq!(journal.next_seq().get(), 4);

    let replayed = replay(journal.bytes(), &CoveredRanges::none()).expect("intact log");
    assert_eq!(replayed.truncation(), None);
    assert_eq!(seqs(replayed.records()), [1, 2, 3]);
    for (record, payload) in replayed.records().iter().zip(PAYLOADS) {
        assert_eq!(record.payload(), payload, "payloads round-trip in order");
    }
}

// Requirements: JRN-001
//   A torn tail is detected and safely truncated: for every byte cut of
//   the log — swept over all of them, frame boundaries and every
//   position inside every frame — replay returns exactly the records of
//   the longest intact frame prefix, with a truncation naming the valid
//   length and the dropped bytes when the cut is not on a boundary and
//   no truncation when it is; and a complete final frame whose checksum
//   fails (the interrupted-append shape at the tail) truncates the same
//   way, never refusing and never losing a record short of the tail.
// Evidence: a_torn_tail_is_detected_and_safely_truncated_at_every_cut
#[test]
fn a_torn_tail_is_detected_and_safely_truncated_at_every_cut() {
    let journal = journal_with(&PAYLOADS);
    let full = journal.bytes();
    let ends = frame_ends(&PAYLOADS);
    assert_eq!(*ends.last().expect("nonempty"), full.len());

    for cut in 0..=full.len() {
        let replayed = replay(&full[..cut], &CoveredRanges::none())
            .unwrap_or_else(|refused| panic!("cut {cut} must never refuse: {refused:?}"));
        let valid = *ends
            .iter()
            .filter(|&&end| end <= cut)
            .max()
            .expect("0 is always a boundary");
        let intact = ends.iter().position(|&end| end == valid).expect("member");
        assert_eq!(
            seqs(replayed.records()),
            (1..=u64::try_from(intact).expect("small")).collect::<Vec<_>>(),
            "cut {cut}: exactly the intact prefix's records survive"
        );
        if cut == valid {
            assert_eq!(replayed.truncation(), None, "cut {cut} is a clean boundary");
        } else {
            assert_eq!(
                replayed.truncation(),
                Some(TornTail {
                    valid_len: valid,
                    dropped_len: cut - valid,
                }),
                "cut {cut}: the tail past {valid} is truncated"
            );
        }
    }

    let mut tail_damaged = full.to_vec();
    let last = tail_damaged.len() - 1;
    tail_damaged[last] ^= 0xFF;
    let replayed = replay(&tail_damaged, &CoveredRanges::none()).expect("tail damage truncates");
    assert_eq!(seqs(replayed.records()), [1, 2]);
    assert_eq!(
        replayed.truncation(),
        Some(TornTail {
            valid_len: ends[2],
            dropped_len: full.len() - ends[2],
        }),
        "a damaged complete final frame is the interrupted-append shape"
    );

    let mut tail_payload_damaged = full.to_vec();
    tail_payload_damaged[ends[2] + 13] ^= 0x01;
    assert_eq!(
        replay(&tail_payload_damaged, &CoveredRanges::none()).expect("tail damage truncates"),
        replayed,
        "damage anywhere inside the final frame truncates identically"
    );
}

// Requirements: JRN-001
//   Interior damage refuses rather than truncates: a checksum mismatch
//   with bytes behind it, a duplicated frame (the sequence not
//   advancing — a shape the append-only writer cannot produce), a
//   forged sequence zero, and a complete frame claiming an over-bound
//   payload each return a typed refusal naming the defect and its
//   offset — safe truncation is the tail's rule alone, so damage can
//   never silently shorten the middle of a journal; the same
//   over-bound claim cut short at the tail truncates instead, because
//   an interrupted append can leave exactly those bytes.
// Evidence: interior_damage_refuses_rather_than_truncates
#[test]
fn interior_damage_refuses_rather_than_truncates() {
    let journal = journal_with(&PAYLOADS);
    let ends = frame_ends(&PAYLOADS);

    let mut interior_damaged = journal.bytes().to_vec();
    interior_damaged[ends[1] + 12 + 1] ^= 0x01;
    assert_eq!(
        replay(&interior_damaged, &CoveredRanges::none()),
        Err(ReplayRefused::InteriorChecksumMismatch {
            seq_claimed: 2,
            frame_start: ends[1],
        })
    );

    let single = journal_with(&[b"solo"]).bytes().to_vec();
    let mut doubled = single.clone();
    doubled.extend_from_slice(&single);
    assert_eq!(
        replay(&doubled, &CoveredRanges::none()),
        Err(ReplayRefused::SequenceRegression {
            previous: Some(SeqNo(1)),
            found: 1,
            frame_start: single.len(),
        })
    );

    assert_eq!(
        replay(&forged_frame(0, b"zero"), &CoveredRanges::none()),
        Err(ReplayRefused::SequenceRegression {
            previous: None,
            found: 0,
            frame_start: 0,
        })
    );

    let over_bound_payload = vec![0u8; MAX_PAYLOAD_LEN + 1];
    let mut log = forged_frame(1, b"good");
    let over_bound_at = log.len();
    log.extend_from_slice(&forged_frame(2, &over_bound_payload));
    assert_eq!(
        replay(&log, &CoveredRanges::none()),
        Err(ReplayRefused::FrameOverBound {
            claimed_len: u64::try_from(MAX_PAYLOAD_LEN).expect("small") + 1,
            frame_start: over_bound_at,
        }),
        "a complete over-bound frame is corruption this writer cannot produce"
    );

    let cut_short = &log[..log.len() - 10];
    let replayed = replay(cut_short, &CoveredRanges::none()).expect("incomplete claim truncates");
    assert_eq!(seqs(replayed.records()), [1]);
    assert_eq!(
        replayed.truncation(),
        Some(TornTail {
            valid_len: over_bound_at,
            dropped_len: cut_short.len() - over_bound_at,
        }),
        "the same over-bound claim cut short at the tail is a torn tail"
    );
}

// Requirements: JRN-004
//   Obligation 11's core (ADR-0029, shared with increment 4): replay
//   classifies every sequence gap three ways — compaction-covered
//   proceeds, a torn tail truncates under JRN-001's rule, and any
//   uncovered or partially covered gap refuses as the named
//   mid-chain-gap corruption case, at the journal's head and in the
//   chain alike, with the missing range and its neighbours named in
//   the refusal. The covered ranges are the classification's typed
//   input here, constructed directly with spans validated and merged;
//   deriving them from durable compaction records — and retention, the
//   per-apply budget, and monotonicity across real compaction — is
//   increment 4's, as the assignment records.
// Evidence: replay_classifies_every_gap_compaction_covered_torn_or_corruption
#[test]
fn replay_classifies_every_gap_compaction_covered_torn_or_corruption() {
    let entries: [(u64, &[u8]); 5] = [(1, b"r1"), (2, b"r2"), (3, b"r3"), (4, b"r4"), (5, b"r5")];
    let spliced = forged_log(&[entries[0], entries[1], entries[3], entries[4]]);

    assert_eq!(
        replay(&spliced, &CoveredRanges::none()),
        Err(ReplayRefused::MidChainGap {
            preceding: Some(SeqNo(2)),
            resumed: SeqNo(4),
            missing_first: SeqNo(3),
            missing_last: SeqNo(3),
        }),
        "an unexplained mid-chain gap is the named corruption case"
    );

    let covered = CoveredRanges::new([(SeqNo(3), SeqNo(3))]).expect("valid span");
    let replayed = replay(&spliced, &covered).expect("compaction-covered proceeds");
    assert_eq!(seqs(replayed.records()), [1, 2, 4, 5]);
    assert_eq!(replayed.truncation(), None);
    assert_eq!(replayed.next_seq().get(), 6);

    let head_gap = forged_log(&[entries[2], entries[3], entries[4]]);
    assert_eq!(
        replay(&head_gap, &CoveredRanges::none()),
        Err(ReplayRefused::MidChainGap {
            preceding: None,
            resumed: SeqNo(3),
            missing_first: SeqNo(1),
            missing_last: SeqNo(2),
        }),
        "a head gap is classified the same way"
    );
    let head_covered = CoveredRanges::new([(SeqNo(1), SeqNo(2))]).expect("valid span");
    let replayed = replay(&head_gap, &head_covered).expect("covered head gap proceeds");
    assert_eq!(seqs(replayed.records()), [3, 4, 5]);
    assert_eq!(replayed.next_seq().get(), 6);

    let wide_gap = forged_log(&[entries[0], entries[4]]);
    let partial = CoveredRanges::new([(SeqNo(2), SeqNo(3))]).expect("valid span");
    assert_eq!(
        replay(&wide_gap, &partial),
        Err(ReplayRefused::MidChainGap {
            preceding: Some(SeqNo(1)),
            resumed: SeqNo(5),
            missing_first: SeqNo(2),
            missing_last: SeqNo(4),
        }),
        "a partially covered gap still refuses — coverage is all or nothing"
    );
    let pieced = CoveredRanges::new([(SeqNo(2), SeqNo(2)), (SeqNo(3), SeqNo(4))]).expect("spans");
    let replayed = replay(&wide_gap, &pieced).expect("adjacent spans merge into full coverage");
    assert_eq!(seqs(replayed.records()), [1, 5]);

    let mut torn_and_gapped = spliced.clone();
    torn_and_gapped.truncate(spliced.len() - 5);
    let replayed = replay(&torn_and_gapped, &covered).expect("tail rule composes with coverage");
    assert_eq!(seqs(replayed.records()), [1, 2, 4]);
    assert_eq!(
        replayed.truncation(),
        Some(TornTail {
            valid_len: spliced.len() - 18,
            dropped_len: 13,
        }),
        "the torn tail truncates while the covered gap proceeds"
    );

    assert_eq!(
        CoveredRanges::new([(SeqNo(5), SeqNo(3))]),
        Err(InvalidSpan {
            first: SeqNo(5),
            last: SeqNo(3),
        }),
        "a backwards span is a typed refusal, not an empty range"
    );
}

// Requirements: JRN-002
//   The durability rule as a typed boundary: a WriteClearance for a
//   record exists only behind the durable watermark — an appended but
//   uncommitted record is a typed NotYetDurable refusal naming the
//   record and the watermark, commit offers the seam exactly the
//   not-yet-durable byte suffix and nothing else (asserted against the
//   fake's captured bytes), a seam refusal leaves the watermark and
//   the pending suffix untouched for re-offer, an already-durable
//   journal commits without calling the seam again, and the
//   DurableThrough receipt and the clearance are constructible only
//   through commit — so the storage-writing code the helper packages
//   build can demand proof of prior journal durability instead of a
//   comment, over an injected seam whose platform truth is their
//   acceptance obligation, not this test's claim.
// Evidence: storage_write_clearance_requires_prior_durability
#[test]
fn storage_write_clearance_requires_prior_durability() {
    let mut journal = Journal::new();
    let mut seam = FakeSeam::acknowledging();
    assert_eq!(
        journal.commit(&mut seam).expect("empty commit succeeds"),
        None,
        "a journal with no records has no watermark to receipt"
    );
    assert!(seam.offered.is_empty(), "nothing pending, seam not called");

    let first = journal.append(b"one").expect("bounded").seq();
    assert_eq!(
        journal.clearance(first),
        Err(crate::NotYetDurable {
            record: first,
            durable_through: None,
        }),
        "an appended record is not storage-write-eligible before commit"
    );

    let receipt = journal
        .commit(&mut seam)
        .expect("seam acknowledges")
        .expect("records exist");
    assert_eq!(receipt.through(), first);
    assert_eq!(seam.offered, [journal.bytes().to_vec()]);
    assert_eq!(
        journal.clearance(first).expect("durable now").record(),
        first
    );

    let second = journal.append(b"two").expect("bounded").seq();
    let third = journal.append(b"three").expect("bounded").seq();
    let durable_len = journal.durable_len();
    assert_eq!(
        journal.clearance(third),
        Err(crate::NotYetDurable {
            record: third,
            durable_through: Some(first),
        })
    );
    assert!(
        journal.clearance(first).is_ok(),
        "earlier durability is not revoked by later appends"
    );

    let mut refusing = FakeSeam::refusing();
    assert_eq!(
        journal.commit(&mut refusing),
        Err(DurabilityRefused {
            reason: "injected refusal".to_owned(),
        })
    );
    assert_eq!(
        refusing.offered,
        [journal.bytes()[durable_len..].to_vec()],
        "the refused seam was offered exactly the pending suffix"
    );
    assert_eq!(
        journal.durable_through(),
        Some(first),
        "no advance on refusal"
    );
    assert_eq!(journal.durable_len(), durable_len, "no advance on refusal");
    assert!(journal.clearance(second).is_err());

    let mut second_seam = FakeSeam::acknowledging();
    let receipt = journal
        .commit(&mut second_seam)
        .expect("seam acknowledges")
        .expect("records exist");
    assert_eq!(receipt.through(), third);
    assert_eq!(
        second_seam.offered,
        [journal.bytes()[durable_len..].to_vec()],
        "the retried commit re-offers the same suffix, nothing more"
    );
    assert!(journal.clearance(second).is_ok());
    assert!(journal.clearance(third).is_ok());

    let mut idle_seam = FakeSeam::acknowledging();
    assert_eq!(
        journal
            .commit(&mut idle_seam)
            .expect("nothing pending")
            .expect("records exist")
            .through(),
        third
    );
    assert!(
        idle_seam.offered.is_empty(),
        "nothing pending, seam not called"
    );

    assert_eq!(
        journal.clearance(SeqNo(4)),
        Err(crate::NotYetDurable {
            record: SeqNo(4),
            durable_through: Some(third),
        }),
        "a never-appended record has no clearance"
    );
}

// Requirements: JRN-003
//   Replay is idempotent and derives solely from the journal's bytes:
//   the same bytes and covered ranges replay to equal results, replay
//   of the truncated valid prefix reproduces the same records with no
//   further truncation (recovery is a fixpoint, so replaying a
//   recovered journal never truncates again), and a journal recovered
//   from a torn log continues the sequence exactly where the surviving
//   records end, treating the surviving bytes as its durable baseline
//   — nothing is derived from writer memory, and recovering the
//   recovered log reproduces the same records. The fresh-re-discovery
//   half of JRN-003's rule is an execution-layer input this crate does
//   not model; the roll-forward edge tests it by name in increment 5,
//   as the assignment records.
// Evidence: replay_is_idempotent_and_recovery_reaches_a_fixpoint
#[test]
fn replay_is_idempotent_and_recovery_reaches_a_fixpoint() {
    let journal = journal_with(&PAYLOADS);
    let ends = frame_ends(&PAYLOADS);
    let torn = &journal.bytes()[..ends[2] + 7];

    let first_pass = replay(torn, &CoveredRanges::none()).expect("torn tail truncates");
    let second_pass = replay(torn, &CoveredRanges::none()).expect("torn tail truncates");
    assert_eq!(
        first_pass, second_pass,
        "replay is a pure function of its inputs"
    );
    assert_eq!(seqs(first_pass.records()), [1, 2]);
    let truncation = first_pass.truncation().expect("tail was torn");
    assert_eq!(truncation.valid_len, ends[2]);

    let prefix = &torn[..truncation.valid_len];
    let fixpoint = replay(prefix, &CoveredRanges::none()).expect("clean prefix");
    assert_eq!(fixpoint.records(), first_pass.records());
    assert_eq!(fixpoint.truncation(), None, "recovery is a fixpoint");

    let (mut recovered, replayed) =
        Journal::recover(torn, &CoveredRanges::none()).expect("recoverable");
    assert_eq!(replayed, first_pass);
    assert_eq!(
        recovered.bytes(),
        prefix,
        "the torn tail is dropped from the log"
    );
    assert_eq!(
        recovered.next_seq(),
        SeqNo(3),
        "the sequence continues, never resets"
    );
    assert_eq!(
        recovered.durable_through(),
        Some(SeqNo(2)),
        "surviving bytes are the recovered journal's durable baseline"
    );

    let appended = recovered.append(b"third-again").expect("bounded");
    assert_eq!(appended.seq(), SeqNo(3));
    assert!(
        recovered.bytes().starts_with(prefix),
        "append-only holds across recovery"
    );
    let after = replay(recovered.bytes(), &CoveredRanges::none()).expect("intact");
    assert_eq!(seqs(after.records()), [1, 2, 3]);
    assert_eq!(after.records()[2].payload(), b"third-again");

    let (again, replayed_again) =
        Journal::recover(recovered.bytes(), &CoveredRanges::none()).expect("recoverable");
    assert_eq!(replayed_again.records(), after.records());
    assert_eq!(again.bytes(), recovered.bytes());
    assert_eq!(
        again.next_seq(),
        SeqNo(4),
        "recovering the recovered changes nothing"
    );
}
