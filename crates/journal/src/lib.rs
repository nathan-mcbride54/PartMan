//! The WP-070 journal core (increment 2).
//!
//! JRN-001's frame mechanics as a pure library: an append-only byte log
//! of checksummed frames with strictly monotonic sequence numbers, torn-
//! tail detection with safe truncation, the JRN-002 durability rule as a
//! typed injected boundary ([`DurabilitySeam`]), idempotent replay
//! (JRN-003), and the three-way gap classification JRN-004 states —
//! compaction-covered proceeds, a torn tail truncates, anything else
//! refuses — imported by the assignment as obligation 11's core from
//! ADR-0029, with the mid-chain gap as the named corruption case
//! ([`ReplayRefused::MidChainGap`]).
//!
//! What this crate deliberately is not:
//!
//! - **No record semantics at this layer.** A frame payload is bytes.
//!   The JRN-006 record vocabulary — transition and checkpoint
//!   records, the authorization act, disposal linkage, protection and
//!   compaction records — is the [`records`] module (increment 3),
//!   layered strictly above the frames: the frame layer never
//!   interprets a payload.
//! - **No retention, no budget, no real compaction.** [`CoveredRanges`]
//!   is the classification's typed input; increment 4 derives it from
//!   durable compaction records and owns the liveness-scoped exemption,
//!   the per-apply budget, and monotonicity across compaction
//!   (ADR-0029).
//! - **No platform durability.** [`DurabilitySeam`] is JRN-002's rule as
//!   a type: an fsync-shaped boundary this crate calls and never
//!   implements. Real fsync truth is the helper packages' acceptance
//!   obligation, exactly as the assignment's boundary section records.
//! - **No storage writes.** [`WriteClearance`] exists so that the code
//!   which *does* write storage (the M3 helper packages) can demand a
//!   proof of prior journal durability instead of a comment; nothing
//!   here performs the write.
//!
//! The frame layout is fixed here, pinned byte-for-byte by test, and
//! documented in `schemas/journal/framing.md`; the versioned record
//! schema above it (JRN-006, MODEL-003) is `schemas/journal/records.md`
//! and the [`records`] module.

pub mod records;

/// The one-based sequence number a journal record carries. Sequence
/// numbers are strictly monotonic over the journal's whole life and are
/// never reused or reset (JRN-001, JRN-004); `0` is not a valid
/// sequence number and replay refuses it as corruption.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeqNo(u64);

impl SeqNo {
    /// The first sequence number a fresh journal assigns.
    pub const FIRST: SeqNo = SeqNo(1);

    /// The raw value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    const fn next(self) -> SeqNo {
        SeqNo(self.0 + 1)
    }

    /// Rebuild a sequence number a schema decoder read from a record.
    /// Crate-internal: callers outside this crate obtain sequence
    /// numbers only from appends and replay, so a `SeqNo` in public API
    /// always names a position a journal actually assigned or a record
    /// actually declared.
    pub(crate) const fn from_raw(raw: u64) -> SeqNo {
        SeqNo(raw)
    }
}

/// The frame-level payload bound: one mebibyte. A larger payload is
/// refused at append ([`AppendRefused::PayloadOverBound`]) and refused
/// at replay ([`ReplayRefused::FrameOverBound`]) — JRN-005's
/// boundedness at the layer this increment owns; record-class bounds
/// arrive with the JRN-006 vocabulary.
pub const MAX_PAYLOAD_LEN: usize = 1024 * 1024;

const SEQ_LEN: usize = 8;
const LEN_LEN: usize = 4;
const HEADER_LEN: usize = SEQ_LEN + LEN_LEN;
const CRC_LEN: usize = 4;
const MIN_FRAME_LEN: usize = HEADER_LEN + CRC_LEN;

/// CRC-32 (IEEE 802.3, polynomial `0xEDB88320`), the per-record
/// checksum JRN-001 requires, implemented in-crate so the shipped
/// dependency closure stays empty.
const CRC_TABLE: [u32; 256] = build_crc_table();

