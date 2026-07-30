# Changelog

All notable implementation changes are recorded here. Specification changes
remain controlled by the changelog in `AGENT_BUILD_SPEC.md`.

## Unreleased

### Added

- WP-010 increment 1: the `pce/1` canonical encoding. `schemas/canonical-encoding.md`
  specifies it normatively, and `crates/domain` implements the encoder, a strict
  validating decoder, and SHA-256 hashing over canonical bytes (MODEL-005).
  Golden vectors pin the encoding byte-for-byte, including the `2^53` and
  `2^64 - 1` boundaries that RFC 8785 could not have carried as JSON numbers.
  The decoder rejects rather than repairs: non-shortest arguments, floats, tags,
  indefinite lengths, non-text or misordered map keys, duplicate keys,
  ill-formed UTF-8, lengths beyond the input, nesting past a fixed depth limit,
  and trailing bytes.
- WP-010 increment 2: `packages/canonical`, the TypeScript half of MODEL-005.
  Both languages now read one shared fixture,
  `schemas/canonical-encoding-vectors.json`, so parity is proven against a
  single source rather than two per-language tables that could drift.
  `cargo xtask cross-language` runs the proof and gates CI as its own job. The
  package has no runtime dependencies: hashing uses Web Crypto and testing uses
  `node:test`.
- WP-020 increment 1: `crates/fixtures`, the deterministic disk-image generator,
  and the SAFE-007 disposable-target interlock. `cargo xtask fixtures` writes
  synthetic disk images — GPT, 4Kn GPT, MBR, blank, damaged-primary,
  conflicting-tables, hybrid MBR/GPT, APM, and on-disk signatures for LUKS2,
  LVM2 and mdraid — into the gitignored `tests/generated/`, each
  a pure function of the code that builds it, so two machines produce identical
  bytes and nothing binary is ever committed. The interlock requires all three of
  SAFE-007's proofs and computes disposability from a target's own bytes rather
  than accepting an assertion: a block device cannot pass, because its bytes will
  never equal a generated fixture. Tier 2 and Tier 3 still refuse, now for the
  honest reason that no destructive suite exists to run.
- WP-020 increment 1d: `crates/fixtures/src/evidence.rs`, which binds every
  fixture's bytes to the rationale recorded beside it. Until now nothing did:
  every layout and signature test rebuilt its own image from its own literals,
  so the catalogue was free to produce something else. Measured before it was
  fixed — with the LUKS2 builder emptied to a blank image and the
  multi-signature builder stripped of the stale mdraid superblock the Part 5
  asymmetry finding rests on, **all 64 tests passed**. Each catalogue entry now
  has a claim computed from its bytes; the set is exhaustive in both directions;
  each claim is paired with a mutation it must reject, so a check that cannot
  fail is caught; and `generate` refuses to write an image that no longer serves
  its purpose, naming what was lost. The oracles reimplement CRC-32, LVM2's CRC
  and mdraid's folded sum by different methods from the writers they check, each
  anchored outside the repository — to the published IEEE check value, and to
  three checksum fields `libblkid` 2.41 accepted, pinned with their provenance.

  The module was put through an adversarial pass before being proposed, and it
  found the first version repeating the defect it was written to end. The gate in
  `generate` was load-bearing on nothing — deleting it kept all 74 tests green,
  because every test fed it the real catalogue, which passes. Its
  "anchored outside this repository" claim held for one checksum of three:
  changing an initial constant in both writer and oracle kept everything green
  while making every fixture undetectable. And ten claims accepted mutations that
  destroyed a fixture's purpose while leaving its checksums valid — most sharply,
  "two tables that disagree" was proven by comparing entry-array CRCs, which one
  character of a partition *name* satisfies while both copies describe identical
  extents. All are closed, and the details are in
  `docs/work-packages/WP-020.md`.
- WP-020 increment 1e: `cargo xtask probe`, which re-runs `blkid` and `wipefs`
  over every generated fixture and compares against the expectations recorded in
  `crates/fixtures/src/prober.rs`. This closes the project review's open finding
  that real-prober acceptance was "manual, not regression-protected" — the last
  place in the package where an important property rested on someone having
  looked once. It needs Linux, so CI runs it as its own job; both tools are
  read-only and are handed regular files, never a device.

  A verbatim capture of `libblkid` 2.41's output is embedded in the tests and the
  recorded table is compared against it, so the table is checked on machines with
  no prober too — otherwise a transcription slip between the measurement and the
  table would look exactly like a passing test. The comparison is proved capable
  of failing in all four directions: a format no longer detected, a changed
  answer, a lost signature, and an added one.

  **Its first run falsified a claim increment 1 had recorded.** On util-linux
  2.39.3, which stock `ubuntu-24.04` ships, `blkid -p` reports nothing at all for
  `mdraid-1.2-member-512.img`, while `wipefs` still lists the superblock and both
  tools agree about every other fixture — including the 0.90 superblock in the
  stale-pair image. Increment 1 checked the signature writers by hand against
  2.41 on one machine and recorded the result as unconditional, so FS-004 Linux
  RAID and LIN-005 are **not** established on that platform. The expectation is
  now version-keyed rather than relaxed: below 2.41 the recorded answer is
  silence, so a prober that starts naming the fixture fails just as one that
  stops does. Which condition 2.39.3 rejects is unestablished — both versions'
  checksum routines are arithmetically identical, and the fixture satisfies the
  magic, `major_version` and `super_offset` checks — and it is recorded as
  unestablished rather than guessed at.
