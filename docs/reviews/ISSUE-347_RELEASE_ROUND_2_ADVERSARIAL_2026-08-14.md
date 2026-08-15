# Issue #347: round 2's candidate — adversarial round, 2026-08-14

Untracked session artifact, `docs/reviews` convention.

> **This file was rewritten after the panel completed.** My first version
> of it was written from findings recovered mid-run out of agent
> transcripts, before the verification phase had reported. Several of its
> headline claims were then **refuted by that phase** — notably "the
> mirror of round 1 §10.2", "the release has zero strength", and "the
> amendment argument is refuted by the delivered type". Those are
> corrected below. The verdict is unchanged: **REJECT**.
>
> What follows is the panel's own synthesis, kept verbatim because it is
> more accurate than my summary of it. 13 agents, 0 errors; every FATAL
> was handed to a separate refuting agent.

---

# ISSUE-347 — Table Release, Round 2: Panel Conclusion

**Candidate:** `work/wp010-347-release` @ 6e1706b (base main b3de0cf)
**Panel:** four adversarial lenses (over-reach, authored-lever, theorem, blast-radius); every FATAL handed to a separate refuting agent.
**Date:** 2026-08-14

---

## VERDICT: REJECT

Reject the design shape, not the goal. #347 is real, the release is the right instinct, and the candidate introduces **no fail-open path** — measured over 4200 (table-extent length x target x operation) rows the candidate's `affected` set is a superset of baseline's in every row, with 364 `Clear`->refusal transitions and **zero** refusal->`Clear` transitions. Nothing dangerous constructs that did not construct at HEAD.

It is rejected on three grounds that survived refutation:

1. **Measured sibling capture from a partition-target step, with no extent inflation at all.** On a disk whose first partition starts at LBA 34 (the parted/gdisk default) while the table declares the conventional `[0, 1 MiB)` — the exact convention the committed `root_on_zfs` fixture uses — destroying that partition captures every sibling and everything under it. HEAD 10/10 `Clear` on the `bios_grub` target; candidate `Shrink/Move/Encrypt/Wipe` = `Unsupported{Zfs}`, `affected(Wipe)` 3 -> 7 with ESP, member and pool all reached. Two guard-shaped assertions (`f11`, `f12`) pass at HEAD and fail under the candidate while all 21 committed `model::protection_tests` stay green — because the entire committed fixture population has `table.start + table.length == p1.start` exactly. That is the ISSUE-354 rejection ground verbatim: a green suite over a population containing no instance of the failing shape.

2. **The `conflicting-table-entry` half of the pair set is unjustified and has zero coverage.** Mutation: delete `"conflicting-table-entry"` from `describes_rather_than_nests` — **558 passed, 0 failed**. A surviving mutation on an uncovered clause is the shape ADR-0039 treated as a proposal blocker. (No refutation of this finding is on record.)

3. **The offered ADR-0018 theorem amendment does not discharge ADR-0018:210-217** (see below). An amendment that is measurably insufficient cannot satisfy a precondition of acceptance.

Landing requires a different trigger, not an edit. Every surviving objection attacks the same mechanism — `range_destroyed` membership decided by one-byte `HostRange::intersects` against `Facts.extents`, an unauthenticated field with no producer and no well-formedness rule — and the panel measured that no predicate over `Facts.extents` can be repaired into shape (see "The impossibility result").

---

## Findings that SURVIVED verification (live)

### L1 — FATAL, measured. Sibling capture from a partition-target step on overlapping table/partition geometry
Fixture `bios_boot_gpt` (root-on-ZFS plus a BIOS boot partition at LBA 34 = `[17408, 1 MiB)`; table `[0, 1 MiB)`).
```
gate[bios_grub]  HEAD      10/10 Clear;  affected(Wipe)=3, esp/member/pool all false
                 CANDIDATE Shrink/Move/Encrypt/Wipe = Unsupported{Zfs}; affected(Wipe)=7, all true
```
`f12` sharpens it: the destroyed range `[17408, 1 MiB)` contains **no GPT structure** (protective MBR `[0,512)`, primary header `[512,1024)`, entry array `[1024,17408)` are all outside it), yet the closure treats the table as destroyed. Probes at `crates/domain/src/model/adversary_probe.rs`.

Two corrections from the refuter, neither weakening it:
- **Citation.** ADR-0018:199-206's committed regression is about *creating* a BIOS boot partition, and creation still constructs under the candidate (`Create` declares `written_table_extents`; `the_root_on_zfs_regression_pair_holds` passes). The property actually violated is the same passage's "the ESP at `sda1` is never captured by its sibling's pool ... (containment descent is range-bounded)" plus ADR-0039:186-191. Same named layout, different operation — state it that way.
- ADR-0036's 1 MiB reserved floor is **not** a defense: PartMan reads disks partitioned by other tools and the decode path enforces no floor.

