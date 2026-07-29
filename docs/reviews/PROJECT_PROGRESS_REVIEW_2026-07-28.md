# Project progress review for the next agent

- Review date: 2026-07-28
- Repository baseline: `c7a4ba1` (`main`)
- Normative specification: `AGENT_BUILD_SPEC.md` 4.0.0
- Scope: current WP-000, WP-010, and WP-020 implementation and evidence;
  previous review/remediation history; open specification blockers; local gates
- Review mode: code and documentation review. No production code was changed.

## Executive conclusion

The project remains pre-product and says so honestly. There is no inventory,
planner, GUI, CLI, helper, or storage mutation path. That is appropriate: the
domain model is blocked on unresolved safety semantics, and Tier 2/3 still
refuse instead of reporting an empty success.

Progress since the previous review is material:

- the forged-manifest and false GPT-classification findings were remediated;
- every generated fixture is now bound to an executable claim about its bytes;
- real `libblkid`/`wipefs` probing is automated in Linux CI;
- the new prober gate immediately found a real util-linux version difference;
- all local Tier-1, cross-language, and supply-chain gates pass.

The project should not start WP-020 increment 2 or resume WP-010 increment 3
yet. The interlock still authorizes a pathname rather than the file object that
was verified, a malformed TypeScript boolean can be hashed as the opposite
logical value, and two new fail-open edges exist in fixture generation and
prober-output parsing. The work-package and issue-status documents have also
drifted enough that they no longer provide a reliable dependency gate.

## Findings

### High — `Authorization` still does not bind the verified file through destructive use

`Authorization` is a cloneable vector of paths
(`crates/fixtures/src/interlock.rs:54-63`). `authorize` verifies each path and
returns only its canonical pathname (`crates/fixtures/src/interlock.rs:200-205`,
`:208-299`). After the hash check, another process can replace the directory
entry before a future loop/VHD attachment or destructive operation opens it.
Cloning the proof also permits it to outlive whatever transient state made the
check true.

This was a High finding in the previous review and was deliberately deferred
until WP-020 increment 2. It remains open. The historical response records that
fact, but the current WP-020 delivery description again says the type answers
“did anyone check?” without carrying the file-lifetime qualification
(`docs/work-packages/WP-020.md:89-91`).

There is no destructive consumer today, so this is not a current host-write
exploit. It is a hard blocker on adding one.

Required before WP-020 increment 2:

1. Open with platform no-follow semantics and validate metadata and bytes
   through the opened object.
2. Keep an exclusive or replacement-preventing handle alive through attachment
   and destructive use.
3. Make the authorization non-cloneable and consume it in the operation it
   authorizes, unless a reviewed design proves reuse safe.
4. On Unix, operate through the descriptor or a descriptor-derived path and
   compare device/inode identity. On Windows, use sharing flags and stable file
   identity that prevent or detect replacement.
5. Add deterministic replace-after-authorization tests on both platform
   families.

Requirements: SAFE-001, SAFE-005, SAFE-007, Section 11.3.

### High — malformed TypeScript booleans are silently authenticated as another value

The TypeScript encoder validates runtime payload types for integers, byte
strings, and text, but the boolean arm uses JavaScript truthiness directly:

```text
case 'bool': {
  out.push(value.value ? 0xf5 : 0xf4)
}
```

(`packages/canonical/src/canonical.ts:155-157`)

A runtime-forged value `{ kind: "bool", value: "false" }` therefore encodes as
`f5`, canonical `true`, instead of being refused. This review reproduced it:

```text
node ... { kind: "bool", value: "false" } ...
f5
```

The test named “a value whose payload has the wrong runtime type is refused”
checks only text and bytes (`packages/canonical/src/canonical.test.ts:230-236`).
Array and map container types are also not checked; those usually fail with a
native exception rather than `CanonicalError`, while the boolean case silently
changes meaning.

This is on the MODEL-005 authorization boundary. TypeScript types do not protect
objects deserialized from JSON, RPC, plugins, or `unknown`, which is why this
module already performs runtime validation for other variants.

Required before any domain artifact uses this encoder:

- validate the runtime type and shape of every `Value` variant, including
  boolean, array, map, all map keys, and every map value;
- reject malformed values with `CanonicalError`, never native coercion;
- add a table-driven test with at least one forged payload per variant;
- assert that every accepted encoded value decodes to the same logical value.

Requirements: MODEL-005, SAFE-005, SEC-001.

### Medium — a foreign file named `MANIFEST` enables in-place deletion

`generate_from` decides that a directory is safe to prune using only:

```text
let existing = root.join(MANIFEST_FILE).is_file();
```