const fn build_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n: u32 = 0;
    while n < 256 {
        let mut crc = n;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 == 1 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[n as usize] = crc;
        n += 1;
    }
    table
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        let index = ((crc ^ u32::from(byte)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC_TABLE[index];
    }
    !crc
}

fn read_u64_le(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + SEQ_LEN].try_into().expect("eight bytes"))
}

fn read_u32_le(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + LEN_LEN].try_into().expect("four bytes"))
}

/// A receipt for one append. Holding one proves the record exists in
/// the journal's byte log — and proves nothing about durability: the
/// sequence number becomes storage-write-eligible only through
/// [`Journal::clearance`], after a [`DurabilitySeam`] acknowledgement
/// covers it (JRN-002).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Appended {
    seq: SeqNo,
}

impl Appended {
    /// The sequence number the journal assigned.
    #[must_use]
    pub const fn seq(self) -> SeqNo {
        self.seq
    }
}

/// The typed refusal [`Journal::append`] returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendRefused {
    /// The payload exceeds [`MAX_PAYLOAD_LEN`]; nothing was appended.
    PayloadOverBound {
        /// The offered payload's length.
        len: usize,
        /// The bound it exceeded.
        bound: usize,
    },
}

/// JRN-002's durability rule as the typed boundary this pure crate
/// injects instead of implementing: "journal records for a state
/// transition or checkpoint MUST be durable (fsync or platform
/// equivalent) before the corresponding storage write begins."
///
/// [`Journal::commit`] calls [`DurabilitySeam::make_durable`] with
/// exactly the byte suffix that is appended but not yet durable; an
/// `Ok` advances the journal's durable watermark, and only records at
/// or below that watermark can obtain a [`WriteClearance`]. A real
/// implementation persists the bytes and syncs them; this crate's
/// tests inject fakes, and asserting platform fsync truth is the
/// helper packages' acceptance work, said here so a pure test is never
/// read as a platform proof.
///
/// Honesty rule for implementers: after `make_durable` has failed
/// once, an implementation must keep refusing unless it can truly
/// re-establish durability — a platform whose sync failure poisons the
/// file cannot answer `Ok` on retry, and the journal trusts the
/// answer.
pub trait DurabilitySeam {
    /// Make `new_bytes` — the journal's not-yet-durable suffix, in
    /// journal order — durable, or refuse.
    ///
    /// # Errors
    ///
    /// [`DurabilityRefused`] when durability cannot be established;
    /// the journal's watermark then does not advance and the same
    /// suffix is re-offered on the next commit.
    fn make_durable(&mut self, new_bytes: &[u8]) -> Result<(), DurabilityRefused>;
}

/// A seam's refusal to establish durability. Carries the seam's own
/// stated reason; the journal adds nothing to it and treats any
/// refusal the same way — the watermark stays where it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurabilityRefused {
    /// The seam's stated reason.
    pub reason: String,
}

/// Proof that every record up to and including `through` is durable —
/// constructible only by [`Journal::commit`] after the seam
/// acknowledged the bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurableThrough {
    through: SeqNo,
}

impl DurableThrough {
    /// The highest durable sequence number.
    #[must_use]
    pub const fn through(self) -> SeqNo {
        self.through
    }
}

/// Proof that one record's durability precedes any storage write made
/// on its behalf — JRN-002's ordering as a type. Constructible only
/// through [`Journal::clearance`], which refuses while the record is
/// not covered by the durable watermark. The code that performs
/// storage writes (the helper packages, M3) demands this token; this
/// crate only mints it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteClearance {
    record: SeqNo,
}

impl WriteClearance {
    /// The record this clearance covers.
    #[must_use]
    pub const fn record(self) -> SeqNo {
        self.record
    }
}