- WP-030 increment 1: `schemas/design-tokens.json` and `crates/tokens`, the
  design tokens and the accessibility harness that computes UI-001, UI-007 and
  UI-008 from them. The token file is the single source of truth and lives in
  `schemas/` for the reason `AGENTS.md` already records for the canonical
  vectors: when the front end arrives it must read *this* file, because an
  implementation checked against a table it also owns proves only
  self-consistency. `cargo xtask tokens` runs the audit and is part of
  `cargo xtask ci`.

  **The first palette failed its own harness on ten counts.** Chosen by eye and
  entirely reasonable-looking, it put `severity.reversible` — PLAN-004's "fully
  undoable" — at delta-E 10.1 from `severity.destructive` — "data is
  intentionally destroyed" — under deuteranopia, against a floor of 12. In the
  high-contrast theme, the one a low-vision user is most likely to choose, the
  same pair measured 4.8. Three further risk pairs collapsed the same way and
  three borders sat below WCAG's 3:1 floor for interface components. The floor
  was not lowered: the severity ramp now varies in lightness as well as hue,
  because lightness survives every colour-vision deficiency and the red-green
  axis does not, and the light theme's `reversible` is teal-leaning so it keeps
  a blue component deuteranopia preserves. Closest surviving pair is 21.9.

  Every check is paired with a mutation it must reject, and each was confirmed
  by deleting the check it targets and watching the table go red — the deletion
  sweep WP-020 established after finding a gate that was load-bearing on
  nothing. The colour maths is anchored outside the repository (black on white
  is WCAG's published 21:1; black against white is delta-E 100 because CIELAB
  lightness runs 0..=100), and the colour-vision matrices are checked by their
  defining property — red and green converge under protanopia and deuteranopia
  but not tritanopia, and greys are untouched by all three — rather than by
  trusting transcribed digits.

  What it does **not** establish is recorded in `docs/work-packages/WP-030.md`
  and repeated in the harness output on every run: it renders nothing, so the
  keyboard, screen-reader, zoom and reduced-motion halves of UI-008 are
  untouched; only declared pairings are checked, so a combination the front end
  invents is invisible to it; and the colour-vision check is a model, not a
  proof — UI-007's redundant channels are the guarantee. M0's "accessibility
  harness runs" criterion is therefore **partially** met.
- `cargo xtask verify-ownership`, closing the mechanically decidable half of
  Section 1.10. Every `docs/work-packages/WP-*.md` now carries an `owned-paths`
  block, which is the same text a reviewer reads, so the prose and the enforced
  data are one thing rather than two that drift. The check refuses a tracked
  file no package claims and a claim matching no file — both mutations were run
  — and reports overlaps rather than forbidding them, because `tools/xtask/**`
  is genuinely shared by three packages and forbidding that would push the
  sharing into prose where nothing can see it. All 100 tracked files are
  claimed. It runs inside `cargo xtask ci`.

  Only exact paths and `directory/**` are understood, and anything else is an
  error rather than a pattern silently matching nothing — the failure mode the
  action scanner was audited for twice.

  WP-030's increment-2 assignment is reserved *ahead of the work*, in an
  `owned-paths-reserved` block the checker reports rather than requires to
  match. Both audits observed that WP-030's assignment did not authorize
  creating a Tauri shell and that widening scope in a pull-request description
  afterwards is the pattern ownership exists to prevent. The reservation also
  records the audits' design constraint: the front end consumes a generated
  typed accessor, never a copy of the palette.

  What this does **not** do is decide whether a given change came from the
  package owning the path — that needs a pull-request-to-package mapping this
  repository does not carry, and it is the remaining half of issue #39.
