# Handoff — WP-020 increment 2e review and merge, 2026-08-03

**From:** Codex, independent reviewer of the Proxmox acceptance and PR #119.
**To:** Claude, for continuation after the WP-020 increment 2e merge.
**Outcome:** PR **#119** merged as `23a5e9d`; repository issue **#94**
automatically closed.

> This is an **untracked local handoff artifact**. `docs/reviews/**` belongs to
> WP-000, not WP-020. Do not stage it into a WP-020 or WP-035 commit. If Nate
> wants it tracked, land it separately under WP-000 ownership.

---

## 1. What I reviewed

I read `AGENT_BUILD_SPEC.md` 5.0.0 in full, the repository instructions, and
Claude's `HANDOFF_2026-08-03_CLAUDE_TO_CODEX.md`. I then independently checked
the six load-bearing claims from that handoff rather than merging on green CI
alone.

### Stopping condition

The exact command:

```text
git diff --name-only c75b340 HEAD
```

listed only:

```text
CHANGELOG.md
README.md
docs/quality/test-tiers.md
docs/work-packages/WP-020.md
```

No Rust source, manifest, or lockfile changed after the acceptance-proven
commit. The compiled artifact cited by the record was therefore unchanged.

### Acceptance record and limitations

I reviewed `docs/work-packages/WP-020.md`'s complete “Increment 2e acceptance
record.” Its claim is appropriately narrower than “the loop path is safe.” In
particular, it states rather than hides that:

- the guest retained DHCP connectivity and a default route;
- the run establishes disposable VM/storage isolation and enumerated actor
  exclusions, not an air gap;
- digest and status checks are discrete samples and cannot defeat an ABA
  change entirely between samples;
- this is not a continuous-binding guarantee or evidence for a future
  destructive path;
- no destructive Tier-2 suite was registered.

I accepted the networked sitting. Neither SAFE-001 nor the increment-2e
acceptance requires network isolation specifically. The record does not infer
an air gap from VM isolation, and consequences remained bounded to the
disposable VM and generated backing files.

### Increment status

The WP-020 delivery table correctly says:

- increment 2e: **Delivered**;
- increment 2: **Unblocked, and still not delivered**.

No surface claims that a destructive harness now exists. Generic destructive
Tier 2 and every Tier 3 request still refuse.

### `snapd` deviation

The stock Ubuntu cloud image initially held four squashfs loop attachments.
The record explicitly calls purging `snapd` and `udisks2` a **deliberate
deviation from stock**, required to establish the no-other-loop-administrator
condition. It is not presented as routine hygiene. The inert residual
`snapd.mounts-pre.target` is also disclosed as active-but-not-found, with no
unit file or binary remaining.

### Transcript custody

I independently recomputed SHA-256 for the cited merged-tree transcript:

```text
C:\Users\nmcbr\PartMan-evidence\WP-020-increment-2e-2026-08-03\
  guest-merged\20-transcript.txt
```

Result:

```text
ffef5541d679d6736d7e87e2698c9f30f39f2e6fdd7f38d8098682cfef7ffca8
```

It exactly matched both `guest-merged/21-transcript.sha256` and the digest
recorded in `WP-020.md`. The transcript names commit `c75b340`, exit status 0,
two configured legs, two confirmed detachments, detected/discarded adversarial
rebind, confirmed partition teardown, catalogue-matching initial hashes, and
unchanged final hashes.

The teardown record independently states that VM 9420's config, every matching
LVM volume, and the snapshot volume were absent after destruction.

### Ownership boundary

No WP-035-owned source, package document, traceability document, or
`docs/quality/observability.md` changed. The README changes are limited to
WP-020's specifically granted tier-availability paragraphs and WP-020 status
row. The stale WP-035 status statements remain for a separately authorized
follow-up.

## 2. Privileged implementation review

I reviewed the implementation rather than only the evidence prose.

The relevant safety chain holds:

1. The xtask parser recognizes one exact closed selector.
2. Native Linux and effective UID 0 are checked before authorization.
3. The SAFE-007 interlock authorizes exactly two generated regular-file
   fixtures and returns held descriptors in a non-cloneable `Authorization`.
4. `run_authorized` accepts no path, raw descriptor, loop number, or generic
   ioctl from its caller.
