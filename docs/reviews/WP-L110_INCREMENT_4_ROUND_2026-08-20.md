# WP-L110 increment 4 (the apply): the shape round

**2026-08-20. Owner: WP-000. Decision sought — and taken the same day:
the adversarial pass over §4 and §7 found three of this round's own
grounds false against the delivered code, and the corrected decisions
are recorded in §9.**

Increment 4 cannot be started as written, for a reason the assignment
itself states: two route decisions are recorded as owed **before
increment 4**, and neither has been taken. Beyond that gate, the
increment as specified is not one increment — it is roughly fourteen
independently mergeable pull requests across five work packages, and at
least three of them belong to packages other than WP-L110.

This round measures the ground, records three defects in **delivered**
code that increment 4 would activate, proposes a re-cut into 4a and 4b,
and puts the decisions that re-cut needs to the owner. It does **not**
take either of the two gating rounds: those are their own acts, and one
of them still lacks the measured substrate to be honest.

---

## 1. The gate, measured

`docs/work-packages/WP-L110.md` records three route decisions "none
decided by drift". Route (a) was taken (the launch round, 2026-08-19).
The other two read, verbatim:

> (b) LIN-001's authorization/mutation half (ADR-0054: UDisks2,
> libblockdev or authoritative native tools; the UDisks2 ≥ 2.9 tool floor
> enters the CAP-006 store with the first invoker), **before increment 4**;
> (c) the launcher's home (WP-035's `SystemLauncher` is in `apps/cli`; a
> helper cannot depend on an app), **before increment 4**, with WP-035 in
> the room.

**Neither round exists.** `docs/reviews/` holds the transport round, the
UDisks2 route round, the launch round, the apply-ceremony round and the
assignment plan for 2026-08-19, and nothing else; the ADR series ends at
0055, and ADR-0054 took the *discovery* half only, handing the mutation
half over explicitly.

The delivery-status row says the same thing in its own words: *"Not
started (the toolset and launcher-home rounds first)."*

So the first question this round answers is not "what does increment 4
build" but **"what may be built before those rounds are taken"**.

---

## 2. What increment 4 actually is

The increment line reads: *"The state machine over the journal; CONC-001
locking; the product's first GPT/MBR table writer …; file-system
operations through the launcher per installed capability; cancel and
resume; the first CAP-006 entries filed on WP-050."* The delivery-status
row adds the two-phase apply wire (S2), the journal's on-disk home and
first real `DurabilitySeam`, the act's consumption through `admit_apply`,
and a backward-clock bound.

Measured against the tree, that decomposes as follows. **Bold** rows are
not WP-L110's to write.

| # | Piece | Owner | Gated on |
|---|---|---|---|
| 1 | The mutation-toolset round | **WP-000** | a tool-presence row (§5) |
| 2 | The launcher-home round | **WP-000** (WP-035 in the room) | — |
| 3 | `Governance:` PR reserving the launcher's path and its manifest share | **WP-000** | 2 |
| 4 | Move `ToolLauncher`/`SystemLauncher`; give `launch` a caller-stated deadline | **WP-035** | 3 |
| 5 | A recorded instant on the journal's transition record (schema v2) | **WP-070** | — |
| 6 | The journal's on-disk home and the first real `DurabilitySeam` | WP-L110 | 5 |
| 7 | The backward-clock bound off the journal's high-water mark | WP-L110 | 5, 6 |
| 8 | CONC-001 locking over the ADR-0018 bind set | WP-L110 | 6 |
| 9 | The two-phase `apply-plan` wire, with the `Revalidating` leg | WP-L110 | 6, 7, 8 |
| 10 | CONC-002/003/004 (revalidation, draft invalidation, transitional capture) | WP-L110 | 9 |
| 11 | Cancel and resume from the journal | WP-L110 | 9 |
| 12 | `journal-query` served | WP-L110 | 6 |
| 13 | The first GPT/MBR table writer | WP-L110 | **1**, 6, 9 |
| 14 | File-system operations through the launcher | WP-L110 | **1**, **4**, 13, 15 |
| 15 | The first CAP-006 floors and the store's runtime reader | **WP-050** | **1** |

