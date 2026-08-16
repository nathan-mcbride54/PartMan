# Handoff — 2026-08-16, issue #360 (ADR-0044) and the r25 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #360."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-16_FABLE_ISSUE_347_TO_NEXT.md`.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r25 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`be93aa2`** — the merge of PR #394 (the r25 re-pin), on top of `91cd1c9` |
| Spec | **15.0.0** (ADR-0044) |
| `cargo xtask ci` | **exit 0** on the act — 626 annotations, 50 evidence rows, 85 requirements, 662 live tests; workspace 669 passed |
| WP-020 pin | **`91cd1c9`** — `git diff --name-only 91cd1c9 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **8** — #319, #333, #354, #365, #366, #370, #371, **#392** (new; **#360 closed** by ADR-0044) |
| Proxmox | no `partman-wp020-*` guest; VMID **9450** is next; the `-r25` script set is current |

**Nothing is owed.** The next Rust merge owes r26.

---

## 1. What landed

| PR | Package | What |
| --- | --- | --- |
| #391 | Governance | ADR-0044's path reserved under WP-010. |
| #393 | WP-010 | **ADR-0044: destruction carries through the cascade, and a volume carries a partition table.** The `volume → partition-table` row; the destroyed class re-separated from reach (`destroyed`, `destroy`, `carry` in `protection.rs`) — seeded by the target when the step's own ranges reach it, carried along the four arms under `descends_into`, releasing by `released_by_table`, released partitions destroyed in turn; the `partitioned_mdraid` fixture, the GPT-in-LUKS layout, the plain control, the reach-never-releases guard with the extentless-target limit pinned; the two topology tests re-spelled. Spec **15.0.0**. Closes #360; files #392. |
| #394 | WP-020 | r25 re-pin at `91cd1c9` (VMID 9449, 2026-08-16 UTC, custody run 35, transcript `0a00b8c2…`). |
| this | WP-000 | this handoff and `ISSUE-360_ROUND_2026-08-16.md`. |

Four non-Markdown paths in the act:
`crates/domain/src/model/{protection,protection_tests,topology,topology_tests}.rs`.
(#393's body said six; it is four — two of the six it counted are `.md`.)
No consumer package moved; `crates/capability` and `crates/planner` green
unchanged.

---

## 2. What was learned

### 2.1 ADR-0039 had merged the destroyed class into reach

`cascade_destroyed` after ADR-0039 means "reached by descent from any
member of the set". That is why a table on a volume reached by
`Wipe(member)` did not release, and why "any reached table releases" is
wrong (`Label(member)` reaches the same table; mutation M2 dies on five
tests). The act re-separates the two without touching reach: `destroyed
⊆ cascade_destroyed`, carried only from destroyed sources. If you find
yourself asking "is this node destroyed or merely reached", the answer
is now a set membership, not an inference.

### 2.2 The seed is the target, and only the target

Seeding from every range-destroyed node re-derives round 2's L1 (M3
kills four guards). A range-destroyed non-target node keeps HEAD's
behaviour exactly — reached, descending — and establishes no destruction
of its own. Consequence, stated in the ADR: any device-target step whose
declared destroyed range touches the device destroys the table it
carries and releases every partition — ADR-0043's priced limit one level
up. Nothing delivered emits that spelling.

### 2.3 Measure both trees with one probe

The before/after tables were produced by one probe test appended to both
worktrees (`d67d4df` and the candidate) and diffed. That is what makes
"every existing layout byte-identical" a measurement rather than a
reading. Keep the probe out of the commit; the permanent tests assert
the rows that matter.

### 2.4 The whole-frame candidate is cheap and still not free

`Wipe(volume)` closes with one line in `canonical_ranges` (an
extentless target destroys `[0, u64::MAX)` in its own frame) and the
workspace stays green — with only the pinned limit row moving. It was
held because it changes the planner's `destroyed_closure` on a
population no planner test covers; #392 carries the measurement. Whoever
takes it: cover the simulation first, consumer-first if the planner
moves, and note `Wipe(array)` stays `Clear` even under the candidate.

### 2.5 Two traps avoided this time, one hit

- `python3` on this workstation defaults to cp1252 on write: a splice
  script that read a UTF-8 file with `→` in it and wrote it back
  **truncated the file** on the encode error. Always `io.open(...,
  encoding='utf-8')` both ways, and check `git diff --stat` before
  running anything.
- A poll that `pgrep -f`s the script name over `ssh` matches its own
  `bash -c` line and never terminates — key on the log's `run status`
  line instead (the runbook already says so; I wrote it anyway and had to
  kill it).
- The sitting was launched by absolute path, after `settle-r25.sh`, with
  snapd present, partman locked, `uname -r` = -186 before and after.

---

## 3. What is next

The chain **#347 → #360 → #354's kind half → #333's enforcement** has
two links closed; **#354's kind half is the head**, on one question: a
`FileSystem` (or table) hosted on a `MultipathNode` is admitted by no
row — is that ADR-0011's intent (then the derived check should enforce
it) or a further omission (then a row first)? Comment left on #354.

- **#354's kind half** — decide the multipath population, then derive
  the check from `endpoint_pair_allowed`; the held-half test is the one
  to change deliberately.
- **#392** — the extentless-target limit, with the measured candidate.
- **#319's authorization half** — unmeasured since #338 closed.
- **The per-kind `canonical_ranges` entry** — ADR-0042's revisit
  condition, unfiled.
- **#365**, **#366** — small / parallelizable.
- An observation, not filed (read, not measured): `device_scope_verdict`
  ascends containment roots only, so content on a produced volume does
  not inherit the underlying disk's device-scope verdict. Worth a probe.

Any of these that ships Rust owes r26.

---

## 4. Operational

`-r25` → `-r26`, VMID 9450; the sequence has run void-free five times.
