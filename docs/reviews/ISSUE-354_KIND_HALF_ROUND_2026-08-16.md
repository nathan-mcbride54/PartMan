# Issue #354 — the kind half, 2026-08-16

`docs/reviews` artifact, committed under WP-000 in its own pull request
beside the act it records (WP-010, ADR-0045). **Single-author**, standing
on the two adversarial rounds this issue already had: the four-design
judge panel of 2026-08-14 (`ISSUE-354_REFERENT_SWEEP_PANEL_2026-08-14.md`),
whose winner — derive the kind check from `endpoint_pair_allowed` — this
act is once the table is right; and the fixed-kind round of the same day
(`ISSUE-354_FIXED_KIND_ROUND_2026-08-14.md`), whose fatal is now a
committed control. This round's adversarial content is: the one question
the previous handoff left (is the multipath omission ADR-0011's intent?)
answered by measurement rather than by reading; an enumeration of every
(field, kind) pairing rather than a sample — the fixed-kind round's caveat
was that two of five lenses produced no output, and an enumeration is the
answer to a lens that did not run; a seven-mutation battery, each proven
applied; and the workspace run with the check on, since the previous
candidates died on honest layouts the fixtures did not contain. That is
weaker than a panel and is stated as such; the ADR is where the decision
owner is put the question.

## 1. What the round had to satisfy

1. Decide the multipath population — deliberate omission (then the
   derived check enforces it) or a further omission (then a row first).
2. Derive the check from the pair table, with no second authored list
   (the panel's finding; the fixed-kind round's fatal in the negative).
3. Not decide edge/name agreement — the third strength — by accident
   (#333's, and the issue's own instruction).
4. MODEL-003: a versioned behaviour change at the decode boundary,
   priced.
5. Discharge ADR-0037:146-150 so `:217`'s precondition is satisfied.
6. Change `a_wrong_kind_referent_still_builds_and_that_is_the_held_half`
   deliberately, and keep the honest-layout controls building.

## 2. The findings

*Finding 1 — the multipath omission was a fail-open, not a decision.*
ADR-0011 represents "what the kernel itself materializes", the node and
its members, and refuses mutation on both; it says nothing about content
hosted on the node. Read the code: `device_scope_verdict` inherits a
containment root's own arm by walking containment edges upward. An xfs
naming `/dev/mapper/mpatha` as `host` built at HEAD (the panel's third
honest layout, and the standing control); no row could carry an edge, so
the ascent found the xfs its own root, inherited nothing, and every one
of its ten mutating gates was `Clear`. The capability engine's multipath
arm (`multipath_scoped`) scopes the node and its members only. So the
answer to the handoff's question is *both*: an omission, and one that
left content on a detection-only device unprotected. The three rows a
volume has (`backing-signature`, `file-system`, `partition-table`) are
what a multipath node needs, for the same reason — it is an extentless
frame with content framed on it — and with them the node is a
containment root, its own arm (`Refused{RemoteTransport}`) is inherited,
and the gate is `Unsupported{InheritedDeviceScope}` ×10.

*Finding 2 — the rule is a map from field to relation.* Each of the eight
naming fields names the *source* of an incoming edge of a stated kind,
with the owner as target: containment for `PartitionTable.parent`,
`Partition.parent_table`, `BackingSignature.host`, `FileSystem.host`,
`ConflictingTableEntry.table`; backing for `EncryptionLayer.backing_signature`;
production or host-backing for `Volume.producer`. The admissible kinds are
`endpoint_pair_allowed(kind, referent_kind, owner_kind)` at the moment of
the check — the same call the edge check makes two lines below. What is
authored is which *relation* a field names, which is a fact about the
field's meaning and not a catalogue of layouts; the catalogue is the
table. `BackingExtent.host` is open: no edge kind targets a backing
extent, so the table has no opinion, and any list written for it would be
the panel's second list. An unclassified field admits nothing (fail
closed), and a pin test names all eight.

*Finding 3 — the workspace tells you what the fixtures know.* With the
check on, every crate is green — `body_vectors`, the cross-language
golden vector, the planner's rebuilds, the capability engine — and the
one red is the held-half pin. That is a measurement of the committed
population and it is not evidence the check is right (the fixed-kind
candidate was green on 645 too); the honest-layout controls and the
enumeration are where the argument lives.

## 3. Measured (`ea299eb` → candidate)

| what | before | after |
| --- | --- | --- |
| `Partition.parent_table` = physical device | builds; decodes; planner rebuild stands | `ForbiddenNamingReferent{partition, parent_table, physical-device}` at assembly and at decode, the same value |
| `Volume.producer` = partition; `EncryptionLayer.backing_signature` = file system | build | refuse, naming the pairing |
| the naming enumeration: 7 relation-bound fields × 11 kinds, + the open field × 11 | — | 17 admitted, 60 refused, 11 open-admitted; every row asserted with the full error value |
| GPT inside LUKS (`PartitionTable.parent` = volume) | builds | builds with `volume → partition-table` |
| partitioned mdraid (`aggregate → volume → table`) | builds | builds with edges; `aggregate → partition-table` asserted refused |
| xfs on multipath (`FileSystem.host` = multipath node) | builds, edgeless | builds with `multipath-node → file-system` |
| partitioned multipath node (kpartx) | builds, edgeless | builds with edges |
| loop-backed volume (`Volume.producer` = backing extent) | builds | builds with `HostBacking` (the fixed-kind round's fatal, as a control) |
| xfs on multipath, edge present | unrepresentable | `Unsupported{InheritedDeviceScope}` ×10 |
| xfs on multipath, edge omitted | `Clear` ×10 — the fail-open | `Clear` ×10 — the named limit, pinned; **#397** |
| the workspace | 669 passed | 673 passed; `cargo xtask ci` exit 0 (630 annotations, 666 live tests) |

## 4. Mutations (each proven applied by grep of the mutated line, the domain suite run)

| # | mutation | outcome |
| --- | --- | --- |
| M1 | kind check disabled (`Sources(_) => true`) | killed ×4: the wrong-kind test, the enumeration, the pin, the decode forgery |
| M2 | `Volume.producer` bound to `Production` alone | killed ×2: the loop-backed control, the pin |
| M3 | unclassified field admits everything (`_ => Open`) | killed ×1: the pin's own assertion, written for it |
| M4 | the three multipath rows removed | killed ×2: the honest layouts, the inheritance test |
| M5 | relation direction reversed (`endpoint_pair_allowed(kind, owner, referent)`) | killed ×73 |
| M6 | `BackingExtent.host` bound to containment | killed ×8, `one_of_each` itself refusing |
| M7 | `FileSystem.host` left unclassified | killed ×20 — the loud-refusal property |

## 5. Rejected

A per-field list of lawful kinds (the fixed-kind design; the loop-backed
control and M2 re-kill it). Deriving with the multipath rows absent (the
panel's winner at `7fdba38`; false-refuses the xfs, and the absence was a
fail-open). Refusing content on a multipath node outright (INV-008 says
represent; the refusal §2.1 wants is mutation, delivered through the
arm). A `multipath-node → partition` row (a partition's carrier is its
table). Comparing name against edge (#333). Device scope by name (filed,
#397).

## 6. Not established

- A panel verdict. The claims most worth attacking: (a) that the
  field→relation map is not itself "a second authored list" — the
  defence is that it names a relation per field and no kinds, and that
  the pin test plus fail-closed default make a drift a red test rather
  than a silent widening; (b) that `BackingExtent.host` is rightly open
  rather than bound to something; (c) that the three multipath rows are
  the right three, and that a multipath node should be a containment root
  for device-scope purposes rather than only the capability engine's
  concern.
- Whether a table inside a partition (a BSD disklabel, a raw image on a
  partition) is a population the model must express — refused as a name
  now, unrepresentable rather than fail-open, a row when a fixture
  demands one.
- The reason surface: content on a multipath node reports the inherited
  device-scope ground, not `Reason::MultipathDetectionOnly` — truthful;
  widening `multipath_scoped` by root is WP-050's (`crates/capability`).
- Anything about #397 beyond the pinned row and the candidate in its
  filing.
