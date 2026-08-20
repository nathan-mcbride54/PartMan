//! The journal-borne apply, to the authorization boundary (increment 4a;
//! the shape round `docs/reviews/WP-L110_INCREMENT_4_ROUND_2026-08-20.md`
//! §9).
//!
//! **Where 4a ends, and why.** Everything here stops at
//! `AwaitingAuthorization`. `Executing` publishes no exit a refusal could
//! honestly take, `Protecting`'s PART-013 store has no owning assignment,
//! EXE-001's mechanism is route-gated, and CAP-003's `supported` is
//! unconstructible while the CAP-006 store is empty — so everything from
//! `AuthorizationGranted` onward is increment 4b's, and this module
//! cannot represent an apply past the boundary: no function here appends
//! `AuthorizationGranted` or anything beyond it.
//!
//! **What this module holds:**
//!
//! - **The journal, made real.** [`ApplyCore`] owns two
//!   [`partman_journal::Journal`]s — the Section 8 journal and the
//!   validation store's own append-only log — both recovered from bytes
//!   (torn tails truncated by JRN-001's rule) and both made durable
//!   through injected [`DurabilitySeam`]s before any answer leaves. The
//!   first *real* seam is the Linux module's file-and-fsync
//!   implementation; every seam here is injected, exactly the crate
//!   boundary WP-070 recorded.
//! - **The backward-clock bound** ([`clock_bound`]): a reading below the
//!   journal's high-water instant refuses the operation. The exposure
//!   this closes is the one `clock.rs` names — a clock stepped backwards
//!   between a plan's validation and its presentation — and it works
//!   because `ValidatorPasses` is journaled *at validation* (WP-070
//!   schema v2, transition-only as decided): the mark is populated
//!   before any act exists.
//! - **The durable `ValidationRecord`** ([`RecordedValidation`]): the
//!   validation store is a sibling append-only log in ADR-0030's
//!   sibling-store shape (never inside the journal — the record
//!   vocabulary there is closed, and plan bodies are bulk the journal's
//!   budget must not carry). Consumption is an appended entry, never a
//!   flipped bit, so a consumed validation refuses re-presentation
//!   **across a restart** — the store, not the flag. SEC-002's admission
//!   arms ([`crate::validate::admit_presented_plan`]) get their
//!   production caller here.
//! - **The two-phase wire's decisions** ([`apply_plan`]): phase one
//!   consumes the validation, journals `ApplySubmitted` and answers
//!   `awaiting-authorization`; phase two refuses exactly where increment
//!   3 already refuses — the interactive ceremony's own arm, verbatim —
//!   and a plan whose window closed while awaiting terminates on the
//!   published `DeclinedOrExpired → Cancelled` edge, `NoWrites` as the
//!   row constrains. Even a completed ceremony reaches no grant on this
//!   build: the grant edge is 4b's, and the refusal says so.
//! - **CONC-003 on its published edge**: a stale presentation of a
//!   `Validated` plan journals `EditOrInvalidation` (Validated → Draft)
//!   rather than merely refusing.
//! - **CONC-004's predicate** ([`transitional_now`]): whether any
//!   journaled lifecycle stands past the authorization boundary and
//!   before its terminal — the flag `capture` must carry instead of the
//!   hard-coded `false` increment 2 shipped.
//!
//! **Ordering rules, stated because a crash lands between any two
//! writes:** at validation, the journal's `ValidatorPasses` is committed
//! before the store's `recorded` entry, and both before the answer — a
//! crash between leaves a `Validated` chain with no record, which
//! re-validation heals. At phase one, the store's `consumed` entry is
//! committed **before** `ApplySubmitted` — the fail-closed order: a
//! crash between costs the client a re-validation, where the reverse
//! order would let one validation submit twice.
//!
//! **The tier in phase two** is read from the helper's own store —
//! written at validation from the helper-recomputed severity and flags,
//! root-owned on disk — because the `AdmittedPlan` was consumed at phase
//! one. The store's two tier words are parsed by a private function
//! here; this is not a client-namable tier (CAP-007's concern): no wire
//! field feeds it, and the journal crate's deliberate no-parse rule
//! stands untouched.

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Hash, Value};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_journal::records::{
    AuthorizationTier, PlanHashRef, Record, RecordedInstant, TransitionRecord,
};
use partman_journal::retention::{DecodedJournal, decode_journal};
use partman_journal::{CoveredRanges, DurabilityRefused, DurabilitySeam, Journal, lifecycle};
use partman_statemachine::{Effect, State, Transition};

