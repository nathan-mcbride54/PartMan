# Issue #347: a destroyed table releases its partitions — round, 2026-08-14

Untracked session artifact, `docs/reviews` convention. Everything
load-bearing must be restated in whatever ADR lands the decision.

> **HEADER NOTE, WRITTEN AFTER THE ADVERSARIAL ROUND. The candidate in
> section 3 is REJECTED, on two fatals I reproduced by hand.** Sections
> 1, 2, 7 and 8 stand and are the useful part — the defect is real, the
> `Containment` finding is real, and the population is narrower than the
> issue says. **Section 3's design must not be rebuilt without reading
> section 10.** The measured rows in section 4 are true and also
> irrelevant, because they measure the one spelling of the step that the
> predicate happens to answer correctly.

Measured by hand at **`c9cd4bb`** (current main, post-ADR-0039 and post
PR #357) in a detached worktree with its own target directory. Candidate
applied with a scripted edit and **proven applied** by `grep` before each
run. **No `cargo xtask ci` run backs this** — `cargo test` only. No
adversarial pass has run yet.

## 1. The premise holds, re-measured on current main

#347 was filed against `5b795df`, two closure changes ago. The rule is to
measure an issue before working it, so the first act was to re-take its
own measurement on `c9cd4bb`:

```
target = table, destroyed = [0, 1 MiB)   (exactly the GPT's extent)
affected set size: 2      esp: false   member: false   pool: false
step_constructs:   Ok
gate(all ten mutating operations, table target): Clear
control gate(Wipe, sda):                         Unsupported{Zfs}
```

**Unchanged from the filing.** ADR-0039's carried-content reach does not
touch it, and the reason is precise: the table *is* in the affected set,
so descent is attempted, and `descends_into`'s `(Some, Some)` clause
refuses the hop because the table's extent `[0, 1 MiB)` does not contain
the ESP at `[1 MiB, 257 MiB)`. The bound is doing exactly what ADR-0039
designed it to do. The defect is upstream of it.

## 2. The finding under the issue, which the filing does not state

**`Containment` is documented as one thing and used as two.**
`EdgeKind::Containment`'s own words are "Positional nesting inside one
addressable byte space (device → table → partition; a host carrying a
signature or file system)" (`topology.rs:26-28`).

For `partition → backing-signature` that is true: the signature's bytes
lie inside the partition. For **`table → partition` it is false**, and
the committed fixture says so in its own facts: the table's extent is
`[0, 1 MiB)` while the partitions it contains sit at `[1 MiB, 257 MiB)`
and `[512, 768) MiB` (`protection_tests.rs:106-129`). A partition is
*described by* its table, not *nested inside* it. ADR-0036 made this
sharper by re-parenting partitions onto the table node for injectivity
under hybrid views — a naming decision with a geometric consequence
nobody has had to state until now.

So ADR-0039's geometric bound reads the doc's promise, and on this one
edge the promise is not kept. **Every design in this family should be
argued against that fact rather than against the filing's framing**, and
whichever act lands should correct the edge's documentation, because a
reader who trusts it today will re-derive this defect.

## 3. The candidate, and it is small

A node whose extent is **wholly** covered by the destroyed ranges
releases everything it contains, regardless of the geometry between
parent and child:

```rust
// in the initial scan
if ranges.destroyed.iter().any(|range| range.contains(extent)) {
    wholly_destroyed.insert(id);
}
// in the fixpoint, on the propagating arms
if source_destroyed
    && (wholly_destroyed.contains(&edge.source)
        || descends_into(topology, facts, edge.kind, edge.source, edge.target))
```

It **adds** an admit condition and removes none, so it can only ever add
reach — ADR-0039's standing invariant, held by construction rather than
by argument.

## 4. Measured

| measurement | `c9cd4bb` | candidate |
| --- | --- | --- |
| the defect: wipe the GPT over a live vdev | **`Ok`**, affected 2, pool unreached | `Err(Refused{Zfs})`, affected 6, pool reached |
| `gate(Wipe, table)` on that disk | **`Clear`** | `Unsupported{Zfs}` |
| control: ordinary disk, table target, ten operations | 10/10 `Clear` | **10/10 `Clear`** |
| control: delete one partition, sibling captured | false | **false** |
| control: delete one partition, sibling's file system captured | false | **false** |
| whole committed workspace | green | **green** |

**Every row measured by execution.** The false-refusal controls are the
ones that matter: a table wipe on a disk carrying nothing protected stays
wholly `Clear`, and a partition-sized destroy does not make the table
"wholly destroyed", so no sibling is captured — the committed guard
`an_ordinary_disk_keeps_its_siblings_out_of_the_set` and
`a_sibling_esp_is_never_captured` both pass untouched.

## 5. The prior rejection, and why it does not bind this

`ISSUE-338_CLOSURE_SEED_ROUND_2026-08-13.md` rejected its `seed` design
partly on this same table-wipe measurement, reading `n=6` with the ESP
captured as re-deriving ADR-0018's committed outcome that "the ESP at
`sda1` is never captured by its sibling's pool" (0018:190-192).

That outcome is stated about **destroying `sda2`** — a sibling
relationship. When the table itself is wholly destroyed, both partitions
are released, and reaching the ESP is correct reach rather than sibling
capture. The two scenarios are distinguished here by the
`wholly_destroyed` predicate, which a partition-sized range cannot
satisfy on a table. **This round does not claim the `seed` design was
right**; it claims the ground of that rejection does not separate the two
cases, and measures the separation.

## 6. What the decision owner must still decide

1. **Is this a decided act or a defect fix?** I judge it a **decided
   act** — it changes what a step's affected set contains, which is
   exactly the ground ADR-0038 and ADR-0039 each took an ADR and a spec
   bump for. It should not land as a quiet fix on the strength of a green
   suite, and this round does not propose that it should.
2. **Whether release is computed or declared** — the closure-side
   candidate above, versus `canonical_ranges` enumerating a table's
   partitions (the issue's option 2), which keeps the closure unchanged
   but makes the gate walk the table's children.
3. **Whether `Containment`'s documentation is corrected in the same act.**
   Section 2's finding is independent of which design wins and outlives
   all of them.
4. **Sequencing against #356.** The candidate reads `Facts.extents` to
   decide whole-destruction, and extents are unauthenticated body
   content. It cannot *lose* reach that way — the invariant holds — but a
   body that under-declares a table's extent gets less release than the
   truth warrants. That is #356's territory, and the ADR should say so
   rather than leave a reader to find it.

## 7. The shapes section 4 left out — measured, with HEAD controls

Written after the fact, because leaving them unmeasured is where a
reviewer would rightly start.

| shape, table target | `c9cd4bb` | candidate |
| --- | --- | --- |
| **extent-less table** (ADR-0036's own shape) | affected 5, pool reached, `Unsupported` | **identical** |
| **whole-disk vdev**, no table | `Unsupported` | **identical** |
| **hybrid disk** with a `ConflictingTableEntry` | affected 2, CTE unreached, **`Clear`** | affected 4, CTE reached, **`Blocked{Unrecognized}`** on 4 of 10 |

**Two findings, and the first narrows the issue itself.**

1. **The defect requires the table to carry an extent.** On an
   extent-less table the closure already releases correctly at HEAD,
   because `descends_into`'s `(None, _) => true` clause admits descent
   out of an extentless source unconditionally. ADR-0036's own committed
   shape was never broken. **Neither the issue nor section 1 of this
   round says so**, and any ADR must, because it decides how much of the
   real population the defect actually covers.
2. **The hybrid cost is real but narrow, and it is not the shape that
   killed the #319 arm.** Only the four operations ADR-0038 gave a
   destroyed entry — `Wipe`, `Encrypt`, `Move`, `Shrink` — block, because
   only they declare a range wholly covering the table. **`Repair` stays
   `Clear` on both the table and device targets**, so ADR-0024's repair
   family remains reachable on a repairable device — the very property
   the rejected #319 arm defeated (`planner/src/tests.rs:3009-3011`). The
   four that block do so because destroying the table genuinely releases
   the conflicting entry, whose own arm is `Indeterminate{Unrecognized}`
   — an honest "this is not understood", not a manufactured refusal.
   The ADR must still price it.

## 8. Mutations

Asserting regressions were written first, because the section 4 probes
**print rather than assert and could not have killed anything** — a
mutation pass over print-only probes measures nothing, which is the same
gate-that-examined-nothing shape in miniature.

- **`contains` → `intersects`** in the whole-destruction test: **killed**,
  and killed by the *committed* guard
  `an_ordinary_disk_keeps_its_siblings_out_of_the_set` rather than by the
  new tests. The strictness of `contains` is load-bearing and the
  existing suite already protects it.
- **the release clause removed** entirely: **killed** by
  `assert_347_defect_closes_and_controls_hold`.

## 10. THE ADVERSARIAL ROUND — the candidate is rejected

Four lenses, four fatals, all measured by the reviewers. **I reproduced
the two decisive ones by hand before accepting them.**

### 10.1 FATAL — the predicate reads the step's *spelling*, not its bytes

`ranges.destroyed.iter().any(|range| range.contains(extent))` asks
whether **one** declared range covers the extent. The same destroyed
bytes, declared as two adjacent ranges, never satisfy it.

**CONFIRMED by hand** on the committed `root_on_zfs` fixture, table
target:

```
destroyed = [0, 1 MiB)                          affected 6, pool reached, REFUSES
destroyed = [0, 512 KiB) + [512 KiB, 512 KiB)   affected 2, pool unreached, CONSTRUCTS
identical destroyed bytes:                      true
```

This is fatal rather than cosmetic because of *where* the ranges come
from. `canonical_ranges` synthesizes a single range from the target's
extent, so the **capability gate is trivially self-satisfying and hides
the defect entirely** — which is why section 4's table looks clean. The
plan constructor, the boundary ADR-0018 makes load-bearing, decodes
`destroyed` **verbatim from an authored plan body** (`plan.rs:953`). One
line of re-declaration and the fix is gone.

Every other range predicate in the layer is union-semantic
(`intersects`). This was the first partition-sensitive one, and it sat on
the enforcing boundary.

### 10.2 FATAL — anti-monotone in the node's own declared extent

`contains` gets **harder** to satisfy as the declared extent grows, so
over-declaring subtracts the release.

**CONFIRMED by hand**, same fixture, honest destroyed range:

```
extents[table] = [0, 1 MiB)       affected 6, pool reached, REFUSES
extents[table] = [0, 1 MiB + 1)   affected 2, pool unreached, CONSTRUCTS
```

**One byte of inflation removes the refusal.** That is precisely the
lever `descends_into`'s own doc forbids — "extents are authored body
content, and a predicate that can subtract reach hands that content a
lever" — and the candidate was the first admit path whose truth is a
function of an unauthenticated number.

**And section 6.4 of this document stated the direction backwards.** It
said a body that *under-declares* gets less release; the truth is that
*over-declaring* removes it. That error is mine, it was load-bearing for
the #356 sequencing note, and it is corrected here rather than quietly
edited above.

### 10.3 SERIOUS — an unnamed re-proof obligation

ADR-0039 amended ADR-0018's non-interference theorem into "no node whose
declared extent is comparable with its reacher's and lies outside it is
ever in the set" (`0018:187-190`), and ADR-0018:210-217 makes re-proving
it a **precondition of acceptance** for any change to the edge set or its
traversal. The candidate admits containment descent that ignores geometry
entirely, so the theorem is false as written under it — the ESP's extent
is comparable with the table's and lies outside it, and the candidate
puts the ESP in the set.

Section 3 substituted "it can only ever add reach" for that obligation.
Those are different invariants: the first is about unauthenticated
extents, the second about geometry. **Whichever design lands must
re-prove or amend the theorem, and say which.** Note this cuts both
ways — capturing the ESP when the table is destroyed is arguably *correct
reach*, which would mean the theorem needs amending rather than the fix
abandoning. That is a decision, and it was missing from section 6.

### 10.4 The one that survives, and it reframes the issue

The reviewers measured that **#347's candidate already closes #356's
recorded body** — the containment/extent contradiction — because the
partition is wholly destroyed and the bypass no longer needs
`descends_into`. That is an argument for the two issues being one act,
and against the validation document's claim that option A "retires #356".

## 11. What the next design must satisfy

1. **Union semantics.** Whole-destruction must be decided over the
   *union* of declared ranges, never over any single one. Anything else
   is defeated by re-spelling.
2. **Monotone in the declared extent, not anti-monotone.** No admit path
   may become *less* true as an authored number grows.
3. **The re-proof obligation from 10.3, named explicitly.**
4. The acceptance test that catches all three is the one this family
   keeps re-learning: **can any authored field — a number, or the way a
   set is partitioned — remove the refusal this adds?** Section 4's
   measurements never asked it.

## 9. Not established here

- No `cargo xtask ci`, no acceptance sitting.
- **No adversarial round has reported yet.** One is running. Every
  predicate in this family that reached four rounds died on a fixture
  nobody had written yet; a green suite is not evidence.
- A LUKS chain is still unmeasured against the candidate.
- The candidate reads `Facts.extents` to decide whole-destruction, so a
  body that under-declares a table's extent gets less release than the
  truth warrants. It cannot *lose* reach HEAD has — the invariant holds —
  but the floor moves with authored content, which is #356's territory.
