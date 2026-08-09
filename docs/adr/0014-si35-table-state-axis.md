# ADR-0014: The helper is the sole author of partition-table state

- Status: **Proposed — awaiting acceptance by the decision owner. The
  *axis* this drafts was accepted by Nate McBride on 2026-08-08 from the
  same day's adversarially reviewed round
  (`docs/reviews/SI-35_AXIS_ROUND_2026-08-08.md`, untracked session
  artifact; everything load-bearing is restated here). This text is its
  instrument and awaits its own acceptance; until this line is replaced
  by an acceptance record, SI-35's register row does not change.**
- Date: 2026-08-08
- Spec version: none — deliberately. This ADR fixes a design axis inside
  the specification's existing texts; the normative amendments it makes
  necessary are enumerated under Consequences and land with SI-35's
  resolution round under their own grants. Section 0.2 item 4 forbids an
  ADR being the instrument of a requirement change, and this one is not.
- Work packages blocked: WP-010 increment 3 (SI-35 remains a direct
  blocker — axis decided, resolution waiting on the parser and its
  refusal demonstration)
- Requirement IDs: ADR-C2, ADR-C3, ADR-C4, MODEL-004, MODEL-005,
  PLAN-006, PLAN-007, CONC-004, HLP-002, HLP-003, SAFE-002, SAFE-003,
  SAFE-005, INV-002, INV-003, PART-001, Section 11.4
- Decision owners: Nate McBride

## Prior decision recorded here

**The ADR-C4 guard fork was decided separately by Nate McBride on
2026-08-08**, from its own adversarially reviewed round
(`docs/reviews/ADR-0014_FORK_RECOMMENDATION_2026-08-08.md`): the guard —
"a positively absent partition table and an unreadable one produce
different body values" — is not an absolute veto but a priced permission,
amendable only by a draft that simultaneously keeps the vocabulary
representable, replaces the protection with a normative helper
categorical invariant plus its mutation-verified two-fixture test, names
what the authorization hash stops committing to and assigns the
remainder, and weighs the body-resident alternative explicitly. **This
ADR does not exercise that permission** — the design below satisfies the
guard unamended — and the price list stands for any future draft that
must.

## Context

SI-35 filed the conflict from a measurement: ADR-C3 requires three
partition-table states, INV-003 requires detecting inconsistent and
hybrid tables, and no measured client projection encodes the distinction.
The campaign is complete — a descriptor-bound non-WSL Linux loop run
(2026-08-03, valid on its third sitting), a decision-complete Windows
rerun (2026-08-04), a macOS matrix (2026-08-05, valid on its second
sitting) — and two facts from it collapse the design space:

**No client contract computes the states, and neither do the helper's
tools.** The decisive pair — `gpt-basic-512` against
`gpt-conflicting-tables-512` — is byte-identical to every enumerated
client projection on all three platforms, and the labelled privileged
`blkid -p -o udev` and `wipefs -n` probes returned identical output for
the pair too. What separated it (M10, 2026-08-05) was raw bytes:
identical first-64-KiB digests, differing last-64-KiB digests, the
disagreement living in the backup table. The separating contract is not
"run a privileged tool"; it is *parse the tables from raw sectors* —
both GPT copies, CRCs validated. Section 11.4 already lists GPT, MBR,
and APM header parsers as planned fuzz-obligated parsers.

**ADR-C3's own vocabulary makes `Present` privileged by definition.**
`Present` means "read and hashed": producing it requires reading table
bytes, and the unprivileged raw read is denied on all three platforms,
measured on each. Spec 7.0.0 (ADR-0015) recorded the same for `Absent`
where the client contract does not separate it. The client's honest
ceiling is therefore not "some states, sometimes"; it is **no table
state at all**. That is a measured fact plus a definition, not a design
preference, and this ADR mostly writes it down.

## Safety analysis

**Who computes: the privileged helper, from its own raw-sector table
parser.** Both GPT copies read and CRC-validated, MBR and APM per their
structures. `blkid` and `wipefs` are measured non-separating for the
decisive pair and are not the contract. The parser is a Section 11.4
parser of on-disk bytes: `unsafe`-free, fuzz target landing with it, and
SI-35's third evidence category — the demonstration that the chosen
option refuses rather than proceeds on `gpt-conflicting-tables-512` —
landing with it. Two independently valid tables describing different
partitions is ADR-C3's "ambiguous": the parser stamps `Indeterminate`,
SAFE-005 disables the affected write, SAFE-003 keeps the record's
strength honest.

