# Issue #353 — the canonical-ranges round, 2026-08-15

`docs/reviews` artifact, committed under WP-000 in its own pull request
beside the act it records (WP-010, ADR-0042). **Single-author.** The
adversarial content is a five-mutation battery, each proven applied, plus
per-layout gate tables measured before and after. Stated as such; the
ADR is where the decision owner is put the question.

## 1. The question

`canonical_ranges` wrote the target's whole extent for six operations —
for a device, the parent device wholesale, which §2.1:110 forbids — and
the whole-disk gates depended on it: correcting the entry alone opens six
`Clear` gates over a live pool with a green suite (the issue's table). So
the question was not "fix the entry" but "where does a whole-disk
layout's protection come from once the entry is truthful", and the issue
listed three routes.

## 2. The finding that chose the route

On a whole-disk layout the label is reached by the byte scan alone
because ADR-0039's `descends_into` refuses descent out of a self-framed
extent — a clause that exists to stop a disk's own extent capturing every
sibling when a partial range intersects it. **That hazard is a property
of how a node entered the set, not of the node.** The step's target is in
the set by identity. Admitting the hop for the target and for nothing
else gives ADR-0039's own headline — a step reaches the content its
target carries — to a disk, and leaves the sibling-capture guard exactly
where it was for every other node. Measured under M2 (widen the hop to
every source): the pre-existing ordinary-disk guard reds. Under M1
(remove the hop): the whole-disk pin reds.

## 3. The finding that narrowed the entry

The first candidate declared nothing for all six operations on every
target ("what they write cannot be named here"). It survived the domain
suite and moved exactly one planner test — an unsized create overlapping
a wipe only through the over-claim — but a reading of the planner's
`touched_devices` showed it would also silently drop the PART-013
parse-backup obligation and the indeterminate-table guard for every
partition-target write, with nothing pinning either. So the entry is
narrowed to the frame root: a self-framed target declares no written
range; below it the entry is unchanged and now pinned on both sides (a
domain assertion on its shape; WP-060's
`a_partition_write_still_touches_its_disk_for_the_protection_arms` on
its consequence, which is the test that kills M4).

## 4. Measured

Gate tables (Create Grow Shrink Move Repair Label Uuid Encrypt Decrypt
Wipe; C/R/B) at `1f450c6` → candidate:

| layout / target | before → after |
| --- | --- |
| whole-disk vdev / sda | `RRRRRRRRRR` → `RRRRRRRRRR` |
| root-on-ZFS / sda | `RRRRRRRRRR` → `CCRRCCCRCR` |
| root-on-ZFS / table, esp, member, signature | unchanged |
| LUKS chain / sdb, part, mapper | `R…` unchanged (the disk has no self-extent; descent out of it was already unconditional) |
| ordinary disk / sdz | `BBBBBBBBBB` unchanged (the stale device-hosted superblock is carried content) |
| ordinary disk / table, esp, data | `C…` unchanged |
| BIOS-boot / sda | `RRRRRRRRRR` → `CCRRCCCRCR` |
| BIOS-boot / table, boot, esp, member | unchanged |

`Label sda` on root-on-ZFS: affected = `{sda, table}`. `Create sda` on
BIOS-boot: `{sda, table, boot}` — bios_grub through the table because it
lies inside the table's declared bytes; not the ESP, member or pool.

ADR-0040's pin unmoved. Workspace with WP-060's PR #382: 661/0;
`cargo xtask ci` exit 0.

## 5. Mutations

| # | mutation | outcome |
| --- | --- | --- |
| M1 | target exemption removed | killed ×3 (whole-disk pin, frame-root reach, BIOS-boot bound) |
| M2 | self-framed always descends | killed by the pre-existing ordinary-disk guard |
| M3 | wholesale write restored | killed ×3 |
| M4 | written dropped for every target | **survives the domain suite**; killed by WP-060's new planner test |
| M5 | M1 + M3 (pre-act behaviour) | killed ×3 |

## 6. Rejected

Correct the entry alone; declare nothing everywhere; widen the hop; a
per-kind truthful entry (needs the request or the topology at a
signature two other packages call — the next act on this entry, not this
one); a reach rule independent of containment (not needed).

## 7. Not established

- Anything about #347: `Wipe table` over a disk with a live pool member
  is still `Clear`.
- A multi-agent adversarial verdict. The claim most worth attacking:
  that "the target is in the set by identity" is the *only* case in
  which descent out of a self-framed extent is safe — i.e. that no
  honest step exists whose device target's carried content should *not*
  be reached. `Create` on a disk with a stale device-hosted superblock
  reaches it (Blocked, remediable), which is today's behaviour and
  ADR-0039's rule; if the owner considers that a false refusal it is a
  question for the per-kind act, not this one.
