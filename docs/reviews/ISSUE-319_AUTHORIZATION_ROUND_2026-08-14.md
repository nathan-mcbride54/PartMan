# Issue #319: the authorization half — round, 2026-08-14

Untracked session artifact, `docs/reviews` convention. Everything
load-bearing must be restated in whatever ADR lands the decision.

**Follows** `ISSUE-319_EXTENT_ABSENCE_ROUND_2026-08-13.md`, which landed
the planner half (ADR-0036, spec 12.13.0) and left the authorization
half open because **no domain-side fix survived measurement**. The thing
that killed them all has since been fixed by something else, which is
the first finding below.

Everything marked **[measured]** was run by hand in this session against
`main` at `8e03e68` (spec 13.0.0), in a detached worktree with its own
`CARGO_TARGET_DIR`. An adversarial pass is running against the candidate
and its results are not yet folded in. **No `cargo xtask ci` run backs
any of this** — `cargo test` only.

## What changed since the last round, and it changes the whole problem

The 2026-08-13 round's **finding 5** was its decisive fatal: with every
extent present, re-anchoring the ZFS label into its partition's address
space (the cross-language golden vector's own convention) defeated
`the_root_on_zfs_regression_pair_holds` — a whole-device wipe over a
live pool **constructed**. Every domain-side route died on it, because
each keyed its guard on extent *presence* while the arms consume
*address-space agreement*.

**That is fixed, by ADR-0039 rather than by anything aimed at #319.**
**[measured]**

| whole-device wipe, `root_on_zfs` | affected | pool reached | outcome |
| --- | --- | --- | --- |
| label device-framed | 6 | yes | refused `{Zfs}` |
| label **partition-framed** (the golden vector's shape) | 6 | yes | **refused `{Zfs}`** |

The mechanism is ADR-0039's `child.host == source` clause: a child whose
extent is expressed in the frame of the node reaching it is admitted, so
the label inside the destroyed member is descended into and the Backing
edge to the pool fires. The clause was written for the ADR-0037
frame-crossing shape; it closes finding 5 as a side effect.

**So the blocker on the authorization half is gone, and what remains is
narrower and sharper than the issue as filed.**

## The defect, reduced to one measured shape

Absence alone is no longer enough to hide a node — but absence *of a
frame* still is. Compose two facts, each individually lawful:

1. a node's content expressed in **that node's** address space (the
   golden vector's convention; unlawful under ADR-0037's decided rule,
   but **ADR-0037's enforcement is held**, so bodies carry it today), and
2. **that node** carrying no extent fact of its own (`extract_extent`
   returns `Ok(None)` and assembly proceeds; nothing requires an extent
   on an extent-bearing kind).

The frame is then invisible to the byte scan — its own bytes are
undeclared — and ADR-0039's bound refuses containment descent into it,
because `(Some(parent), None)` under `Containment` is exactly the clause
that stops an unlocated sibling being captured. Everything framed by it
disappears with it.

```
root_on_zfs, ZFS label expressed in the member's frame, member has no extent
whole-device wipe:  affected = 3, pool unreached, step_constructs = Ok
```

**[measured]** The whole device, over a live ZFS vdev, approved.

**And the body is lawful at every boundary.** It assembles, encodes,
decodes and recomputes; `TopologySnapshot::from_canonical_body` returns
`Ok`; and the **decoded** snapshot's own closure approves the same wipe
(`n=3`, pool unreached). **[measured]** No layer refuses it.

Two shapes the issue leads with are **no longer the sharpest**, and the
round should say so rather than repeat the filing: a create declaring
`consumed` over a buried extent-less member, and a destructive step over
the same, both **refuse** today (`n=4`, pool reached) — because the
label's own device-framed extent is caught by the byte scan and ADR-0038
brings its consumer. The defect survives only where the *frame* is the
thing that is missing.

## The candidate

`crates/domain` only, in `affected_set`'s initial scan. A node with no
extent fact is placed by **its own hashed name**:

```rust
fn declared_position(topology: &Topology, id: NodeId) -> Option<(NodeId, u64)>
```

- `Partition` → its `start_offset`, resolved through `PartitionTable.parent`
  to the device. The field's own doc already says "in the containment
  root's address space".
- `BackingSignature` → `primary_offset`, in the `host` it names.
- `FileSystem` → `superblock_offset`, in the `host` it names.
- `PartitionTable` → its device's origin.
- `ConflictingTableEntry` → `entry_start`, through its table.
- Everything else → `None`. A produced node has no position in any name,
  which is the same fact ADR-0039's `may_carry_extent` reads.

A declared range **reaches** the node when it covers that point. **No
length is ever derived** — the point is authenticated, a length would be
invented. Covering the start is sound because a node occupies its own
start, and the answer can only ever *add* reach, which is ADR-0039's
standing invariant.

**Why the name and not the fact:** `Facts.extents` is unauthenticated
body content, while `start_offset`, `primary_offset`, `superblock_offset`
and `entry_start` are hashed into the node's address (`derive_id`). This
is ADR-0036's own move — occupancy from naming fields, never from edges
— applied one layer up.

## Measured, candidate versus `8e03e68`

| measurement | before | after |
| --- | --- | --- |
| the hole: wipe over an extent-less frame | **Ok** | refused `{Zfs}` |
| the same through the **decoded body** | **Ok** | refused `{Zfs}` |
| create `consumed` over a buried member | refused | refused (member now in the set) |
| finding 5 (partition-framed, extent present) | refused | refused |
| sibling guard: ESP in set / pool in set | no / yes | no / yes |
| ordinary disk: delete, shrink | Ok | Ok |
| ordinary disk, partition targets | Clear | Clear |
| plain ext4 partition + device, plain LVM stack | Clear | Clear |
| **device targets on a disk with an extent-less occupant** | **Clear** | **Blocked** |
| whole workspace | green | **green** |

**[measured]**, every row. The single behavioural cost is the last one,
and it is worth stating precisely rather than as a category: a
device-targeted mutating operation moves to `Blocked` when the disk
carries an occupant that declares no bytes — an unlocated partition, or
a hybrid view's conflicting entry. `Blocked` is the remediable arm, and
the remediation is to supply the fact.

**That cost is coupled to issue #353 and should be decided with it.** It
arises because `canonical_ranges` claims a device target's **whole
extent** as written — the §2.1 violation #353 files. Correct that, and
a device-targeted `Label` stops claiming the bytes an unrelated occupant
sits on, and this cost largely evaporates. Landing #319's authorization
half first therefore prices in a refusal that #353 would refund.

## The candidate is rejected — four fatals, and the premise is false

The adversarial pass ran 25 agents; 12 findings survived its verify
phase, four of them fatal. The decisive one I confirmed by reading code
I wrote myself yesterday.

1. **FATAL — the offsets are not byte positions.** The candidate reads
   `BackingSignature.primary_offset` and `FileSystem.superblock_offset`
   as positions in the host they name. Their docs claim no such thing:
   "the primary signature offset **the parser fixed**"
   (`naming.rs:240`, `:249`), with no address space stated. The
   counter-example is **committed, in PR #351's own guard**: an
   end-anchored mdraid 0.90 superblock named `primary_offset: 0`
   (`protection_tests.rs:793-798`) whose extent sits at
   `1 GiB − 64 KiB` (`:853-859`) — a whole-disk disagreement between
   the hashed name and the bytes. Remove that extent, which is exactly
   #319's population, and the candidate places the node at the head of
   the disk where every table write lands: the table target moves from
   ten `Clear` to ten `Blocked{OrphanSignature}`. **[measured by the
   pass; the fixture and the field docs verified by hand]** Only
   `Partition.start_offset` states its frame, and only it survives.
2. **FATAL — naming referents are validated by nobody, so one field
   evades it.** `Topology::build` validates *edge* endpoints only
   (`topology.rs:147-167`); no layer requires `parent_table`, `host` or
   `table` to resolve, to be the right kind, or to agree with the
   containment edge. Point `parent_table` at an absent node — or at a
   real node of the wrong kind — and `declared_position` returns `None`,
   restoring the hole verbatim, on a body that decodes cleanly.
   **[measured by the pass]** The delivered *planner* already refuses
   this shape (`OccupancyGround::TableIsNotThisHosts`), so the proposed
   authorization gate would be strictly weaker than the solver's own
   check.
3. **FATAL — a point is not a span.** `covers` tests one byte, so only a
   range containing the node's declared *start* reaches it. The whole-
   device wipe closes because it starts at zero. A tail destroy, a
   sibling operation and an interior create all still construct over the
   live vdev — including **ADR-0039's own worked vector** (freed tail
   `[640,768) MiB`, label at `[512,513) MiB`) whenever the target is not
   the buried node. **[measured by the pass, reproduced by its own
   skeptic on a rebuilt non-overlapping fixture]**
4. **FATAL — it re-derives what ADR-0039 rejected, by another route.**
   That ADR killed "admit extentless containment children" on the line
   *a partition delete captured a sibling that merely lacks an extent
   fact*. The candidate leaves `descends_into` alone and reaches the
   same nodes from the initial scan instead. The committed guard still
   passes, but on a one-byte margin — its ESP sits at 1 MiB and its
   table write ends at 1 MiB. **And my own summary of the cost was
   wrong**: partition-targeted deletes and shrinks flip `Ok` → `Err`
   too on a hybrid disk, not only device-targeted operations.

**Recorded, and it is the general lesson of the last three rounds:** the
candidate was green on the full workspace, closed the filed hole at both
the pure layer and the decoded body, and was still wrong four ways. A
closure change is not evidenced by the suite it does not move.

## What survives the rejection

- **The defect stands**, in the shape measured above: an unlocated
  *frame* hides its subtree, the body is lawful at every boundary, and a
  whole-device wipe over a live vdev is approved.
- **Derived position is not generally available.** Only
  `Partition.start_offset` states its address space; signature and
  file-system offsets are parser-fixed identifiers. Any design that
  reads them as geometry is unsound, which retires this whole family
  rather than only this member.
- **A precondition surfaced that is worth its own act**: nothing
  validates a naming field's referent. The planner checks it, the domain
  does not, and every name-derived route depends on it.
- **The remaining arm is the one the issue itself proposed**: refuse
  rather than reach — an extent-bearing node that declares no bytes in
  the step's own frame makes the closure's answer unsound, so the step
  is `Indeterminate` rather than `Ok`. It needs no geometry, so findings
  1–3 do not reach it. Its cost is availability, and the 2026-08-13
  round's objection 4 already named the shape: a device's self-extent
  spans everything, so one unlocated occupant can block every
  destructive step on that disk. **Unmeasured. It should not be
  recommended until it is.**

## Open, for the decision owner

1. **Is the fail-closed arm's availability cost acceptable in
   principle**, before it is measured in detail? Every disk carrying one
   unlocated occupant would refuse destructive work until the fact is
   supplied. That is `Facts`' own stated contract, and it is a product
   decision rather than a defect fix.
2. **Should the unvalidated-referent gap be filed and fixed first?** It
   is a precondition for any name-derived route, the planner already
   guards it, and it is independently a hole.
3. **Sequencing against #353.** The over-refusal cost of anything in
   this family is inflated by `canonical_ranges` claiming a device
   target's whole extent. Fixing #353 first changes the measured cost of
   every option here.