use crate::authorize::Ceremony;
use crate::validate::{AdmissionRefusal, ValidationRecord, admit_presented_plan};

/// The validation store's schema identity (MODEL-003). Helper-owned,
/// documented in `schemas/helper/operations.md`; the journal's own
/// record vocabulary stays closed and carries none of this.
pub const VALIDATION_SCHEMA: &str = "partman.helper.validation";
/// The validation store's schema version.
pub const VALIDATION_SCHEMA_VERSION: u64 = 1;

/// One validation as the store holds it durably: everything phase one
/// and phase two need that the wire must never re-supply — the body the
/// admission arms check (a client presents a *hash*; the bytes come from
/// here), the uid it answered, the helper-computed tier, the window's
/// end, and whether its one submission is spent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedValidation {
    /// The validated plan's body hash bytes.
    pub plan_hash: [u8; 32],
    /// The RPC-001-authenticated user the validation answered.
    pub validated_for_uid: u32,
    /// The helper-computed tier, written at validation (HLP-003).
    pub tier: AuthorizationTier,
    /// PLAN-007's window end, as the body carries it.
    pub not_after: u64,
    /// The plan body's canonical bytes — what admission re-checks.
    pub body: Vec<u8>,
    /// Whether the record's one submission is spent (an appended
    /// `consumed` entry, not a flipped bit).
    pub consumed: bool,
}

/// Why the core could not be recovered from stored bytes.
#[derive(Debug)]
pub enum RecoverRefused {
    /// The Section 8 journal's bytes refuse replay or decode.
    Journal(String),
    /// The validation store's bytes refuse replay or decode.
    Store(String),
}

/// Why a durable write refused. One shape for both files: the seam's
/// stated reason travels, and the operation is refused rather than
/// served un-journaled.
#[derive(Debug)]
pub struct WriteRefused {
    /// Which log refused: `journal` or `validation-store`.
    pub log: &'static str,
    /// The seam's reason.
    pub reason: String,
}

/// The 4a state: the Section 8 journal, the validation store's log, and
/// the store's replayed view. Pure over injected seams; the Linux module
/// wires the real files.
pub struct ApplyCore {
    journal: Journal,
    validations: Journal,
    records: BTreeMap<[u8; 32], RecordedValidation>,
}

impl ApplyCore {
    /// A fresh core: empty journal, empty store.
    #[must_use]
    pub fn new() -> Self {
        ApplyCore {
            journal: Journal::new(),
            validations: Journal::new(),
            records: BTreeMap::new(),
        }
    }

    /// Recover both logs from stored bytes: torn tails truncate
    /// (JRN-001), interior damage refuses, and the store's view is
    /// replayed entry by entry — a restart recomputes identically from
    /// bytes, holding nothing the logs do not (the obligation-8 posture
    /// at this layer).
    ///
    /// # Errors
    ///
    /// [`RecoverRefused`], naming the log that refused.
    pub fn recover(journal_bytes: &[u8], validation_bytes: &[u8]) -> Result<Self, RecoverRefused> {
        let (journal, _) = Journal::recover(journal_bytes, &CoveredRanges::none())
            .map_err(|refusal| RecoverRefused::Journal(format!("{refusal:?}")))?;
        decode_journal(journal.bytes())
            .map_err(|refusal| RecoverRefused::Journal(format!("{refusal:?}")))?;
        let (validations, replayed) = Journal::recover(validation_bytes, &CoveredRanges::none())
            .map_err(|refusal| RecoverRefused::Store(format!("{refusal:?}")))?;
        let mut records = BTreeMap::new();
        for frame in replayed.records() {
            apply_store_entry(&mut records, frame.payload()).map_err(RecoverRefused::Store)?;
        }
        Ok(ApplyCore {
            journal,
            validations,
            records,
        })
    }

    /// The Section 8 journal's full byte log (what the storage owner
    /// persists; the seam has already made the durable prefix durable).
    #[must_use]
    pub fn journal_bytes(&self) -> &[u8] {
        self.journal.bytes()
    }