**What the client does: never emits a table state, on any platform.**
Not `Indeterminate`-from-here — ADR-C3's `Indeterminate` is a
determination about the medium ("unreadable or ambiguous"), and a client
that has read no table bytes reporting it would represent blindness as a
determination, the shape ADR-0013 forbade. Client surfaces carry raw
interface-labelled observations and the published INV-003 reach
declaration, exactly the surface WP-035 shipped. Two questions dissolve
here rather than being answered: the option (a) privilege-tag has
nothing to tag when one observer class writes the field, and **INV-003's
`Present` face — deliberately left to this issue by SI-39's filing —
resolves for the client** because a client that makes no table-state
report cannot make the forbidden "reporting a table as consistent"
report. The helper's full detection duty stands per ADR-0013's
privileged scope.

**Where it lives: in the hashed body, helper-authored at validation.**
The specification's flow already orders this correctly: the helper
validates (HLP-001's validate-plan), the plan waits in
`AwaitingAuthorization`, HLP-003 binds the user's fresh interactive
authorization to the exact post-validation hash, and `Revalidating`
follows authorization before apply. So the artifact the user authorizes
carries table state stamped by the observer that read the bytes — and
ADR-C4's guard is **satisfied, not amended**: `Absent` and
`Indeterminate` produce different body values, authored by the only
observer that can tell them apart. ADR-C2's "bound device identities are
body" row holds. MODEL-005's body-stability rule holds: the parser is
deterministic over unchanged bytes. PLAN-006's body-hash comparison
becomes helper-authored against helper-recomputed — one observer class,
so the two-observers-two-bodies unsatisfiability that ADR-C2 was created
to prevent never arises for this field.

**The sweep the axis round demanded, performed, and its finding.** The
round's second attack required checking MODEL-005, CONC-004, Section
20's snapshot vocabulary, and SI-27's scope for any duty requiring a
client-produced hashed snapshot. The finding is sharper than the attack:
Section 6's plan content list binds a "source topology snapshot body
hash" at drafting time — unprivileged — while PLAN-006 has the helper
re-discover and demand body-hash equality. If the bound snapshot's body
carried client-authored table state, no plan could ever validate; if it
carried none while the helper's recomputation carried some, equality
fails identically. **The design therefore requires what the flow already
permits: the snapshot hash the authorized plan binds is the one
validation produces.** Drafting proceeds against the client's view as
the proposal; validate-plan re-discovers under HLP-002, produces the
authoritative snapshot with table state stamped, binds its hash, and the
user authorizes the result. A welcome consequence travels with it: bound
identities and strength in the *authorized* plan are validation-verified
— the user who approves a Strong record approves one the helper
established, not one the client guessed — while client-side inventory
displays honestly show strength pending validation (a client-derived
record is Weak wherever its table state is undetermined, which under
this axis is everywhere; that is ADR-0015's population logic carried to
its measured conclusion, and the strength *rule* is untouched).

**When the contract is silent: client-only artifacts carry no table
state and are never plan-bound.** HLP-002 already treats client
discovery output as an untrusted hint; this makes the hint status
structural. CONC-004 is unaffected — helper-produced snapshots can be
transitional too, and the transitional marking stays body content.

**PART-001's categorical invariant, adopted although the guard
survives.** Initialization proceeds only on the helper's own fresh,
positively determined `Absent` at apply time — never on a plan-carried
claim. The representational protection (the guard) and the procedural
one (the invariant) close different routes to the same loss, one costs
one sentence, and ADR-0015 already shaped it.

## Options considered

### Option (a) — privilege-tagged state

Dissolved rather than rejected: with a single authoring observer there
is nothing to tag. Were a second author ever admitted, the tag returns
as hash-visible body content — the register's recorded cost — which is a
reason to keep authorship single, not to price the tag now.

### Option (b) — clamp to a named reproducible client projection

Rejected on its own evidence clause: it requires positive evidence that
a selected client contract separates every state in scope, and two
decision-complete campaigns produced none. A future named-and-measured
interface is a revisit condition below, not a foundation.

### Option (c) as filed — `Indeterminate` helper-only, the client claims the rest

Rejected because the client cannot honestly claim the rest: `Present`
requires reading and hashing table bytes the client is denied
everywhere, measured. This decision is (c)'s direction carried to where
the measurements and ADR-C3's definitions point — helper-only
*everything* — and (c)'s inherited monotonicity obligation dissolves
with it: there is no client-claimed state for the helper to tighten.

### Out-of-body residence — the fork's priced route

Not taken. Helper-authored body residence keeps every property the
guard protects — the authorization commits to content state, PLAN-006
compares meaningfully, `Absent`/`Indeterminate` stay body-distinct — at
no amendment cost. The priced permission remains available if the
resolution round's schema work surfaces a shape the body genuinely
cannot carry; its four conditions bind any such draft.

### Helper-authored body state (accepted)

Accepted, as specified in the Safety analysis.

## Decision

The axis is fixed: **the privileged helper is the sole author of
ADR-C3's partition-table state, computing it from its own raw-sector
parser; the client emits no table state; the state lives in the hashed
body of helper-produced artifacts, stamped at validation; client
artifacts are unhashed-or-unbound hints.**

**SI-35 remains Open — axis decided**, the ADR-0012/SI-11 shape the
reservation prescribed. Resolution waits on the parser, its fuzz target,
and the refusal demonstration on `gpt-conflicting-tables-512` that
SI-35's evidence clause has always required of the chosen option. The
direct-blocker count does not change.

## Consequences

The normative amendments this axis makes necessary, each landing with
SI-35's resolution round under its own grant — enumerated here so they
are obligations on the record rather than discoveries:

- **PART-001** gains the categorical invariant (initialize only on the
  helper's fresh positively determined `Absent`) — a narrowing of an
  existing MUST, therefore major when it lands.
- **ADR-C2's placement table** gains the authoring verb: "table state |
  body | helper-authored at validation, recomputed at revalidation" —
  the round's first attack, discharged as spec text rather than left as
  three verbs where two were stated.
- **Section 6's plan content list** clarifies "source topology snapshot
  body hash" to "as bound at validation," making the sweep's finding
  text rather than inference.
- **INV-002/INV-003 client surface**: the client-emits-no-table-state
  prohibition stated in terms (an added prohibition, minor by the 6.1.0
  precedent, but travelling with the major above).

Other consequences:

- **Negative, accepted knowingly: client-side inventory shows no table
  state and no Strong identity, on every platform.** The moment a user
  browses an inventory is the moment this design declines to promise a
  table-state answer; the honest display is the observation set, the
  reach declaration, and strength-pending-validation. Full determination
  happens before any write, where it always had to.
- **The parser is a new Section 11.4 obligation** with the same
  discipline the plist reader just exercised: bounded, `unsafe`-free,
  fuzz target and boundary tests landing with it, refusal demonstration
  recorded as SI-35 resolution evidence.
- SI-34 is untouched: protection-verdict placement and the freshness
  projection remain its questions, over the PLAN-006 comparison this
  design leaves exactly where the spec put it.

## Verification

- When the parser lands: the refusal demonstration —
  `gpt-conflicting-tables-512` stamps `Indeterminate` and the affected
  write is disabled (SAFE-005) — mutation-verified, closing SI-35's
  third evidence category.
- The two-fixture PART-001 test from ADR-0015's shape, extended to the
  invariant: `blank-512` proceeds only via helper-determined `Absent`;
  `gpt-conflicting-tables-512` and every `Indeterminate` refuse.
- A validated plan's bound snapshot hash is helper-produced: a test that
  a plan binding a client-produced snapshot hash does not validate.
- Register: SI-35 reads "Open, axis decided (ADR-0014)"; any text
  reading it as Resolved is an error against this ADR.

## Revisit conditions

- A client-readable interface is named and measured, under the standing
  custody rules, to separate the three states on every supported
  platform — option (b)'s foundation arriving after all; the axis would
  deserve re-examination, not silent survival.
- The plan flow's ordering changes such that authorization can precede
  validation, which would break the stamp point this design rests on.
- The resolution round's schema work finds a shape the body cannot
  carry, at which point the fork's priced permission — not this ADR —
  governs what an out-of-body draft owes.
