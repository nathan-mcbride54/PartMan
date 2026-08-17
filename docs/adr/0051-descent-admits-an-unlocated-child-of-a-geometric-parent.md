# ADR-0051: Descent admits an unlocated child of a geometric parent — issue #319's authorization half

- Status: Accepted
- Date: 2026-08-17. Made on the measured round of 2026-08-17
  (`docs/reviews/ISSUE-319_GEOMETRIC_DESCENT_ROUND_2026-08-17.md`), with
  a two-mutation battery, each proven applied and each killed. The
  reservation and its grant, which lifts the standing denial on altering
  the closure's reach for this act only, landed first in PR #427.
  Merging is not acceptance; the decision owner has not been put the
  question in person, and this ADR is where it is put.
- Spec version: **17.2.0 — minor under §0.1.** §2.1's descent sentence
  gains a precision it never stated. The argument is below, and it
  corrects the framing recorded on the issue.
- Work packages blocked: none. Issue #319 closes here — the planner half
  landed with ADR-0036, the authorization half lands now.

## Context

This is the oldest live hole in the tree, filed 2026-08-13, and three
predicates have died on it under adversarial review.

`descends_into`'s last arm read
`(Some(_), None) => kind != EdgeKind::Containment`: a source that
declares bytes never descends into a containment child that declares
none. Measured at `646a0b8` on the committed `whole_disk_vdev` — a
device carrying a ZFS signature that backs a live pool — removing the
signature's **one extent fact**, which nothing requires, takes every
mutating operation on the disk from **refusing 10 of 10 to `Clear` 10 of
10**:

```
baseline:             sda 0/10 Clear
unlocated signature:  sda 10/10 Clear
descent into it:      signature unreached, pool unreached
```

Honest absence fails **open**, at the one arm that needs it, against the
`Facts` doc's own promise that absence "fails closed at the arm that
needs it".

## What the round found, and why three predicates died

The arm's own comment justifies itself as preventing capture of "a
sibling that merely lacks a fact". **That is not the job it does**, and
mistaking the comment for the mechanism is what the earlier attempts ran
into.

Admitting wholesale does red
`an_ordinary_disk_keeps_its_siblings_out_of_the_set`. But reading the
fixture, the captured node is an ESP whose containment parent is the
**partition table** — and a partition table's extent is its own header
bytes, not the region it governs. ADR-0041 states exactly this and gives
it a predicate, `containment_pair_is_geometric`, under which
`partition-table → partition` carries no span claim. For an
extent-bearing partition the geometric comparison in the arm above
already refuses the hop. The old spelling was extending that refusal to
extentless children **by accident**, under a name describing something
else.

`descends_into` had never consulted that predicate. Before this act it
had two mentions in the file: its definition, and one call site in
`containment_agrees_with_extents`.

## The decision

> **Descent admits an unlocated child where the containment pair is
> geometric, and refuses it where the pair is structural.**
>
> Geometric — a device, partition, volume or multipath node containing a
> signature, file system or table — means the parent's extent is the
> region its children lie in, so an unlocated child is inside a known
> region with its position within it unstated, and refusing the hop lets
> authored absence subtract reach.
>
> Structural — a partition table containing a partition or a conflicting
> entry — means the parent's extent is its own bytes and the children lie
> beside them, so the hop is refused whether or not the child declares an
> extent, which is the same answer the geometric comparison already gives
> when it does.

The clause names the predicate that decides it rather than a kind check
standing in for one.

## Measured

At `646a0b8`. **One red**, and it is the pinned open limit ADR-0048
committed for this very issue —
`the_identity_seed_never_weakens_a_gate_on_an_absent_extent`, which
asserted `Clear` 10/10 on the unlocated-signature shape so that closing
it would be deliberate. It is rewritten in place as the closure.

**Both sibling pins survive**, which is the result that distinguishes
this from the wholesale form:
`an_ordinary_disk_keeps_its_siblings_out_of_the_set` and
`a_released_partition_refuses_only_for_what_it_carries` both pass,
because a table's pairs are structural and stay refused.

`crates/capability` and `crates/planner` are green unchanged, so no
consumer-first pull request was needed.

**Mutation battery**, each applied with an editor and proven applied:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | the clause reverted to the old kind check | killed (2 tests) |
| M2 | the structural half dropped — admit wholesale | killed (3 tests, including the new structural regression and both pre-existing sibling pins) |

M2 is the one worth stating: the act's own new regression,
`a_structural_parent_never_descends_into_an_unlocated_child`, catches it
alongside the two pins that already existed. The structural half is not
incidental to this act; it is half of it.

## The spec price

**Minor under §0.1**, and this **corrects the framing recorded on issue
#319 earlier**, which read the act as a possible unversioned conformance
fix.

§2.1's descent sentence says descent is "refused only where a child's
declared extent positively contradicts containment within one frame, and
admitted on every absence, mismatch or ambiguity". ADR-0039's clause list
carved containment out deliberately, so the sentence and the delivered
code have disagreed since 13.0.0.

The round narrows that gap rather than resolving it in one direction:

- For a **geometric** pair the sentence was right and the code deviated.
  That half is conformance.
- For a **structural** pair the refusal is real and correct, and the
  sentence **does not carry it**. An unlocated partition under a table is
  an absence, and descent is refused — which the sentence as written
  forbids.

So the sentence gains the structural precision it never stated. That is
an addition to previously unstated territory, which §0.1 prices minor,
and it is why "unversioned conformance fix" would have been wrong.

**ADR-0018's theorem is not disturbed.** Its premise is over the edge
taxonomy, which is untouched; its consequence is a no-over-reach claim,
and this act only ever *adds* reach — no node reached before is unreached
now. The descent bound moves in the admitting direction only.

## Consequences

- The whole-disk ZFS vdev shape closes: an unlocated signature is reached
  through its host, and the pool with it.
- The `Facts` doc's promise — absence fails closed at the arm that needs
  it — now holds at this arm.
- ADR-0039's clause list is amended, and the amendment records that its
  stated ground was a mis-description rather than a mistake in the
  behaviour: the behaviour it protected is kept, under the predicate that
  actually decides it.

## Verification

`cargo xtask ci` exit 0 at the act's head. Any claim that this ADR alters
`canonical_ranges`, the destroyed seed, any other closure arm, or the
endpoint-pair table, is an error against this ADR.

## What stays open

- **Issue #319's shape 3** — a device-attached label beside an extentless
  partition, with the *partition* as target — did not reproduce on the
  committed `partitioned_mdraid` fixture during this round (`md0p1` is
  0/10 before and after). It was measured on a purpose-built body in the
  2026-08-14 round and is not re-measured here. Whether it survives this
  act is unmeasured, and this ADR does not claim it closed.
- **ADR-0047's named limit**, unchanged: an aggregate carries no naming
  referents.

## Revisit conditions

- A containment pair changes between geometric and structural, or a new
  pair is added: this clause reads `containment_pair_is_geometric`
  directly, so it follows automatically — but the enumeration behind that
  predicate should be re-read when the pair table moves.
- Issue #319's shape 3 is re-measured: if it survives, it needs its own
  round, and this ADR's "what stays open" is where it starts.
