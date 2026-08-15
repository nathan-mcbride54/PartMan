# ADR-0040: The relocation exemption is retired, and the release entry stands

- Status: Accepted
- Date: 2026-08-14. Made on the adversarially reviewed round of
  2026-08-14 (`docs/reviews/ISSUE-348_RELOCATION_ROUND_2026-08-14.md`,
  an untracked session artifact; everything load-bearing is restated
  here), under the decision owner's directive to resolve issue #348.
  Merging is not acceptance — every element below is reviewable against
  the round's recorded alternatives.
- Spec version: **13.0.1 — patch under §0.1.** The argument is made in
  full below, with both counter-arguments recorded and declined.
- Work packages blocked: none. Issue #348 closes here. Two residuals are
  filed rather than carried.
- Requirement IDs: MODEL-002, SAFE-005, PART-005, ADR-0018, ADR-0038
- Decision owners: Nate McBride

## Context

Issue #348 recorded that two accepted records disagree about what a
`Move` step's affected set contains:

- **ADR-0018:141-145** exempted "the relocated target's own subtree from
  destruction descent — its content is preserved by contract
  (PART-005)".
- **ADR-0038:56-61** gave `Shrink` and `Move` the conservative entry —
  the whole target extent in `destroyed` — which puts that same subtree
  into the set.

**The issue's framing is superseded, and the measurement is what
supersedes it.** #348 was filed at `5b795df`. ADR-0039 landed after it
and made descent run from *any* node in the affected set
(`protection.rs:269-271`), while the target is seeded unconditionally
(`:239`). The issue's option 1 — "`Move` needs a distinct canonical
entry" — was therefore worth measuring before being argued.

## Measured

At `b3de0cf` (main), own `CARGO_TARGET_DIR`, mutations applied with
`Edit` and proven applied by printed diff before each run.

**M1 — `Move` moved from the destroyed arm to the written-extents arm.**

| target | `Move` gate | vs baseline |
| --- | --- | --- |
| `part(luks-host)` | `Unsupported{InheritedFromConsumerOrProducer}` | unchanged |
| `sda(device)` | `Unsupported{Zfs}` | unchanged |
| `member(part)` | `Unsupported{Zfs}` | unchanged |
| `esp(part)`, `table(gpt)` | `Clear` | unchanged |
| `pool(zfs)` | `Unsupported{Zfs}` | unchanged |

Only the range *class* changed. The full domain suite was green: not one
committed regression could observe M1.

**M3 — `Move` surrenders its canonical entry entirely
(`StepRanges::default()`).** This is the most generous reading of the
exemption that `canonical_ranges` alone can express.

| target | `Move` gate | vs baseline |
| --- | --- | --- |
| `part(luks-host)` | `Unsupported{InheritedFromConsumerOrProducer}` | **unchanged — still refuses** |
| `sda(device)` | **`Clear`** | **`Unsupported{Zfs}` → `Clear` over a live pool** |
| `member(part)` | `Unsupported{Zfs}` | still refuses |

**The full domain suite was green under M3.** A surviving mutant that
opens a whole-disk gate over a live ZFS pool.

**Why the split.** On a partition target, carried-content reach alone
refuses — the target seeds the set and descent runs from it. On a
whole-disk target it does not: `descends_into` refuses a self-framed
extent as a descent source, which is precisely what stops a device's own
extent capturing its siblings. So **on a disk target reach is entirely
range-driven, and ADR-0038's release entry is the only thing refusing.**

## The decision

**ADR-0038's `Move` entry stands. ADR-0018's relocation exemption is
retired as void. No production line changes.**

1. **The exemption clause is retired**, on the narrow ground that it was
   void where it stood: §0.2's rule 4 says an ADR "MUST NOT weaken any
   MUST", and §2.1's enforcement paragraph is a MUST NOT. A clause that
   would have exempted a subtree from that enforcement never had force
   to begin with. It was additionally never delivered, never cited by
   any requirement (swept: no citation exists outside ADR-0018 itself),
   and not expressible in the delivered closure, which takes no
   `Operation` at either call site (`protection.rs:449`, `step.rs:417`).
2. **The byte-wise-preservation duty in the same paragraph survives.**
   It is a plan-layer obligation, not a closure exemption, and nothing
   about it weakens §2.1.
3. **One regression closes the measured hole.** Nothing committed
   asserted that a release over a whole disk carrying a protected
   aggregate refuses.

