# Forges and the push gate

This repository lives on two forges, and a push can be refused by a gate that
runs on neither your machine nor GitHub. Both facts are load-bearing and neither
was written down before this page.

## Which forge is which

| | |
| --- | --- |
| **Gitea** — `https://git.yarwood.network/N8/PartMan` | Where work originates. Self-hosted, public. |
| **GitHub** — `https://github.com/nathan-mcbride54/PartMan` | Backup mirror, and the **authoritative CI gate**. |

Both hold identical refs. The `origin` remote carries two push URLs, so an
ordinary `git push origin` reaches both.

**GitHub gates because the self-hosted tier cannot.** Six of the twelve checks
need `windows-2025` and `macos-15` runners that the local tier does not have,
and GitGuardian is a GitHub App with no Gitea equivalent. That is an
arrangement, not a migration in progress: merges are decided by GitHub's eleven
required contexts.

## Issue and pull-request numbers

The migration preserved numbering. Issues and pull requests **1–434 carry the
same number on both forges**, so every `#N` an ADR or work-package document
cites resolves on either one.

Gitea's index was then advanced past GitHub's range, so a Gitea-native object
cannot take a number GitHub has already used. This matters more than it looks:
a record that says "the sitting was named in the pull request body before the
merge" is a checkable claim, and a number that silently resolved to a different
object on the wrong forge would make such a record **false rather than broken**.

## CI runs in two tiers

- **GitHub Actions** — twelve checks, eleven required. This is the gate.
- **Gitea Actions**, on a self-hosted Linux runner — the five Linux checks.
  Fast feedback. It gates nothing.

The local tier is for finding out in about two minutes whether a Linux job will
fail, before spending a full gate on it. A green local run is not evidence that
a change may merge.

## The push gate

`gitleaks` runs as a **pre-receive hook on the Gitea server**. It refuses the
push, so a detected secret never enters the shared repository.

It runs gitleaks 8.30.1's default ruleset plus one added rule for **AWS access
key IDs**. The default set does not flag a lone `AKIA`-shaped key — defensibly,
since an access key ID authenticates nothing by itself — but the ID names the
account, survives in history after the secret is rotated, and is exactly what an
attacker needs in order to know which account to attack.

### When it refuses you

The push is rejected with `pre-receive hook declined` and the finding printed
with the secret redacted. Two things follow, in this order:

1. **Rotate the credential.** It existed in a commit you created and was written
   to disk; the push being refused is not the same as the value never having
   existed. Treat it as disclosed.
2. **Rewrite the commits** so the value is not in the history you push.

### False positives

A `gitleaks:allow` comment on the offending line works.

**A `.gitleaksignore` file does not.** The hook scans a *bare* repository, which
has no working tree for gitleaks to read one from. The escape hatch is a
rule-level allowlist in the server-side configuration.

That configuration is server-side deliberately. `--config` is gitleaks' highest
precedence source, so a `.gitleaks.toml` arriving in a push cannot replace the
ruleset that judges it. **A gate a push can reconfigure is not a gate.**

### It fails closed

A missing scanner binary or an unreadable ruleset refuses the push rather than
passing it. gitleaks exits 0 when the git command beneath it fails — reporting
"0 commits scanned" and "no leaks found", which by exit code alone is
indistinguishable from a clean scan — so the hook additionally checks that the
scan examined the commits it was given.

## What the push gate does not cover

Stated because a gate believed to cover more than it does is worse than one
whose limits are known.

- **It runs only on the Gitea server.** A push straight to GitHub —
  `git push github …`, the web UI, or a pull request from a fork — never reaches
  it. **GitGuardian is the only scanner on that path**, which is the reason it
  stays despite costing one second of a gate the local tier could not replace.
- **Content introduced by a merge commit is not scanned.** `git log -p` produces
  no diff for a merge, so a conflict resolution that introduces a secret is not
  seen. The branch's own commits were scanned when the branch was pushed.
- Findings print a `Link:` line pointing at `github.com` whichever forge refused
  the push. Cosmetic.