It then removes every unexpected regular file
(`crates/fixtures/src/catalogue.rs:345-367`). `is_file()` does not establish that
the manifest was generated by this project; it also follows a symlink. Any
directory containing an unrelated regular file or symlink named `MANIFEST` is
treated as owned and can lose its other regular files.

The regression test covers only a directory with no manifest at all
(`crates/fixtures/src/catalogue/tests.rs:307-320`). It therefore proves less than
its name and comment claim. The normal `cargo xtask fixtures` path is fixed to
`tests/generated`, which limits current exposure, but the library function
accepts an arbitrary root and its documented safety guard is not real.

Recommended resolution:

- do not infer directory ownership from a filename;
- generate into a newly created empty directory, verify the complete set, then
  publish it under a narrowly validated repository-owned root;
- if in-place pruning remains, require a parsed manifest that exactly matches
  the current directory and a non-following ownership marker, and add foreign
  manifest and manifest-symlink tests;
- perform all validation before any pruning or writing.

Requirements: SAFE-001, SAFE-005, Section 11.3.

### Medium — prober parsers discard evidence they do not understand

`parse_udev` silently drops every line without `=`, and duplicate keys overwrite
earlier values when collected into a map
(`crates/fixtures/src/prober.rs:375-382`). `parse_wipefs` uses `filter_map`, so an
unrecognized or malformed signature row disappears
(`crates/fixtures/src/prober.rs:385-401`). The process wrapper additionally
converts stdout with `String::from_utf8_lossy`
(`tools/xtask/src/main.rs:368-376`).

This contradicts the claim that the *full* signature set is compared in both
directions. A newly emitted row that the parser cannot read is not an
“unexpected signature”; it is no observation at all. On a fixture expected to
be blank, an entirely changed output shape can therefore parse as empty and
pass.

The current parser test proves only that known 2.41 output and empty output are
accepted (`crates/fixtures/src/prober/tests.rs:105-140`). It contains no
malformed, duplicate, or unknown-shape refusal.

Recommended resolution:

- return `Result` from both parsers;
- reject malformed rows, duplicate keys/rows, unexpected headers, and invalid
  UTF-8;
- let an explicitly empty tool output be the only path to an empty observation;
- add negative fixtures for every rejected shape.

Requirements: SAFE-005, FS-004, Section 11.7.

### Medium — blocker and safety-status documents no longer agree

The issue register says “five remain” and then names seven direct blockers,
plus SI-12 and the SI-29/SI-30 inputs
(`docs/spec-issues/README.md:45-55`). It lists SI-31 as blocking while the
WP-010 status table says SI-31 is settled
(`docs/work-packages/WP-010.md:63-74`). SI-34 and SI-35 are absent from that
work-package status table even though the register says both block increment 3.

WP-020 says it generates 13 fixtures, but its table lists 12 and still names the
withdrawn `gpt-corrupt-header-512` as `Indeterminate`
(`docs/work-packages/WP-020.md:32-58`). The current catalogue instead contains
the honest recoverable fixture and a separate conflicting-tables fixture
(`crates/fixtures/src/catalogue.rs:59-74`).

The same WP-020 overview says the token cannot be known without generating the
fixture set (`docs/work-packages/WP-020.md:68-72`), while the later correction
correctly says it is a pure function of public source. That is not cosmetic:
the first paragraph describes an independent factor the implementation does not
have.

Recommended resolution:

- replace hand-counted blocker summaries with one current status table;
- explicitly classify each issue as direct blocker, transitive blocker, input,
  mitigated-open, or resolved;
- update the fixture table from the catalogue and delete superseded claims;
- state at the top of WP-020 that the token is weak and that file-handle binding
  is a precondition of increment 2.

Requirements: Section 1, Section 11.7, Definition of Done item 10.

### Medium — raw-byte hash constructors remain open

Rust still exposes `hash_canonical_bytes(&[u8]) -> Hash`
(`crates/domain/src/canonical/mod.rs:131-139`), and TypeScript exposes the
equivalent function (`packages/canonical/src/canonical.ts:462-475`). Neither
validates that its input is canonical.

This was accepted as an open finding in the previous review and is correctly
scoped to “before WP-010 increment 3,” but it is not carried in the current
WP-010 work-package or traceability gap list. Once public plan types exist, this
API is a bypass around strict decode.

Recommended resolution: make raw hashing private, accept a `CanonicalBytes`
type constructible only by encoding or strict decoding, or validate and return a
failure result.

Requirements: MODEL-005, SAFE-005, SEC-001.

### Low — the action-pin gate does not enforce its trailing-comment rule