**Fix required:** separate "the table was destroyed" from "something inside the region the body attributes to the table was destroyed." The candidate has no such test.

### L2 — FATAL, measured. The one-byte price drop, end to end through `plan`
Honest step, `Wipe` on the ESP, root-on-ZFS layout, only the table's declared extent varied:
```
main       [0,1MiB) PLANS | [0,1MiB+1) PLANS | [0,1MiB+1KiB) PLANS | [0,257MiB) PLANS | [0,768MiB) Unsupported/ProtectionRefused{Zfs}/NoneExists
candidate  [0,1MiB) PLANS | every row from [0,1MiB+1) on: Unsupported / ProtectionRefused{Zfs} / Remediation::NoneExists
```
The inflated body **round-trips through decode** (`TopologySnapshot::from_canonical_body`); `snapshot.rs:380-462` gates extents only on `may_carry_extent`, and nothing compares a table's extent to its containment children (issue #356 verbatim). A 1-byte inclusive/exclusive-end slip in a capture adapter produces it without an adversary.

**Adjudication — this one split the panel 2-1 and must be read narrowly.** Three lenses filed it; two refutations succeeded and one did not. What was **refuted, measured**:
- *Novelty of the class.* Baseline b3de0cf has identical one-byte flips on other authored fields: the table extent at 768 MiB (`805306367 -> 805306368`), the ZFS signature's `extent_start` at `257 MiB -> 257 MiB - 1`, the ESP's length at `511 MiB + 1`. `range_destroyed`-by-`intersects` is HEAD's committed semantics for every extent-bearing node.
- *"Mirror of round 1 §10.2."* False, and inverted. §10.2 was fatal for being **anti**-monotone (inflation *removed* a refusal; fail-open). This is monotone — inflation *adds* one — which is round 1 §11 requirement 2 in as many words.
- *"HEAD `Clear` is the baseline of honesty."* False on the inflated geometry: HEAD itself puts the table in `range_destroyed` there and still lets `Wipe(GPT)` through over a live vdev. That column is #347 re-instantiated.
- *"An ordinary first-partition offset below 1 MiB."* Drop it. The cited fixtures establish a *no-overlap* convention, and the finding's own control (table tightened to `[0, 17408)`) is `Clear` on the candidate.

What **survives**: the **price and the amplification**. HEAD needs >=768 MiB of over-declaration for that refusal and its reach grows continuously with the over-declaration; the candidate needs 1 byte and, once `released` fires, captures the entire partition population and everything backing it regardless of how much was over-declared. Round 2 §3 rows 2 and 4 record the inflation lever as discharged having tested only the *removal* direction and only with the table as target — the house standard's other half ("or cause it to refuse something honest") was never asked.

On its own this is a **priced limit, not a fatal**: it is fail-closed and the class is pre-existing. It becomes decisive only in combination with L1, which needs no inflation at all.

### L3 — FATAL, measured, no refutation on record. The CTE pair is wrong and uncovered
- ADR-0019:205-208 holds conflicting entries "verbatim" as table *entries* — records inside the table's own bytes. ADR-0036:121-125,155-158 decided a CTE is **not** an occupant of the region it names and rejected requiring it to carry an extent. Neither supports "described by".
- Measured three ways: (a) CTE nested in the table's own bytes `[0,512)` — HEAD already reaches it (affected=3, constructs=false); **the release clause is redundant there**; (b) extentless CTE — HEAD affected=2 constructs, candidate affected=7 refuses; (c) CTE naming the aliased ESP region — HEAD unreached, candidate reached. So the pair fires only on the two shapes ADR-0039 rejected by measurement.
- Mutation, proven applied by `git diff`: remove the CTE arm -> **558 passed, 0 failed**.
- A reached CTE is never `Permitted` (`Indeterminate{Unrecognized}`, `protection.rs:495-498`), so the clause turns every destroying step on a hybrid table into a refusal.

**Answer to the panel's question: the pair set is not right.** Drop `conflicting-table-entry`, or justify it on its own ground and commit the fixture that kills the mutation *before* proposing.

### Surviving majors (unrefuted; FATALs only were handed to refuters)

