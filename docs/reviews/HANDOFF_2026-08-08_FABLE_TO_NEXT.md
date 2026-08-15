# Handoff — 2026-08-08, end of session

**From:** Claude (Fable), working with Nate through 2026-08-08.
**To:** whoever picks this up next.
**Pick up here:** §2. Every remaining thread needs a decision from Nate
before implementation work can proceed honestly; nothing is blocked on
measurement or custody any more.

> **Untracked local handoff artifact.** `docs/reviews/**` belongs to WP-000.
> Do not stage this into a WP-035 or WP-010 commit. Earlier handoffs sit
> untracked beside it for the same reason.

Repository state as this was written: `main` at PR #173 merged and PR #174
merging on green (WP-010 register currency; enable-merge was attempted, the
repo forbids auto-merge, and the session merged it manually on green — if it
is somehow still open, merge it, the content is done). Spec 6.1.0. No open
issues.

---

## 1. What this session did

**The 2026-08-05 handoff's pickup is discharged.** This session was the
independent reader the macOS second-reader obligation required — it produced
none of the three records and computed none of the recorded digests. All
three transcripts were retrieved through their evidence-store locators and
rehashed: sitting 2's pre (18 647 bytes) and post (5 320 bytes) transcripts
and the M10 transcript all match their recorded digests. The M10 capture's
full 172-entry SHA-256 inventory was rehashed too, every entry matching.

Two custody facts were carried into the record rather than erased, and the
next session should not let anyone flatten them:

- **Sitting 2's rehash carries the weaker property.** Its digests were
  computed 2026-08-05 from the retained capture, not at retention. The
  discharge paragraph in `docs/quality/observability.md` refuses the
  stronger reading in terms.
- **The M10 record had a digest and no byte length** — an omission of
  sitting 2's class, found during this readback and recorded the same way
  (23 516 bytes, a readback-time measurement, stated as such).

