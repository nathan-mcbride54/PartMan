# Handoff — 2026-08-15, the body-validity act (issues #349, #356) and the r22 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #349 plus #356."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-15_FABLE_R21_TO_NEXT.md` (the r21 sitting),
whose §3 named this slice.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r22 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`931a539`** — the merge of PR #380 (the r22 re-pin), on top of `b002ac3` |
| Spec | **13.1.0** (ADR-0041) |
| `cargo xtask ci` | **exit 0** on `b002ac3` — 613 annotations, 50 evidence rows, 85 requirements, 649 live tests |
| WP-020 pin | **`b002ac3`** — `git diff --name-only b002ac3 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **10** — #319, #333, #347, #353, #354, #360, #365, #366, #370, #371 (**#349 and #356 closed** by ADR-0041) |
| Proxmox | no `partman-wp020-*` guest; VMID **9447** is next; the `-r22` script set is current |
| Local branches | `main` plus the merged `work/*` branches of this session (safe to delete) |

**Nothing is owed.** The next Rust merge owes r23.

---

## 1. What landed, in order

| PR | Package | What |
| --- | --- | --- |
| #377 | WP-060 | `occupancy_ground(located, host, declared_start)` extracted; every `OccupancyGround` arm asserted on it; the other-host case names an absorbed device. **Landed first, by design** — see §2.1. |
| #378 | Governance | ADR-0041's path reserved under WP-010, with the grant's reach and limits recorded in `WP-010.md`. |
| #379 | WP-010 | **ADR-0041: the body's facts are validated against its topology at assembly.** `validate_facts` in `protection.rs`, called from `TopologySnapshot::assemble`; six rules; the decode path's four placement checks deleted; `SnapshotSchemaError::MisplacedFact` retired; the `bios_boot_gpt` fixture with `f11`/`f12` committed. Spec 13.1.0. Closes #349, #356. |
| #380 | WP-020 | r22 re-pin at `b002ac3` (VMID 9446, custody run 32, transcript `696c4ed5…`). |
| this | WP-000 | this handoff and `ISSUE-349-356_BODY_VALIDITY_ROUND_2026-08-15.md`. |

Six non-Markdown paths in the arc: `crates/planner/src/{solve,tests}.rs`,
`crates/domain/src/model/{protection,protection_tests,snapshot,snapshot_tests}.rs`.

---

## 2. What was learned

### 2.1 A breaking cross-package change has to land consumer-first

One PR carries one package. A domain change that reddens a planner test
has **no green ordering** unless the planner lands first in a form valid
under both regimes. The "jointly-sequenced" precedent (3l/3m, #362/#363)
was additive — WP-010 first, WP-060 consumes — and does not cover this.
The shape that works: WP-060 asserts the affected behaviour where the
domain change cannot reach it (a pure helper), keeps the consumer's own
guard, and lands green under both; measured on a throwaway merge of the
two branches before either landed. **Do that measurement**; the WP-060 PR
body should say it did.

### 2.2 Containment is not span containment for tables

The first draft of the #356 rule was a blanket "child within parent". It
refuses every honest GPT disk: partitions are containment children of the
*table* and lie outside the table's `[0, 1 MiB)` extent, because that
extent is the table's own header bytes, not the region it governs. Caught
by the `bios_boot_gpt` fixture *and* by a pre-existing step test under
mutation. The delivered rule is per pair (`containment_pair_is_geometric`):
seven geometric, the two `partition-table` pairs structural. Any predicate
comparing a table's extent to its partitions' — reach, release (#347),
validation — is comparing the wrong things. Recorded in the ADR and in
memory.

### 2.3 What validation buys, and does not

The 2026-08-14 scoping pass's §8.5 stands and was re-measured: #356's
absent-extent spelling **still constructs** (`affected=2`, pool unreached).
That is #319's class. Validation buys self-consistency; the act's record
says so in four places rather than letting a green suite imply more.

### 2.4 Process notes

- The act's adversarial content is a twelve-mutation battery (each proven
  applied, each killed) plus probes — **single-author**, no panel. The ADR
  and the round record say so. If the owner wants a panel, §3–§6 of the
  round record are the claims to attack; the most valuable target is any
  honest layout in which a geometric pair's child legitimately lies outside
  its parent's span (none found in the committed population).
- The pin-vs-`HEAD` lesson from r21 was applied on purpose: the r22 re-pin
  checked `git diff --name-only b8d6a90 b002ac3` and it listed exactly the
  arc's six paths.
- Heredocs nested inside `ssh '…'` or inside `bash -c` broke twice more
  this session; writing the patch to a scratch file and piping/running it
  worked every time.

---

## 3. What is next

The previous handoff's ordering, minus the two closed:

- **#353** (`canonical_ranges` writes the target's whole extent, §2.1:110)
  — the self-contained one. Note ADR-0040's record: its interaction with
  the whole-disk regression must be re-measured before it merges.
- **#347 round 3** — the fixture population now has `bios_boot_gpt`,
  `f11` and `f12` committed, which the round-2 panel required before any
  candidate is measured. Read `ISSUE-347_RELEASE_ROUND_2_ADVERSARIAL_2026-08-14.md`
  first; the extent-keyed family is dead, the naming-relation direction is
  reasoned not measured, and §2.2 above bears on it directly.
- **#319's authorization half** — its recorded blocker #338 closed; nobody
  has re-measured. ADR-0041 leaves the absent-extent spelling to it.
- **#365** (host-backed producing relation) — small, buys a fixture.
- **#366** (WP-035) — parallelizable with any of the above.

Any of these that ships Rust owes r23 — name it in the PR body **and**
check the stopping condition against `HEAD` before merging.

---

## 4. Operational

- Copy `-r22` → `-r23` on `root@10.7.7.100`; VMID 9447; `CANDIDATE_COMMIT`
  in 02; header prose; evidence path. The sequence in the previous
  handoff's §4 ran void-free twice today; keep all of it.
- Evidence: `/root/partman-wp020-evidence-r22` on the host,
  `C:\Users\nmcbr\PartMan-evidence\partman-wp020-evidence-r22\` here.
