# Issue #338: the closure's single seed class — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. Everything
load-bearing is restated in the ADR that lands the decision.

Three designs were put. All three fix defect (b) at the constructor;
all three carry sustained majors, and two of them were measured to
open a new under-refusal or re-derive round two's sibling capture on a
body the delivered code authors. **No design survived intact, so this
round recommends less than any of them** — the half of (a) that is a
defect against ADR-0018's own normative text — and holds the closure
widening with its acceptance bar named.

**What is being decided.** `affected_set` has two entry routes that are
not equivalent (crates/domain/src/model/protection.rs:230-249): a node
intersecting `ranges.destroyed` enters `range_destroyed`, a node
intersecting `written_table_extents` or `consumed` enters `affected`
and nothing else. The fixpoint propagates only from `range_destroyed`
and `cascade_destroyed` — containment descent, upward backing and
downward production all gate on `source_destroyed`
(protection.rs:254-255, :259-289). Two defects follow from that single
seed class. **(a)** `canonical_ranges` emits `destroyed: vec![]` for
eight operations — `Move | Shrink | Grow | Create | Repair | Label |
Uuid | Decrypt` (capability.rs:159-170) — so its two consumers,
`protection_gate` (capability.rs:189) and the non-sized `plan` path
(crates/planner/src/lib.rs:747), get a closure with no propagating
seed. **(b)** partial destruction misses children outside the destroyed
sub-range, which survives even where `destroyed` is populated: the
sized path emits the solver's real freed tail (lib.rs:986) and still
under-reaches a label whose bytes are outside it.

## What the round established

Code, ADR and spec citations below were re-read by hand at HEAD
(`fc8b607`, spec 12.14.0). Figures from the three design passes are
marked **[round-reported]**; each pass ran in its own detached
worktree at `fc8b607`. **Only one pass ran `cargo xtask ci`** — the
`reach` pass, on a tree whose delivered patch contains zero test
functions — so no variant's clippy, traceability or deny result covers
the guards it claims. No pass ran the Linux/WSL or macOS halves.

**Finding 1 was re-run by hand at `fc8b607` rather than taken from the
passes**, because it corrects the issue's own filed text and the
session's first correction to it. Measured directly through
`protection_gate` on the committed `root_on_zfs` fixture: all eight
operations on the pool member, and a device-targeted `Shrink`, return
`Unsupported { ground: InheritedFromConsumerOrProducer }` — **not
`Clear`**. The issue body's "the closure does not run at all" and the
first correction's "reports `Clear`" are both wrong on that fixture;
issue #338 carries a second correction naming the LUKS fixture as the
one with teeth. This round's framing is the third statement of the
defect and the first that survives measurement — recorded here so the
two superseded ones are not re-derived from the issue's history.

1. **"Runs no closure at all" is wrong, and two passes corrected it
   independently.** The `written_table_extents`/`consumed` scan inserts
   into `affected` (protection.rs:240-247) and `step_constructs`
   consults every member's own verdict, so those seeds contribute
   verdicts — what they cannot do is **propagate**. Measured on
   `root_on_zfs`, all eight operations on the pool member return
   `Unsupported{InheritedFromConsumerOrProducer}` today, because the
   target's whole extent byte-intersects the nested ZFS signature.
   `Clear` is what the LUKS and frame-crossing layouts return.
   **[round-reported, twice, independently]**
2. **The fixture with teeth is the round-three killer, not the
   flagship.** On `the_luks_descent_reaches_the_pool_below`
   (protection_tests.rs:240-345) seven of the eight gates measure
   `Clear` with `Refused{Zfs}` present throughout; only `Wipe`, which
   seeds `destroyed`, refuses. `root_on_zfs` masks (a) through the
   signature's own arm. **[round-reported]**
3. **(a) splits along ADR-0018's own release list, and no round said
   so.** 0018:136-141 is normative: "*destroyed ranges*, where
   **release is destruction**: a range freed from its owner — a deleted
   partition's extent, a shrink's truncated tail, a move's source
   extent at commit". Verified verbatim. Of the eight, only **Shrink**
   and **Move** are named releases. The other six destroy nothing, so
   the ADR licenses no destroyed entry for them and their reach needs
   (b)'s widening, not (a)'s.
