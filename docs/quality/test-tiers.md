# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 4.2.0.

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
- Slint feasibility evidence tests: exact 41-ID ADR registry parsing;
  duplicate/missing/unknown gate refusal; duplicate-key, unknown-field, and
  evidence-owned-verdict refusal; shared `pce/1` hashing; mechanical
  supply-chain rejection; missing-proof inconclusiveness; integer artifact
  ratios; and byte-fresh generated Markdown (WP-030). This tool contains no
  Slint runtime or candidate application.

Filesystem access is all of repository-controlled text, and it has grown with
each gate: workflow and composite-action YAML plus any Dockerfile they build for
the action-pin check; Cargo and npm manifests, `cargo metadata` output and the
two licence texts for the licence check; the `owned-paths` blocks, every tracked
path from `git ls-files`, and the workspace membership `cargo metadata` reports
for the two ownership checks; both lockfiles; `.cargo/config.toml`;
`schemas/canonical-encoding-vectors.json` for the shared vectors;
`schemas/design-tokens.json` for the WP-030 accessibility harness;
`docs/adr/0009-bounded-slint-desktop-feasibility.md`, the normalized
`docs/quality/slint-feasibility-data/evidence.json`, and the generated
`docs/quality/slint-feasibility.md`; and temporary directories the tests create
and remove themselves. Tier 1 also launches `git` and the compile-time-selected
`cargo` as structured subprocesses. The report reads fixed repository paths and
writes only its one Markdown target under explicit `--write`; ordinary CI is a
read-only freshness check.

*This paragraph previously said access was limited to `.github/workflows/`, two
schema files and temporary directories. That stopped being true as gates were
added, and a boundary description that lags the code is worse than none — it is
the sentence a reader would rely on to decide the tier is safe.*

**No code in this repository opens a block device with write intent, at any
tier, and no command launches an external tool against a block device** — an
open performed by a tool this repository launches counts as this repository's
open. This sentence changed in two directions on 2026-08-01: its subject
widened from tests to all code — verified by inspection, nothing in the
repository opens or enumerates a device today — and its predicate narrowed to
write intent, ahead of the read-only inspection package (WP-035, created by
the 4.2.0 spec change), whose inspector will read device state through
unprivileged interfaces. The narrowing lands before the first device-reading
commit rather than being discovered false after it, for the reason recorded in
the paragraph above: the boundary sentence is what a reader relies on to
decide a tier is safe, and one that lags the code is worse than none. The
write-intent boundary must be enforced by an open-flags assertion and a test
that proves the assertion can fail — an obligation recorded on WP-035, whose
first device-reading increment is not complete without it — and it holds until
a destructive tier lands inside SAFE-007's interlock, which will renarrow this
sentence again before its first device-writing commit.

**At Tier 1 the stronger claim does not expire: no Tier-1 test opens a block
device at all, read or write.** Regular files are all SAFE-001 permits there —
SI-35's filing records that limitation directly — and device reads, when they
arrive, are operator-run or Tier-2 work, never Tier-1 tests.

**SAFE-007's interlock provides zero coverage for the read path.** A read-only
inspector never calls `authorize`, so nothing about the interlock's strength —
the held handles, the share modes, the byte verification — protects a read.
Stated as a decision rather than left to be inferred (recorded in the
2026-08-01 review handoff and carried into WP-035's assignment): the interlock
gates writes to disposable targets, and the read path's safety must rest on
the write-intent boundary above, on SAFE-004's rules wherever external tools
are invoked, and on INV-006's no-repair/no-auto-mount discipline. The
loop-device binding question on the destructive path remains open and is
tracked as issue #94.
Later packages may add pure planner, validator, and regular-file fixture tests.

Run it with:

```text
cargo xtask test --tier 1
```

The evidence report has a fixed-path check and an explicit regeneration mode.
The first form runs inside `cargo xtask ci`:

```text
cargo xtask slint-report
cargo xtask slint-report --write
```

The MODEL-005 Rust/TypeScript parity proof is Tier 1 too, but needs a Node
toolchain, so it has its own entry point and its own CI job:

```text
cargo xtask cross-language
```

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

No command in this repository writes a block device or opens one with write
intent, at any tier. Today none enumerates or opens one at all; that stronger
sentence is retired deliberately ahead of the read-only inspection package
(WP-035) — see the Tier 1 section for the narrowing, its subprocess rule, and
its reason. Filesystem access remains limited to repository-controlled files
and to the generated fixture tree under `tests/generated/`, which `.gitignore`
excludes, until that package lands with the boundary statement its assignment
obliges it to carry.
