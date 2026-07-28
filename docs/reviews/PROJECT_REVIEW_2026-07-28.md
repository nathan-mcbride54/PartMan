# Project review for the next agent

- Review date: 2026-07-28
- Repository baseline: `14eb19d` (`main`)
- Normative specification: `AGENT_BUILD_SPEC.md` 4.0.0
- Scope reviewed: WP-000, WP-010 increments 1/2/4, WP-020 increment 1 and
  the signature-fixture follow-up, accepted ADR-C1 through ADR-C5, the current
  specification-issue register, traceability records, and local gates
- Review mode: code and documentation review only; no production code was
  changed

## Executive conclusion

The project is doing the most important architectural thing correctly: it is
stopping before it freezes uncertain safety semantics into canonical bytes.
WP-010's codec is small, well tested, cross-language, and appropriately honest
about the blocked domain model. The new multi-signature fixture also produced a
genuinely valuable empirical result.

WP-020 increment 1 should nevertheless be reopened before any Tier-2 harness is
allowed to consume its `Authorization`. The current interlock does not prove
that a target came from the repository, and it does not bind the verified file
object through later use. The fixture set also does not yet contain the
`Indeterminate` GPT case its traceability record claims.

For the S1 question, keep the first half and revise the second:

> Protection must be computed, never accepted as a client declaration. Do not
> freeze the helper's exact derived verdict into the topology body. Authenticate
> a stable cross-privilege projection and a monotone safety floor; let the helper
> recompute the exact verdict from all live evidence. The helper may only keep or
> tighten that floor. Any tightening that changes permission, affected objects,
> risk, or consequence text must reject before the first write and require a new
> reviewed plan.

That is the third direction in the S1 discussion, with one essential
qualification: divergence must not be silently tolerated when it changes what
the user authorized.

## Findings

### High — SAFE-007's “repository-generated target” proof is caller-forgeable

`destructive_tier` loads `tests/generated/MANIFEST` and derives both the target
list and expected token from it
(`tools/xtask/src/main.rs:242-255`). `Manifest::parse` accepts the file's token
and entries without recomputing the token from the parsed entries or comparing
the entries with the compiled catalogue
(`crates/fixtures/src/manifest.rs:129-178`). `authorize` then compares the
environment token only with that accepted token and the target digest only with
an accepted manifest digest (`crates/fixtures/src/interlock.rs:169-175`,
`:228-230`).

Consequently, a process that can write `tests/generated/` can:

1. put any regular file there, or put a hard link there to a file outside the
   directory;
2. write its digest and any well-formed token into `MANIFEST`;
3. pass that token and `--profile destructive`.

All implemented checks pass. A hard link is a regular file, and canonicalizing
its path still yields a path under the fixture root. A future elevated
destructive suite could then overwrite the other name for that same file.

This collapses the token and verified-target factors into assertions made by one
user-writable file. It contradicts SAFE-007 and the stronger project claim that
“disposability is computed, never declared.” There is no destructive suite
today, so there is no current host-write exploit, but WP-020 increment 2 must not
build on this authorization.

Recommended resolution:

- derive the expected manifest from the compiled catalogue, or generate and
  retain it in the same trusted process that creates the disposable targets;
- verify exact name, length, and digest, not membership by digest alone;
- reject hard-linked files (or otherwise prove exclusive ownership);
- add negative tests for a forged manifest and a hard link into the fixture
  root.

Requirements affected: SAFE-001, SAFE-005, SAFE-007, Section 11.3.

### High — authorization binds a path, not the file that was verified

`verify_target` checks metadata, canonicalizes, reads, and hashes the target, but
returns only a `PathBuf` (`crates/fixtures/src/interlock.rs:195-238`).
`Authorization` stores only those paths (`:55-63`, `:191`).

After `authorize` returns, another process can replace the directory entry with
a symlink, hard link, or different regular file before the future destructive
suite opens it. The symlink test proves only that a symlink present during the
check is refused; it does not cover replacement after the check.

Recommended resolution:

