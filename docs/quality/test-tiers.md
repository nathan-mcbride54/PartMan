# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 3.1.0.

## Tier 1

Tier 1 is unprivileged and safe on every developer host. It currently contains:

- Task-runner tests: command parsing, tier fail-closed behavior, and the
  SEC-010 action-pin check (WP-000).
- Canonical encoding tests: golden vectors, strict-decode rejection cases, and
  the shared cross-language fixture (WP-010).

The only filesystem access is reading `.github/workflows/` for the action-pin
check and `schemas/canonical-encoding-vectors.json` for the shared vectors.
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
- the **token**, `PARTMAN_DISPOSABLE_TOKEN`, which must match the manifest
  `cargo xtask fixtures` writes, so it cannot be known without having generated
  that fixture set;
- the **verified target**, re-read and re-hashed against that manifest.

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