Two things follow immediately. Rows 13–15 cannot be specified at all
until round 1 decides whether the first partition-table write is *our own
encoder* or *a launched binary* — that is the whole content of the
decision. Rows 6–12 do not invoke a tool, do not launch anything and do
not write a device byte, so neither gating round says anything about
them.

---

## 3. Three defects in delivered code that increment 4 would activate

These are not plan risks. They are in the tree now, and each is latent
only because nothing executes yet.

### 3.1 CONC-004: the capture hard-codes `transitional: false`

`services/helper-linux/src/capture.rs:246` calls
`TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges,
facts)`. The second parameter is `transitional`
(`crates/domain/src/model/snapshot.rs:97-103`), and the requirement
imported as WP-L110's obligation 10 says *"discovery during execution is
transitional"*.

Today the literal `false` is true by accident: this build cannot execute,
so no capture is ever taken during an apply. **The moment increment 4
makes the helper capable of executing, that literal becomes a fail-open**
— every capture taken mid-apply will assert it is a settled view of the
world, and the flag exists precisely so that it cannot.

This is the shape increment 3 found three times over (the epoch-0 clock,
the idle watchdog, the discarded audit write): a value that is harmless
until the increment that gives it a caller. It should be closed by the
same increment that creates the caller, and it needs a Tier-1 test that a
capture taken with an apply in flight is **hash-distinct** from the same
topology captured with none.

### 3.2 `WriteClearance` proves less than its documentation claims