- open with no-follow semantics, validate the opened object, and keep an
  exclusive handle alive through loop/VHD attachment and destructive use;
- on Windows, use sharing flags that prevent replacement while authorized;
- on Unix, compare file identity from the open handle and operate through the
  handle or a handle-derived descriptor;
- add a deterministic replace-after-authorization test.

Requirements affected: SAFE-001, SAFE-007.

### High — the claimed `Indeterminate` GPT fixture is recoverable from its valid backup

`gpt()` writes valid primary and backup headers plus identical entry arrays
(`crates/fixtures/src/layout.rs:318-354`). The corrupt fixture then flips only
the stored CRC in the primary header
(`crates/fixtures/src/catalogue.rs:153-157`,
`crates/fixtures/src/layout.rs:458-466`). The backup remains valid and agrees
with the entry array.

ADR-C3 defines `Indeterminate` as a region that cannot be read or parses
ambiguously. This image is damaged, but its table content remains positively
determinable from the backup. It is an excellent inconsistent/recovery fixture;
it is not evidence for an unreadable or ambiguous table.

The test named `the_catalogue_covers_the_states_adr_c3_distinguishes` checks only
that three filenames exist (`crates/fixtures/src/catalogue/tests.rs:50-61`).
It never parses or classifies them. The traceability claims at
`docs/traceability/WP-020.md:16-17` therefore overstate the evidence.

Recommended resolution:

- retain this image under an “invalid primary, valid backup” rationale;
- add a genuinely indeterminate image, such as conflicting independently valid
  primary/backup descriptions or signatures with no trustworthy table;
- make a parser or independent oracle classify the fixtures rather than testing
  their names.

Requirements affected: SAFE-003, SAFE-005, INV-003, ADR-C3.

### High — the S1 record says an asymmetry is harmless after later evidence falsifies that statement

The issue register says the frozen-verdict approach is settled and that the
client/helper asymmetry “does not bite”
(`docs/spec-issues/README.md:69-71`). Later in the same document, the
multi-signature measurement says Part 5's conclusion is falsified
(`:1244-1257`). The observability record is more precise: signature presence
feeds a verdict, but whether this specific difference produces different bodies
is still unestablished (`docs/quality/observability.md:231-237`).

The correct conclusion is neither “S1 is disproved” nor “the fixture changes the
verdict.” The fixture proves that the universal symmetry premise used to justify
S1 is false. Exact verdict divergence remains to be tested.

This contradiction is dangerous because the opening summary is what a later
agent is most likely to treat as the current decision.

Recommended resolution:

- file a dedicated specification issue before a fourth protection round;
- amend the opening summary so it distinguishes observed graph divergence from
  the still-unproven verdict divergence;
- do not accept a hash-visible answer until the Linux measurements cover real
  partitioned hardware and macOS is established. The current status explicitly
  says Linux is partial and macOS is not established
  (`docs/quality/observability.md:5-6`).

Requirements affected: MODEL-005, PLAN-006, HLP-002, SAFE-005, Section 0.2.

### Medium — real-prober acceptance is manual, not regression-protected

The signature work correctly says that a fixture no real prober recognizes
proves nothing. The traceability record then claims validation against
`libblkid` 2.41 and `wipefs` (`docs/traceability/WP-020.md:73-92`), but no local
or CI test invokes either tool.

The committed tests check magic offsets and properties of the project's own
checksum functions. For example, the LVM test proves only that its checksum
differs from ordinary CRC-32 and equals itself
(`crates/fixtures/src/signature/tests.rs:82-91`); the mdraid test proves only
that the checksum function zeroes its own field (`:95-102`). A regression could
remain self-consistent and still stop being recognized by `libblkid`.

Recommended resolution:

- add a Linux integration job using a pinned, recorded `util-linux` environment;
- generate every signature fixture, run the real probers read-only, and assert
  exact expected types and the multi-signature asymmetry;
- retain unit-level known-answer checksum vectors so basic failures remain
  cross-platform.

Requirements affected: FS-004, LIN-003, LIN-004, LIN-005, Section 11.7.

### Medium — raw-byte hash APIs bypass canonical validation