/// The typed refusal [`Journal::clearance`] returns while the record's
/// durability has not been established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotYetDurable {
    /// The record whose clearance was requested.
    pub record: SeqNo,
    /// The current durable watermark, if any record is durable at all.
    pub durable_through: Option<SeqNo>,
}

/// One record as replay reconstructed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayedRecord {
    seq: SeqNo,
    payload: Vec<u8>,
}

impl ReplayedRecord {
    /// The record's sequence number.
    #[must_use]
    pub const fn seq(&self) -> SeqNo {
        self.seq
    }

    /// The record's payload bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// A detected torn tail: bytes past `valid_len` are an incomplete or
/// tail-damaged frame, safely truncatable under JRN-001's rule. The
/// journal core reports the truncation; physically shortening the
/// stored file is the storage owner's act, performed through its own
/// durability path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TornTail {
    /// The byte length of the valid prefix.
    pub valid_len: usize,
    /// How many trailing bytes the truncation drops.
    pub dropped_len: usize,
}

/// A completed replay: every surviving record in order, the torn-tail
/// truncation if one was detected, and the sequence number the journal
/// continues at. Replay is a pure function of its inputs (JRN-003):
/// the same bytes and the same covered ranges always produce the same
/// `Replay`, and replaying the valid prefix reproduces the same
/// records with no further truncation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    records: Vec<ReplayedRecord>,
    truncation: Option<TornTail>,
    next_seq: SeqNo,
}

impl Replay {
    /// The surviving records, in journal order.
    #[must_use]
    pub fn records(&self) -> &[ReplayedRecord] {
        &self.records
    }

    /// The torn-tail truncation, if one was detected.
    #[must_use]
    pub const fn truncation(&self) -> Option<TornTail> {
        self.truncation
    }

    /// The sequence number the recovered journal assigns next.
    #[must_use]
    pub const fn next_seq(&self) -> SeqNo {
        self.next_seq
    }
}

/// The corruption refusals replay returns. Every variant is a refusal
/// to proceed, never a truncation: JRN-001's safe truncation governs
/// the tail, and JRN-004's classification names everything interior
/// that a compaction record does not cover as corruption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayRefused {
    /// A complete interior frame failed its checksum. The claimed
    /// sequence number is reported as claimed — its own bytes are
    /// inside the failed checksum and may themselves be damaged.
    InteriorChecksumMismatch {
        /// The frame's claimed sequence number, unverified.
        seq_claimed: u64,
        /// The frame's byte offset in the replayed log.
        frame_start: usize,
    },
    /// A sequence gap no covered range explains — the named corruption
    /// case (ADR-0029, imported obligation 11): the mid-chain gap.
    MidChainGap {
        /// The last good sequence number before the gap; `None` when
        /// the gap is at the journal's head.
        preceding: Option<SeqNo>,
        /// The sequence number the log resumes at after the gap.
        resumed: SeqNo,
        /// The first missing sequence number.
        missing_first: SeqNo,
        /// The last missing sequence number.
        missing_last: SeqNo,
    },
    /// A frame's sequence number does not advance — a duplicate, a
    /// regression, or the invalid `0`. An append-only journal cannot
    /// produce this; damage did.
    SequenceRegression {
        /// The last good sequence number, if any record preceded.
        previous: Option<SeqNo>,
        /// The raw sequence value found, reported unverified.
        found: u64,
        /// The frame's byte offset in the replayed log.
        frame_start: usize,
    },
    /// A complete frame claims a payload over [`MAX_PAYLOAD_LEN`].
    /// This writer never produces one, so a complete over-bound frame
    /// is corruption; an *incomplete* claim at the tail is a torn tail
    /// and truncates instead.
    FrameOverBound {
        /// The claimed payload length.
        claimed_len: u64,
        /// The frame's byte offset in the replayed log.
        frame_start: usize,
    },
}