4. **Two further code-against-text contradictions, both verified.**
   `canonical_ranges` puts the *target's own extent* in
   `written_table_extents` (capability.rs:167), against 0018:133-135
   ("the exact ... extents of the host's table node — never the parent
   device") and §2.1's own sentence, "Table writes target the table
   node's own extents, never the parent device wholesale"
   (AGENT_BUILD_SPEC.md:110). And ADR-0018's rule 3 is route-agnostic —
   "a `BackingSignature` **in the set** brings its consumer"
   (0018:153-154), contrasted in the same paragraph with rule 4's "in
   the set **through a destroyed range**" (0018:155-156) — while the
   code gates it on destruction (protection.rs:269-279).
5. **Making the entry *truthful* is a safety regression; making it
   *conservative* is not.** Four entries measured on the committed pool
   member: today's → `Refused`; the honest empty entry → `Ok`, pool
   unreached; the solver's real freed tail → `Ok`, pool unreached; the
   whole-target-extent-destroyed entry → `Refused{Zfs}`. All 255 legal
   solved shrinks of that member construct with the pool unreached and
   are saved only by the gate. **[round-reported]**
6. **The conservative entry closes the frame-crossing hole the
   anchoring round left open.** On the ADR-0037 shape (ZFS label
   volume-framed on an LV, PV partition device-framed) it gives
   `affected=7` with the pool in the set and refuses, where every
   truthful solved shrink constructs. Neither committed guard is in its
   blast radius: both pass explicit `StepRanges` literals to
   `affected_set`/`step_constructs` and neither calls
   `canonical_ranges` (protection_tests.rs:203-215, :162-188 — verified
   by hand). **[round-reported for the figures]**
7. **The guard is one example test and it is a membership assertion.**
   `a_sibling_esp_is_never_captured` calls `affected_set`, not
   `step_constructs`, and asserts only `!affected.contains(&esp)` and
   `affected.contains(&pool)` (protection_tests.rs:201-230, verified).
   ADR-0018 lists "the no-sibling-capture property test" as an
   increment-3 obligation (0018:531-534) and MODEL-002's final bullet
   puts the re-proof duty in the spec (AGENT_BUILD_SPEC.md:378); a
   `sibling` grep over `crates/` finds no such property.
   **[round-reported]**
8. **The theorem the widenings amend, quoted.** "a step's affected set
   contains no node whose extent is disjoint from the step's destroyed,
   consumed, and table ranges, unless reached through backing or
   production from a node inside them" (0018:177-179), with the
   handover discipline "must either preserve the premise or **re-prove
   the theorem** ... before acceptance" (0018:200-203). Its committed
   consequence is stated as an unconditional outcome: "the ESP at
   `sda1` is never captured by its sibling's pool" (0018:190-192).
9. **A live panic on the unmodified tree, unrelated to #338.**
   `impossibility` is total only over Wipe/Shrink/Label/Uuid and
   `unreachable!("refused before reversal emission")` sits at
   crates/planner/src/lib.rs:715; an unsized `Create` through `plan_set`
   reaches it, masked today only by a fixture that overlaps and refuses
   first. Its own issue and its own regression, in WP-060.
   **[round-reported; the arm verified by hand]**

## Routes

- **(seed) Re-let the target, upper-bound the gate.** A step destroying
  part of its own target's extent makes that target a containment-
  descent source; rule 3 ungated; `protection_gate` runs a conservative
  `gate_ranges` for eight operations and nothing for Create/Repair.
- **(author) Content descent from a non-frame target, canonical entry
  as a typed bound.** Forward closure of the target over four edge
  kinds, containment descent refused out of a frame kind; a
  `CanonicalBound` enum with only its `Exact` arm authorable.
- **(reach) Carried-content reach.** The target's carried set — its
  content, its consumers, its products — computed from topology and
  target alone, with containment descent admitted only from a kind
  that carries content only, the predicate read off the endpoint-pair
  table (topology.rs:244-254).

**None is recommended.** `seed` re-derives 0018:190-192's outcome on a
body `canonical_ranges` itself authors; `author` opens a new
under-refusal on the whole-disk ZFS layout; `reach`'s structural claim
was falsified by two mutations that left the workspace green. Full
grounds under *Rejected, recorded*.

**Recommended act, and it fixes part of (a) only.** In `crates/domain`:
give `Shrink` and `Move` the conservative entry — the whole target
extent in `destroyed`, nothing in `written_table_extents` — and ungate
the membership half of ADR-0018's rule 3. **It does not fix (b), and it
does not fix (a) for the six non-release operations.** Both are held.

## The adversarial pass on the recommended act

1. **"Correcting the effect table is strictly weaker than what
   ships."** **Sustained against the truthful correction, not against
   the conservative one** — finding 5. It is why the act destroys the
   whole target extent rather than the solver's freed tail, and why
   emptying an entry is never part of it.
2. **"The upper-bound argument rests on monotonicity, and monotonicity
   is false."** **Sustained.** The three propagating arms each carry a
   negative guard `!range_destroyed.contains(&edge.target)`
   (protection.rs:261, :274, :284, verified), so a larger destroyed set
   can move a node out of `cascade_destroyed` and stop descent through
   it; one pass measured a concrete inversion under its own rule, with
   a baseline control monotone on a single fixture only.
   **[round-reported]** The ADR must argue conservatism per operation
   by measurement and must not derive it from CAP-005 (0018:275-280).
3. **"It fixes two of the eight operations."** **Sustained, and it is
   the headline.** Finding 3: the eight split along ADR-0018's own
   release list. Grow, Create, Repair, Label, Uuid and Decrypt destroy
   nothing; a conservative destroyed entry for them over-claims against
   0018:136-141 and was measured to turn planned label/uuid writes into
   `unsupported` with `remediation: NoneExists` on a disk carrying
   nothing protected. **[round-reported]**
4. **"Emptying `written_table_extents` deletes the accidental refusal
   that masks (b)."** **Sustained as a landing condition.** The
   conservative destroyed range is a superset of the removed write on
   the same target, so the same-frame refusal should survive and now
   fire cross-frame too (finding 6) — but that is an argument, and the
   act does not land until it is a measurement on `root_on_zfs`, the
   LUKS fixture and the ADR-0037 shape.
5. **"Rule 3's membership half changes nothing measurable."**
   **Sustained.** Ungating it cannot reach the LUKS pool, because rule 4
   stays destroyed-gated by the ADR's own words (0018:155-156, code at
   protection.rs:282-289). It is text conformance. Landing condition:
   it ships with a test that fails without it, or it does not ship.
