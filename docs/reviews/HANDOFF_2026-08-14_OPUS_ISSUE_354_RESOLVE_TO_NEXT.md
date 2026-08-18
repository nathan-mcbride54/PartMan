# Handoff — 2026-08-14, the flake fix and #354's resolve-only half

**From:** Claude (Opus 5), the session Nate directed with "assess the
current progress and take the next logical step", then through the
carried-over flake fix and #354's resolve-only landing.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-14_FABLE_TO_NEXT.md`, whose §5 named #354
as the clear next step. That reading held up.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at **`b3de0cf`**, **spec 13.0.0** (unchanged — see §3 for why this
was not a spec change). `cargo xtask ci` on merged main: **exit 0**, 603
annotations over 85 requirements, 639 live tests. WP-020 re-pinned at
**`86db930`** after the r19 sitting; `git diff --name-only 86db930 HEAD`
must list Markdown only, and does. **Nothing is owed and nothing is in
flight:** no branch, no open PR, no VM.

Open issues: **#318**, **#319**, **#333**, **#347**, **#348**, **#349**,
**#353**, **#354** (partially discharged — *not* closed), **#356**,
**#360**. Nothing in flight: no branch, no open PR, no VM.

**`D:\pm-354` still exists**, detached at the old `7fdba38`, carrying an
uncommitted 49-line print-only probe. It is superseded by what landed and
can be removed.

## 1. What this session did

| PR | What |
| --- | --- |
| #361 | fixtures: the two out-of-root decoys carried fixed names |
| #362 | WP-010: naming referents must resolve (issue #354, partially) |
| #363 | WP-060: the destruction closure and the naming sweep read one roster |
| #364 | WP-020 r19 re-pin: the record sweep and the new pin at `86db930` |

Comment added to **#354** recording the partial discharge, what each of
the three decisions that issue named was resolved to, and why it stays
open.

## 2. The thing worth carrying above all others

**A green workspace was, again, worth nothing on its own.** #362's sweep
passed 639 tests on first run with no committed test moved. Three
mutations were run, each applied with `Edit` and proven applied by
`git diff` before running. One of them changed the work:

> Dropping the `Partition` arm from the roster **survived**
> `every_naming_referent_must_resolve`.

`one_of_each` names five distinct addresses across eight referent fields
— two of them name the table — so a per-*address* enumeration lets an arm
be dropped silently. `the_naming_referent_roster_is_pinned_per_kind`
exists because of that measurement, and pins all eleven kinds by field.

**The general shape, which is the fourth session in a row to find it:**
an enumeration test is only as strong as the thing it enumerates over.
Enumerating the *values a fixture happens to produce* is not enumerating
the *cases the code distinguishes*. Ask which one you wrote.

## 3. The three decisions #354 said a fix must make, and what each became

1. **Where the check lives** — `Topology::build`, before any edge is
   read. Verified structurally rather than by reading prose:
   `Topology` has private fields and exactly one `Self {}` construction,
   so `build` is the sole constructor, and both `from_canonical_body` and
   `simulate` route through `assemble` into it. No bypass.
2. **What "valid" means** — **resolve-at-all only.** Resolve-to-kind is
   held behind **#360**; agree-with-the-containment-edge is untouched and
   still intersects #333.
3. **MODEL-003** — taken under the **explicit-rejection** limb,
   `SCHEMA_VERSION` left at **1**, **no spec bump**. Byte format and
   parse rules untouched (`fields_from_map` accepts exactly what it
   accepted); the refused population was never lawful under MODEL-002,
   only unvalidated; bumping would make every existing v1 body
   undecodable, golden vector included — a migration cost with nothing to
   migrate. Evidence that no conforming artifact changed meaning: the
   golden vector and all 639 prior tests are unmoved. **This is the
   decision most open to being overruled**, and it is written out in the
   CHANGELOG rather than buried, deliberately.

## 4. The finding that made the WP-060 half more than tidying

The panel argued unification because `destroyed_closure` closes over the
roster. True, but **the risk runs backwards from the obvious**, and this
was measured:

| with the `Volume` arm dropped from the roster | result |
| --- | --- |
| whole planner suite | **51 of 51 green** |
| domain sweep, same tree | refused the plan outright |

A closure gap does not produce a slightly wrong prediction once the sweep
exists. It produces a hard `SimulateRefusal::Assembly` that kills the
whole plan. `Volume` and `EncryptionLayer` are the only referent-bearing
kinds that may carry no extent, so the referent walk is the *only* thing
that reaches them — the layer had coverage, the volume had none.
`a_produced_volume_is_removed_with_its_producer_and_the_rebuild_stands`
asserts the rebuild **stands**, not merely that the volume is gone,
because refusing is the failure being pinned.