/// The sequence ranges legitimately absent from a replayed log —
/// JRN-004's "compaction-covered is policy" as the classification's
/// typed input. This increment constructs it directly (tests and
/// callers state the ranges); increment 4 derives it from durable
/// compaction records, which are the only production authority for
/// one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoveredRanges {
    /// Normalized: sorted, non-adjacent, non-overlapping inclusive
    /// spans of raw sequence values.
    spans: Vec<(u64, u64)>,
}

/// The typed refusal [`CoveredRanges::new`] returns for a span whose
/// bounds are not an inclusive range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSpan {
    /// The span's stated first sequence number.
    pub first: SeqNo,
    /// The span's stated last sequence number, below `first`.
    pub last: SeqNo,
}

impl CoveredRanges {
    /// No covered ranges: every gap is unexplained.
    #[must_use]
    pub const fn none() -> Self {
        CoveredRanges { spans: Vec::new() }
    }

    /// Build from inclusive `(first, last)` spans, in any order;
    /// overlapping and adjacent spans merge.
    ///
    /// # Errors
    ///
    /// [`InvalidSpan`] for any span with `last` below `first`.
    pub fn new(spans: impl IntoIterator<Item = (SeqNo, SeqNo)>) -> Result<Self, InvalidSpan> {
        let mut raw: Vec<(u64, u64)> = Vec::new();
        for (first, last) in spans {
            if last < first {
                return Err(InvalidSpan { first, last });
            }
            raw.push((first.get(), last.get()));
        }
        raw.sort_unstable();
        let mut merged: Vec<(u64, u64)> = Vec::new();
        for (first, last) in raw {
            match merged.last_mut() {
                Some((_, prior_last)) if first <= prior_last.saturating_add(1) => {
                    *prior_last = (*prior_last).max(last);
                }
                _ => merged.push((first, last)),
            }
        }
        Ok(CoveredRanges { spans: merged })
    }

    /// Whether every sequence value in `first..=last` lies inside one
    /// covered span.
    fn covers(&self, first: u64, last: u64) -> bool {
        self.spans
            .iter()
            .any(|&(span_first, span_last)| span_first <= first && last <= span_last)
    }
}

/// Replay a journal's byte log: verify every frame's checksum, enforce
/// strictly monotonic sequencing, truncate a torn tail (JRN-001), and
/// classify every sequence gap three ways (JRN-004) — covered by
/// `covered` proceeds, a tail defect truncates, anything else refuses
/// as corruption.
///
/// The tail rule, precisely: a defect *reaching the end of the bytes*
/// — an incomplete frame, or a complete final frame whose checksum
/// fails — is a torn tail, because an interrupted append produces
/// exactly that shape. A checksum failure with further bytes behind
/// it, a complete over-bound frame, a non-advancing sequence number,
/// or an uncovered gap can not be produced by an interrupted append,
/// so each refuses instead of truncating.
///
/// # Errors
///
/// [`ReplayRefused`], naming the corruption and where it sits. A
/// refusal means the log needs repair authority this crate does not
/// have; nothing is truncated on a refusal.
pub fn replay(bytes: &[u8], covered: &CoveredRanges) -> Result<Replay, ReplayRefused> {
    let mut records: Vec<ReplayedRecord> = Vec::new();
    let mut previous: Option<SeqNo> = None;
    let mut offset = 0usize;

    let truncation = loop {
        let remaining = bytes.len() - offset;
        if remaining == 0 {
            break None;
        }
        if remaining < MIN_FRAME_LEN {
            break Some(torn(offset, bytes.len()));
        }
        let seq_raw = read_u64_le(bytes, offset);
        let claimed = read_u32_le(bytes, offset + SEQ_LEN);
        let frame_len = (MIN_FRAME_LEN as u64) + u64::from(claimed);
        let complete = frame_len <= remaining as u64;
        if claimed as usize > MAX_PAYLOAD_LEN {
            if complete {
                return Err(ReplayRefused::FrameOverBound {
                    claimed_len: u64::from(claimed),
                    frame_start: offset,
                });
            }
            break Some(torn(offset, bytes.len()));
        }
        if !complete {
            break Some(torn(offset, bytes.len()));
        }
        let payload_len = claimed as usize;
        let payload_end = offset + HEADER_LEN + payload_len;
        let computed = crc32(&bytes[offset..payload_end]);
        let stored = read_u32_le(bytes, payload_end);
        if computed != stored {
            if payload_end + CRC_LEN == bytes.len() {
                break Some(torn(offset, bytes.len()));
            }
            return Err(ReplayRefused::InteriorChecksumMismatch {
                seq_claimed: seq_raw,
                frame_start: offset,
            });
        }
        let expected = previous.map_or(SeqNo::FIRST, SeqNo::next);
        if seq_raw < expected.get() {
            return Err(ReplayRefused::SequenceRegression {
                previous,
                found: seq_raw,
                frame_start: offset,
            });
        }
        if seq_raw > expected.get() && !covered.covers(expected.get(), seq_raw - 1) {
            return Err(ReplayRefused::MidChainGap {
                preceding: previous,
                resumed: SeqNo(seq_raw),
                missing_first: expected,
                missing_last: SeqNo(seq_raw - 1),
            });
        }
        let seq = SeqNo(seq_raw);
        records.push(ReplayedRecord {
            seq,
            payload: bytes[offset + HEADER_LEN..payload_end].to_vec(),
        });
        previous = Some(seq);
        offset = payload_end + CRC_LEN;
    };

    let next_seq = previous.map_or(SeqNo::FIRST, SeqNo::next);
    Ok(Replay {
        records,
        truncation,
        next_seq,
    })
}

