//! Retention and compaction under ADR-0029's liveness rule
//! (increment 4): the liveness-scoped exemption with its linkage
//! closure, the per-apply budget with fail-closed exhaustion,
//! compaction records legitimizing reclaimed ranges, and monotonicity
//! across compaction — imported obligations 9, 10, 11's
//! derivation half, 12, and 13's fixture.
//!
//! The decided shape, held structurally:
//!
//! - **Retention MAY reclaim only records of terminal applies**, and
//!   the exemption closes over ADR-0027's linkage graph: a terminal
//!   apply whose disposal chain ends at a plan that is not itself
//!   terminal — a live recovery, or one named but not yet started,
//!   the conservative reading — keeps every record pinned, the whole
//!   chain included. Once the chain terminates, all of it ages into
//!   ordinary history.
//! - **No code path reclaims a live record.** [`compact`] is the one
//!   reclamation entry point, and it computes the reclaimable set
//!   itself from the journal's own records; no API on this module
//!   accepts a caller-supplied sequence range to delete.
//! - **Compaction records are journal infrastructure and are never
//!   reclaimed**: reclaiming one would orphan the gap it legitimizes
//!   and turn legal history removal back into corruption on the next
//!   replay.
//! - **The budget fails closed toward the writer, never the
//!   recoverer**: [`BudgetExhausted`] resolves only to an existing
//!   Section 8 edge's record ([`BudgetExhausted::journaled_failure`]),
//!   and nothing in the exhaustion path can reach a reclamation.
//! - **Sequence numbers are never reused or reset**: a compacted
//!   journal keeps every retained frame's sequence number and appends
//!   its compaction records at the continuing position.
//!
//! Rotation, in this pure crate, is recover-and-continue — increment
//! 2's fixpoint property — and the monotonicity obligation is proven
//! here across compaction and continued appends; a segment-file
//! architecture is ADR-0029's named revisit condition, not this
//! module's surface.

use std::collections::BTreeMap;

use crate::records::{
    CompactionAuthority, CompactionRecord, DecodeRefused, PlanHashRef, Record, TransitionRecord,
};
use crate::{
    CoveredRanges, Journal, MIN_FRAME_LEN, Replay, ReplayRefused, SeqNo, encode_frame, replay,
};
use partman_statemachine::Transition;

/// A journal replayed and fully decoded, with its gap classification
/// derived from its own durable compaction records — obligation 11's
/// derivation half: the covered ranges come from the journal, not from
/// a caller's claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedJournal {
    replay: Replay,
    records: Vec<(SeqNo, Record)>,
}

impl DecodedJournal {
    /// The frame-level replay.
    #[must_use]
    pub const fn replay(&self) -> &Replay {
        &self.replay
    }

    /// Every surviving record with its sequence number, in journal
    /// order.
    #[must_use]
    pub fn records(&self) -> &[(SeqNo, Record)] {
        &self.records
    }

    /// Per-apply journal spend in encoded frame bytes — the quantity
    /// ADR-0029's per-apply budget bounds. Compaction records carry no
    /// plan and are journal infrastructure, charged to no apply.
    #[must_use]
    pub fn spend(&self) -> Vec<(PlanHashRef, u64)> {
        let mut per_apply: BTreeMap<PlanHashRef, u64> = BTreeMap::new();
        for (frame, (_, record)) in self.replay.records().iter().zip(&self.records) {
            let Some(plan) = record_plan(record) else {
                continue;
            };
            let frame_len = (MIN_FRAME_LEN + frame.payload().len()) as u64;
            *per_apply.entry(plan).or_insert(0) += frame_len;
        }
        per_apply.into_iter().collect()
    }
}

