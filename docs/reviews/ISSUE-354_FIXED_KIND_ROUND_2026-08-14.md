# Issue #354: the fixed-kind half — adversarial round, 2026-08-14

Untracked session artifact, `docs/reviews` convention.

> **Candidate REJECTED as written.** The ninth consecutive design in this
> area to be green on the full workspace and wrong. Read this before
> proposing any kind check.

**What was proposed** (my own candidate, from
`ADR-0037_PRECONDITION_READING_2026-08-14.md` §6.3): add a kind check to
`Topology::build`'s naming sweep for four `(node_kind, field)` pairs
whose lawful referent kind was claimed to be "fixed by MODEL-002's chain
and stated in the field's own doc comment":

```
("partition",               "parent_table")      => ["partition-table"]
("encryption-layer",        "backing_signature") => ["backing-signature"]
("conflicting-table-entry", "table")             => ["partition-table"]
("volume",                  "producer")          => ["aggregate", "encryption-layer"]
```

Three adversarial lenses reported. Verdicts: **reject**, **reject**,
**land-with-changes**.

## 1. The fatal, found independently by all three lenses, measured

**`("volume","producer") => [aggregate, encryption-layer]` false-refuses
every host-backed virtual device** — loop devices, VHD/VHDX, dm-linear,
plain dm-crypt, attached images — because a host-backed volume's producer
is the `BackingExtent` that carries its bytes.

Confirmed by hand against delivered code and normative text, not taken on
a subagent's word:

| fact | source |
| --- | --- |
| the delivered producer set already includes `backing-extent` | `protection.rs:534` — `producer_verdict` folds over `Production \| HostBacking` |
| the pair table admits it explicitly | `topology.rs:315` — `HostBacking => &[("backing-extent", "volume")]` |
| loop devices are MUST-discover | INV-001, `AGENT_BUILD_SPEC.md:516` |
| VHD/VHDX are MUST-discover-and-manage | WIN-003, `:577` |
| the host-backing edge exists for exactly this | MODEL-002, added 11.1.0 "closing CONC-001's empty loop-device bind set" |

So **the naming-side producer set I proposed is strictly narrower than
the protection-side one the product already ships.** The two would
disagree by construction — the precise defect PR #363 was written to end,
reintroduced one layer up.

Worse than a refusal: measured, **no node in an honest loop body is a
lawful producer** under the proposed set. `Volume.producer` is a
mandatory `NodeId`, so the layout becomes *unrepresentable* — the
"fail-closed-by-unencodability is not fail-closed" failure that
`naming.rs`'s own module doc records as discharged at its layer.

## 2. The premise was false

The candidate justified the four pairs as "fixed by MODEL-002's chain".
Measured against the sources:

- MODEL-002's chain (`:370`) contains **neither `backing-signature` nor
  `conflicting-table-entry`**, so it cannot fix two of the four.
- Read *as* a referent rule it gives the **opposite** answer for the
  third: the chain's predecessor of "encryption/container" is
  `partition`, not `backing-signature`.
