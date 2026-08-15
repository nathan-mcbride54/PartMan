# Issue #319: extent absence fails open — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. Everything
load-bearing is restated in the ADR that lands the decision.

Two of the three routes died in the adversarial pass, of the same
cause, and the cause is **not the defect #319 filed**. This round
therefore recommends less than any route proposed, and files what it
found instead.

**What is being decided.** A node present in the topology with
containment edges but absent from `Facts.extents` is invisible to two
delivered arms. `free_extents` subtracts only children whose
`range.host == host` (crates/planner/src/solve.rs:257), never
consulting containment, so an extent-less child's bytes read as free.
`affected_set` reaches nodes by range only through
`facts.extents.get(&id)` (crates/domain/src/model/protection.rs:232),
so on a create — which declares `consumed` and destroys nothing —
`node_verdict` is never consulted for a buried node. `Facts`' own doc
says the opposite is the design: "absence of a fact is honest absence
and fails closed at the arm that needs it" (protection.rs:72-74).
**Q1** which kinds must carry an extent, and whether the table gains
one or the head/tail regions get a different guard; **Q2** solver,
protection closure, or both; **Q3** whether the decode boundary should
require extents outright.

## What the round established

Findings 1, 2 and 5 were re-executed by hand at HEAD (`ecb3dc6`) in a
throwaway worktree outside the repo; the transcript of each run is in
this session. Findings 3, 4 and 6 are read off the cited text. Claims
sourced only from the round's agents are marked **[round-reported]**
and were not independently re-run.

1. **The 1 MiB-at-offset-0 placement is real. Measured.** Against
   unmodified `solver_fixture` (crates/planner/src/tests.rs:504),
   `free_extents` returns `[(0, 1048576), (68157440, 36700672),
   (138412544, 935329280)]` and `place_create(&snapshot, host, 1 MiB)`
   returns `start=0 length=1048576 end_placement=Aligned`. A create of
   exactly the default alignment lands on the protective MBR and the
   GPT header, recorded as conforming. The mechanism is
   `align_up(0, 1 MiB) == 0` (solve.rs:190-192) with the skip at
   solve.rs:312, so **zero is the only sub-1 MiB start reachable** — a
   head guard need defend one range, not a class.
2. **The fail-open has a committed witness asserting it is correct.**
   `free_extents_are_the_hosts_minus_its_children` asserts the first
   free range is exactly `(0, DEFAULT_ALIGNMENT)`
   (crates/planner/src/tests.rs:583), claimed under PLAN-001 at
   docs/traceability/WP-060.md:32. The table node in that fixture has
   a containment edge and no extent (tests.rs:507-511, 523-540).
3. **The "extent-less table" convention is not a rule.** ADR-0018
   names the table in its roster: "extents exist on the extent-bearing
   kinds — device, table (explicit primary/backup/MBR/EBR extents),
   partition, signature (primary offset), file system (superblock
   offset)" (docs/adr/0018-si11-protection-closure.md:416-419).
   Verified verbatim. **That sentence sits under `## Safety analysis`
   (0018:54), in `### Bind set, extents, free extents, rulesets`
   (0018:406) — not the Decision.** Cite it as ADR-0018's safety
   analysis, never as spec text. The fixtures are split: crates/domain
   gives tables extents (protection_tests.rs:102-109), crates/planner's
   solver fixture does not.
4. **That roster is unsatisfiable under the delivered carriage.**
   `Facts.extents` is `BTreeMap<NodeId, HostRange>` — one range per
   node (protection.rs:79) — so "primary/backup/MBR/EBR extents"
   cannot be carried, and the GPT backup header at the tail is
   unprotected under any required-extent scheme. Read off the type.
5. **DECISIVE, and it is a different defect: presence is not the
   property either arm consumes.** Both computations are
   host-qualified — `HostRange::intersects` opens `self.host ==
   other.host` (protection.rs:42), `free_extents` filters
   `range.host == host` (solve.rs:257) — and nothing validates
   `HostRange.host`: `assemble` stores facts unvalidated
   (snapshot.rs:91-106). The committed fixtures split on which address
   space a child inside a partition uses:

   | Fixture | Child | Anchored on |
   | --- | --- | --- |
   | protection_tests.rs:126-132 | ZFS signature | `sda_id` — the **device** |
   | body_vectors.rs:248-255 (golden vector) | mdraid signature | `part_id` — the **partition** |
   | plan_tests.rs:356-363 | file system | `part_id` — the **partition** |

   **Measured, with every extent present and nothing removed:**
   re-anchoring only the ZFS signature into the address space its own
   hashed name declares (`BackingSignature { host: member_id }`,
   protection_tests.rs:53-58) — the golden vector's convention —
   yields `pool reached: false`, `signature reached: false`,
   `member reached: true`, affected set 4 nodes, and a whole-device
   wipe **constructs**. `the_root_on_zfs_regression_pair_holds`
   (protection_tests.rs:157-193), ADR-0018's flagship destructive
   refusal, is defeated without removing a single fact.

   The mechanism is the closure's own guard: the member is
   `range_destroyed`, containment descent runs only from
   `cascade_destroyed` (protection.rs:259-266), so the signature
   inside the destroyed member is never descended into and the
   Backing edge to the pool never fires. **The restriction written to
   prevent round two's sibling capture is what opens this hole.**