- ADR-0007, accepted, deciding what SAFE-007's disposable-test token proves.
  WP-020 carried "decide a genuinely independent token factor" as an open
  precondition since increment 1, and both audits repeated it. The queued
  answer — add an entropy source — turned out to be wrong in an instructive
  way: `authorize` trusts nothing inside the directory it verifies, because
  accepting a caller-supplied manifest was a defect that let a hand-written one
  authorize an arbitrary target, and a per-generation random token cannot be
  compiled in. The interlock would have to read it from the fixture root,
  re-creating that exact trust dependency, so randomness would have added a
  dependency and a writable-file trust while defeating nobody.

  Read exactly, SAFE-007 requires the three factors to be *present* and forbids
  one environment variable from standing in for all of them; both hold, and it
  does not require independence. The token is an operator-intent proof, the
  documents already said so, and the precondition is closed by decision rather
  than by code. A real third factor needs state outside both the source tree and
  the fixture root, which is a T2/T3 lab-architecture question recorded as the
  ADR's revisit condition.
- SI-36 filed: SAFE-009 neither permits nor forbids reviewed `unsafe` in a
  test-fixture crate. WP-020's Windows other-name check needs link count by
  handle, `MetadataExt::number_of_links` is unstable behind `windows_by_handle`
  on the pinned 1.96.0 toolchain, and the FFI alternative runs into SAFE-009's
  two lists naming `crates/fixtures` in neither. An enumeration is not a rule,
  so per Section 0.2 it is filed rather than guessed. The residual is recorded
  and narrow: while an authorization is held the Windows share mode refuses
  writes through any name for the object.
- ADR-C1, accepted, fixing the canonical encoding and hash strategy.
- ADR-C5, accepted, fixing the aggregation vocabulary: one `Aggregate` node in
  place of three undefined Section 5 names, on-disk signatures as their own
  nodes, and `StorageSnapshot`. Landed as spec 4.0.0. It resolves four of the
  conflicts blocking WP-010 increment 3 and does not unblock it.

- WP-000 repository foundation: pinned Rust workspace, Tier-1 task runner,
  cross-platform CI, formatting/lint policy, dependency policy, and ADR
  template.
- `cargo xtask verify-actions` enforces SEC-010 digest pinning for GitHub
  Actions. It runs inside `cargo xtask ci` and as a Tier-1 test, and fails
  closed when no workflow can be read.
- `.gitattributes` normalizes line endings to LF in every working tree.
- `SECURITY.md` defines a private disclosure channel and reporting scope.
- Job timeouts on both CI jobs.

### Changed

- `cargo xtask verify-actions` enforces the rule it reports. SEC-010 and
  `AGENTS.md` require every action pinned to a full commit SHA **with the release
  tag in a trailing comment**, and the error message said exactly that — but the
  scanner stripped the comment before checking and `is_pinned` validated only the
  SHA, so a bare 40-character digest passed a gate that claimed to require a tag.
  The comment is now carried through and must name a version. Without one a
  reviewer cannot tell which release a digest corresponds to, so a bump becomes
  40 hex characters to resolve by hand. The repository's own workflows already
  complied, so nothing had to change to pass it — which is why the gap survived.
- `xtask` separates command parsing from execution, so every documented task,
  rejected task, and tier decision is unit-tested without launching a
  subprocess.
- Removed `[build] rustflags = ["-Dwarnings"]` from `.cargo/config.toml`. Cargo
  discovers that file from the working directory, so the flag applied to every
  crate compiled from anywhere inside the repository: third-party dependencies,
  out-of-workspace manifests, and the supply-chain job's
  `cargo install cargo-deny`, which built its entire dependency tree under
  `-D warnings`. That tree compiles warning-free today, so nothing was failing
  yet, but the exposure grows with every dependency added and every rustc
  release that introduces a lint. Workspace lint scope now comes from
  `[workspace.lints]`, and `cargo xtask ci` still fails on any warning in
  workspace code through `cargo clippy -- -D warnings`.

- ~~The project is deliberately unlicensed until it is complete.~~ Superseded
  below. PartMan is now `MIT OR Apache-2.0`.

- The project is licensed `MIT OR Apache-2.0` at the recipient's choice
  (ADR-0006), the Rust and Tauri ecosystem standard. `LICENSE-MIT` and
  `LICENSE-APACHE` carry the texts; every workspace member, the out-of-workspace
  `fuzz` crate, and `packages/canonical/package.json` declare the expression.
  Apache-2.0 supplies the explicit patent grant that MIT lacks — worth having
  for code that drives NTFS, exFAT, and APFS paths — while the MIT arm keeps the
  result usable by GPL-2.0-only projects, which Apache-2.0 alone would not.
  Both arms were already on `deny.toml`'s allow-list, so no supply-chain rule
  was relaxed to accommodate the choice.

  `[licenses.private]` is now `ignore = false`. That exemption existed only to
  stop `cargo deny` reporting `error[unlicensed]` against the unlicensed
  workspace; with the cause gone it is removed rather than left dormant, so the
  project's own crates are checked by the gate that checks every dependency.
  This closes the WP-000 known gap that recorded SEC-005's license inventory as
  unsatisfiable. Two manifests remain outside the gate and are recorded as gaps
  rather than counted: `fuzz/Cargo.toml` is outside cargo-deny's graph, and no
  license gate reads `packages/canonical/package.json`.

  ADR-0006 also makes the GPL boundary binding, which the unlicensed state had
  made moot: PartMan invokes GPL storage tools as separate processes under
  SAFE-004 and reaches UDisks2 over D-Bus, may link LGPL libraries such as
  `libblkid` and `libblockdev` dynamically, and MUST NOT link a GPL library.
  `libparted` is named specifically — it is the obvious dependency for a
  partition editor, and `cargo deny` cannot catch it, because a `-sys` crate
  declares its own license and not that of the C library it links.