- `:372` immediately says the chain is not exhaustive ("It MUST also
  represent non-linear relationships").
- ADR-0019's naming maps — normatively incorporated by Section 5 — name a
  referent **kind** for five rows but **deliberately name none for
  Volume/producer** (`0019-si27-node-naming.md:82`: "producer id", with
  no kind, unlike "parent **device** id", "parent **table** id", "its
  **backing signature's** id", "host **table** id").

The one normative source that could have grounded the fourth pair
deliberately declines to. **Doc comments are not the spec**, and this
repository had already measured one comment in that same set to be wrong
(`PartitionTable.parent`'s "the device the table describes").

## 3. It also buys almost nothing, measured

- **The increment removes zero instances of the harm ADR-0037:146-150
  names.** A partition whose `parent_table` names a real
  `partition-table` whose `parent` names a `file-system` still builds —
  the guarded hop is the first of two, and the unguarded second hop is
  where the forbidden pairing lives.
- **Two of the four guarded fields can carry no extent** (`volume`,
  `encryption-layer`, per `may_carry_extent`), so no frame is ever
  derived for them — they sit outside ADR-0037's concern entirely.
- **A kind-correct decoy buys the same escape**: a second
  `PartitionTable` view with `role: HybridMbr` is lawful under ADR-0019
  and is a `partition-table` under any kind check, so a partition named
  off the decoy survives a wipe of the real table.
- **The volume arm had zero coverage.** Mutating its key from
  `"producer"` to `"produced_by"` left the domain suite **bit-identical**
  — the arm was dead and nothing noticed.

## 4. The MODEL-003 obligation I denied

Measured: a `schema_version: 1` body holding a loop-device volume, which
encodes, hashes and decodes at HEAD, **stops decoding** with the check
on. `from_canonical_body` → `assemble` → `Topology::build`. Issue #354
lists MODEL-003 as one of three things a fix must decide; the candidate
did not mention it, having assumed #362's explicit-rejection reasoning
carried over. It does not: #362 refused bodies that were never lawful
under MODEL-002, whereas this refuses bodies two MUST requirements
require.

## 5. What survives, and the one constructive path

Lens 1 could construct no honest counterexample for the other three
pairs, having tried EBR logical chains, BSD disklabels inside MBR,
APM/GPT hybrid views, LUKS1/2, BitLocker, FileVault and ZFS native
encryption. It recommends landing those three and holding the fourth.
**That is not endorsed here**, because §3 shows the three buy nearly
nothing and §2 shows their stated justification is false — a correct
result reached by a wrong argument still needs a right one.

The only measured constructive path: derive the producer set from the
delivered relation instead of authoring it —

```
{k : endpoint_pair_allowed(Production, k, "volume")
  || endpoint_pair_allowed(HostBacking, k, "volume")}
```

which is exactly what `protection::producer_verdict` folds over, so the
two cannot drift. Measured: it re-admits every false-refused layout while
the wrong-kind case still refuses. But note where that lands — a check
derived from the pair table is **the design the panel already rejected**,
and it reinherits #360.

## 6. What this round did NOT establish

The orchestration died mid-run: **two of five lenses produced no
recoverable output** — *bypass/vacuity* (collision groups, node ordering,
`Unrecognized` enum variants, and whether the check is already implied by
another gate) and *blast radius* (the planner's simulated rebuild, the
golden vector, interaction with #319/#333/#347/#349/#356). Findings from
those angles are **unknown**, not absent. The verification phase that
would have adversarially re-checked each fatal also did not run; the
central fatal was instead verified by hand against delivered code and
spec, and the rest are recorded with their `measured` flag as reported.

## 7. Consequences

1. **Do not land the four-pair check.** Do not land the three-pair
   variant either without a fresh justification, since the one it had is
   false.
2. **Correct `ADR-0037_PRECONDITION_READING_2026-08-14.md` §3**, whose
   MODEL-002-chain premise this round refutes. Its §1–2 conclusion —
   ADR-0037:217 is not satisfied and #333 stays blocked — is untouched by
   this and stands.
3. **Add a committed host-backed volume body** (extent-produced, no
   aggregate, no encryption layer) to the fixtures. Its absence is why
   645 green tests could not see this: every committed `Volume` names an
   aggregate or an encryption layer, and `one_of_each`'s single volume
   conflates the two producing relations.
4. **Correct two doc comments in `naming.rs`** — `Volume.producer` ("the
   producing aggregate or encryption layer", contradicted by its own
   variant doc "a volume **or produced virtual device**") and
   `PartitionTable.parent`.
5. A lawful-referent-kind table, if one is ever wanted, is
   **requirement-shaped** and belongs in the spec or an ADR, not authored
   into code with a doc comment as its citation.
