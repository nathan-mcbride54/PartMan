# PartMan

PartMan is a safety-first, cross-platform disk partition manager defined by
`AGENT_BUILD_SPEC.md` 2.0.0. The intended product is a dark-first Tauri desktop
application plus a scriptable CLI, backed by a shared Rust domain, planner,
validator, journal, image engine, and per-platform privileged helpers.

## Current status

WP-000 foundation work is in progress. The repository currently contains only
unprivileged development infrastructure. It does not discover, plan, or mutate
storage, and it must not be represented as a usable partition manager.

## Safe local gate

```text
cargo xtask ci
```

The command verifies the pinned toolchain, GitHub Action digest pinning,
formatting, linting, and Tier-1 unit tests. Tier 1 never requires elevation and
contains no destructive storage operations.

```text
cargo xtask test --tier 1
```

Tier 2 and Tier 3 deliberately fail closed until WP-020 implements the
disposable-environment proof required by SAFE-007.

## Work-package order

The dependency order is normative in Section 14 of the build specification.
After WP-000, the M0 packages that can begin are WP-010, WP-020, and WP-030.
WP-010 remains blocked on an accepted ADR-C1.

## License

None yet. This project is deliberately unlicensed while it is incomplete, so
default copyright applies and no rights to use, copy, modify, or distribute it
are granted. A license will be chosen before the first release.

Until then, do not submit outside contributions: without license terms, the
rights in a contribution are undefined for both sides.

