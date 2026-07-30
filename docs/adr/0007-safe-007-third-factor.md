# ADR-0007: What SAFE-007's disposable-test token actually proves

- Status: Accepted, with one claim corrected 2026-07-29 (see "Correction")
- Date: 2026-07-29
- Spec version: 4.0.0
- Work packages blocked: WP-020 increment 2 (precondition 2)
- Requirement IDs: SAFE-007, SAFE-001, SAFE-005
- Decision owners: @nathan-mcbride54

## Correction

This ADR said the token "establishes that whoever requested destruction ran the
generator and copied a value out of its output." The second follow-up audit
pointed out that a pure function of public source cannot establish that history:
anyone with the repository can compute the value without running anything.

What the code proves is narrower and is what the decision below actually rests
on: **the invocation presented the exact build-derived value.** That is accident
friction — a destructive run cannot be produced by ambient environment state
alone — and it satisfies SAFE-007's "all present" clause. It is not evidence of
operator provenance. The decision is unchanged; the justification is corrected,
because "proves the operator ran the generator" is exactly the kind of claim
this project has been audited for three times.

## Context

WP-020 has carried this as an open precondition since increment 1, and both
2026-07-29 audits repeated it: *"Decide a genuinely independent token factor, or
state plainly that SAFE-007 rests on two factors. A per-generation value not
derivable from source is the only thing that would make it three."*

The token is `PARTMAN_DISPOSABLE_TOKEN`, compared in
`crates/fixtures/src/interlock.rs` against the token of the **compiled**
catalogue. It is a pure function of source: identical on every machine that
builds a given commit, and printed to stdout where CI captures it. WP-020's own
table has called it "**Weak, and known to be**" from the day it was written.

The obvious fix — make it random per generation — was queued as "needs an
entropy source, so it is a dependency decision." That framing was wrong, and
finding out why is what this ADR records. The problem is not where to get
randomness. It is that randomness does not go anywhere useful.

## Safety analysis

Nothing here changes what the interlock accepts today. The decision is about
which claim the project is entitled to make, and a wrong claim about a safety
interlock is the failure mode this repository has already been audited for
twice.

The load-bearing observation is about **where a verifier learns the secret**.
`authorize` deliberately trusts nothing inside the directory it is verifying:
expectations come from `catalogue::expected()`, the compiled catalogue, because
accepting a caller-supplied manifest was a defect that let a hand-written one
authorize an arbitrary target. A per-generation random token cannot be compiled
in — by construction, the binary predates the generation. So the interlock would
have to read it from disk, from the fixture root, which is the exact trust
pattern that was removed.

That is not a smaller problem than the one it solves. An attacker who can write
the fixture root could write the token they intend to present. The token would
still be "random", still 64 hex characters, and would prove nothing about the
attacker it was added to stop. Against the *accidental* run it protects no
better than the current value, because both require the operator to have looked
a value up and passed it.

So a genuinely independent third factor requires state in a location neither the
generator's output directory nor the source tree — CI secret storage, or a path
outside the fixture root. That is a decision about where test state lives, and it
buys protection only against an actor who already has write access to the
repository's own build tree and can run `cargo` in it. Under SAFE-001, that actor
can generate their own fixtures and address them legitimately.

## What SAFE-007 requires, read exactly

> The test runner MUST refuse destructive suites unless a disposable-test token,
> a verified image/VM target, and an explicit destructive-test profile are all
> **present**. A single environment variable is not sufficient proof.

It requires three things to be present, and forbids one environment variable
from standing in for all of them. It does not say the three must be
cryptographically independent, and the sentence it does add — the one about a
single environment variable — is satisfied precisely: the profile is a
command-line argument that cannot be inherited from a parent shell, the target
is verified from its own bytes against compiled expectations, and the token is
the only environment variable of the three.

The heading is **"Host protection in CI."** The threat is a destructive suite
running against something it should not, by accident or by ambient state. All
three factors address that. The "three independent factors" framing that WP-020
and the progress report used is the project's own stronger reading, and holding
the code to a standard the specification does not set produced a precondition
that could not be closed by any amount of work.

## Options considered

### Option A — per-generation random token, read from the manifest

Rejected. It moves the verifier's source of truth from compiled code into the
directory under verification, re-creating the trust dependency that a previous
defect established was wrong. Strictly worse than the status quo: it adds a
dependency and a writable-file trust, and defeats no attacker.

### Option B — per-generation random token, stored outside the fixture root

Rejected for now, not on principle. It is the only option that yields a real
third factor. It requires deciding where privileged-test state lives, which
belongs with the T2/T3 lab architecture (Section 11.2, WP-020 increment 3) and
not with a fixture generator. Filed as the revisit condition below rather than
guessed at.

### Option C — state what the token proves, and stop calling it a third
independent factor

Accepted. The token is an **operator-intent proof**: it establishes that whoever
requested destruction ran the generator and copied a value out of its output.
That is a real property, it is the property SAFE-007's "all present" clause asks
for, and it is what the code already delivers.

### Option D — remove the token as pointless

Rejected. It would breach SAFE-007's "all present" literally, and it would
remove the one factor that makes a destructive invocation impossible to produce
by ambient environment alone — which is the accident SAFE-007 is named for.

## Decision

**SAFE-007 is satisfied by two independently-derived factors plus one
operator-intent proof, and that is what the project claims.**

- The **profile** is a command-line argument. Independent of the environment.
- The **verified target** is computed from the target's own bytes against the
  compiled catalogue, through a no-follow open that cannot leave the fixture
  root (increment 2b). This is where the interlock's strength rests.
- The **token** proves operator intent. It is a pure function of source, it is
  documented as such in `docs/work-packages/WP-020.md` and
  `docs/quality/test-tiers.md`, and it is **not** an independent factor.

WP-020 precondition 2 is closed by this decision rather than by code. No
`getrandom` dependency is added.

## Consequences

Positive:

- A precondition that no implementation could close stops blocking increment 2.
- The claim in the documents and the property in the code now match, which is
  the whole point of the exercise.
- No dependency added, and no new trust placed in a writable file.

Negative and accepted:

- An actor with write access to the fixture root and the ability to run `cargo`
  is not stopped by the token. Recorded plainly; under SAFE-001 that actor can
  generate fixtures and address them legitimately anyway.
- Anyone reading "disposable-test token" and expecting a secret will be wrong
  until they read this ADR or the WP-020 table. Both now say so in the first
  sentence.

## Verification

No new automated evidence: this decision changes no behaviour. The existing
tests continue to establish that the token is *required* and that an
approximate match is refused (`a_wrong_token_is_refused`,
`no_single_factor_is_sufficient`, `two_of_three_factors_are_still_refused`).

What would falsify the decision is the documents drifting back toward calling
the token independent. `docs/work-packages/WP-020.md` and
`docs/quality/test-tiers.md` both carry the weak-factor statement, and this ADR
is linked from both.

## Revisit conditions

- **WP-020 increment 3 / Section 11.2 decides where privileged-test state
  lives.** If that produces a location outside both the source tree and the
  fixture root, Option B becomes cheap and should be taken: a real third factor
  for the cost of writing a nonce to a path the lab already owns.
- **A T2 or T3 harness gains an unattended trigger.** Everything above rests on
  a human or a CI job having copied a value out of generator output. An
  unattended path removes the operator whose intent the token proves, and the
  factor has to be re-derived.
- **SAFE-007 is amended** to require independence explicitly, at which point
  this ADR is superseded rather than amended.
