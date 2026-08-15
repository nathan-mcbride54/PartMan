# Issue #338: the held half — the reach round, 2026-08-14

Untracked session artifact, `docs/reviews` convention. Everything
load-bearing must be restated in whatever ADR lands the decision.

> **Decided, and this document is superseded in one part.** The decision
> owner chose the **wide** act (variant U), a **major** bump (13.0.0),
> and separate issues for the three residuals — filed as #347, #348 and
> #349. **The predicate written below is not the one that shipped.** The
> adversarial pass killed it, and the replacement, with two further
> fatals found and fixed during implementation, is recorded in
> **ADR-0039** together with the four rejected versions and what each
> was measured to do. Read the ADR for the delivered rule; read this for
> how the round got there.

**Follows** `ISSUE-338_CLOSURE_SEED_ROUND_2026-08-13.md`, which acted on
the release half (ADR-0038, PR #345) and **held** the rest behind a
four-part acceptance bar. This round works that held half: defect **(b)**
— partial destruction missing children outside the destroyed sub-range —
and defect **(a)** for the six operations that destroy nothing.

Every figure below marked **[measured]** was run by hand in this session
against `main` at `5b795df` (spec 12.14.0), in detached worktrees outside
the repository, each with its own `CARGO_TARGET_DIR`. The previous
round's figures were treated as leads and re-measured; where they
disagree, this document says so. An eight-agent adversarial pass ran
against the first proposal and **killed it with two fatals**; both were
reproduced by hand before the design was changed, and the replacement was
re-measured against the attackers' own harnesses. **No `cargo xtask ci`
run backs any of this yet** — these are `cargo test` measurements on
scratch trees.

## The headline, which is worse than the issue says

`PlanStep::mutating_declared` — **the constructor `parse_step` calls when
a recorded plan body is re-validated** (`plan.rs:912`, `step.rs:391`,
closure at `step.rs:417`) — accepts a declared partial shrink that
truncates 128 MiB off a live ZFS vdev:

```
declared partial shrink over the pool member
  (root_on_zfs, destroyed = [640,768) MiB, ZFS label at [512,513) MiB)

  HEAD (5b795df):  CONSTRUCTS
  proposal:        refused: Reached { node: <pool>, verdict: Refused { ground: Zfs } }
```

**[measured]** No capability gate sits in that path, deliberately — the
affected set is not body content. ADR-0012's unrepresentability axis is
therefore **not** discharged at the body boundary for this shape today.
That is the reason to act on (b) rather than hold it again.

## What this round established

1. **(b) reproduces exactly as filed.** `root_on_zfs`, member 256 → 128
   MiB, `destroyed = [640,768) MiB`: affected set size **2**, signature
   unreached, pool unreached, `step_constructs` = `Ok`, while
   `node_verdict(pool)` is `Refused { ground: Zfs }`. **[measured]**
2. **(a) for the six reproduces on the LUKS fixture.** `Create`, `Grow`,
   `Repair`, `Label`, `Uuid`, `Decrypt` gate `Clear` with a live
   `Refused{Zfs}` pool below; the four destroying operations refuse.
   Exactly what ADR-0038's pin asserts. **[measured]**
3. **New — three of the four negative guards are structurally dead, and
   the whole delta is the containment arm.** `!range_destroyed.contains(&edge.target)`
   at `protection.rs:283` and `:292` guards the `Backing`-substrate and
   `Production`/`HostBacking` arms, whose targets are `aggregate`,
   `encryption-layer` and `volume` — and `snapshot.rs:420-431` **refuses
   an extent on exactly those kinds plus `multipath-node`**. A node with
   no extent can never enter `range_destroyed`, so on any decodable body
   those two guards can never fire. **[measured, and read off the decode
   rule]** Only the containment arm's guard is live.
4. **New — the committed closure is non-monotone, with the inversion
   measured on the delivered code.** The previous round sustained
   "monotonicity is false" against a *rejected design* and reported the
   inversion second-hand. On a LUKS chain whose mapper volume carries a
   declared device-framed extent:

   | declared `destroyed` | affected | pool reached | constructs |
   | --- | --- | --- | --- |
   | `[1 MiB, +16 KiB)` — the LUKS header alone | 6 | yes | **no** |
   | `[1 MiB, +512 MiB)` — the whole partition, a strict superset | 4 | **no** | **yes** |

   **A strictly larger destroyed range reaches strictly less, and the
   more destructive step is the one that constructs.** **[measured]**
   **Scope, stated honestly:** finding 3 means this shape is *not*
   decodable — a volume may not carry an extent in a body. It is
   reachable through `TopologySnapshot::assemble` (`snapshot.rs:91`),
   which enforces no such rule, and that is the constructor
   `crates/planner` and `crates/capability` use. So it is a live hazard
   for in-process callers and a latent one for any capture adapter that
   records an LV's extent, but it is not a body attack.
5. **New — the delivered closure fails the previous round's own
   Mutation B, and the property MODEL-002 requires does not exist.**
   Adding `("aggregate","partition")` to the `Production` pair table
   (`topology.rs:259`) with a `pool → esp` edge: HEAD captures the
   disjoint ESP on the committed sibling-guard body (n=6,
   `esp_captured = true`). **[measured]** MODEL-002
   (`AGENT_BUILD_SPEC.md:378`) requires the no-sibling-capture theorem
   "re-proved under the extended edge set **as a property test**";
   ADR-0018:531-534 lists it as an increment-3 obligation; a `sibling`
   grep over `crates/` finds two example tests, no property, and the
   workspace carries no property-testing dependency at all. **[measured]**
6. **New — (c), a third under-reach, live at HEAD and fixed by nothing
   proposed here.** Destroying a partition table's own extent releases
   every partition it describes, and the closure reaches none of them:
   wipe the GPT on `root_on_zfs` (`destroyed = [0,1 MiB)`, target =
   table) gives n=2 — table and device — with the ESP, the member **and
   the pool** unreached, and the step constructs. **[measured]** ADR-0018
   says "release is destruction" (0018:136-141); a table wipe is the
   largest release there is.
7. **New — two forgery holes at HEAD that the act closes as a
   side-effect.** A ZFS label whose declared extent has **length 0**, or
   whose start+length **saturates u64**, is missed by the byte scan
   (`HostRange::intersects`, `protection.rs:41-45`) and HEAD constructs
   the (b) shrink over it; under the act the label is reached by descent
   anyway and the step refuses. **[measured]** The decode boundary
   accepts all three shapes — zero length, a `host` naming no node, and
   `start + length` overflowing — with `decode/validate/recompute: Ok`
   and the body hash stable. **[measured]** That belongs on an issue of
   its own: `extract_extent` (`snapshot.rs:453-472`) checks only that
   `host` is 32 bytes and the numbers are unsigned.
8. **§2.1:110 is the sentence that has to move, so this half is a spec
   change, and four normative sites carry the same words.** The spec:
   "a mutating step's affected set closes over the substrate it destroys
   (**downward containment bounded by the destroyed ranges**, upward
   backing, downward production)" (`AGENT_BUILD_SPEC.md:110`). ADR-0018
   rule 2: "**Downward containment, range-bounded:** every node whose
   host-qualified extent intersects a destroyed range joins the set"
   (0018:151-152). The ADR's Decision section repeats it (0018:481-483),
   and so does the §0.3 changelog row for 11.0.0
   (`AGENT_BUILD_SPEC.md:54`). Defect (b) **is** that bound: the label at
   `[512,513) MiB` is not in `[640,768) MiB`. **Any fix to (b) falsifies
   all four sentences.** ADR-0038 was correctly priced as code-to-text;
   this half cannot be, and ADR-0038's pre-commitment that the price is
   "**minor**, never major" (0038:196-198) does not transfer — it was
   reasoning about an act that changed no text.
9. **The same sentence is already violated in the other direction, and
   the whole-disk gates depend on the violation.** §2.1:110 ends "Table
   writes target the table node's own extents, never the parent device
   wholesale, which is what keeps a protected member's siblings
   unconstrained." `canonical_ranges` puts the *target's own* extent in
   `written_table_extents` for the six operations (`capability.rs:179`),
   which for a device target is the parent device wholesale. On a
   whole-disk ZFS vdev all ten mutating gates refuse **because** of that
   over-claim. **[measured]** **Ordering constraint: the reach fix lands
   first, that correction after, never the reverse** — inverting it is
   exactly how the previous round's `author` design moved five operations
   from `Unsupported` to `Clear` over a live pool.
10. **MODEL-003 is dischargeable without a schema bump, and no vector is
    affected.** Re-validation runs `parse_step` →
    `PlanStep::mutating_declared` → the closure → a typed `StepRefusal`
    wrapped as `PlanSchemaError::Step`. A body lawful under the old
    closure that reaches a protected node under the new one therefore
    fails to decode **with a typed artifact** — MODEL-003's own "explicit
    rejection" arm, which the plan schema already uses for retired
    versions 1–3 (`plan.rs:342-346`). Bumping `LINKED_SCHEMA_VERSION`
    (4, `plan.rs:54`) would additionally refuse every *lawful* old body:
    strictly worse, and it buys nothing. `body_vectors.rs` and the shared
    vectors stay green under every variant measured. **[measured]** The
    ADR must still record the direction that has no artifact at all: a
    body that was **refused** under the old closure and is *still*
    refused is fine, but a helper holding a hash-frozen reversal
    advertisement across the change has no version signal — that is the
    residual, and it is named below.

## The act

`crates/domain` only. One predicate plus one enumerated property.

```rust
/// Whether descent may cross this edge. A child is refused only where
/// the declared geometry positively contradicts containment: both
/// extents declared, in the same frame, and the child outside the
/// parent. Anything else is admitted, so the closure can never reach
/// less than it does today.
fn descends_into(facts: &Facts, source: NodeId, target: NodeId) -> bool {
    let (Some(parent), Some(child)) = (facts.extents.get(&source), facts.extents.get(&target))
    else { return true };               // one side declares no bytes
    if child.host == source { return true }        // framed by the node reaching it
    if child.host != parent.host { return true }   // different frames: not comparable
    parent.contains(child)
}
```

`Containment`, the `Backing` substrate half, `Production` and
`HostBacking` all become `source_destroyed && descends_into(..)`, and the
three negative guards are deleted. Two sizes:

- **D (narrow).** Descent runs from the destroyed classes only. Fixes
  (b). **Holds (a)-six.** Whole workspace green.
- **U (wide).** D plus `|| affected.contains(&edge.source)` — descent
  also runs from nodes the target/written/consumed seeds reach, so a
  mutating step reaches the content its target carries. Fixes (b) **and**
  (a)-six. Reds exactly one committed test: ADR-0038's held-half pin.

**The safety property that makes this defensible: the act can never
remove reach.** Every arm HEAD has is preserved and two are widened; the
predicate refuses only on a positive geometric contradiction, and a
missing, foreign-framed or unauthenticated extent admits. Verified
against the attack harnesses below.

**And the premise, generalized and enumerated** — the second half of the
act, which is what makes the pair table's role visible:

> No `Backing`, `Production` or `HostBacking` pair may target a kind that
> can carry an extent — where "can carry an extent" is read off
> `snapshot.rs:420-431`, not hand-authored.

Measured: **0 violations on the committed table; 1 violation under
Mutation B** (`Production: aggregate -> partition`). **[measured]** This
is ADR-0018:180-184's premise ("no backing or production edge targets a
physical device") in the form MODEL-002:378 asks for, and it is what
stops unbounded descent on those three arms from ever reaching a node
with declared bytes of its own.

## Measured, both sizes

| measurement | HEAD | D | U |
| --- | --- | --- | --- |
| (b) partial shrink: affected / pool reached | 2 / no | **5 / yes** | **5 / yes** |
| (b) at `mutating_declared` (the body boundary) | **constructs** | refuses | refuses |
| LUKS six (`Create Grow Repair Label Uuid Decrypt`) | Clear | Clear (held) | **all refuse** |
| LUKS four (`Shrink Move Encrypt Wipe`) | refuse | refuse | refuse |
| sibling guard body: n / ESP in set / pool in set | 5 / no / yes | 5 / no / yes | 5 / no / yes |
| whole-disk ZFS, device and signature targets | 10/10 refuse | 10/10 refuse | 10/10 refuse |
| ESP, plain ext4 partition + device, plain LVM stack | Clear | Clear | Clear |
| finding 4's inversion (16 KiB vs 512 MiB destroyed) | 6 → **4** | 6 → 6 | 6 → 6 |
| forged label frame on a volume-hosted signature (5 shapes) | refuse ×5 | **refuse ×5** | refuse ×5 |
| zero-length / saturating label extent on the (b) shrink | **constructs** | refuses | refuses |
| two-leg VG, LUKS-erase and whole-device shapes (8, plus the decoded body) | refuse ×9 | refuse ×9 | refuse ×9 |
| whole workspace | green | **green** | one red: ADR-0038's pin |

**[measured]**, every cell. U's single red is
`release_operations_seed_the_closure_and_the_others_do_not`
(`protection_tests.rs:574`), which ADR-0038 wrote *as* the decision gate
— "pinned so the held half cannot drift shut silently or open further
without a decision". Landing U means rewriting that pin as part of the
decision; landing D leaves it green.

## The acceptance bar the previous round set

1. **"Widening (1) applied on top must still red the sibling guard."**
   **Met.** With the bound removed, five committed tests red:
   `a_sibling_esp_is_never_captured`,
   `ungating_rule_three_membership_never_captures_a_sibling`,
   `the_root_on_zfs_regression_pair_holds`,
   `a_release_over_an_unprotected_target_still_constructs`, and the
   ADR-0038 pin. **[measured]**
2. **"Mutation A (`("volume","partition-table")`) must not silently
   disable it."** **Met.** Every figure unchanged. **[measured]** This is
   the mutation that switched the previous round's `reach` design off; a
   geometric per-target bound never reads the pair table.
3. **"Mutation B (`("aggregate","partition")`) must not silently capture
   a sibling."** **Met on "silently", not on "capture" — and the round
   recommends accepting that, with the reason measured.** The act
   captures the ESP exactly as HEAD does (n=6). Making the closure
   *immune* is possible and was measured: bound descent out of an
   extentless source to its own framed content only. **That variant was
   killed** — see the adversarial pass — because produced nodes can never
   carry an extent (finding 3), so the bound falls back on the child's
   declared `extent_host`, which nothing authenticates: moving that one
   field turned `gate(Wipe, pv)` from `Unsupported` to **`Clear`** over a
   live pool on the ADR-0037 shape. **[measured]** Immunity costs a
   forgery hole. The premise property test answers the word that matters:
   the mutation is no longer *silent* — it reds.
4. **"The whole-disk ZFS layout must not lose a gate."** **Met.** 10/10
   refuse under both, on device and signature targets. **[measured]**

Two more, owed rather than set:

5. **Monotonicity in the declared ranges.** Restored: no negative guard
   remains and the predicate never reads `ranges`, so more declared
   destruction can only grow the closure. Structural, read off the
   delivered signature; an empirical control agrees (4 624 range pairs,
   0 violations) and finding 4's inversion is gone. **[measured]** This
   retires the previous round's sustained objection 2 — conservatism no
   longer has to be argued per operation.
6. **False-refusal controls.** The ESP, an ordinary ext4 partition and
   its device, and an LVM stack with nothing protected under it: all ten
   operations stay `Clear` under both sizes. **[measured]**

## The adversarial pass

The first four are mine; 5–8 are the workflow's, each reproduced by hand
before being accepted.

1. **"The first predicate repeats the defect it was written to end."**
   **Sustained, and it changed the act.** Version 1 bounded descent
   against the source's own extent with an extentless source *barring*
   descent. Measured on finding 4's fixture, it **lost reach HEAD has**:
   an extentless encryption layer could no longer reach a mapper carrying
   a device-framed extent — 6 → 3, refuse → construct.
2. **"U contradicts §2.1:110's stated closure."** **Sustained.** The spec
   says the set "closes over the substrate it destroys"; U closes over
   what the target carries as well. A fourth arm, not a re-reading.
3. **"D contradicts it too."** **Sustained, and the previous round missed
   it.** D's descent is bounded by destroyed *nodes' extents*, not by the
   destroyed *ranges* — that is exactly why (b) is fixed. Neither size is
   a code-to-text correction.
4. **"Move's over-reach now contradicts ADR-0018's relocation
   exemption."** **Sustained as a recording obligation.** 0018:141-145:
   "Relocation classes (move, copy-then-commit) **exempt the relocated
   target's own subtree from destruction descent**". ADR-0038 gave `Move`
   the whole target extent as destroyed, so the subtree is already
   captured by the byte scan; the act additionally descends into it.
   Either the exemption is stale or ADR-0038's `Move` entry is wrong.
   Named, not resolved.
5. **"A forged extent frame flips a live-ZFS refusal to `Clear` where
   HEAD refuses."** **FATAL, sustained, and it killed version 2.**
   Reproduced by hand: on the ADR-0037 shape, moving the ZFS label's
   `extent_host` (to the device, to the PV, to a node that does not
   exist) left the node's identity bit-identical — `BackingSignature`
   hashes its `host` *field*, not the extent's frame — and version 2
   answered `Clear` for all three, where HEAD answered `Unsupported`.
   The delivered predicate admits on every one of those bodies.
   **[measured, both sides]**
6. **"Clause (c) is dead for every volume source"** and **"on a
   multi-device aggregate the predicate defeats ADR-0018's flagship
   whole-device refusal."** **Both FATAL against version 2, both fixed.**
   Re-run against the attackers' own harnesses: the LUKS-erase shapes,
   the volume-framed and device-framed label shapes, the two-leg VG, the
   whole-device wipe of one leg, and the decoded-body case all refuse
   under HEAD **and** under the delivered predicate. **[measured]**
7. **"The act is requirement-shaped and the bump is major."**
   **Sustained** — finding 8, four normative sites, and the house's own
   precedent (`AGENT_BUILD_SPEC.md:51`: "**Major under §0.1**: PLAN-008's
   and Section 6's existing texts change meaning"). My earlier reading
   was minor; I withdraw it.
8. **"The predicate reads body content to decide reach."** **Sustained as
   the standing cost, and bounded.** It still does — but only ever to
   *admit* less reach than HEAD in the one case where both extents are
   declared, same-framed, and the child is positively outside the parent.
   The residual is real and measured: a label declared one byte before
   its parent's start escapes the (b) fix — **and escapes HEAD too**
   (both construct). **[measured]** It is a forgery hole in the byte
   scan, not one the act opens; finding 7's issue owns it.

## Rejected, recorded

- **Version 1 — bounding against the source's own extent** (extentless
  source bars). Rejected on measurement: loses reach HEAD has.
- **Version 2 — bounding against any destroyed node's non-self-framed
  extent** (extentless source admits only its own framed content).
  Rejected on the fatal in adversarial pass 5: it makes protection depend
  on an unauthenticated frame field, and ADR-0037's enforcement is held.
  It is the only variant measured that defeats Mutation B outright; that
  is the trade, and it was declined.
- **A hand-authored "frame kind" list.** Rejected before measurement: a
  source-keyed kind predicate is exactly what Mutation A switched off
  last round.
- **Keying the bound on the declared ranges' frames.** Rejected: makes
  the closure non-monotone, which is the objection this round retires.
- **Fixing (c) here.** **Deferred**, and it is the one place the previous
  round's rejected `seed` design was arguably right: its rejection rested
  on `gate(Wipe, table)` capturing the ESP, read as re-deriving
  0018:190-192 — but 0018:190-192 is about destroying `sda2`, not about
  wiping the table that describes both partitions. Capturing the ESP when
  the table itself is destroyed is correct reach, not sibling capture.
  Its own round.

## Decision carried forward — the recommendation

**Act: variant D plus the enumerated premise test, as a major spec change
(13.0.0). Put U to the decision owner in the same sitting, measured and
ready, but do not land it without an explicit answer.**

- (b) has a live under-refusal at the body boundary, it blocks #319's
  authorization half, and D fixes it with every committed test green —
  ADR-0038's pin included, so the held half stays *visibly* held.
- U is measured green apart from the pin it is designed to move. The
  question it asks is not "does it work" but "should `Grow`, `Repair`,
  `Label`, `Uuid`, `Create` and `Decrypt` over a target carrying
  protected content refuse". §2.1's own posture ("MUST still detect,
  correctly represent, and protect everything below; it MUST NOT mutate
  them") says yes; it is a product-behaviour decision, not a defect fix.

**Spec price: major, 13.0.0.** Four normative sentences change meaning
(finding 8). §0.1's rule is explicitly not a narrowing test, and its own
worked example is a mis-numbering recorded "so the rule is not read as
optional". The counter-argument — that reach is *added* and nothing
weakens, so it is minor like ADR-0037 — must be recorded and declined:
ADR-0037 added a rule in unspecified territory and changed no existing
sentence; this changes four. **Not a §1.11 filing**: that register needs
two conflicting requirements, named and quoted, and there is no such
pair. §2.1:110 against MODEL-002:378 is an unmet obligation, not a
conflict.

**PR sequencing.** (0) `Governance:` PR reserving `docs/adr/0039-*.md` in
WP-010's `owned-paths-reserved` — the block ends at 0038
(`WP-010.md:76`), the checker reads the assignment from the base
revision, and a governance change may edit only
`docs/work-packages/WP-*.md` (`AGENTS.md:30-32`); the a6341e6 → 1d03cd3
and e0bb990 → 4d030fe precedents. (1) **WP-010**, `spec-change`-labelled:
§2.1's enforcement paragraph, ADR-0018's rule 2 and Decision text, the
document-control version row, the §0.3 changelog row, ADR-0039,
CHANGELOG, the closure, the premise test, the regressions. (2) **WP-050**
is owed nothing — the CAP-005 enumeration does not move under either
size **[measured]**. (3) New issues, none blocking: **(c)**; the
`canonical_ranges` table-write correction (finding 9, ordered *after*
this); the extent-validation holes at the decode boundary (finding 7);
and the `assemble`/`from_canonical_body` asymmetry (finding 3).

**Evidence obligations.** A regression per finding 1, 4, 5, 7 and the
headline — the headline at `mutating_declared`, not at `affected_set`,
because that is the boundary with no gate in front of it. The forgery
shapes committed as tests, since they are what killed version 2. The
false-refusal controls committed. The premise property test, which is
also MODEL-002:378's outstanding obligation. Mutations applied with
`Edit` and proven applied. `cargo xtask ci`'s real exit code checked
directly after the last edit. Bare IDs on Requirements lines; the
traceability map regenerated last.

## Open, for the decision owner

1. **D or U** — does (a)-six close now or stay held? U is measured and
   ready; the question is a product one.
2. **Major (13.0.0) confirmed?** I recommend it and withdraw my earlier
   minor reading; ADR-0038's "never major" sentence does not govern here.
3. **Mutation B's bar item** — accepting "not silent" instead of
   "immune", given that immunity was measured to cost a forgery hole.
4. **(c), the relocation exemption, and the three validation holes** —
   separate issues, or recorded consequences in ADR-0039?