- Contributions are accepted, inbound=outbound under Apache-2.0 §5, with no CLA.
  `CONTRIBUTING.md` previously barred outside contributions because the rights
  in one were undefined for both sides; that reason no longer holds.

- `cross-language` and `supply-chain` run on all three operating systems instead
  of Linux alone. Both were narrowed to save metered private-repository runner
  minutes, where Windows bills 2x and macOS 10x, and that constraint is gone.
  The widening is not symmetric bookkeeping: the MODEL-005 parity proof can fail
  on CRLF translation of the shared vector file or on a platform-specific Node
  build, and cargo-deny resolves a per-target graph, so once the platform
  helpers add `windows-sys` or `core-foundation` a Linux-only run would be blind
  to advisories reachable only from Windows or macOS. `prober-acceptance` and
  `fuzz-smoke` stay Linux-only for reasons that were never cost — `blkid` and
  `wipefs` have no Windows or macOS counterpart, and cargo-fuzz has no supported
  Windows target while the decoder under test is byte-oriented and
  endian-independent. Both reasons are now comments in the workflow, so neither
  job gets widened later for the appearance of consistency.

### Fixed

- Action discovery is a structural YAML parse, reversing a decision this project
  defended twice. Three text-based attempts were each defeated by valid YAML,
  and every one of them reported *success with one fewer reference* — silence
  shaped like a pass. The third attempt, a sweep for `owner/repo@ref` tokens
  described here as "syntax-independent" and "unbypassable", fell three ways at
  once: `"actions/checkout@v7"` hides the `@` behind a YAML escape no text
  search decodes; `docker://alpine:3.20` is a documented, mutable step reference
  containing no `@` at all; and a local action outside `.github/actions/` was
  never recursed into, so its own remote references went unread.

  `yaml-rust2` now parses each workflow and every `uses` mapping key in the tree
  is a reference with its value decoded — context-free, so a position GitHub
  adds later cannot be missed. Containers must be pinned by `@sha256:` digest.
  Local references are resolved wherever they live, must carry action metadata
  if they name a directory, must stay inside the repository, and are recursed
  into with a visited set that survives cycles. Unparseable YAML is a violation
  rather than a skip, and the release-tag comment survives as a separate textual
  auditability layer rather than as discovery.

  All three bypasses are permanent regressions. A deletion sweep also caught one
  of the *new* tests not being load-bearing: removing the container-digest
  branch still refused `docker://alpine:3.20`, because `is_pinned` reports it as
  "not pinned to a full commit SHA" — true, but it tells a reader to look for a
  git SHA on a Docker image. The test now asserts the container-specific
  guidance the branch exists to produce.

- **WP-020 precondition 1 is reopened.** `O_NOFOLLOW` constrains only the final
  path component, which `open(2)` documents plainly and increment 2b overlooked.
  Renaming the fixture root aside and putting a symlink in its place redirects
  the open to an out-of-root file, and matching length, digest, type and link
  count then all pass — the same lesson as
  `object_verification_alone_cannot_prove_root_membership`, one directory up.
  Closing it needs a held root-directory object and an `openat`-style
  direct-child open; more `canonicalize` calls cannot. Tier 2 stays unavailable
  on every platform until that lands.

- SI-36 is **withdrawn the day it was filed.** SAFE-009 permits `unsafe` *only*
  in adapter, FFI, and helper crates, which forbids it in `crates/fixtures` and
  names the route in the same clause. Reading the omission of that crate from
  both lists as ambiguity was using the §0.2 process to convert an
  implementation-location constraint into permission by omission. Precondition 3
  is ordinary work with a known route, not a blocked decision.

- ADR-0007's justification is corrected. It said the token proves the operator
  ran the generator; a pure function of public source cannot prove that history,
  since anyone with the repository can compute the value. It proves only that
  the invocation presented the exact build-derived value — accident friction,
  which is what the decision actually rests on. The decision stands.

