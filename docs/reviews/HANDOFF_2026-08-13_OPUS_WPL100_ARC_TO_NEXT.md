# Handoff — 2026-08-13, the WP-L100 arc (assignment + increments 1–2 + corrections)

**From:** Claude (Opus 5), the session Nate directed with "pickup where
the last agent left off", then steered to the read-only adapter front
and Linux as its first platform.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-12_FABLE_REGISTER_RESIDUE_TO_NEXT.md`. The
arc plans this session wrote before their first lines of code are
`WP-L100_ASSIGNMENT_PLAN_2026-08-12.md` and
`WP-L100_INCREMENT_2_PLAN_2026-08-13.md`.

> Untracked local artifact, docs/reviews convention: never stage into a
> commit; `verify-change-ownership` refuses it — and caught exactly that
> mistake once this session, via `git add -A docs`.

## 0. Repository state

`main` at the #317 merge (`b50dd19`), **spec 12.10.0 — unchanged by this
arc**: no normative text moved, every claim landed under an existing
requirement. Working tree clean apart from untracked docs/reviews. No
open PRs. One open issue, #318. Local branches pruned to `main` alone;
no worktrees.

**A WP-020 re-pin sitting is owed.** The stopping condition pins at
`77b0dd7`, and this arc landed three Rust merges after it (#314, #316,
#317) — the eighth, ninth and tenth trips from outside that package.
VMID 9436 is next per the PLAN-005 arc's handoff. No sitting has run.

## 1. What this session did — five merged PRs, one issue

| PR | What |
| --- | --- |
| #313 | Governance: create the WP-L100 assignment the Section 14 charter row names, carrying in ADR-0033 §Verification 2's presentation obligations |
| #314 | Increment 1 — the contract, its bounded seam, and its published INV-003 reach |
| #315 | WP-000: rejoin the README work-package table (a defect this session flagged; delivered by a parallel session) |
| #316 | Increment 2 — devices and their identity material |
| #317 | Two corrections to increment 2, recorded rather than edited away |
| #318 | Issue: the six unrecorded Linux observability rows, two of which block increment 3 |

`crates/adapter-linux` now holds a byte-returning read seam over two
closed interfaces, whole-device enumeration, MODEL-004 observations in
the domain's own vocabulary, and the reach declaration. 23 tests and one
compile-fail proof, **none platform-gated** — the crate is pure over the
seam, so the whole suite runs on all three CI legs.

## 2. Decisions worth review

Merging is not acceptance. Each of these is reviewable and reversible.

- **The assignment itself** (#313), made under a broad directive. Its
  register-gate list is *not* empty — SI-28 and SI-37 are recorded open
  with structural conservative answers.
- **Transport is `Unrecognized` for every Linux device**, recorded as
  the *discharge* of imported obligation 6 rather than a shortfall.
  ADR-0018's own fabric-versus-local discrimination rows are outstanding
  on every platform, no Linux row records a classifying value, and
  ADR-0018 already prices this availability cost under "Negative,
  accepted knowingly". A source-text guard holds that no positive class
  is constructible in the module.
- **`udev`-database values are `Method::Heuristic`** and therefore
  derive `inferred`, while sysfs attributes are `Direct` and derive
  `authoritative`. A cached third-party computation is not this client's
  observation. This changes derived confidence, so it is a decision, not
  a detail.
- **`removable` and `queue/physical_block_size` are read with no
  observability row behind them**, and nothing is derived from either.
  Declining would make an SI-28 floor input structurally `Unavailable`
  for every device — a silent gap traded for a recorded one.
- **Property keys carry their interface** (`interface:native-property`),
  electing nothing. The attribute layer's serial and the database's
  serial-shaped key are two properties, because they are two interfaces'
  different answers.
- **Increment 2 builds no `NodeId`, no `Facts`, no snapshot.** The
  assignment's phrase "SI-28's floor inputs are reported as facts" reads
  as an instruction to construct a `protection::Facts`; it is not, and
  doing so would import increment 3's whole naming layer.

**One decision directed but not yet recorded in the repository.** Nate
directed option (a) for the `size` unit — proceed by recording a
decision that accepts the 512-byte convention with its cost and a named
revisit condition, rather than holding increment 3. Increment 3 was
blocked before that text could land, so **the decision exists only in
this handoff and the session transcript.** Whoever resumes increment 3
must write it into the increment's record, not re-derive it.

## 3. What remains open

1. **Increment 3 is blocked, on a governance act rather than on code.**
   ADR-0019 requires that an identifier used in naming be "the byte
   string returned by the one named source the evidence contract
   designates for it, per platform" and that "choosing the single source
   is the normative act", landing "only with a spec change". **The Linux
   designation has never been made.** Without it,
   `NamingFields::PhysicalDevice` has no referent for canonicalized
   serial bytes and no node can be addressed.
2. **The designation is itself blocked on #318 items 1 and 2**, both
   readbacks of the archived 2026-08-04 transcript rather than new
   sittings. A recommendation round ran this session and **defeated** its
   own proposal (designate sysfs `device/serial` / `device/wwid`): the
   observability row bundles four attributes and credits "sysfs"
   generically, so the designation cannot rest on it. That round's
   findings are worth re-reading before proposing again — in particular
   that designating `device/wwid` would be born firing ADR-0019's own
   revisit condition, since no value of it has ever been observed on
   Linux.
3. **ADR-0019's verbatim rule and the delivered contract are
   incompatible.** The ADR requires the designated source's bytes
   "verbatim — no case folding, no prefix stripping, no re-encoding,
   non-UTF-8 bytes legal". Increment 1's `read_attribute` decodes to
   `String`, refuses non-UTF-8 as `NotText`, and strips one trailing
   newline. Naming inputs need a bytes-preserving path through the seam.
   This surfaced only when increment 3 tried to consume the contract.
4. **ADR-0019 has no naming rule for two outcomes the contract
   produces**: a measured-absent source (ADR-C4's `ObservedAbsent`, not
   `unavailable`) and a failed read. Whoever makes the designation must
   close those seams in the same act.
5. **Increments 4 and 5** are untouched, and increment 4's multipath arm
   remains evidence-gated on observability rows that do not exist.
6. **A finding outside this package, worth an independent look:**
   `planner::solve::free_extents` treats absent child extents as free
   space, so a device self-extent with no partition extents reports a
   fully partitioned disk as entirely free. That is delivered WP-060
   code and this session did not touch it.
7. **A governance question, raised early on purpose** (in #318):
   WP-035's `docs/quality/observability.md` share is an *enumerated*
   grant that does not visibly extend to a transport-discrimination
   protocol, so that row may have no grant-covered home today.

## 4. Corrections this session made to its own work

Three claims shipped stronger than their evidence. All three were caught
by an adversarial pass, verified by hand before acting, and recorded
against the increment that made them rather than edited away.

1. **Increment 1's `ID_PART_TABLE_TYPE` sentence** claimed the `udev`
   database carries that key. The token appears under the
   direct-signature-probe column — an interface measured *denied* to the
   unprivileged client — and those probes ran over regular files.
   Corrected in #316.
2. **The missing `DeviceIdentity` guard.** Increment 2's own text said
   the rule was "held by a test over the crate's public surface" and it
   shipped without one. Added and mutation-verified in #317.
3. **`fields.md`'s `device/serial` strength.** Given `real-hardware` on
   a bundled row that credits "sysfs" generically. Restated in #317.

**The pattern is worth carrying forward: all three were misattributions
of *which interface* established something.** That is now the first
thing to check when this repository's records are cited.

## 5. Operational notes

- **A spawned parallel session can share this checkout.** The #315
  session created and checked out its branch in `D:\PartMan` itself,
  moving this session off its branch mid-turn; the first symptom was
  `Cargo.toml` appearing to lose a workspace member. Commit and push
  before spawning, and take a worktree **outside** the repo afterwards.
- **The worktree pattern works**: `D:\PartMan-wpl100-inc2`, outside the
  repo, with its own target dir. `cargo xtask ci` was verified exit 0
  from inside it before any code was written there.
- **`gh pr checks --watch` labels read local `HEAD`, not the PR head.**
  One watcher reported "FINAL for <other session's commit>". Verify
  `gh pr view <n> --json headRefOid` against the pushed SHA before
  merging.
- **Mutation discipline held**: 7 mutants in increment 1, 7 in increment
  2 (two killed by two tests each), 1 for the correction guard. The most
  valuable was increment 2's fifth — increment 1's `shipped_sources()`
  was a fixed array both SAFE-002 scans iterate, so a new module would
  have been exempt from **both** guards while leaving both tests green.
  That roster is now pinned against the crate's own module declarations.
- The adversarial lenses repeatedly overturned the ground readers' own
  `[GROUNDED]` recommendations. Two examples worth the pattern: "port
  WP-035's whole-device admission rule" (a code precedent is not a
  measurement) and "keep WP-035's field roster" (`removable` has no row
  on any Linux host). Do not skip the attack pass.
