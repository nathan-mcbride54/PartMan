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

Both tiers currently fail closed. WP-020 must implement the disposable-test
token, independently verified image or VM target, and explicit destructive
profile before either tier can run. A single environment variable is never
sufficient proof.

No command in this repository enumerates, opens, or writes a block device, at
any tier. The only filesystem reads are the two listed above, both of
repository-controlled files.