- Stale documentation corrected across the review set: `HANDOFF`'s execution
  order, `DECISION_NOTES`' disproved claims, the progress report, the audit
  response, README's WP-000/WP-020/WP-030 rows, WP-000's "only filesystem reads"
  sentence, and four traceability headers that named fewer increments or
  requirements than their own evidence tables contained. The token-mutation
  count said 12 where the table holds 26, and the ownership count said 100 where
  the tree holds 101 — that one is now left to the tool to print rather than
  restated in prose that goes stale on the next file added.

- WP-020 increment 2b: the object binding now starts at the open. Increment 2a
  bound every check to the handle but opened the target by path a second time,
  and the 2026-07-29 follow-up audit showed what lives in that gap: replace
  `root/name` with a symlink to an out-of-root file holding the fixture's exact
  bytes, and the handle is outside the fixture tree while every handle-based
  check — regular file, link count, length, digest — accepts it. Increment 2a
  had claimed a raced symlink was harmless *because* the object is verified
  after opening; `object_verification_alone_cannot_prove_root_membership` now
  records why that was wrong, by demonstrating the object checks accepting an
  out-of-root file. They establish fixture shape; containment is not a property
  of content, and a user's ordinary file may hold those bytes.

  The open refuses to leave the directory: `O_NOFOLLOW` on Unix, taken from
  `libc` because the value differs across Linux, macOS and the BSDs, and
  `FILE_FLAG_OPEN_REPARSE_POINT` on Windows. A test seam fires between
  canonicalization and open, so the race is scheduled rather than sampled —
  `a_symlink_swapped_in_before_open_is_refused` performs the audit's exact
  substitution, and a portable companion covers the seam on Windows, where
  creating a symlink needs a privilege CI cannot be relied on to hold. Removing
  the seam fails both by name.

  Still not claimed closed: the Windows hard-link vector. A hard link is not a
  reparse point, and stable Rust exposes link counts on Windows only behind an
  unstable feature — that is WP-020 precondition 3, and the reason precondition
  1 is narrowed rather than finished.

- The verified handle is handed over rewound. Hashing the contents left the
  cursor at EOF, so a consumer assuming a fresh file would have appended;
  the replace-after-authorization test having to seek explicitly was the smell.

- The action-pin gate no longer depends on recognising the `uses` key. Two
  audits in a row defeated the key-shaped reader with valid YAML it could not
  parse — a quoted key, then an anchored one, `&pin uses: actions/checkout@v7` —
  and each time it reported success having counted one *fewer* reference. A
  mutable tag was invisible rather than rejected, which is the worst failure
  mode a gate has: silence that looks like a pass. Discovery is now
  syntax-independent. An action reference must contain `owner/repo@ref`
  verbatim, whatever surrounds it, so a sweep for that shape finds every
  reference and anything the reader could not attribute to a `uses:` key is a
  violation. Anchors, tags, flow mappings, and every future spelling are
  covered by the same property, rather than by extending a subset one
  demonstrated bypass at a time. Verified against four spellings including both
  the audit's bypass and the tag variant it named as the same class.

- `verify-licenses` is semantic rather than lexical. It matched trimmed lines,
  so the follow-up audit moved the JSON property under `metadata`: the line
  still read `"license": "MIT OR Apache-2.0"` while the document's root
  `license` was `undefined`, and nine artefacts passed. `package.json` is now
  parsed as JSON with the property required at the root, Cargo licences come
  from `cargo metadata --locked --no-deps` (which resolves
  `license.workspace` inheritance the way the toolchain does), and a Cargo
  manifest that neither workspace includes is a violation because no gate
  resolves it. The blanket skip for directories named `generated` is gone.

- `supply-chain` no longer repairs the fuzz lock before auditing it. The
  preflight lived only in `fuzz()`, but `cargo deny` resolves the manifest to
  build its graph, so `supply-chain` silently restored a deleted
  `fuzz/Cargo.lock` entry and audited the repaired copy — the policy tool
  committing the fail-open shape it exists to catch, and leaving a later `fuzz`
  preflight nothing to refuse. `verify_fuzz_lock` is now shared and runs first
  in both entry points; the same mutation now refuses and the lock stays stale.

- `tokenSetVersion` is validated instead of merely present. It was only
  required to be non-empty, so `"not-a-version"` passed while WP-030 and the
  audit response both described parsing as "versioned" — a field nothing
  compares against is documentation. It is now compared against
  `REQUIRED_TOKEN_SET_VERSION` in `policy.rs`, alongside `specVersion`.