## 4b. A second kind-check candidate was proposed and rejected

After the sitting, I read ADR-0037:217 (not satisfied — §5.3 below) and
proposed a narrower kind check: four `(node_kind, field)` pairs whose
lawful referent kind was claimed "fixed by MODEL-002's chain". It was
green on 645 tests. An adversarial round killed it; record in
`ISSUE-354_FIXED_KIND_ROUND_2026-08-14.md`, summary comment on #354.

**The fatal:** `volume.producer => [aggregate, encryption-layer]`
false-refuses every host-backed virtual device (loop, VHD/VHDX,
dm-linear, plain dm-crypt), because such a volume's producer is the
`BackingExtent` carrying its bytes. The proposed set was **strictly
narrower than the producer set the product already ships** —
`producer_verdict` folds over `Production | HostBacking`
(`protection.rs:534`) and the pair table admits `backing-extent → volume`
(`topology.rs:315`). Filed as **#365**.

Three lessons worth more than the candidate:

- **Doc comments are not the spec.** The design cited `naming.rs`
  comments; ADR-0019's normative naming map deliberately names no kind
  for Volume/producer, and the adjacent variant doc contradicted the
  field doc.
- **A green suite over the committed fixtures cannot see a defect the
  fixtures have no instance of.** Every committed `Volume` names an
  aggregate or an encryption layer; `one_of_each`'s single volume wears
  both producing relations at once. 645 passing tests were structurally
  blind.
- **A self-identified weakest link is what to attack, not what to ship
  with a caveat.** The write-up called this pair "the weakest of the
  four" and proposed it anyway. Every lens killed it there first.

**Do not re-derive this candidate.** The only measured constructive path
for a producer check derives the set from
`endpoint_pair_allowed(Production|HostBacking, k, "volume")` — which is a
pair-table-derived check, the shape the panel already rejected, and it
reinherits #360.

## 5. What remains open

1. **#354's kind half**, blocked on **#360**. Deriving it from
   `endpoint_pair_allowed` is still the right idea — the only version
   with no second list to drift — but the table must cover the real
   population first. Three standing controls
   (`honest_layouts_the_kind_check_would_have_refused_still_build`) fail
   if it leaks in early, and
   `a_wrong_kind_referent_still_builds_and_that_is_the_held_half` is the
   test that must be deliberately changed when it lands.
2. **#360** — the pair table cannot express a partitioned mdraid array or
   a partition table inside a mapped volume. **Probed this session; three
   measured findings, recorded on the issue:**
   - The **re-proof obligation does not bite**. ADR-0018:210-217 states it
     over "any **edge kind** it adds", and the delivered theorem test
     quantifies over `Backing`/`Production`/`HostBacking` only —
     `Containment` is excluded because its descent is range-bounded. A
     new *row* on an existing kind preserves the premise trivially.
   - **One row suffices, not two.** `("volume","partition-table")` models
     a partitioned mdraid as `aggregate --Production--> volume
     --Containment--> table --Containment--> partition`, keeping
     aggregates out of the containment forest. It is the missing third of
     a set already two-thirds present. With it, the workspace is 646/646
     — which proves only non-regression, since a body using a new pair
     was previously unconstructible.
   - **It must not land yet.** The newly-representable population
     under-protects: wiping the array's only substrate reaches
     member→sig→array→volume→table and **stops** — the partition
     survives. Control on an ordinary disk shows the same gap, so the
     cause is **#347**, pre-existing, not the row. Landing the row first
     would ship representation that builds and silently under-reaches.

3. **The measured dependency chain, newly established:**
   **#347 → #360 → #354's kind half → #333's enforcement.** #347 reads as
   a self-contained closure defect and is in fact the head of the queue
   for this whole family.
