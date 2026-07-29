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
