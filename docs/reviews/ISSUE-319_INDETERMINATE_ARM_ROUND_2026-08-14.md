# Issue #319: the Indeterminate arm — measured, then rejected. Round, 2026-08-14

Untracked session artifact, `docs/reviews` convention. Everything
load-bearing must be restated in whatever ADR lands the decision.

> **HEADER NOTE, WRITTEN AFTER THE ADVERSARIAL ROUND. The candidate in
> this document is REJECTED.** Sections 1–5 record the measurement as it
> was taken and are preserved because the numbers are real and the
> negative controls stand. **Section 6 records six defects, every one
> reproduced by hand, and two factual errors in this document's own
> earlier claims.** Do not rebuild the arm from sections 1–5 without
> reading section 6 first. The recommendation at the end of the original
> draft — "adopt the refined arm" — is withdrawn.

**Follows** `ISSUE-319_AUTHORIZATION_ROUND_2026-08-14.md`, which rejected
the point-reach candidate on four fatals and left one arm standing,
unmeasured: *refuse rather than reach — an extent-bearing node that
declares no bytes in the step's frame makes the closure's answer unsound,
so the step is `Indeterminate` rather than `Ok`.*

Everything was run by hand against `main` at `8e03e68` (spec 13.0.0) in a
detached worktree (`D:\pm-wt-319`) with its own target directory. Every
variant's application was proven by `git diff --stat` before its run, and
the load-bearing probes were run against HEAD's own `protection.rs` to
confirm they go red without the arm. **No `cargo xtask ci` run backs any
of this** — `cargo test` only. Issue **#354** (naming referents validated
by nobody) was filed first, from this worktree's probes.

## 1. The arm, in two formulations — and the naive one is dead on arrival

**Naive form:** every `may_carry_extent()` kind in a forest the step
touches must carry an extent fact. **Measured cost: 18 committed tests
fail** — 1 domain, 4 capability, 13 planner — and two are committed
statements of intent, not fixture accidents:

- `crates/planner/src/tests.rs:580-593` asserts that "the fixture's GPT
  table node carries a containment edge and no extent" **by
  construction** — ADR-0036 locates tables by their hashed scheme.