The policy error says a pinned action must retain its release tag in a trailing
comment (`tools/xtask/src/main.rs:638-641`). The scanner removes the comment
before checking and `is_pinned` validates only the SHA
(`tools/xtask/src/main.rs:646-683`). A full SHA with no release-tag comment
passes, contrary to `AGENTS.md`.

The current workflow entries do include comments, so current CI is compliant.
Add the comment to the parsed result and test its required form so the gate
enforces the policy it reports.

Requirements: SEC-010 and repository mechanics.

### Low — fixture test sandboxes collide across concurrent test runs

Catalogue and interlock tests use fixed paths such as
`partman-catalogue-{tag}` and `partman-interlock-{tag}` and delete them at setup
and drop (`crates/fixtures/src/catalogue/tests.rs:8-20`,
`crates/fixtures/src/interlock/tests.rs:11-45`). Two concurrently running test
binaries can erase each other’s directories. WP-020 already records this gap
(`docs/work-packages/WP-020.md:390-395`).

Use a collision-resistant temporary-directory facility and keep each directory
owned by one test process.

## Progress and requirement status

| Area | Status | Review basis |
| --- | --- | --- |
| WP-000 repository foundation | Mostly passes | Three-OS workflow exists; local Tier-1 and supply-chain gates pass. Generated traceability and mechanical owned-path enforcement remain documented gaps. The action comment rule is not actually enforced. |
| WP-010 increments 1/2/4 | Partial pass | Rust and TypeScript agree on all shared vectors; strict decoding and fuzz targets exist. The malformed-boolean and raw-byte-hash findings must close before domain artifacts use the codec. |
| WP-010 increment 3 / MODEL-001…004 | Blocked, no implementation | No Section 5 domain types, provenance model, schema migrations, or body/envelope artifacts exist. Stopping remains correct, but the blocker register must be reconciled. |
| WP-020 fixture generation | Partial pass | Thirteen deterministic fixtures, byte-level evidence, and automated Linux prober checks exist. The pruning ownership check fails review. |
| WP-020 SAFE-007 interlock | Not ready for a destructive consumer | Current checks safely gate nothing because Tier 2/3 do not exist. The path/file lifetime defect, weak source-derived token, and missing Windows link/file-identity design must be resolved in increment 2. |
| WP-020 Tier 2 and Tier 3 | Not started | Correctly fail closed; no fake success path exists. |
| WP-030 accessibility shell | Not started | M0 cannot exit without it. |
| M0 milestone | Not met | MODEL-003/domain schema work is blocked and WP-030 is absent. |
| M1–M5 | Not started | No read-only product or write path exists. |

## Verification performed

- `cargo xtask ci` — passed: formatting, Clippy with warnings denied, toolchain
  check, action-pin scan, and 131 Rust tests.
- `cargo xtask cross-language` — passed: npm audit, TypeScript typecheck, and 20
  TypeScript tests.
- `cargo xtask supply-chain` — passed: advisories, bans, licenses, and sources.
- Targeted malformed-boolean probe — reproduced `f5` (`true`) from a string
  payload `"false"`.
- `cargo xtask probe` — not run locally because this review environment is
  Windows; the Linux CI job and captured-output unit tests were inspected.
- `cargo xtask fuzz` — not run locally because the pinned nightly and
  `cargo-fuzz` are not installed; the required CI smoke job exists.
- No command enumerated, opened, or wrote a host disk, user disk, mounted
  volume, or block device.

The first `ci` invocation used an intentionally short timeout and was rerun to
completion. The first `cross-language` and `supply-chain` invocations hit
sandbox restrictions on user-level package caches and were rerun with the
required access. The final results above are the completed runs; none of those
initial interruptions was a repository test failure.

## Recommended next order

Keep each item in its own work-package remediation or spec-change pull request.

1. **WP-010 codec remediation:** reject every malformed TypeScript variant and
   close the raw-byte hash API before any domain type depends on it.
2. **WP-020 increment-1 remediation:** make generation non-destructive outside
   an explicitly owned root and make prober parsing fail closed.
3. **Reconcile the issue/work-package status records:** establish one exact list
   of direct and transitive blockers before assigning further WP-010 work.
4. **WP-030:** begin the dependency-ready design-token, dark shell, and
   accessibility harness work needed for M0.
5. **WP-020 increment 2 only after its safety design is reviewed:** handle-bound,
   non-reusable authorization; Windows and Unix file-identity semantics; a
   genuinely independent token/freshness factor; and replace-after-check tests.
6. Resume WP-010 increment 3 only after every hash-visible blocker has an
   accepted decision and executable evidence.

Do not start WP-040: it depends on WP-010, which is not complete.