**Where it landed** (PR #173, WP-035): the observability status header, the
sitting 2 and M10 Artifacts paragraphs, README's M0.5 prose and WP-035 row,
and a CHANGELOG entry. PR #174 (WP-010) updated the register's three
currency sentences — SI-34's note, SI-39's Dependencies, and Part 6
precondition 1's hold, which is now lifted.

**The increment 7/8 record lag is repaired** (second commit of #173). The
README said the CLI "observes no real device yet" three days after
increment 8 wired the Linux adapter in. The opening prose, the chassis
bullet, and the WP-035 row now describe increments 7 and 8, and the
CHANGELOG has the entries those PRs never added, marked as recorded late.
The lesson is the standing one: **the record sweep is part of the
increment, not a later increment.** Increment 11 exists for the package-end
sweep; it is not a licence for interim staleness.

## 1b. Added later the same day

Four more artifacts exist from this session's second half:

- **Issue #175**: the increment 2e acceptance proof is stale by its own
  markdown-only stopping condition (`git diff --name-only c75b340 HEAD`
  reports 15 non-markdown paths, including `crates/ffi-linux-loop`).
  Re-take is operator work in the VM; nothing may rely on the acceptance's
  pass until then.
- `SI-39_RECOMMENDATION_ROUND_2026-08-08.md` — recommends option (c),
  scoped as qualifying only the derived sentence, adversarially reviewed,
  rejections recorded. Awaits Nate's decision; the register is untouched.
- `DECISION_BRIEFS_2026-08-08.md` — the ADR-0014/ADR-C4-guard fork, the
  increment 9 plist route (lean: bounded hand-written reader), the
  increment 10 route (restates the prior deferral recommendation).
- `WP-020_INCREMENT_2_AUDIT_AND_PLAN_2026-08-08.md` — all four carried
  preconditions verified closed with their residuals; proposed 2g/2h
  shape; #175's re-take sequenced at the front. Deliberately no increment 2
  code was written: the interlock proof it builds on is the thing #175
  says is stale.

## 2. Open threads, in the order the last handoff had them (updated)

1. ~~The readback~~ — **done** (§1). SI-34's observability element and
   SI-39's measurement half no longer wait on custody.
2. **ADR-0014 (SI-35's axis) is still blocked on Nate**, not on drafting.
   The fork: its central move breaks ADR-C4's regression guard ("a
   positively absent partition table and an unreadable one produce
   different body values"). Do not draft a third version before Nate
   decides whether ADR-C4's guard can be amended. Drafts were in the
   2026-08-05 session's scratchpad, not the repo;
   `docs/adr/0014-si35-table-state-axis.md` is reserved and deliberately
   empty.
3. **Increment 9 (macOS adapter) needs a route decision.** `diskutil`
   emits plists; `apps/cli` has an enforced empty dependency closure.
   Either hand-write a bounded plist reader (stays inside the guard,
   attracts a Section 11.4 fuzz obligation) or take a dependency and
   restate the guard (a governance change). Neither is this session's to
   pick.
4. **Increment 10 (Windows adapter) needs Nate's route decision first.**
   No route is simultaneously dependency-free, `unsafe`-free and
   Section-16-clean. The prior analysis recommended deferring Windows to
   WP-W100 and shipping the reach declaration with the existing typed
   refusal — that recommendation is on record but undecided.
5. **SI-39, then SI-11 and SI-27** — the genuine design rounds. Nothing
   routes around them, and nothing gates them any more except decision
   work itself.

## 3. Traps this session hit or confirmed

- **`gh pr checks --watch` races `update-branch`.** The watch returns green
  for the pre-update run while the post-update run is still pending, and
  a merge attempt then reports BLOCKED. Re-watch after every
  update-branch. Auto-merge is disabled on this repo.
- **Checking out another branch reverts your working-tree view.** Obvious,
  but when two PRs sweep overlapping stale text, grep the *branch*
  (`git grep <pattern> <branch> -- ':!docs/reviews'`), not the working
  tree, before concluding a mention is unfixed.
- The 2026-08-05 handoff's trap list remains live; nothing in it was
  falsified this session.

---

## Addendum, 2026-08-09 — most of §2 is history now

Written after the sessions that followed; the threads above are retained
as the record but superseded. Current state: `main` at spec **10.0.0**,
register at **six items gating increment 3, three direct** (SI-11,
SI-27, SI-28). Resolved since this handoff was written: SI-39 (7.0.0,
ADR-0015), SI-35 (8.0.0, ADR-0014's axis carried to its instrument —
table parser, fixture, fuzz target, checksum schema all landed), SI-34
(9.0.0, ADR-0016, verdict helper-authored in the body), SI-33 (10.0.0,
ADR-0017, refusal-only continuity witness). Increment 9 (macOS adapter)
shipped; increment 10 closed as deferred; the fuzz roster is four
targets. The round documents and their acceptances are the
`SI-*_ROUND_2026-08-09.md` files beside this one.

Still open, in the order I would take them: the SI-11 protection-closure
round (carries ADR-0016's named-evidence-contract hard input plus SI-29,
SI-30, SI-37), the SI-27 naming round, and — operator-side — the #175
2e-acceptance re-take that unlocks WP-020 increment 2g. The write-path
obligations recorded in the SI-33/SI-34/SI-35 resolution banners all
bind the first write-capable increment; do not let a green write path
merge without them.

**Second addendum, later on 2026-08-09.** The SI-11 round-four
document now exists: `SI-11_ROUND_2026-08-09.md` beside this file — a
recommendation, adversarially reviewed, deciding nothing. It answers
Part 6's eleven items, names the ADR-0016 evidence contract (byte
layer of own enumerating parsers plus a named state layer), carries
recommended answers for SI-29 (narrow, with a geometry line) and
SI-30 (deletion-by-containing-erase severed and routed via MAC-009),
and gives SI-37 its fail-closed home without resolving it. **Nate
accepted it the same day and the chain is landed**: PR #195 reserved
ADR-0018 (governance-first), PR #196 landed the ADR and spec-change
11.0.0 — SI-11, SI-29, SI-30 Resolved; SI-37 reclassified to Later,
off the gate; SI-27's row and entry carry the theorem-premise and
bind-set-semantics handover. **Two items now gate increment 3, both
direct: SI-27 and SI-28.** Two follow-up issues carry the
out-of-grant staleness the landing sweep found: #197 (WP-035's gated
list still cites SI-11 as holding the closure open — re-attribute per
the SI-12 precedent) and #198 (the register's Part 1 "Half the
approach is settled" prose still calls SI-34's placement open, left
behind by 9.0.0). Next, in order: the SI-27 naming round — its
requirements are in ADR-0018's handed-to-SI-27 consequence and
SI-27's register row — then the operator-side #175 re-take. The
evidence obligations ADR-0018 names (state-layer stability rows,
fabric-versus-local discrimination, NVMe shared-capability, the
consumed-versus-released discriminants) are operator sitting work;
the arms consuming them fail to Indeterminate until the rows exist,
so nothing waits on them — the unmeasured populations are
conservatively refused, not blocked from typing.

