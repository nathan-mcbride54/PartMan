# SI-35 axis round — 2026-08-08

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** On acceptance it becomes ADR-0014's input, drafted into
the reserved `docs/adr/0014-si35-table-state-axis.md` for its own
acceptance — the register's discipline runs the round, the decision, and
the instrument separately. Untracked session artifact (`docs/reviews/**`,
WP-000).

**Inputs, all on the record:** SI-35's register entry with its three
options and completed evidence categories; the fork decision accepted
earlier today (ADR-C4's guard is a priced permission under four
conditions, `ADR-0014_FORK_RECOMMENDATION_2026-08-08.md`); ADR-0015's
resolution of the `Absent` face; ADR-0013's privilege scoping; the
measurement campaign, whose two facts drive everything below.

## The two measured facts that collapse the design space

**Fact 1 — no client contract computes the states, and neither do the
helper's *tools*.** The decisive pair (`gpt-basic-512` versus
`gpt-conflicting-tables-512`) is byte-identical to every enumerated client
projection on all three platforms — and the labelled privileged
`blkid -p -o udev` and `wipefs -n` probes returned identical output for
the pair too. What separated it, in M10, was **raw bytes**: identical
head digests, differing tail digests. The separating contract is not "run
a privileged tool"; it is *parse the tables yourself from raw sectors* —
both GPT copies, CRCs validated. Section 11.4 already lists GPT, MBR, and
APM header parsers as planned parser targets with fuzz obligations.

**Fact 2 — ADR-C3's own vocabulary makes `Present` privileged by
definition.** `Present` means "read and hashed": producing it *requires
reading table bytes*, and the unprivileged raw read is denied on all
three platforms (measured on each). `Absent` requires positively
determining nothing is there — spec 7.0.0 just recorded where that is
unreachable. So the client's honest ceiling is not "sometimes Present":
it is **no state at all**. This is not a design preference; it is what
the vocabulary's definitions plus the measurements leave standing.

## Recommendation: the helper is the sole author of table state

Four clauses, each answering one part of the register's question ("which
observer computes which state from which contract, and what it does when
the contract is silent"):

**1. Who computes: only the privileged helper, from its own raw-sector
table parser.** Both GPT copies read and CRC-validated; MBR and APM per
their structures; `blkid`/`wipefs` are measured non-separating and are
not the contract. The parser is a Section 11.4 parser of on-disk bytes:
`unsafe`-free, fuzz target landing with it, refusal demonstration
(SI-35's evidence clause (3)) landing with it too — the register already
records that clause as blocked on implementation, and this round keeps
it so rather than declaring it satisfied by intention.

**2. What the client does: never emits a table state, on any platform.**
Not `Indeterminate`-from-here: ADR-C3's `Indeterminate` is a
determination about the medium ("unreadable or ambiguous"), and the
client has read nothing — reporting it would represent blindness as a
determination, the exact shape ADR-0013 forbade. Client surfaces carry
what WP-035 built: raw interface-labelled observations plus the published
reach declaration. **This dissolves rather than answers the option (a)
tag**: there is nothing to privilege-tag when only one observer class
ever writes the field. **And it resolves the `Present` face SI-39
deliberately left here**: a client cannot make INV-003's forbidden
"reporting a table as consistent" report because it makes no table-state
report at all; the helper's full detection duty stands per ADR-0013's
privileged scope.

**3. Where it lives: in the hashed body, helper-authored at validation —
the guard satisfied, not amended.** The spec's own flow already runs
validate → `AwaitingAuthorization` → `Revalidating` → apply: the helper
validates before the user authorizes, and HLP-003 binds authorization to
the exact post-validation hash. So the bindable artifact — the plan and
the snapshot it binds — gets its table state stamped by the helper's
parser during validation, and the user authorizes a hash that commits to
it. ADR-C4's guard holds with no amendment (`Absent` and `Indeterminate`
produce different body values, authored by the observer that can tell);
ADR-C2's "bound device identities are body" row holds; MODEL-005's
body-stability rule holds (the parser is deterministic over unchanged
bytes); PLAN-006's body-hash freshness comparison compares
helper-authored values against helper-recomputed values — one observer
class, so the two-observers-two-bodies problem never arises. The fork's
priced permission goes **unused** by this design; its price list stands
for any future draft that needs it.

**4. When the contract is silent — client-only artifacts — the field is
absent-because-unproduced, and such artifacts are never plan-bound.**
HLP-002 already treats client discovery output as an untrusted hint;
this clause makes the hint status structural: hashable, bindable
snapshots are helper-produced (at validation, or by an explicit
privileged refresh), and a client-side view is an observation surface,
not an authorization input. PART-001 additionally gains the categorical
helper invariant ADR-0015 shaped — initialize only on the helper's own
fresh, positively determined `Absent` — adopted here as belt-and-braces
even though the guard survives, because it costs one sentence and closes
the route around every representational protection.