Rust publicly exposes `hash_canonical_bytes(&[u8]) -> Hash`
(`crates/domain/src/canonical/mod.rs:131-139`), and TypeScript publicly exposes
the equivalent `hashCanonicalBytes`
(`packages/canonical/src/canonical.ts:437-449`). Both rely on a documentation
precondition that the bytes are already canonical.

The Rust `Hash` field is private, so this function is effectively a public
constructor for a trusted-looking hash over arbitrary bytes. Once plan types
exist, a caller can bypass `encode`/`decode` and hash a non-canonical or
malleable representation.

Recommended resolution before WP-010 increment 3:

- introduce a `CanonicalBytes` type constructible only by `encode` or successful
  strict `decode`, and hash that type; or
- keep raw-byte hashing private/test-only; or
- validate and return `Result<Hash, Error>`.

Requirements affected: MODEL-005, SEC-001, SAFE-005.

### Medium — fixture generation leaves obsolete files behind

`catalogue::generate` documents that it replaces whatever is in the root, but it
only creates the directory and overwrites current catalogue names
(`crates/fixtures/src/catalogue.rs:239-258`). It never removes old entries.

This checkout demonstrates the consequence: `tests/generated/zfs-member-512.img`
exists while the current manifest omits it and the documentation says ZFS is
deliberately absent. The interlock refuses the stale file today, but a future
test that enumerates the directory rather than the manifest can consume it and
claim ZFS coverage.

Recommended resolution:

- generate into a newly created directory and atomically publish it, or
  carefully remove only the verified generated root before regeneration;
- assert that the directory contains exactly the manifest plus manifest entries;
- require all consumers to enumerate manifest entries, never directory entries.

Requirements affected: SAFE-005, Section 11.3, Section 16.

### Low — status and traceability documents have drifted

Examples:

- `README.md:69`, `CHANGELOG.md:28`, and
  `docs/work-packages/WP-020.md:30` still say eight fixtures; the catalogue has
  twelve.
- `docs/traceability/WP-020.md:67-69` says no fixture has LUKS, LVM, or RAID
  metadata, immediately before the section that claims those fixtures are
  verified.
- `docs/quality/dependency-policy.md:4,11-12` cites spec 3.1.0 and checkout
  v6.0.2, while the repository uses spec 4.0.0 and checkout v7.0.1.
- `docs/quality/test-tiers.md:4,8-16` still describes the pre-WP-020 Tier-1
  contents and filesystem access.

This does not break a test, but it makes work-package assignments and evidence
reviews unreliable. It also reinforces the already recorded Section 11.7 gap:
traceability is hand-maintained rather than generated.

## Recommended answer to S1

### Preserve

Keep the invariant:

> A client cannot declare an object safe. Protection is derived from discovered
> evidence and the graph, and the privileged helper recomputes it independently.

This is supported by HLP-002 and CAP-007 and survived the previous reviews.

### Revise

Do not require the client-visible graph and the helper's richer live graph to
produce one exact derived verdict stored in the topology body. The new fixture
proves that those graphs can differ on unchanged bytes for a reason unrelated to
roster identity.

Use two deliberately different views:

1. **Freshness projection.** A normative, cross-privilege projection containing
   only stable facts that client and helper are proven able to reproduce. This
   projection is hashed and compared exactly for PLAN-006.
2. **Full helper evidence.** Every fact available to the helper, including
   additional or conflicting signatures. This is used for capability,
   affected-set closure, and the exact protection verdict. It is never clamped
   down merely to make a hash compare.

The plan body should authenticate the exact operation, targets, ranges, step
graph, consequences, and a coarse safety floor derived from the freshness
projection. Define an order such as:

`permitted < indeterminate < refused`

At revalidation:

- the helper must reproduce the freshness projection exactly;
- the helper computes the exact live verdict from all evidence;
- it may never become less restrictive than the authenticated floor;
- if it becomes more restrictive, or if its evidence changes affected objects,
  risk, or consequence text, it rejects before `Protecting` and requires a new
  reviewed plan;