/// Decode a journal end to end: replay the frames twice — first
/// tolerating gaps to collect the durable compaction records, then
/// classifying every gap against exactly the ranges those records
/// legitimize — and decode every surviving payload. An uncovered gap
/// refuses as the mid-chain corruption case, and no compaction record
/// can hide frame-level damage: checksums, sequence regressions, and
/// bound checks run before any gap is even classified.
///
/// # Errors
///
/// [`CompactedRefused`]: a frame-level refusal from either pass, or an
/// undecodable payload naming its sequence number.
///
/// # Panics
///
/// Never: the internal covered-range constructions are forward spans
/// by construction — the tolerant full span, and ranges a decoded
/// compaction record already validated — stated as panic bounds rather
/// than hidden in `unwrap`s.
pub fn decode_journal(bytes: &[u8]) -> Result<DecodedJournal, CompactedRefused> {
    let tolerant = CoveredRanges::new([(SeqNo::from_raw(1), SeqNo::from_raw(u64::MAX))])
        .expect("a forward span");
    let first_pass = replay(bytes, &tolerant).map_err(CompactedRefused::Frames)?;
    let mut covered_spans = Vec::new();
    for frame in first_pass.records() {
        let record =
            Record::decode(frame.payload()).map_err(|refusal| CompactedRefused::Undecodable {
                seq: frame.seq(),
                refusal,
            })?;
        if let Record::Compaction(compaction) = &record {
            covered_spans.push((compaction.first(), compaction.last()));
        }
    }
    let covered = CoveredRanges::new(covered_spans).expect("compaction ranges validated at decode");
    let second_pass = replay(bytes, &covered).map_err(CompactedRefused::Frames)?;
    let records = second_pass
        .records()
        .iter()
        .map(|frame| {
            Record::decode(frame.payload())
                .map(|record| (frame.seq(), record))
                .map_err(|refusal| CompactedRefused::Undecodable {
                    seq: frame.seq(),
                    refusal,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedJournal {
        replay: second_pass,
        records,
    })
}

/// The refusals of [`decode_journal`] and [`compact`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactedRefused {
    /// A frame-level refusal: damage, regression, or an uncovered gap.
    Frames(ReplayRefused),
    /// A surviving frame's payload is not a valid record.
    Undecodable {
        /// The frame's sequence number.
        seq: SeqNo,
        /// The record decoder's refusal.
        refusal: DecodeRefused,
    },
    /// Two terminal transition records for one plan — an append-only
    /// journal from a correct writer cannot produce this.
    TerminalTwice {
        /// The plan with two terminals.
        plan: PlanHashRef,
        /// The second terminal's sequence number.
        seq: SeqNo,
    },
}

/// One apply's standing in the ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ApplyEntry {
    seqs: Vec<SeqNo>,
    terminal: bool,
    disposal: Option<PlanHashRef>,
}

/// The apply ledger: every plan's records, liveness, and disposal
/// linkage, computed from decoded records alone (JRN-003's posture —
/// nothing here comes from writer memory).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyLedger {
    applies: BTreeMap<PlanHashRef, ApplyEntry>,
    infrastructure: Vec<SeqNo>,
}

/// Build the ledger from a decoded journal.
///
/// # Errors
///
/// [`CompactedRefused::TerminalTwice`] when one plan carries two
/// terminal transition records.
pub fn ledger(decoded: &DecodedJournal) -> Result<ApplyLedger, CompactedRefused> {
    let mut applies: BTreeMap<PlanHashRef, ApplyEntry> = BTreeMap::new();
    let mut infrastructure = Vec::new();
    for (seq, record) in decoded.records() {
        let Some(plan) = record_plan(record) else {
            infrastructure.push(*seq);
            continue;
        };
        let entry = applies.entry(plan).or_insert(ApplyEntry {
            seqs: Vec::new(),
            terminal: false,
            disposal: None,
        });
        entry.seqs.push(*seq);
        if let Record::Transition(transition) = record
            && transition.effect().is_some()
        {
            if entry.terminal {
                return Err(CompactedRefused::TerminalTwice { plan, seq: *seq });
            }
            entry.terminal = true;
            entry.disposal = transition.disposal().map(|linkage| linkage.recovery_plan());
        }
    }
    Ok(ApplyLedger {
        applies,
        infrastructure,
    })
}