6. **Correction to the round's own finding: the anchoring rule IS
   recorded, ambiguously, with a dangling citation.** The round
   reported "no ADR or spec text saying which anchoring is normative."
   That is not right. ADR-0018:420 states "Every range is
   host-qualified; one address space per containment-forest root," and
   the delivered `HostRange` doc echoes it — "one address space per
   containment root (ADR-0018 2.11)" (protection.rs:29-30). Two things
   follow. **ADR-0018 has no section 2.11** — its sections are titled,
   not numbered (verified: no `2.11` string in the file), so the
   delivered type cites a section that does not exist. And the
   sentence is genuinely ambiguous between (i) every range in a
   forest is expressed in the root's space, which makes the golden
   vector unlawful, and (ii) each range names its own host, which the
   `host` field's doc supports — "The node whose address space the
   range lives in" (protection.rs:33) — and which makes the sentence
   false as written. The fixtures split along exactly that ambiguity.
7. **Governance floor.** SAFE-005's seven named states —
   "Unknown file systems, corrupt metadata, ambiguous device identity,
   missing dependencies, stale topology, unsupported encryption
   states, and failed backups" (AGENT_BUILD_SPEC.md:178) — do not
   reach a fact the snapshot does not carry, so #319 is correctly a
   defect filing and **not a §1.11 entry**: that register needs two
   conflicting requirements and there is no such pair. §11.2's
   "Partition extents do not overlap" and "Extents remain inside the
   bound device" (AGENT_BUILD_SPEC.md:855-856) are MUST-prove
   obligations whose only claimed traceability rows are the
   preserved-alignment ones (WP-060.md:15, :29) — verified.

## Routes

- **(a) Extent totality at the constructor.** Six required kinds
  including `PartitionTable`, enforced in `assemble`; protection entry
  points retyped to `&TopologySnapshot`; `SCHEMA_VERSION` 1 → 2.
- **(b) Reserve the table's regions; require located occupants**
  (**recommended, in part**). Head/tail reservation derived from the
  table's own hashed `TableRole` (naming.rs:68-84, already in the
  address preimage); unlocated containment occupants refused inside
  `free_extents`. No schema change.
- **(c) Refuse at every authorizing arm; defer the schema.** A new
  `SolveRefusal` variant plus a bounded containment walk, and a reach
  widening feeding both verdict loops.

**No route survived intact.** (a) and (c) each carry a fatal fail-open
and it is the *same* one: both key their new guard on extent
**presence**, which finding 5 proves is not what `intersects` and
`free_extents` consume. Both were measured still open against a
snapshot with every extent populated and a child anchored per the
golden vector. **[round-reported]** for the patched variants; the
unpatched equivalent is finding 5, which I re-ran.

## The adversarial pass on the recommended route

1. **"The domain half never checks a node's own extent."** Removing
   only the device's self-extent restores the gate-flip verbatim one
   level up. **Sustained** — and it is why the domain half is held.
   **[round-reported]**
2. **"The reservation lives only in crates/planner."** With an
   extent-less table, a `consumed = [0, 1 MiB)` step yields the table
   unreached and `step_constructs = Ok`; with the extent populated the
   table is reached and `own_arm` returns Permitted unconditionally
   (protection.rs:395-408). The route's own Q2 argument applies to its
   own reserved regions. **Sustained**; the recommended act does not
   claim to close the authorization side. **[round-reported]**
3. **"A missing containment edge reopens it."** Nothing validates
   connectivity; omitting the device→table edge places a create at the
   extent-less partition's own `start_offset`. **Sustained**, and it
   sets a landing condition: occupancy must be sourced from the
   **naming fields** (`Partition { parent_table, start_offset }`,
   naming.rs:228-233), which are hashed into the address, not from
   containment edges, which are not. **[round-reported]**
4. **"Folding occupancy into `node_verdict` re-derives the sibling
   capture."** A device's self-extent spans its whole address space,
   so it enters `range_destroyed` on every destructive step; one
   unlocated occupant anywhere then makes the device Indeterminate and
   a wipe of a disjoint ESP refuses. It cannot be narrowed in place —
   `node_verdict` is a pure node function with no step ranges in
   scope. The committed regression asserts only on `affected_set`
   membership (protection_tests.rs:216-229), so it stays green while
   the effect it exists to prevent is re-derived.
   **Sustained, fatal to the domain half.** **[round-reported]**
5. **"It bricks the ADR-0024 repair family."** A `ConflictingTableEntry`
   is a containment child that never carries an extent, and its
   remediation would be impossible by construction. **Sustained**; a
   one-line carve-out repairs it, but it was neither written nor
   measured. **[round-reported]**
6. **"The minor pricing is unargued against INV-004."** INV-004 as
   amended in 12.10.0 specifies free extents as computed "from the
   detected inputs it names (partition extents, device geometry)"
   (AGENT_BUILD_SPEC.md:521); a scheme-derived constant is neither.
   **Sustained as a recording obligation**, not a defect — the ADR
   must argue minor rather than assert it, and carry the major
   counter-argument declined.
