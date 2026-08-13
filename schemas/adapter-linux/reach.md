# The Linux contract's INV-003 reach declaration, `partman.adapter-linux.reach/0`

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §7.1 (INV-003) and
  MODEL-003
- Owner: WP-L100 (`docs/work-packages/WP-L100.md`), increment 1
- Decided semantics carried: ADR-0013 (INV-003 detection is scoped by
  privilege, and the reach is published per contract rather than per
  device), ADR-0014 (the partition-table state is authored by the
  privileged helper, so no client emits one)
- Implementation: `crates/adapter-linux`'s `reach` module; the vocabulary,
  ordering, and basis coupling below are held in agreement with it by the
  `the_linux_reach_declaration_is_complete_and_ordered` and
  `the_linux_reach_declaration_claims_no_state_and_cites_nothing_yet` tests
- Underlying byte profile: none — this is a JSON text surface, not a
  `pce/1` body, so it carries its major version in the identifier suffix
  the way WP-035's text surfaces do

This document records a delivered format. It decides nothing: a field
exists here because `crates/adapter-linux` publishes it, and that module's
tests are the authority wherever a sentence could be read two ways.

## 1. What the declaration is, and is not (INV-003, ADR-0013)

INV-003 requires the unprivileged discovery layer to publish the reach of
its platform contract: for each partition-table state the requirement
names, whether that contract can distinguish it on this platform. The
answer is a property of the contract and the platform. It is declared
independently of any device, is never derived per-device, and is never
omitted when the answer is `no`.

This document publishes that declaration for one contract — the one
WP-L100 delivers on Linux. It is not a claim about interfaces this
contract does not read, not a claim about any other platform, and never a
statement about what state a device is in: the state itself is authored by
the privileged helper from its own raw-sector parser (ADR-0014), and no
client computes one.

## 2. The payload (MODEL-003)

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.adapter-linux.reach/0`. Provisional within major version 0. |
| `contract` | Map | The contract statement of §3. |
| `states` | Array | One cell per INV-003 state, in INV-003's order — never partial. |

Any other identifier is refused rather than migrated — MODEL-003's
explicit-rejection arm. The array is fixed-size in the implementation, so a
missing cell is a compile error rather than an omitted `no`.

## 3. The contract statement

| Field | Type | Meaning |
| --- | --- | --- |
| `state` | Text | One of two words, closed at two: `not-implemented` (this contract touches no platform surface) or `implemented-reaches-no-table-state` (a contract exists and reads no table-state surface). |
| `reference` | Text | What changes the statement — an increment, or a recorded decision. |
| `detail` | Text | One sentence a reader can act on. |

The two words are not interchangeable and an earlier module elsewhere in
this repository collapsed them. "Reads nothing" and "reads identity
attributes but no table-state surface" produce the same all-negative cells
for different reasons, and INV-003 requires the declaration be derived from
the contract — so describing an existing contract as absent, or an absent
one as existing, makes it underived.

As shipped by increment 1 the statement is `not-implemented`, naming
increment 2: the bounded seam and this declaration are delivered, and the
attribute lists that first consult a platform surface are not.

## 4. A cell

| Field | Type | Meaning |
| --- | --- | --- |
| `state` | Text | The INV-003 state: `gpt`, `mbr`, `apple-partition-map`, `missing-table`, `hybrid-or-inconsistent`, `corrupt-metadata` — closed at six, in INV-003's own order. |
| `distinguished` | Bool | Whether this contract can distinguish the state. |
| `basis` | Text | `measured` or `not-measured` — closed at two, so a reader can tell a measured negative from one that was never measured. |
| `citation` | Text or null | The `docs/quality/observability.md` heading the cell rests on. Null **exactly** when the basis is `not-measured`. |

The coupling in the last row runs both ways and is held by test: a citation
beside a not-measured basis, or a measured basis with no citation, is a
malformed cell rather than a lenient one.

## 5. What increment 1 publishes, and why the basis is what it is

Every cell is `distinguished: false`, `basis: not-measured`,
`citation: null`.

The not-measured basis is a decision, not an absence of evidence.
`docs/quality/observability.md` §Linux holds measured rows that bear
directly on these answers: raw sector reads and direct signature probes are
both denied unprivileged, and its subsection "ADR-C3's `Present` and
`Indeterminate` are not distinguishable through either interface" measures
the state collapse across every WP-020 fixture. A measured basis is
therefore available.

It is not claimed yet because a measured basis must be a measurement about
*this* contract, and this contract reads no field. The same rows record
that the `udev` database does carry `ID_PART_TABLE_TYPE`, so whether a
state is reachable depends on which fields increment 2 puts in its lists,
not on the interfaces alone. Increment 2 fixes those lists and re-decides
each cell against those rows. The cells are expected to stay negative —
ADR-0014 forbids this client emitting a table state at all — but the basis
will then be earned rather than defaulted.

## 6. What this schema deliberately does not carry

- **No per-device anything.** No identifier, no path, no selector. A cell
  that varied by device would not be the contract's reach, and INV-003
  forbids deriving the declaration per-device.
- **No partition-table state.** This document says what a contract can
  distinguish, never what a medium contains. The state is helper-authored
  under ADR-0014 and has no field here to occupy.
- **No other platform's answers.** A sibling adapter publishes its own
  declaration for its own contract. Two contracts reading the same
  interfaces may still publish different bases, and reconciling them is a
  review question rather than a field.
- **No privilege dimension.** The reach is a property of the contract and
  the platform, so there is nowhere to record "and as root it would be
  different" — a contract that widened with privilege would already have
  violated INV-003 before reaching this document.