- WP-020 increment 2a: authorization holds the object it verified, not the
  name it found it under. The 2026-07-29 audit ranked this the most important
  precondition before any Tier-2 write: `Authorization` carried a
  `Vec<PathBuf>`, and a name can be rebound between verification and
  destructive use. It now carries open `File` handles — `fstat`, the hard-link
  count, the length, and every content byte are read through the handle, and
  the handle itself is what a destructive consumer receives, so renaming or
  swapping the path afterwards changes what the *name* means, never what the
  authorization holds. On Windows the handle's share mode refuses concurrent
  writes, deletion, and renames — through any name, hard links included — for
  as long as the authorization lives; the replace-after-authorization test
  asserts those refusals there and asserts write-through-handle reaches the
  verified object on POSIX. The proof is non-cloneable (a `compile_fail`
  doctest pins it) and consumed once.

  The first version of this fix repeated the defect it was written to end,
  and only planting regressions found it: downgrading the handle `fstat` to a
  by-path `stat` kept every test green, because the difference only shows
  during a race no unit test can stage. `verify_object` now takes no usable
  path, and its test deletes the path before verifying — handle-purity proven
  deterministically. Both planted regressions (`stat`-by-path,
  `fs::read`-by-path) fail that test by name.

  Deliberately not done here: platform no-follow open flags (hardcoding
  `O_NOFOLLOW` values without `libc` is its own defect factory; the by-name
  symlink refusal stays as hygiene and post-open object verification makes a
  raced symlink harmless), and the independent-token decision, which needs an
  entropy source and is a dependency-policy change — still recorded open in
  `docs/work-packages/WP-020.md`.

- The gate can no longer repair the lockfile it claims to enforce. The
  2026-07-29 audit deleted a package entry from `Cargo.lock` and ran
  `cargo xtask ci`: Cargo silently regenerated the entry while building `xtask`
  itself, and all 160 tests passed against a lockfile the repository had never
  committed — the internal `--locked` flags bind only once the binary is
  built. `--locked` now sits in the `xtask` alias, the boundary that builds the
  gate; the same mutation now refuses with "cannot update the lock file". A
  Tier-1 test fails by name if the alias loses the flag.

- `cargo xtask verify-actions` no longer goes blind on valid YAML. The audit
  rewrote one pinned step as `"uses": actions/checkout@v7` — the same YAML key,
  and GitHub executes it — and the scanner reported *success with one fewer
  reference*: the mutable tag was invisible rather than rejected. The scanner
  now enforces a deliberately small YAML subset and **refuses what it cannot
  positively read**: quoted keys are recognized and checked, while flow
  mappings, block scalars, aliases, anchors, escaped quoted keys, explicit-key
  syntax, and values continuing on the next line are each a named violation.
  Local composite actions under `.github/actions/` are scanned too — exempting
  `./` references is safe only if their own metadata is read. What remains
  manual is recorded: nothing verifies a tag comment resolves to its pinned
  SHA, and that is a review obligation, not an automated check.

- The fuzz crate's dependency graph is no longer outside every gate.
  `fuzz/Cargo.lock` was gitignored and the crate is excluded from the
  workspace, so every fresh CI run resolved `libfuzzer-sys` and `arbitrary` to
  whatever the registry served that day and ran their build scripts — on the
  job that executes hostile-byte parser tests, checked by no advisory, licence,
  or source policy. The lock is committed; `cargo xtask fuzz` refuses a stale
  lock before the nightly toolchain is even involved; `cargo xtask
  supply-chain` checks the fuzz graph as a second graph under the same
  `deny.toml` (which required allowing NCSA — `libfuzzer-sys` is
  `(MIT OR Apache-2.0) AND NCSA`, so the permissive NCSA arm is mandatory, and
  the addition is commented in `deny.toml`); and a `/fuzz` Dependabot entry
  updates what nothing previously watched.

- `cargo xtask verify-licenses` closes the recorded WP-000 gap: it walks every
  `Cargo.toml` and `package.json`, fails unless each declares
  `MIT OR Apache-2.0` and both licence texts exist, and runs inside
  `cargo xtask ci`. Previously `fuzz/Cargo.toml` and
  `packages/canonical/package.json` could lose their licence keys with CI
  green.

- WP-000 is reclassified from Complete to in progress. Section 12 defines done
  as generated traceability showing a package's evidence, and
  `docs/traceability/` is hand-maintained; the audit also demonstrated two
  fail-open evidence paths in what Complete claimed to cover. The README row
  now says what is delivered and what is not, and the hosted-runner deviation
  from SEC-010's builder-image digest rule is documented with its residual risk
  in `docs/quality/dependency-policy.md` instead of being silently absorbed.

