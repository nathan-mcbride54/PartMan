# Handoff — 2026-08-16, issue #347 round 3 (ADR-0043) and the r24 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #347 round 3."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-15_FABLE_ISSUE_353_TO_NEXT.md`.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r24 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`e068ded`** — the merge of PR #389 (the r24 re-pin), on top of `c83d9f1` |
| Spec | **14.0.0** (ADR-0043) |
| `cargo xtask ci` | **exit 0** on `c83d9f1` — 623 annotations, 50 evidence rows, 85 requirements, 666 live tests |
| WP-020 pin | **`c83d9f1`** — `git diff --name-only c83d9f1 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **8** — #319, #333, #354, #360, #365, #366, #370, #371 (**#347 closed** by ADR-0043) |
| Proxmox | no `partman-wp020-*` guest; VMID **9449** is next; the `-r24` script set is current |

**Nothing is owed.** The next Rust merge owes r25.

---

## 1. What landed

| PR | Package | What |
| --- | --- | --- |
| #387 | Governance | ADR-0043's path reserved under WP-010. |
| #388 | WP-010 | **ADR-0043: a destroyed partition table releases the partitions it describes.** A step whose *target* is a partition table and whose own destroyed ranges reach it destroys the table; a destroyed table releases every partition whose *name* says it describes it (`Partition.parent_table` → `NamingFields::released_by_table`). Never a table another step's range touches, never coverage of the table's extent, never the edge set; a `ConflictingTableEntry` is not released. `f11`/`f12` re-spelled on the honest partition target with the table-target spelling pinned beside them as the priced limit; the roster property test; `Containment`'s doc comment corrected. Spec **14.0.0**, carrying a correction to ADR-0042's pricing. Closes #347. |
| #389 | WP-020 | r24 re-pin at `c83d9f1` (VMID 9448, 2026-08-16 UTC, custody run 34, transcript `faeaf05e…`). |
| this | WP-000 | this handoff and `ISSUE-347_ROUND_3_2026-08-16.md`. |

Four non-Markdown paths in the act:
`crates/domain/src/model/{naming,protection,protection_tests,topology}.rs`.
No consumer package moved; `crates/capability` and `crates/planner` green
unchanged.

---

## 2. What was learned

### 2.1 The trigger is the target — and that dissolves both earlier fatals at once

Round 1 died on coverage (anti-monotone), round 2 on intersection
(sibling capture from a partition-target step). Both keyed the release on
the table's own authored extent. Keying it on **the step's target and the
step's own destruction of it** makes the table's geometry irrelevant
except for the trivially-true "a step that destroys its own target
intersects it": inflate, deflate, under-declare — the release fires;
touch the table from a partition-target step — nothing releases. The
panel's L1, L2, M3 and M5 all measured closed on the fixture it required.
Six mutations, each killed; release-on-target-identity-*without*-destruction
is killed by four pre-existing guards.

### 2.2 `f11`/`f12` were spelled on the wrong target

Both committed assertions deleted `bios_grub` with **the table** as
target. Under a target-keyed release that spelling releases — the closure
cannot tell one GPT entry from the header (round 2's impossibility
result). The honest spelling is `Wipe` on the partition, which is what the
panel measured as L1 and what the planner emits: 10/10 `Clear`, only the
entry reached. Both are pinned side by side; the table-target one is the
one priced limit, fail-closed. If the owner reads that as sibling
capture rather than a conservative reading of "destroy part of my own
target", it is the first thing to attack.

### 2.3 Cut what you cannot cover

The first candidate propagated the release from a wholly-destroyed target
through the cascade so a table on a destroyed volume releases too. It
closes the #360 chain — measured, with the row added for the probe only:
`Wipe(member disk)` HEAD constructs, with propagation refuses — and it was
**uncovered** at HEAD because the chain cannot be built without #360's
row. Two mutations survived on it. Cut, and recorded as #360's remainder
with the numbers, so #360's act adds row + fixture + propagation together.
Round 2 rejected an uncovered clause; I did not want to ship one.

### 2.4 ADR-0042 was mispriced, and I said so

§2.1:113 read "a node's own address space is never a descent source".
ADR-0042 made the target an exception and I priced it "spec unchanged".
Wrong under §0.1 and ADR-0039's precedent. Corrected in this act's spec
edit (the sentence now carries the exception with an *(ADR-0042; carried
by 14.0.0)* marker), and stated in the ADR, the CHANGELOG and the PR.
**Rule for next time:** when an ADR touches `descends_into` or the
closure's seeds, grep §2.1:113 and ADR-0018's theorem for a sentence it
falsifies before pricing.

### 2.5 The Wipe(volume) limit

An extentless target's canonical `destroyed` is empty, so the closure
cannot see it destroyed and `Wipe(volume)` reaches a table it carries only
as content, not as release. Named in the ADR; #360's territory.

---

## 3. What is next

The chain #347 headed is now **#360 → #354's kind half → #333's
enforcement**, and #360 is the head:

- **#360** — the `volume → partition-table` row, **with** the release
  propagation ADR-0043 measured and cut, the chain fixture, and a
  decision on `Wipe(volume)`. Start from `ISSUE-347_ROUND_3_2026-08-16.md`
  §4 and the ADR's *remainder* paragraph; the probe layout is in the round
  record. Consumer-first if `crates/planner` moves.
- **#319's authorization half** — unmeasured since #338 closed.
- **The per-kind `canonical_ranges` entry** — ADR-0042's revisit condition,
  unfiled.
- **#365**, **#366** — small / parallelizable.

Any of these that ships Rust owes r25.

---

## 4. Operational

`-r24` → `-r25`, VMID 9449; the sequence has run void-free four times.