3. **#333's enforcement — the ADR-0037:217 question is now answered:
   NOT satisfied.** `:217`'s "**the** capture-side referent sweep" is the
   one defined at `:146-150` by the harm it prevents — "a naming-derived
   frame can be computed from a pairing the pair table forbids". Resolve-
   only refuses a referent resolving to *nothing* and never asks what one
   resolves *to*, so that harm is untouched. The literal reading ("some
   sweep exists") makes the sentence vacuous. The precondition is the
   **pairing** check, which needs the pair table right first — so **#333
   is gated on #360**, and for ADR-0037's own derivation path
   (`partition --parent_table--> table --parent--> root`) the single open
   hop is **`PartitionTable.parent`**. Argument in
   `ADR-0037_PRECONDITION_READING_2026-08-14.md` §1–2, which stand; its
   §3 premise and §6.3 recommendation are **withdrawn** (see §4b).

4. **#365** — the host-backed producing relation is under-represented
   outside the pair table and `protection.rs`: a wrong doc comment, no
   committed fixture, and the suite blindness that let §4b's candidate
   through. Filed this session.
4. **#347**, **#319**'s authorization half, **#356**, **#348**, **#349**,
   **#353**, **#318** — all untouched this session; see the previous
   handoff, which remains accurate on each.

## 6. The r19 sitting: run, passed, and re-pinned — no debt outstanding

**Discharged.** VMID 9442, 2026-08-14 UTC, all three acceptances re-taken
on `86db930`, re-pinned by PR #364. `git diff --name-only 86db930 HEAD`
lists Markdown only. Identical value sets to r18: 2e passed;
`fixtures_executed=1`/`ranges_written=1`/`contracted_bytes_written=8` for
2h; `=1`/`=2`/`=16` for 2j; both 2j ranges restored with
`unchanged_outside_contract=true`; eleven controls refused, zero
unexpected passes; custody agreeing across guest, host and workstation
(run 27); teardown verified with nothing remaining.

**Three findings from it, all in the record:**

1. **Void first invocation** (run 26, retained). The sitting script must
   be launched by its **absolute** path: it sets
   `SELF="$(readlink -f "$0")"` *after* `cd "$WORKDIR"`, so a relative
   `$0` re-execs as `/root/partman/05-…sh` and exits 127. Nothing was
   measured.
2. **A rollback is a reboot.** Recovering to `pre-acceptance` brought the
   guest up on 5.15.0-187-generic, not the -186 it provisioned on —
   unattended upgrades had staged it. That invalidated the
   provisioning-time environment record, which was regenerated on the
   actual run kernel from the provision script's own lines.
3. **A test-only merge trips this.** #361 was two fixture filenames and
   re-opened all three acceptances. The stopping-condition paragraph now
   says so at the point the reader is most tempted; the exemption is
   declined at `WP-020.md:1174-1176`.

**One judgment to review rather than inherit:** this arc is the first to
span both sides of the trip counter — #361 is WP-020's own
`crates/fixtures`, #362 and #363 are outside it. The outside count was
advanced to fourteen on #362/#363's account and #361 recorded as an
inside trip within the same arc, rather than incrementing "outside" for
all three. That keeps the counter meaning what it always meant, but it is
a decision, not a derivation.

## 7. Operational notes

- **The stale-rollup trap, caught live.** After `gh pr update-branch` on
  #363, the API reported **12 passed, 0 pending** — every one of them a
  check against the *previous* head. `mergeStateStatus` was `UNKNOWN`,
  then `BLOCKED`, and the true state was 11 `IN_PROGRESS` + 1 `QUEUED`.
  **Compare the rollup against `headRefOid`**, or a stacked PR merges on
  its parent's evidence.
- **One PR, one work package.** `verify-change-ownership` refuses a
  change declaring two, so a change spanning `crates/domain` (WP-010) and
  `crates/planner` (WP-060) is two PRs, stacked. CI judges ownership
  against `origin/${{ github.base_ref }}`, so the stacked child passes
  while based on its parent.
- **A commit carried over from a previous session had no
  `Work-Package:` trailer** and looked finished. Run the ownership gate
  on anything you did not commit yourself.
- **Annotation blocks must be contiguous.** A bare `//` line inside a
  `Requirements:`/`Evidence:` block splits it in two, and the first half
  fails traceability for having no `Evidence:`. Regenerate the maps last.
- `cargo xtask fmt` and clippy's `redundant_closure_for_method_calls`
  both fired only at the full-gate stage; `cargo test` is not a
  pre-check for either.
- **`does not close #NNN` auto-closes #NNN.** PR #362's body carried the
  heading "Resolve-only, and this does not close #354"; merging it closed
  #354, which is precisely what the panel forbade and what the sentence
  existed to prevent. GitHub's parser matches keyword-plus-number and
  ignores the negation. Reopened, with the reason recorded on the issue.
  This is the **inverse** of the trap already in the repo notes (`Closes
  issue #NNN` does *not* fire, because the number must be adjacent);
  adjacency is all the parser reads, in both directions. On a partial
  discharge, keep closing verbs away from the number entirely — and
  **check the issue's state after merging**, rather than assuming the
  wording carried.

## 8. Session evidence

Scratchpad holds the concurrency logs for #361 (24 four-up runs), the
mutation-pass outputs, every gate log with its real exit code, and both
PR bodies.