**Third addendum, later still on 2026-08-09.** The SI-27 round-four
document now exists: `SI-27_ROUND_2026-08-09.md` beside this file — a
recommendation, adversarially reviewed, deciding nothing. Its central
move is the collision group: equal derived addresses collapse into a
counted, flagged, indeterminate group that always encodes, aligned
with the ADR-0011/SAFE-005 ambiguity rules and the measured L9
silent-last-writer-wins pair. Its central decision for Nate is attack
1: the filing's "without silently merging" is met by non-silence and
preserved two-ness, because individually distinct addresses for
byte-identical simultaneous devices require an excluded input —
accept the group, or the round returns to the wall. Also inside: the
canonicalization-by-named-source rule riding ADR-0018's contract, the
file/byte-range node with the host-backing edge (closing CONC-001's
empty loop bind set and round three's untestability defect), the
table-view re-parenting that restores partition injectivity under
hybrid tables, the platform-membership edge typed without preempting
ADR-0011's deferred path-set encoding, and preconditions 2–4
discharged or carried. **Nate accepted it the same day and the chain is landed**: PR #199
reserved ADR-0019, PR #200 landed the ADR and spec-change 11.1.0
(minor, as assessed — additions only, no retext found). SI-27 is
Resolved; **the increment-3 gate holds one item: SI-28,
Mitigated-open, its floor in force.** Issue #197 was extended to cover
WP-035's now-stale SI-27/SI-34/SI-35 gate citations alongside SI-11's.
What remains before increment 3 can be written: SI-28's disposition —
whether its Mitigated-open state (floor decided and in force, SI-33's
witness landed as the refusal input, relaxation parked behind
ADR-0017's revisit condition) still holds the *type*, or only the
populations the floor refuses. That is a decision for Nate, not a
design round: the mechanism question was answered by Part 7 and
SI-33's resolution, and what's left is whether the register keeps
SI-28 on the gate or reclassifies it the way SI-37 was — open, floor
in force, off the type's critical path. Operator-side work
unchanged: the #175 2e-acceptance re-take, and the evidence rows
ADR-0018/0019 name (state-layer stability, fabric-versus-local, NVMe
shared-capability, consumed-versus-released discriminants, Windows
and macOS designators, backing-designator rows).

**Fourth addendum, 2026-08-10.** The gate emptied and increment 3
started the same night. Landed: PR #201/#202 (the SI-28
reclassification grant and pass — nothing gates increment 3; SI-28
Mitigated-open at Later, floor in force, priced schema-major cost in
its banner); PR #203 (increment 3a: `model::naming` — derived
positional `NodeId`s over a domain-separated canonical preimage,
collision-group absorption, twelve tests carrying the ADR-committed
regressions); PR #204 (increment 3b: `model::topology` — the five
edge kinds with ADR-0018's semantics classes, `Topology::build` as
the fail-closed construction boundary, the theorem premise enforced
by the endpoint-pair table and proved by exhaustive enumeration).
Two CI lessons cost a round-trip each and are worth keeping: the
traceability checker reads `// Requirements:` lines as bare
comma-separated IDs (parentheticals become invented requirements),
and `cargo xtask traceability --write` must run **after** every
evidence source it reads — including WP-010.md's own increment table
— or the drift check refuses; also check `cargo xtask ci`'s exit
code, never a grep over its output. **Next slice (3c), in the shape
the plan records**: the snapshot body/envelope with MODEL-003 schema
versioning and MODEL-004 provenance observations, then the typed
decode/validate/hash boundary WP-010's codec-remediation section
mandates before anything authorizes through a digest. The identity
record (SAFE-003 with the witness field), the verdict and effect
table (ADR-0018), and the plan/step constructors with their
compile-fail proofs follow behind it.

