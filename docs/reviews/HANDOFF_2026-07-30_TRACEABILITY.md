# Handoff — 2026-07-30, issues #51 and #39

Written to resume from, not to summarise. Everything below is either a decision
already made and landed, or a decision still open with the reason it is open.

## Landed

| PR | What | Issue |
| --- | --- | --- |
| #60 | WP-020 increment 2d: Windows fixture-root containment and other-name refusal | closes #51 |
| #62 | #39 increment 1: requirement annotations, anchored to the spec and to live tests | — |
| #63 | `deny.toml` licence election; the duplicate-major blind spot | closes #61 |
| #64 | Governance: each package declares how its traceability is produced | — |

## Open, and held deliberately

**PR #65 — #39 increment 2.** CI green, 12/12, judging `92c28aa`.

It does what it says: `docs/traceability/WP-000.md` is generated, hand edits fail
CI, and seven mutations are each refused by name.

**It also converts a 33-row hand-written table into a 13-row generated one.** The
18 missing rows are evidence the generator cannot express, not drift being
removed — so the file is currently *less complete* than the one it replaces.

**Decided 2026-07-30 by the decision owner: hold #65 and close the two gaps
below first.** Green CI is not the merge condition here; a conversion that loses
no evidence is. Do not merge it on the strength of the checkmark, and do not
merge it as a clean conversion — it is not one until the gaps close, and the PR
carries a comment saying so.

The work therefore lands on the #65 branch rather than after it: extend the
generator, regenerate, confirm the row count no longer drops, then merge one PR
that is a genuine conversion.

## The two gaps — now blocking the merge of #65, not just increment 3

Close both on the #65 branch. They also block increment 3, the rollout to
WP-010, WP-020 and WP-030, which would otherwise multiply the gap by four.

**Definition of done for #65:** regenerating `docs/traceability/WP-000.md` yields
a table that carries the evidence the 33-row hand-written version carried. If the
count is still short, say which rows are missing and why rather than rounding up.

1. **Section references have no requirement ID.** The vocabulary is built from
   IDs the specification *defines* (208 of them, from `### ID:` headings and
   `- **ID:**` list items). Eight of WP-000's lost rows cited Section 1.10, which
   is a numbered list item rather than a heading, so there is nothing to anchor
   it to. WP-000's largest body of evidence — the ownership gates — is therefore
   absent from its own table.

   Likely route: parse top-level section numbers from `## N.` headings and accept
   `Section N` or `Section N.M` where `N` is a real section. Weaker than the ID
   anchor but still anchored to the document rather than to a list this tool owns.

2. **Non-test evidence has nowhere to go.** Rows naming `deny.toml`,
   `.gitattributes`, `.github/CODEOWNERS`, `rust-toolchain.toml` or a command are
   real evidence and are not tests, and an annotation binds to a function.

   Likely route: a file-level annotation form, or a block in the work-package
   document that the generator folds in — the second needs care, because the
   point of generation is that the table cannot be hand-written.

## Facts measured this session, so they are not re-derived

**Windows containment (#51).** Holding the fixture root as a directory handle
with a share mode excluding `FILE_SHARE_DELETE` makes NTFS, `ReFS` and the
Windows SMB server refuse to rename or delete it. The **WSL 9p redirector does
not** — a swap staged from the Linux side succeeded with the handle held and the
child open returned the decoy. Hence UNC roots are refused outright. Containment
on Windows is enforcement by the filesystem, not resolution from a handle, and is
unproven for any root not on a local volume.

**Neither half of #51 needed FFI.** `std` opens a directory given
`FILE_FLAG_BACKUP_SEMANTICS`; `winapi-util` exposes `GetFileInformationByHandle`
safely. So the crate-placement decision the issue asked for was not made, and the
three rejected routes are recorded in `docs/work-packages/WP-020.md`.

**`FILE_FLAG_BACKUP_SEMANTICS` must never reach `open_child`.** Measured, junction
at the child name: no flags → refused; `OPEN_REPARSE_POINT` → refused;
`OPEN_REPARSE_POINT | BACKUP_SEMANTICS` → **opened**, and it reports `is_file()`.

**Test leaf names are unique workspace-wide** (216 at the time), which is what
makes annotation binding unambiguous. The generator refuses a duplicate rather
than assuming.

## Things that bit, and would bite again

- **Positional binding hides renames.** An annotation attaches to the next
  function below it, so renaming a test renames what it documents and nothing
  goes stale — and nothing is detectable. Fixed by making the annotation name its
  evidence. Found by running the mutation; reading the code, it looks correct.
- **A document defeated its own parser.** The paragraph explaining the
  ```` ```traceability ```` block names it, that mention precedes the real block,
  and a substring search read the sentence. Now a line-structural read. This is
  the fourth text scanner in this repository to be defeated this way.
- **Tests can be decoration.** Three written this session passed with the fix
  removed. Only the mutation run found them. Run the mutations.
- **A rebase nearly reverted a merged PR.** The #65 branch predated #63, so
  rebasing would have silently dropped 8 lines from `deny.toml` and 21 from the
  dependency policy. Read the staged diff, do not trust the rebase.

## Local toolchain, as of this session

Everything CI runs can now be run locally except macOS.

| Gate | Where |
| --- | --- |
| `cargo xtask ci` | Windows, and WSL Debian (pinned 1.96.0) |
| `cargo xtask cross-language` | Windows — Node 24.18 |
| `cargo xtask supply-chain` | Windows — cargo-deny 0.19.4, cargo-audit 0.22.2 |
| `cargo xtask probe` | WSL — util-linux 2.41, the version `prober.rs` records |
| `cargo xtask fuzz` | WSL — nightly-2026-07-01, cargo-fuzz 0.13.2, needs `g++` |

Set `CARGO_TARGET_DIR=/tmp/partman-linux-target` in WSL: the source lives on the
Windows drive and build artifacts on the Linux one, which is most of the speed
without moving the repository. **Do not move the working copy onto the WSL
filesystem** — that is the 9p path the interlock now refuses.

`C:` is NTFS and `D:` is `ReFS`, and this repository is on `D:`. `%TEMP%` is
therefore a different filesystem from the fixture root, which is why
`PARTMAN_TEST_ROOT` exists. CI runs the default root, so CI is evidence about its
own temporary directory.
