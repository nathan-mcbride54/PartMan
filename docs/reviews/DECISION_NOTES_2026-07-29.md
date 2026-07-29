# Decision notes — 2026-07-29, second audit round

For the next agent. This covers the round that answered
`PROJECT_AUDIT_FOLLOW_UP_2026-07-29.md`, and it is about **decisions and their
reasoning** rather than a list of what changed — the CHANGELOG and the pull
requests already carry that.

Read `PROJECT_AUDIT_FOLLOW_UP_2026-07-29.md` first, then this. Two of its six
findings were answered by choosing something other than what it recommended, and
you should know why before you either build on those choices or reverse them.

Merged this round: #43 (foundation gates), #44 (WP-020 no-follow open), #45
(ADR-0007, SI-36, `verify-ownership`).

---

## 1. I did not take the auditor's YAML-parser recommendation

**Their position**, stated twice and with justification: replace the
line-oriented action scanner with a structural YAML parser. *"The concern about
putting a YAML dependency inside the dependency gate does not outweigh the
correctness problem now demonstrated twice."*

**What I did instead.** Discovery no longer depends on recognising the `uses`
key at all. An action reference must contain `owner/repo@ref` verbatim — no
anchor, tag, quoting style or flow mapping changes the reference *text* — so a
sweep for that shape finds every reference, and anything the key-shaped reader
cannot attribute to a `uses:` key is a violation.

**Why.** The audits' underlying demand was *unbypassable discovery*, and the
sweep delivers that by a property rather than by enumerating spellings. A parser
would also deliver it, at the cost of a YAML dependency inside the tool that
gates dependencies. I judged the property cheaper than the dependency. That is a
judgement, not a proof.

**What would make me wrong**, and what to watch for:

- The sweep **over-refuses by design.** A reference-shaped token inside a
  `run:` script is reported, and the fix is to rewrite the step in block style.
  Today the workflows contain exactly the seven real references and nothing else
  shaped like one — verified before writing it. If a legitimate workflow needs
  such a token and cannot be rewritten, the sweep becomes an obstacle rather
  than a gate, and the parser is the documented reopen path
  (`docs/quality/dependency-policy.md`).
- The **key-shaped reader is still a partial YAML implementation**, and I have
  not made it smaller. It now only has to extract the release-tag comment, since
  the sweep owns discovery. If you find yourself extending it again, that is the
  signal the parser decision should be revisited — not another spelling added.

**If you disagree, reverse it.** It is one function
(`reference_shaped_tokens`) plus one loop in `verify_action_pins`, and the
regression tests (`an_action_reference_cannot_hide_behind_key_syntax`,
`the_reference_sweep_does_not_fire_on_prose_or_comments`) express the property in
a way a parser would satisfy too.

## 2. I reversed myself on adding a dependency, and the auditor was right

Increment 2a's WP-020 note said no-follow flags were skipped because
*"hardcoding `O_NOFOLLOW`'s per-platform values without `libc` is its own defect
factory."* The audit's reply was one sentence: *"Avoiding a dependency is not
itself a safety property."*

That was correct and my reasoning had been backwards — `libc` exists precisely so
that nobody hardcodes those values. Increment 2b takes `O_NOFOLLOW` from `libc`
(Unix only, constants only, no `unsafe`, already in the lockfile via
`sha2 → cpufeatures`).

Note the asymmetry with decision 1, because it is deliberate rather than
inconsistent: I accepted a dependency where it was the *only* way to get a
correct answer, and declined one where a property-based check got the same
answer without it. If you think that line is in the wrong place, the two cases
are the evidence.

`FILE_FLAG_OPEN_REPARSE_POINT` and `FILE_SHARE_READ` **are** written out as
constants, with the reasoning in place: std exposes `share_mode` and
`custom_flags` without exposing the values, and pulling `windows-sys` in for two
integers costs more than it saves. Both are fixed Win32 contract. That is a
smaller version of the same judgement and equally reversible.

## 3. Two preconditions closed by deciding, not by coding

This is the part most likely to look like under-delivery, so here is the
reasoning in full.

**The token (ADR-0007).** WP-020 had carried "decide a genuinely independent
token factor" since increment 1, and I had told the user it "needs your call on
adding `getrandom`." That framing was wrong. `authorize` deliberately trusts
nothing inside the directory it verifies — expectations come from the compiled
catalogue, because accepting a caller-supplied manifest was a defect that let a
hand-written one authorize an arbitrary target. A per-generation random token
**cannot be compiled in**, so the interlock would have to read it from the
fixture root, re-creating exactly that trust dependency. An actor who can write
the fixture root could write the token they intend to present.

So randomness would have added a dependency **and** a writable-file trust while
defeating nobody. Read exactly, SAFE-007 requires the three factors to be
*present* and forbids one environment variable standing in for all of them; both
hold, and it never required independence. The token is an operator-intent proof.
A real third factor needs state outside both the source tree and the fixture
root — a T2/T3 lab-architecture question, recorded as the ADR's revisit
condition.