- WP-030 increment 1a: the accessibility harness no longer takes its standards
  from the file it audits. The 2026-07-29 project audit demonstrated two live
  bypasses through the whole Tier-1 gate: lowering the token file's own `text`
  threshold from 4.5 to 3.0 let normal-size text pass at **3.33:1**, and
  deleting `entity.container` from every theme, pairing and channel table at
  once passed with six fewer checks — a coordinated omission indistinguishable
  from a smaller product, while UI-003 requires containers to be represented.
  Both are the self-consistency failure `AGENTS.md` records for the canonical
  vectors, committed inside the harness written to enforce that rule on
  colours: increment 1's mutation table mutated colours thoroughly and never
  mutated the policy.

  The WCAG floors, the colour-separation floor, the required themes and the
  full UI-003/PLAN-004/UI-011 role vocabulary now live in
  `crates/tokens/src/policy.rs`, outside the audited file. The JSON restates
  the floors for a front end to read, and the audit requires the restatement to
  agree exactly — a lowered value is a finding, not a new setting. Twelve
  mutations were added (threshold lowering with and without a hidden colour,
  threshold removal, role deletion, pairing removal, risk-pair removal, role
  invention, version mismatches); re-running the audit's own reproductions now
  yields 3 and 2 findings where both yielded none. The reader is genuinely
  strict too: `deny_unknown_fields` throughout, so a misspelled
  `nonColorChannels` key can no longer silently disable the UI-007 check.

- `docs/quality/test-tiers.md` overstated the SAFE-007 token: it said the token
  "cannot be known without having generated that fixture set", but the token is
  a pure function of the source, identical on every machine building the same
  commit. The file now carries the honest account `docs/work-packages/WP-020.md`
  always had — the factor is weak, three factors are effectively two, and the
  interlock's strength rests on targets byte-equalling generated images.

- The TypeScript encoder had no `default` arm, so an unrecognized value kind fell
  through the switch, `encode` returned zero bytes, and `hash` published SHA-256
  of the empty string as a well-formed digest over an artifact with no encoding.
  Rust cannot reach this, because its `match` is exhaustive at compile time.
  Payload runtime types are now checked too: `TextEncoder` coerces a non-string
  rather than failing, and `Uint8Array.from` truncates modulo 256. `fromHex`
  refused nothing outside hex, where `Number.parseInt` yields NaN and stores as
  0, so two distinct textual digests decoded to identical bytes.

- Three signature fixtures wrote fields at the wrong offsets, found by an audit
  of the project review. The mdraid 0.90 set UUID occupied `utime`, `state` and
  `active_disks` rather than words 13 to 15, leaving the array identity three
  quarters zero — `blkid` reported it as `fb2871eb-0000-0000-0000-000000000000`.
  LUKS2 wrote its checksum algorithm and UUID inside the 48-byte label field,
  leaving the fixture with no UUID at all. ext4 declared 8 MiB of blocks on a
  4 MiB device, having reused a sector count as a block count. Each correction is
  confirmed against `libblkid` rather than against the struct definition.

- The TypeScript encoder authenticated a forged boolean as the **opposite**
  logical value. The `bool` arm used JavaScript truthiness, so a runtime-forged
  `{ kind: 'bool', value: 'false' }` encoded as `f5` — canonical `true` — and
  `hash` published a digest over the other value, on the MODEL-005 and SEC-001
  authorization boundary. TypeScript types do not protect an object that arrived
  as JSON, over RPC, from a plugin, or as `unknown`, which is why `text` and
  `bytes` already validated at runtime; the reasoning had simply not been carried
  to the rest. Every variant now validates, including map keys — where
  `requireWellFormed` iterated `.length`, `undefined` on a number, so its loop
  never ran and `utf8` then coerced `1` to `"1"`, silently turning a map keyed by
  a forged number into one keyed by text. Rust was unaffected: its `match` is
  exhaustive over a real enum.

  An adversarial pass on that fix found the guards checked fields while a payload
  can lie *between two reads*: `kind` was read twice, containers declared a count
  and then wrote a body from a second read, `bytes` trusted a `Symbol.iterator`
  that `Uint8Array.from` truncated modulo 256, and `instanceof` let a
  prototype-only fake through to a native `TypeError`. Each field is now read
  exactly once and containers are snapshotted before being measured. The tests
  were vacuous the same way — the `array` case used a string, which has a
  `.length`, so the guard never ran and deleting it left the suite green. Every
  case now names the phrase its refusal must contain.
- Raw-byte hashing is no longer a way around strict decode. Both languages
  exported a function that hashed whatever it was handed, documented as "use this
  only for bytes produced by `encode` or accepted by `decode`" — an instruction,
  not a guarantee. `hash_canonical_bytes` and `hashCanonicalBytes` are replaced by
  `hash_encoded` / `hashEncoded`, which decode first, so canonicality is proven
  rather than asserted. The TypeScript version validated a *prefix* and hashed a
  *buffer* until the same pass caught it: `decode` walks the array through its
  `length` property while `crypto.subtle.digest` reads the underlying buffer. No
  digest changed; both languages still reproduce every recorded `sha256`.