6. **"Holding (b) leaves a live under-refusal at body
   re-validation."** **Sustained, and it is the cost.**
   `from_canonical_body` → `parse_step` re-runs the closure with no
   capability gate in the path, deliberately, because the affected set
   is not body content; a recorded grow or partial-shrink body over a
   live vdev still parses. **[round-reported]** The gate correction
   does not reach that boundary and the ADR must say so.
7. **"An ADR recording a held widening is a spec change."** **Not
   sustained.** No `AGENT_BUILD_SPEC.md` sentence changes and ADR-0018's
   text is not amended — the code moves to the text. **Sustained as a
   recording obligation**: the counter-argument (that §2.1:110 delegates
   the closure to ADR-0018 by name, so any ADR-0018-shaped act is
   requirement-shaped) must be stated and declined, not ignored.
8. **"A third package is in the blast radius."** **Sustained.**
   `crates/capability` carries its own CAP-005 enumeration asserting
   ground equality between the engine and the constructor
   (engine_tests.rs:205-285, :257-261) **[round-reported]**, and the act
   changes which ground each side reports. It must be measured before
   PR 1; if it moves, that edit is WP-050's and lands as its own PR
   first. One design hid this file entirely.

## Rejected, recorded

- **(seed).** Measured: with the design applied, `target = table`,
  `destroyed = [0, 1 MiB)` — exactly what `canonical_ranges(Wipe,
  table)` emits (capability.rs:158) — gives `n=6`, ESP captured,
  `Refused{Zfs}`; baseline `n=2`, ESP false. The committed guard passes
  vacuously because its own destroyed range `[512, 768) MiB` misses the
  table's extent `[0, 1 MiB)`. That is 0018:190-192's outcome
  re-derived by a third route, on a body the delivered entry authors.
  Also measured: non-monotonicity turning a refusal into a
  construction, and `gate(Wipe, device) = Clear` while `gate(Wipe,
  partition)` refuses — reach inversely ordered with destructiveness.
  **[round-reported]**