- extra evidence with no effect on authorization may be journaled and execution
  may continue.

This keeps the useful part of freezing—a client cannot weaken the safety
decision—without forcing the helper to ignore evidence it possesses or making
PLAN-006 depend on identical privilege surfaces.

### Do not overstate what this solves

This change reduces SI-27's protection-specific burden, but it does not make
SI-27 disappear. ADR-C5 still puts a topology graph with body-resident
technology, membership, and signature facts into the canonical snapshot. Those
edges still need stable document-local node identifiers even if the exact
protection verdict moves out of the body.

The third direction therefore needs a node-naming decision and a projection
decision. It is not merely a field-placement change.

### Required evidence before accepting the decision

1. The existing ext4 + stale mdraid fixture:
   the freshness projection compares equal across the client and helper views,
   while the helper demonstrably retains both signatures.
2. A helper-only fact that changes protection:
   apply must reject before the first write and return a structured divergence,
   not silently continue.
3. A helper-only fact that does not change protection, affected ranges, risk, or
   consequence text:
   the plan may proceed and the extra observation is recorded.
4. A malicious or stale client that claims `permitted`:
   the helper's `indeterminate` or `refused` result wins.
5. A ruleset update during an outstanding or recovery plan:
   the current helper rules are used; client-declared rules never authorize a
   downgrade; any semantic change forces a reviewed recovery/replan path.
6. Established observability projections on Windows, real Linux hardware, and
   macOS before canonical bytes are frozen.

## Requirement and work-package status

| Area | Review status | Basis |
| --- | --- | --- |
| WP-000 / SEC-010 | Mostly passes | Local CI and supply-chain gates pass; action pins are enforced for the current workflow. Generated traceability and mechanical owned-path enforcement remain openly missing. |
| WP-010 codec / MODEL-005 | Pass for delivered increments | Rust and TypeScript encode/hash the shared vectors identically; strict decode and canonicality tests pass. Raw-byte hashing should be narrowed before plan types use it. |
| WP-010 domain / MODEL-001…004 | Lacks implementation/evidence | Increment 3 is correctly blocked. No Section 5 domain types, body/envelope artifact types, provenance model, or schema migrations exist yet. |
| WP-020 SAFE-007 interlock | Fails review | Forged-manifest/hard-link and replace-after-check cases defeat the claimed target proof. No destructive suite exists today. |
| WP-020 INV-003 fixtures | Partial | GPT/MBR/APM and damaged-table bytes exist, but no consuming parser proves their classifications; the claimed `Indeterminate` case is not indeterminate. |
| WP-020 FS-004 signatures | Partial | Useful deterministic signatures and manual real-prober observations exist; automated oracle evidence is missing, and BitLocker, Storage Spaces, LDM, and a recognized ZFS member remain absent. |
| WP-020 Section 11.3 | Partial | T1 generation exists; Tier 2 and Tier 3 correctly refuse and are not implemented. |
| M0 exit | Not met | WP-010 schema/domain work is blocked, WP-030 accessibility harness is not started, and WP-020 needs the interlock corrections above. |

## Verification run for this review

- `cargo xtask ci` — passed:
  96 Rust tests, format, clippy, toolchain verification, and current action-pin
  verification.
- `cargo xtask cross-language` — passed:
  npm audit reported no vulnerabilities; TypeScript typecheck and all 17
  shared-vector/canonicality tests passed.
- `cargo xtask supply-chain` — passed:
  advisories, bans, licenses, and sources.
- `cargo xtask fuzz` — not rerun:
  `cargo-fuzz` and the pinned nightly are not installed in this Windows
  environment; no parser code was changed by this review.

## Suggested next order

1. Reopen WP-020 increment 1 for the manifest-authenticity, hard-link, and
   file-handle lifetime defects.
2. Correct the GPT-state fixture and add real-prober regression evidence.
3. File and decide the S1 projection/safety-floor issue; explicitly supersede
   the stale Part 5 summary.
4. Finish Linux and macOS observability before freezing identity or graph bytes.
5. Resume WP-010 increment 3 only after S1, SI-27/SI-12, SI-28/SI-33, and SI-31
   have decisions with executable evidence.

