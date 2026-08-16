# Issue #333: the frame rule enforced — measured round, 2026-08-16

`docs/reviews` artifact, committed under WP-000 after the arc merges.
Everything load-bearing is restated in ADR-0046
(`docs/adr/0046-the-frame-rule-is-enforced.md`), which is where the
decision is put to the owner. Single-author round; the adversarial
material it stands on is ADR-0037's round of 2026-08-13
(`ISSUE-333_ANCHORING_ROUND_2026-08-13.md`), the precondition reading of
2026-08-14 (`ADR-0037_PRECONDITION_READING_2026-08-14.md`) and issue
#354's two rounds.

Findings measured on `43872c0` in the working checkout, on branches that
became PRs #403 (WP-010, issue #401), #404 (WP-060, the fixtures) and
#406 (WP-010, the ADR-0046 act; first opened as #405 against #404's
branch, which GitHub closed on that branch's deletion), merged in that
order at `ca2bc0f`, with the r27 sitting taken there (#407); the numbers below are from those branches at the state
committed. Every gate figure is `cargo xtask ci` exit 0 unless it says
otherwise.

## 1. What the round set out to do

ADR-0037 held the frame rule's enforcement and named its front-runner —
a **naming-field-derived** frame predicate in **derive-and-compare**
form — with a precondition (the capture-side referent sweep, both
halves) that ADR-0045 discharged earlier the same day. The task was to
build that predicate, measure it green, regenerate the golden vector and
`plan_tests.rs` in the same act with the MODEL-003 discharge, and record
what it does and does not make sound.

## 2. What the round established

1. **The naming-derived predicate is buildable and green.** `frame_root`
   walks the one naming field per kind that `naming_referent_rule`
   classifies as naming a containment source — the same map ADR-0045
   authored, so there is no second roster — until a kind that names none.
   The comparison is against `HostRange.host`; a mismatch refuses at
   `validate_facts` with `ExtentFrameDisagreesWithName { node, declared,
   derived }`. Applied to the population as committed at `43872c0` it
   produced **14 reds across 3 packages** — the number ADR-0037's edge
   walk produced too, but a different fourteen: seven `plan_tests`
   (all `reversal_worlds`, whose file system was partition-framed), one
   `snapshot_tests` (ADR-0041's test, which pinned the partition-framed
   twin as *lawful*), two `body_vectors` (the golden vector), and four
   planner tests. Every one is a fixture in the shape ADR-0037 called
   unlawful, or a test that pinned that shape on purpose;
   `the_guard_stands_with_every_containment_edge_removed` — the test that
   made the edge walk unbuildable — was green throughout, because the
   frame is read off the name.

2. **The enforcement cannot land alone: `Precondition::violated_by` is
   vacuous on the lawful population.** Filed as **issue #401**. Under
   the frame rule a partition is never a frame; `HostUnoccupied` and
   `RegionUnoccupied` found an occupant only where an extent was
   *framed on* the host, so a decayed reversal binds. Measured on the
   unmutated reading at `43872c0`: re-frame `reversal_worlds`' file
   system alone onto the device and `a_decayed_precondition_refuses_at_binding`
   fails. ADR-0037 had seen the shape (`:134-144`) and priced it against
   derive-and-*replace* only. This is what turned one PR into three:
   the planner's `a_decayed_reversal_refuses_instead_of_destroying` has
   **no fixture form valid under both** the old reading and the frame
   rule, so the reading had to land first (WP-010), the planner's
   fixtures next in a form valid under both regimes (WP-060, PR #377's
   precedent), then the enforcement (WP-010, ADR-0046).

3. **Occupancy read as bytes is a strict superset and every reading is
   load-bearing.** Three readings — framed on the host (kept), lying on
   the host's bytes in the frame the host's own extent is expressed in
   (a region translated through it) with the host's frame ancestors
   excused, and named inside the host — plus a fail-closed arm for a
   host whose own extent is absent. Each is killed by its own mutation
   (M6–M11); the ancestor exclusion (M8) is killed by twenty tests,
   because a device's self-extent overlaps everything on it. The old
   reading's mutation (M11) **survived at first**: on every constructible
   snapshot it is subsumed. It is kept because it preserves the strict
   superset claim, and one committed corner now keeps it live — a
   backing extent (the one kind the frame rule lets be framed anywhere)
   framed on a bare device past its self-extent, which no other reading
   sees.

4. **The "third strength" — a containment edge agrees with the name —
   costs nothing and was taken.** ADR-0045 held it beside this issue.
   Priced as a throwaway across the whole workspace: **0 reds, 0
   committed violations**. Taken as ADR-0046's second decision because
   the frame is derived from the name while the closure descends along
   edges, and with it a node's three positional claims (name, edge,
   extent) are pairwise consistent; ADR-0041's rule 6 then has one live
   branch and is collapsed to it.

5. **The enumeration is what makes "every forest, every depth" a
   measurement.** One body holding a device forest, a produced-volume
   forest and a multipath forest, an extent-bearing node at every depth
   of each (17) plus a backing extent, every absorbed node a candidate
   frame (21): **340 refused, 38 admitted** — one lawful frame per
   forest node, all 21 for the backing extent. Every containment edge
   (16) re-sourced onto every other node (19): **59 refused by the name,
   245 by the pair table first, 0 admitted**. Three mutations (a node
   outside every forest treated as its own root; a root left unchecked;
   the open rule ignored) are killed by the enumeration **alone** — the
   lens that would otherwise not have run.