impl ApplyLedger {
    /// Whether the plan is exempt from retention: non-terminal, or a
    /// terminal whose disposal chain has not wholly terminated — the
    /// liveness-scoped exemption closing over ADR-0027's linkage
    /// graph. A disposal-named plan with no journal presence counts as
    /// non-terminal (an unstarted recovery is not a terminated one),
    /// which fails closed toward retention.
    #[must_use]
    pub fn exempt(&self, plan: PlanHashRef) -> bool {
        let mut visited = Vec::new();
        let mut current = plan;
        loop {
            let Some(entry) = self.applies.get(&current) else {
                // Named but unstarted: not terminal, so the chain is
                // still live.
                return true;
            };
            if !entry.terminal {
                return true;
            }
            let Some(next) = entry.disposal else {
                return false;
            };
            if visited.contains(&next) {
                // A disposal cycle of terminals: everything in it has
                // terminated, so nothing live depends on it.
                return false;
            }
            visited.push(current);
            current = next;
        }
    }

    /// The sequence numbers retention may reclaim: every record of
    /// every terminal apply outside the exemption closure — and
    /// nothing else. Live applies' records never appear here by
    /// construction (only non-exempt terminal entries are consulted),
    /// and journal infrastructure is never offered.
    #[must_use]
    pub fn reclaimable(&self) -> Vec<SeqNo> {
        let mut seqs: Vec<SeqNo> = self
            .applies
            .iter()
            .filter(|(plan, entry)| entry.terminal && !self.exempt(**plan))
            .flat_map(|(_, entry)| entry.seqs.iter().copied())
            .collect();
        seqs.sort_unstable();
        seqs
    }
}

/// A completed compaction: the compacted journal and the durable
/// declarations that legitimize what it no longer carries.
#[derive(Clone, Debug)]
pub struct Compacted {
    /// The compacted journal: retained frames byte-identical, sequence
    /// numbers unchanged, compaction records appended at the
    /// continuing position.
    pub journal: Journal,
    /// The compaction records appended, one per contiguous reclaimed
    /// range.
    pub compaction_records: Vec<CompactionRecord>,
}

/// Run one retention pass: decode the journal, build the ledger,
/// compute the reclaimable set under the liveness rule, and produce
/// the compacted journal with its compaction records appended. The
/// reclaimable set is computed here, from the journal alone — this is
/// the only reclamation entry point, and it has no parameter by which
/// a caller could name a record to delete.
///
/// A journal with nothing reclaimable comes back unchanged with no
/// compaction record appended. Writing the compacted bytes durably in
/// place of the old log is the storage owner's act, through its own
/// durability path.
///
/// # Errors
///
/// [`CompactedRefused`], exactly as [`decode_journal`] and [`ledger`]
/// refuse; nothing is reclaimed from a journal that does not decode
/// whole.
///
/// # Panics
///
/// Never: replayed frames are bounded, contiguous runs are forward by
/// construction, and a record this module built encodes — each stated
/// as a panic bound rather than hidden in an `unwrap`.
pub fn compact(journal: &Journal) -> Result<Compacted, CompactedRefused> {
    let decoded = decode_journal(journal.bytes())?;
    let ledger = ledger(&decoded)?;
    let reclaim = ledger.reclaimable();
    if reclaim.is_empty() {
        return Ok(Compacted {
            journal: journal.clone(),
            compaction_records: Vec::new(),
        });
    }

    let mut bytes = Vec::new();
    let mut last: Option<SeqNo> = None;
    for frame in decoded.replay().records() {
        if reclaim.binary_search(&frame.seq()).is_ok() {
            continue;
        }
        let len = u32::try_from(frame.payload().len()).expect("replayed frames are bounded");
        bytes.extend_from_slice(&encode_frame(frame.seq(), len, frame.payload()));
        last = Some(frame.seq());
    }
    let mut compacted = Journal::reassemble(bytes, last, decoded.replay().next_seq());

    let mut compaction_records = Vec::new();
    for (first, last) in contiguous_runs(&reclaim) {
        let record =
            CompactionRecord::new(first, last, CompactionAuthority::TerminalHistoryRetention)
                .expect("runs are forward by construction");
        let payload = Record::Compaction(record)
            .encode()
            .expect("a record this module built encodes");
        compacted.append(&payload).expect("bounded record payload");
        compaction_records.push(record);
    }
    Ok(Compacted {
        journal: compacted,
        compaction_records,
    })
}