**On `gpt-conflicting-tables-512`, the chosen option refuses**: two
independently valid tables describing different partitions is ADR-C3's
"ambiguous", the helper's parser stamps `Indeterminate`, SAFE-005
disables the affected write, and SAFE-003's strength rule keeps the
record Weak. The demonstration lands with the parser.

## The adversarial round

**Attack 1 — "validation now authors body content; ADR-C2 says enforcing
is not re-deriving, and this is a third verb it never defined."**
Sustained as the ADR's sharpest drafting obligation rather than a defect.
The stamp is re-derivation *placed into the artifact before
authorization* — the helper computing the value it would recompute at
apply anyway, with `Revalidating` already in the state machine to catch
drift between stamp and apply. But ADR-0014 must define the verb: the
draft adds the authoring rule to ADR-C2's table ("table state | body |
helper-authored at validation, recomputed at revalidation") rather than
leaving three verbs where two were stated.

**Attack 2 — "client snapshots become second-class, and something may
require a hashed client snapshot."** The round could not refute this from
memory, so it is a named verification obligation on the draft: sweep
MODEL-005, CONC-004, Section 20's snapshot vocabulary, and SI-27's
naming scope for any duty that requires a *client-produced* hashed
snapshot. If one exists, the fallback shape — client snapshots hash
without the table-state field, which is schema-typed as helper-section
content — keeps clause 3's guard satisfaction and clause 2's silence,
at the cost of two snapshot schemas. The recommendation survives either
way; which shape the draft takes waits on that sweep.

**Attack 3 — "you are promising a parser that does not exist and calling
it the contract."** The contract is named, not promised: raw sectors,
both GPT copies, CRC validation — the exact reads M10 performed and
measured as separating. What does not exist is the *implementation*,
and the round keeps every obligation that waits on it (the fuzz target,
the refusal demonstration, evidence clause (3)) recorded as waiting
rather than satisfied. SI-35's own entry already has this shape: "that
category cannot open until an option and an implementation exist."

**Attack 4 — "clause 2 overreaches: a Linux client in the `disk` group
could read raw sectors and compute states honestly."** Out of contract
by the product's own texts: SAFE-002 places discovery at no elevation,
and the enumeration surface "does not widen its reads when run with
privilege it did not need" (spec 6.1.0, shipped as increment 8/9
behavior). A privileged *measurement* is a labelled comparison leg; a
privileged *product client* is the auto-elevation SAFE-002 forbids.

**Attack 5 — "this quietly decides SI-34."** Refuted by scope: SI-34's
protection-verdict placement and freshness projection are untouched;
clause 3 places one field, and the freshness comparison it relies on
(PLAN-006 over body hashes) is the one the spec already mandates.
SI-34's option (c) monotonicity obligation is not inherited, because
there is no client-claimed state for the helper to tighten — the
monotonicity question dissolves with the tag.

## Options not recommended, and why

- **(a) Privilege-tagged state.** Dissolved rather than rejected: with a
  single authoring observer there is nothing to tag. Were a second
  author ever admitted, the tag returns as hash-visible body content —
  the register's recorded cost — which is a reason to keep authorship
  single, not to price the tag now.
- **(b) Clamp to a named client projection.** Dead on the evidence
  clause it set for itself: two decision-complete campaigns produced no
  separating client contract, and "equality in one finite libblkid
  projection neither supplies that contract nor refutes the existence of
  another" cuts both ways — a future named-and-measured interface is a
  revisit condition, not a foundation.
- **(c) as filed — helper-only `Indeterminate`, client claims the
  rest.** Rejected because the client cannot honestly claim the rest:
  `Present` requires reading and hashing table bytes the client is
  denied. The recommendation is (c)'s direction carried to where the
  measurements and ADR-C3's definitions actually point — helper-only
  *everything*, with the monotonicity obligation dissolving as a bonus.
- **Out-of-body (the fork's priced route).** Not taken: clause 3 keeps
  every property the guard protects at no amendment cost. Priced and
  available if attack 2's sweep forces a shape the body cannot carry.

## If accepted, the mechanics

ADR-0014 is drafted into its reserved path under WP-010, in the ADR-0013
shape: this round as input, the fork decision recorded inside it ("taken
separately by Nate McBride on 2026-08-08"), attack 1's authoring-verb
rule and attack 2's sweep as drafting obligations, SI-35 moving to
Resolved-on-acceptance with its evidence clause (3) explicitly carried
open until the parser lands. Spec change: likely minor-to-major depending
on whether any MUST is narrowed — the draft classifies it against §0.1
and says so before anyone asks.