- **(author).** Measured on a whole-disk ZFS vdev (no table): `Label`,
  `Uuid`, `Repair`, `Create` and `Grow` go `Unsupported` → **`Clear`**,
  with `Label`/`Uuid` authorable at empty ranges, so both layers agree
  on the permissive answer over a live `Refused{Zfs}` aggregate. A
  design for #338 that opens a §2.1 reach hole is disqualified on that
  alone. Its reach is also keyed on the declared `target` field, which
  is independent of the declared ranges at the anti-forgery boundary,
  and it contributes nothing to any create step because the delivered
  sized create targets the host device (lib.rs:903-916). Its regression
  report omitted a fourth failing test, the CAP-005 agreement test
  (capability_tests.rs:151-170). **[round-reported]**
- **(reach).** Its load-bearing claim — that reading the table
  discharges 0018:200-203 by construction — is false as delivered: the
  predicate inspects only Containment pairs, while Backing, Production
  and HostBacking are admitted unconditionally. Two mutations measured,
  each leaving the whole workspace green: adding `("aggregate",
  "partition")` to Production (topology.rs:259) captures the disjoint
  ESP; adding `("volume", "partition-table")` to Containment
  (topology.rs:252-253) silently switches the fix **off** for both
  flagship chains. Its delivered patch contains zero test functions, so
  its `xtask ci` green and its self-attack cover a tree without the
  guards. **The closest of the three**, and the constructive remedy —
  bound per edge *target* across all four propagating kinds, derive the
  frame set rather than hand-authoring it — is named and unmeasured.
- **Widening (1)** (descend from `range_destroyed`): reds
  `a_sibling_esp_is_never_captured`. **Widening (2)** (descend from
  `affected`): reds it and the flagship. **Widening (3)** (descend only
  from fully-covered sources): green, and **inert on partial
  destruction** — it fixes neither defect. **[round-reported]**

## Decision carried forward

**Act: the release-operation correction in `crates/domain`, alone.
Hold the widening.** This is the ADR-0037 pattern — rule recorded,
enforcement held — applied one layer down: **defect fixed, reach
held.** Say plainly in the ADR that #338 stays open on (b) and on six
of the eight operations, and that this PR must not close it.

**Spec price, argued.** The act is a **defect fix against ADR-0018's
own stated closure and effect table, not a spec change**: 0018:136-141
already names a shrink's truncated tail and a move's source extent as
destroyed, and 0018:153-154 already states rule 3 on set membership.
No `AGENT_BUILD_SPEC.md` sentence changes, so **no §0.1 bump**
(AGENT_BUILD_SPEC.md:17). Counter-argument recorded and declined:
§2.1:110 delegates the closure to ADR-0018 by name, so a reader may
price any ADR-0018-shaped act as requirement-shaped — declined because
the ADR text is unchanged and the code is being brought to it; if the
register disagrees it is **minor**, never major, since nothing weakens.
Not a §1.11 filing either: that register needs two conflicting
requirements, and this is code against text.