- `an_ordinary_disk_keeps_its_siblings_out_of_the_set` (PR #351's guard)
  deliberately gives its ESP **no extent fact**.

A form that fails those two contradicts two decided acts. Retired.

**Refined form:** a node located by an authenticated route other than an
extent fact is exempt — `PartitionTable` (hashed scheme) and `Partition`
(hashed `start_offset`, the one naming field whose address space is
documented). Everything else that may carry an extent must be
**anchored**: its extent's frame chain must ground out in the forest's
root.

## 2. What the refined arm closed — measured, with HEAD controls

| layout, device target | HEAD (`8e03e68`) | refined arm |
| --- | --- | --- |
| **extentless-frame hole** (ZFS label framed by the member, member unlocated) | **all ten `Clear`** — over a live pool | all ten `Blocked{MissingFact}` |
| **whole-disk vdev, signature unlocated** | **all ten `Clear`** — over a live pool | all ten `Blocked{MissingFact}` |

The second row is a **new measured shape of #319's defect** found by this
round and noted on the issue: on a whole-disk vdev the signature is
reached only by the byte scan, so removing its extent fact alone opens
all ten gates over a live pool at HEAD. That finding is independent of
the arm and survives its rejection.

**Controls that stand.** With `protection.rs` restored to HEAD the two
arm probes go red and every other probe passes. Under the arm these are
byte-identical to HEAD: pool layout (device and member targets), ordinary
disk (device and partition targets), whole-disk vdev with a located
signature, and the ordinary disk carrying an unlocated partition.

## 3. The availability cost, measured

**The 2026-08-13 objection 4 shape is gone.** An ordinary disk with an
unlocated `Partition` gates all ten `Clear` on both targets; the #351
guard passes unchanged.

**Committed workspace under the refined arm: 4 failures**, all in
`partman-capability`'s engine tests, all from one unlocated XFS
`FileSystem` in `engine_tests::fixture()`. Domain 112/112, planner 51/51.

**This accounting is corrected in section 6.** One of those four is
`the_engine_and_the_plan_constructor_agree_on_every_pair`, and it is not
a displaced expectation — it is a real disagreement between two
surfaces, which the proposed fixture regeneration would have made green
**vacuously**.

## 4. The #353 interplay — orthogonal, measured both directions

- The arm does not rescue #353: with its correction stacked on, the six
  gates open on located pool layouts (`Unsupported{Zfs}` → `Clear`).
- #353 does not refund the arm: both hole layouts stay `Blocked` under
  arm + correction, since the arm's scope is the target's forest
  regardless of declared ranges.

**Incidental finding for #353:** under the maximal correction the planner
test `an_unordered_overlap_refuses_with_both_steps_named` fails —
plan-level overlap detection loses the write claim it keyed on. A
truthful `Create` entry must claim the host's table extents and the
consumed range, which is #353's decision point 1 with a committed test
now naming it.

## 5. The three questions from the prior round

1. **Availability cost** — measurable and small *for the population the
   arm actually examined*; section 6's hybrid finding adds a population
   it did not.
2. **File the referent gap first** — filed as **#354**.
3. **#353 sequencing** — free; the acts are orthogonal.

---

## 6. THE ADVERSARIAL ROUND — the candidate is rejected

Six lenses, 28 findings, 6 fatal and 11 serious as filed. **The verify
phase of the first workflow returned nothing: all eight of its skeptics
died on a model quota error, and the run reported an empty survivor
list.** An empty survivor list produced by a verify phase that never
executed reads exactly like a clean bill of health — the
gate-that-examined-nothing failure, here in the harness built to check
the harness. The findings were therefore re-run **by hand**, and then a
second verification round of 17 skeptics (one per fatal/serious finding)
was run to completion.

**Verification outcome: 17 of 17 survived, none refuted, 16
independently measured by the skeptics.** Four were **downgraded**, and
two of those downgrades correct claims *I* made in the first draft of
this section:

| finding | filed | verified | note |
| --- | --- | --- | --- |
| wrong boundary (6.1) | fatal | **fatal** | measured at three surfaces |
| bogus extent (6.2) | fatal | **minor** | my framing refuted — not the arm's class |
| decoy parent (6.3) | fatal | **serious** | HEAD-identical; contained repair |
| hybrid disks (6.4) | fatal | **serious** | my "no honest extent" claim refuted by the type |
| exemption hole (6.5) | serious | **serious** | stands |
| backing extents (6.6) | serious | **serious** | stands; my coverage claim was false |
| bare partition | serious | **minor** | ADR-recording item on HEAD behavior |

**The rejection does not depend on the downgrades.** 6.1 is fatal on its
own, measured by me and independently by two skeptics at three surfaces,
and it breaks a committed CAP-005 property.

### 6.1 FATAL — the arm is at the wrong boundary

`PlanStep::mutating_declared` (`step.rs:417-441`) runs its **own**
`affected_set` + `node_verdict` loop and never calls `step_constructs`.
The only non-test callers of `step_constructs` are `capability.rs:202`
(`protection_gate`, the advisory surface) and the `snapshot.rs:144`
wrapper. The arm was placed at the tail of `step_constructs`, so **it
never runs at the sole step constructor** — the same boundary ADR-0039's
context singles out as having no capability gate in front of it, and the
same one the 2026-08-14 handoff named: *the measurement that mattered was
at `mutating_declared`, not at `affected_set`.*

**CONFIRMED by hand:** on the hole body, `protection_gate(Wipe, device)`
answers `Blocked{MissingFact}` while `PlanStep::mutating` with the
identical canonical ranges **constructs**, affected = 3. Section 2's
"both live holes close" is true **only at the gate surface**. This also
breaks CAP-005 agreement-by-construction: the advisory answer is
*stronger* than the authoritative one, which is the wrong direction.

### 6.2 FATAL — a bogus extent respells the hole, and the arm is the lever

The arm closes only the **extentless spelling**. Add one device-framed
extent to the hidden node, at an offset disjoint from the destroy range:
`is_anchored` now reports it **anchored**, the committed closure's
`descends_into` blocks the containment hop on geometry, and the step
returns `Ok`.

**CONFIRMED by hand, at every boundary:** a partition delete over a live
pool constructs, affected = 2, pool unreached, **arm silent**. The body
**assembles, encodes, decodes and rebuilds with agreeing hashes**, and
the **decoded** snapshot's own closure approves the same delete
(`probe_the_contradictory_extent_body_decodes_cleanly`). Nothing
anywhere cross-validates a containment edge against the extent facts:
the edge says the signature sits inside the partition, the fact puts it
400 MiB past the partition's end, and no layer objects.

> **CORRECTED after the verification round — this finding is downgraded
> to MINOR, and my first reading of it was wrong.** The skeptic refuted
> the framing, and the refutation checks out. In this body the
> signature's extent is `{host: device, start: 500 MiB}` and the device
> **is** the forest root, so the node *does* declare bytes in the root
> frame — it is **outside the arm's stated population by the arm's own
> rule**, not hidden by a blind spot in it. The escape belongs entirely
> to `descends_into`'s `(Some, Some)` clause — the bounded descent pinned
> by spec 13.0.0 / ADR-0039 — which this arm neither touches nor claims
> to cover, and which ADR-0039:158-161 already assigns to issue #349 as
> an accepted, filed negative. Control measured by the skeptic and
> consistent with my own run: **remove that extent and the arm fires.**
> So the added fact does not reopen a hole the arm had closed; it
> exhibits a **third, differently-caused body**, and the outcome is
> byte-identical to HEAD.
>
> **Retracted:** my claim that "the arm's invariant is false against
> itself" and that the arm "is the lever." Neither is supported. The arm
> only ever adds refusal, at HEAD-identical outcomes on this body.
>
> **What survives, and it is still worth carrying:** nothing anywhere
> cross-validates a containment edge against the extent facts. The edge
> says the signature sits inside the partition; the fact puts it 400 MiB
> past the partition's end; the body decodes and the decoded closure
> approves the delete. That is a real gap, adjacent to but distinct from
> #349 (which covers extent *well-formedness* — zero length, ghost host,
> overflow — not *agreement between edges and facts*). See 6.7.

