# ADR-0047: Inheritance ascends the name — an omitted edge is not an escape from device scope or a producer

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-397_NAMED_INHERITANCE_ROUND_2026-08-16.md`,
  single-author with a six-mutation battery, each mutation proven applied
  by `git diff`, three killed and three surviving as recorded proofs;
  committed under WP-000 beside this act). The reservation and its grant
  block landed first in PR #410, as ownership requires. Merging is not
  acceptance; the decision owner has not been put the question in person,
  and this ADR is where it is put.
- Spec version: **15.3.0 — minor under §0.1.** The argument is made
  below.
- Work packages blocked: none. Issue #397 closes here as filed.
  ADR-0045's named limit is discharged. Issues #319, #392, #365 and #409
  are untouched and are named below where this act's reasoning meets
  theirs.

## Context

`protection::device_scope_verdict` inherits a containment root's own arm
— a physical device's transport arm, and since ADR-0045 a multipath
node's detection-only refusal — by walking containment **edges** upward.
`producer_verdict` folds a node's producers by walking `Production` and
`HostBacking` **edges** inward. Neither reads the naming relation.

A body may omit an edge. It may not omit a naming field: `FileSystem.host`,
`BackingSignature.host`, `PartitionTable.parent`, `Partition.parent_table`
and `Volume.producer` are each in their node's hashed id preimage, so
removing or altering one produces a different node at a different
address rather than the same node with a hidden host.

The consequence, measured at `eaa99e8` on the fixture ADR-0045 committed
as its own named limit (`content_on_a_multipath_node_inherits_its_detection_only_refusal`):

| body | gate on the xfs, ten mutating operations |
| --- | --- |
| `multipath-node → file-system` edge present | `Unsupported{InheritedDeviceScope}` ×10 |
| edge omitted, `host` still names the multipath node | **`Clear` ×10** |

The same shape holds for a `RecognizedRemote` physical device — the
network-block-device non-goal — and for a volume naming a ZFS aggregate
as its producer with the `Production` edge omitted.

ADR-0043 closed exactly this class of escape for *release*, reading
`Partition.parent_table` rather than the containment edge, and recorded
the reason in one sentence: an omitted edge is not an escape. The
inheritance verdicts never followed.

## The decision

> **A node inherits along the relations its own name declares, as well as
> along the edges the body authors.**
>
> 1. **Device scope ascends the named containment parent.** In
>    `device_scope_verdict`'s upward walk, a node's
>    `NamedPosition::Inside` referent — the single containment-naming
>    field per kind that `naming_referent_rule` classifies, already
>    delivered as `named_position` — is pushed alongside every incoming
>    containment edge, and a node with such a referent is not a
>    containment root.
> 2. **The producing relation ascends the named producer.** In
>    `producer_verdict`, the producers a node's name declares are folded
>    alongside the sources of incoming `Production` and `HostBacking`
>    edges. The set of naming fields that count is read off
>    `naming_referent_rule` — a field whose rule names only producing
>    edge kinds — never from a second copy of the field list beside the
>    predicate.
>
> Both verdicts continue to fold with `worst` over the whole set. An
> added ancestry can therefore only ever add refusal, and a body whose
> name and edges agree answers exactly as it did before.

`affected_set` is untouched. No descent bound moves and the edge set is
unchanged.

## Why this shape and not a comparison

ADR-0046 derives a frame from a name and **compares** it with the
declared `host`, refusing the disagreement. That is the right instrument
where the body states the same fact twice and the two can contradict.

Inheritance is not that. There is no second claim to contradict: an
absent edge asserts nothing. Refusing a body for an omitted containment
edge would refuse the honest partial captures ADR-0041 deliberately
admits, and would turn this act into a body-format change with a golden
vector regeneration behind it. Adding an ancestry adds a refusal and
nothing else, which is why the act is a union rather than a check, and
why it needs no `FactError` arm, no schema movement and no vector.

## Measured

At `eaa99e8`, in a detached worktree outside the checkout with its own
target directory.

- **Cost: one red across the whole workspace** — ADR-0045's own pinned
  named limit, the test that asserts the defect. Its revisit conditions
  name this exact rewrite. The producer half cost **zero** additional
  reds.
- `cargo xtask ci` **exit 0** after the rewrite and the new regressions:
  637 annotations, 50 structured evidence rows, 85 requirements, 275 spec
  references, 673 live tests.

**Mutation battery**, each applied with an editor and proven applied by a
non-empty `git diff`, the workspace suite run:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | the named ascent removed from `device_scope_verdict` | killed: the inverted pin and the three-arm test |
| M2 | `named_producers` dropped from `producer_verdict` | killed: the three-arm test's ZFS arm |
| M3 | the non-empty guard removed from the kind test | **survives** — see below |
| M4 | `.all` weakened to `.any` over the rule's kind list | **survives** — see below |
| M5 | `Containment` admitted as a producing kind | killed: four tests, two of them the pre-existing decoy-parent regressions |
| M6 | the named parent pushed without clearing the root flag | **survives** — see below |

**The three survivors are proofs, and are recorded rather than patched**
— ADR-0046's precedent, where a surviving mutation whose premise was
unconstructible was a reason to collapse a branch and say so.

- **M3.** Every `(kind, field)` pair `NamingFields::naming_referents`
  can emit has an explicit row in `naming_referent_rule`, so the
  catch-all `Sources(NONE)` is unreachable for a real referent. Even were
  it reached, `naming_referent_kind_allowed`'s `any` over an empty slice
  is `false`, so `Topology::build` would refuse the node outright. Two
  independent reasons; the guard is kept because both are claims about a
  *different* function, and a cross-module unreachability argument is the
  kind that rots.
- **M4.** Every rule's kind list is homogeneous — `CONTAINMENT` is
  `[Containment]`, `BACKING` is `[Backing]`, `PRODUCING` is
  `[Production, HostBacking]` — so the two quantifiers agree row for row
  on the delivered table. `.all` is kept as the fail-closed spelling: a
  future mixed list would have to be read deliberately rather than
  admitted by one member.
- **M6.** No kind with a non-`Permitted` `own_arm` can be a containment
  *intermediate*. Physical devices and multipath nodes are containment
  sources only; aggregates appear in no containment pair; backing
  signatures and conflicting table entries are targets only, never
  sources. The kinds that can be intermediates — partition tables and
  partitions — are `Permitted` by `own_arm` unconditionally. So counting
  an intermediate as an extra root cannot change the fold. Read off the
  endpoint-pair table and `own_arm` together, not from prose.

## Options considered, and rejected

- **Compare the name against the edge and refuse the disagreement.**
  Rejected: it refuses honest partial captures ADR-0041 admits, and it
  prices as a body-format change. Argued above.
- **Read the name only, dropping the edge walk.** Rejected: it would
  *subtract* reach wherever a body authors an edge its names do not carry
  — the `Backing` and `PlatformMembership` relations name nothing — and
  ADR-0039's standing rule is that a predicate able to subtract reach
  hands an author a lever.
- **Fix device scope and leave the producing relation.** Rejected: the
  issue asks the same act to decide both or say why not, and the ZFS
  producer arm is the one with a live pool behind it. Splitting them
  would leave the cheaper escape open while the ADR read as closed.
- **Widen `crates/capability`'s `multipath_scoped` in the same act.**
  Rejected as WP-050's to decide, exactly as the issue records.

## The spec-price argument

**Minor under §0.1.** No existing requirement's text narrows.

§2.1 fixes the per-node verdicts by reference — "three-valued and total
with an `Indeterminate` residual — never `Permitted` by default —
computed from ADR-0018's named two-layer helper evidence contract" — and
delegates the contract's content to ADR-0018. This act extends that
contract's device-scope decision (ADR-0018:243-267) to ancestry the name
declares. Nothing in §2.1's sentence changes, and no numbered requirement
gains or loses a duty; a population of bodies that previously answered
`Clear` now answers with an inherited refusal, which is an addition to
what the contract reaches.

**The pricing this ADR does not claim.** It would be convenient to
exempt the act from §2.1:117 wholesale on the ground that `affected_set`
is untouched. Only that sentence's *reach* half is inapplicable. Its
verdict clause is precisely what an inheritance population is priced
against, and it is priced against it here rather than waved past.

**ADR-0018's non-interference theorem is not disturbed.** Its premise is
over the edge taxonomy — which pairs may target a kind that declares an
extent — and this act changes no pair, no edge kind and no descent bound.
The theorem's re-proof obligation (ADR-0018:236-241) fires where the
premise moves; it does not fire here, and this ADR does not claim to
have re-proved it.

## MODEL-003

No schema version moves. No body field is added, removed or reinterpreted;
no hashed artifact changes shape; the golden vector is unmoved and
`SCHEMA_VERSION` stays at 1. The act is a change to two derived verdicts
over an unchanged body.

## Consequences

- A body that names a multipath node, a recognized-remote device or a
  producing aggregate now inherits that node's arm whether or not it
  authors the edge. The network-block-device non-goal is no longer
  escaped by dropping one edge.
- `crates/capability`'s `multipath_scoped` still reports
  `Reason::MultipathDetectionOnly` for the node and its members only, so
  content on a multipath node refuses through the protection gate with
  the inherited device-scope ground rather than the multipath reason.
  Truthful, and narrower than the reason a reader might expect; widening
  it is WP-050's, unchanged by this act.
- `producer_verdict`'s doc comment kept its issue #355 justification —
  nothing bounds a node's producer in-degree — which remains correct for
  the edge half and is now joined by at most one named producer per node.

## Verification

`cargo xtask ci` exit 0 at the act's head. The three new or rewritten
regressions are: the ADR-0045 pin inverted in place, so the discharge of
its named limit is visible in the diff rather than asserted; the
three-arm `an_omitted_edge_is_not_an_escape_from_inheritance`, covering
multipath scope, the `RecognizedRemote` transport arm and the ZFS
producer, each on a body that builds and whose facts validate; and
`removing_any_one_containment_edge_never_weakens_a_verdict`, an
enumeration over three committed layouts that drops each containment edge
in turn and asserts no verdict weakens — the lens, because a fixture is
one shape and a partial fix passes every fixture written against the kind
it handles.

Any claim that this ADR decides what frames or hosts a backing extent
(issues #365 and #409), widens the destroyed seed (#392), or closes issue
#319's authorization half, is an error against this ADR.

## What stays open

- **The `Backing` signature-to-aggregate omission.** `NamingFields::Aggregate`
  carries no naming referents at all, so an aggregate is
  name-unrecoverable in principle and this act's technique cannot reach
  it. A signature whose `Backing` edge to its aggregate is omitted leaves
  the aggregate unreached. This is the act's **named limit**, stated here
  rather than left for a reader to discover, and it is why this ADR does
  not say "the omitted-edge escape is closed".
- **Issue #409**, measured the same day: a backing extent's host is on no
  edge at all — `Topology::build` refuses `containment(file-system →
  backing-extent)` — so ascending the name does not reach it, and the
  whole host-backed class keeps the reach break that issue records.
- **Issue #319's authorization half**, and the §2.1:117-versus-`descends_into`
  conformance question recorded on it. Untouched here.

## Revisit conditions

- A naming field is added whose rule names a mixed set of edge kinds:
  M4's equivalence ends and the `.all` spelling becomes load-bearing
  rather than defensive — re-read it deliberately.
- A kind is added that is both a containment target and a containment
  source **and** carries a non-`Permitted` `own_arm`: M6 becomes
  observable and the root flag's handling needs its own regression.
- `NamingFields::Aggregate` gains a naming referent: the named limit
  above becomes reachable and should be closed in the same shape.
