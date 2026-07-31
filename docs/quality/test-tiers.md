# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 4.1.0.

## Tier 1

Tier 1 is unprivileged and safe on every developer host. It currently contains:

- Task-runner tests: command parsing, tier fail-closed behavior, and the
  SEC-010 action-pin check (WP-000).
- Canonical encoding tests: golden vectors, strict-decode rejection cases, and
  the shared cross-language fixture (WP-010).
- Fixture and interlock tests: deterministic image synthesis, partition-table
  state classification, signature layout, and the SAFE-007 refusal cases
  (WP-020).
- Design-token and accessibility tests: WCAG contrast, colour-vision
  simulation, the specification-derived role vocabulary, and the mutation table
  that proves each check can fail (WP-030).
- Desktop-shell tests: generated-token drift, typed colour-boundary policy,
  TypeScript and React lint/type checks, synthetic server-rendered UI semantics,
  browser-like theme/selection/keyboard/drawer interactions, locale-aware
  BigInt-backed IEC and exact-byte formatting, the Tauri CSP/capability
  boundary, a Vite production build, and a native Tauri release build without
  packaging (WP-030).

Filesystem access is all of repository-controlled text, and it has grown with
each gate: workflow and composite-action YAML plus any Dockerfile they build for
the action-pin check; Cargo and npm manifests, `cargo metadata` output and the
two licence texts for the licence check; the `owned-paths` blocks, every tracked
path from `git ls-files`, and the workspace membership `cargo metadata` reports
for the two ownership checks; both lockfiles; `.cargo/config.toml`;
`schemas/canonical-encoding-vectors.json` for the shared vectors;
`schemas/design-tokens.json` for the WP-030 accessibility harness; and temporary
directories the tests create and remove themselves. The separate
Node-dependent desktop entry point also reads the committed desktop lockfile,
application and shared-UI sources, Tauri configuration, capability manifest,
and generated token sources. Within the repository it writes only ignored build
state under `apps/desktop/node_modules/`, `apps/desktop/dist/`,
`apps/desktop/src-tauri/gen/`, `apps/desktop/tsconfig.tsbuildinfo`, and the
workspace `target/`. npm and Cargo may also populate their configured package
caches outside the repository; those are dependency caches, not device access.
Tier 1 launches `git`, `cargo`, `npm`, Node, Vite, and the Tauri CLI as
structured subprocesses.

*This paragraph previously said access was limited to `.github/workflows/`, two
schema files and temporary directories. That stopped being true as gates were
added, and a boundary description that lags the code is worse than none — it is
the sentence a reader would rely on to decide the tier is safe.*

**No test opens a block device at any tier.** That has been true throughout and
is the claim this section exists to make.
Later packages may add pure planner, validator, and regular-file fixture tests.

Run it with:

```text
cargo xtask test --tier 1
```

The MODEL-005 Rust/TypeScript parity proof is Tier 1 too, but needs a Node
toolchain, so it has its own entry point and its own CI job:

```text
cargo xtask cross-language
```

The WP-030 desktop proof follows the same Node-dependent pattern and remains a
step in the existing required Tier-1 matrix on Windows, macOS, and Linux:

```text
cargo xtask desktop
```

It runs `npm ci` from the committed lockfile and ends with
`tauri build --no-bundle` with Cargo's `--locked` forwarded. The native host
registers no commands or plugins, its main-window capability grants no
permissions, and the command neither requests elevation nor opens a device.

## Tier 2 and Tier 3

Both tiers still refuse, and will keep refusing until a destructive suite exists
to run.

WP-020 increment 1 supplies the SAFE-007 interlock itself. All three proofs are
implemented and enforced together:

- the **profile**, `--profile destructive`, taken from the command line and never
  from the environment, so it cannot be inherited from a parent shell;
- the **token**, `PARTMAN_DISPOSABLE_TOKEN`, which must match what
  `cargo xtask fixtures` records. **This factor is weak, and recorded as such.**
  This file used to say the token "cannot be known without having generated that
  fixture set", which was wrong: the token is a pure function of the source, so
  it is identical on every machine that builds the same commit, and it is
  printed where CI captures it. It proves only that the invocation presented the
  exact build-derived value — anyone holding the repository can compute it
  without running the generator, so it is accident friction rather than evidence
  of provenance, and not an independent factor. That is a recorded decision
  rather than an open
  question: [ADR-0007](../adr/0007-safe-007-third-factor.md) explains why making
  it random would have been worse, since the interlock would then have to learn
  the token from the very directory it is verifying;
- the **verified target**, re-read, re-hashed, and required to byte-equal an
  image the compiled fixture catalogue produces. This is where the interlock's
  strength actually rests. Since 2026-07-29 the verification runs through an
  **open file handle that the authorization then holds**: `fstat`, length, and
  every content byte are read from the handle, and that same handle is what a
  destructive consumer receives, so rebinding the path after authorization
  cannot redirect a write. On Windows the handle's share mode also refuses
  concurrent writes, deletion, and renames while the authorization lives. The
  authorization is non-cloneable and consumed once.

A single environment variable is never sufficient proof, and disposability is
computed from a target's own bytes rather than asserted by whoever asked. A block
device cannot pass, because its bytes will never equal a generated fixture, and a
target that is not a regular file is refused before its contents are read at all.

Running a destructive tier with all three proofs present *still* fails, reporting
that the interlock authorized its targets but no suite is registered. That is
deliberate: a green destructive tier is exactly the signal someone would trust
when deciding whether the interlock works, so it must never be produced by a run
of nothing (Section 12, Section 16).

No command in this repository enumerates, opens, or writes a block device, at
any tier. Filesystem access is limited to repository-controlled files and to the
generated fixture tree under `tests/generated/`, which `.gitignore` excludes.