**Fifth addendum, 2026-08-10, end of the run.** Slices 3c, 3d, and 3e
are landed (PRs #205, #206, and the 3e PR merged on green after
this note's writing): the snapshot body/envelope with the typed
decode/validate boundary and its decode-recompute equality; SAFE-003's
identity record with strength derived-never-stored, ADR-C4's
three-distinct-body-values guard held in bytes, and ADR-0017's witness
semantics measured-arm by measured-arm; and ADR-0018's protection
layer as pure functions — verdicts with the inverted residual, the
enumerated arms, node-local inheritance, release-as-destruction, and
the affected-set fixpoint whose two destruction classes exist because
the first draft re-derived round two's sibling capture and **the
committed sibling regression caught it before anything merged**. The
traceability tool's real rule was found and recorded (it scans
git-tracked files only; `git add` before `--write`, verify a named
row, regenerate-then-final-gate). **Remaining in increment 3, in
order**: ~~body carriage of the protection facts~~ — **landed as 3f**
(the snapshot body carries extents, transports, and member counts;
fact edits move the hash; misplaced facts refuse typed; and the
full-stack regression decodes a body and refuses the pool from its
own authenticated facts); ~~the canonical-step capability computation~~ — **landed as 3g**
(CAP-002's fourteen operations with ADR-0018's class partition;
`protection_gate` runs the constructor's own closure over canonical
effect-table entries, CAP-005 agreement enumerated over every
target/operation pair; source classes never suppressed; refusals →
`unsupported`, indeterminacies → `blocked`; `Clear` claims nothing
WP-050 owns); ~~the acknowledgment vocabulary and the plan/step constructor~~ —
**landed together as 3h**: `PlanStep` with private fields and the one
closure-running constructor, the `compile_fail` doctest as ADR-0012's
construction-refusal proof (spec 4.4.0's commitment discharged,
compiler-verified on every run), the vocabulary closed at ADR-0018's
three with `Release` converting exactly the orphan arm and refused
nodes coverable by nothing. ~~The plan body and its boundary~~ — **landed as 3i** (PLAN-004's
risk model; the decided Section 6 skeleton with the snapshot hash as
bound at validation and PLAN-007's window in the body;
`from_canonical_body(bytes, &snapshot)` refusing the wrong-snapshot
presentation and re-running every step through the sole constructor,
so the hand-forged artifact — ADR-0012's second verification row —
refuses by recomputation). ~~The helper-authoring stamp point~~ — **landed as 3j**: the table
state rides the helper-produced snapshot as an authenticated
per-device fact (ADR-0014's stamp point as fact carriage), bound
identities enter the plan body, and a plan identity whose table state
disagrees with the stamp refuses as `AuthoredFieldMismatch` — the
client-authored value that never validates, held by test. The derived
verdict is committed through its body-carried inputs rather than
stored — the anti-assertion shape ADR-C4 set — noted in the 3j
CHANGELOG as the implementation reading of ADR-0016's substance;
**Nate should glance at that reading** (stored-and-checked would also
satisfy the ADR; derived-only is stronger and was taken as
implementer authority). **Remaining before increment 3 can be called
delivered**: the remaining Section 6 items as WP-050/060 vocabularies
arrive, the TypeScript parity vectors for the new schemas, and the
schemas/ documentation of the three body formats. The ten landed
slices (3a–3j) carry the whole decided architecture: naming, edges,
snapshots, identity, closure, facts, gate, constructor, plan,
authoring. **3j merged as PR #212; main is green at 81 domain tests
plus the compile-fail proof.** The increment's remaining items are
genuinely gated on other packages' vocabularies now, which makes this
the natural handoff boundary: nothing in WP-010 is startable without
either a WP-050/060 delivery, the TypeScript parity work, or the
schema documentation pass — each a clean, separately assignable
piece.

---

## Session close, 2026-08-10 — state at rest, read this instead of the addenda

The addenda above grew turn by turn; this block is the summary a
pickup should trust.

**Where everything is.** Spec **11.1.0**. Register: **no open item
gates increment 3** — eighteen resolved (through SI-11 by ADR-0018
and SI-27 by ADR-0019), SI-28 Mitigated-open at Later with its floor
in force, SI-37 open at Later with its dual-path matrix as relaxation
evidence, SI-36 withdrawn. `main` is green at PR #212: ten increment-3
slices (3a–3j) delivering naming, edges, snapshots, identity,
closure, facts, capability gate, the sole constructor with the
compile-fail proof, the plan boundary with the hand-forged refusal,
and the authoring set. 81 domain tests plus the doc-test proof; both
ADR-0012 verification obligations discharged.

**Decisions awaiting nothing; one reading awaiting a glance.** Every
register decision was Nate's, recorded in ADRs 0018/0019 and the
11.0.0/11.1.0 changelog rows. One implementation reading deserves
Nate's eyes when convenient (flagged in 3j's CHANGELOG entry): the
derived protection verdict is committed through its body-carried
inputs rather than stored beside them — stronger than the literal
ADR-0016 stamp, taken as implementer authority, additively
convertible if the stored-and-checked shape is preferred.

