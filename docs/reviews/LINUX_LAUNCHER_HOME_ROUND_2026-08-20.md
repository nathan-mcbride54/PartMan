# The launcher-home round — route (c), with WP-035 in the room

**Date:** 2026-08-20. **Base:** main at the ADR-0056 arc (spec 20.0.0).
**Directive:** Nate — answer the open decisions so increment 4b can
proceed; this is the last route gate.
**Question:** WP-L110's route (c), owed before increment 4b: *"the
launcher's home (WP-035's `SystemLauncher` is in `apps/cli`; a helper
cannot depend on an app), before increment 4b, with WP-035 in the
room."* Concretely: **where does the one SAFE-004 launcher live so that
both its consumers — the unprivileged CLI and the privileged helper —
can reach it**, and what must change in its contract in the same move.

> Committed session record. `docs/reviews/**` is in WP-000's
> `owned-paths` block and lands in its own `Work-Package: WP-000`
> commit, never bundled with code. Nothing below is decided; §4 is for
> the decision owner.
>
> **Decided 2026-08-20 (Nate): option A, with the sequence confirmed —
> `crates/launcher` under WP-035, `launch` gaining a caller-stated
> deadline in the same move; the `Governance:` act first, then WP-035's
> move (owing its own WP-020 sitting), then 4b's first code.** Both of
> increment 4b's route gates are now taken.

## 0. The texts

- **SAFE-004**: external tools through structured argv, a fixed
  executable allow-list, verified identity/version, bounded output,
  timeout, sanitized environment, trusted absolute locations.
- **ADR-0056** (route b, taken today): file-system operations are native
  tools **through the SAFE-004 launcher**; the version-verification
  discipline is the package record and/or a content digest, the
  launched query never the sole source. The helper therefore *will*
  launch tools in 4b — route (c) is no longer hypothetical.
- **The ceremony round**'s verified findings
  (`docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md:82-98`): the
  only implementation, `ToolLauncher`/`SystemLauncher`, lives in
  `apps/cli/src/doctor.rs` — **a helper cannot depend on an app** — and
  `LAUNCH_TIME_LIMIT` is a **private 5-second constant** while only the
  output limit is caller-stated; *"the launcher must therefore change,
  in whichever package ends up owning it."* If the ceremony follow-up
  later chooses R1 (`pkcheck`), it launches through this same seam.
- **The shape round** (`WP-L110_INCREMENT_4_ROUND_2026-08-20.md` §5):
  route (c)'s substrate is complete; the deadline must become
  caller-stated **in the same move**; the round needs a `Governance:`
  PR before any code, reserving both the new path *and* WP-035's share
  of the workspace manifest (its recorded share is *"the `members`
  entry for `apps/cli` only"*).
- **WP-035's grant and record**: `apps/cli/**` is its reserved path; the
  launcher is its delivered, tested SAFE-004 implementation (*"the one
  SAFE-004 implementation is its today"* — WP-L110's obligations
  block); its macOS increment 9 runs "through the existing SAFE-004
  launcher seam".

## 1. What is measured, off the tree

1. **Every consumer of the trait is CLI-internal today**: `doctor.rs`
   (`examine`), `devices.rs`, `inspect.rs` (`enumeration_json`/`_human`),
   `lib.rs` (`dispatch_with`, the injected-launcher test seam).
   `SystemLauncher` (`doctor.rs:186`) is the sole real implementation:
   spawn from an absolute path, structured argv, per-stream drain
   threads under `OUTPUT_LIMIT_PER_STREAM`, a deadline loop that kills
   at `LAUNCH_TIME_LIMIT`, no shell anywhere.
2. **The contract as delivered**: `launch(&self, path: &Path,
   arguments: &[&str], output_limit: usize) -> ProbeOutcome`
   (`doctor.rs:176`) — the output bound is caller-stated; the time
   bound is not.
3. **The allow-list is the caller's, not the mechanism's**: the CLI's
   `ROSTER` (`doctor.rs:78`, "the roster *is* the allow-list") is CLI
   policy — read-only probing of `blkid`/`wipefs`. The helper's 4b
   allow-list (mkfs-class tools, per ADR-0056) is a different policy
   with different entries and floors. The mechanism and the policy are
   separable as delivered, and SAFE-004's "fixed executable allow-list"
   binds each caller's roster, not the spawn code.
4. **The workspace shape**: `Cargo.toml` is WP-000's file with
   enumerated per-package `members` shares; a new crate needs a
   `Governance:` act widening the owning package's share before any
   code (the standing two-PR rule, ownership read from the base).

## 2. The options

**A. A new workspace crate — the mechanism moves, each caller keeps its
policy; WP-035 owns it.** `crates/launcher` (`partman-launcher`):
`ToolLauncher`, `SystemLauncher`, the bounded spawn/drain/kill core,
`ProbeOutcome` — with `launch` gaining a **caller-stated deadline**
beside the caller-stated output limit in the same move. `apps/cli`
depends on the crate and keeps its `ROSTER`, its `ToolSpec`/`examine`
reporting, and its injected-launcher tests; the helper (4b) depends on
the crate and carries its own fixed allow-list and ADR-0056's
version-verification discipline. WP-035 owns the new crate: it authored
and tests the one SAFE-004 implementation, its macOS increment 9 uses
the same seam, and ownership follows the code that moves. *Costs:* a
`Governance:` PR first (the path and WP-035's manifest share); the move
is Rust, so it owes a WP-020 sitting; two packages' docs gain a
sentence each.

**B. The same crate, owned by WP-L110.** *Against:* the mover would own
code whose only current consumers are WP-035's, inverting the review
relationship; WP-035's macOS increment would depend on a helper
package's crate for its own seam; and the launcher's history — its
tests, its bounded-drain fixes — is WP-035's. Nothing is gained over A.

**C. A helper-private launcher in `services/helper-linux`.** *Against:*
two SAFE-004 implementations to review and keep from drifting, where
the ceremony round's own text says the tool goes "through the **one**
reviewed launcher"; and the duplicate would start life needing the same
deadline fix A makes once.

**D. Leave it in `apps/cli`; the helper depends on the app.** *Against:*
the layering the route names — a privileged helper depending on an
unprivileged app's crate drags the CLI's dependency tree into the
helper's supply chain. Rejected on the route's own sentence.

## 3. What the move must not change

The mechanism's guarantees travel verbatim: absolute paths only,
structured argv, no shell, bounded output per stream, kill at the
deadline, sanitized environment. The CLI's observable behaviour is
unchanged (its call sites pass the same 5-second value the constant
holds today — the doctor's probes *should* die at five seconds). The
helper's 4b call sites state deadlines fit for their operations, which
is the point. Nothing about rosters, floors, or verification moves into
the crate: mechanism in the crate, policy with each caller.

## 4. The decisions for the owner

1. **The home: option A** — `crates/launcher`, owned by WP-035, the
   deadline made caller-stated in the same move?
2. **The sequence** (the standing rules applied): (i) `Governance:` PR
   reserving `crates/launcher/**` for WP-035 and widening its
   `Cargo.toml` share to the two members entries; (ii) WP-035's move
   act — Rust, owing its WP-020 sitting; (iii) only then 4b's first
   code. Confirm?

## 5. Next acts, in order

1. This round (WP-000, `docs/reviews/`). Decision.
2. The `Governance:` PR (path + manifest share).
3. WP-035's move act with the caller-stated deadline; its sitting.
4. Increment 4b opens — both route gates taken, the remaining owed-first
   items being 4b's own (`Governance:` for PART-013's store, the
   CONC-001 mechanism decision, the EXE-001/EXE-003 decision).
