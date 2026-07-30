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

## PR #65 resumed — the deliberate hold is satisfied locally

The first version of PR #65 generated WP-000's document and refused hand edits,
but it converted 31 hand-written evidence rows into 13 generated evidence rows.
That 18-row difference contained evidence the generator could not express. The
decision owner correctly held the green PR until the conversion could prove zero
unexplained evidence loss.

The work has now landed on the PR branch:

- Stable references come from requirement definition sites and numeric
  specification headings. `Section 1.N` is narrower: an item is accepted only
  when that exact numbered operating-contract item exists, so mentioning an
  invented `Section 1.99` does not add it to the vocabulary.
- Non-test and aggregated evidence lives in a typed
  ```` ```traceability-evidence ```` block in the package-owned work-package
  document. Paths are checked against git and ownership, tests against compiled
  binaries, and commands against xtask's real parser.
- `docs/reviews/WP_000_TRACEABILITY_MIGRATION_2026-07-30.md` gives every one of
  the 31 old rows a disposition. None remains unsupported.
- The generated output currently has 46 evidence rows from 15 source-local
  annotations and 26 structured declarations. The larger number is expected:
  evidence carrying several requirements renders once for each requirement.
- Malformed/duplicate/unclosed blocks, repeated relationships, unknown
  requirements, invented sections, missing or cross-owned paths, missing tests,
  invalid commands, stale annotations, and hand-edited output are automated
  refusals.

The merge condition remains zero unexplained evidence loss, not equal row
counts. That condition is now met locally. PR #65 still requires its updated
three-platform CI and final diff review before merge; a green historical run on
the earlier 13-row version is not evidence for this implementation.

## Facts measured this session, so they are not re-derived

**Windows containment (#51).** Holding the fixture root as a directory handle
with a share mode excluding `FILE_SHARE_DELETE` makes NTFS, `ReFS` and the
Windows SMB server refuse to rename or delete it. The **WSL 9p redirector does
not** — a swap staged from the Linux side succeeded with the handle held and the
child open returned the decoy. Hence UNC roots are refused outright. Containment
on Windows is enforcement by the filesystem, not resolution from a handle.
`root_namespace_is_local` only distinguishes UNC prefixes; it cannot identify
WinFsp, Dokan, sshfs-win, or a mapped drive that canonicalizes to a drive letter.
The executable claim is therefore UNC refusal with a known third-party
drive-letter residual, not proof that every accepted root is on a locally served
Microsoft filesystem.

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
