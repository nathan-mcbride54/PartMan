# Progress report — 2026-07-29, after the audit

> **Corrections, added after `PROJECT_AUDIT_FOLLOW_UP_2026-07-29.md`.** Two
> claims below were wrong, and both were things this report presented as
> settled:
>
> - **"The action scanner is a subset enforcer" that fails closed** — it was
>   not. An anchored mapping key (`&pin uses: …`) was silently skipped, so the
>   gate passed while a mutable tag went unread. Discovery is now
>   syntax-independent (a sweep for `owner/repo@ref` tokens), which is what
>   makes the fail-closed claim true.
> - **"Post-open object verification makes a raced symlink harmless"** — it does
>   not. Content identity proves fixture *shape*; it proves nothing about root
>   membership or disposability. A symlink swapped in between canonicalization
>   and `open` yields a handle on an out-of-root file that passes every
>   handle-based check. WP-020 precondition 1 is reopened.
>
> The "where I would look first" list below was useful but incomplete: it
> flagged the flow-mapping detector and the lexical licence check as suspect,
> and both turned out to be genuinely broken — while the pre-open race, the more
> serious defect, was not on it at all.
>
> **A third round then disproved the replacements too**
> (`PROJECT_AUDIT_SECOND_FOLLOW_UP_2026-07-29.md`): the sweep that replaced the
> subset scanner was defeated by a YAML escape, a container tag with no `@`, and
> a local action outside `.github/actions/`; and the no-follow open closed only
> the final path component, leaving the fixture-root directory swappable. Both
> claims are corrected at source, and the scanner is now a structural YAML
> parse.

Written for the next audit pass. It covers everything merged since
`PROJECT_AUDIT_2026-07-29.md` was written at `89aa5de`, states what each change
claims and how that claim was tested, and ends with what I would examine first
if I were you — including where this work itself is most likely to be wrong.

`AUDIT_RESPONSE_2026-07-29.md` holds the finding-by-finding disposition table;
this report does not repeat it.

## Merged since the audit

| PR | Scope |
| --- | --- |
| #38 | WP-030 increment 1a: harness policy moved out of the audited file |
| #40 | Foundation: lock boundary, action scanner, fuzz graph, `verify-licenses` |
| #41 | WP-020 increment 2a: authorization holds the verified object |

Issue #39 tracks generated traceability and machine-readable owned paths.
Issue #35 (scheduled runs, workflow splitting) predates the audit and is
unchanged. `main` is `61ef0a3`; every merge went through the 11 protected
checks.

## The shape of all three changes is the same

Your audit found one defect wearing four costumes: **evidence that could be
weakened by the thing it was evidence about.** The token file supplied the
thresholds it was judged against; the palette owned the roster that defined
its own coverage; the gate could repair the lockfile it claimed to enforce;
the scanner silently shrank when shown YAML it could not read. Every fix moves
the standard outside the audited artifact and makes silence impossible:
policy in code with the JSON required to agree, the roster derived from the
specification, `--locked` at the alias boundary, unrecognized `uses`
constructs as named violations.

#41 extends the same principle to the interlock ahead of any Tier-2 work:
authorization now holds open file handles, verified through `fstat` and
content reads on the handle, delivered to the consumer as the handle — so a
rebound name changes what the *name* means, never what the proof *holds*.

## What was verified, and how

Every reproduction in your audit was re-run before acting and re-run after
fixing. The before/after table is in `AUDIT_RESPONSE_2026-07-29.md`; the one
addition since is #41's:

- Windows (real hardware, not reasoning): with an authorization held, rename,
  delete, and second write-handles on the target all fail with sharing
  violations, and succeed again the moment the authorization drops.
- POSIX (Linux and macOS CI): rename the verified object away, plant an
  impostor at its path, write through the held handle — the write reaches the
  verified object, the impostor never sees a byte.
- Handle-purity is proven deterministically, not probabilistically: 
  `verify_object` takes no usable path, and its test deletes the path before
  verifying, so a by-path regression fails on a missing file instead of only
  during a race.

