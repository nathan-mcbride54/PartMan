# Contributing

This repository implements `AGENT_BUILD_SPEC.md` 19.0.0. Safety requirements in
that document override all other instructions.

## License of contributions

PartMan is `MIT OR Apache-2.0` (ADR-0006). Contributions are inbound=outbound:
unless you state otherwise in writing, work you submit is offered under those
same dual terms, per Apache-2.0 §5. There is no CLA to sign.

Do not paste in code under other terms — including GPL or LGPL sources — even
in small quantities, and even reworded. PartMan reaches GPL storage tools by
running them as separate processes, never by copying or linking them; ADR-0006
explains why that boundary is what keeps the permissive license honest.

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

This is unprivileged Tier 1 only. Two higher-tier acceptances are registered —
`linux-loop-read-only` below and `si35-loop-capture`, WP-035's read-only
measurement instrument — plus a compiled destructive-suite registry reachable
only through `--suite <registered-name>`:

```text
cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only
```

Run any of them only with explicit privilege in a disposable non-WSL Linux VM,
after generating the fixtures and supplying the exact
`PARTMAN_DISPOSABLE_TOKEN` the generator records. The read-only acceptance
applies SAFE-001, SAFE-002, and every SAFE-007 factor to a non-destructive,
logical-content-read-only loop-device check: it launches no external storage
tool, issues no logical write, discard, or zero operation, and must prove the
fixture hashes unchanged. Linux may `fsync` inside the mapping ioctls and
write back already-dirty data or metadata, so the disposable-VM requirement
still matters. A registered destructive suite writes exactly its declared
byte ranges of one generated fixture and nothing else, with the fixture tree
regenerated and re-verified afterwards. Every generic destructive Tier-2
request and every Tier-3 request remains unavailable and refuses: a generic
request selects no suite.

If you changed anything under `crates/domain/src/canonical/`,
`packages/canonical/`, or `schemas/`, also run the MODEL-005 parity proof. It
needs a Node toolchain, which is why it is separate from `cargo xtask ci`:

```text
cargo xtask cross-language
```

Both implementations read `schemas/canonical-encoding-vectors.json`. Never give
either language its own copy of the vectors; an implementation checked against a
table it also owns proves only self-consistency.

Supply-chain checks use separately installed, pinned tools:

```text
cargo install cargo-deny --version 0.19.4 --locked
cargo install cargo-audit --version 0.22.2 --locked
cargo xtask supply-chain
```

## Reporting a vulnerability

Do not use an issue or a pull request. Follow `SECURITY.md`.

## Where to push

This repository lives on two forges, and a push can be refused by a gate that
runs on neither your machine nor GitHub. `docs/quality/forges-and-the-push-gate.md`
is the full arrangement; the part you need before your first push is short.

`origin` carries two push URLs, so `git push origin <branch>` reaches both
forges. Gitea is where work originates; GitHub holds the eleven required checks
that decide whether a change may merge.

A `gitleaks` pre-receive hook on the Gitea server refuses any push that
introduces a detectable secret, so the value never enters the shared repository.
If it refuses you, **rotate the credential first** — it existed in a commit you
created, and the push being refused is not the same as the value never having
existed — then rewrite the commits.

For a false positive, a `gitleaks:allow` comment on the offending line works. A
`.gitleaksignore` file does not: the hook scans a bare repository, which has no
working tree to read one from. Anything broader needs a rule-level allowlist in
the server-side configuration, which a push deliberately cannot change.

## Pull requests

Use one pull request per work package or assigned subtask. Complete every field
in the pull-request template, including assumptions, tests, changed owned
paths, requirement IDs, and follow-up packages.
