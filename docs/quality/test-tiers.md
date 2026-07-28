# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 2.0.0.

## Tier 1

Tier 1 is unprivileged and safe on every developer host. During WP-000 it
contains repository and task-runner tests only: command parsing, tier
fail-closed behavior, and the SEC-010 action-pin check. The only filesystem
access is reading `.github/workflows/`. Later packages may add pure model,
planner, validator, and regular-file fixture tests.

Run it with:

```text
cargo xtask test --tier 1
```

## Tier 2 and Tier 3

Both tiers currently fail closed. WP-020 must implement the disposable-test
token, independently verified image or VM target, and explicit destructive
profile before either tier can run. A single environment variable is never
sufficient proof.

No WP-000 command enumerates, opens, or writes a block device.