const fn torn(valid_len: usize, total_len: usize) -> TornTail {
    TornTail {
        valid_len,
        dropped_len: total_len - valid_len,
    }
}

/// The append-only journal writer. It owns the encoded byte log, hands
/// out sequence numbers strictly in order, tracks the durable
/// watermark the [`DurabilitySeam`] advances, and mints
/// [`WriteClearance`] only behind that watermark. Nothing removes or
/// rewrites an existing frame: the byte log only ever grows, and every
/// earlier snapshot of [`Journal::bytes`] is a byte prefix of every
/// later one.
#[derive(Clone, Debug)]
pub struct Journal {
    bytes: Vec<u8>,
    durable_len: usize,
    durable_through: Option<SeqNo>,
    last_appended: Option<SeqNo>,
    next_seq: SeqNo,
}

impl Default for Journal {
    fn default() -> Self {
        Journal::new()
    }
}

impl Journal {
    /// A fresh, empty journal; the first append is [`SeqNo::FIRST`].
    #[must_use]
    pub const fn new() -> Self {
        Journal {
            bytes: Vec::new(),
            durable_len: 0,
            durable_through: None,
            last_appended: None,
            next_seq: SeqNo::FIRST,
        }
    }

    /// Recover a journal from stored bytes: replay them (torn tail
    /// truncated, gaps classified against `covered`), then continue
    /// appending after the last surviving record. The surviving bytes
    /// are the recovered journal's durable baseline — they were read
    /// back from storage; what JRN-002 gates on is re-established for
    /// them by their survival, and only *new* appends need the seam
    /// again. The torn tail's physical removal from storage is the
    /// storage owner's act, reported via the returned
    /// [`Replay::truncation`].
    ///
    /// # Errors
    ///
    /// [`ReplayRefused`] exactly as [`replay`] refuses; no journal is
    /// constructed over a corrupt log.
    pub fn recover(
        bytes: &[u8],
        covered: &CoveredRanges,
    ) -> Result<(Journal, Replay), ReplayRefused> {
        let replayed = replay(bytes, covered)?;
        let valid_len = replayed
            .truncation
            .map_or(bytes.len(), |tail| tail.valid_len);
        let last = replayed.records.last().map(ReplayedRecord::seq);
        let journal = Journal {
            bytes: bytes[..valid_len].to_vec(),
            durable_len: valid_len,
            durable_through: last,
            last_appended: last,
            next_seq: replayed.next_seq,
        };
        Ok((journal, replayed))
    }