/// The plan an apply-owned record belongs to; `None` for journal
/// infrastructure (compaction records).
fn record_plan(record: &Record) -> Option<PlanHashRef> {
    match record {
        Record::AuthorizationAct(act) => Some(act.plan()),
        Record::Transition(transition) => Some(transition.plan()),
        Record::Checkpoint(checkpoint) => Some(checkpoint.plan()),
        Record::Protection(protection) => Some(protection.plan()),
        Record::Compaction(_) => None,
    }
}

fn contiguous_runs(seqs: &[SeqNo]) -> Vec<(SeqNo, SeqNo)> {
    let mut runs = Vec::new();
    let mut iter = seqs.iter().copied();
    let Some(mut first) = iter.next() else {
        return runs;
    };
    let mut last = first;
    for seq in iter {
        if seq.get() == last.get() + 1 {
            last = seq;
        } else {
            runs.push((first, last));
            first = seq;
            last = seq;
        }
    }
    runs.push((first, last));
    runs
}

/// ADR-0029's fail-closed exhaustion: an apply whose journal spend
/// reached the budget. The only resolution this type offers is a
/// journaled failure through an existing Section 8 edge — it exposes
/// no reclamation, so exhaustion can stop the writer but can never
/// blind the recoverer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetExhausted {
    /// The apply that exhausted its budget.
    pub plan: PlanHashRef,
    /// Its journal spend in encoded frame bytes.
    pub spent: u64,
    /// The budget it reached.
    pub budget: u64,
}

impl BudgetExhausted {
    /// The journaled failure the exhaustion routes through: the
    /// published `Executing → RecoveryRequired` row
    /// ([`Transition::StepFailureOrInterruption`]) — an existing edge,
    /// exactly as ADR-0029 requires, never a new one and never a
    /// reclamation.
    ///
    /// # Panics
    ///
    /// Never: `StepFailureOrInterruption` is a published non-terminal
    /// row, so the constructor's terminal refusal is unreachable —
    /// stated as a panic bound rather than hidden in an `unwrap`.
    #[must_use]
    pub fn journaled_failure(&self) -> TransitionRecord {
        TransitionRecord::non_terminal(self.plan, Transition::StepFailureOrInterruption)
            .expect("StepFailureOrInterruption is a published non-terminal row")
    }
}

/// Check every apply's spend against ADR-0029's budget
/// ([`crate::records::PER_APPLY_JOURNAL_BUDGET_BYTES`]).
#[must_use]
pub fn over_budget(decoded: &DecodedJournal) -> Vec<BudgetExhausted> {
    over_budget_against(decoded, crate::records::PER_APPLY_JOURNAL_BUDGET_BYTES)
}

/// Check every apply's spend against a stated budget. The constant
/// above is the shipped bound; the parameter exists so the exhaustion
/// path is testable without a quarter-gigabyte fixture, and a
/// production caller has no reason to pass anything else.
#[must_use]
pub fn over_budget_against(decoded: &DecodedJournal, budget: u64) -> Vec<BudgetExhausted> {
    decoded
        .spend()
        .into_iter()
        .filter(|&(_, spent)| spent >= budget)
        .map(|(plan, spent)| BudgetExhausted {
            plan,
            spent,
            budget,
        })
        .collect()
}

#[cfg(test)]
mod tests;