| # | Finding | Key measurement |
|---|---|---|
| M1 | Amendment does not cover the L1 route — the reacher is a **partition**, not the table | f11: destroyed node is `bios_grub` `[17408,1 MiB)`; ESP `[1 MiB,257 MiB)` is comparable, outside, and in the set |
| M2 | False refusals with **no Refused technology anywhere** | plain SATA disk: hybrid CTE -> `Blocked{Unrecognized}`; orphan LVM2 signature -> `Blocked{OrphanSignature}`; affected 3 -> 5 |
| M3 | Wiping a hybrid MBR (512 B, no partition touched) refuses through a pool 512 MiB away | two tables GPT `[0,1MiB)` + MBR `[0,512)`: HEAD 10/10 Clear, affected=3 -> candidate `Unsupported{InheritedFromConsumerOrProducer}`, affected=7 |
| M4 | Whole-device wipe of a hybrid disk: `Clear` -> `Blocked{Unrecognized}` with **no acknowledgment arm** and a false remediation | `Release{cte}` and `OpaqueDestruction{cte}` both `Err(UnlawfulAcknowledgment)`; remediation string promises "establish the undetermined fact", which INV-008/REC-003 make impossible by construction |
| M5 | Escape is **complete and undetected**: omit the two `table -> partition` edges, or hang them off an extent-less `HybridMbr` decoy | edges omitted: gate(table) `CCCCCCCCCC`, affected=2, pool=false — bit-identical to HEAD. No delivered check prices the omission |
| M6 | Round 1 §10.1 survives at coarser granularity: same bytes under `written_table_extents` or `consumed` remove the refusal | destroyed: n=6 pool=true refuses / written: n=3 pool=false constructs / consumed: n=3 pool=false constructs. `PlanStep::mutating_declared` (step.rs:391-441) enforces no consistency law |
| M7 | Issue #349's `length: 0` removes the refusal outright on the candidate's own fixture | honest `[0,1MiB)` -> `Unsupported{Zfs}`; `length: 0` -> `Clear`. `intersects` requires `start < other.start + other.length` |
| M8 | The release is **unbounded in the source's extent** | a lawful 1-byte `HybridMbr` table node releases a 256 GiB member and its pool: HEAD affected=3 `Ok(3)`, candidate affected=6 `Err(Refused{Zfs})` |
| M9 (argued) | `describes_rather_than_nests` is behaviourally identical to `kind_of(source) == "partition-table"` | `endpoint_pair_allowed` lists exactly those two partition-table-sourced Containment pairs. The nearest mutation (`Some(_)` on the target arm) is a no-op, so §4's "killed by five" tests only the source-kind restriction, not the narrowness |
| M10 (argued) | The doc comment names the **under**-protecting default as fail-closed | "a pair added ... keeps the geometric bound, which is the fail-closed direction" — keeping the geometric bound is #347's own defect |

---

## Findings REFUTED by verification (results, not omissions)

- **"One byte on the table extent is the mirror of round 1 §10.2 / a false claim rather than a conservative one"** (filed twice, over-reach and authored-lever). **Refuted.** Baseline has identical one-byte flips on other extents; the direction is monotone and fail-closed, which round 1 §11.2 *demanded*; and on the inflated geometry HEAD's `Clear` is #347 recurring. Demoted to the priced limit in L2. Round 2 §6 must carry it, worded honestly.
- **"The release has zero strength — one destroyed byte releases the disk."** **Refuted.** One-byte destruction cascading is HEAD's committed rule and a committed guard *requires* it (`a_partial_destruction_reaches_the_content_it_truncates`, protection_tests.rs:676-700, MODEL-002/SAFE-005). The widening measured over all 262144 one-byte offsets on a 1 GiB disk: HEAD 65536/262144 = 25.000% refusing, candidate 65792/262144 = **25.098%** — 256 offsets, 1 MiB on a 1 GiB disk. The finding's mechanism claim ("the capability gate can never surface this") is also contradicted by its own shape: the gate surfaces it as `CCCCCCCCCC -> UUUUCCCCCC`.
- **"The release gate is not a destruction gate" (a step declaring no ranges releases).** **Reproduced, refuted.** True that `cascade_destroyed` is post-ADR-0039 a descent-reach set. But the only body where a merely-reached table releases is one whose device declares no self-framed extent — and on such a body HEAD's capability surface is **10/10 `Clear` over a live pool** (deleting one extent fact empties every `canonical_ranges` entry), versus the candidate's 0/10, matching the honest body's 0/10 under both. The candidate makes an omitted-extent lever *inert*. Demoted to a doc-comment correction plus a named limit.
- **"A descriptive edge relates no two extents" is false on the delivered facts.** **Refuted, and the evidence runs the other way.** Under one universal frame the same predicate answers **INSIDE** on `physical-device->partition-table`, `partition->file-system`, `partition->backing-signature`, and **OUTSIDE** only on `partition-table->partition` — root-framing *implements* the describes/nests split rather than dissolving it. ADR-0037 enforcement is held (`body_vectors.rs:249-255`, `plan_tests.rs:356-363` are unlawful and uncorrected), so totality is an authored property of `extent_host`. And at main the *lawful* root-framed body gates `Wipe(table)` `Clear` over a live pool while the unlawful framing refuses — the determinate answer the finding relies on is the answer that produced #347. Survives only as a **wording correction**: "relates no two extents" should read "makes no geometric claim."
- **"Implemented literally, the drafted amendment reds four committed guards."** **Refuted by method.** Implementing ADR-0039's *unamended* theorem literally as a construction rule reds two committed guards at base, including `the_root_on_zfs_regression_pair_holds`. A non-interference theorem is an upper bound; its exception is a permission, not an obligation. Demoted to a drafting note. The sub-claim "the release is structural rather than geometric is false" is separately refuted: across six placements of the released target's extent — inside, straddling, 900 MiB away, foreign frame, zero length, fact absent — the release fires identically. The release **across the edge** consults no target geometry; what is geometric is how the *source* was classified destroyed, which is HEAD's rule.