**Startable next, in no forced order:** (1) WP-050's capability
engine, consuming 3g's `ProtectionGate`; (2) the TypeScript parity
vectors for the three new body schemas (snapshot, plan, node-entry
formats) plus their `schemas/` documentation; (3) WP-060's planner
vocabularies, which unlock the deferred Section 6 items; (4) the
WP-035 gate re-attribution (#197) and the register Part 1 prose pass
(#198). **Operator-side:** the #175 2e-acceptance re-take, and the
evidence rows ADR-0018/0019 name (state-layer stability,
fabric-versus-local, NVMe shared-capability, consumed-versus-released
discriminants, Windows/macOS designators, backing-designator rows) —
sitting discipline, whenever the bench is free.

**Traps this session confirmed, beyond the standing lists:** the
traceability generator reads git-tracked files only (add before
regenerating, verify a named row, regenerate last); never trust a
grep-masked exit code; `gh pr checks --watch` races freshly-created
PRs (poll until checks register); and the closure's first draft
re-derived round two's sibling capture — the committed regressions
are load-bearing, run them before believing any closure change.
Operator-side threads unchanged: #175, the ADR-0018/0019 evidence
rows, issues #197/#198. Operator-side threads
unchanged: #175, the ADR-0018/0019 evidence rows, and issues
#197/#198.

---

## Session close, 2026-08-10 second run — supersedes the block above

**What landed.** Six PRs, all merged on green, main at #218:

- **#213** closed issue #197: WP-035's five gated-list entries
  re-attributed from resolved register items to ADRs 0014/0016/0018/0019
  in the SI-12 shape, prohibitions unchanged. The sweep found the same
  staleness class in **product bytes** — the chassis's in-band refusal
  references (`apps/cli/src/lib.rs`, `inspect.rs`, pinned by tests) and
  the README sentences describing them — deliberately not swept into the
  docs pass; **filed as issue #215**, a WP-035 chassis change under its
  own grant.
- **#214 + #216** closed issue #198, grant-then-pass: the register's
  Part 1 "Half the approach is settled" block is banner-marked dated
  history (paragraphs verbatim), the sweep of the rest of Part 1 found
  nothing else superseded, recorded in the pass CHANGELOG entry.
- **#217 + #218** delivered **increment 3k**, the last startable
  in-increment-3 item: reservation-first (four `schemas/domain/` paths),
  then the three body-format documents (topology-snapshot body, plan
  body, node-entry format — each records a delivered format and decides
  nothing), and `body-vectors.json` — the constructors' own pinned
  output (4 snapshots, 2 plans binding their snapshot's recorded digest,
  9 node entries required verbatim in their bodies), proven byte- and
  digest-identical in both languages (new Rust test ×4, TS tests ×7,
  riding the required cross-language job). Main is green at 429 live
  tests.

**Startable next, updated:** (1) WP-050's capability engine (consumes
3g's `ProtectionGate`); (2) WP-060's planner vocabularies (unlock the
deferred Section 6 items). ~~(3) issue #215~~ — **done later the same
day, PR #219**: the standing gated list's table-state entry is
`helper-authored (ADR-0014)` in the ADR-0011 entry's shape, the
inventory/topology refusals are `not-implemented` naming the decisions
plus the unconsumed WP-010 types, SI-28 the one live SI citation;
pinned literals, doc comments, and the README sentences moved with the
bytes; the MODEL-003 assessment is in the CHANGELOG entry (value-level
change on the documented-provisional envelope/0, no version to bump).
Increment 3's remaining items are all gated on WP-050/060 vocabularies
now. **Operator-side unchanged:** #175's 2e re-take, the ADR-0018/0019
evidence rows.

**Method notes this run:** the fixture generator lived in the
scratchpad and was discarded — the committed Rust test re-runs the same
constructions, so fixture-versus-constructor drift is structurally
impossible; new `schemas/domain/` files needed reservation before first
delivery (the #69 shape, done as #217); rustfmt on a new test file is
part of `ci`, run `cargo fmt` before calling the gate green.

**Sixth addendum, later on 2026-08-10 — WP-050 started.** PR #220
created the assignment (Section 14's charter transcribed, claimed
versus consumed kept distinct, milestone sequencing recorded honestly:
package start asserts no milestone exit); #221 corrected its
owned-paths block (the prose described the CHANGELOG/README/Cargo.toml
shares, the machine-readable block never granted them — the gate
refused increment 1's records exactly as designed, and the fix is the
separate governance PR it prescribes); #222 landed **increment 1**,
`crates/capability`: CAP-003's four statuses with `supported`
compile-fail-proven unreachable (no `QualificationEvidence`
constructor until increment 3's store; no apply path exists anywhere),
the closed MODEL-003-versioned reason enum re-enumerating the domain's
protection grounds through exhaustive `From` impls (domain growth
fails compilation there, making the version bump a reviewed decision),
`from_protection_gate` carrying 3g's couplings, and the evidence-built
reason panicking in every assertive constructor (CAP-007 at the type
layer). **Increment 1 surfaced a genuine §1.11 conflict**: FS-007's
"as explicit blocked reasons" versus CAP-003's `blocked` definition —
one immutable-limit case, two statuses, both texts normative. Filed as
**SI-40** (grant #223, filing #224): Part 2, Later, gating exactly
WP-050 increment 2's technology-limit composition; three readings
recorded as options, none decided — **the decision is Nate's**.
Increment 2's other arms (protection composition, ACC-009 tool
preconditions, floors) are decided text and startable now; increments
3 (the CAP-006 store) and 4 (consumer seams) follow. Main is green at
#224 with 435 live Rust tests plus the two compile-fail proofs.

**Seventh addendum, 2026-08-10, end of the run — SI-40 resolved,
increment 2 landed.** Nate decided reading (a). The chain: #225
reserved ADR-0020 (no spec-change share — the decision amends no
normative text, and the grant records that as the reason); #226 landed
the ADR and the register resolution (SI-40 Resolved, the banner and
the authoritative table's Resolved row each stating the absent spec
change is deliberate; the ADR's recorded safety property: `blocked`
keeps meaning remediable, so a permanent impossibility never invites
remediation of the unremediable); #227 landed **WP-050 increment 2**,
the engine core — `capability()` composing protection → technology
limits (`unsupported`/`TechnologyLimit`/`NoneExists` per ADR-0020) →
Section 9 floor → ACC-009 tools → `preview`, unknown targets a typed
error, CAP-005 agreement enumerated over all 84 operation/target
pairs against `PlanStep::mutating` with grounds matching. Two domain
semantics the fixture drafts got wrong, worth keeping: an extent-less
target's canonical destructive entry is empty and the gate clears at
capability time (the plan step's declared ranges refuse later), and a
consumerless non-goal signature is the **orphan** indeterminacy — the
refusal needs the consumer chain. **Startable next:** WP-050
increment 3 (the CAP-006 store: docs/capabilities/ format, the
evidence token's one constructor, the schema check in CI) and
increment 4 (consumer seams); WP-060's planner vocabularies.
Main is green at 440 live Rust tests plus the compile-fail proofs.

**Eighth addendum, 2026-08-10 — increment 3 landed (#228).** The
CAP-006 store exists structured and truthfully empty:
`docs/capabilities/` with its format document, an advertised set empty
with the vacuity named (advertising and qualifying are each reviewed
acts), an empty floors file with its reason stated, and the CI gate as
a Tier-1 store test in `crates/capability` (the `shared_vectors`
pattern, dev-dependency only) pinning the qualified-row count at zero.
One deliberate narrowing, recorded in the module doc, the CHANGELOG,
and the PR: the evidence token gained **no constructor at all** — the
assignment's "one constructor" sentence has both preconditions vacuous
(no row to qualify, no runtime consumer that could possess the store),
so the increment-1 `compile_fail` proof holds verbatim until a real
consumer and a qualifying row exist. **Remaining in WP-050:**
increment 4, the consumer seams — the public API documented for its
three consumer classes, integration-shaped tests over WP-020 fixture
topologies exercising every CAP-003 status and every reason at least
once (note: `supported` and `QualifiedByEvidence` are unreachable, so
the coverage test's honest shape asserts their unreachability rather
than exercising them), and the package record sweep. Main is green at
#228 with 443 live Rust tests plus the compile-fail proofs.

**Eleventh addendum, 2026-08-10, actual end of the run — WP-040
started (#235/#236).** The RPC protocol layer is chartered and its
message layer landed: the versioned envelope over `pce/1` (MODEL-005's
cross-language proof consumed — both RPC ends already encode and hash
identically), RPC-004's 1 MiB bound binding the wire before any
parsing, bodies re-proved canonical so envelopes cannot launder
refused bytes, RPC-002's handshake as a refuse-never-degrade total
function at exact version equality (a compatibility window is a
reviewed decision nobody has made), and RPC-003's strictness as one
validator for both ends. The assignment's two load-bearing calls:
**every OS transport is route-decision-gated** — the WP-035
increment-10 triangle three times over (Win32 security APIs,
SO_PEERCRED, XPC frameworks), so the protocol layer is
complete-but-endpoint-less until routes are decided — and **SI-18
keeps all authorization vocabulary out of the authentication
skeleton** (SAFE-002 and HLP-003 are written in contradiction;
identity claims only until the register decides). **Remaining in
WP-040**: increments 2–4 (streams/reattach vocabulary, redaction
boundary, the skeleton), then the three transport route decisions —
Nate's to convene, each needing dependency-policy and unsafe costs
priced. Main is green at #236 with 466 live Rust tests plus the
compile-fail proofs. **The day's whole arc**: #213–#236, twenty-four
PRs merged on green — the re-attribution sweep (#197/#198/#215
closed), WP-010 3k, SI-40 filed-and-resolved (ADR-0020), WP-050
delivered entire, WP-060 delivered entire, WP-040 chartered with its
message layer landed. Registers current, every gate either decided or
refused by name. **Startable next**: WP-040 increments 2–4; WP-080's
charter (CLI surfaces consuming engine + planner); WP-070's charter
(the helper — needs SI-18 and transport routes first); the
SI-15/16/17/18/19 rounds. Operator-side unchanged: #175, the
ADR-0018/0019 evidence rows.

**Tenth addendum, 2026-08-10, end of the run — WP-060 delivered
(#230–#234), read this block and the ninth as the day's close.** The
planner went assignment-to-delivered in five PRs: #230 the assignment
(both WP-050 lessons pre-applied — shared paths in the owned-paths
block from birth, the five register gates SI-15/16/17/19/24 named
with their conservative refusals); #231 increment 1 (the pure
chassis: no clock, byte-equal determinism, capability answers carried
verbatim, source-class requests not plan material, Reversible
withheld while PLAN-008 waits on SI-19); #232 increment 2 (the step
graph: explicit dependencies, cycles with members named, and the
committed conflict rule — dependency-unordered overlap refuses,
ordered overlap is a chain); #233 increment 3 (the extent solver:
free space from authenticated extents alone, 1 MiB first-fit,
deviation inexpressible until its vocabularies arrive, SI-15's
misaligned-growth case refusing with the gate string carried); #234
increment 4 (the simulated final topology: `Planned { plan,
simulated }` everywhere because PLAN-002 makes simulation mandatory —
unrepresentable effects produce no valid plan, wipes empty the
container and drop the stamp, creates mint under a single table view
or refuse, and a plan can never revalidate against a prediction,
held by test; the rule enforced itself when the old chain test's
unsized create became honestly unplannable). **Main is green at #234
with 462 live Rust tests plus the compile-fail proofs.** Startable
next: WP-040 (RPC) or WP-080 (CLI surfaces consuming the engine and
planner) as fresh charters; the SI-15/16/17/19 rounds whenever Nate
convenes them (each unlocks its named planner increment); SI-24
before dry-run exists. Operator-side unchanged: #175, the
ADR-0018/0019 evidence rows.

**Ninth addendum, 2026-08-10 — WP-050 delivered (#229), read this as
the package's close.** Increment 4 landed the consumer seams (CLI /
planner / adapter classes documented, none with authority over
answers), the coverage test, and **the multipath arm the coverage
requirement caught missing before the test was written**:
`MultipathDetectionOnly` had no producing arm, so a multipath mutation
refused with protection's `RemoteTransport` ground where LIN-006
requires the multipath reason. The arm precedes protection
deliberately — LIN-006 names the reason that population reports, the
closure refuses the same population anyway, so precedence moves
reporting and never permission (held by test; `Detect` stays
`preview`, detection-only means detection works). All four increments
are delivered: the vocabulary with `supported` compile-fail-proven
unreachable, the engine core with CAP-005 agreement enumerated over
every pair, the CAP-006 store truthfully empty with the qualified-row
count pinned at zero, and the seams. **WP-050's remaining obligations
are consumer-driven, named in `docs/capabilities/format.md`**: a
first qualification row (a reviewed act that moves the pin), the
evidence loader for the first consumer that can possess a store, and
per-tool floors as tools join the roster. **Startable next, for
whoever picks up:** WP-060's planner vocabularies (unlock WP-010's
deferred Section 6 items and consume this engine per the seams doc);
the WP-040 RPC layer; operator-side unchanged (#175, the
ADR-0018/0019 evidence rows). Main is green at #229 with 445 live
Rust tests plus the compile-fail proofs; spec 11.1.0; the register
holds SI-40 Resolved by ADR-0020, SI-28 and SI-37 open at Later,
eighteen-plus-SI-40 resolved.