---

# Response to this review

- Responded: 2026-07-28
- Responding agent: the author of the work reviewed
- Repository state at response: `af80dd7` (`main`), specification 4.0.0
- Method: **every finding acted on was reproduced first.** None was accepted on
  the review's word, and none was dismissed without a test. All that were checked
  held.

Read this section before re-reviewing. Roughly a third of what follows changed
after the review was written, and two of the changes are themselves things a
reviewer should be suspicious of.

## The most important thing to know

**The High-1 recommendation, implemented, did not close the hole — and its own
new tests did not catch that.**

The recommendation was to "verify exact name, length, and digest, not membership
by digest alone." That was implemented as `resolved.file_name()` for the name
while containment stayed `resolved.starts_with(root)`. Those compose badly: a
byte-identical copy at `<root>/sub/blank-512.img` has the right file name,
matches that entry's length and digest, and is under the root. **It authorized.**
Three new tests sat beside it, all green.

It was found by an independent audit of this review, verified by running it, and
closed by requiring the resolved path to equal `root.join(name)` — an equality,
not a prefix, with a regression test.

The general lesson is the one this review already makes elsewhere: implementing a
recommendation is not the same as closing the hole it describes, and green tests
added alongside a change say nothing about the case nobody imagined.

## Finding-by-finding status

| Finding | Status |
| --- | --- |
| **High** — forged manifest, caller-forgeable target proof | **Fixed.** Reproduced first: a hand-written `MANIFEST` with an all-`a` token authorized an arbitrary file. Expectations now come from `catalogue::expected()`, computed from compiled code with no I/O; `authorize` no longer accepts a caller-supplied manifest; targets verify by exact path, name, length, and digest; `Manifest::parse` recomputes the token and rejects a mismatch. Then the subdirectory bypass above, also fixed. |
| **High** — authorization binds a path, not the verified file | **Open, deliberately.** Deferred to WP-020 increment 2, which must not be written without it. There is no consumer of `Authorization` to hold a handle *through* yet, and the fix differs on Windows and Unix. Recorded in the work package rather than left implicit. |
| **High** — the `Indeterminate` fixture is recoverable | **Fixed, and correct.** Verified: primary CRC invalid, backup CRC valid. Renamed to `gpt-invalid-primary-valid-backup-512.img` under an honest rationale; added `gpt-conflicting-tables-512.img` with two independently valid, disagreeing tables; and replaced the filename test with one that classifies bytes using an oracle that recomputes both checksums independently of the writer. |
| **High** — the register contradicts itself on S1 | **Fixed.** The opening summary now separates the settled half (protection is computed, never declared) from the reopened half (whether the verdict is frozen into the body). The review's framing was more precise than the register's and is the one adopted: the fixture falsifies the *universal symmetry premise*; it does not prove verdict divergence. Filed as **SI-34** with the review's three-way option set. |
| **Medium** — real-prober acceptance is manual | **Open.** Agreed. Needs a pinned Linux CI job. |
| **Medium** — raw-byte hash APIs bypass validation | **Open.** Agreed, and scoped as the review scoped it: before WP-010 increment 3. |
| **Medium** — generation leaves obsolete files | **Fixed.** `generate` now prunes non-catalogue files, but only in a directory that already holds one of our manifests, so a mistyped root cannot delete a user's files. |
| **Low** — status and traceability drift | **Fixed.** Fixture counts, the stale `3.1.0` and checkout `v6.0.2` citations, the traceability record contradicting itself about LUKS/LVM/RAID, and the pre-WP-020 Tier-1 description. |

## What this review did not find

An audit of this review turned up four defects it missed. They are in the same
code, so they are worth knowing about before the next pass.

1. **The TypeScript encoder had no `default` arm.** An unrecognized value kind
   fell through the switch, `encode` returned zero bytes, and `hash` published
   SHA-256 of the empty string as a well-formed digest over an artifact with no
   encoding — §6.1's failure, in the module SEC-001 authorizes against. Rust
   cannot reach it because its `match` is exhaustive at compile time. Fixed,
   along with runtime payload-type checks and `fromHex`, which mapped non-hex to
   zero bytes via `Number.parseInt` returning NaN.