    /// The validation store's full byte log.
    #[must_use]
    pub fn validation_bytes(&self) -> &[u8] {
        self.validations.bytes()
    }

    /// The journal, decoded. Refusal is reported, never repaired.
    ///
    /// # Errors
    ///
    /// The retention module's refusal, rendered.
    pub fn decoded(&self) -> Result<DecodedJournal, String> {
        decode_journal(self.journal.bytes()).map_err(|refusal| format!("{refusal:?}"))
    }

    /// One stored validation, by hash.
    #[must_use]
    pub fn validation(&self, plan_hash: &[u8; 32]) -> Option<&RecordedValidation> {
        self.records.get(plan_hash)
    }

    fn append_transition(
        &mut self,
        seam: &mut dyn DurabilitySeam,
        record: TransitionRecord,
    ) -> Result<(), WriteRefused> {
        let payload = Record::Transition(record)
            .encode()
            .map_err(|error| WriteRefused {
                log: "journal",
                reason: format!("{error:?}"),
            })?;
        self.journal
            .append(&payload)
            .map_err(|error| WriteRefused {
                log: "journal",
                reason: format!("{error:?}"),
            })?;
        self.journal
            .commit(seam)
            .map_err(|DurabilityRefused { reason }| WriteRefused {
                log: "journal",
                reason,
            })?;
        Ok(())
    }

    fn append_store_entry(
        &mut self,
        seam: &mut dyn DurabilitySeam,
        entry: &Value,
    ) -> Result<(), WriteRefused> {
        let payload = canonical::encode(entry).map_err(|error| WriteRefused {
            log: "validation-store",
            reason: format!("{error:?}"),
        })?;
        self.validations
            .append(&payload)
            .map_err(|error| WriteRefused {
                log: "validation-store",
                reason: format!("{error:?}"),
            })?;
        self.validations
            .commit(seam)
            .map_err(|DurabilityRefused { reason }| WriteRefused {
                log: "validation-store",
                reason,
            })?;
        Ok(())
    }
}

impl Default for ApplyCore {
    fn default() -> Self {
        ApplyCore::new()
    }
}

/// The journal's high-water instant: the maximum recorded instant over
/// every transition record. `None` on a journal with no transitions.
#[must_use]
pub fn high_water_instant(decoded: &DecodedJournal) -> Option<u64> {
    decoded
        .records()
        .iter()
        .filter_map(|(_, record)| match record {
            Record::Transition(transition) => Some(transition.instant().secs()),
            _ => None,
        })
        .max()
}

/// The backward-clock bound: a reading below the journal's high-water
/// instant refuses. This is the debt `clock.rs` named — monotonicity the
/// clock module deliberately did not claim — paid with the fact it
/// lacked: the journal's own mark.
///
/// # Errors
///
/// The offending pair, for the refusal's detail.
pub fn clock_bound(now: u64, high_water: Option<u64>) -> Result<(), (u64, u64)> {
    match high_water {
        Some(mark) if now < mark => Err((now, mark)),
        _ => Ok(()),
    }
}

/// The lifecycle states past the authorization boundary and before a
/// terminal — the window in which discovery is transitional (CONC-004).
/// `AwaitingAuthorization` and everything before it is settled ground:
/// nothing can be mid-write while no grant exists.
fn mid_apply(state: State) -> bool {
    !state.is_terminal()
        && !matches!(
            state,
            State::Draft | State::Validated | State::AwaitingAuthorization
        )
}

/// CONC-004's predicate: whether any journaled lifecycle currently
/// stands mid-apply. This is the value `capture` carries as
/// `transitional` — computed from the journal, never hard-coded. On this
/// build no lifecycle can pass the boundary, so the value is `false` at
/// runtime — but it is now false because the journal says so, and the
/// moment 4b grants an apply the same predicate answers `true` with no
/// edit here.
#[must_use]
pub fn transitional_now(decoded: &DecodedJournal) -> bool {
    last_states(decoded)
        .values()
        .any(|(state, _)| mid_apply(*state))
}