6. **A root-framed rule on a step's declared ranges is zero-cost and was
   not taken.** Applied as a throwaway at the step constructor: 0 reds,
   0 violations across every committed step. Not taken because a range
   over a host-backed file's bytes is expressed in its file system's own
   address space (`ExtentLocator::Range`), and the rule as stated would
   refuse it — issue #365's open question. Recorded in the ADR with its
   measurement.

7. **The golden vector moves by exactly the two fields the rule speaks
   to.** `snapshot-full-captured` and `node-entry-backing-signature-7`:
   `extent_host` partition → device, `extent_start` 4096 → 1052672
   (`start_offset + primary_offset`); the fourteen other generated
   entries byte-identical; `cargo xtask cross-language` exit 0 with the
   TypeScript suite unchanged. MODEL-003 under the explicit-rejection
   limb, `SCHEMA_VERSION` 1, on PR #362's and ADR-0041's precedent — the
   debt ADR-0037 said travels with the enforcement, discharged.

8. **Two planner grounds become unreachable through a snapshot**, as
   `RangeIsEmpty` did under ADR-0041: `RangeOnAnotherHost` (a partition
   framed on another device is now refused at assembly) and
   `TableIsNotThisHosts` (a partition's extent host is its own table's
   root, so "located on this host under a table this host does not
   carry" cannot be built). WP-060 extracts the foreign-table arm as
   `occupant_ground` beside `occupancy_ground` and asserts both on the
   helpers; nothing is deleted from the solver.

## 3. The mutation battery

Fifteen, each applied by exact substitution with the anchor asserted
unique, the file's content hash taken before, the domain and planner
suites run with `--no-fail-fast`, and the file restored by the reverse
substitution with the hash asserted equal. (A `git checkout --` was run
once by mistake on an unstaged file mid-round and reverted branch A's
test; recovered from the checkpoint copy. A second slip — opening a file
for write before reading it in the same expression — emptied two source
files during a throwaway's revert; recovered from the index. Both are in
the memory notes now.)

| # | mutation | killed by |
| --- | --- | --- |
| M1 | frame check not applied | 4 |
| M2 | compare against the immediate host, not the root | 45 |
| M3 | a node outside every forest is its own root | 1 (enumeration) |
| M4 | a root is not checked | 1 (enumeration) |
| M5 | the open rule ignored | 1 (enumeration) |
| M6 | occupancy by geometry dropped | 1 |
| M7 | occupancy by name dropped | 1 |
| M8 | the host's ancestors are occupants | 20 |
| M9 | the region not offset by the host's start | 1 |
| M10 | an unlocated host binds | 1 |
| M11 | ADR-0022's framed-on-host reading dropped | 0 → 1 after the corner test |
| M12 | the planner's foreign-table arm returns nothing | 1 |
| M13 | the edge-name rule not applied | 1 |
| M14 | rule 6 dropped entirely | 4 (ADR-0041's) |
| M15 | rule 6 admits a child in another frame | **0 — survives; premise unconstructible** |

M15 is the recorded survivor: rules 1 and 2 make a child and its edge
parent reaching rule 6 in different frames unconstructible, and the
enumeration in finding 5 shows every cross-frame spelling refused
before rule 6 runs. The branch is collapsed rather than kept as dead
code with dead evidence.

## 4. Adversarial pass on the act, by the author

1. **"This is derive-and-replace by another name."** Not sustained.
   `HostRange.host` is compared and refused, never overwritten;
   `parse_ranges`, `HostUnoccupied`'s framed-on-host reading and
   `RangeOnAnotherHost` all still exist and are asserted.
2. **"The occupancy change widens beyond #333."** Sustained as a fact,
   answered as a necessity: without it the enforcement has no green
   ordering (finding 2), and it is a superset — every occupant the old
   reading found is still found (M11's corner proves the reading is
   live). It is filed (#401), measured on the unmutated code, and lands
   in its own PR with its own CHANGELOG entry.
3. **"The edge-name rule is scope creep."** Partly sustained: ADR-0037
   did not ask for it. Answered: ADR-0045 held it explicitly beside this
   issue, it is measured zero-cost, and it is stated as a separate
   decision in the ADR rather than folded in.
4. **"Fail-closed on an unlocated host is a behaviour change on
   `violated_by`'s contract."** Sustained and accepted: "the first node
   violating, or `None` where it holds" now returns the host itself
   where its emptiness cannot be established. Without it the act would
   *subtract* a refusal (an fs framed on the partition was caught before;
   framed on the device with the partition extentless it would not be).
5. **"The rule still does not make the reach sound."** Sustained,
   exactly as ADR-0037 priced it, and stated in the ADR's consequences.
6. **"Regenerating the vector without a version bump is a break."**
   Answered as ADR-0041 answered it: format and parse rules unchanged;
   the refused population was unlawful since 12.14.0; every other body
   assembles; the TypeScript suite is unchanged.

## 5. Decision carried forward

ADR-0046: the frame rule enforced derive-and-compare at assembly; a
containment edge agrees with the name; occupancy read as bytes; the
vector regenerated. Spec **15.2.0, minor**. Issues #333 and #401 close.
What stays open is in the ADR: #319's authorization half, #365 (what
frames a backing extent — the carve-out is pinned), the step-range rule
priced and held on #365, `start_offset` against its extent's `start`.