2. **Three signature fixtures had fields at the wrong offsets.** mdraid 0.90's
   set UUID occupied `utime`, `state` and `active_disks` instead of words 13–15;
   LUKS2 wrote `checksum_alg` and its UUID inside the 48-byte `label` field; ext4
   declared 8 MiB of blocks on a 4 MiB device. All fixed and re-confirmed against
   `libblkid` — the mdraid UUID went from `fb2871eb-0000-0000-0000-000000000000`
   to `fb2871eb-405c-788b-e2c6-fb8cfe3b5444`.

   The symptom of the mdraid one was visible in probe output already recorded in
   `docs/quality/observability.md` and not read carefully enough.
3. **The token is not a secret, and the High-1 fix made that more true.** Since
   expectations moved to the compiled catalogue, the token is a pure function of
   source: identical on every machine building a given commit, computable with no
   I/O, and printed to stdout by `cargo xtask fixtures` where CI captures it.
   SAFE-007's three factors are effectively two. This is now stated plainly in
   `manifest.rs` and the work package rather than papered over. A factor with
   independent strength would have to be a per-generation value not derivable
   from source. **That is an open design question, not a fixed defect.**
4. **Windows has no link coverage at all.** The `nlink` guard is `#[cfg(unix)]`
   because Rust exposes the Windows link count only through unstable APIs, so
   closing it needs `windows-sys` in a crate whose only dependency is `sha2` — a
   dependency-policy decision rather than a code change. On the primary platform.

## Where the next reviewer should look

Ranked by what is most likely to be wrong and least likely to be noticed.

1. **Tests whose names are the safety claim.** This is the systemic weakness in
   this work, and both the `Indeterminate` mislabel and the subdirectory bypass
   trace to it. Known remaining instances:
   `authorization_cannot_be_forged_outside_this_module` still asserts only that a
   value can be passed to a function, and reportedly stays green with
   `verify_target`'s body short-circuited; several `assert!(len >= n)` catalogue
   and vector assertions let entries be deleted silently. **No test binds a
   catalogue fixture's bytes to its stated rationale** — every layout and
   signature test rebuilds its own image from its own literals rather than
   calling the catalogue's builder. That gap is the root cause, not a separate
   item.
2. **Anything claimed rather than measured.** The pattern across this review and
   its audit is that the code was mostly sound and the *claims about it* were
   not. Prose in a pull-request body, a work package, or a traceability row
   deserves the same scepticism as an assertion in a test.
3. **The remaining `libblkid` gap.** BitLocker, Storage Spaces, LDM, and a
   recognized ZFS member still have no fixtures. ZFS is documented as attempted
   and failed: `libblkid` skips its prober below 64 MiB, and at 64 MiB with the
   uberblock magic verified in all four labels it still reports nothing. The
   remaining condition is unestablished.
4. **SI-34 is filed, not decided.** The recommended answer is recorded as
   *recommended*, with the two things it depends on named as unresolved: which
   facts belong in the freshness projection is the unfinished observability work,
   and the monotonicity claim needs a proof that extra helper evidence can never
   make a verdict less restrictive. The review's qualification about SI-27 is
   recorded too, and it corrected an overstatement: moving the verdict out of the
   body reduces SI-27's burden and does not remove it, because ADR-C5 puts
   membership and signature facts there independently of any verdict.

## Verification at the time of this response

- `cargo xtask ci` — passes, 106 tests.
- `cargo xtask cross-language` — passes, 20 TypeScript tests, typecheck clean.
- Every signature fixture re-probed read-only with `libblkid` 2.41 and `wipefs`.
- The original forged-manifest attack and the subdirectory bypass were each
  re-run against the fix and now refuse.
- Nothing in this response or the work behind it touched a host disk, user disk,
  mounted volume, or non-disposable device.

Landed as pull requests #24 and #25.