**What this decision does not do.** It does not declare the resulting
refusal acceptable. See *The residual, named not settled* below.

## The residual, named not settled

Retiring the exemption leaves a real availability gap, and this ADR
records it as a gap rather than as an answer:

> A length-preserving relocation of a partition carrying a protected
> structure refuses although copy-then-commit would preserve every byte.

The exemption's underlying grievance is **correct** — copy-then-commit
does preserve content. This ADR argues only that the refusal is right
*given what the closure can know*: nothing in the range sets
distinguishes a relocation from a deletion, and relief must come from a
computed and authenticated preservation proof, not from an
author-assertable class flag. R2 below shows the obvious form of that
proof is structurally impossible today.

The decision owner was put the alternative — recording the refusal as
settled product policy — and **declined it**. The gap is filed as its
own issue. Retiring the clause without carrying the gap forward would
launder a known false refusal into settled design, which would be worse
than the contradiction it removes.

**Today the user-visible cost is zero, and that is what makes this act
dangerous.** No user can attempt a move at all: `crates/planner`
refuses every `Operation::Move` as `NotRepresentable` — "moves and
copies need a destination vocabulary this model does not carry yet". The
bill is deferred, and will be paid by whoever implements PART-005, who
will find the protection layer already committed against the case their
requirement exists to serve.

## Options considered, and rejected

| # | Option | Rejected on |
| --- | --- | --- |
| R1 | **Give `Move` a distinct canonical entry** — issue #348's own option 1, reverting ADR-0038's `Move` half | **Measurement.** M1 changes no verdict on any of six targets with the suite green; M3 changes none on a partition and **opens `Clear` over a live pool on a whole disk**. The exemption is not reachable through `canonical_ranges`, and reaching for it there is a safety regression. |
| R2 | **Relocation as a fourth range class, with preservation computed rather than exempted** | **Structure, read off the delivered types.** The predicate must key on `facts.extents`, but no §2.1 aggregate can carry an extent — `may_carry_extent` forbids one on aggregate, volume, encryption-layer and multipath kinds. The predicate is therefore **inert on exactly the class it must relieve, and live on the guard standing in for it**: inverted with respect to safety. Not curable by ADR-0037. |
| R3 | **Pass `Operation` into `affected_set` and exempt on relocation class** | **ADR-0039's invariant.** This subtracts reach. `facts.extents` is authored body content that nothing authenticates (ADR-0037's enforcement is held), and `mutating_declared` has no capability gate in front of it — so the exemption would be keyed on a class reachable from an authored body. A predicate able to subtract reach hands an author a lever on protection; four earlier predicates died on exactly that. |
| R4 | **Retire the clause and record the refusal as settled product policy** | **Declined by the decision owner**, 2026-08-14. It commits the product against the case PART-005 exists to serve, on a real and common layout (encrypted-root Linux over ZFS is the measured case), on the strength of measurements that decide only that the *exemption as written* cannot deliver it safely. |
| R5 | **Delete the byte-wise-preservation duty along with the exemption** | **Judgement, and the weakest call here.** It is the only recorded statement in the repository that a relocation can lose a hosted signature. Carrying one undelivered sentence is cheaper than losing that statement. |
| R6 | **Hold #348 open until a preservation-proof design is funded** | Leaves a self-contradicting record in the tree, and leaves the measured whole-disk hole uncovered. The record correction and the coverage are separable from the product question and are landed here. |

## The spec price argument

**Patch, 13.0.1.** §0.1: "editorial fixes bump patch". No requirement
text changes. The retired clause never entered `AGENT_BUILD_SPEC.md`; a
sweep for its normative footprint — `relocat`, `copy-then-commit`,
`exempt`, `byte-wise`, `hosted signature`, `preserved by contract`,
`PART-005` — returns **no citation of it anywhere outside ADR-0018
itself**. Nothing narrows, no reach moves, and the closure is unchanged
line-for-line.

**Two counter-arguments, recorded and declined.**

- **No bump at all**, on ADR-0038's precedent (a defect fix with the
  spec untouched took no version). Declined: ADR-0038 left ADR-0018's
  text untouched and this act changes it, and §2.1 delegates the closure
  to ADR-0018 by name — a reader tracing that delegation now finds
  different content. The changelog row is what makes that traceable.
- **Major**, on the reading that removing a sentence from the delegated
  closure description is a semantic change. Declined: §0.1's major arm
  is about *requirements*, and the clause was never one — it was void
  under §0.2 and had no force to lose. Recording it as major would price
  a correction as a change in obligation.