---

## Does the theorem amendment argument hold? No.

Round 2 §5 offers: *"...is ever in the set, except across an edge that describes rather than nests, where the two extents stand in no containment relation and the release is structural rather than geometric."*

- Its **factual premise** needs a wording fix, not a rebuttal (both endpoints declare extents; on the committed body they are comparable and answer "outside"). That correction is editorial.
- It **does not restore the theorem**, which is fatal to discharging ADR-0018:210-217. In L1 the step's reacher is a *partition*, not the table; the excusing clause names the `table -> partition` edge, so the measured outcome is outside its scope. ADR-0018:199-206's "the ESP at `sda1` is never captured by its sibling's pool" and ADR-0039's "holds by geometry rather than by the edge taxonomy alone" are both left with nothing holding them.
- It **carries no destruction condition**, while the code's gate is "in `range_destroyed` or `cascade_destroyed`, never `affected` alone" — and `cascade_destroyed` is a descent-reach set, so even that is imprecise as documented.
- It is **quantified over neither edge kinds nor the pair table**, so it cannot be delivered as the property test ADR-0018:210-217 mandates. ADR-0039 discharged its own arm with `no_propagating_pair_targets_a_kind_that_declares_bytes` (topology_tests.rs:263), which reds when a pair is added. Nothing here enumerates `endpoint_pair_allowed` against `describes_rather_than_nests`; a future `partition-table -> <described kind>` row falls silently to the geometric bound and re-derives #347.
- Round 2 §5 enumerates **one** falsified ADR sentence and **no** spec sentence. At least two more are falsified: AGENT_BUILD_SPEC §2.1:110 ("descent is bounded per edge target by the declared geometry — refused only where a child's declared extent positively contradicts containment within one frame" — the child's extent does positively contradict, and descent happens anyway) and ADR-0018's Decision as amended in 13.0.0.

**Do not land this amendment.** Any replacement must still forbid the L1 outcome — a step destroying one partition reaching a disjoint sibling — or it deletes the property the theorem was written for.

---

## The impossibility result (measured — the most useful thing this round produced)

Round 1 §11 imposed two requirements jointly. They are **jointly unsatisfiable over `Facts.extents`**, and this is now measured, not argued:

- **§11.1 (union semantics).** Built `union_covers` — merge destroyed ranges by frame, then test coverage — and put it in the real closure (mutation proven by `git diff`). 1 range / 2 adjacent / 3 reversed-and-overlapping all give the identical set, affected=7, refuses. §11.1 holds by construction.
- **§11.2 (monotone in the declared extent).** On that same repaired predicate, inflate the table's extent by **one byte**: affected 7 -> 3, pool=false, **CONSTRUCTS**. That is round 1 §10.2 verbatim. It is not a bug: coverage is `destroyed ⊇ extent`, so growing the number on the right can only make it harder to satisfy. **Every** coverage-strength predicate is anti-monotone in `Facts.extents`.
- The intersection test (this candidate) is monotone but has no strength — which is L1, L2 and M8.

Whatever ADR lands must state this bind. It is the argument for deciding the release **structurally**, and the honest reading of the trade: the strength test would buy back exactly one thing — honest wipes of table-overlapping partitions — at the cost of a one-byte fail-open, which round 1 §11.2 already decided against.

## Direction for round 3 (proposed, unmeasured)

Derive the release from the **naming relation**, not the edge set or the extents. `NamingFields::Partition { parent_table }` and `ConflictingTableEntry { table }` are on the one roster swept by `Topology::build` (`naming.rs:755-772`), so a partition cannot be represented without naming its table — which closes M5's omitted-edge escape by construction, is enumerable over `naming_referents` as the property test ADR-0018:210-217 demands, and is a property of a delivered type rather than a doc comment. Gate it on something structural about the *step* — target identity plus operation class, or an explicit destroyed-node set at the plan layer — never on whether a declared range happens to touch an authored extent. ADR-0036 established by measurement that occupancy is read from naming fields precisely because "an edge rides in no node's address preimage."

Before any candidate in this family is measured again, **commit the overlapping-geometry shape to the fixture population**: `f11` and `f12` are already written as assertions and pass at HEAD (`crates/domain/src/model/adversary_probe.rs`), plus a partition-target gate row on the committed geometry asserting `Clear` so the boundary is pinned rather than incidental.

---

## Gate claims (control — these check out)

The proposer's gate figures reproduce in a clean out-of-repo clone at 6e1706b, with real exit codes captured on the command itself, never through a pipe:
```
cargo test --workspace --no-fail-fast  -> exit 0, 648 passed, 0 failed
cargo xtask ci                         -> exit 0, 605 annotations, 50 evidence rows, 85 requirements, 641 live tests
```
The `every_repository_manifest_declares_the_project_licence` failure seen in nested worktrees is the known `fuzz/Cargo.toml` artifact — reproduced identically at main b3de0cf — and is **not** attributable to the candidate. Do not report it against this round.

Also confirmed as sound and worth keeping from the candidate: `NamingFields::kind_name` is a `&'static str` read off the enum variant, so the predicate cannot be steered by re-kinding; collision groups can only *add* refusal (`absorb` refuses unequal fields at equal addresses with `AddressCollision`); ADR-0024's repair family (`Repair`, `Label`, `Uuid`, `Create`, `Grow`, `Decrypt`) is untouched because it declares `written_table_extents` and never `destroyed`; CAP-005 agreement between the gate's synthesized entry and the plan constructor's verbatim ranges is intact; the planner, `simulate`, reversal drafts and cancellation classes are unreachable from the change — and `destroyed_closure` already agreed with the candidate's answer, so at HEAD the same wipe was simultaneously predicted to destroy the partition subtree and gated as reaching nothing.

---

## What this round did NOT establish

1. **Whether the overlapping shape is in-population or attacker-only.** No adapter computes extents at all (`crates/adapter-linux/src`, `crates/table-parser/src`: zero hits for `extent`), and no code derives a table node's extent from parsed bytes. Whether the delivered adapter would emit `[0, 17408)` or the conventional `[0, 1 MiB)` decides whether L1 and L2 are honest-body false refusals or attacker-authored ones. **This must be measured before round 3.**
2. **What a `partition-table` node's extent *is*.** ADR-0019 gives the table node *plural* extents while `Facts.extents` holds one `HostRange` per node, so the fixture's `[0, 1 MiB)` is the head region only. `schemas/domain/node-entry-format.md:102` defines the field as "the node's byte range on its host" with no geometric rule, and `snapshot.rs:421` checks only `may_carry_extent`. Part of #347's diagnosis is a fact-model limitation, not an edge-taxonomy fact; the ADR must say which.
3. **Whether any structural release actually closes #347.** The naming-referents shape and the target-identity gate were proposed and reasoned; neither was implemented or measured. Only the union-coverage shape was built and run, and it failed §11.2.
4. **Whether gating on `range_destroyed` alone (dropping the `cascade_destroyed` disjunct) still closes #347** — a table reached only by descent from a destroyed device would then release nothing. Not measured.
5. **The real-population cost of the CTE pair and of M2/M4.** Both were priced on synthetic fixtures only; no hybrid-disk or orphan-signature population figures exist.
6. **Issue #360's `volume -> partition-table` row.** Reasoned to inherit every finding above (the release fires on the source's kind, and that row adds a second way for a table to enter a destroyed class) — read from the pair table, not run.
7. **The `Some(_)` nearest-mutation result.** Read off `endpoint_pair_allowed`; not applied and run. Until it is, round 2 §4's "killed by five, four committed" establishes only that the source-kind restriction is load-bearing, not the pair set's narrowness.
8. **Whether `HostRange::intersects`' one-byte destruction semantics should itself be filed as a HEAD issue.** Two refuters concluded the honest-step refusals they measured belong there rather than on this candidate; nobody opened it.
9. **Nothing was measured on macOS**, and no acceptance-tier run was performed. This round is domain/planner/capability only.