/// Every plan's last journaled transition: its resulting state and
/// instant, in journal order. A report, not a validation — the chain
/// discipline lives in `lifecycle::trace`.
fn last_states(decoded: &DecodedJournal) -> BTreeMap<[u8; 32], (State, u64)> {
    let mut states = BTreeMap::new();
    for (_, record) in decoded.records() {
        if let Record::Transition(transition) = record {
            states.insert(
                *transition.plan().as_bytes(),
                (transition.transition().to(), transition.instant().secs()),
            );
        }
    }
    states
}

/// One plan's row in the journal-query answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPlanRow {
    /// The plan's body hash bytes.
    pub plan_hash: [u8; 32],
    /// The last journaled state's name (Section 8's own words).
    pub state: &'static str,
    /// The last transition's recorded instant.
    pub instant: u64,
}

/// The journal-query answer: every plan's last journaled state, the
/// high-water instant, and the record count — all helper-authored, no
/// identifier anywhere (a plan hash is the identity the wire already
/// speaks; SEC-006's classes have no field here).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalReport {
    /// The high-water instant, if any transition is journaled.
    pub high_water_instant: Option<u64>,
    /// How many records the journal holds.
    pub records: u64,
    /// Per-plan rows, in hash order.
    pub plans: Vec<JournalPlanRow>,
}

/// Serve journal-query from the decoded journal.
#[must_use]
pub fn journal_query(decoded: &DecodedJournal) -> JournalReport {
    let plans = last_states(decoded)
        .into_iter()
        .map(|(plan_hash, (state, instant))| JournalPlanRow {
            plan_hash,
            state: state.name(),
            instant,
        })
        .collect();
    JournalReport {
        high_water_instant: high_water_instant(decoded),
        records: decoded.records().len() as u64,
        plans,
    }
}

/// The audit log's closed words for the transitions this module can
/// journal — the journal schema's own wire tags, transcribed; the audit
/// vocabulary test pins them.
pub const VALIDATOR_PASSES: &str = "validator-passes";
/// `EditOrInvalidation`'s wire tag.
pub const EDIT_OR_INVALIDATION: &str = "edit-or-invalidation";
/// `ApplySubmitted`'s wire tag.
pub const APPLY_SUBMITTED: &str = "apply-submitted";
/// `DeclinedOrExpired`'s wire tag.
pub const DECLINED_OR_EXPIRED: &str = "declined-or-expired";

/// Note one successful validation: journal `ValidatorPasses` (schema
/// v2's instant is the same `now` HLP-004 dated the window from), then
/// record the validation durably in the store — both committed before
/// the caller answers. Idempotent against the chain: a plan already
/// `Validated` gets no second `ValidatorPasses` row (the chain's `from`
/// discipline would refuse it), and a plan past the boundary gets its
/// store record refreshed but no row. Returns whether a row was
/// journaled, so the caller's audit line matches the journal.
///
/// # Errors
///
/// [`WriteRefused`] — the operation is refused rather than answered
/// un-journaled.
#[allow(
    clippy::too_many_arguments,
    reason = "every parameter is one helper-authored fact of the validation being recorded; \
              a struct would add a name for a set used exactly once"
)]
pub fn note_validation(
    core: &mut ApplyCore,
    journal_seam: &mut dyn DurabilitySeam,
    store_seam: &mut dyn DurabilitySeam,
    plan_hash: &Hash,
    validated_for_uid: u32,
    tier: AuthorizationTier,
    not_after: u64,
    body: &[u8],
    now: u64,
) -> Result<bool, WriteRefused> {
    let plan = PlanHashRef::from(plan_hash);
    let state = current_state(core, plan);
    let fresh_lifecycle = match state {
        None | Some((State::Draft, _)) => true,
        Some((state, terminal)) => terminal && state.is_terminal(),
    };
    if fresh_lifecycle {
        let record = TransitionRecord::non_terminal(
            plan,
            Transition::ValidatorPasses,
            RecordedInstant::from_secs(now),
        )
        .map_err(|error| WriteRefused {
            log: "journal",
            reason: format!("{error:?}"),
        })?;
        core.append_transition(journal_seam, record)?;
    }
    let recorded = RecordedValidation {
        plan_hash: *plan_hash.as_bytes(),
        validated_for_uid,
        tier,
        not_after,
        body: body.to_vec(),
        consumed: false,
    };
    let entry = encode_recorded(&recorded);
    core.append_store_entry(store_seam, &entry)?;
    core.records.insert(recorded.plan_hash, recorded);
    Ok(fresh_lifecycle)
}

