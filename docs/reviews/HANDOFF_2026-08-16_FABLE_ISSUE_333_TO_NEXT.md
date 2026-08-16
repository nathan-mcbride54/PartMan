# Handoff — 2026-08-16, issue #333's enforcement (ADR-0046) and the r27 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #333's enforcement".
**To:** whoever picks this up next.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r27 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`43f2c99`** — the merge of PR #407 (the r27 re-pin), on top of the ADR-0046 act's merge `ca2bc0f` |
| Spec | **15.2.0** (ADR-0046) |
| `cargo xtask ci` | **exit 0** on the act — 635 annotations, 50 evidence rows, 85 requirements, 671 live tests; workspace 678 passed |
| WP-020 pin | **`ca2bc0f`** — `git diff --name-only ca2bc0f HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **7** — #319, #365, #366, #370, #371, #392, #397 (**#333 and #401 closed** by ADR-0046) |
| Proxmox | no `partman-wp020-*` guest; VMID **9452** is next; the `-r27` script set is current |

**Nothing is owed.** The next Rust merge owes r28.

---

## 1. What landed

| PR | Package | What |
| --- | --- | --- |
| #402 | Governance | ADR-0046's path reserved under WP-010. |
| #403 | WP-010 | **Occupancy is read as bytes** (issue #401, found by this round): `Precondition::violated_by` finds an occupant framed on the host (kept), lying on the host's bytes in the frame the host's own extent is expressed in (a region translated through it) with the host's frame ancestors excused, or named inside the host; an unlocated host fails closed. Unversioned; a defect fix on ADR-0022's mechanism. |
| #404 | WP-060 | The planner's occupancy fixtures take the frame ADR-0037 requires; `occupant_ground` extracted beside `occupancy_ground`; two grounds now asserted on the helpers only, since the frame rule makes their shapes unbuildable through a snapshot. |
| #406 | WP-010 | **ADR-0046: the frame rule is enforced.** `frame_root` over `naming_referent_rule`, derive-and-compare at `assemble`, `ExtentFrameDisagreesWithName`; a containment edge agrees with the name, `ContainmentEdgeDisagreesWithName`; ADR-0041's rule 6 collapsed to its live branch; the golden vector regenerated (two entries, `extent_host` and `extent_start`), `SCHEMA_VERSION` 1. Spec **15.2.0**. Closes #333 and #401. (Opened first as #405 against #404's branch; GitHub closed it when that branch was deleted on merge rather than retargeting — open the head PR of a chain against `main` once its base has merged, or expect to re-open.) |
| #407 | WP-020 | r27 re-pin at `ca2bc0f` (VMID 9451, 2026-08-16 UTC, custody run 37, transcript `4b0fd020…`, teardown 2026-08-16T16:09:22Z). |
| this | WP-000 | this handoff and `ISSUE-333_ENFORCEMENT_ROUND_2026-08-16.md`. |

Non-Markdown paths in the arc: `crates/domain/src/model/{protection,step,plan_tests,protection_tests,snapshot_tests}.rs`, `crates/domain/tests/body_vectors.rs`, `crates/planner/src/{solve,tests}.rs`, `schemas/domain/body-vectors.json`.

---

## 2. What was learned

### 2.1 The enforcement's real cost was a precondition, not a fixture

ADR-0037 priced `HostUnoccupied`'s frame-name reading against
derive-and-*replace* only. Under derive-and-compare the partition is
still never a frame on a lawful body, so the same reading is vacuous
and a decayed reversal binds. That is issue #401, and it is why the act
is three PRs (reading → WP-060 fixtures → enforcement) rather than one:
`a_decayed_reversal_refuses_instead_of_destroying` has no fixture form
valid under both the old reading and the frame rule. When an act makes
a class of body unlawful, ask what *else* reads the property that class
carried before asking which fixtures to move.

### 2.2 The enumeration killed what the fixtures could not

Three of fifteen mutations — a node outside every forest treated as its
own root, a root left unchecked, the open rule ignored — are killed by
`the_frame_rule_reaches_every_forest_at_every_depth` alone. Every
committed fixture is one forest deep in one shape; only the constructed
body holding all three forests at every depth × every candidate frame
sees those arms. Same lesson as ADR-0045's naming enumeration.

### 2.3 A survivor can be a proof

M15 (rule 6 admits a child in another frame) survives because rules 1
and 2 make its premise unconstructible; the enumeration shows every
cross-frame spelling refused before rule 6 runs. That is a reason to
collapse the branch, not to add a test — and to say so in the record.

### 2.4 Two things I did wrong, both already in the memory notes

- `git checkout -- <file>` on an unstaged test file mid-round reverted
  branch A's work; recovered from the checkpoint copy.
- `io.open(p,'w').write(io.open(p).read()...)` — the write-mode open
  runs first — emptied two source files inside a throwaway's revert;
  recovered from the index. Read into a variable, then open for write.

### 2.5 The third strength was free

The edge-name agreement ADR-0045 held beside this issue cost nothing
across the workspace (0 reds, 0 violations as a throwaway). It was taken
as a separately stated decision because the frame is derived from the
name while the closure descends along edges. The step-range rule was
also free and was *not* taken: it would refuse a range over a host-backed
file's bytes, which is #365's open question.

---

## 3. What is next

The chain **#347 → #360 → #354 → #333** is closed. Nothing is a chain
now.

- **#365** — what hosts and frames a `BackingExtent`. ADR-0046 pins the
  carve-out (a backing extent assembles framed on any absorbed node) and
  holds the step-range frame rule on it. The nearest to the closure.
- **#397** — device scope by name (fail-closed candidate in the filing).
- **#392** — the extentless-target limit, with its measured candidate.
- **#319's authorization half** — unmeasured since #338 closed.
- **`Partition.start_offset` against its extent's `start`** — a name
  versus a fact, both in one frame now; adjacent to ADR-0041's open
  `total_bytes` check. Unfiled.
- **The per-kind `canonical_ranges` entry** — ADR-0042's revisit
  condition, unfiled.
- **#366**, **#370**, **#371** — small / parallelizable / plan-vehicle.

Any of these that ships Rust owes r28.

---

## 4. Operational

`-r27` → `-r28`, VMID 9452. Creating and settling the guest while the
first PR's CI runs, then provisioning on the arc's head, kept the
sitting inside the merge chain's own wall clock this time.
