# Handoff — 2026-08-15, issue #353 (ADR-0042) and the r23 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #353."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-15_FABLE_BODY_VALIDITY_TO_NEXT.md`, whose §3
named this slice as the self-contained one.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r23 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`4acd94d`** — the merge of PR #385 (the r23 re-pin), on top of `53c90f1` |
| Spec | **13.1.0** — ADR-0042 is a defect fix, spec unchanged |
| `cargo xtask ci` | **exit 0** on `53c90f1` — 618 annotations, 50 evidence rows, 85 requirements, 661 live tests |
| WP-020 pin | **`53c90f1`** — `git diff --name-only 53c90f1 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **9** — #319, #333, #347, #354, #360, #365, #366, #370, #371 (**#353 closed** by ADR-0042) |
| Proxmox | no `partman-wp020-*` guest; VMID **9448** is next; the `-r23` script set is current |

**Nothing is owed.** The next Rust merge owes r24.

---

## 1. What landed, in order

| PR | Package | What |
| --- | --- | --- |
| #382 | WP-060 | The unordered-overlap test re-based on two wipes whose ranges truthfully overlap (it had paired a wipe with an *unsized* create that overlapped only through the over-claim); a **new** test pinning that a `Label` on a partition carries its disk's PART-013 obligation and refuses on Indeterminate media. **Landed first, by design.** |
| #383 | Governance | ADR-0042's path reserved under WP-010. |
| #384 | WP-010 | **ADR-0042: a frame root is never written wholesale, and a target frame root reaches what it carries.** `canonical_ranges` declares no written range for a self-framed target; `descends_into` admits descent out of a self-framed extent when, and only when, that node is the step's target. Four regressions incl. the whole-disk pin the issue asked for. Spec unchanged. Closes #353. |
| #385 | WP-020 | r23 re-pin at `53c90f1` (VMID 9447, 2026-08-16 UTC, custody run 33, transcript `70001808…`). |
| this | WP-000 | this handoff and `ISSUE-353_CANONICAL_RANGES_ROUND_2026-08-15.md`. |

Four non-Markdown paths in the arc: `crates/planner/src/tests.rs`,
`crates/domain/src/model/{capability,protection,protection_tests}.rs`.

---

## 2. What was learned

### 2.1 The whole-disk gates rested on the over-claim, and the fix is a hop, not an entry

On a whole-disk vdev every refusal came from the byte scan over the
over-claimed whole-device write, because ADR-0039's `descends_into`
refuses descent out of a self-framed extent. That refusal exists for a
*route* — a range intersecting the disk's self-extent — not for the node.
The step's target is in the set by identity, so admitting the hop for the
target alone gives ADR-0039's headline to a disk and leaves the
sibling-capture guard where it was. Measured: widen the hop to any source
and the pre-existing ordinary-disk guard reds; remove it and the
whole-disk pin reds.

### 2.2 "Declare nothing" was right in one sense and wrong in another

The first candidate declared no ranges for all six write operations. The
domain suite survived it (M4). Reading the planner's `touched_devices`
showed it would silently drop PART-013 obligations and the
indeterminate-table guard for every partition-target write, with nothing
pinning either. The entry is therefore narrowed to the frame root, and
the partition-target entry — an over-approximation the record names — is
pinned on both sides. **When a mutation survives the domain suite, read
the consumers before concluding it is inert.**

### 2.3 What is honestly left on this entry

A per-kind truthful entry (a create writes the host's table extents; a
grow writes one entry; a label writes inside its own structure) needs the
request or the topology at `canonical_ranges`, whose signature is shared
with WP-050 and WP-060. That is a cross-package act — consumer-first,
per the previous handoff's §2.1 — and ADR-0042's revisit condition names
it.

### 2.4 Process notes

- Consumer-first sequencing worked a second time: WP-060 first (green
  under both regimes, proven on a throwaway merge), Governance in
  parallel, then the act; one r23 sitting at the arc's head named in
  both PR bodies before the first merge; the pin checked against `HEAD`
  at re-pin (`git diff --name-only b002ac3 53c90f1` = exactly the arc's
  four paths).
- Single-author review again; the ADR names the claim most worth
  attacking (that the target is the *only* safe case for the hop).

---

## 3. What is next

- **#347 round 3** — the fixture population has `bios_boot_gpt`; ADR-0041
  §2.2 (a table's extent bounds only itself) and ADR-0042 (the target
  hop) both bear on it. `Wipe table` over a live pool member is still
  `Clear`. Read `ISSUE-347_RELEASE_ROUND_2_ADVERSARIAL_2026-08-14.md` first.
- **#319's authorization half** — unmeasured since #338 closed.
- **The per-kind `canonical_ranges` entry** (§2.3) — not filed as an
  issue; ADR-0042's revisit condition carries it. File it if it is to be
  worked.
- **#365**, **#366** — small / parallelizable.

Any of these that ships Rust owes r24.

---

## 4. Operational

`-r23` → `-r24`, VMID 9448; the sequence in the r21 handoff's §4 has now
run void-free three times.