5. `LOOP_CONFIGURE` receives the held verified backing descriptor atomically.
6. Verification binds kernel-reported backing device/inode, loop flags, zero
   offset and size limit, block size, loop number, and held loop-node identity.
7. The clean observation remains unpublished until post-probe verification,
   both confirmed detachments, the adversarial discard proof, and both final
   fixture hashes.
8. The `unsafe` exception is confined to `sys.rs`; wrappers pin exact Linux
   UAPI layouts and opcodes and expose no generic ioctl surface.

I found **no blocking code or contract defect**.

## 3. Verification run by Codex

- `cargo xtask ci` on Windows: passed.
- `cargo xtask verify-change-ownership --base origin/main`: passed; 17 WP-020
  paths plus the derived `Cargo.lock`.
- WSL Debian Linux-only loop crate: **42/42 tests passed**.
- Linux compile-fail doctest: passed.
- Linux Clippy with `-D warnings`: passed.
- GitHub current-head matrix: **12/12 checks passed** — Tier 1,
  cross-language parity, and supply-chain on Ubuntu/Windows/macOS; real prober;
  fuzz smoke; GitGuardian.

The PR description was rewritten before merge to replace the stale pre-run
state with the actual acceptance, exclusions, teardown, limitations, and
follow-ups. PR #119 was then marked ready and merged with a merge commit.

Post-merge verification:

```text
PR #119: MERGED at 2026-08-03T20:26:55Z
merge commit: 23a5e9dec049bdfdbf939b59db051d6d5c131864
issue #94: CLOSED at 2026-08-03T20:26:56Z
```

## 4. Codex Security plugin note

I attempted to start the optional Codex Security diff workflow because this PR
introduces the sole privileged Tier-2 loop boundary. Its mandatory preflight
could not run: Windows has the `py` launcher but no registered Python runtime.
The durable scan remains paused at preflight and produced no security verdict.

Do not cite that paused scan as evidence. The merge decision rests on the
manual source review, the repository's required tests, the real Proxmox
acceptance, and the 12-check GitHub matrix above. Installing Python is optional
future tooling work, not a newly discovered product blocker.

## 5. Next work, in dependency order

### A. Authorize and land the WP-035 status correction

Two README statements are stale now that #94 is closed:

- the M0.5 prose around line 169;
- the WP-035 roadmap row.

WP-020 could not legally change those surfaces. Obtain a WP-035 authorization
row, then update only WP-035-owned status text and any exact related
observability/status header. Do not fold new measurements into that bounded
correction unless the authorization explicitly owns them.

### B. Take the preregistered hardened non-WSL SI-35 measurement

`docs/quality/observability.md` requires:

- a **fresh** disposable non-WSL distro VM;
- #94's implementation to have landed and been reviewed — now satisfied;
- the unprivileged measurement shell and separately bounded privileged setup
  and comparison actors exactly as preregistered.

Do not reuse the increment-2e acceptance as SI-35 evidence. It establishes the
descriptor-bound mechanism, not the SI-35 client projection.

Operational lesson from the acceptance: run privileged repository acceptance
as root over direct SSH with no `sudo` and no injected environment variable
whose value collides with CLI text. The WP-035 redaction tripwire correctly
refuses such collisions before privileged work begins.

### C. Continue the operator sittings

Still needed:

1. SI-35 Windows completion rerun.
2. macOS increment-6 measurement matrix.
3. real-partitioned-Linux matrix after the applicable authorization and fresh
   disposable setup.
4. SI-33/SI-28 S4 only after a genuinely same-model second reader exists.

### D. Hardware caution

The two 128 GB SanDisk devices attached to the Proxmox host are not currently
byte-identical:

- one has a single 32 GB partition;
- the other has a hybrid-ISO/live-installer-style layout.

L9 requires two byte-identical media. Making them identical is destructive and
requires Nate's explicit decision about whether either device contains
anything worth preserving. Never infer authorization from their physical
presence.

## 6. Local state left for the successor

- `origin/main` contains merge commit `23a5e9d`.
- The local checkout remains on
  `work/wp020-2e-acceptance-evidence` at `b38ed4a`; it was not silently switched
  or deleted after merge.
- Claude's incoming handoff and this reciprocal handoff are both untracked.
- An untracked `.claude/` directory was treated as user-owned and left
  untouched.
- The durable evidence bundle remains at the path in §1.

