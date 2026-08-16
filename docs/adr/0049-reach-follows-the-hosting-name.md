# ADR-0049: Reach follows the hosting name — the host-backed class enters the closure

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-409_HOSTING_NAME_ROUND_2026-08-16.md`), in which
  all three candidate routes were **built and run** rather than argued,
  and a four-mutation battery applied to the chosen one — three killed,
  and one killed only after the round added the regression it was
  missing. The reservation and its grant block, which lifts the standing
  denial on altering the closure's reach for this act only, landed first
  in PR #418. Merging is not acceptance; the decision owner has not been
  put the question in person, and this ADR is where it is put.
- Spec version: **17.0.0 — major under §0.1.** §2.1's closure sentence
  enumerates the arms and this adds a fourth.
- Work packages blocked: none. Issue #409 closes here, and with a
  correction to its own text. Issue #365's "what frames a backing extent"
  stays open; this act does not decide it, but — corrected after review —
  does not escape it either: the bound's one refusal reads that undecided
  frame, and the limit is pinned rather than claimed away.

## Context

A `BackingExtent` names the node whose bytes carry it. That field is in
its hashed address, so the relation is authenticated. But `backing-extent`
occurs in `endpoint_pair_allowed` exactly once — as the **source** of
`HostBacking` — and as the target of no edge kind at all. Measured, not
read off the table: `Topology::build` refuses
`containment(file-system → backing-extent)` with
`ForbiddenEndpoint { kind: Containment, source_kind: "file-system",
target_kind: "backing-extent" }`.

`affected_set` walks edges. A relation no edge can express is a relation
the closure cannot traverse.

Measured at `40e19d8`, after ADR-0047 and ADR-0048, on a body that
validates — a device carrying an ext4 file system, an image file on it,
the loop volume that image produces, and a live ZFS pool on the volume:

| target | gate | `Wipe` reaches |
| --- | --- | --- |
| the device | **`Clear` 10/10** | `{device, file-system}` |
| its file system | **`Clear` 10/10** | `{device, file-system}` |
| the image | `Clear` 0/10 | the pool, correctly |
| the volume | `Clear` 0/10 | the pool, correctly |

The closure works perfectly *from* the backing extent downward. What was
missing is the hop *into* it. **The filing understated the defect**: issue
#409 recorded the device target; the file-system target is `Clear` 10/10
on the same body, the same break one level down. That correction is
recorded on the issue.

This is the class INV-001 requires be discovered (loop devices) and
WIN-003 requires be discovered and managed (VHD/VHDX), plus dm-linear,
plain dm-crypt and attached images.

## The decision

> **Reach follows the hosting name.** In `affected_set`'s propagation, a
> `BackingExtent` whose `host` field names a node already in the set is
> descended into, under **its own bound** (`hosting_descends_into`):
> absence admits, an extent framed on the host admits, a frame the host's
> extent cannot be compared against admits, and the one refusal is a
> positive geometric contradiction — same frame, wholly outside.

The arm **descends only**. It carries destruction exactly when the host
is destroyed, never when the host is merely reached.

It reads the **name** to decide *whether* the relation exists — that part
needs no answer to what frames a backing extent, and ADR-0046's carve-out
is untouched. It reads the frame only to refuse a positive contradiction,
and **that clause is not authenticated for this kind**, which is a
measured limit recorded below rather than a claim of independence from
issue #365.

## What adversarial review changed

The first form of this act reused `descends_into` with
`EdgeKind::Containment`. Review measured two defects in it, both on
bodies that validate, and both are fixed here rather than filed:

1. **It was inert on its own flagship population.** That spelling carries
   containment's absent-child carve-out — `(Some(_), None) => kind !=
   EdgeKind::Containment` — so a backing extent declaring **no extent**
   was never descended into. An `ExtentLocator::Path` image has no
   contiguous device range, which is this ADR's own argument against the
   rejected route, so the natural body declares no extent at all and
   nothing requires one. Measured: the body validated and both targets
   gated `Clear` **10/10** over the live pool — issue #409's filed
   measurement, restored by removing one optional fact. Honest absence
   was failing **open**. The arm now has its own bound, in which absence
   admits, and the carve-out's justification is recorded as not
   transferring: `host` is hashed into the node's address, so a backing
   extent naming H is under H by construction and cannot be a sibling.
2. **The bound's remaining refusal is author-controlled**, because
   nothing authenticates a backing extent's frame. That is recorded as a
   named limit below and pinned by a regression at its measured cost,
   rather than described as absent.

## The routes rejected, each measured

Both alternatives were built and run. Neither is a straw man.

**Add `backing-extent` as a containment target in the pair table.**
Closes the defect at **zero reds** — and makes the honest body
**unrepresentable**. With the edge present, ADR-0041's
`containment_agrees_with_extents` compares the image's extent against the
file system's; they are in different frames, `contains` fails closed
across frames, and `validate_facts` returns
`ExtentOutsideContainmentParent`. Reframing the image's extent onto the
containment root satisfies the check — and then `Wipe(image)` reaches
`{device, file-system, …}`: a 512 MiB image file reads as destruction of
the whole disk. That is not a tuning problem. A file has no contiguous
device range; that is why `ExtentLocator::Path` exists beside `Range`,
and why the extent belongs in the file system's address space.

**Additionally change `("backing-extent", "host")` from
`ReferentRule::Open` to `Sources(CONTAINMENT)`,** so the backing extent
joins the containment forest and the frame rule applies to it. **Five
reds**, and they are the expensive ones:
`occupancy_is_read_by_geometry_and_by_name` — the single committed
witness keeping ADR-0022's occupancy reading killable by mutation —
`the_frame_rule_reaches_every_forest_at_every_depth`, which is ADR-0046's
own carve-out pin, `a_containment_edge_that_disagrees_with_the_name_refuses`,
and two pair-table/referent-rule pins. It inherits the representational
problem above on top of them.

## Measured

- **Cost: zero reds** across the workspace. `crates/capability` and
  `crates/planner` are green under the act unchanged, so no consumer-first
  pull request was needed.
- The defect closes on **both** targets: the device and its file system
  each move from `Clear` 10/10 to refusing every mutating operation, and
  the affected set reaches the image, the volume, its signature and the
  pool.
- The honest body stays lawful, with the image's extent framed on the
  file system.
- `Wipe(image)` reaches `{image, volume, signature, pool}` and **not** its
  host — the asymmetry the rejected route could not preserve.
- `cargo xtask ci` exit 0. The battery was re-run against the rewritten
  bound, not carried over from the reviewed form.

**Mutation battery**, each applied with an editor and proven applied:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | the arm removed | killed |
| M2 | the bound dropped entirely | killed |
| M3 | membership read off the backing extent instead of its host | killed |
| M4 | destruction carried from a merely reached host | **killed, but only after this round added the regression that catches it** |
| M5 | the bound's absence clause refuses instead of admitting (the reviewed defect, re-applied) | killed by the absent-extent regression |

**M4 is the round's own finding.** It survived the committed suite
because the reach-versus-destruction distinction is invisible on a chain
with no partition table: both readings put the same nodes in the affected
set, and only a *release* separates them. The round added
`the_hosting_arm_reaches_without_destroying`, which puts a GPT table and a
partition inside the image, and asserts that a `Wipe` of the disk
releases that partition while a `Label` — which destroys nothing —
reaches the image and releases nothing inside it. M4 is killed by it. A
mutation that survives is a coverage report, not a licence.

## The spec price

**Major under §0.1.** §2.1's closure sentence enumerates the arms the
affected set closes over — "downward containment, upward backing,
downward production" — and this adds a fourth, downward hosting. Every
act that amended that enumeration priced major: 13.0.0 (ADR-0039),
14.0.0 (ADR-0043), 15.0.0 (ADR-0044). This is the same shape.

**ADR-0018's theorem is restated, not asserted untouched.** Its premise
is enumerated over the *edge taxonomy* — "no backing, production or
host-backing edge may target a kind that declares an extent" — and this
arm has **no edge** and **does** target a kind that may declare an
extent. Asserting the theorem undisturbed would be an error against it.
The theorem holds for a reason that must be stated in terms: that premise
exists to protect the arms whose descent is **unbounded**, where a target
declaring an extent could capture siblings. This arm is bounded by
`descends_into` exactly as containment is, so it belongs to the bounded
family the premise does not govern, and the sibling-capture consequence
follows from geometry rather than from the taxonomy. The theorem's
membership sentence gains a fifth parenthetical saying so.

## Consequences

- The host-backed class enters the closure where it was outside it.
  **Narrower than the first draft of this ADR claimed**, and corrected
  after review measured the difference: a backing extent whose declared
  extent lies in the frame of the step's own declared ranges — a
  device-hosted `ExtentLocator::Range` image, the dm-linear shape — was
  **already** reached at base, by `seed_from_ranges`, because a destroyed
  range simply intersects its extent. What had no upward reach is a
  backing extent whose extent is *not* in that frame, or absent
  altogether: the `Path`-located image framed on its file system, which
  is the loop/VHDX population. The act closes that.
- Nothing about what frames a backing extent is decided. `named_position`
  still returns `Outside` for one, `frame_root` still returns `None`, and
  ADR-0046's carve-out and its enumeration pin are untouched — which is
  what the zero red count reflects.
- No adapter constructs a `BackingExtent` today, so the class is still
  latent in production. The pure layer is the proof and must not depend
  on producers being absent.

## Verification

`cargo xtask ci` exit 0 at the act's head. Any claim that this ADR
decides what frames or hosts a backing extent (#365), closes issue #319's
authorization half, or alters `canonical_ranges`, is an error against
this ADR.

## What stays open

- **An authored frame can still suppress the arm**, measured and pinned.
  `("backing-extent", "host")` is `ReferentRule::Open`, so
  `named_position` is `Outside`, `frame_root` is `None`, ADR-0046's frame
  rule never runs on a backing extent, and no edge may target it so the
  edge-versus-extent cross-check never sees it. An author may frame the
  extent on the host's own frame root and place it outside the host; the
  bound's one refusal then fires on a body that validates, at a measured
  cost of `Clear` **10/10** on both targets.
  `an_authored_frame_can_still_suppress_the_hosting_arm` pins that, so
  closing it is deliberate. It does **not** make protection worse than
  before this act — the arm only ever adds reach — but it is issue #365's
  undecided question reaching into this arm, and this ADR does not claim
  independence from it.
- **The two-layer disagreement is untested.** The planner's
  `destroyed_closure` propagates to any node whose naming referents
  contain a removed node, unconditionally and with no geometric bound, so
  it removes a backing extent whose host is removed in every case where
  this arm is conditional. No test in `crates/planner` or
  `crates/capability` constructs a host-backed body. Covering it is
  **WP-060's own pull request under its own grant**, and is this act's
  named obligation — the shape ADR-0048 named and discharged one act
  earlier.
- **#365's frame question**, deliberately. This act does not decide it,
  and the limit above is where it still bites.
- **The `Backing` signature-to-aggregate omission** ADR-0047 named as its
  limit is untouched and still open.
- **Issue #319's authorization half.**

## Revisit conditions

- A second kind gains a naming field that no edge kind can carry: the arm
  here is written for `BackingExtent` specifically, and a second such
  field would want the general form read off `naming_referent_rule`
  rather than a second hand-written arm.
- `("backing-extent", "host")` is ever given a `Sources` rule: this arm
  becomes redundant with the edge walk and should be removed rather than
  left as a second route to the same nodes.