**PR sequencing, three packages.** (0) **Governance PR** reserving
`docs/adr/0038-*.md` in WP-010's `owned-paths-reserved` — the block
ends at 0037 (WP-010.md:40, :76), the checker reads the assignment from
the base revision, and a `Governance:` change may edit only
`docs/work-packages/WP-*.md` (AGENTS.md:30-32); the a6341e6 → 1d03cd3
precedent. (1) **WP-010** — ADR-0038, CHANGELOG, the two domain
corrections, the regressions. No signature changes, so the other two
packages compile unchanged. (2) **WP-050**, *before* PR 1 and only if
measurement shows it needed: `crates/capability`'s CAP-005 enumeration
(objection 8). (3) **WP-060** — the `lib.rs:715` panic, its own issue,
independent of #338. Under CONTRIBUTING.md:80 one PR per package;
a PR that must land red is not one-package, whatever compiles.

**Evidence obligations.** Regressions on the LUKS fixture and the
ADR-0037 frame-crossing shape, not `root_on_zfs`, which masks (a)
(finding 2). Both committed guards re-measured on **membership**, not
merely re-run green — the flagship's create half is a size-2 smallness
guard and the sibling test asserts membership only (finding 7). A
false-refusal control on each new refusal. Objections 4 and 5 as
landing conditions. Mutations applied with `Edit` and proven applied
before each run. `cargo xtask ci`'s **real exit code** checked directly,
not through a pipe — no pass in this round did so on a tree carrying
its own guards. Bare IDs on Requirements lines; the traceability map
regenerated last.

**The acceptance bar for the held (b) round, set now.** Whatever
widening is proposed must survive, as red-then-green measurements, all
four: widening (1) applied on top must still red the sibling guard;
Mutation A (`("volume", "partition-table")`) must not silently disable
it; Mutation B (`("aggregate", "partition")`) must not silently capture
a sibling; and the whole-disk ZFS layout must not lose a gate. Two
designs died on the third and fourth of these.

**Revisit conditions.** The (b) widening lands. ADR-0037's condition
(0037:226-229) fires only "if the fix makes the closure seed from
something other than `destroyed`" — this act populates `destroyed` for
two more operations and changes no seed class, so I read it as **not
firing**; that is a judgment, recorded as one. Also: a kind becomes
both a Containment source and target (0037:230-232); or `plan_repair`
stops targeting a physical device (lib.rs:1279), which is the single
fact that makes every carried-content design a measured no-op for the
entire repair family.

## What this unblocks — and what it does not

It does **not** unblock issue #319's authorization half. That half is
blocked on (b) — the reach — and (b) is held here, so #338 must stay
open and #319 must not be closed against it. Nor does the act touch
#319's own finding: `canonical_ranges` reads
`facts.extents.get(&target)` (capability.rs:151), so an extent-less
target still yields an empty entry under the conservative arm and the
closure still reduces to `{target}`. Extent absence remains silent, and
the `reach` pass measured a fourth adjacent hole there — a wholly
destroyed partition whose backing signature carries no extent leaves a
live pool unconsulted. **[round-reported]** Reconcile with
`docs/reviews/ISSUE-319_EXTENT_ABSENCE_ROUND_2026-08-13.md` before
anyone calls either issue closed.

What it does buy: for `Shrink` and `Move`, the frame-crossing hole
ADR-0037 priced as an accepted negative — "the frame boundary is safe
only where a destroyed seed exists — which is issue #338, not this
ADR" (0037:167-169) — is closed at both consumers, because those two
operations now have the seed. That is the whole of the delivery, and
it should not be described as more.

## Open, for the decision owner

1. **Is the six-operation half acceptable held for one more cycle?**
   Today seven of ten gates on the LUKS stack are `Clear` with
   `Refused{Zfs}` present. Every measured design that closes them also
   over-refuses, re-derives sibling capture, or opens a new hole.
2. **Which discriminator does the (b) round pursue?** Carried-content
   with a per-edge-target bound and a derived frame set is the
   front-runner and is unmeasured. Nothing about it is green.
3. **MODEL-003 (AGENT_BUILD_SPEC.md:382).** Any closure widening
   changes what an already-signed body decodes to with no version
   signal separating "forged" from "valid under the old closure". All
   three designs owed this and none discharged it; the held round must.