### 6.3 SERIOUS (downgraded from fatal) — `containment_root` is steerable

`containment_root` resolves a forest root by first-matching-edge. A node
may legally carry **two** containment parents — `Topology::build` refuses
self-edges and byte-identical duplicates but enforces no in-degree limit,
and both `("physical-device","backing-signature")` and
`("partition","backing-signature")` are in the pair table. An attacker
grinds a decoy device serial until its address sorts below the real
parent's, adds one legal edge, and the hidden node's computed root moves
to an untouched forest — so `roots.contains(&root)` is false and the arm
skips it.

**CONFIRMED by hand:** with the decoy edge added, the whole-device wipe
over the live pool **constructs**, affected = 3. The arm's single
headline closure, reopened by one edge on a body that builds.

**Downgraded to serious on verification, and the grounds are fair:** the
outcome is HEAD-identical (the arm still only adds refusal, so nothing
committed is lost), the shape needs an adversarially authored second
parent plus a ground serial rather than #319's honest-absence class, and
the repair is contained — fire if **any** containment parent's root is in
`roots`, or give `Topology::build` a single-containment-parent rule.

**But it led somewhere worse: see 6.9.** The identical first-match walk
is already load-bearing in committed code.

### 6.4 SERIOUS (downgraded from fatal) — hybrid disks refuse everything, including their repair

A `ConflictingTableEntry` is in the strict population, and no committed
fixture stamps an extent on one. Its containment root is the device,
which is in `roots` for any target on that disk.