/// The plan's current journaled state, with whether the chain has
/// terminated (a terminated chain accepts a fresh lifecycle).
fn current_state(core: &ApplyCore, plan: PlanHashRef) -> Option<(State, bool)> {
    let decoded = core.decoded().ok()?;
    let chain = lifecycle::trace(&decoded, plan).ok()?;
    chain.current().map(|state| (state, state.is_terminal()))
}

/// What `apply-plan` answers, wire-agnostic; the backends render it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyAnswer {
    /// Phase one succeeded: the submission is journaled and the apply
    /// awaits its authorization (S2's first phase).
    Awaiting {
        /// The plan's body hash bytes.
        plan_hash: [u8; 32],
        /// The helper-computed tier the authorization will require.
        tier: AuthorizationTier,
        /// The window's end.
        not_after: u64,
    },
    /// Refused, with the arm named and the ground in this crate's own
    /// words. Every arm is fail-closed.
    Refused {
        /// The refusing arm's name (a closed set; the wire test pins it).
        arm: &'static str,
        /// The ground.
        detail: String,
    },
}

/// One `apply-plan` decision: the answer, plus which transitions were
/// journaled while deciding — the caller's audit lines, in order, so the
/// log matches the journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyDecision {
    /// The answer.
    pub answer: ApplyAnswer,
    /// The journaled transitions' wire tags, in append order.
    pub journaled: Vec<&'static str>,
}

fn decision(answer: ApplyAnswer) -> ApplyDecision {
    ApplyDecision {
        answer,
        journaled: Vec::new(),
    }
}

/// Drive `apply-plan` for one presented hash — both phases of S2, split
/// on the plan's journaled state. See the module documentation for the
/// full arm order and the ordering rules.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn apply_plan(
    core: &mut ApplyCore,
    journal_seam: &mut dyn DurabilitySeam,
    store_seam: &mut dyn DurabilitySeam,
    plan_hash: &[u8; 32],
    fresh_capture: &TopologySnapshot,
    now: u64,
    peer_uid: u32,
    ceremony: &dyn Ceremony,
) -> ApplyDecision {
    let decoded = match core.decoded() {
        Ok(decoded) => decoded,
        Err(detail) => {
            return decision(ApplyAnswer::Refused {
                arm: "journal-decode",
                detail,
            });
        }
    };
    if let Err((reading, mark)) = clock_bound(now, high_water_instant(&decoded)) {
        return decision(ApplyAnswer::Refused {
            arm: "clock-behind-journal",
            detail: format!(
                "the clock reads {reading}, behind the journal's high-water instant {mark}; \
                 nothing is validated, presented or applied from a clock behind the record"
            ),
        });
    }
    let plan = PlanHashRef::from_bytes(*plan_hash);
    let chain = match lifecycle::trace(&decoded, plan) {
        Ok(chain) => chain,
        Err(broken) => {
            return decision(ApplyAnswer::Refused {
                arm: "chain-broken",
                detail: format!("{broken:?}"),
            });
        }
    };
    let current = chain.current();
    let terminated = current.is_some_and(State::is_terminal);
    match current {
        Some(State::AwaitingAuthorization) => {
            phase_two(core, journal_seam, plan, plan_hash, now, peer_uid, ceremony)
        }
        Some(State::Validated) => phase_one(
            core,
            journal_seam,
            store_seam,
            plan,
            plan_hash,
            fresh_capture,
            now,
            peer_uid,
        ),
        None | Some(State::Draft) => decision(not_validated_or_replayed(core, plan_hash)),
        Some(_) if terminated => decision(not_validated_or_replayed(core, plan_hash)),
        Some(state) => decision(ApplyAnswer::Refused {
            arm: "beyond-authorization",
            detail: format!(
                "the journal places this plan in {}, past the authorization boundary; \
                 everything from AuthorizationGranted onward is increment 4b's",
                state.name()
            ),
        }),
    }
}