7. **"1 MiB of tail is over-reserved."** `PartitionTable { parent,
   role }` carries no geometry (naming.rs:219-224) and no sector size
   reaches `free_extents`. **Not sustained as a defect; sustained as a
   priced judgment** — the ADR must record the reservation as a
   **bound**, never a measurement. The head reservation costs zero
   placements (finding 1: zero is the only reachable sub-1 MiB start).
8. **"`TableState::Absent` still yields offset 0."** A device
   positively attested to have no table reserves nothing.
   **Sustained as a recorded residual** — fails closed on absence,
   open on a positive false attestation.

## Rejected routes, recorded

- **(a) Extent totality.** Rejected on a measured fatal, not on cost:
  key presence is not address-space agreement, and `HostRange` has
  three public fields and no invariant (protection.rs:31-38), so
  zero-length and past-the-end extents were also accepted. Its §11.2
  discharge claim is false as written. **[round-reported]**
- **(c) Refuse at every authorizing arm.** Same fatal, reached
  independently; its seed is itself gated on `facts.extents.get(&id)`,
  so the guard is vacuous exactly when the containment root is
  unlocated — the golden vector's own shape. **[round-reported]**
- **Solver-only, narrow.** A refusal restricted to extent-less
  `Partition` nodes breaks no delivered test and leaves the 1 MiB
  create at offset 0 — it does not fix finding 1. **[round-reported]**
- **Widening containment descent to `source_destroyed`.** Re-breaks
  `a_sibling_esp_is_never_captured`. **[round-reported]**

## Decision carried forward

**Act: (b)'s planner half, alone — and file the anchoring question as
its own governance act.** The head/tail reservation is independent of
the anchoring ambiguity: finding 1's defect exists because the table
carries no extent at all, not because of which space a child uses. It
is small, measured, and lands under existing grants.

Reserve ADR-0036 in its own PR (the f0ef237 → 58c1b87 pattern), land
the spec change, then one WP-060 PR against crates/planner. The ADR
records: reserved head/tail regions derived from the table's hashed
`TableRole`, stated as a **bound** with the sector-size argument;
occupancy sourced from the naming fields, not containment edges
(objection 3 is a landing condition, not a follow-up); the §0.1 minor
argument stated against INV-004 rather than asserted, with the major
counter-argument recorded and declined; and the amendment to
ADR-0018:416-420 replacing an asserted accessor totality with a
computed check.

Evidence obligations: a refusal test per new `SolveRefusal` variant;
placement measured off offset 0 on `solver_fixture`; a **retained
extent-less-table fixture**, so the witness at tests.rs:583 survives
the migration rather than vanishing; mutation-verification before
proposal; WP-060.md regenerated last, bare IDs on Requirements lines,
`xtask ci`'s real exit code checked. First Rust since the r13 pin
(`b50dd19`), so a WP-020 re-pin sitting is owed — plan its economics
before the first merge, which the WP-L100 arc did not.

**Q1: the table gets a different guard, not a required extent** — the
one-range-per-node carriage cannot discharge ADR-0018's own plural
roster (finding 4), and the scheme is already authenticated in the
node's address. **Q2: both layers are required, and only one is
delivered here** — no domain-side fix survived measurement, so #319
stays open on the authorization side and **must not be closed by this
PR**. **Q3: defer** — a required-fact decode rule enforces presence,
which finding 5 proves is not the property; it would be defeated by
the same snapshot that killed (a) and (c).

## Open questions for the decision owner

1. **Which address space does a child inside a partition use?** The
   rule is stated but ambiguous and its delivered citation is dangling
   (finding 6). This is the fatal in (a) and (c) and blocks every
   domain-side reach fix. Picking reading (i) makes the committed
   cross-language golden vector unlawful and regenerating it is a
   versioned act; picking (ii) makes ADR-0018:420 false as written and
   needs the closure repaired instead. **Filed 2026-08-13 as issue
   #333**, with the measurement and the four decisions a fix owes; it
   probably needs its own ADR. Not a §1.11 filing — no requirement
   pair.
2. **Is leaving the authorization side open for one more cycle
   acceptable?** Today a whole-device wipe over a partition-anchored
   pool member constructs (finding 5). Every proposed domain fix was
   measured either fail-open or whole-disk-disabling.
3. **Table extent carriage, which decides the GPT tail** (finding 4):
   plural extents per node, a backup `TableRole` variant, or table
   regions as distinct nodes.
4. **Is `free_extents` INV-004's delivered surface or a PLAN-001
   placement computation?** ADR-0033:133-137 says the former;
   WP-060.md:32 claims the latter. The answer decides minor vs major.
5. **The WP-L100 collision.** Increment 3 is chartered to emit a body
   with no table-state stamp on any path (WP-L100.md:295, :399) while
   a stamp-conditioned guard would refuse stamp-less hosts —
   **[round-reported]**, and worth confirming before increment 3
   starts, since it is the next scheduled work.
