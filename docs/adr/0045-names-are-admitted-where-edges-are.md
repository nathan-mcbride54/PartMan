# ADR-0045: Names are admitted where edges are — the naming kind check, and content on a multipath node

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-354_KIND_HALF_ROUND_2026-08-16.md`, single-author
  with a seven-mutation battery, each mutation proven applied and each
  killed; committed under WP-000 beside this act), standing on the two
  adversarial rounds this issue already had — the four-design judge panel
  of 2026-08-14 (`ISSUE-354_REFERENT_SWEEP_PANEL_2026-08-14.md`), whose
  winner this act is once the table is right, and the fixed-kind round
  the same day (`ISSUE-354_FIXED_KIND_ROUND_2026-08-14.md`), whose fatal
  is now a committed control. Merging is not acceptance; the decision
  owner has not been put the question in person, and this ADR is where
  it is put.
- Spec version: **15.1.0 — minor under §0.1.** The argument is made
  below.
- Work packages blocked: none. Issue #354 closes here as filed.
  ADR-0037:217's precondition is satisfied; issue #333's enforcement is
  unblocked and is its own round. The device-scope limit is filed as
  issue #397.
- Requirement IDs: MODEL-002, MODEL-003, MODEL-005, SAFE-005, CAP-003,
  ADR-0011, ADR-0019, ADR-0037, ADR-0044
- Decision owners: Nate McBride

## Context

Eight node kinds embed a `NodeId` referent in their hashed name
(`PartitionTable.parent`, `Partition.parent_table`,
`BackingSignature.host`, `FileSystem.host`,
`EncryptionLayer.backing_signature`, `Volume.producer`,
`BackingExtent.host`, `ConflictingTableEntry.table`). Issue #354 measured
that no layer required any of them to resolve, to resolve to the right
kind, or to agree with the containment edge; PR #362 landed the first
strength (every referent resolves) and deliberately held the second,
because the only design with no second authored list — asking
`endpoint_pair_allowed` the same question about the *name* it is asked
about the *edge* — was measured to refuse three honest layouts the table
could not express. That was issue #360, closed by ADR-0044, which admits
two of the three (`volume → partition-table`); the third, content hosted
on a multipath node, was left as the one population this act had to
decide before deriving anything from the table.

The harm the kind check exists to prevent is ADR-0037:146-150's, and it
is a precondition, not a preference: *"a naming-derived frame can be
computed from a pairing the pair table forbids"*, and `:217` makes the
sweep's existence a verification condition of #333's frame enforcement.
The precondition-reading record of 2026-08-14 established that the
resolve-only half does not satisfy it — a wrong-kind referent still
built, pinned on purpose by
`a_wrong_kind_referent_still_builds_and_that_is_the_held_half`.

The fixed-kind round of the same day rejected a narrower design (four
fields, kinds fixed "by MODEL-002's chain") on three measured findings,
the fatal being that `Volume.producer => [aggregate, encryption-layer]`
false-refuses every host-backed virtual device — the producer of a loop
or VHD volume is the `BackingExtent` carrying its bytes, which the pair
table admits under `HostBacking` and `producer_verdict` already folds
over. Its standing lesson: the delivered relation is the pair table, and
a naming rule that does not read it is strictly narrower than the
product already ships. That round's caveat — two of five adversarial
lenses produced no output — is why this round enumerates rather than
samples.

## The decision

> **Every naming referent must resolve to an absorbed entry whose kind
> the endpoint-pair table admits as the source of the relation the field
> names. And a multipath node carries content the way a volume does: the
> table admits `multipath-node → backing-signature`, `→ file-system` and
> `→ partition-table`, and content so hosted inherits the node's
> detection-only refusal.**
>
> - *A map from field to relation, never a list of kinds.*
>   `naming_referent_rule(owner_kind, field)` classifies each of the
>   eight fields: `Sources(&[Containment])` for a table's `parent`, a
>   partition's `parent_table`, a signature's or file system's `host`, a
>   conflicting entry's `table`; `Sources(&[Backing])` for an encryption
>   layer's `backing_signature`; `Sources(&[Production, HostBacking])`
>   for a volume's `producer`; and `Open` for a backing extent's `host`.
>   `naming_referent_kind_allowed` then asks `endpoint_pair_allowed(kind,
>   referent_kind, owner_kind)` for each kind in the rule — the same call,
>   the same table, the edge check reads two lines below. A row added to
>   the table admits the name in the same act; a row absent refuses it;
>   there is nothing to drift. The rule is pinned per field by
>   `the_naming_referent_rule_is_pinned_per_field`, and an unclassified
>   field admits **nothing** — a field added to the roster without a rule
>   reds twenty tests rather than admitting silently (mutation M7).
> - *The one open field.* No edge kind targets a `BackingExtent`; the
>   table has no opinion on what hosts one (a file system, for a loop
>   file; a partition or device, for a byte range), and any list written
>   here would be the second authored list the panel rejected. It must
>   resolve, and nothing more is asked. This is not a gap in the harm's
>   sense: no frame is derived through a backing extent's host, and #365
>   carries what the host-backing relation still lacks.
> - *The refusal, and where it lands.* `Topology::build` refuses with
>   `TopologyError::ForbiddenNamingReferent { node, kind, field,
>   referent, referent_kind }` after the resolve check and before any edge
>   is read; `from_canonical_body` and the planner's simulated rebuild
>   route through the same constructor, so the decode boundary refuses
>   the same body with the same value
>   (`a_wrong_kind_naming_referent_refuses_at_decode`).
> - *The multipath rows, and why.* Asked in the round: is `multipath-node
>   → file-system`'s absence ADR-0011's intent? Measured: no, and it was a
>   fail-open. ADR-0011 represents "what the kernel itself materializes",
>   the multipath node and its members, and refuses mutation on both; it
>   says nothing about content hosted on the node, and §2.1's entry says
>   *never mutate a multipath device*. An xfs naming `/dev/mapper/mpatha`
>   as `host` built at HEAD; no row could carry an edge; its device-scope
>   ascent found itself its own root; and every one of its ten mutating
>   gates was `Clear` — the capability engine's multipath arm scopes the
>   node and its members only, and the closure had nothing to inherit
>   through. With the three rows and the edge, the node is a containment
>   root like a device, `device_scope_verdict` folds its own arm in, and
>   the gate is `Unsupported{InheritedDeviceScope}` ten times over. The
>   three rows mirror the volume's three (ADR-0044) because a multipath
>   node is the same kind of frame: extentless, its content framed on it.
>   No `multipath-node → partition` row and no `PlatformMembership` change;
>   the membership edge stays closure- and bind-inert exactly as ADR-0019
>   left it.

**What is not consulted.** Whether a *containment edge* agrees with the
name — that is #333's derive-and-compare, the third strength this issue
named and deliberately did not decide. No extent. No operation. No
ordering: the sweep reads the absorbed set, and the enumeration builds
every probe with the whole roster present.

**Two limits, pinned rather than hidden.**

- *Device scope ascends the edge set, not the name.* A body that names
  the multipath node in `host` and omits the edge still gates its content
  `Clear`, because `device_scope_verdict` walks containment edges. Pinned
  as the second half of
  `content_on_a_multipath_node_inherits_its_detection_only_refusal`, and
  filed as issue #397 with the fail-closed candidate (ascend the naming
  relation as well; under #355's own argument an added ancestry can only
  add refusal). It is the escape ADR-0043 closed for release and this act
  leaves open for scope; it is not made worse here — HEAD had the same
  hole for a remote-transport disk — and it is not this act's to decide.
- *A table inside a partition is expressible by no row.* A BSD disklabel
  in a slice, an image written raw onto a partition: `PartitionTable.parent`
  naming a `Partition` is refused, because `partition → partition-table`
  is not in the table and this act adds no row it has not measured a
  fixture for. The direction is unrepresentable, not fail-open — the
  layout cannot be captured, so it cannot be acted on — and it is the
  same shape as #360 was: a row, with its fixture and its protection
  measured, when a capture adapter meets one.

## Measured

At `ea299eb` → candidate, in a detached worktree with its own target dir.

| what | before | after |
| --- | --- | --- |
| `Partition.parent_table` = physical device (the issue's probe 2) | builds; decodes | `ForbiddenNamingReferent{partition, parent_table, physical-device}` at assembly and at decode, the same value |
| `Volume.producer` = partition; `EncryptionLayer.backing_signature` = file system | build | refuse, naming the pairing |
| the naming enumeration (7 relation-bound fields × 11 kinds, + the open field × 11) | — | 17 admitted, 60 refused, 11 open-admitted; every admitted pairing builds, every refused one refuses with the full value |
| GPT inside LUKS; partitioned mdraid (`aggregate → volume → table`); xfs on multipath **with its edge**; partitioned multipath node (kpartx); loop-backed volume (`producer` = backing extent) | build (the multipath one without an edge) | build, edges and all |
| xfs on `/dev/mapper/mpatha`, edge present | unrepresentable (no row) | `Unsupported{InheritedDeviceScope}` ×10 |
| xfs on `/dev/mapper/mpatha`, edge omitted, `host` names the node | `Clear` ×10 — **the fail-open** | `Clear` ×10 — the named limit, pinned, #397 |
| `aggregate → partition-table` | refused | refused, asserted |
| the workspace: every fixture, `body_vectors`, the cross-language golden vector, `crates/planner`'s rebuilds, `crates/capability` | green | green with the check on; the one red was the held-half pin, replaced |

**Mutation battery** (seven, each proven applied by grep of the mutated
line, the domain suite run): M1 the kind check disabled — killed ×4; M2
`Volume.producer` bound to `Production` alone (the fixed-kind round's
fatal re-derived) — killed ×2, the loop-backed layout among them; M3 an
unclassified field admits everything — killed ×1 (the pin test's own
assertion, written for it); M4 the three multipath rows removed — killed
×2 (the honest layout, the inheritance test); M5 the relation's direction
reversed (`endpoint_pair_allowed(kind, owner, referent)`) — killed ×73;
M6 a backing extent's `host` bound to containment — killed ×8, `one_of_each`
itself refusing; M7 `FileSystem.host` left unclassified — killed ×20, the
loud-refusal property demonstrated. Workspace: 673 tests, 0 failed;
`cargo xtask ci` exit 0; `crates/capability` and `crates/planner`
unchanged and green.

## Options considered, and rejected

- **A per-field list of lawful kinds** (the fixed-kind round's design).
  Strictly narrower than the delivered relation; false-refuses every
  host-backed volume; rejected there, re-asserted here as the loop-backed
  control and mutation M2.
- **Deriving from the table with the multipath rows absent** (the panel's
  winner as measured at `7fdba38`). False-refuses an xfs on a multipath
  node; and the absence itself was a fail-open. Rejected: the table is
  corrected first, as it was for #360.
- **Refusing content on a multipath node outright** (reading ADR-0011 as
  "no content is represented"). INV-008 says represent rather than
  discard, ADR-0011 says detect and represent, and the refusal the
  spec wants is *mutation*, which the rows deliver through the ordinary
  arm. Rejected.
- **A `multipath-node → partition` row, or admitting the aggregate as a
  table's parent.** Neither is a shape; a partition's carrier is its
  table, and ADR-0044 already decided the aggregate. Not taken.
- **Making the check also compare name against edge.** #333's held
  enforcement, decided nowhere by accident (the issue's own instruction).
  Not taken.
- **Device scope by name.** Fail-closed and cheap, and a different
  decision about what the naming relation is load-bearing for. Filed as
  #397, not folded in.

## MODEL-003

Taken under the **explicit-rejection** limb, `SCHEMA_VERSION` unchanged,
on #362's own reasoning restated: the byte format and the parse rules
are untouched (`fields_from_map` accepts exactly what it accepted); the
refused population is bodies that were never lawful under MODEL-002,
only unvalidated; the golden vector, `body_vectors` and every committed
fixture decode as before; and a bump would make every conforming v1 body
undecodable, the cross-language vector included, for a rule that changes
no conforming artifact's meaning.

## The spec-price argument

**Minor, 15.1.0.** Section 5's naming paragraph gains the referent rule;
MODEL-002 gains the multipath-carries-content sentence and the three
rows; §2.1's multipath entry gains that content hosted on a multipath
device is refused with it. Every one is an addition to previously
unspecified territory — no numbered requirement's text changes meaning,
and reach only grows (content on a multipath node refuses where it was
`Clear`). The patch reading (a check nothing captures yet is editorial)
is declined on ADR-0037's own words: a rule is requirement-shaped whether
or not anything implements it. The major reading is declined because no
existing sentence is narrowed — MODEL-002's chain never claimed a
multipath node carries nothing, and ADR-0011's decision is refined in the
direction it points.

**Not a §1.11 filing.** No two requirements conflict; ADR-0037's
precondition was an unmet obligation, and this act discharges it.

## Consequences

- **Positive:** the harm ADR-0037:146-150 names cannot occur — no frame
  can be derived from a pairing the table forbids, at capture, at decode,
  or in the planner's rebuild; #333's enforcement is unblocked; content on
  a multipath node is representable *and* refuses; the layering inversion
  #354 named narrows further (`OccupancyGround::TableIsNotThisHosts` is
  now the edge-agreement check alone, which is #333's).
- **Negative, accepted knowingly:**
  - The device-scope limit (#397).
  - A table inside a partition is unrepresentable as a name — the
    fail-closed direction, and a row when a fixture demands one.
  - The capability reason for content on a multipath node is the
    inherited device-scope ground, not `Reason::MultipathDetectionOnly`;
    truthful, and WP-050's `multipath_scoped` may widen by root on its
    own round.

## Verification

- `a_wrong_kind_referent_refuses_naming_the_pairing`,
  `the_naming_referent_rule_is_pinned_per_field`,
  `naming_admits_exactly_what_the_pair_table_admits`,
  `honest_layouts_the_kind_check_would_have_refused_still_build`
  (`topology_tests.rs`); `a_wrong_kind_naming_referent_refuses_at_decode`
  (`snapshot_tests.rs`);
  `content_on_a_multipath_node_inherits_its_detection_only_refusal`
  (`protection_tests.rs`); `every_triple_outside_the_pair_table_is_refused`
  enumerates the extended table.
- Any text implying that a naming field may reference a kind the pair
  table does not admit under its relation, that a backing extent's host
  is kind-checked, that this act compares a name against an edge, or that
  ADR-0011 forbids representing content on a multipath node, is an error
  against this ADR.
- Any claim that this ADR delivers ADR-0037's frame enforcement (#333),
  or that content on a multipath node with its edge omitted refuses, is
  an error against this ADR.

## What stays open

- **#333** — ADR-0037's enforcement, derive-and-compare form, golden
  vector regenerated in the same act; now unblocked.
- **#397** — device scope by name.
- **#365** — the host-backing relation's representation; this act's
  open field is where its host question lives.
- A table inside a partition, when a fixture demands it.

## Revisit conditions

- A field is added to the naming roster: classify it in
  `naming_referent_rule` — the pin test reds and the suite refuses until
  it is.
- A row is added to the pair table: the naming check admits it in the
  same act; the honest-layout controls are where its fixture belongs.
- #397 lands: the omitted-edge row moves and its pin is rewritten
  deliberately.
- ADR-0011's revisit fires (multipath becomes a write target): the three
  rows are where its content's protection then flows from, and
  `multipath_scoped` should be re-read.
