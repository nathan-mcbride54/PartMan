# WP-020 increment 2i: the general destructive executor — audit and plan, 2026-08-11

Untracked session artifact (`docs/reviews/**`, WP-000). Follows
`WP-020_INCREMENT_2_AUDIT_AND_PLAN_2026-08-08.md`, whose two proposed slices
(2g, 2h) are both Delivered with operator-run acceptances. Increment 2's row
still reads "unblocked, and still not delivered": one read-only acceptance
plus one single-range destructive suite is not a destructive harness. This
document measures what is and is not general today, then proposes the next
slice. The plan is a proposal; nothing here authorizes code.

## Audit: where the generality already exists, and where it does not

**The registry is already general.** 2g compiled `Suite` as N fixture
contracts of N `IntendedChange` ranges each; `Admission::admit` validates the
general shape (catalogue membership, non-empty sets, replacement lengths,
bounds, overlap refusal) and — since #249 (PR #251) — binds it to exactly one
verified handle per declared fixture by counting, not set comparison. That
fix was named the prerequisite for this slice, and it is discharged.

**The executor is deliberately not.** `consume_admission` refuses anything
but exactly one fixture with exactly one range (`WrongSuiteShape`), and its
comment records the intent: "a suite that outgrows this shape gets a new
executor, not a widened one." The narrowness is not incidental — it was 2h's
reviewed boundary — so generalizing it is a new reviewed boundary, not a
widening in place.

**The protocol is single-range in three specific places.** (i)
`execute_destructive` takes one `expected_replacement` and drives one
pre-read, one write, one post-read. (ii) `DestructiveController` has singular
`read_declared_range`/`write_contracted` methods. (iii) The bracket is
`digest_outside_range` over one `(offset, length)`.

**Everything else is already general or indifferent.** The xtask suite runner
maps *all* declared fixtures into the interlock request and consumes the
admission; restoration (regenerate + `verify_on_disk`) is whole-tree; the
selector, refusal counts, and negative controls do not depend on suite shape.

**Constraints that bind, carried verbatim.** ADR-0007: an operator stays in
the trigger path; no unattended trigger. The 2e record: a destructive path
states its own pre-write discipline, inheriting nothing. The 2g/2h bright
line: no product write path, no storage-tool invocation, no domain types, no
plan or hash surfaces. And the standing price: any Rust change re-opens both
acceptances, so the slice budget includes a VM sitting.

**One stale row found while measuring (the sitting-lands-in-more-places rule
again):** WP-020.md delivery row "2" still says the registry's "shipped state
is empty: no destructive Tier-2 suite is registered", which 2h falsified.
Fix it in the same change that adds the 2i boundary.

## Proposed shape — two slices again

**2i — the executor becomes general, and nothing new can run.** Tier-1 only;
the registry keeps exactly the 2h suite, so no generic-refusal test changes
meaning and no new selector exists.

- `AdmittedContract` becomes N admitted fixtures, each with its held backing
  object and N compiled ranges. `consume_admission` states the general
  preconditions as its own: the suite must be registered (pointer identity,
  as today); at least one fixture; per fixture at least one range, every
  replacement non-empty and length-matching, ranges non-overlapping; exactly
  one verified target per declared fixture, matched by basename. Every
  refusal remains a refusal — the arity check is replaced by checks that are
  *stronger*, not absent.
- The pure protocol generalizes per fixture to N ranges: every declared
  range pre-read and required to differ from its replacement (per range —
  a suite may not ride one changed range past another that would prove
  nothing), one bracket over the complement of the union, attach → verify →
  forbidden-rebind probe → one write per range → `fdatasync` → re-verify →
  confirmed detach, then per-range post-equality and the unchanged bracket.
  On the 1-fixture-1-range shape this produces **exactly the event sequence
  the 2h tests pin** — that equivalence is itself a pinned test, so the
  general protocol provably contains the accepted one.
- Multi-fixture runs are sequential complete chains, with an executor-level
  pre-flight first: every fixture's held bytes must match the compiled
  catalogue before *any* fixture is attached, so a suite whose second
  fixture is wrong refuses before its first is touched. The pre-flight and
  the per-fixture ordering are pure-protocol facts driven through fakes at
  Tier 1, not Linux-only glue.
- The bracket generalizes to `digest_outside_ranges` (complement of a sorted
  non-overlapping union), with the existing single-range tests extended to
  multi-range shapes including ranges sharing a read chunk.
- Report: totals (fixtures executed, ranges written, bytes written,
  detachments confirmed) with the same allowlist discipline.
- Test obligations: every existing destructive-protocol test re-read at this
  edit (the `WrongSuiteShape` refusal changes meaning from "not 1×1" to
  "malformed general shape"); new ordering tests for multi-range and
  multi-fixture shapes; every new gate mutation-verified; the compile-fail
  entry-point proofs unchanged.

**2j — the second registered suite, behind its own boundary.** Not in 2i.
The candidate shape (to be decided at 2j): a two-range suite erasing both
GPT header signatures of `gpt-basic-512.img` (offset 512 and the backup
header signature at 4 MiB − 512), or a two-fixture suite adding
`mbr-basic-512.img`'s signature bytes. Registering it flips the compiled
edit-detectors, re-opens every generic-refusal test for re-reading, and
carries its own operator-run VM acceptance.

**Sequencing:** 2i Tier-1, mergeable on green → the merge trips the 2e
stopping condition (pinned at `68298f2`) → one VM sitting re-takes 2e and
the 2h suite, now executing through the general executor's 1×1 path on the
real kernel → 2i's row moves to Delivered on that sitting → 2j proposed
separately.

**What 2i must not contain:** a second registry entry, a new selector, any
change to `Admission::admit`'s semantics, any product write path,
storage-tool invocation, or domain type. The deliverable is that the
executor's shape stops being the reason a general suite cannot exist.
