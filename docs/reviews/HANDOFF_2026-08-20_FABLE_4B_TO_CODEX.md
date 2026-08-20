# Handoff — 2026-08-20, the 4b sequence opened: the store delivered, one gate left

**From:** Claude (Fable 5), the session Nate directed with "Start 4b".
**To:** Codex, for review — Nate's ask is feedback on everything below,
so §5 and §6 name the surfaces where a hostile read would be most
valuable. Nothing in this document is a decision; where a decision is
cited it has an owner and a record, and where a question is open it is
labelled open.
**Follows:** the increment-4b opening round
(`LINUX_4B_OPENING_ROUND_2026-08-20.md`, all three decisions taken by
Nate 2026-08-20) and the morning's acts that discharged its first two
next-steps: the `Governance:` store grant (PR #568) and WP-L110's
consequential edit (PR #569).

> Committed session record. `docs/reviews/**` is in WP-000's
> `owned-paths` block (`docs/work-packages/WP-000.md`) and lands in its
> own `Work-Package: WP-000` commit, never bundled with code.

## 0. Repository state

`main` at `6773c33`, **spec 20.0.0** (unchanged this session — nothing
here moved a spec sentence). WP-020 re-pinned at **`b6f8ee8`** after
the r59 sitting; `git diff --name-only b6f8ee8 HEAD` must list Markdown
only, and does (the three doc PRs #571–#573). Working tree clean;
no branch, no open PR, no VM alive on the apparatus.

Open issues: GitHub **#370** (WP-010, byte-preserving relocation of a
protected structure); Gitea **#1003** (WP-L100 increment 3b, the
table-role route ADR-0036's second branch — its input, the
helper-authored table node, was delivered by WP-L110 increment 2; the
3b work stays WP-L100's). Evidence bundles live off-repo at
`%USERPROFILE%\PartMan-evidence\` (r54–r59 current), custodian Nate.

## 1. What this session did

| PR | Package | What |
| --- | --- | --- |
| [#570](https://github.com/nathan-mcbride54/PartMan/pull/570) | WP-070 | Increment 6: the protection-artifact store — `crates/artifact-store` + `schemas/artifact-store.md`, ADR-0030's four rules as a pure library. Eight tests, ten mutants killed by named tests before proposal |
| [#571](https://github.com/nathan-mcbride54/PartMan/pull/571) | WP-020 | Re-pin at `b6f8ee8` after the r59 sitting (VMID 9506, on the merge commit; 2e/2h/2j all exit 0; transcript `c9fbc3d6…` agreeing in guest, host, workstation; teardown 17:42:18Z, nothing remaining) |
| [#572](https://github.com/nathan-mcbride54/PartMan/pull/572) | WP-070 | The increment-6 delivery row reads the sitting record |
| [#573](https://github.com/nathan-mcbride54/PartMan/pull/573) | WP-L110 | 4b's standing-items list resolves: the store half discharged, the ceremony follow-up round alone remains, its own precondition stated |

Sequencing followed the opening round's §6 exactly: the WP-070 store
increment first, because it stands before 4b's `Protecting`; 4b's code
has not started and does not start until §4's gate clears.

## 2. The store increment, and the design choices worth attacking

`crates/artifact-store` (`partman-artifact-store`): dependencies are
exactly `partman-journal` and `sha2`. The journal is the metadata
authority — which plan an artifact insures, which PART-013 arm produced
it, which regions a raw capture covers are the journal's protection
records' facts — so the store holds bytes by hash and nothing else: no
index, no sidecar, no kind tag, no second reference vocabulary.

- **Identity (Rule 2).** SHA-256 over the artifact's exact bytes;
  `ObjectName` renders it as 64 lowercase hex, the one on-disk
  spelling. `from_hex` refuses length and character defects — uppercase
  included, deliberately, so one directory cannot hold two objects with
  one identity.
- **Verification.** `deposit` hashes, stores through the seam,
  **re-reads and recomputes** before any `ProtectionArtifactRef`
  exists; a reference in a caller's hands is proof of a verified
  deposit. Empty bytes refuse (`DepositRefused::Empty` — a zero-length
  artifact witnesses a failed capture, never a backup). Depositing
  identical bytes twice is one artifact. `fetch` re-verifies against
  the reference before returning bytes.
- **Retention (Rule 3, grant obligation 2's store half).**
  `retention_pass` classifies every held object from
  `DecodedJournal` alone: **exempt** while any referencing apply is
  exempt under `ApplyLedger::exempt` (ADR-0029's linkage closure; one
  live reference suffices — content addressing makes shared artifacts
  real); **terminated-closure** eligible for an explicit decision and
  nothing automatic; **orphan** (unreferenced) fail-closed, never
  reclaimed; **corrupt** never reclaimable whatever the liveness; a
  journal reference the store cannot fulfill is surfaced as
  `MissingReference`.
- **End of life (Rule 4's crate half).** `reclaim` recomputes the pass
  itself (the ADR-0029 obligation-10 shape — no caller-computed
  liveness is accepted) and deletes only behind a `DeleteDecision`;
  silence retains. The two consequence sentences are `pub const`s
  pinned in doc-code agreement with `schemas/artifact-store.md`.
- **Not discharged here, stated:** obligation 3's raw-read
  impossibility is per-platform (each helper's acceptance); obligation
  4's deciding *surface* is the surface package's; and **PART-013's
  discharge order — artifact durable and verified before the
  protection record that references it is journaled — is a documented
  contract on the depositing helper, not a type.**

That last clause is the one I most want challenged. The journal's own
precedent is the opposite: `WriteClearance` exists precisely so
storage-writing code demands *proof* of prior journal durability
instead of a comment. I chose documentation because a pure store crate
cannot see the journal append it must precede — but a stronger shape
may exist: e.g. `deposit` returning a `VerifiedDeposit` token that the
helper's protection-record construction site *consumes*, making
reference-follows-verification structural at 4b's integration point
even though neither crate can enforce the cross-crate ordering alone.
If Codex thinks that carries its weight, it is a 4b-integration design,
not a store rework — the store's return value already is the proof
object in all but name.

Other deliberate choices a reviewer may reasonably dislike, with the
reasoning to attack: the seam's `put` is **durable-on-return by
contract** (the journal's `DurabilitySeam` instead receives the
not-yet-durable suffix explicitly — the asymmetry is argued from the
store's content-addressed idempotence, but it is an asymmetry); a
**corrupt exempt artifact is a pass finding, not an error** — the live
apply's recovery asset is gone, which SAFE-005 cares about, and the
crate only *reports* it (the helper's 4b behaviour on that finding is
undesigned); `SeamRefused.reason` is a free `String` (redaction is the
seam implementor's obligation, on the trait contract — the journal made
the same choice for `DurabilityRefused`); and the consequence
sentences' wording generalizes REC-011's LUKS-specific text
("passphrase or key", "metadata corrupted or lost") because the store
class also holds table backups — check the generalization against
REC-011's normative sentence.

Evidence: eight tests (the NIST `abc` vector as the independent hasher
check; one authored journal holding live, terminated,
linkage-recovered and shared-artifact closures; the fail-closed arms;
reclaim's refusal arms including a ledger-refused double-terminal
journal; the SEC-006 exemplar sweep; the name spelling). Ten mutants,
each applied by edit, killed by a named test, reverted, with the
baseline swept for residue afterward — the list is in WP-070.md's
increment-6 row.

## 3. The r59 sitting, briefly

The r58 flow repeated: guest created before the merge, settled while CI
ran, provisioned on the merge commit `b6f8ee8`, never rebooted; all
three acceptances passed; digest agreed in guest, on host, on
workstation; teardown proven empty. Scripts are the host's `-r59` set
(`mk-r59.py` copied `-r58` forward). One process note is recorded in
WP-020's Commit row rather than fixed quietly: `cargo xtask
traceability --write` scans **git-tracked** files only, so run against
the brand-new untracked crate it silently wrote an unchanged map — the
same class the r57/r58 records name. Caught at the empty diff, re-run
after `git add`, before any commit or gate saw it.

## 4. Where 4b stands: one gate, with a precondition that is Nate's call

Everything from `AuthorizationGranted` onward is 4b's
(`docs/work-packages/WP-L110.md`, the 4b rows). The opening round
settled its three owed-within items; the store is delivered; **the one
remaining standing item is the ceremony follow-up round** — a single
bus-vs-binary route decision with two consumers: the interactive
tier's polkit mechanism (`pkcheck` vs D-Bus `CheckAuthorization`) and
EXE-001's logind inhibitor (`Inhibit` over the bus vs launched
`systemd-inhibit`).

That round's precondition is fixed by Nate's own R8 decision
(`LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md` §4a): **a client
`auth_admin` observed succeeding once — "a row before it is a round"**
— needing a terminal and an administrator password. DR22–DR24 measured
everything short of it. Two honest ways to take the row, pending
Nate's choice: an instrumented pty in a disposable guest (the
instrument owns the guest, so it knows `muser1`'s password; `ssh -t`
gives `pkttyagent` the tty DR24's script lacked; expect answers the
challenge), or Nate at a real terminal. Either way `systemd-inhibit`
presence belongs in the same sitting — the opening round §1.6 flags it
unmeasured, and the launched-binary option cannot be weighed honestly
without it.

## 5. Open 4b design questions Codex could usefully attack now

1. **How does 4b's destructive Tier-2 acceptance ever apply a plan?**
   Two independent blockers are already on record: (a) every plan
   reachable over this build's wire takes the interactive ceremony
   (the floor act is unreachable — `ValidateWire` has no sized-create
   spelling, pinned by an increment-3 test precisely so adding one
   must be a decision), and the only shipped `Ceremony` refuses; and
   (b) the helper leaves transport `Unrecognized` until ADR-0018's
   rows land, so the protection closure refuses every mutating
   validate on real hardware. 4b's writing half cannot be demonstrated
   end-to-end in a guest until at least one of: the ceremony mechanism
   lands (the follow-up round), a sized-create spelling is decided, or
   ADR-0018 transport rows land (do virtio-scsi guests even satisfy
   them?). This sequencing question is undecided and unowned; a
   worked-out proposal would be genuinely valuable.
2. **CONC-001's `flock` half, concretely.** The decision is
   journal-first arbitration + `flock(LOCK_EX | LOCK_NB)` on every
   bind-set member's read-write handle. Open sub-questions: lock
   acquisition order across a multi-device bind set (deadlock
   avoidance vs the NB-refusal answer); interaction with the r55
   finding that a guest kernel **auto-assembled** an imported mdraid
   member into a stale `md127` on attach (what does locking do when
   udev/mdadm already hold the device the instant it appears?); and
   whether the loop devices Tier-2 fixtures ride on honor the
   convention the same way whole disks do.
3. **The store's 4b integration.** The Linux store root (a
   `/var/lib/partman`-sibling, root `0700`) lands under WP-L110's
   grant — naming and creation order relative to the journal's
   directory, and the `VerifiedDeposit`-token question from §2.
4. **EXE-003 on the delivered stream.** Progress rides envelope v2's
   `event` channel at step granularity, byte counts only where a step
   measures bytes, no ETA surface — the never-backward property is to
   be pinned on the stream's existing sequence discipline; worth
   checking the delivered WP-040 sequence rules actually carry that
   pin's weight.

## 6. Review surfaces, ranked

If Codex reads only three things: (1) `crates/artifact-store/src/lib.rs`
against ADR-0030 and the §2 choices above; (2)
`crates/artifact-store/src/tests.rs` asking the mutation question the
project always asks — does each test *falsify* its claim, or merely
exercise the code; (3) the 4b rows of `docs/work-packages/WP-L110.md`
against §5.1's sequencing question. Beyond those:
`schemas/artifact-store.md` (is the layout contract complete enough
for an independent seam implementation?), the r59 record in
`docs/work-packages/WP-020.md` (does the re-pin sweep miss a stale
count? — the "sixteen times" in `docs/quality/test-tiers.md:104` and
the "twenty-six times" in WP-020's trip narrative are *known*
stale-by-convention, left by every re-pin since r27), and the opening
round's decisions 2–3 as recorded in WP-L110's 4b row (any daylight
between the round's text and the row's paraphrase is a defect).

## 7. Gates and traps, for whoever runs anything

- `cargo xtask ci`, `cargo xtask test --tier 1`,
  `cargo xtask verify-change-ownership --base origin/main` — check
  **real exit codes** (`${PIPESTATUS[0]}`; never trust `cmd | tail`).
- The ownership gate compares **committed** state — run it after
  `git commit`, and expect it to name a count of paths.
- `cargo xtask traceability --write` scans **git-tracked** files —
  `git add` new crates first, and regenerate the map **last**.
- One package per commit; the `Work-Package:` trailer and
  `Co-Authored-By:` in one paragraph; commit text via `git commit -F
  <file>` (PowerShell here-strings mangle literals).
- Never run gates from *inside* a nested worktree (`fuzz/Cargo.toml`
  is claimed by the outer workspace); never share
  `CARGO_TARGET_DIR` with a second checkout.
- GitHub merges need the pass **count** and
  `mergeStateStatus=CLEAN` asserted, not `--watch`'s exit alone; after
  merging one PR of a set, `gh pr update-branch` the siblings and
  merge in order.
- Any Rust in a product crate trips WP-020's stopping condition: name
  the sitting in the PR body **before** merge, take it on the merge
  commit, re-pin after (`mk-rNN.py` on the Proxmox host copies the
  script set forward; VMID 9507 is next).

## 8. Pointers

`AGENT_BUILD_SPEC.md` (20.0.0) · `docs/work-packages/WP-L110.md` (the
4b rows) · `docs/work-packages/WP-070.md` (the store grant + increment
6) · `docs/adr/0030-si23-protection-artifact.md` ·
`docs/adr/0056-the-linux-mutation-toolset.md` ·
`docs/reviews/LINUX_4B_OPENING_ROUND_2026-08-20.md` ·
`docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md` (§4a, the
follow-up's precondition) ·
`docs/reviews/LINUX_LAUNCHER_HOME_ROUND_2026-08-20.md` ·
`docs/quality/observability.md` (DR20–DR25) ·
`schemas/artifact-store.md` · `schemas/journal/records.md`.
