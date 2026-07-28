# Security policy

PartMan runs privileged code against block devices. A defect in the identity,
validation, authorization, or journal path can destroy data that has no backup,
so vulnerability reports are treated as higher severity than in most desktop
software.

## Reporting a vulnerability

Report privately through this repository's GitHub **Security → Report a
vulnerability** form, which opens a private security advisory. Do not open a
public issue, pull request, or discussion for a suspected vulnerability, and do
not include recovery keys, passphrases, key files, or unredacted diagnostic
bundles in a report (SAFE-006).

Private vulnerability reporting must remain enabled in the repository settings
for this channel to exist.

Useful detail, where you have it:

- The affected component from Section 4.2 of `AGENT_BUILD_SPEC.md`.
- The operating system, product version, and helper version.
- Whether the issue is reachable without elevation.
- A synthetic reproduction on a disposable image (SAFE-001). Never send a
  reproduction that requires targeting a real user disk.

## Scope

In scope, in descending priority:

- Any path that lets an unprivileged caller cause a privileged write.
- Bypass of device-identity binding, plan-hash binding, validity windows, or
  per-apply authorization (SAFE-003, SEC-001, SEC-002, HLP-003).
- Memory-unsafety or panics reachable from on-disk metadata parsers (SEC-003).
- Leakage of secrets into logs, journals, plans, or diagnostics (SAFE-006).
- Supply-chain weaknesses in the build, signing, or update path (SEC-004,
  SEC-008, SEC-010).

Out of scope: operations the product already reports as unsupported or
`blocked`, and data loss from a plan the user explicitly authorized after
accurate consequence text.

## Current status

The repository is pre-release and contains no storage discovery or mutation
code. See `README.md` for what actually exists today.
