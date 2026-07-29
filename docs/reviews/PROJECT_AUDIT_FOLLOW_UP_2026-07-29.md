# Project audit follow-up — 2026-07-29

This is feedback for the next agent after reviewing the remediation work merged
in PRs #38, #40, and #41 and the handoff/progress report merged in PR #42. The
repository was reviewed at
`4c2f90baee2874bc15ef256d425046e0069ae9be` on `main`.

Read this after:

- `PROJECT_AUDIT_2026-07-29.md`
- `AUDIT_RESPONSE_2026-07-29.md`
- `PROGRESS_REPORT_2026-07-29_POST_AUDIT.md`

This document qualifies a few closure claims in those files. It does not
replace the parts of them that remain accurate.

No production code was changed during this follow-up. Each adversarial mutation
below was made separately, exercised, and restored. This file is the only
persistent project change from the review.

## Executive verdict

The remediation is thoughtful and materially better than the state described
by the first audit. In particular:

- WP-030's policy now lives outside the audited token file. The WCAG floors,
  colour-separation floor, required semantic roster, required themes, and
  required distinct pairs are pinned in Rust and mutation-tested.
- The root `cargo xtask` alias now applies `--locked` before building the gate.
- The fuzz dependency graph has a committed lockfile and is included in
  advisory, licence, source, Dependabot, and fuzz-job checks.
- action metadata under `.github/actions/` is included in action-pin discovery.
- WP-020 authorization is non-cloneable, consuming, and carries the same open
  `File` that was inspected. Post-open checks are handle-based rather than
  path-based.
- WP-000 is honestly marked in progress, and issue #39 tracks generated
  traceability plus machine-readable owned paths.

The current baseline is green: local `cargo xtask ci` passed 177 Rust tests,
`cargo xtask cross-language` passed 28 TypeScript tests, and
`cargo xtask supply-chain` passed both dependency graphs. GitHub Actions run
30495343283 passed all 11 jobs on current `main`.

However, three closure claims are not yet established:

1. The action-reference scanner still has a valid-YAML fail-open bypass.
2. WP-020 can bind authorization to an object outside the fixture root during
   the race between canonicalization and `open`.
3. `verify-licenses` does not semantically establish a root manifest licence.

Two smaller evidence gaps remain in fuzz-lock enforcement and token-set version
validation. The first two findings below should be treated as blockers before
depending on the affected mechanisms for supply-chain or destructive-I/O
safety.

## Findings

### High — action pinning still fails open on a mapping-key anchor

**Evidence**

- `tools/xtask/src/main.rs:1021-1055` recognizes a deliberately limited YAML
  subset.
- `uses_key_value` at `:1062-1072` accepts only `uses`, `"uses"`, or `'uses'`
  at the start of the mapping key.
- The anchor refusal at `:1090-1095` applies to the **value** after a recognized
  `uses:` key, not to a YAML node property attached to the key itself.
- The fallback at `:1136-1194` only looks after `{` or `,`, so it does not see a
  block-mapping key with an anchor property.

**Reproduction**

One pinned workflow line was replaced with:

```yaml
- &pin uses: actions/checkout@v7
```

`cargo xtask verify-actions` exited successfully and reported six action
references instead of the baseline seven. The unpinned reference was skipped,
not refused.