Three regressions were planted in #41 to prove its tests are load-bearing
(share mode deleted; `fstat` downgraded to `stat`; handle read downgraded to
`fs::read`). Each failed exactly one named test. The middle one is the
important story: **the first version of #41 repeated the defect it was written
to end** — its handle-`fstat` could be silently downgraded to a path-`stat`
with every test green, and only the planted regression found it. The fix was
structural (a function that cannot see a path), not a better comment.

## Claims I am explicitly NOT making

- **SAFE-007 still rests on two independent factors.** The token remains a
  pure function of source. Deciding an independent factor needs an entropy
  source stable `std` does not expose, so it is a dependency decision awaiting
  its own review, recorded open in WP-020's preconditions.
- **The Windows `nlink` guard still does not exist.** Narrowed — while an
  authorization is held, the share mode refuses writes through any name,
  hard links included — but between generation and authorization the count
  check is Unix-only.
- **No-follow open flags were not used** in #41. The by-name symlink refusal
  is raceable hygiene; the safety claim rests on post-open object
  verification. If you disagree that this is equivalent-or-stronger, the
  reasoning to attack is in `interlock.rs` at the `OpenOptions` block and in
  WP-020's precondition 1.
- **The action scanner is a subset enforcer, not a YAML parser.** Your
  first-choice correction (structural parsing) was deliberately not taken:
  it puts a YAML dependency inside the tool that gates dependencies. The
  subset refuses rather than skips, which satisfies the fail-closed rule; the
  reopen condition is recorded in the dependency policy.
- **Tier 2 still refuses, correctly.** No destructive suite exists. Nothing
  in this repository has ever written to a real block device.
- **M0 is still not met**, and WP-000 is now labelled in progress rather than
  Complete, per your Section 12 argument.

## Where I would look first, auditing this work

1. **The flow-mapping detector in `action_reference`** (`flow_style_uses`).
   It is a hand-rolled character walk over quoting states, which is exactly
   the kind of code your audit caught last time. It is deliberately
   over-refusing rather than under-reading, and prose without `{`/`,` in key
   position stays ignored — but an adversarial pass over its edge cases
   (nested quotes, `}` before `{`, tabs) would be well spent.
2. **`verify-licenses` matches exact lines.** `license = "MIT OR
   Apache-2.0"` as a trimmed whole line. A manifest declaring the licence via
   a TOML multi-line string, or `license-file =`, would be refused (fail
   closed, good) — but a *workspace* manifest that renames
   `[workspace.package]` would also pass members that inherit from nothing.
   cargo-deny still gates the real graph, so the exposure is the two
   out-of-graph manifests only; still, the check is lexical, not semantic.
3. **The share-mode constant is a raw `1`** (`FILE_SHARE_READ`) with the
   meaning documented in place. If Windows semantics of sharing-violation on
   rename ever differ by filesystem (ReFS, network shares), the
   replace-after-authorization test would catch it on CI's runners but not
   elsewhere. The fixtures tree is always local-temp, so I judged this
   acceptable; you may not.
4. **The fuzz-lock preflight runs `cargo metadata --locked`** on the stable
   toolchain against a manifest whose build needs nightly. Resolution does not
   need nightly, so this works — but if a future fuzz dependency gains a
   `rust-version` gate or resolver difference, the preflight could diverge
   from what `cargo +nightly fuzz` actually resolves. Watch for that seam.
5. **`Authorization::targets()` still exposes `&[VerifiedTarget]`** for
   reporting, and `VerifiedTarget::into_file` hands over the handle. Nothing
   yet *forces* a consumer to write through the handle rather than reopening
   the reported path — the consuming API makes the right thing easy, not the
   wrong thing impossible. The real Tier-2 consumer, when it exists, is where
   that contract must be enforced; its design should take the handle, not the
   path, and ideally never see the path at all.

## State of the milestone map, one line each

- WP-000: in progress; remediated; open on generated traceability, ownership
  enforcement (issue #39), and the documented runner-image deviation.
- WP-010: unchanged, correctly blocked at increment 3 on SI-31 and friends.
- WP-020: increments 1–1f and 2a delivered; preconditions 2 (independent
  token) and 3 (Windows nlink) still gate increment 2 proper.
- WP-030: increments 1 and 1a delivered; increment 2 (shell) not started, and
  its assignment must be expanded with exact Tauri/application paths first.
- M0: not met. Product: still not a partition manager, and the README still
  says so.