**The Windows link count (SI-36).** I probed the toolchain instead of assuming:
`MetadataExt::number_of_links` is unstable behind `windows_by_handle`
(rust-lang/rust#63010) on the pinned 1.96.0. The only route is FFI, and
SAFE-009's two lists — forbidden in "domain, planner, validator, journal, rpc",
permitted in "adapter, FFI, and helper crates" — name `crates/fixtures` in
**neither**. That is an enumeration, not a rule, so §0.2 says file it. Three
options with their costs are in `docs/spec-issues/README.md` and none is proposed
as the answer. Option 3 is in force **because it is what the code does, not
because it was chosen** — do not read the status quo as a decision.

## 4. `verify-ownership`: what it does and does not decide

Section 1.10 claims CI enforces path ownership via CODEOWNERS. It cannot.
Every work package now declares machine-readable `owned-paths`, and the block
**is** the prose a reviewer reads, so the two cannot drift.

Three choices worth knowing:

- **Overlaps are reported, not forbidden.** `tools/xtask/**` is genuinely shared
  by three packages. Forbidding overlap would have pushed the sharing back into
  prose where nothing can see it, which is the failure this check exists to end.
- **Only exact paths and `directory/**` are understood.** Anything else is an
  error, not a pattern that quietly matches nothing — the failure mode the action
  scanner was audited for twice. Prefix matching is segment-exact, so
  `crates/tokens/**` cannot annex `crates/tokens-extra/`.
- **Sub-file grants are not expressible.** WP-030 owns "its own status rows" in
  `README.md`; a path checker sees the whole file. That stays a review
  obligation and is stated as one.

Writing the declarations found real drift on its own: WP-010 never claimed
`packages/canonical/` or `fuzz/` despite delivering them in increments 2 and 4,
and claimed `schemas/` wholesale while `design-tokens.json` there belongs to
WP-030.

**Not done:** deciding whether a given change came from the package owning the
path. That needs a pull-request-to-package mapping this repository does not
carry, and it is a process decision — where does that mapping live, and who
maintains it — rather than code. It is issue #39's open half.

## 5. The process lesson, now twice-confirmed

In both audit rounds, **my fix repeated the defect it was written to end**, and
in both cases the only thing that caught it was planting a regression:

- Round one: the token harness took its WCAG floors from the file it audited —
  the exact self-consistency failure `AGENTS.md` records for the canonical
  vectors, committed inside the harness written to enforce that rule.
- Round two: increment 2a's handle `fstat` could be downgraded to a path `stat`
  with every test still green, because the difference only shows during a race a
  unit test cannot stage. The structural fix was a function that cannot see a
  path (`verify_object`), plus a test that deletes the path before verifying.

The rule that works: **do not trust a passing test suite to tell you a check
exists.** Delete the check and watch a named test go red. Every gate added this
round was swept that way, and the sweeps are recorded in the pull requests.

The corollary for a test that cannot be written deterministically: build a seam.
The pre-open race is scheduled, not sampled, because a `#[cfg(test)]` hook fires
between canonicalization and open. Timing-dependent tests of security properties
are worse than none, because they pass.

## 6. Where I would attack this round's work

Calibration first: last round's equivalent list flagged two things that turned
out genuinely broken, and **missed the most serious defect entirely** (the
pre-open race). Treat this list as a starting point, not a perimeter.

1. **`reference_shaped_tokens`'s tokenizer.** It splits on a character class and
   trims `:` and `.`. `docker://` references and future reference shapes are the
   interesting inputs. Over-refusal is safe; the failure I would hunt is a
   reference shape it *fails* to see — which would restore the exact hole
   decision 1 claims to have closed.
2. **`verify-licenses` now shells out to `cargo metadata` twice.** It is
   authoritative but it is also a second process whose failure I map to a
   `Policy` error. Check the behaviour when a workspace member is present but
   unbuildable, and when `--locked` refuses — the licence gate should not be the
   thing that reports a lockfile problem in a confusing way.
3. **The `verify_fuzz_lock` ordering test asserts on source text.** It reads
   `main.rs` and compares byte offsets of `verify_fuzz_lock()` against
   `"deny"`. It works and it is honest about why (no runtime assertion inside the
   process can see the ordering), but it is brittle to refactoring and would pass
   if both moved into a helper. A better mechanism would be welcome.
4. **Windows share-mode assumptions.** The replace-after-authorization test
   asserts rename/delete/second-write all fail while an authorization is held.
   Verified on local Windows and on the runner; ReFS and network shares are
   untested. The fixture tree is always local temp, which is why I accepted it.
5. **`docs/quality/test-tiers.md` is now shared between WP-020 and WP-030** in
   the ownership blocks, with each owning a different part of one file. That is
   exactly the sub-file grant the checker cannot express, in the document that
   describes the safety tiers. If sub-file ownership is ever mechanised, start
   here.

## 7. State, one line each

- **WP-000** — in progress. Gates remediated; generated traceability and
  PR-to-package mapping open (issue #39). Runner-image digest deviation
  documented.
- **WP-010** — unchanged, correctly blocked at increment 3 on SI-31 and friends.
- **WP-020** — increments 1–1f, 2a, 2b delivered. Precondition 1 closed,
  2 closed by ADR-0007, 3 filed as SI-36 with a narrow recorded residual.
  Increment 2 is no longer gated on a precondition; **Tier 2 still refuses,
  correctly, because no destructive suite exists.**
- **WP-030** — increments 1, 1a, 1b delivered. Increment 2's paths are now
  reserved, so the shell is authorized to begin; it must consume a generated
  typed accessor rather than a copied palette.
- **M0** — not met. **Product** — still not a partition manager, and the README
  still says so.

The next substantive step is WP-030 increment 2, and it is the first work this
round has actually *unblocked* rather than repaired.