/// The fresh-lifecycle refusals: nothing validated, or the validation's
/// one submission already spent — SEC-002's replay arm, durable across a
/// restart because consumption is a store entry.
fn not_validated_or_replayed(core: &ApplyCore, plan_hash: &[u8; 32]) -> ApplyAnswer {
    match core.records.get(plan_hash) {
        Some(record) if record.consumed => ApplyAnswer::Refused {
            arm: "replayed",
            detail: "this validation's one submission is spent; validate again for a fresh \
                     lifecycle"
                .to_owned(),
        },
        _ => ApplyAnswer::Refused {
            arm: "not-validated",
            detail: "no journaled validation stands for this hash; validate-plan first".to_owned(),
        },
    }
}

/// Phase one: the presentation. SEC-002's arms over the stored body and
/// the fresh capture; a stale presentation invalidates the draft on the
/// published edge (CONC-003); success consumes the validation (durably,
/// first) and journals `ApplySubmitted`.
#[allow(clippy::too_many_arguments)]
fn phase_one(
    core: &mut ApplyCore,
    journal_seam: &mut dyn DurabilitySeam,
    store_seam: &mut dyn DurabilitySeam,
    plan: PlanHashRef,
    plan_hash: &[u8; 32],
    fresh_capture: &TopologySnapshot,
    now: u64,
    peer_uid: u32,
) -> ApplyDecision {
    let Some(stored) = core.records.get(plan_hash).cloned() else {
        return decision(ApplyAnswer::Refused {
            arm: "not-validated",
            detail: "the journal holds a validation this store does not; validate-plan again"
                .to_owned(),
        });
    };
    let record_hash = match canonical::hash_encoded(&stored.body) {
        Ok(hash) => hash,
        Err(error) => {
            return decision(ApplyAnswer::Refused {
                arm: "validation-store",
                detail: format!("the stored body does not hash: {error:?}"),
            });
        }
    };
    let record = ValidationRecord {
        plan_hash: record_hash,
        validated_for_uid: stored.validated_for_uid,
        consumed: stored.consumed,
    };
    let admitted = match admit_presented_plan(&stored.body, fresh_capture, now, peer_uid, &record) {
        Ok(admitted) => admitted,
        Err(AdmissionRefusal::Stale) => {
            // CONC-003 on its published edge: the world moved, so the
            // Validated standing is journaled back to Draft rather than
            // merely refused.
            let invalidation = TransitionRecord::non_terminal(
                plan,
                Transition::EditOrInvalidation,
                RecordedInstant::from_secs(now),
            )
            .expect("EditOrInvalidation is a published non-terminal row");
            if let Err(refused) = core.append_transition(journal_seam, invalidation) {
                return decision(write_refused(&refused));
            }
            return ApplyDecision {
                answer: ApplyAnswer::Refused {
                    arm: "stale",
                    detail: "the plan binds a snapshot the fresh capture contradicts; the draft \
                             is invalidated on the published EditOrInvalidation edge (CONC-003) \
                             — validate again"
                        .to_owned(),
                },
                journaled: vec![EDIT_OR_INVALIDATION],
            };
        }
        Err(refusal) => return decision(admission_refused(&refusal)),
    };
    // Consume before submitting — the fail-closed order: a crash between
    // the two writes costs a re-validation, where the reverse order
    // would let one validation submit twice.
    let consumed = encode_consumed(plan_hash);
    if let Err(refused) = core.append_store_entry(store_seam, &consumed) {
        return decision(write_refused(&refused));
    }
    if let Some(entry) = core.records.get_mut(plan_hash) {
        entry.consumed = true;
    }
    let submitted = TransitionRecord::non_terminal(
        plan,
        Transition::ApplySubmitted,
        RecordedInstant::from_secs(now),
    )
    .expect("ApplySubmitted is a published non-terminal row");
    if let Err(refused) = core.append_transition(journal_seam, submitted) {
        return decision(write_refused(&refused));
    }
    // The tier is recomputed from the admitted plan — the same
    // provenance discipline as increment 3; the store's copy serves
    // phase two, where the admitted plan no longer exists.
    let tier = crate::authorize::required_tier(admitted.severity(), &admitted.flags());
    ApplyDecision {
        answer: ApplyAnswer::Awaiting {
            plan_hash: *plan_hash,
            tier,
            not_after: stored.not_after,
        },
        journaled: vec![APPLY_SUBMITTED],
    }
}