- Two fail-open edges and a flaky test harness, found by a progress review. The
  sharpest is one the prober increment introduced in the module the evidence
  increment wrote about: both prober parsers **discarded what they could not
  read**, so an unreadable row was not an "unexpected signature" the comparison
  would report — it was no observation at all, and on the fixture whose
  expectation is *nothing*, an entirely changed output shape parsed as empty and
  passed. The module documentation claimed the signature set was compared in both
  directions. Both parsers now return `Result` and refuse a missing `=`, an empty
  or repeated key, a bad offset, a typeless row, a misplaced header, and a
  repeated row; `probe_output` no longer uses `from_utf8_lossy`.
- Fixture-directory pruning inferred ownership from a filename:
  `root.join(MANIFEST_FILE).is_file()` establishes nothing about who wrote the
  file and follows a symlink besides, so any directory holding an unrelated file
  or link named `MANIFEST` was treated as ours and could lose its other regular
  files. Ownership is now computed — a regular file reached without following a
  link, parsing as one of our manifests, with the token recomputed from its own
  entries. Every failure is a refusal to prune.
- Test sandboxes used fixed paths and deleted them at setup and drop, so two
  concurrent `cargo test` runs of the fixtures crate erased each other's trees —
  in the suite that gates destructive execution. Names now carry the process id
  and a per-process counter.
- Two documented claims that had outrun their code, corrected in opposite
  directions. `corrupt_primary_header_crc` still described itself as producing
  ADR-C3's `Indeterminate` state, contradicting `write_conflicting_backup` in the
  same file, which had already been corrected; the layout test repeated it. Both
  now say *recoverable*. And the review response's note that
  `authorization_cannot_be_forged_outside_this_module` "reportedly stays green
  with `verify_target`'s body short-circuited" — read as evidence the interlock
  suite was blind to that mutation — is false. It was run: **ten tests fail**,
  covering traversal, subdirectory copies, modified bytes, wrong names, missing
  targets and mixed requests. The named test does stay green, because it asserts
  a compile-time property rather than target verification. "Reportedly" marked a
  claim that had never been executed, in a document written about that exact
  failure.
- `every_generated_fixture_authorizes` asserted `targets.len() >= 8`. A floor
  lets catalogue entries be deleted silently while the test still reads as
  coverage; it is now an equality against the catalogue's own length.
- `gpt-missing-backup-512.img` had a backup. It zeroed the last sector only,
  leaving 16 KiB of byte-identical backup entry array at LBAs 8159 to 8190 —
  which any recovery tool that scans rather than seeking to the last LBA would
  find, on a fixture named for having no backup. The whole backup copy is now
  erased.
- `gpt-basic-512.img` and `gpt-basic-4kn.img` shared a disk GUID, both deriving
  it from the literal `"gpt-basic"`. Two different media with one identity is a
  manufactured instance of the collision SI-27 is trying to reason about. Found
  by a new catalogue-wide identity check that no single-image claim could see.

- A subdirectory bypass in the SAFE-007 interlock, introduced by the fix for the
  forged-manifest defect and missed by its own new tests. Containment was a path
  prefix while the name came from `file_name()`, so a byte-identical copy at
  `<root>/sub/blank-512.img` passed the root, name, length and digest checks at
  once. The resolved path must now equal the exact location that fixture is
  generated at.

- The TypeScript encoder could emit bytes its own decoder rejects, violating
  `schemas/canonical-encoding.md` §6.1. `TextEncoder` substitutes U+FFFD for an
  unpaired surrogate rather than failing, so two distinct values encoded
  identically and `encode` was not injective; a map holding both keys emitted a
  declared size of two with byte-identical keys, which §3 makes invalid. The
  encoder now refuses an ill-formed string instead of repairing it, and validates
  map keys before sorting so the refusal cannot depend on insertion order. Rust
  needed no change — `String` is validated UTF-8 — which is the point: the two
  implementations had disagreed about what was *encodable*. Reachable without an
  attacker, since NTFS permits unpaired surrogates in volume labels and INV-008
  requires such structures be represented rather than discarded.

- Windows could not pass `cargo fmt --check`. Git for Windows sets
  `core.autocrlf=true` in its system configuration by default, and the
  GitHub-hosted `windows-*` runner images do not override it, so checkout
  produced CRLF working-tree files that `newline_style = "Unix"` rejects.
  `.gitattributes` now pins LF in every working tree.


- WP-010 increment 4: `cargo-fuzz` targets for the canonical codec (Section
  11.4), plus `crates/domain/tests/canonicality.rs`, which asserts the same
  canonicality property on stable over every single-bit flip, truncation, and
  boundary substitution of every known-good encoding, and every one- and
  two-byte input exhaustively. `cargo xtask fuzz` runs the smoke pass and gates
  CI as its own job. `fuzz/` is excluded from the workspace and is the only
  place a nightly toolchain is permitted; it is pinned by exact date.