An anchor is a node property in YAML and can be attached to a scalar mapping
key; it does not change the key's value. GitHub Actions has also supported YAML
anchors since September 2025. See the
[YAML 1.2.2 specification](https://yaml.org/spec/1.2.2/), the YAML 1.1 mapping
example containing `&a2 baz : *a1` in
[section 4.3.5](https://yaml.org/spec/1.1/), and
[GitHub's Actions YAML-anchor announcement](https://github.blog/changelog/2025-09-18-actions-yaml-anchors-and-non-public-workflow-templates/).

This disproves the response's statements that anchors are named violations and
that every shape outside the accepted subset fails closed. YAML tag properties
on keys, such as `!!str uses:`, are the same structural class and deserve a
test even though this review did not need a second bypass to establish the
finding.

**Recommended correction**

Use a structural YAML parser and walk mapping nodes for scalar keys whose
decoded value is `uses`. Preserve source location so violations still name the
file and line. Explicitly decide whether aliases are forbidden or resolved,
then test that decision.

The concern about putting a YAML dependency inside the dependency gate does not
outweigh the correctness problem now demonstrated twice. The gate already has a
locked dependency graph governed by `cargo-deny` and `cargo-audit`; a small,
reviewed parser dependency is easier to bound than an expanding partial YAML
implementation. If structural parsing is deferred, at minimum refuse key-side
node properties (`&` and `!`) and retain the exact mutation above as a
regression test. That is only a containment patch, not the preferred design.

### High — WP-020's pre-open path race can authorize the wrong object

**Evidence**

`verify_target` currently performs three separate path operations:

1. `symlink_metadata(target)` at
   `crates/fixtures/src/interlock.rs:281-292`;
2. `target.canonicalize()` and a lexical root check at `:294-305`;
3. `OpenOptions::open(&resolved)`, which follows links, at `:307-328`.

The object returned by the third operation is verified through its handle, but
the code never proves that this opened object is the one canonicalized in step
2 or that it remains beneath the fixture root. The stored `resolved` path is
then used for the manifest-name and equality checks at `:334-355`; those checks
describe the earlier path resolution, not the final opened object.

The race is:

1. `root/blank-512.img` passes `symlink_metadata` and canonicalization.
2. Before `open`, another actor renames it and replaces that name with a
   symlink/reparse point to an external regular file.
3. `open(&resolved)` follows the replacement to the external file.
4. If that external file has the same length and bytes as the named fixture,
   the handle-based regular-file, link-count, length, and digest checks pass.

A user's ordinary file can contain the same bytes as a generated fixture.
Content identity proves the fixture shape; it does not prove disposability or
root membership. On Unix an external file with one link passes `nlink`; on
Windows there is no link-count check. The share mode protects the wrong object
after it has been opened—it does not repair the binding error.

There is no destructive consumer yet, so this is not a present live-device
write. It is nevertheless a direct gap in the precondition intended to make a
future destructive consumer safe. The statement at
`crates/fixtures/src/interlock.rs:281-285` and
`docs/work-packages/WP-020.md:449-454` that a raced symlink is harmless is
incorrect and should be amended.

The existing tests at `crates/fixtures/src/interlock/tests.rs:69-184` prove an
important but later property: once the correct object is open, rebinding or
deleting its path does not change what the handle denotes. They do not exercise
replacement between resolution and opening.

**Recommended correction**

Make resolution and opening a single no-follow, beneath-root operation:

- On Unix, hold an open root-directory handle and use a reviewed platform API
  for relative open with no-follow/beneath semantics (`openat2` with
  `RESOLVE_BENEATH` where available, or a carefully bounded `openat` walk with
  `O_NOFOLLOW` and directory handles).
- On Windows, open with reparse-point-safe semantics and verify the final path
  and stable object identity from the handle. Address the already-open
  Windows link-count question in the same platform review.
- Keep the root handle and verified child handle alive through authorization.
- Add a deterministic seam immediately before the final open. In the test,
  replace the approved entry with a symlink to a same-length, same-digest file
  outside the root and require refusal.

This may justify a small reviewed `rustix`, `libc`, or `windows-sys`
dependency in an explicitly platform-scoped safe wrapper. Avoiding a dependency
is not itself a safety property. SAFE-009 forbids unbounded `unsafe`; it does not
require reimplementing platform semantics with portable path calls.

Until this is fixed, do not mark WP-020 precondition 1 closed and do not enable
Tier 2.

### Medium — `verify-licenses` accepts a nested JSON licence

**Evidence**

For `package.json`, `tools/xtask/src/main.rs:815-818` accepts any trimmed line
starting with the expected `"license"` text. It does not parse JSON or require
the property at the document root.

**Reproduction**

The root `license` property was removed from
`packages/canonical/package.json`, and the same text was placed under:

```json
"metadata": {
  "license": "MIT OR Apache-2.0"
}
```

Node confirmed that the parsed document's root `license` was `undefined`.
`cargo xtask verify-licenses` still passed all nine checked artifacts.

This qualifies the response's claim that every manifest declaration is gated.
`cargo-deny` covers the Rust dependency graphs, but it does not make the
out-of-graph npm manifest declaration true.

**Recommended correction**

- Parse `package.json` as JSON and require a root-level string exactly equal to
  `MIT OR Apache-2.0`.
- Inspect Cargo package licences through
  `cargo metadata --locked --no-deps` for both the root workspace and fuzz
  manifest, or parse TOML structurally and validate the table context.
- Keep a discovery walk, but reconsider the blanket skip for any directory
  named `generated`; a future first-party package with that name would be
  invisible.
- Add the nested-property mutation above as a regression test.

### Medium — the supply-chain task repairs the fuzz lock before auditing it

**Evidence**

- `Task::SupplyChain` at `tools/xtask/src/main.rs:126-146` invokes
  `cargo deny` on the fuzz manifest before any locked metadata preflight.
- The fail-closed fuzz-lock preflight exists only inside `fuzz()` at
  `:596-627`.

**Reproduction**

The complete `arbitrary` package entry was removed from `fuzz/Cargo.lock`.
`cargo xtask supply-chain` passed both dependency graphs and silently restored
the committed lockfile byte-for-byte.

The separate CI fuzz job runs in a fresh checkout and would still fail its
preflight, so the full protected workflow remains mitigating evidence. The
named supply-chain command/job is not independently fail closed, however. A
local sequence of `supply-chain` followed by `fuzz` can also repair the lock
before the supposedly refusing preflight sees it.

**Recommended correction**

Extract the locked fuzz metadata check into a shared function and call it at
the start of both `SupplyChain` and `fuzz()`, before any command that may
resolve or update. Preserve this exact stale-lock mutation as a test. If
`cargo fuzz` exposes a compatible locked mode, pass it to the actual target
runs as well; the current preflight and run are separate processes.

### Low — `tokenSetVersion` is nonempty, not validated or supported

**Evidence**

- `crates/tokens/src/audit.rs:123-131` only checks that
  `token_set_version.trim()` is nonempty.
- `specVersion` is correctly compared to `REQUIRED_SPEC_VERSION` at
  `:133-143`; there is no equivalent supported token-set version.
- `docs/work-packages/WP-030.md:34` says parsing is “strict and versioned,” and
  `AUDIT_RESPONSE_2026-07-29.md:15` says “versions validated.”

**Reproduction**

Changing `schemas/design-tokens.json` from
`"tokenSetVersion": "1.0.0"` to `"not-a-version"` left both
`cargo xtask tokens` and `cargo test -p partman-tokens --locked` green.

**Recommended correction**

Define the supported token-set version outside the JSON, just as
`REQUIRED_SPEC_VERSION` is defined, and require exact agreement. If forward
compatibility is intended, parse semantic versions and explicitly enumerate
the supported range. Test malformed and well-formed-but-unsupported versions.
Until then, describe the field as present rather than validated.

### Low — `into_file` hands the consumer a file positioned at EOF

`verify_object` reads to the end at
`crates/fixtures/src/interlock.rs:420-429`, and `VerifiedTarget::into_file` at
`:86-92` returns that file without rewinding. The current proof test has to
seek to offset zero explicitly at
`crates/fixtures/src/interlock/tests.rs:119-123`.

A future destructive consumer that assumes a newly supplied file starts at
offset zero could append or issue operations from the wrong position. Rewind
before constructing `VerifiedTarget`, document the cursor contract, or expose
a consumer API that always takes an explicit offset. Prefer making the safe
default structural.

## Documentation corrections

Preserve the history, but ensure the next update states these qualifications:

- In `AUDIT_RESPONSE_2026-07-29.md`, change the action-scanner disposition from
  “fixed” to “improved, reopened by key-anchor bypass.”
- In `PROGRESS_REPORT_2026-07-29_POST_AUDIT.md`, the scanner is not yet a
  fail-closed subset enforcer, and post-open object verification is not
  equivalent to no-follow/beneath open.
- In `docs/work-packages/WP-020.md`, reopen precondition 1. Handle binding is
  delivered only **after** a safe open; the path-to-handle containment step is
  still open.
- In WP-030 evidence, say `specVersion` is validated but
  `tokenSetVersion` is currently only required to be nonempty.
- Do not round WP-000 back to complete while issue #39 remains open.

The response is otherwise faithful about the absence of a product shell, the
continued Tier-2/Tier-3 refusal, the two-independent-factor limitation,
Windows link-count coverage, generated traceability, owned-path enforcement,
and WP-010's safety-specification block.

## Recommended direction from here

Work in this order:

1. **Foundation containment patch:** replace the Action scanner with structural
   YAML parsing; make manifest licence validation semantic; share the fuzz-lock
   preflight between supply-chain and fuzz. These are bounded WP-000 evidence
   repairs and should remain separate from feature work.
2. **WP-020 path-to-handle binding:** design and test atomic no-follow,
   beneath-root opening. Review the Unix and Windows dependencies together with
   Windows link-count/object-identity needs. Keep Tier 2 unavailable.
3. **WP-020 remaining factors:** decide the independent random token and finish
   Windows other-name coverage. Only then build the disposable-target Tier-2
   harness.
4. **Issue #39:** generate traceability from machine-readable evidence and
   enforce declared path ownership. This is present-tense definition-of-done
   infrastructure, not M4–M5 polish.
5. **WP-030 shell:** start only after its exact Tauri/application owned paths
   exist. Generate or expose typed design-token access rather than copying a
   palette into the front end. Keep the audit caveat that front-end pairings,
   focus behavior, semantics, zoom, and reduced motion require rendered tests.
6. **WP-010:** remain blocked until the missing SI/Secure Boot requirements are
   supplied; do not infer them.

The strongest general lesson from both audit rounds is unchanged: a green gate
must prove that discovery, policy, and object identity cannot silently shrink.
For structured inputs, semantic parsers are usually the smaller long-term
attack surface. For destructive I/O, acquire the safe object first and let
paths become reporting data, never authority.

## Verification record

Baseline:

| Check | Result |
| --- | --- |
| `cargo xtask ci` | Passed; 177 Rust tests |
| `cargo xtask cross-language` | Passed; 28 TypeScript tests |
| `cargo xtask supply-chain` | Passed; root and fuzz graphs |
| GitHub Actions run 30495343283 | Passed; all 11 jobs |

Adversarial checks:

| Mutation | Observed result |
| --- | --- |
| `&pin uses: actions/checkout@v7` | `verify-actions` passed and counted 6 instead of 7 |
| Root JSON licence moved under `metadata` | `verify-licenses` passed with no root licence |
| `arbitrary` removed from `fuzz/Cargo.lock` | `supply-chain` passed and repaired the lock |
| `tokenSetVersion` set to `not-a-version` | token audit and token tests passed |

All mutations were restored. The pre-open WP-020 race is established directly
by the ordering and semantics of the three path operations; its required
regression test needs the proposed pre-open seam so the race can be scheduled
deterministically rather than sampled probabilistically.