**Not a §0.2 / §1.11 filing.** That register needs two conflicting
**requirements** in `AGENT_BUILD_SPEC.md`, named and quoted. Here one
side is an ADR clause and the other is §2.1; §0.2 rule 4 settles that
pair by precedence rather than by filing, which is why this is a
correction and not a conflict. PART-005 against §2.1 is also not such a
pair: PART-005 requires that moves preserve data, not that every
partition be movable.

## Consequences

**Positive**

- The self-contradicting record is repaired: ADR-0018 stops contradicting
  §2.1's enforcement paragraph and stops contradicting ADR-0038's entry.
- One measured, uncovered hole closes. The existing control
  (`a_release_over_an_unprotected_target_still_constructs`) covers the
  false-refusal direction on a partition; nothing covered the
  false-`Clear` direction on a disk.
- Zero closure change means zero MODEL-003 exposure, superset by
  identity, no schema-version question, and no new authored body field.
- Two durable structural facts are recorded for later rounds: **(a)** on
  a whole-disk target reach is entirely range-driven, because a
  self-framed extent is never a descent source; **(b)** no §2.1
  aggregate can carry an extent, so **any relief predicate keyed on
  `facts.extents` is structurally inert on exactly the class it must
  relieve.** (b) disqualifies a whole family in advance.

**Negative, accepted knowingly**

- **The availability gap above is real and stays open.** It is filed,
  not settled, and this ADR must not be read as having priced it.
- **A misleading asymmetry survives untouched.** `Move` refuses where
  `Copy` is `Clear` — but `Copy`'s `Clear` comes from the Source-class
  short-circuit, not from lack of reach (its closure reaches the pool
  identically). A reader concluding "copy instead" has been misled by a
  boundary this act does not examine.
- **The pin's `affected_set` assertion is currently unexercised.** The
  range-class and gate assertions trip first under every mutation tried.
  It is not redundant in principle — it is what stops the pin encoding
  `canonical_ranges`'s known over-claim rather than the property — but
  it should be recorded as an unexercised guard, not counted as
  coverage.

## Verification

- Every figure in the tables above, on the committed fixtures.
- **The pin was proven to bite, not merely to pass.** Under M3 it is the
  **only** red (116 passed, 1 failed, real exit 101); under M1 it is
  again the only red, failing on the range-class assertion. M1 is the
  mutation the round's own candidate pin declared in advance that it
  would survive; asserting the range class is what closes it.
- Mutations applied with `Edit`, proven applied by printed diff before
  each run, and reverted; the post-revert diff is one test file.
- `cargo test`'s real exit code read directly, never through a pipe.

## What stays open

1. **The availability gap** — filed as its own issue.
2. **The byte-wise-preservation duty is delivered nowhere**, and has no
   vehicle: `Move` is `NotRepresentable` in the planner and no
   destination operand type exists. Filed as its own issue.
3. **`canonical_ranges`'s table-write over-claim (issue #353) interacts
   with this pin.** Measured this round: correcting the written arm to
   §2.1's "never the parent device wholesale" moves **six whole-disk ZFS
   gates from `Unsupported{Zfs}` to `Clear` over a live pool**, with the
   committed suite green including this pin. Recorded on #353; this pin
   covers the `Move`/`Shrink` half only and must not be read as covering
   the whole-disk case.
4. **The #347 interaction is unmeasured here.** This act's worktree is
   at main and carries no `released` clause. Whoever lands #347 should
   confirm nothing here implies a release path for `Move` that its
   destroyed-only gating does not provide.

## Revisit conditions

- **PART-005 acquires a destination vocabulary.** `canonical_ranges`
  gains request parameters, the closure can be told a destination by a
  first-party producer rather than an author, and a preservation proof
  becomes conceivable — though R2 stands until an aggregate can be
  relieved without going through `facts.extents`.
- **Issue #353 lands.** Re-run this pin against it before either merges.
  If both of the pin's assertions go red, a whole-disk `Move` genuinely
  stops reaching the pool once the over-claim is corrected, and this
  ADR's "the release entry stands" verdict must be re-argued from the
  corrected table.
- **ADR-0037's anchoring enforcement lands.** Necessary but not
  sufficient: it addresses R3's lever, not R2's structural inertness.
- **`may_carry_extent`'s list changes**, or a relief channel appears
  that does not key on `facts.extents`. Either reopens R2's family.
