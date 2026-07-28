# Contributing

This repository implements `AGENT_BUILD_SPEC.md` 2.0.0. Safety requirements in
that document override all other instructions.

## Before making a change

1. Confirm that the work package's prerequisites are complete.
2. Create a dedicated branch for the work package.
3. Record the requirement IDs, owned paths, test tier, and assumptions.
4. Stop if the work would require an unverified assumption or could target a
   host or user disk.

## Local verification

```text
cargo xtask ci
```

This is unprivileged Tier 1 only. Tier 2 and Tier 3 are unavailable until
WP-020 supplies the required disposable-environment interlocks.

Supply-chain checks use separately installed, pinned tools:

```text
cargo install cargo-deny --version 0.19.4 --locked
cargo install cargo-audit --version 0.22.2 --locked
cargo xtask supply-chain
```

## Reporting a vulnerability

Do not use an issue or a pull request. Follow `SECURITY.md`.

## Pull requests

Use one pull request per work package or assigned subtask. Complete every field
in the pull-request template, including assumptions, tests, changed owned
paths, requirement IDs, and follow-up packages.