/// Phase two: the submitted apply's completion request. A closed window
/// terminates on the published `DeclinedOrExpired → Cancelled` edge; an
/// open one runs the authorization — which on this build refuses exactly
/// where increment 3 refuses, and past which no grant exists until 4b.
fn phase_two(
    core: &mut ApplyCore,
    journal_seam: &mut dyn DurabilitySeam,
    plan: PlanHashRef,
    plan_hash: &[u8; 32],
    now: u64,
    peer_uid: u32,
    ceremony: &dyn Ceremony,
) -> ApplyDecision {
    let Some(stored) = core.records.get(plan_hash).cloned() else {
        return decision(ApplyAnswer::Refused {
            arm: "validation-store",
            detail: "the journal holds a submitted apply this store does not; the store and \
                     journal disagree, and nothing proceeds over a disagreement"
                .to_owned(),
        });
    };
    if stored.not_after < now {
        let declined = TransitionRecord::terminal(
            plan,
            Transition::DeclinedOrExpired,
            Effect::NoWrites,
            None,
            RecordedInstant::from_secs(now),
        )
        .expect("DeclinedOrExpired is a published terminal row constrained to NoWrites");
        if let Err(refused) = core.append_transition(journal_seam, declined) {
            return decision(write_refused(&refused));
        }
        return ApplyDecision {
            answer: ApplyAnswer::Refused {
                arm: "declined-or-expired",
                detail: "the window closed while awaiting authorization; the plan is Cancelled \
                         on the published DeclinedOrExpired edge, NoWrites"
                    .to_owned(),
            },
            journaled: vec![DECLINED_OR_EXPIRED],
        };
    }
    match stored.tier {
        AuthorizationTier::InteractiveCeremony => {
            match ceremony.perform(plan, peer_uid) {
                Err(unavailable) => decision(ApplyAnswer::Refused {
                    arm: "ceremony-unavailable",
                    detail: unavailable.to_string(),
                }),
                // Structurally unreachable in a shipped build
                // (CeremonyCompleted has no constructor outside tests) —
                // and even a completed ceremony reaches no grant here,
                // because the AuthorizationGranted edge is 4b's. Pinned
                // by test.
                Ok(_completed) => decision(grant_not_served()),
            }
        }
        AuthorizationTier::FloorAct => decision(grant_not_served()),
    }
}

fn grant_not_served() -> ApplyAnswer {
    ApplyAnswer::Refused {
        arm: "grant-not-served",
        detail: "no authorization can be consumed on this build: the AuthorizationGranted \
                 edge and everything past it are increment 4b's, behind the toolset and \
                 launcher-home rounds"
            .to_owned(),
    }
}

fn write_refused(refused: &WriteRefused) -> ApplyAnswer {
    ApplyAnswer::Refused {
        arm: "durability",
        detail: format!(
            "the {} could not be made durable; nothing is answered ahead of its record",
            refused.log
        ),
    }
}

fn admission_refused(refusal: &AdmissionRefusal) -> ApplyAnswer {
    let (arm, detail) = match refusal {
        AdmissionRefusal::Replayed => (
            "replayed",
            "this validation's one submission is spent; validate again".to_owned(),
        ),
        AdmissionRefusal::CrossUser { .. } => (
            "cross-user",
            "the plan was validated for another user".to_owned(),
        ),
        AdmissionRefusal::HashMismatch => (
            "hash-mismatch",
            "the stored body does not hash to the recorded validation".to_owned(),
        ),
        AdmissionRefusal::Stale => unreachable!("the stale arm invalidates above"),
        AdmissionRefusal::CrossDevice => (
            "cross-device",
            "a bound identity contradicts the fresh capture".to_owned(),
        ),
        AdmissionRefusal::Altered { boundary } => {
            ("altered", format!("the stored body refuses: {boundary}"))
        }
        AdmissionRefusal::Expired { not_after, now } => (
            "expired",
            format!("the window closed at {not_after}; the clock reads {now}"),
        ),
    };
    ApplyAnswer::Refused { arm, detail }
}

// --- The validation store's wire, private to the helper. -------------

