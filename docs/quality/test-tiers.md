# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 4.0.0.

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

Filesystem access is limited to reading `.github/workflows/` for the action-pin
check, `schemas/canonical-encoding-vectors.json` for the shared vectors,
`schemas/design-tokens.json` for the WP-030 accessibility harness, and temporary
directories the fixture and token tests create and remove themselves. No test
opens a block device at any tier.
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