**CONFIRMED by hand:** on a hybrid GPT/MBR disk, **all ten** mutating
gates answer `Blocked{MissingFact}` on the partition target, and `Repair`
on the device target likewise — including ADR-0024's repair family, the
operation whose purpose is to fix the hybrid conflict. The planner
records the contrary intent in its own words
(`planner/src/tests.rs:3009-3011`: the entry "carries no length, so no
bound over it is computable, and ADR-0024's repair family stays reachable
on a repairable device").

> **CORRECTED after verification, and the correction is one of this
> project's own rules used against me.** I wrote that "no extent can be
> honestly stamped" and derived it from the naming fields. **The
> delivered type refutes that:** `may_carry_extent` (`naming.rs:750-758`)
> *permits* an extent on a `ConflictingTableEntry`, and the decode path
> admits one (`snapshot.rs:421` refuses an extent only when
> `!may_carry_extent()`). `Facts.extents` comes from the evidence
> contract's byte layer, not from the naming preimage; the planner
> comment governs occupancy derivation from naming fields, not what the
> byte layer may stamp. Measured by the skeptic: **stamping a CTE extent
> restores all ten partition-target gates.** So the remediation *is*
> deliverable in kind. **Structural claims come from the delivered type,
> never from spec or comment text** — the rule I already had written
> down, and did not apply here.
>
> Realism is also lower than I claimed: no committed production path
> constructs a CTE node. **Downgraded to serious** — a real over-refusal
> the ADR must price, not a defect that sinks the design on its own.

### 6.5 SERIOUS — the `Partition` exemption lets the hole class through

The round justified the exemption with "its subtree, if any, fires the
arm through the strict frame chain." That holds only when the content is
framed by the unlocated partition. Attach the live pool's label under the
**device**, device-framed — where a byte scan would put it — beside an
extentless partition, and target the partition.

**CONFIRMED by hand:** all ten mutating gates `Clear` with the signature's
own verdict `Refused{InheritedFromConsumerOrProducer}`; the device-target
control correctly refuses. Releasing that partition releases substrate
nobody bounded. This is a HEAD hole the arm does not close rather than a
regression, but it destroys the exemption's stated justification.

### 6.6 SERIOUS — a coverage claim in this document is false

Section 3's residual class named "unlocated signatures, filesystems,
**backing extents** and conflicting table entries." **No containment pair
targets `backing-extent`** (`topology.rs:244-254`); it appears only as a
`HostBacking` source. So a `BackingExtent` is always its own containment
root, `is_anchored` short-circuits true at `id == root`, and the `roots`
filter would skip it regardless. **Confirmed by reading the pair table.**
Loop-file stacks are a third live hole the arm does not close, and a
`Path`-locator extent plausibly never carries a range fact at all.

### 6.9 THE MOST IMPORTANT RESULT — a protection bypass in committed code

6.3's decoy-parent trick was chased one step further by a skeptic, and it
lands on **HEAD, not on the candidate**. `device_scope_verdict`
(`protection.rs:527-541` at `8e03e68`) walks reverse containment to the
root with the **same first-matching-edge** `find`. A node may carry two
containment parents. So which device's scope verdict a node inherits is
decided by **which parent's `NodeId` sorts first** — grindable, because
`derive_id` hashes a `PhysicalDevice`'s serial.

**MEASURED against pristine HEAD** (`protection.rs` reverted, empty
`git diff --stat`, `probe_head_device_scope_is_steerable_by_a_decoy_parent`):

```
HONEST  (ext4 filesystem contained by a RecognizedRemote device):
        Refused { ground: InheritedDeviceScope }
ATTACK  (one extra legal containment edge from a decoy SATA device):
        Permitted
STEERED: true
```

**One added edge turns a remote-transport refusal into `Permitted`, in
committed code, on a body that builds.** ADR-0018's device-scope arm is
the closed positive local list whose whole purpose is that everything not
positively local fails closed; here the *inheritance* of that verdict is
steerable by authored topology. This is the "safety is computed, never
declared" discipline defeated at the computation itself.

**Filed as #355**, widened after further measurement: the same
first-match selection appears in **three** arms, and two of them are
end-to-end gate bypasses. `producer_verdict` flips **all ten** mutating
gates from `Unsupported` to `Clear` on a volume produced by a live ZFS
pool, via one added `Production` edge. `own_arm`'s consumer selection
steers the verdict but the gate holds, because `affected_set` enumerates
every consumer — defence in depth catching the one case where a second
enumeration exists. The bodies decode cleanly and the decoded snapshot's
own `node_verdict` returns `Permitted`.

It is **not** a consequence of the rejected arm — the arm made it
visible — and it is plausibly more urgent than #319's remaining half.

### 6.7 What survives the rejection

- **The defect stands**, in three measured shapes now: the
  extentless-frame subtree, the whole-disk vdev with an unlocated
  signature, and the device-attached label beside an extentless partition
  (6.5). All three approve destructive work over a live pool at HEAD.
- **Any fix must live in the law both surfaces share** — inside
  `mutating_declared` or a helper it and `protection_gate` both call.
  A check that only `protection_gate` runs guards the advisory surface
  and leaves the authoritative one open.
- **A predicate that reads unauthenticated body content to decide whether
  to refuse can be silenced by authoring that content.** The test to
  apply to the next candidate is not only "does it add refusal against
  HEAD" but **"can any authored field remove the refusal it just
  added?"** 6.3 is that question answered yes, via a naming field's
  address ordering. (6.2 is *not* an instance — corrected above.) 6.9
  shows the same question already has a "yes" at HEAD.
- **The root defect underneath all of this is that the body's structural
  content is never validated against itself.** Four filed faces now:
  a containment edge saying the signature is inside the partition while
  the extent puts it 400 MiB past its end (**#356**, filed from 6.2's
  surviving half); naming referents validated by nobody (**#354**); the
  body boundary accepting zero-length, ghost-hosted and overflowing
  extents (**#349**); and unvalidated structural multiplicity resolved
  by sort order (**#355**, from 6.9). **These are one defect wearing four
  numbers**, and a validation act may retire more of #319 than any
  further closure predicate. Every closure predicate proposed across
  three rounds has died on unauthenticated body content.
- **Derived position remains unavailable in general** (prior round), so
  the geometric family stays retired.

### 6.8 Process notes

- **The first workflow's verify phase examined nothing and reported an
  empty survivor list.** Never read an empty result as a negative result
  without checking the agents ran; the journal (`journal.jsonl`) records
  six results for six finders and zero for eight verifiers.
- Every finding above was re-run by hand. The workflow's value was
  breadth — it found the boundary defect I had missed twice, having been
  warned about that exact boundary in the handoff I read at the start of
  this session.
- **Worktree state as left:** `D:\pm-wt-319` has `protection.rs` reverted
  to pristine HEAD (the arm is rejected, and 6.9's probe requires HEAD),
  with all probes still in the three test files. The armed
  `protection.rs` is preserved at `scratchpad/protection-armed.rs`; drop
  it back in to re-run 6.1–6.6. `gates-head.txt`, `gates-v2.txt`,
  `gates-v3.txt`, `fatals-measured.txt`, `findings.json` and
  `verdicts.json` are in the session scratchpad.
- **The second verification round cost ~2M subagent tokens across 17
  agents and refuted nothing, but corrected two of my own claims and
  produced 6.9.** Verification earns its keep by correcting the
  verifier, not only by killing findings.

## 7. Recommendation

**Do not reserve an ADR for this arm.** It never runs at the
authoritative boundary and breaks a committed CAP-005 property there
(6.1 — fatal, and sufficient on its own); it is silenceable by an
authored second containment parent (6.3); it blocks every operation on a
hybrid disk including the repair that fixes it (6.4); its exemption
admits the hole class it was built to close (6.5); and its stated
coverage of backing extents is false (6.6).

**Two things I claimed in the first draft of section 6 were wrong**, and
both were caught by the verification round rather than by me: the
bogus-extent body is outside the arm's population by the arm's own rule
(6.2), so the arm is *not* "the lever" and its only-adds-refusal
invariant holds; and a CTE *can* honestly carry an extent, which the
delivered type says plainly (6.4). The second is a direct violation of
this project's own standing rule that structural claims come from types.

The recommendation instead is to put one question to the decision owner
before any further closure work: **should the next act validate the body
— edges against facts against naming fields against structural
multiplicity — rather than add another predicate that reads it?**
#349, #354, #355 and #356 are four faces of that one defect, and every
closure predicate proposed for #319 across three rounds has died on
unauthenticated content. The alternative, if a closure route is still
wanted, must be designed against 6.7's test — *can any authored field
remove this refusal?* — and must live where both surfaces run.

**#355 is the one to look at first regardless of how #319 proceeds.** It
is a live bypass in committed code — ten gates from `Unsupported` to
`Clear` over a live ZFS pool on one added edge — and it does not wait on
any decision this round raised.

## 8. Filed from this round

| number | what |
| --- | --- |
| **#354** | naming referents validated by nobody |
| **#355** | the verdict computation picks one edge by sort order (HEAD bypass) |
| **#356** | containment edges and extent facts never cross-validated |
| #319 comments | the whole-disk-vdev shape, and the partition-target shape from 6.5 |