fn encode_recorded(recorded: &RecordedValidation) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(VALIDATION_SCHEMA.to_owned()),
    );
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(VALIDATION_SCHEMA_VERSION),
    );
    map.insert("kind".to_owned(), Value::Text("recorded".to_owned()));
    map.insert(
        "plan_hash".to_owned(),
        Value::Bytes(recorded.plan_hash.to_vec()),
    );
    map.insert(
        "uid".to_owned(),
        Value::Unsigned(u64::from(recorded.validated_for_uid)),
    );
    map.insert(
        "tier".to_owned(),
        Value::Text(recorded.tier.wire_name().to_owned()),
    );
    map.insert("not_after".to_owned(), Value::Unsigned(recorded.not_after));
    map.insert("body".to_owned(), Value::Bytes(recorded.body.clone()));
    Value::Map(map)
}

fn encode_consumed(plan_hash: &[u8; 32]) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "schema".to_owned(),
        Value::Text(VALIDATION_SCHEMA.to_owned()),
    );
    map.insert(
        "schema_version".to_owned(),
        Value::Unsigned(VALIDATION_SCHEMA_VERSION),
    );
    map.insert("kind".to_owned(), Value::Text("consumed".to_owned()));
    map.insert("plan_hash".to_owned(), Value::Bytes(plan_hash.to_vec()));
    Value::Map(map)
}

/// The store's own tier words, parsed privately. Not a client-namable
/// tier: no wire field feeds this, the store is root-owned, and the
/// journal crate's deliberate no-parse rule stands untouched — this
/// match exists so the store can be replayed, nothing else.
fn parse_tier(name: &str) -> Option<AuthorizationTier> {
    match name {
        "floor-act" => Some(AuthorizationTier::FloorAct),
        "interactive-ceremony" => Some(AuthorizationTier::InteractiveCeremony),
        _ => None,
    }
}

fn apply_store_entry(
    records: &mut BTreeMap<[u8; 32], RecordedValidation>,
    payload: &[u8],
) -> Result<(), String> {
    let value = canonical::decode(payload).map_err(|error| format!("{error:?}"))?;
    let Value::Map(mut map) = value else {
        return Err("a store entry is a map".to_owned());
    };
    match map.remove("schema") {
        Some(Value::Text(schema)) if schema == VALIDATION_SCHEMA => {}
        _ => return Err("wrong store schema".to_owned()),
    }
    match map.remove("schema_version") {
        Some(Value::Unsigned(VALIDATION_SCHEMA_VERSION)) => {}
        _ => return Err("wrong store schema version".to_owned()),
    }
    let Some(Value::Text(kind)) = map.remove("kind") else {
        return Err("a store entry names its kind".to_owned());
    };
    let plan_hash: [u8; 32] = match map.remove("plan_hash") {
        Some(Value::Bytes(bytes)) => bytes
            .try_into()
            .map_err(|_| "plan_hash is 32 bytes".to_owned())?,
        _ => return Err("plan_hash is bytes".to_owned()),
    };
    match kind.as_str() {
        "recorded" => {
            let uid = match map.remove("uid") {
                Some(Value::Unsigned(uid)) => {
                    u32::try_from(uid).map_err(|_| "uid fits u32".to_owned())?
                }
                _ => return Err("uid is unsigned".to_owned()),
            };
            let tier = match map.remove("tier") {
                Some(Value::Text(name)) => {
                    parse_tier(&name).ok_or_else(|| "an unknown tier word".to_owned())?
                }
                _ => return Err("tier is text".to_owned()),
            };
            let Some(Value::Unsigned(not_after)) = map.remove("not_after") else {
                return Err("not_after is unsigned".to_owned());
            };
            let Some(Value::Bytes(body)) = map.remove("body") else {
                return Err("body is bytes".to_owned());
            };
            if !map.is_empty() {
                return Err("a store entry carries no other field".to_owned());
            }
            records.insert(
                plan_hash,
                RecordedValidation {
                    plan_hash,
                    validated_for_uid: uid,
                    tier,
                    not_after,
                    body,
                    consumed: false,
                },
            );
            Ok(())
        }
        "consumed" => {
            if !map.is_empty() {
                return Err("a store entry carries no other field".to_owned());
            }
            match records.get_mut(&plan_hash) {
                Some(entry) => {
                    entry.consumed = true;
                    Ok(())
                }
                // A consumption for a record this log never recorded:
                // refuse the recovery rather than guess.
                None => Err("a consumption names an unrecorded validation".to_owned()),
            }
        }
        _ => Err("an unknown store entry kind".to_owned()),
    }
}