`crates/journal/src/lib.rs` mints the write token:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteClearance { record: SeqNo }
pub const fn record(self) -> SeqNo            // public accessor
pub fn clearance(&self, record: SeqNo) -> Result<WriteClearance, NotYetDurable>
//   Some(through) if record <= through => Ok(WriteClearance { record })
```

The field is private, which is what makes the token unforgeable. But the
type is `Copy`, `SeqNo::FIRST` is a public const, and `clearance` mints a
token for **any** record at or below the watermark. So
`journal.clearance(SeqNo::FIRST)` succeeds as soon as anything at all is
durable, and one clearance can be reused across every step of an apply.

A writer whose only gate is "takes a `WriteClearance` by value" therefore
does **not** hold the property "this step's own record is durable". The
structural claim available from the delivered type is weaker than the
sentence the module's own doc comment implies, and a writer built on the
stronger reading would be a fail-open that passes every happy-path test.

**The fix is cheap and belongs in the writer's entry point, not in
WP-070's crate:** take the clearance *and* the `SeqNo` returned by
appending this step's own record, and refuse unless
`clearance.record() == that seq`. The mutations that bite are `pass
clearance(SeqNo::FIRST)` and `reuse one clearance across two steps`.

This is a structural-claims-come-from-types finding, and it is recorded
here rather than fixed quietly because a future reader of that doc
comment would otherwise inherit the stronger reading.

### 3.3 Increment 2's SEC-002 admission arms have no production caller

`services/helper-linux/src/validate.rs:294` delivers
`admit_presented_plan` with seven arms — replayed, cross-user,
hash-mismatch, stale, cross-device, altered, expired — and
`ValidationRecord` (`:179`) carries the `consumed` flag the replay arm
reads. A grep for either name outside tests returns **only doc comments
and the definitions themselves**.

`ValidationRecord` also has no durable home: `consumed` is a plain `pub
bool` that nothing persists. So SEC-002's replay arm is, today, a tested
function wired to nothing — which is honest (increment 2 said the apply
would consume it, and the apply moved to increment 4) but means increment
4 owes it a caller **and a durable store**, or the arm is decorative.

The journal's own `admit_apply` does not supply this: it enforces *one
act, one apply* over journaled acts, which is a different property from
*one validation, one presentation*.

---

## 4. The proposed re-cut: 4a and 4b

*(Superseded in part by §9. The re-cut itself stands, but this section's
boundary — "as far as the first device byte, then refuses" — was found
unsound against the delivered state machine: `Executing` publishes no
exit a refusal could honestly take, `Protecting` stands before any byte
with an unowned artifact store, and CAP-003's `supported` is
unconstructible on this build. The decided boundary is the
**authorization boundary**.)*

**4a — the journal-borne apply, refusing at the byte.** Rows 5–12: the
on-disk journal and the first real `DurabilitySeam` (today the only two
`impl DurabilitySeam` in the tree are `FakeSeam` and `AcknowledgingSeam`,
both inside test modules), the backward-clock bound, CONC-001 locking
over the ADR-0018 bind set, the two-phase `apply-plan` wire including the
`Revalidating` leg the state machine already publishes
(`crates/statemachine/src/lib.rs:310-313`), CONC-002/003/004, cancel and
resume, `journal-query`, and §3's three fixes.

4a drives the state machine as far as the first device byte and then
**refuses**, behind a seam whose only shipped implementation refuses —
exactly the shape the owner approved for the ceremony in increment 3
(R8). Nothing in 4a launches a tool, writes a table, or reads the
capability store, so neither gating round bears on it.

**4b — the writing half.** Rows 1–4, 13–15: both rounds, the Governance
PR, WP-035's launcher move, the table writer, the filesystem operations,
and WP-050's floors. Gated on the two rounds, as the assignment requires.

**Why this respects the gate rather than routing around it.** Read route
(b)'s own words: it decides *"UDisks2, libblockdev or authoritative
native tools"* — a choice about which binary or library performs a
mutation. Route (c) decides where the launcher that invokes it lives.
Both are decisions about **tool invocation**. 4a invokes no tool. The
gate is not weakened by splitting on that line; it is applied to exactly
the work it names.

**What the re-cut costs.** It is itself an act: a WP-L110 edit to the
increment list and the delivery-status row, moving the two route gates
onto 4b. Without that edit the assignment gates increment 4 wholesale,
and any 4a pull request would be building against a stated gate. That
edit should land first and on its own.

---

## 5. What the two rounds must decide, and what each has to stand on

### Round (b), the mutation toolset — substrate is *partly* there

The record already costs one of the three options to a decision, and this
round should say so rather than commissioning measurement that exists:

- **UDisks2 is not installed by default on two of the three pinned
  tiers.** DR18 measured Arch: *"`udisks2` is not installed by default"*
  — no binaries, no unit. DR19 measured Debian 12: *"`udisks2` is not
  installed by default"* — `dpkg-query` rc 1, all three daemon paths
  absent by name. DR20 adds the polkit substrate: jammy ships polkit
  0.105 with setuid `pkexec`; Debian 12 ships `polkitd` 122 but **not**
  `pkexec`; Arch ships no polkit at all.
- **The native tools are unmeasured everywhere.** `sgdisk`, `sfdisk`,
  `mkfs.*` and `libblockdev` appear essentially nowhere in
  `docs/quality/observability.md`. A round costing "authoritative native
  tools" against nothing would be the fourth route decision taken on
  belief in a record that has refused to do that three times.

So round (b) needs **one preregistered row**: presence and version of the
candidate mutation tools on the three pinned tiers. Note the ordering
trap — WP-035's grant over `observability.md` is *enumerated*, not
general, and admits another package's row only where that package's own
assignment has filed it. So the sequence is: a WP-L110 obligations edit
filing the row by name (the DR20/DR21 shape its obligations block already
uses) → WP-035 preregisters and records it → round (b).

### Round (c), the launcher's home — substrate is complete

Nothing further needs measuring. What the round must decide is the home,
and it should also fix a defect the move makes urgent:
`LAUNCH_TIME_LIMIT` is a **private** 5-second constant
(`apps/cli/src/doctor.rs:118`) while only `output_limit` is caller-stated
(`fn launch(&self, path: &Path, arguments: &[&str], output_limit: usize)`,
`:176`). A `mkfs` over a large volume outlives five seconds; so does any
prompt. The deadline must become caller-stated in the same move, or 4b
inherits a launcher that kills its own long operations.

The round also needs a `Governance:` PR **before** any code, because
ownership is read from the base revision — an act cannot widen its own
assignment. That PR must reserve both the new path *and* WP-035's share
of the workspace manifest for the new member; WP-035's recorded share is
*"the `members` entry for `apps/cli` only"*.

### A stale sentence to correct in passing

The launch round says the apply ceremony's mechanism is *"decided … by
the toolset round (`pkcheck` through the launcher, or the bus)"*, while
the later ceremony round routes R1/R2 to its own follow-up and lists the
toolset round as out of its scope. The ceremony round is the later,
decision-bearing document and governs. This is a one-line correction, not
a blocker: increment 4 ships `RefusingCeremony` either way.

---

## 6. Recommendation

1. **Re-cut increment 4 into 4a and 4b** on §4's line, as a WP-L110 act
   landing on its own.
2. **Build 4a now**, with §3's three defects closed inside it, and with
   the execution boundary refusing behind an unconstructible-completion
   seam (the R8 shape).
3. **Sequence 4b behind its two rounds**, and file the tool-presence row
   first so round (b) has a substrate.
4. **Take WP-070's record-schema act (row 5) before any journal byte
   reaches disk.** `RECORD_SCHEMA_VERSION` is `1`
   (`crates/journal/src/records.rs:75`), decode refuses any other version,
   and no migration path exists in the delivered types. A v1 journal
   written now becomes unreadable the moment the instant forces v2, with
   nothing to migrate it.

**The one decisive ground for (2):** every piece of 4a is provable at
Tier 1 today, and none of it depends on a decision nobody has taken. The
alternative — waiting for both rounds — leaves the three §3 defects in
the tree, leaves SEC-002's admission arms wired to nothing, and leaves
`journal-query` answering `not-yet-served` naming an increment that is
not being worked.

*(§9 adjusts recommendation 2's shape: 4a as decided contains no
execution boundary at all — the refusal is phase two's, on the ground
increment 3 already refuses — and §3.2's entry-point discipline moves
to 4b with the writer; only the doc-comment correction stays in 4a.)*

---

## 7. Decisions for the owner

*(All five are answered in §9 — questions 1 and 2 taken by the decision
owner, 3–5 resolved as consequences of the decided boundary. Question
2's stated ground is false as written; see §9.1 finding 2.)*

1. **Accept the 4a/4b re-cut?** If not, increment 4 waits on both rounds
   and nothing is built now.
2. **Where does the recorded instant go — `TransitionRecord` only, or
   `AuthorizationAct` as well?** Transition-only is one WP-070 PR with no
   consumer to break. But transition records exist only *inside* an
   apply, so a high-water mark read from them is empty until a grant is
   journaled — and the exposure `clock.rs` actually names is a clock
   stepped back **between a plan's validation and its presentation**,
   which is before any transition exists. Transition-only would deliver a
   bound that does not cover the case it was written for.
   **Recommendation: on both**, accepting the three-PR consumer-first
   sequence (`AuthorizationAct::new` is the helper's only constructor, so
   the consumer cannot adapt in a form valid under both regimes without
   it).
3. **What is the CONC-001 lock mechanism?** Nothing in the repository
   names one — no ADR, spec issue or work package mentions `flock`,
   `LOCK_EX`, `O_EXCL`, a lock file or a helper-held table. CONC-005
   requires "exactly one wins; the loser is explained". This needs a
   decision before row 8.
4. **Does 4a ship with the transport still `Unrecognized`?** The r56
   acceptance measured that no plan reaches `Validated` on any real
   device, so 4a's Tier-2 acceptance can prove the wire, the journal, the
   lock and the refusals, but **not** an end-to-end apply. WP-L110's
   Verification clause asks for "cancel and resume proven, the device
   lock held for the full execution". Note this has **two** independent
   grounds, not one: the transport rows *and* the wire's inability to
   spell a sized create, which increment 3 pinned by test deliberately.
   Landing WP-010's transport rows alone would not make the clause
   reachable.
5. **Does increment 4 owe EXE-001 (sleep/hibernation inhibition during
   Protecting/Executing/Verifying) and EXE-003 (progress reporting)?**
   Neither is named in the increment line, and nothing in the tree
   inhibits anything. 4a is the first increment that could.

---

## 8. What does not depend on any of this

Recorded so it is not re-litigated: the three §3 defects are real and
worth closing whatever is decided about the re-cut; the WP-070 schema act
is required before any journal byte reaches disk under every option; and
the tool-presence row is worth filing under every option, because round
(b) cannot be taken honestly without it.

Out of scope for increment 4 entirely, and named so: LIN-003…007 and
LIN-010 (LUKS, LVM, mdraid, dm/multipath, GRUB, fstab/crypttab), the
SI-13 round, the transport-discrimination protocol (WP-010's, recorded as
"nothing owed now"), the interactive ceremony route, a sized-create
spelling on the wire, `packaging/**` (no directory, no assignment),
SEC-009's user-controlled retention, and increment 5's record sweep.

---

## 9. The adversarial pass, the corrections, and the decisions taken

Written after §§1–8 were drafted and before anything landed; the round
above is retained as proposed, and this section records what an
adversarial pass over its own §4 and §7 found, verified against the
delivered code, and what the decision owner then decided. Every claim
below was read off delivered types and published transitions, never off
spec text or this round's own prose.

### 9.1 Four findings, verified

1. **`Executing` publishes no exit a refusal could take.** Its exits are
   `FinalStepComplete → Verifying`, `UserPauses → Paused`,
   `RebootStepReached → RebootPending`, `StepFailureOrInterruption →
   RecoveryRequired` and `CancelHonored → Cancelled`
   (`crates/statemachine/src/lib.rs:316-320`); `Failed` is reachable
   only from `RecoveryRequired` (`:329`). So §4's "drives the state
   machine as far as the first device byte and then refuses" has no
   honest published row — every such apply would be journaled either as
   a cancel nobody requested or stranded in `RecoveryRequired` as
   designed behaviour.

2. **§7.2's stated ground is false.** `ValidatorPasses => (Draft,
   Validated)` (`:307`) is a published transition taken *at validation*.
   Transition records do not exist only inside an apply: a
   transition-only instant is journaled before any act exists — which is
   exactly the window `clock.rs` names, a clock stepped *"backwards
   between a plan's validation and its presentation"*
   (`services/helper-linux/src/clock.rs:20`).

3. **§4 missed `Protecting` entirely.** First entry into `Executing` is
   only `BackupsVerified => (Protecting, Executing)` (`:314`), so
   PART-013's backup discharge stands between revalidation and any
   device byte — and its metadata-backup artifact store has no owning
   assignment anywhere in the catalogue.

4. **A third gate, independent of everything above.** CAP-003's
   `supported` is constructible only through `QualificationEvidence`
   (`crates/capability/src/lib.rs:281-282`), which has no constructor
   until its store-loading constructor exists (`:274`), and the CAP-006
   store is empty. No apply is structurally permissible on this build,
   whatever increment 4 ships.

### 9.2 Decision 1, taken: the re-cut is accepted, with 4a ending at the authorization boundary

4a delivers: the journal's on-disk home and the first real
`DurabilitySeam`; `ValidationRecord` made durable and its consumption
through `admit_presented_plan` structurally disciplined — §3.3's arms
get their production caller *and* their store; the backward-clock bound;
the two-phase wire's phase one — `ApplySubmitted` journaled,
`awaiting-authorization` answered; phase two refusing exactly where
increment 3 already refuses, which finding 4 makes the structurally
*true* answer rather than a stub; a plan's terminal path the published
`DeclinedOrExpired → Cancelled` edge (`:311`); `journal-query` served;
CONC-003's draft invalidation (it attaches to `Draft`, before the
boundary); §3.1's CONC-004 fix with its hash-distinct test; and the
one-line correction of §3.2's doc comment. 4a invokes no tool, no bus,
no EXE-001, and writes no device byte — neither gating round bears
on it.

Everything from `AuthorizationGranted` onward moves to 4b:
`Revalidating` (CONC-002), `Protecting` and PART-013's store,
`Executing`, the writer and §3.2's entry-point discipline, CONC-001 and
its undecided mechanism, cancel and resume.

**Rejected, recorded so it is not re-litigated:**

- *Keep 4a through `Executing`.* Three independently fatal costs: a
  Section 8 spec change adding an `Executing → Failed` edge (a
  published-exits sentence becomes false — major, by the pricing rule);
  PART-013's artifact store, which has no owning assignment and needs a
  `Governance:` PR before any code; and EXE-001 inhibition, which on
  Linux is a launched binary or a logind bus client — route (b)/(c)
  subject matter, so 4a would no longer be gate-clean and the re-cut
  would have routed around its own gate. And were all three paid,
  finding 4 leaves `Executing` honestly unreachable: machinery with no
  honest Tier-2 exercise on this build.
- *Journal only, the wire deferred.* Re-creates the two complaints §6
  gave as the decisive ground for building now: `apply-plan` answering
  `not-yet-served` naming an increment not being worked, and SEC-002's
  seven arms wired to nothing for another increment.

### 9.3 Decision 2, re-taken: the recorded instant is transition-only

§7.2 recommended "on both" on one ground — that a transition-record
high-water mark stays empty until a grant is journaled. Finding 2 shows
that ground false: the mark is populated from the first validation
onward, covering precisely the exposure the bound was written for. With
the ground gone, "on both" has costs and no purchase: `AuthorizationAct::new`
is the helper's only constructor, so an act-borne instant forces the
breaking three-PR consumer-first sequence, and every act milestone that
reaches the journal already carries an instant on its own transition
record.

**Decided: `TransitionRecord` only** — one WP-070 PR, schema v2, landed
before any journal byte reaches disk (§6's point 4 stands unchanged).
**Rejected:** "on both", as unpriced insurance. The re-open trigger is
named now so it is not re-argued from taste: a grant-time window that
only an act-borne instant covers. None is known.

### 9.4 §7's remaining questions, resolved by the boundary

- **§7.3, the CONC-001 mechanism:** moves to 4b with the locking. The
  decision is owed before 4b's locking row and blocks nothing in 4a.
- **§7.4, Tier-2 with the transport `Unrecognized`:** dissolves for 4a —
  nothing in its scope needs a real device. The Verification clause's
  "cancel and resume proven, the device lock held for the full
  execution" moves to 4b with those pieces; 4a's own acceptance proves
  the wire, the journal, the refusals, phase one's
  `awaiting-authorization` and the published cancel edge. §7.4's two
  independent grounds still stand against any end-to-end apply claim,
  and bind 4b.
- **§7.5, EXE-001/EXE-003:** 4a owes neither — both attach to
  `Protecting`/`Executing`/`Verifying`, none of which 4a constructs. 4b
  owes the decision, taken in the room with its rounds.

### 9.5 The acts this section commissions

1. This record lands as WP-000's, on its own.
2. The re-cut edit — WP-L110's increment list, delivery-status row and
   route-gate sentences, the gates moving onto 4b — lands as WP-L110's
   own act, on its own, before any 4a code (§4's costing stands).
3. WP-070's transition-only schema act (§2 row 5) precedes any journal
   byte reaching disk, as 4a's first prerequisite.