    /// Append one record. The frame is encoded into the byte log and
    /// assigned the next sequence number; it is **not** durable until
    /// a [`Journal::commit`] covers it, and [`Journal::clearance`]
    /// refuses for it until then.
    ///
    /// # Errors
    ///
    /// [`AppendRefused::PayloadOverBound`] for a payload over
    /// [`MAX_PAYLOAD_LEN`]; the journal is unchanged.
    pub fn append(&mut self, payload: &[u8]) -> Result<Appended, AppendRefused> {
        let len = match u32::try_from(payload.len()) {
            Ok(len) if payload.len() <= MAX_PAYLOAD_LEN => len,
            _ => {
                return Err(AppendRefused::PayloadOverBound {
                    len: payload.len(),
                    bound: MAX_PAYLOAD_LEN,
                });
            }
        };
        let seq = self.next_seq;
        let frame_start = self.bytes.len();
        self.bytes.extend_from_slice(&seq.get().to_le_bytes());
        self.bytes.extend_from_slice(&len.to_le_bytes());
        self.bytes.extend_from_slice(payload);
        let crc = crc32(&self.bytes[frame_start..]);
        self.bytes.extend_from_slice(&crc.to_le_bytes());
        self.last_appended = Some(seq);
        self.next_seq = seq.next();
        Ok(Appended { seq })
    }

    /// Make every appended record durable through the seam (JRN-002).
    /// The seam receives exactly the not-yet-durable byte suffix; on
    /// its `Ok` the watermark advances to cover everything appended so
    /// far. With nothing pending the seam is not called. Returns the
    /// watermark receipt, or `None` for a journal that has no records
    /// at all.
    ///
    /// # Errors
    ///
    /// The seam's [`DurabilityRefused`], verbatim. The watermark does
    /// not advance, the pending suffix stays pending, and the same
    /// bytes are re-offered on the next commit.
    pub fn commit(
        &mut self,
        seam: &mut dyn DurabilitySeam,
    ) -> Result<Option<DurableThrough>, DurabilityRefused> {
        if self.durable_len < self.bytes.len() {
            seam.make_durable(&self.bytes[self.durable_len..])?;
            self.durable_len = self.bytes.len();
            self.durable_through = self.last_appended;
        }
        Ok(self
            .durable_through
            .map(|through| DurableThrough { through }))
    }

    /// Mint the JRN-002 ordering proof for one record: the clearance
    /// the storage-writing code must hold before its write begins.
    /// Refuses while the record is not covered by the durable
    /// watermark.
    ///
    /// # Errors
    ///
    /// [`NotYetDurable`], naming the record and the current watermark,
    /// when no committed durability covers the record yet.
    pub fn clearance(&self, record: SeqNo) -> Result<WriteClearance, NotYetDurable> {
        match self.durable_through {
            Some(through) if record <= through => Ok(WriteClearance { record }),
            durable_through => Err(NotYetDurable {
                record,
                durable_through,
            }),
        }
    }

    /// The full encoded byte log, durable prefix and pending suffix
    /// alike. What a crash preserves of the pending suffix is the
    /// storage's affair; replaying any prefix cut of these bytes is
    /// safe by JRN-001's tail rule.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// How many leading bytes the durable watermark covers.
    #[must_use]
    pub const fn durable_len(&self) -> usize {
        self.durable_len
    }

    /// The highest durable sequence number, if any.
    #[must_use]
    pub const fn durable_through(&self) -> Option<SeqNo> {
        self.durable_through
    }

    /// The sequence number the next append will be assigned.
    #[must_use]
    pub const fn next_seq(&self) -> SeqNo {
        self.next_seq
    }
}

#[cfg(test)]
mod tests;
