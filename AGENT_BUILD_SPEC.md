# Canonical AI Agent Build Specification

Project: Cross-Platform Disk Partition Manager  
Document role: Normative implementation contract and agent prompt source  
Primary platform: Windows 11  
Additional platforms: Windows 10 ESU/LTSC compatibility, macOS, Debian/Ubuntu, Arch Linux  
Interface: Dark-first desktop GUI plus a scriptable CLI

## 0. Document control

- **Spec version:** 12.7.0
- **Status:** Active normative contract
- **Last updated:** 2026-08-12

### 0.1 Versioning and stability

- This spec uses semantic versioning. Additions bump minor; semantic changes to existing requirements bump major; editorial fixes bump patch. The rule is about requirements, not about whether anything implements them: a semantic change is major even while the product is unbuilt. **3.1.0 was mis-numbered under this rule** — it redefined SAFE-003's identity strength, which is a semantic change to an existing requirement and should have been 4.0.0. It is left as issued, because renumbering an already-published version would break every citation to it, and recorded here so the rule is not read as optional.
- Requirement IDs are permanent. They are never renumbered or reused. A withdrawn requirement keeps its ID, marked **Withdrawn**, with a one-line rationale.
- Every change lands through a pull request labeled `spec-change`, with a changelog entry. Architectural changes additionally require an ADR.

### 0.2 Precedence

When documents disagree, resolution order is:

1. Section 3 safety constraints override everything, including user instructions to an agent.
2. This spec overrides `AGENTS.md` on product behavior and requirements.
3. `AGENTS.md` overrides this spec on repository mechanics (commands, tooling, local conventions).
4. ADRs refine this spec but MUST NOT weaken any MUST.

If two requirements in this spec conflict, agents MUST stop, file a spec issue describing the conflict, and not silently pick a side.

### 0.3 Changelog

| Version | Summary |
| --- | --- |
| 12.7.0 | Resolved SI-22 (ADR-0029): **liveness-scoped retention — bounded and unbounded stop colliding when they stop sharing a population.** Retention MAY reclaim only records of terminal applies; a non-terminal apply's records — `RecoveryRequired` included, the authorization act's record included (ADR-0028's fed-forward fact, absorbed) — are retention-exempt until their apply reaches `Completed`, `Failed`, or `Cancelled`, and the exemption closes over ADR-0027's linkage graph so a running recovery plan pins the terminal records it references, the closure finite because chains are. JRN-004's "bounded size" stays true universally through two mechanisms: terminal history bounded by SEC-009's retention controls, and the live segment bounded by a **per-apply journal budget** whose exhaustion is a journaled failure through Section 8's existing edges — fail-closed toward the writer, never toward the recoverer, which is what turns "bounded by construction" from an assertion into an enforced property (the round's sharpest finding: a journaled retry loop grows without bound unless the budget stops it, and stopping the writer honestly beats blinding the recoverer silently). Reclamation is a declared act: it writes a durable **compaction record** stating the reclaimed range and its authority, so replay classifies every gap — compaction-covered is policy, torn tail is an incomplete write truncated safely (JRN-001's rule, governing the tail while compaction governs the head), and anything else is corruption and refuses. Sequence numbers are never reused or reset across rotation or compaction. The execution journal and the audit log stay distinct: the exemption is the enforced correctness floor, and audit retention beyond it remains SEC-009's explicit user-controlled domain. ADR-0028's revisit condition is discharged by this reconciliation, before either decision ships machinery. Rejected and recorded in the ADR: retention-wins (the filed trap ratified — SAFE-005 turned against the machinery it protects), recovery-wins-transitively-forever (unbounded journal, audit references pinning everything ever written), and a time-capped exemption (re-creates the hazard on exactly the state Section 8 makes unbounded in time). **Minor under §0.1**: JRN-004's sentence stands verbatim and the rule is additions; JRN-001, JRN-003, SEC-009, Section 8, and SAFE-005 untouched. The budget's magnitude and the compaction record's byte encoding land with JRN-006 under WP-070, jointly sequenced; no re-attribution follows — no WP-070 assignment exists, and the ADR records the verification obligations so its creation cannot omit them. |
| 12.6.0 | Resolved SI-21 (ADR-0028): **an authorization act authorizes one apply, and an apply is a journal-continuous execution lifecycle — interruption suspends it, only a terminal state ends it.** The filing read Section 8's three re-entry edges (Paused → Executing, RebootPending → Revalidating per WIN-009, RecoveryRequired → Executing per ADR-0027's roll-forward arm) as reusing an authorization HLP-003 forbids, and it read sharper against ADR-0021's single-use floor act; the question was one definition deep. An apply runs from its act to `Completed`, `Failed`, or `Cancelled`, identified by the plan hash and an unbroken journal chain (JRN-001's monotonic sequence, the torn-tail rule bounding "unbroken") from the act's record to the current position — so resume and roll-forward continue the *same* apply under the *same* journaled, hash-bound act, consumed once at the apply's start and never again. The helper-exit worry dissolves because **the authorization is a journal fact, never process state**: JRN-003 reconstructs, HLP-005's idle exit discards nothing that was ever supposed to persist in a process, and the caching prohibition forbids approvals outliving their apply, not applies outliving interruptions. Freshness has its boundary already: **PLAN-007's window bounds every entry to the apply path** — a re-entry past expiry is rejected exactly as HLP-004 requires and readmitted only through PLAN-007's existing re-approval against a fresh snapshot, a fresh act for the same continuing apply ("one act, one apply" is a ceiling on an act's reach, never a floor on their count). Each re-entry edge keeps its named verification untouched, and authorization continuity never substitutes for revalidation. WIN-009 reads as same-apply continuity, not as a retained grant — the user authorized an apply whose body declared its reboot span. Rejected and recorded in the ADR: re-prompting on every resume (rubber-stamp training on multi-reboot migrations plus new table edges breaking "No other transitions exist"), authorization as retained helper state (contradicts HLP-003 outright), and severity-scaled resume prompting (a second encoding of a dimension the ladder already carries, keyed to the accident of interruption). Sustained and accepted: a recovery stale past its window takes one re-approval — PLAN-007 doing its job; and the authorization record is fed forward to SI-22 as recovery-critical, undecided. **Minor under §0.1**: additions defining a term the ladder used; PLAN-007, HLP-004, HLP-005, WIN-009, Section 8, and JRN-* stand verbatim. No re-attribution follows — no WP-070 assignment exists; the ADR records the verification obligations so its creation cannot omit them. |
| 12.5.0 | Resolved SI-20 (ADR-0027): **the transition table's two RecoveryRequired exits are the two arms, and the table is complete under the reading that splits recovery actions the way the architecture already splits plans.** A roll-forward action continues the *original* plan — same hash, same journal, execution resuming from the last durable checkpoint through the existing → Executing edge, with re-verification inherited from JRN-003's journal-plus-fresh-re-discovery rule rather than added — and is the one recovery act that is not its own plan, stated as a scoping of the prose sentence whose every other instance remains true. Any **distinct** recovery action is its own `OperationPlan` — own draft, validation, authorization, lifecycle — and **selecting it is accepting the original's failure**: the original leaves through the existing → Failed edge with its honest effect summary (`partial`, per journal), the full report, and a **journaled linkage** naming the recovery plan, so "Failed, recovered by plan X" is one reconstructable record chain; one user act may drive both records, which remain two records. **Disposal is durable before the recovery plan may apply** — the JRN-002 shape, and on shared device sets structural rather than procedural, since HLP-005's one-plan-per-bound-device-set already makes the torn state (original undisposed, recovery running) unreachable. No `→ Cancelled` edge is added: Cancelled's unwind semantics belong to the Executing era, and the honest user-initiated terminal after interrupted writes is Failed with its report. No state, edge, or trigger is added anywhere — the rows, the terminal list, and "No other transitions exist" stand verbatim. Rejected and recorded in the ADR: recovery-executes-as-the-original (a different plan's steps under the original's authorized hash, the substitution plan-hash binding exists to forbid), new exits or a `Superseded` terminal (couples two lifecycles or pays a schema state to rename a fact the linkage carries; UI-010 owns humane display, not the state vocabulary), and rewording the Failed row's trigger (retexts a machine-readable table row — major — for what one prose sentence achieves at minor). SI-21's authorization-reuse question is untouched on both edges. **Minor under §0.1**: closing-prose additions only. No re-attribution follows: no WP-070 assignment exists; the ADR records the verification obligations (the two-arm property tests, the disposal ordering, the linkage chain replay, the JRN-003 inheritance on the roll-forward edge) so that assignment's creation cannot omit them. |
| 12.4.0 | Resolved SI-24 (ADR-0026): **CAP-003's "simulation" is the planner's prediction; a PLAN-009 dry run is an apply rehearsal — it runs, and it refuses exactly where apply would.** The conflict turned on one undefined word, and the spec's own vocabulary already split it: PLAN-002 names its output the simulated final topology, PLAN-009 never calls the dry run a simulation, and the glossary defines Preview with no dry-run mention. `preview` therefore licenses exactly the pure surface — PLAN-001 planning and PLAN-002's simulated final topology — and a dry run of a preview-backed plan **runs**, not refused upfront from the client's advisory capability view (that would make the advisory view authoritative, CAP-007's inversion in the refusing direction), terminating at the helper's own recomputed capability gate with a typed CAP-003 refusal: reason pending-qualification, remediation naming the CAP-006 evidence gap, distinguishable by type from every validation-failure class — "your plan is fine, the combination is unqualified" is never conflatable with "your plan is broken." Such a dry run is never *successful*, so PLAN-009's guarantee survives absolute: a successful dry run still means only physical outcomes remain, with **no success-with-caveat class representable**. The pipeline's internal gate order is deliberately not decided — parity is the property, sameness of the dry-run/apply refusal pair is what verification asserts, and the order is WP-070's. Rejected and recorded in the ADR: success-with-carried-caveat (the asterisk that eats the one crisp guarantee dry-run makes), a partial pipeline excluding the capability gate (the second pipeline PLAN-009 exists to forbid), narrowing `preview` to forbid simulation (amputating the capability's value), and upfront client-side refusal (CAP-007's inversion). **Minor under §0.1**: both existing texts stand verbatim — CAP-003 gains the definitional sentence, PLAN-009 the preview-arm sentence — defining an undefined word and an unaddressed case; no existing claim narrows. WP-060's last register gate clears; the decision constrains WP-070's unbuilt pipeline rather than reading it, the SI-19 precedent's class. SI-20, SI-25, and the CAP-006 qualification process stay open. |
| 12.3.0 | Resolved SI-17 (ADR-0025): `irreversible-after-start` is **defined temporally, for the first time** — a step carries it when a reachable interrupted state exists from which the pre-step state cannot be restored by unwinding: once the step's first write lands, stopping cannot go back, and interruption recovery is roll-forward per the journal (Section 8), never unwind. The criterion is a reachable unrestorable intermediate, not the existence of a write — a step whose every interruption resolves to landed-entirely-or-not-at-all does not carry it (the journaled PART-005-shape copy is the unflagged fixture; the in-place multi-sector rewrite the flagged one). The flag therefore claims the **mid-execution window** while severity claims **endpoints** — "fully undoable before or after apply" quantifies over before-first-write and after-completion, the same completed-apply boundary ADR-0022 drew — so **severity 1 with the flag is legal**, the contradiction dissolves, and PLAN-004's declared orthogonality becomes true rather than aspirational. One coupling rule lands with the definition: a flagged step's cancellation claims effect `no-writes` only before its first write; after it, the honest outcomes are `partial` or completion — Section 8's two existing effect values, selected, not extended. Cannot-stop (PLAN-005's `non-cancellable`) and cannot-unwind (this flag) are independent facts in both directions, and the vocabulary now says so. The risk surface needed no new guard: any flag binds the interactive ceremony (ADR-0021), the severity-1 reversal-draft obligation stands (ADR-0022), and UI-005 displays severity and flags as two facts — collapsing them into an inflated severity would repeat the dimension conflation 2.0.0 unwound, and would lie in the other direction. Rejected and recorded in the ADR: permanent illegality (severity inflation or flag suppression — corrupting one vocabulary to avoid defining another), endpoint-irreversibility as the definition (redundant with severity, contradictory by construction, chosen only to manufacture the filed conflict), and dropping the flag (deletes the one word that warns about the interruption window before Apply). **Minor under §0.1**: the flag had no prior definition to change, severity 1's text stands verbatim, and PLAN-005 and Section 8 are untouched. The planner's named combination refusal unlocks, riding the crate's next Rust increment; flag assignment to concrete step families is each building package's testable declaration; SI-24 is WP-060's one remaining register gate. |
| 12.2.0 | Resolved SI-16 (ADR-0024): PART-013 discharges by the helper's authored table state — each of the filing's three options is right somewhere, and the error was choosing one for all cases. On `Present`, the parse-level backup stands untouched: primary and secondary metadata backed up and verified before the first table write, failure → Failed with no writes. On the helper's fresh positively determined `Absent`, the obligation discharges as a **journaled determination** — the backup record is the positively determined absence, a value not a skip (ADR-C4's principle reaching the journal), the same fresh determination PART-001 already requires for initialization, one fact with two consumers, and **no user acknowledgement**, which could only train the rubber stamp on a fact it cannot inform. On `Indeterminate`, ordinary operations stay SAFE-005-disabled before PART-013 is reached — stated so silence does not read as license — while the **typed REC-001 repair family** backs up a **verified raw capture of exactly the regions the plan will write**: for an unsound source the raw bytes are the only truthful backup, a parsed one would launder corruption into a clean-looking artifact, and REC-001's restore-with-identity-validation puts raw bytes back. Capture-impossible (unreadable sectors inside the write region) refuses per Section 8's existing Failed row, with the one exit the MUST-NOT list already carved — the user's separately supported recovery strategy — formalized as a **plan-creation journaled acknowledgement naming the exact uncapturable regions**, the SAFE-003 override shape, never a mid-flight prompt, never available to ordinary operations; the family is a typed step class, not an intent flag, per the safety-is-computed discipline. A blank device and an unreadable one never take the same arm. Rejected and recorded in the ADR: uniform vacuous satisfaction (fail-open on corrupt media, re-conflating what ADR-C4 separated), uniform acknowledgement (ceremony spent where it cannot inform), uniform block (PART-001 unrunnable on its population, the repair family fail-closed against itself — the filing's own reductio). The protection record's journal encoding lands with JRN-006 under WP-070, jointly sequenced; REC-011's corrupt-encryption-header twin stays open for WP-R100 under this shape when designed. **Minor under §0.1**: PART-013's sentence stands verbatim, the arms are additions, and SAFE-005, Section 8's rows, REC-011, and the MUST-NOT clause all read naturally under every arm, untouched. SI-17 and SI-24 stay open. |
| 12.1.0 | Resolved SI-15 (ADR-0023): a PART-009 deviation is **authored, not inherited** — an act the plan performs, never a state it finds. An authored boundary is one whose byte offset the plan sets; every authored boundary meets the 1 MiB default, is placed coincident with a pre-existing structural edge (a neighbor's boundary, the device end — conformant and recorded as coincident, because aligning down instead would mint an unusable sliver), or carries one of PART-009's two existing recorded deviation causes; there is no fourth state. A pre-existing boundary the plan does not move — byte-identical before and after — is an inherited fact: it demands no override, blocks no operation, and the plan records it in its consequence text phrased as a fact about the device, never as a grant by the user. The filed case therefore proceeds: growing a legacy misaligned MBR partition at its tail authors only the new end, which follows policy; the untouched start is inherited; realignment stays available only as an explicit PART-005 move at its honest severity 3, so a grow is never silently a move in either direction — the severity-laundering alternative was rejected as the silent-consequence shape this register has refused every time it has appeared, and the strict reading was rejected as safety theater that fixes no alignment while blocking maintenance on the whole legacy population. Section 11.2's preserved-alignment invariant reads accordingly with no text change: authored boundaries meet policy, inherited boundaries are byte-identical. The solver's named refused case unlocks without the deviation-override vocabulary, which stays deliberately inexpressible for the day a user authors an off-policy boundary on purpose; no typed alignment-fact field is minted (the offsets are already in the bound snapshot — a duplicate field would add only an agreement obligation), recorded as the ADR's revisit condition should a querying consumer ever exist. **Minor under §0.1**: PART-009's two pre-existing sentences stand verbatim, the scoping sentences are additions, and no existing MUST narrows — ADR-0020's precedent shows reading-selection alone amends nothing, and the bump pays for the added text; the major counter-argument (disambiguation as semantic change, the 3.1.0 caution) is recorded in the ADR. SI-16, SI-17, SI-24 stay open, refused conservatively as before. |
| 12.0.0 | Resolved SI-19 (ADR-0022): the reversal is an ordinary `OperationPlan` draft, linked by reference — **`OperationPlan` is not recursive**, the register's named question answered. The filing predated 8.0.0, which dissolved its core: since binding is a validation act for every plan, a reversal emitted at planning time is exactly as unbound as every other draft. What this resolves is carriage, the created-node residue, and truth decay. The reversal draft's planning-time source proposal is the forward plan's simulated final topology; its binding happens at its own validate-plan, after the forward apply, when HLP-002's re-discovery can capture the topology it runs against — the delivered Simulated-never-binds rule is untouched, a prediction proposes and only a helper capture binds. Section 6's body item becomes **reversal linkage**: the draft's plan ID and draft body hash (or PLAN-008's per-step impossibility statement), with the reference asymmetry acyclic by construction — forward names the draft by hash, the draft names the forward plan by ID only, since mutual hash references are unconstructible. Created-node targets (round three's residue: container-minted volumes, LVM snapshots, no positional address until they exist) are typed step-output references, resolved to derived addresses only at the reversal's validation against the helper's own capture per ADR-0019's recompute-at-decode discipline; unresolvable references refuse. Truthfulness is a two-time property: PLAN-008's emission-time judgment, re-checked at the reversal's validation through the draft's own body-content preconditions — the volume-that-gained-data case refuses by precondition rather than silently becoming a destructive plan wearing a reversal's advertisement; the reversal's severity is computed from its own steps and is not bounded by the forward plan's. The regress terminates: a reversal draft's own PLAN-008 field is the machine-readable statement that its reversal is re-application of the forward plan, by plan ID. Applying a reversal is an apply — its own ADR-0021 floor act and, at its severity or flags, its own ceremony; a stale or refused draft is re-planned under PLAN-007's existing rule. Rejected and recorded in the ADR: binding the simulated topology (collides with a delivered mutation-tested structural rule), exemption from binding (the fail-open arm — the only plan class without PLAN-006 protection, at the moment topology is least certain), lazy re-planning with no emission (kills REC-010's advertisement, UI-005's pre-apply display, and severity 1's own definition; survives as the staleness fallback), and recursive embedding (regress under PLAN-008, canonical depth budgets, and a frozen draft whose supersession is the design). **Major under §0.1**: PLAN-008's and Section 6's existing texts change meaning. SI-15, SI-16, SI-17, SI-20, SI-24 and every REC-* behavior stay open; the linkage field's byte encoding lands as the jointly-sequenced WP-060/WP-010 schema change when implemented. |
| 11.2.0 | Resolved SI-18 (ADR-0021): authorization is a two-tier ladder, and SAFE-002 is untouched. Every apply of every plan, at every severity including 0, requires a **floor** authorization — a fresh, explicit act by the RPC-001-authenticated user naming the exact plan hash, single-use (one act, one apply, never a second plan and never a second apply), valid only inside the plan's PLAN-007 window, journaled, never cached, session-wide, or remembered, and satisfiable programmatically, which is what keeps SAFE-003's unattended/scripted-apply population a live surface instead of dead text. The interactive OS-mediated **ceremony** HLP-003 already required at severity ≥ Disruptive stands verbatim, and additionally binds any plan carrying a step flag — the severity-plus-flags participation PLAN-004 promised and HLP-003 never stated; the concrete gap was a LUKS keyslot addition, fully reversible (severity 1) yet `security-sensitive`, which a severity-only ladder would have given the lightest authorization in the product. A flagged plan therefore can never be applied unattended. The enforced tier derives from the helper's own recomputed severity and flags (HLP-002), never from client-claimed values, and **no authorization-requirement field enters the plan** — the register's named question answered: the requirement is a total function of body content the plan already carries, a stored copy would add only an agreement obligation (ADR-0016's lesson reached with no field at all), and WP-040's authorization vocabulary unlocks with no jointly-sequenced WP-010 schema change. SAFE-002's context-1 sentence becomes satisfied at every severity rather than read down — the SI-38 precedence shape: a Section 3 constraint is never bent to fit a lower section — and Section 8's AwaitingAuthorization stays on every apply path with no severity-conditional bypass edge. Rejected and recorded in the ADR: reading SAFE-002 through HLP-003's silence (inverts §0.2, licenses the caching complement, forks the transition table), the ceremony everywhere (rubber-stamps the ceremony where it carries real load and forecloses a population SAFE-003's own text contemplates), and a helper-authored plan-carried authorization field. **Minor under §0.1**: both pre-existing HLP-003 sentences stand verbatim, the floor and flags clauses are additions, and no existing MUST narrows — the reading they foreclose was never licensed, because §0.2 gave SAFE-002 precedence the whole time. |
| 11.1.0 | Resolved SI-27 (ADR-0019), the last naming blocker, on its round four. Node identifiers are derived, kind-discriminated positional addresses — round three's surviving decomposition kept: an address, never a device identity — computed from fields ADR-0018's evidence contract reads, canonicalized by the contract's one named source per platform verbatim (no transformation, so round three's divergence-worse-than-collision hazard is structurally absent), and recomputed at every decode by the schema-validation pass, which rejects unknown referents. Equal derived addresses collapse into counted, flagged, indeterminate **collision groups** that always encode — the governing finding's whole-host unencodability failure cannot occur — with every covered operand `blocked` pairwise under ADR-0018's closure; the group is the representation of the ambiguity Section 2.1/ADR-0011 already declare for equal-identifier pairs, preserves two-ness, and never changes an address (the ancestor-only address property is a committed property test, and a cloned duplicate-designator aggregate re-designates nothing). Section 5 gains `BackingExtent` and `ConflictingTableEntry`; MODEL-002 gains the **host-backing** edge (closing CONC-001's empty loop-device bind set and round three's own-fixtures-collide defect) and the **platform-membership** edge (typing ADR-0011's deferred multipath membership relation without preempting its deferred path-set encoding), each carrying a semantics class per ADR-0018's handover, with the no-sibling-capture theorem re-proved under the extended edge set as a property test. Partitions re-parent onto the table node whose role-discriminated views restore parent-plus-offset injectivity under hybrid tables; conflicting entries materialize verbatim as indeterminate evidence nodes scoped by the closure. The preserved-unknown budgets are fixed (depth 4, 32 KiB, normative truncation-with-digest outcome, versioned redaction). **Minor**: Section 5 and MODEL-002 gain additions, LIN-006's deferred-edge-kind clause gains its promised pointer, and no existing requirement's claim narrows. The gate on increment 3 drops to one item: SI-28. |
| 11.0.0 | Resolved SI-11 (ADR-0018), the register's longest-running direct blocker, with SI-29 and SI-30 resolved within it and SI-37 reclassified. The protection closure is computed, total, and fail-closed: per-node verdicts are three-valued with an `Indeterminate` residual — never `Permitted` by default, round three's fail-open arm inverted and property-tested — computed from a named two-layer helper evidence contract (the helper's own bounded, enumerating, fuzz-obligated parsers over raw device bytes, generalizing ADR-0014's architecture from the table to every on-disk verdict input, plus named per-platform state APIs, joined protectively), which discharges ADR-0016's named-contract hard input. A mutating step's affected set closes over destroyed substrate — downward containment range-bounded, upward backing, downward production — with **release counted as destruction**, so the recorded root-on-ZFS-over-LUKS destruction path and its `vgremove`/`--zero-superblock` siblings refuse, while the no-sibling-capture theorem is a committed property test and creating a partition beside a pool member still constructs. Device scope inverts to a closed positive local-transport list (round three's mutable NVMe-over-TCP counterexample ends); SI-37's unequal-identifier multipath population gets its fail-closed home — per-transport path-multiplicity contracts, unmeasured populations `blocked` — without resolving SI-37, whose dual-path matrix now gates relaxation rather than increment 3. Capability status is computed from canonical steps by the same closure, so CAP-005 agreement holds by construction; source classes are never suppressed (WIN-004's copy-off-LDM survives); PART-014 classification is exhaustive, Regime B, and outside the body. A closed three-entry acknowledgment vocabulary (release, opaque-destruction, identity-bound-restore) replaces both silent permission and forever-refusal, with the consumed-member case deliberately unrepresentable. SI-29: the narrow boundary — file systems inside a Storage Space are ordinary targets within the space's provisioned block interface, health-gated; space/pool geometry and membership are refused (**the major**: it narrows what "protect" claims over space contents). SI-30: deletion-by-containing-erase is severed from sealed-object modification and routed via MAC-009 and the documented-paths clause, an empty-in-v1 step family. The gate on increment 3 drops to two items, both direct: SI-27, SI-28. |
| 10.0.0 | Resolved SI-33 (ADR-0017): the continuity witness is a refusal input, never an assurance. SAFE-003's identity record gains an epoch-token/counter witness field — client-readable and helper-verified like a serial, deliberately not a MODEL-005 authoring-set entry, so the set stays closed at two — scoped to exchange-capable targets on qualified apparatus (one today: the measured Windows counter with the storage-node PDO epoch token, per the reach pattern; absence is the status quo, never a regression). Semantics follow the measurements: comparable only within an unchanged epoch and never on a decrease (a reset the token failed to witness — the adversarial round's finding), movement or incomparability rejecting covered targets under SAFE-003's existing identity-change rule, and `no-exchange-observed` — the liveness ceiling's own words — relaxing nothing, so a stale counter on unmeasured hardware costs exactly the assurance that was never claimed, the fail-closed inversion of the filing's evaluable-but-stale trap. **Major** because an existing requirement's record contents change. The S4-measured undetectable vector — swap between plan and apply on media whose every identifier is identical — becomes a refusal on qualified apparatus. SI-28's floor and Mitigated-open state are untouched; its relaxation route is ADR-0017's named revisit condition requiring apparatus-qualification evidence and its own round. SI-33 becomes hash-visible through placement (the identity record is body content) and its register row is corrected accordingly. Write-path demonstrations join the SI-35/SI-34 obligations on the first write-capable increment. The gate on increment 3 drops to six items, three direct: SI-11, SI-27, SI-28. |
| 9.0.0 | Resolved SI-34 (ADR-0016): the derived protection verdict is hashed-body content, helper-authored at validation — ADR-0014's architecture applied to the second and last field only the helper derives. **Major** because it changes what 8.0.0's closed authoring-set sentence claims; the set holds exactly two named entries and stays closed to creep. The filed options all bridged a two-observer world that 8.0.0 removed: the clamp (a) blinded the helper for an agreement no longer needed, dropping the verdict (b) un-authenticated the value the user most needs bound, and the review's untested option (c) — freshness projection plus monotone floor — dissolves with the second author, its two open dependencies (projection membership, monotonicity proof) having been costs of bridging authors rather than of the safety property. What survives of (c) is its point, by construction: a client cannot weaken the safety decision, because no client claim is representable in a bindable artifact. Divergence within the bound target topology rejects under the existing SAFE-003/PLAN-006 rules — stricter than (c)'s journaled-continue arm, which is foreseen as a possible future relaxation, not foreclosed. The round's sharpest finding transfers to SI-11 as a hard input: the verdict must bind to a named, deterministic helper evidence contract with measured re-probe stability, because the intra-helper interface asymmetry (wipefs enumerates both stale-pair signatures where blkid -p reports exactly the stale one) makes an unnamed evidence set round two's refuted premise returned. The write-path demonstrations land as named obligations on the first write-capable increment, in SI-34's resolution banner beside SI-35's. SI-34 moves to Resolved; the gate on increment 3 drops to seven items, four direct (SI-11, SI-27, SI-28, SI-33); the entry's stale M10-not-taken currency sentences are corrected in the same change, M10 having been taken 2026-08-05 with its readback discharged 2026-08-08. |
| 8.0.0 | Resolved SI-35 (ADR-0014's axis carried to its instrument): the privileged helper is the sole author of ADR-C3's partition-table state, computed from its own raw-sector parser — the only contract the completed measurement campaign found separating (every client projection failed on three platforms, and the privileged `blkid`/`wipefs` probes failed the decisive pair too; M10's raw-byte reads separated it). Four amendments land together, each enumerated in ADR-0014's Consequences before any was drafted: PART-001 gains the categorical invariant (initialize only on the helper's fresh positively determined `Absent` — **the major**, narrowing an existing MUST); MODEL-005's placement rules gain the named third verb, authoring-at-validation, closed to the one field only the helper can derive; Section 6's plan contents bind the validation-produced snapshot hash, making PLAN-006's body-hash equality satisfiable as a theorem rather than an aspiration; and INV-003 states the client-emits-no-table-state consequence in terms. The `Present {checksum}` basis — open since Part 6's round one — is fixed as SHA-256 over copy-invariant content per the new `schemas/table-checksum.md`, so two agreeing copies hash identically from either copy and "both copies agree" and "the checksum" are one fact. The evidence clause's refusal demonstration is discharged at its honest scope and no further: `crates/table-parser` classifies the decisive `gpt-conflicting-tables-512` as `Indeterminate` (ambiguous arm) and `gpt-both-copies-invalid-512` as `Indeterminate` (unreadable arm), both mutation-verified, with the claimed-never-`Absent` line a searched fuzz property; what is *not* demonstrated — an end-to-end refusal by a running write path, which cannot exist before increment 3 — is recorded in SI-35's resolution banner as a named obligation on the first write-capable increment, not rounded up. SI-35 moves to Resolved; the register's gate on increment 3 drops to eight items, five direct; SI-34, SI-27, SI-28, SI-33, and SI-11 are untouched, and the `Present` face question SI-39 parked at SI-35 is resolved for the client by construction (no report, no forbidden report) while the helper's full detection duty stands per ADR-0013. |
| 7.0.0 | Resolved SI-39 (ADR-0015): SAFE-003's blank-can-be-Strong derivation is scoped to the observing contract. The conflict was measured, not read: INV-003 (6.0.0) forbids the unprivileged layer reporting a medium as positively without a table where its contract does not separate that case, and the macOS increment 6 matrix established that `blank-512` and media carrying ext4-with-stale-mdraid, an mdraid member, a LUKS2 container, and an LVM2 orphan project byte-identically — so on macOS no client-derived blank record is positively determined, and SAFE-003's sentence said such a device can be Strong. **The strength rule itself is untouched**: Strong still requires a stable hardware identifier, sizes, both sector sizes, and a positively determined table state, invariantly on every platform — a consumer reading Strong receives the same guarantee everywhere, which is what rejected the reach-relative alternative (option (a); it would weaken the guarantee rather than the population, and ADR-C3 chose the absolute notion deliberately). Only the derived sentence is qualified, contract-relatively rather than platform-named, so a future measured separating client interface restores client-side Strong records with no further amendment. Reportable-`Absent`-under-caveat (option (b)) was rejected as the recorded data-loss path — a macOS client would report `Absent` for a LUKS2 container and PART-001 initializes blank media, the exact report ADR-0013 was written to end. The consequence is accepted knowingly: on macOS, blank media carry Weak identity at plan time, so PART-001 initialization takes typed device-name confirmation (UI-009), an immediate pre-apply re-probe — which M10 measured as separating, so the observer that can see carries the load — and no unattended apply without the recorded override. The plan's claim on such media is "initialize this device, which the client could not distinguish from occupied," never "this medium is blank," so PART-001 stays implementable, routed rather than blocked. ADR-C3's vocabulary and ADR-C4's body-value guard are untouched; the `Present` face of INV-003's sentence stays deliberately with SI-35. **Major under §0.1** because it narrows what an existing requirement's text claims — the class 3.1.0 mis-numbered, not repeated here. SI-39 moves to Resolved; the direct-blocker count drops to six. |
| 6.1.0 | WP-035 gains unprivileged whole-device enumeration and the INV-003 reach declaration, so the read-only CLI can report real attached devices while the spec-issue register decisions proceed. **Minor: an addition.** No requirement in Sections 2, 3, 5, 6 or 7 is retexted — INV-003 is implemented in part, not amended, and 6.0.0 created that obligation with nothing yet implementing it. WP-035's charter sentence survives verbatim and keeps full force over the new scope: raw identifier strings labelled by reporting interface, computing no strength, table state, hash, verdict, or plan. Nothing forbidden becomes permitted; the change adds prohibitions rather than relaxing any. The read-only inventory duty of WP-W100, WP-L100 and WP-M100 is **untouched** — narrowing those rows would remove scope from existing text and would be major, and this package's enumerator is interim and defers to them. M0.5's gate is extended, not rewritten, in the shape 4.2.0 used when it created M0.5 ("an added gate, like any added requirement, which is what makes this a minor bump under 0.1"). The gate deliberately does not require three live adapters: a platform whose access route is an open structural question ships its reach declaration and a typed `not-implemented` answer naming the recorded decision that defers it, so M0.5 — and therefore every sequential milestone after it — is not coupled to that question. |
| 6.0.0 | Resolved SI-38 (ADR-0013): INV-003's detection duty is scoped by privilege, and the unprivileged layer MUST publish the reach of its platform contract. The register filed SI-38 because INV-003 required the discovery layer to detect hybrid and inconsistent partition tables while SAFE-002 places that layer at no elevation, and measurement on all three supported platforms established that it cannot: the enumerated client projections on Linux (2026-08-03), Windows (2026-08-04) and macOS (2026-08-05) do not separate a healthy GPT from one whose two tables describe different partitions, and the macOS privileged leg (M10, 2026-08-05) located the separating fact in the backup table, behind a read SAFE-002 places outside the discovery layer. **This narrows an existing MUST and is therefore major**, not an addition. INV-003's full detection set survives on the privileged path; what changes is that the unprivileged layer is no longer required to do what it measurably cannot, and is instead required to say so. The obvious alternative - detect what you can and report the rest as undetermined - was rejected as unimplementable: the client cannot identify the remainder, a conflicting table presenting as an ordinary valid GPT, so that rule would either never fire or mark every GPT undetermined. SAFE-002 is untouched; qualifying it was rejected because it is a Section 3 constraint and bending it to satisfy a Section 7 functional requirement inverts Section 0.2's precedence order. Where the published reach does not cover a state, the privileged re-discovery HLP-002 already requires before the first write determines it, and the unprivileged layer neither refuses on the ground of its own blindness nor represents that blindness as a determination. SAFE-005 is unchanged and applies to what that re-discovery finds. SI-38 moves to Resolved; SI-35 is unblocked and remains open. |
| 5.0.0 | Replaced Section 4.1's UI line: Svelte and TypeScript instead of React and TypeScript, with SvelteKit excluded and Vite as the build tool (ADR-0010). The decision is taken while `main` carries no UI code and nothing React-specific — verified at acceptance — so no code changes; `packages/canonical` and the design-token schema are framework-independent and unaffected. It approves no desktop shell: Section 4.1 continues to name Tauri 2, which has never been through the gates ADR-0009 applied to Slint, and PR #91's retirement of temporary implementation authority stands. The ten `G-AX-*` accessibility gates remain inconclusive and this change closes none of them. Verification is deferred by the ADR to the point a shell is actually authorized — supply-chain, licence, graph-size-versus-baseline, and attempted accessibility evidence are owed in full before any Svelte code merges; until then this is an intended stack, not a validated one. Major bump under 0.1: a semantic change to an existing requirement is major regardless of implementation state. |
| 4.4.0 | Decided SI-11's axis (ADR-0012), while SI-11 itself stays open. Section 2.1 gains an enforcement-mechanism commitment: a mutating plan step whose target resolves to a Section 2.1 non-goal node is unrepresentable in the plan type — a type error at construction, not a validation failure — with the helper's independent recomputation under HLP-002 retained as an unweakened second layer; for bugs outside the shared verdict computation, a client bug and a helper bug must now coincide before a violating write is even attempted, and the ADR states the two survivals that scoping names — a shared-closure defect is both layers' bug at once, and a protecting fact invisible to unprivileged discovery (a measured input class) lets a plan construct bug-free with the helper as the operative layer. This is the axis the register filed: PART-014's refusal "without an explicit supported plan" is bypassable by construction, Section 0.2 grants override authority only to Section 3, and only unrepresentability survives a bug in the guard. The rejected rounds never adjudicated the axis — round one fell to a PART-014/MAC-009 status-mapping conflict, round two to sibling capture and the SI-27 naming gap, round three to the missing downward production rule, a fail-open residual, and a constructor drifting onto the guard axis — so the axis is fixed deliberately rather than by drift. Deliberately not decided: round three's closure rules remain SI-11's open work. A closure that wrongly computes permitted defeats both axes identically by leaving the node unmarked; a closure that produces no verdict is the fail-closed-residual design space, where the type axis can refuse construction outright — round four's to use. SI-11 therefore remains a direct blocker with an axis-decided state. No existing requirement's text changes; the commitment is an addition, a minor bump under 0.1. Verification lands with the plan type: a construction-refusal proof in the pattern of the CLI chassis's compile-fail non-`Hash` guard, plus a test that the helper refuses a hand-forged artifact bypassing the type layer. |
| 4.3.0 | Resolved SI-12 (ADR-0011): multipath is detection-only in v1. Section 2.1 gains a platform-neutral multipath non-goal entry — the harm SI-12 names exists on every platform, a Windows host reaching a SAN LUN through two HBAs without MPIO being the same case — matching the Section 2.1 backing INV-001's network-block-device precedent already has; LIN-006 carries the Linux mechanics. The inventory carries the platform's own multipath node and its member path devices, connected by the kernel-reported membership relation, whose **edge kind is deliberately left to SI-27's naming round**: round three records that host-assembled devices with no on-disk signature have no legal edge under the surviving taxonomy and need a new edge kind with the no-sibling-capture theorem re-proved, and a multipath node assembled from device-reported WWIDs is that class. The product infers no cross-path device sameness of its own; mutation reports CAP-003 `unsupported` with a multipath reason from CAP-003's reason vocabulary (a closed, versioned enum delivered with WP-050). Two block devices presenting equal stable identifiers with **no platform-assembled multipath node** are SAFE-005 ambiguity, `blocked` — fail-closed without a sameness claim. The retained bridge-synthesis and two-layer-serial measurements establish only that identifier equality cannot be assumed across bridges or observation layers; no retained run measured one LUN on two simultaneous paths with unequal identifiers. That unassembled-and-unequal population remains an unmeasured, uncovered residual filed as SI-37, with its own revisit condition. The path-set encoding — including its body-versus-envelope placement, itself part of what SI-12 left undecided — is deliberately not chosen and lands behind a MODEL-003 schema version bump in the specification change that first makes multipath a supported write target, gated on multipath observability measurements. SAFE-003's text is untouched — its single-connection-path identity record continues to describe every target v1 can bind — and both new texts are additions, a minor bump under 0.1. SI-12 moves to Resolved; its transitive block on SI-27 lifts, with the equal-identifier collision family assigned to SI-27's scope; the direct-blocker set is unchanged. A WP-035-owned follow-up re-attributes the inspect chassis's `same-device-claims` gate from SI-12 to ADR-0011, so no live surface cites a resolved issue as an open gate. |
| 4.2.0 | Added milestone M0.5 (Evidence) and work package WP-035, putting the read-only instrument before the blocked domain model. Every direct blocker on WP-010 increment 3 is gated on measurements nobody has taken — SI-34 and SI-35 carry explicit evidence clauses naming untaken measurements, SI-33 is itself a liveness experiment, and the register's round-four preconditions gate the remaining blockers on established observability rows before any further design round. Rounds two and three, and SI-28's round four, were each rejected for building on unmeasured platform claims, so the measurements precede the model. WP-035 delivers the chassis surface that requires no decision from any register item gating WP-010 increment 3: structured argv, documented exit codes, the `NO_COLOR`/non-TTY/ANSI-free JSON contract, JSON Lines progress, a deny-by-default redaction allowlist, precursor observation records, the dependency doctor, technology-limit facts, redacted diagnostics export, and fixture-backed replay. Its own JSON surfaces — refusal values, progress events, doctor and diagnostics envelopes — carry schema versions per MODEL-003, documented as provisional within major version 0; domain payloads (inventory, topology, capability data) are absent from output entirely, and any request for them refuses with a typed value naming the gate, never merely an exit code or stderr string — so no unversioned JSON is emitted and CLI-001's stable-JSON obligation stays with the packages that own those surfaces. The chassis runs unprivileged; the SI-33 and SI-35 measurements M0.5 also requires are operator-run, read-only experiments recorded in `docs/quality/observability.md` — the SI-28 hardware-confirmation precedent — not repository commands, and the loop-backed portion is gated on repository issue #94. WP-035 is forbidden every surface the register's open items gate: no identity strength, no partition-table state, no typed Section 5 node, snapshot, artifact hash, or plan, no stable device handle, no same-device claim, no protection or per-target capability verdict. No existing row's text changes; under Section 13's sequential rule, M1 and later milestones now additionally gate on M0.5's exit — an added gate, like any added requirement, which is what makes this a minor bump under 0.1. WP-080's dependency row is deliberately untouched and gains its WP-035 dependency in the spec change that unblocks it. The enumeration surface runs unprivileged on Windows, macOS, and Linux. On each platform where an adapter is delivered it reports the host's attached whole devices as interface-labelled raw strings under session-local selectors, with each value's outcome in ADR-C4's vocabulary — a positively determined absence is a value, an unexposed answer is `unavailable`, and a failed read is `failed`; a device list is never empty by default, only by positive determination. Where a platform has no adapter, its answer is the typed `not-implemented` value naming that platform's adapter package, and **that platform names the recorded decision that defers it** — M0.5 does not gate its own exit on an adapter whose access route is an open structural question. The INV-003 reach declaration is published on all three platforms whether or not an adapter is: one answer per state INV-003 lists, derived from the platform contract and not from any device, present with a negative answer where the contract does not separate a state, and citing the observability row that establishes each cell. No enumeration answer carries an identity strength, an ADR-C3 table state or checksum, a typed Section 5 node, an artifact hash, a stable device handle, a same-device claim, a protection verdict, or a CAP-003 status, and the standing gated list still travels in every answer. No Tier-1 test opens a block device or launches a platform enumeration tool: the adapters are exercised through an injected seam over recorded interface payloads. Exiting M0.5 does not close SI-34's evidence list: macOS and real-partitioned-Linux observability rows remain outstanding. |
| 4.1.0 | Added MODEL-006 and ADR-C6, resolving SI-31 before any Section 5 set-valued field is implemented. A schema-declared set encodes as a `pce/1` Array whose elements are strictly ordered by an unsigned lexicographic comparison of each element's complete canonical bytes; equal encodings are rejected rather than deduplicated. Semantic arrays retain their schema-defined order, so `pce/1` and every existing hash remain unchanged. Producer and consumer sort-key encoding inherits the set field's actual enclosing depth instead of resetting to zero, closing the encoder/decoder-symmetry defect SI-31 recorded. The normative algorithm and shared Rust/TypeScript vectors live in `schemas/domain/`, including an extent case that makes bytewise and length-first ordering disagree and exact accepted/rejected depth-boundary cases. |
| 4.0.0 | Fixed the aggregation vocabulary (ADR-C5, resolves SI-07 through SI-10). Section 5 replaces `StorageContainer`, `StoragePool`, and `RaidSet` — three names it listed and never defined — with one `Aggregate` carrying a technology discriminant, adds `BackingSignature`, and renames `Snapshot` to `StorageSnapshot` to end its collision with Section 20's "Snapshot (topology)". MODEL-002 now states how non-linear relationships are modelled: membership has unbounded in-degree, so MAC-003's plural APFS physical stores become representable; an aggregate carries its *self-reported* member count rather than a count of members observed, because deciding from present members would classify a degraded Fusion set as an ordinary mutable container and reach a Section 2.1 MUST NOT by unplugging a cable; Btrfs multi-device is a file system with several backings rather than a container; FS-004's non-file-system signatures materialize as their own nodes so an exported pool or orphaned RAID member is represented rather than discarded (INV-008); and every closed enum over externally observed values carries an unrecognized variant. MODEL-005 gains a body-stability rule narrowing the envelope rule: a hashed body may carry a fact only if it is invariant under re-probe of unchanged hardware. Corrected the Section 20 weak-identity definition, which 3.1.0 left behind when it amended SAFE-003. Noted on ACC-014 that it covers only an *absent* identifier, not an enclosure reporting its own — confirmed on hardware as the more dangerous case and still open. Adversarial review rejected an amendment to MODEL-005's envelope rule that would have swept every descriptive field into the body, reintroducing the unsatisfiable-PLAN-006 failure ADR-C2 exists to prevent; recorded in ADR-C5. |
| 3.1.0 | Identity strength is now a property of a single record (stable hardware identifier plus size, both sector sizes, and a positively determined partition-table state), with identity *matching* split out as a separate helper-side verdict over an ordered pair; partition-table state becomes three-valued so a blank device and an unreadable one are distinguishable (SAFE-003, ADR-C3, resolves SI-01 and SI-02). MODEL-004 provenance becomes a set of observations held in the envelope, with the four confidence values derived rather than stored, and a positively observed absence declared a value rather than an unavailability (ADR-C4, resolves SI-04). Adversarial review rejected two proposals that reached this point: exempting blank-media initialization from severity 4, which would have created a silent whole-device destruction path because an absent table does not mean absent data; and collapsing disputed body values to a single resolution bit, which would have violated SAFE-003's "all available identifiers" and erased the blank-versus-unreadable distinction. Both rejections are recorded in the ADRs. |
| 3.0.0 | Split every hashed artifact into a body and an envelope (MODEL-005), with an explicit rule for which side a field lands on: envelope only for the hash itself and for values the helper independently re-derives under HLP-002, body for everything else. Resolves three contradictions that made version 2.0.0 unimplementable — Section 6 required a plan to contain its own hash; hashing capture metadata and provenance made the PLAN-006 freshness comparison unsatisfiable, since two probes of unchanged hardware always differ; and it was undefined whether provenance sat inside the authorization boundary. Clarified CONC-004 (transitional marking is body content, so a transitional snapshot cannot masquerade as a stable one) and PLAN-006 (comparison is over body hashes). Fixed by ADR-C2. Filed as SI-03, SI-05, and SI-06 in `docs/spec-issues/`. |
| 2.0.0 | Added document control, non-goals, identity-strength policy, helper/RPC/journal/concurrency contracts (HLP/RPC/JRN/CONC), canonical hashing (MODEL-005), plan TTL/reversal/dry-run parity (PLAN-007/008/009), reworked risk model (PLAN-004), full state-transition table, platform support floors, execution-environment requirements (EXE), new functional requirements (INV-009, CAP-007, PART-015/016, FS-010, WIN-011, LIN-010, MAC-009/010, IMG-011, REC-011, UI-013, CLI-008, SEC-010, SAFE-008/009), test-lab architecture, milestone plan, corrected work-package dependencies plus new WPs, required-ADR register, glossary, new acceptance scenarios ACC-011…016. Fixed SAFE-002 self-contradiction, undefined `preview`, CAP-005 helper-trust ambiguity, and the undefined "five primary workflows" in the release gate. See `SPEC_REVIEW_NOTES.md`. |
| 1.0.0 | Initial specification. |

## 1. How agents must use this document

Every implementation agent must:

1. Read this document and the repository `AGENTS.md` before changing code.
2. Work on one explicitly assigned work package at a time.
3. Treat requirement IDs and acceptance criteria as binding.
4. Preserve existing public schemas and interfaces unless the task explicitly authorizes a versioned change.
5. Add or update automated tests for every behavior change.
6. Report exactly which requirement IDs were implemented or affected.
7. Mark unsupported behavior explicitly; never simulate success or add a no-op implementation.
8. Stop if a requested write could touch a host disk, user disk, mounted volume, or non-disposable device.
9. Record every assumption in the pull-request description. Stop and ask instead of acting on an unverifiable assumption.
10. Submit one pull request per work package or assigned subtask. The PR description MUST list the spec version, requirement IDs touched, tests run, and owned paths edited. CI enforces path ownership via CODEOWNERS; do not edit outside owned paths.
11. On discovering a spec conflict or ambiguity, follow Section 0.2.

Normative terms:

- **MUST / MUST NOT:** required for acceptance.
- **SHOULD / SHOULD NOT:** required unless the agent documents a concrete technical reason.
- **MAY:** optional and must not weaken a MUST.

## 2. Prime directive

Build the safest, clearest, and most capable partition manager in its category. The product must combine:

- Complete partition creation, resizing, movement, conversion, cloning, migration, recovery, diagnostics, and rescue workflows.
- Native awareness of Windows, macOS, and Linux storage technologies.
- A polished dark interface that shows the current and planned disk layout before applying changes.
- A strict unprivileged-planning and privileged-execution boundary.
- Honest, machine-specific capability reporting for every operation and file system.

Feature breadth never overrides safety. If an operation cannot be validated and tested, the product must present it as unsupported.

### 2.1 Explicit non-goals (v1 product scope)

The product MUST still detect, correctly represent, and protect everything below; it MUST NOT mutate them. Non-goals prevent scope drift; promoting any item to supported requires a spec change and its own qualified capability.

Enforcement of these MUST NOTs is representational, not merely checked (ADR-0012): a mutating plan step whose target resolves to a node protected by this section is unrepresentable in the plan type — constructing it is a type error, not a validation failure — and the helper's independent recomputation under HLP-002 remains as an unweakened second layer. The closure that decides *which* nodes this section reaches is fixed by ADR-0018 (resolving SI-11): per-node protection verdicts are three-valued and total with an `Indeterminate` residual — never `Permitted` by default — computed from ADR-0018's named two-layer helper evidence contract; a mutating step's affected set closes over the substrate it destroys (downward containment bounded by the destroyed ranges, upward backing, downward production), with a released range counted as destroyed; and a step whose affected set reaches a node this section protects is the unrepresentable sentence, while an `Indeterminate` member refuses construction with a typed artifact. Table writes target the table node's own extents, never the parent device wholesale, which is what keeps a protected member's siblings unconstrained.

- **ZFS:** detect pools and members; never mutate.
- **Windows dynamic disks (LDM):** detect and inspect; migration off dynamic disks only via copy to basic disks; no in-place LDM editing.
- **Windows Storage Spaces:** detect, represent, and protect pools/spaces; no pool or space mutation. The protected objects are the pool, the spaces as structural objects, and the member-disk substrates (ADR-0018, resolving SI-29): a file system inside a space, operated on strictly within the space's already-provisioned block interface through the platform's own supported path, is an ordinary target, health-gated per WIN-003; anything changing the space's own geometry or membership is pool/space mutation.
- **Apple Fusion Drive:** detect only.
- **Apple sealed system volumes and signed system snapshots:** never modified; boot-volume work limited to documented supported paths. Modification or direct deletion of the sealed objects themselves is refused absolutely, with no acknowledgment route (ADR-0018, resolving SI-30); a whole-container erase that reaches them only by destroying their substrate is boot-volume work under this entry's documented-supported-paths clause, gated by MAC-009 — a named, closed step family that is empty in v1, so today every such erase refuses through the closure like any other reached non-goal.
- **Network block devices (iSCSI, NBD, Ceph RBD, etc.):** detect and label; no management.
- **Multipathed attachments (SAN/MPIO/dm-multipath):** detect and represent via the platform's own multipath framework; never mutate a multipath device or a recognized member in v1 (ADR-0011). Two block devices presenting equal stable identifiers with no platform-assembled multipath node are SAFE-005 ambiguity — mutation on either reports `blocked`, refused without asserting the two are one device.
- **Hardware RAID controller configuration and drive firmware updates:** out of scope.
- **File-level backup:** the image engine is not a file-backup product and MUST NOT be presented as one.
- **In-house file-system implementations or resizers:** restated by FS-006.
- **Accounts and cloud services:** core functionality is offline (SEC-007). Telemetry is optional scope; the recommended v1 posture is to ship without telemetry rather than ship SEC-006 partially.

## 3. Non-negotiable safety constraints

### SAFE-001: Disposable media only

Automated tests and agent-driven manual tests MUST use only:

- Regular files containing synthetic disk images.
- Ephemeral virtual disks created inside an isolated VM.
- Loopback devices backed by disposable images inside an isolated test environment.
- Physical devices that are explicitly provisioned and labeled as destructive-test fixtures by the test harness.

Agents MUST NOT run destructive storage commands against the development host, its boot disk, attached user storage, mounted user volumes, or any device selected by inference.

### SAFE-002: No implicit elevation

The GUI, CLI, discovery layer, and default test suites MUST run without elevation. Privileged behavior is confined to exactly two contexts:

1. The platform helper executing a validated plan after fresh, explicit user authorization (HLP-003).
2. Privileged or destructive test suites, which run only inside disposable environments under SAFE-001 and SAFE-007.

No component may auto-elevate, cache an elevation grant across plans, or retain privileged state between plans.

### SAFE-003: Immutable target identity

Every plan that writes storage MUST bind each target to an immutable identity record containing all available identifiers:

- Serial number.
- WWN or equivalent stable hardware identifier.
- OS device instance identifier.
- Connection/location path.
- Total bytes.
- Logical and physical sector size.
- Partition-table type and state, which MUST distinguish `Present` (read and hashed), `Absent` (positively observed to have none), and `Indeterminate` (unreadable or ambiguous). Only the first two are positively determined. A blank device can therefore be Strong **where the observing contract positively determines absence** — the helper's raw read everywhere, and any client contract whose published INV-003 reach separates the absent case. Where a platform's client contract does not separate it, no client-derived record for blank media is positively determined and such records are Weak by this rule's own terms; a plan initializing such a medium (PART-001) claims "initialize this device, which the client could not distinguish from occupied," never "this medium is blank," and travels the weak-identity path below, whose pre-apply re-probe is the separating observation. A device whose table failed to parse cannot be Strong under any contract. `Present`'s checksum is computed over the scheme's **copy-invariant content** as `schemas/table-checksum.md` defines — never over raw header sectors, whose copy-position fields differ between copies by design — so two agreeing copies produce one checksum, from either copy. *(Changed in 3.1.0 by ADR-C3; the blank-can-be-Strong derivation scoped to the observing contract in 7.0.0 by ADR-0015; the checksum basis fixed in 8.0.0 with SI-35's resolution.)*
- Continuity witness, where the target is exchange-capable and the platform's witness apparatus is qualified (ADR-0017): an epoch token plus a media-event counter reading, taken and verified by the helper at validation and re-read at revalidation and before the first write. Readings are comparable only within an unchanged epoch token and never when the value decreased; within-epoch movement or incomparability is an identity change for targets whose other identifiers cannot distinguish exchange, and the plan rejects. The outcome vocabulary is closed — `no-exchange-observed`, `exchange-observed`, `incomparable`, `unavailable` — and a consumer MUST NOT treat `no-exchange-observed` as evidence of continuity: the witness is a refusal input, never an assurance, and it relaxes no confirmation, floor, or policy anywhere in this specification. Where the apparatus is unqualified the field is absent and every existing rule applies unchanged. *(Added in 10.0.0 by ADR-0017, resolving SI-33; SI-28's interim floor is untouched and its relaxation route is that ADR's named revisit condition.)*

The helper MUST reject the plan if identity or topology has changed.

**Identity strength.** Identity strength is a property of a *single* record, computable without a counterpart, so that INV-002 can report it at discovery and Section 6 can carry it in a plan. Each identity record MUST be classified:

- **Strong:** the record carries at least one stable hardware identifier (serial or WWN), together with total size, both logical and physical sector size, and a *positively determined* partition-table state.
- **Weak:** any of the above is missing — most often no stable hardware identifier (common behind USB bridges and SD readers), or a partition-table state that could not be determined.

**Identity match** is a separate verdict over an ordered pair of records, produced only by the helper when it compares a plan's bound record against its own freshly derived one. It is not interchangeable with strength. *(Changed in 3.1.0 by ADR-C3: 2.0.0 defined strength as a comparison outcome while requiring it wherever no counterpart exists.)*

Policy:

- Destructive whole-device operations on weak-identity targets MUST require typed device-name confirmation (UI-009) plus an immediate pre-apply re-probe.
- Unattended or scripted apply against weak-identity targets MUST be refused unless the plan carries an explicit weak-identity override recorded at plan creation.
- For removable devices with strong identity, a changed connection path alone MAY be accepted after replug when every hardware identifier, size, geometry, and table checksum still matches; the acceptance MUST be journaled.

### SAFE-004: No shell strings

Agents MUST NOT implement storage execution by concatenating shell commands. External tools MUST be invoked with structured argument arrays, a fixed executable allow-list, verified executable identity/version, bounded output, timeout behavior, and sanitized environment. Tools MUST be resolved from trusted absolute locations, never from a user-controlled `PATH`, and versions outside the tested range make the dependent capability `blocked` (ACC-009).

### SAFE-005: Fail closed

Unknown file systems, corrupt metadata, ambiguous device identity, missing dependencies, stale topology, unsupported encryption states, and failed backups MUST disable the affected write operation.

### SAFE-006: Secrets

BitLocker, FileVault, LUKS, recovery keys, passphrases, and key files MUST NOT appear in logs, telemetry, crash dumps, plan files, command histories, or UI state snapshots.

### SAFE-007: Host protection in CI

The test runner MUST refuse destructive suites unless a disposable-test token, a verified image/VM target, and an explicit destructive-test profile are all present. A single environment variable is not sufficient proof.

### SAFE-008: Helper isolation

Privileged helpers MUST NOT perform network I/O, load plugins, execute interpreters, or read schemas/configuration from user-writable locations; schemas ship compiled into the helper. Helper updates arrive only through platform packaging (PKG-00x), never self-downloaded.

### SAFE-009: Memory-safety policy

`unsafe` Rust is forbidden (enforced by lint in CI) in the domain, planner, validator, journal, and rpc crates. It is permitted only in adapter, FFI, and helper crates inside reviewed, documented modules. Parsers of on-disk metadata MUST NOT contain `unsafe` and MUST have fuzz targets (Section 11.4).

## 4. Required architecture

### 4.1 Technology

- Core domain, planning, validation, journaling, and image engine: Rust.
- Desktop shell: Tauri 2.
- UI: Svelte and TypeScript. SvelteKit is excluded; the build tool is Vite (ADR-0010).
- Local protocol: versioned, schema-validated RPC over an OS-appropriate local transport (Section 4.5).
- Privileged execution: separate signed native helper per operating system (Section 4.6).
- Linux authorization: polkit.
- macOS authorization: notarized privileged helper and documented Apple authorization mechanisms.
- Windows authorization: signed service/helper with UAC-mediated installation and per-apply consent (HLP-003, ADR-W1).

An agent may propose a change to this stack only through an architecture decision record showing safety, packaging, and cross-platform consequences.

### 4.2 Logical components

| Component | Responsibility | Forbidden responsibility |
| --- | --- | --- |
| Desktop UI | Presentation, user input, plan review, progress | Raw block writes, direct privileged commands |
| CLI | Inventory, planning, dry run, apply submitted plans | Bypassing planner or helper validation |
| Inventory core | Normalize devices, partitions, volumes, file systems, encryption, pools | Mutating storage |
| Capability engine | Report which operations are available and why | Assuming tools or kernel features exist |
| Planner | Produce immutable dependency-ordered plans | Executing steps |
| Validator | Check constraints, preconditions, extent math, risk | Silently correcting ambiguous input |
| Privileged helper | Revalidate and execute approved plans | Accepting arbitrary commands or paths |
| Journal | Durable step/checkpoint/result record | Recording secrets |
| Image engine | Clone, image, verify, resume, damaged-media map | Retargeting destinations |
| Platform adapter | Translate canonical operations to native APIs/tools | Exposing platform-specific behavior as universal |
| Rescue environment | Execute offline plans and repair boot state | Using a different plan or safety model |

### 4.3 Proposed repository layout

```text
apps/
  desktop/
  cli/
crates/
  domain/
  inventory/
  capabilities/
  planner/
  validator/
  journal/
  image-engine/
  rpc/
  adapter-windows/
  adapter-linux/
  adapter-macos/
services/
  helper-windows/
  helper-linux/
  helper-macos/
packages/
  ui/
  design-tokens/
schemas/
tests/
  fixtures/
  model/
  integration/
  fault-injection/
  platform/
docs/
  adr/
  capabilities/
  quality/
  recovery/
  traceability/
packaging/
  windows/
  debian/
  arch/
  macos/
  rescue/
```

Agents MUST avoid circular dependencies. Platform adapters depend on canonical domain interfaces; canonical crates MUST NOT depend on platform adapters.

### 4.4 Concurrency and invalidation

- **CONC-001:** At most one plan may execute against a physical device at a time. An executing plan locks every device it binds for its full execution, including reboot-resumed phases.
- **CONC-002:** Plans queued behind an executing plan MUST be revalidated against post-execution topology before they can be authorized.
- **CONC-003:** External topology changes (hot-plug, third-party tool writes, mount changes) MUST invalidate affected Draft/Validated plans and surface the invalidation in GUI and CLI. Invalidated plans require re-planning; silent rebinding to a new snapshot is prohibited.
- **CONC-004:** Discovery MUST remain read-only and safe to run during execution; snapshots taken while a plan executes MUST be marked transitional and are not valid planning bases. The transitional marking is **body** content under MODEL-005, so a transitional snapshot can never be hash-equal to a stable snapshot of the same topology; capture timestamp and per-property provenance are envelope content, so two stable probes of unchanged hardware do compare equal. *(Clarified in 3.0.0.)*
- **CONC-005:** Multiple clients (GUI and CLI concurrently) MUST observe consistent state through the same helper and journal. When two apply submissions race for the same device, exactly one wins; the loser receives a deterministic, explained rejection.

### 4.5 RPC contract

- **RPC-001:** Transports: Windows — named pipe with an SDDL restricting access to SYSTEM and the authorizing interactive user; Linux — Unix domain socket (0700, root-owned directory) with peer-credential verification; macOS — XPC with code-signing requirement checks, or an equivalently verified Unix domain socket.
- **RPC-002:** Connections begin with a versioned handshake exchanging schema and build versions. Incompatible versions MUST refuse with a remediation message, never degrade silently.
- **RPC-003:** Every message is schema-validated in both directions. The helper side is strict: unknown fields and out-of-range values are rejected.
- **RPC-004:** Messages have bounded sizes and timeouts. Progress/events flow on a stream separate from request/response and are loss-tolerant: clients MUST be able to resynchronize from the journal.
- **RPC-005:** The protocol carries only typed operations defined in `schemas/`. No dynamic code, no path-addressed execution, no raw command passthrough (CLI-004 at the transport layer).
- **RPC-006:** Clients MUST be able to reattach to an in-flight execution after disconnect or crash and reconstruct state from journal plus event replay.

### 4.6 Privileged helper contract

- **HLP-001:** The helper accepts only these operations: status/enumeration queries, validate-plan, apply-plan (by plan hash), cancel, resume, and journal queries. Nothing else exists.
- **HLP-002:** Before the first write, the helper independently re-discovers topology and recomputes capability and validation results. Client-provided discovery, capability, or validation output is an untrusted hint, never an input to authorization.
- **HLP-003:** Every apply of every plan, at every severity (0 included), requires a fresh, explicit **floor** authorization act: performed by the RPC-001-authenticated user, naming the exact plan hash, single-use — one act authorizes one apply of one plan, never a second plan and never a second apply — valid only inside the plan's validity window (PLAN-007), journaled, and never cached, session-wide, or remembered. The act may be programmatic: a scripted apply naming the plan hash is such an act, which is what keeps SAFE-003's unattended-apply population representable; no apply at any severity proceeds from connection standing, cached approval, or session state alone. Every apply of a plan with severity ≥ Disruptive (Section 6, PLAN-004) requires a fresh interactive authorization bound to the exact plan hash: Linux — polkit `auth_admin` without retained grants; macOS — documented authorization APIs with a per-apply prompt; Windows — a fresh administrative consent bound to the plan hash (mechanism fixed by ADR-W1). Cached, session-wide, or remembered approvals MUST NOT exist for these severities. A plan carrying any step flag (PLAN-004) requires this same fresh interactive authorization regardless of severity — the severity-plus-flags rule PLAN-004 states — so a flagged plan can never be applied unattended. The required tier derives from the helper's own recomputed severity and flags under HLP-002, never from client-claimed values, and no plan field carries an authorization requirement: the requirement is a total function of the plan body's severity and flags, and a client-assertable authorization is unrepresentable (CAP-007). Where the UI needs the tier for its authorization-wait display (UI-011), the helper reports its computed tier in the validate-plan response — response data, never plan body. *(Amended in 11.2.0 by ADR-0021, resolving SI-18: the floor tier, the flags trigger, and the tier's helper-side derivation are added; the two pre-existing sentences — the Disruptive threshold and its caching prohibition — stand verbatim, with the floor's own never-cached clause covering the severities they do not reach.)* An authorization act authorizes one **apply**, and an apply is a journal-continuous execution lifecycle: it runs from its act to a terminal state, identified by the plan hash and an unbroken journal chain (JRN-001's monotonic sequence, the torn-tail rule bounding "unbroken") from the act's record to the current position. Interruption — a pause, a declared reboot, a recovery stop — suspends an apply; only `Completed`, `Failed`, or `Cancelled` ends it. A Section 8 re-entry within the plan's validity window (PLAN-007) continues the same apply under the same journaled act, consumed once at the apply's start; a re-entry past the window is rejected (HLP-004) and readmitted only through PLAN-007's re-approval against a fresh snapshot — a fresh act for the same continuing apply, since one-act-one-apply is a ceiling on an act's reach, never a floor on their count. The authorization is a journal fact, never process state: a helper restart holds nothing the journal does not (JRN-003, HLP-005), and the caching prohibition above forbids approvals outliving their apply, not applies outliving interruptions. Each re-entry edge keeps its named verification, and authorization continuity never substitutes for revalidation. *(Amended in 12.6.0 by ADR-0028, resolving SI-21: the apply-lifecycle definition is added; every pre-existing sentence stands verbatim.)*
- **HLP-004:** The helper enforces plan validity windows (PLAN-007) and snapshot-hash freshness (PLAN-006), rejecting expired or stale plans.
- **HLP-005:** The helper executes at most one plan per bound device set (CONC-001), idles locked-down when no work exists, and MAY exit when idle.
- **HLP-006:** Helper logging is structured, redacted per SAFE-006, and appended to the audit log (SEC-009).
- **HLP-007:** The helper performs no work on behalf of non-local or cross-session callers (SEC-002) and verifies caller identity via RPC-001 before processing any request.

### 4.7 Journal contract

- **JRN-001:** The journal is append-only with per-record checksums and monotonic sequence numbers. A torn tail MUST be detected and safely truncated during recovery.
- **JRN-002:** Journal records for a state transition or checkpoint MUST be durable (fsync or platform equivalent) before the corresponding storage write begins.
- **JRN-003:** Replay is idempotent. Recovery state derives solely from the journal plus fresh re-discovery, never from UI or client memory (Section 8).
- **JRN-004:** Journals live in an admin/root-protected, documented location per OS, with bounded size and the retention controls of SEC-009. Retention is **liveness-scoped** (ADR-0029, resolving SI-22): it MAY reclaim only records of terminal applies. Records belonging to a non-terminal apply — the authorization act's record included (ADR-0028) — are retention-exempt until their apply reaches a terminal state, and the exemption closes over disposal linkage (ADR-0027): a terminal record referenced by a non-terminal apply's linkage is exempt until the referencing apply terminates. The live segment's bound is enforced as a per-apply journal budget whose exhaustion is a journaled failure through Section 8's existing edges — never a reclamation of live records. Reclamation writes a durable compaction record stating the reclaimed range and its authority; sequence numbers are never reused or reset across rotation or compaction; and replay classifies every gap — compaction-covered is policy, a torn tail truncates per JRN-001, and any other gap is corruption and refuses. The exemption is the enforced correctness floor; audit-log retention beyond it remains SEC-009's user-controlled domain. *(Amended in 12.7.0 by ADR-0029; the first sentence stands verbatim.)*
- **JRN-005:** Journals never contain secrets (SAFE-006); embedded tool output is bounded and redacted.
- **JRN-006:** The journal format is a versioned public schema under MODEL-003.

## 5. Canonical domain model

The domain crate MUST define serializable, versioned types for:

- `PhysicalDevice`
- `DeviceIdentity`
- `IdentityStrength`
- `DeviceHealth`
- `PartitionTable`
- `Partition`
- `Aggregate`
- `BackingSignature`
- `Volume`
- `FileSystem`
- `EncryptionLayer`
- `StorageSnapshot`
- `Mount`
- `FreeExtent`
- `BackingExtent`
- `ConflictingTableEntry`
- `Capability`
- `TopologySnapshot`
- `OperationRequest`
- `OperationPlan`
- `PlanStep`
- `PlanRisk`
- `ExecutionJournal`
- `ExecutionResult`
- `RecoveryAction`

`Aggregate` replaces `StorageContainer`, `StoragePool`, and `RaidSet`, and
`StorageSnapshot` replaces `Snapshot`. `BackingSignature` is new. *(Changed in
4.0.0 by ADR-C5: the three replaced names were listed and never defined, so no
requirement supplied a boundary between them, and the one-to-one shape "container"
implies cannot express MAC-003's plural APFS physical stores. `Snapshot` collided
with Section 20's "Snapshot (topology)".)*

`BackingExtent` and `ConflictingTableEntry` are added in 11.1.0 by ADR-0019: a
`BackingExtent` is the file or byte range within a host that carries a
host-backed virtual device's bytes (loop, dm-linear, plain dm-crypt, VHD/VHDX,
attached images), and a `ConflictingTableEntry` holds a partition-table entry
that aliases or contradicts across table views, verbatim and marked
indeterminate (INV-008, REC-003). Node identifiers are derived positional
addresses per ADR-0019's naming maps — never stored — and same-kind nodes
deriving equal addresses collapse, before encoding, into a counted, flagged,
indeterminate collision group whose operands are `blocked`, so a snapshot body
always encodes regardless of on-disk content.

### MODEL-001: Units

All offsets and sizes MUST use unsigned byte counts in the canonical model. Sector counts MAY be included as derived platform data. UI values MUST preserve exact byte values while displaying IEC units by default.

### MODEL-002: Layering

The model MUST distinguish:

`physical device → partition table → partition → encryption/container → volume → file system → mount`

It MUST also represent non-linear relationships such as Storage Spaces, LVM, RAID, APFS containers, and Btrfs multi-device file systems. These are modelled as follows. *(Added in 4.0.0 by ADR-C5.)*

- **One `Aggregate` type**, carrying a closed technology discriminant, expresses every aggregation. Membership is an edge from a `BackingSignature` to its consumer with **unbounded in-degree**, which is what makes MAC-003's plural APFS physical stores and MAC-010's two-store Fusion container representable at all.
- An `Aggregate` MUST carry the aggregate's **self-reported member count**, not a count of members currently observed. A Fusion set with one store detached presents one present store; deciding from present members would classify it as an ordinary mutable container and reach a Section 2.1 MUST NOT by unplugging a cable.
- **Btrfs multi-device is a `FileSystem` with an ordered set of n ≥ 1 backings**, not a container. Single-device is the cardinality-1 instance of that same shape, so `btrfs device add` changes the member set and not the node's shape.
- **FS-004's non-file-system signatures are materialized as `BackingSignature` nodes** on the layer this chain assigns, never enumerated into the file-system kind. The consumer edge is optional: an observed signature whose aggregate is not observed — an exported ZFS pool, an unassembled mdraid member, an offline Storage Spaces disk — is represented rather than discarded (INV-008).
- Every closed enum over externally observed values MUST carry an explicit unrecognized variant, or INV-008 becomes unsatisfiable the first time a platform ships a value the product does not know.
- **Two further edge kinds, each carrying a semantics class** *(added in 11.1.0 by ADR-0019, per ADR-0018's handover)*: the **host-backing** edge ("the bytes of A live within B") from a `BackingExtent` to the virtual device it backs, traversed by CONC-001's bind set so a plan writing to a host-backed device binds through to the physical device beneath it; and the **platform-membership** edge (platform-asserted composition, detection-only) from a platform-assembled multipath node to its counted member representation — closure-inert and bind-inert while multipath is detection-only, its activation landing with the spec change ADR-0011 names. Neither edge targets a physical device, and the no-sibling-capture theorem is re-proved under the extended edge set as a property test.

### MODEL-003: Stable serialization

Public plans, topology snapshots, journals, and CLI JSON MUST include a schema version. Breaking changes require a new version and migration or explicit rejection.

### MODEL-004: Provenance

Every discovered property MUST record, in the artifact **envelope** (MODEL-005), the set of observations that produced it. Each observation MUST name its source adapter and adapter version, the method used, and an outcome: the value observed, an unavailability reason, or a read error.

The confidence values `authoritative`, `inferred`, `unavailable`, and `conflicting` are **derived** from that set and MUST NOT be stored independently of it, so that no record can assert a confidence its observations contradict.

A positively observed *absence* is a value, not an unavailability. `unavailable` means the adapter could not determine the property; it does not mean the property was determined to be absent. Conflating the two collapses a blank device and an unreadable one into the same record, which PART-001 would then initialize alike. *(Changed in 3.1.0 by ADR-C4: 2.0.0 named a single source adapter while permitting a `conflicting` value that presupposes two.)*

### MODEL-005: Canonical encoding and hashing

Every hashed artifact (plans, topology snapshots) MUST be split into a **body** and an **envelope**. The body MUST have exactly one canonical byte encoding, defined in `schemas/` (deterministic field ordering, no ambiguous numeric encodings). The artifact hash is SHA-256 over the canonical bytes of the **body**. All components — Rust and TypeScript — MUST produce identical hashes for identical logical body content, proven by cross-language golden tests. The encoding choice is fixed by ADR-C1; the body/envelope boundary is fixed by ADR-C2.

**Envelope rule.** A field belongs in the envelope only if it is the hash itself, or the privileged helper independently re-derives it and treats the client's copy as an untrusted hint (HLP-002). Every other field belongs in the body.

Enforcing a value is not re-deriving it. The helper enforces the validity window (HLP-004) rather than recomputing it, so the window is body content; an unauthenticated expiry could be extended without invalidating the authorization it was bound to. When a field's side is unclear, it belongs in the body: an envelope field is one an attacker may alter without breaking a hash. **Authoring at validation is a third, narrower verb** (added in 8.0.0 per ADR-0014; extended in 9.0.0 per ADR-0016): a body field only the privileged helper can derive is stamped by the helper during validate-plan, before HLP-003 binds authorization to the resulting hash, and recomputed at revalidation and before the first write. The authoring set is closed and named here, at exactly two entries — **partition-table state** (ADR-0014) and **the derived protection verdict** (ADR-0016, authored from a named helper evidence contract that SI-11's shape round must fix) — so "the helper writes some body fields" cannot creep by analogy; a client-authored value for either field never validates, and any within-target divergence between the stamped and recomputed value rejects under SAFE-003/PLAN-006's existing rules. *(Added in 3.0.0. A plan cannot contain its own hash, and hashing capture metadata would make the PLAN-006 freshness comparison unsatisfiable, because two probes of unchanged hardware always differ.)*

**Body-stability rule.** A hashed body MAY carry a fact only if that fact is invariant under re-probe of unchanged hardware. This *narrows* the envelope rule rather than replacing it — the envelope rule decides what is authenticated, this one decides whether PLAN-006 is satisfiable, and both MUST hold. Occupancy figures, mount sets, and storage-snapshot sets therefore belong to the envelope: they change without any storage change, through ordinary background activity. A fact that a verdict needs but that fails this rule is evidence the wrong fact was chosen, not grounds to relax the rule. *(Added in 4.0.0 by ADR-C5.)*

### MODEL-006: Canonical collection semantics

Every schema field declared to be a **set** MUST encode as a `pce/1` Array whose
elements are strictly ascending under an unsigned lexicographic comparison of
each element's complete canonical bytes. Equal encodings are duplicates and
MUST be rejected rather than silently deduplicated. A semantic array is not a
set and MUST retain its schema-defined order; this requirement does not change
the `pce/1` profile.

Both producer and consumer MUST compute an element's canonical bytes using the
set field's actual position in the enclosing artifact. If the set Array is at
depth `d`, element encoding starts at `d + 1` and consumes the remaining depth
budget from there; it MUST NOT reset to the public standalone encoder's depth
zero. The schema validation pass, not the generic `pce/1` decoder, owns
misordered and duplicate-set errors and MUST reject rather than repair them.

The normative algorithm and shared cross-language vectors are in
`schemas/domain/canonical-collections.md` and
`schemas/domain/canonical-set-vectors.json`. The decision is fixed by ADR-C6.
*(Added in 4.1.0 by ADR-C6; resolves SI-31.)*

## 6. Operation-plan contract

An `OperationPlan` MUST consist of a hashed **body** and an **envelope** (MODEL-005).

The **body** MUST contain:

- Plan ID, schema version, creation timestamp, and application version.
- Source topology snapshot body hash, as bound at validation: the client's draft snapshot is a proposal, and the snapshot whose hash the authorized plan binds is the one HLP-002's re-discovery produces during validate-plan. *(Clarified in 8.0.0 per ADR-0014; PLAN-006's body-hash equality is unsatisfiable for any body field the client cannot derive, which the SI-35 measurements made a theorem rather than a scruple.)*
- Complete bound device identities with identity strength (SAFE-003).
- Requested outcome.
- Ordered dependency graph of plan steps.
- Step preconditions and postconditions.
- Risk classification and user-facing consequence text.
- Required privileges.
- Online, offline, reboot, and rescue requirements.
- Estimated affected byte ranges.
- Backup and recovery actions.
- Cancellation behavior for every step.
- Expected capability/dependency versions.
- Validity window (PLAN-007). Body content deliberately: the helper enforces the window rather than re-deriving it, so an unauthenticated expiry could be extended without invalidating the authorization bound to the plan.
- Reversal linkage (PLAN-008): the emitted reversal draft's plan ID and draft body hash, or the per-step reversal-impossibility statement. The draft is carried by reference deliberately — `OperationPlan` is not recursive — and the reference asymmetry is acyclic by construction: the forward body names the reversal draft by hash, while the reversal draft names the forward plan by plan ID only, because mutual hash references are unconstructible. *(Changed in 12.0.0 by ADR-0022, resolving SI-19; was "Reversal plan or reversal-impossibility statement".)*

The **envelope** MUST contain:

- The cryptographic plan hash (MODEL-005). It is the hash *of* the body and therefore cannot be *inside* it.
- Discovery provenance and adapter attribution for values the helper re-derives (MODEL-004, HLP-002).

Nothing else belongs in the envelope. *(Changed in 3.0.0: version 2.0.0 required the plan to contain its own hash, which is not constructible.)*

### PLAN-001: Pure planning

Planning MUST be deterministic and side-effect free for the same snapshot, capabilities, and request.

### PLAN-002: Before and after

Every valid plan MUST produce both the original topology and a simulated final topology.

### PLAN-003: Dependency graph

Steps MUST form an acyclic graph. The planner MUST reject conflicting or impossible steps and explain the conflict.

### PLAN-004: Risk model

Each step carries a **severity** on one ordinal scale:

0. Informational — no change to storage.
1. Reversible — fully undoable before or after apply via an emitted reversal plan.
2. Disruptive — interrupts service (unmount, lock, reboot) with no expected data loss.
3. Data-moving — data is relocated or transformed; loss is possible on failure.
4. Destructive — data is intentionally destroyed.

Each step additionally carries orthogonal **flags**: `security-sensitive` (touches encryption, keys, or authorization state), `irreversible-after-start`, `requires-offline`, `requires-reboot`, `requires-rescue`.

**`irreversible-after-start`, defined.** A step carries this flag when a reachable interrupted state exists from which the pre-step state cannot be restored by unwinding: once the step's first write lands, stopping cannot go back, and interruption recovery is roll-forward per the journal (Section 8), never unwind. The criterion is a reachable unrestorable intermediate, not the existence of a write — a step whose every interruption resolves to landed-entirely-or-not-at-all does not carry it. The flag is a claim about the mid-execution window only; severity claims endpoints ("fully undoable before or after apply" quantifies over before-first-write and after-completion, PLAN-008's completed-apply boundary), so **severity 1 with this flag is legal**: endpoints fully undoable via the emitted reversal draft, mid-window roll-forward-only. One coupling rule: a flagged step's cancellation MAY claim effect `no-writes` only before its first write; after it, the honest outcomes are `partial` or completion (Section 8's existing effect values, selected, not extended). Cannot-stop (PLAN-005's `non-cancellable`) and cannot-unwind (this flag) are independent facts in both directions. *(Added in 12.3.0 by ADR-0025, resolving SI-17; the flag list above and severity 1's definition stand verbatim.)*

Plan severity is the maximum step severity; plan flags are the union of step flags. UI consequence text, confirmation strength (UI-009), and authorization requirements (HLP-003) key off severity plus flags. *(Changed in 2.0.0: v1's five ordinal classes conflated severity with the security-sensitive dimension.)*

### PLAN-005: Cancellation

Each step MUST declare one of: cancellable, checkpoint-cancellable, or non-cancellable. The UI MUST not offer cancellation when the current step cannot safely stop.

### PLAN-006: Stale-plan rejection

The helper MUST re-discover target topology and reject a mismatch before the first write and at declared revalidation checkpoints. The comparison is over **body** hashes (MODEL-005): capture metadata and provenance are envelope content precisely so that an unchanged topology re-probes to an equal hash and this check is satisfiable. *(Clarified in 3.0.0.)*

### PLAN-007: Plan validity window

Every plan carries an explicit expiry: default 24 hours, maximum 7 days. The helper rejects expired plans (HLP-004). Re-approval after expiry requires re-validation against a fresh snapshot.

### PLAN-008: Reversal plans

For every plan, the planner MUST either emit a reversal plan (only where reversal is truthful, e.g., metadata-only changes) or a machine-readable, per-step statement of why reversal is impossible. This output feeds REC-010: rollback may be advertised only where a reversal plan exists.

The emitted reversal is an ordinary `OperationPlan` draft, exempt from no plan rule (ADR-0022, resolving SI-19):

- Its planning-time source-snapshot proposal is the forward plan's simulated final topology; its **binding** happens at its own validate-plan, after the forward apply, when HLP-002's re-discovery can capture the topology it runs against. A simulated snapshot proposes and never binds — it can never satisfy a PLAN-006 comparison. The reversal reverses a completed apply; mid-apply failure recovery is Section 8's, not this requirement's.
- Where a reversal step targets a node the forward plan creates, the draft MUST spell the target as a typed reference to the creating step's output, never as an address; the reference resolves to a derived address only at the reversal's validation, against the helper's own capture, and an unresolvable reference refuses.
- Truthfulness is a two-time property: judged at emission, and re-checked at the reversal's validation through the draft's own preconditions, which are body content. A reversal whose preconditions fail MUST refuse rather than be reclassified silently; its severity is computed from its own steps (PLAN-004) and is not bounded by the forward plan's.
- A reversal draft's own reversal field is the machine-readable statement that its reversal is re-application of the forward plan, named by plan ID — a reference, not a third plan.
- Applying a reversal is an apply: it takes its own HLP-003 authorization at its own severity and flags. A draft past its validity window (PLAN-007) or refused at validation is re-planned against a fresh capture.

*(Amended in 12.0.0 by ADR-0022: the first paragraph stands verbatim; the draft/binding architecture, step-output references, two-time truthfulness, and the regress statement are stated in terms.)*

### PLAN-009: Dry-run parity

Dry run MUST traverse the identical pipeline as a real apply — including helper revalidation (HLP-002) — and stop before the Protecting state. A successful dry run means the only remaining variables are physical execution outcomes, not validation surprises.

A dry run of a preview-backed plan (CAP-003) runs — it is not refused upfront from the client's advisory capability view (CAP-007) — and terminates at the helper's own recomputed capability gate with a typed CAP-003 refusal naming the qualification gap and its CAP-006 remediation, distinguishable by type from every validation-failure class. Such a dry run is never successful, so the success guarantee above stands absolute: no success-with-caveat outcome is representable. The dry run refuses exactly where and how apply would; the pipeline's internal gate order is the implementation's, and sameness of the refusal pair is the tested property. *(Added in 12.4.0 by ADR-0026, resolving SI-24; the two pre-existing sentences stand verbatim.)*
## 7. Functional requirements

### 7.1 Inventory and topology

- **INV-001:** Discover internal and external HDD, SATA SSD, NVMe, eMMC, USB mass storage, SD media, virtual disks, hardware RAID LUNs, and loop devices. Network block devices are represented detection-only (Section 2.1).
- **INV-002:** Report model, vendor, transport, removable status, capacity, sector sizes, read-only status, system/boot role, and stable identity with identity strength (SAFE-003).
- **INV-003:** Detect GPT, MBR, Apple Partition Map, missing tables, hybrid/inconsistent tables, and corrupt metadata. **Detection is scoped by privilege** (ADR-0013), because these states are not all reachable from the unprivileged discovery layer SAFE-002 requires:
  - Unprivileged discovery MUST detect every state its platform contract can distinguish, and MUST NOT report a state its contract cannot reach. Reporting a table as consistent, or a medium as positively without a table, is such a report where the contract does not separate that case.
  - The privileged path MUST detect the full set above when it runs inside a context SAFE-002 permits.
  - Unprivileged discovery MUST publish the reach of its platform contract: for each state above, whether that contract can distinguish it on this platform. This is a property of the contract and the platform. It is declared independently of any device, MUST NOT be derived per-device, and MUST NOT be omitted when the answer is "no".
  - The unprivileged layer emits no partition-table state on any platform (8.0.0, ADR-0014): `Present` requires reading and hashing table bytes the client is denied everywhere measured, `Absent` is unreachable where the contract does not separate it (7.0.0), and `Indeterminate` is a determination about the medium a client that read no table bytes may not assert. Client surfaces carry raw observations and the published reach; the state is authored by the privileged helper from its own raw-sector parser, whose classification and checksums `schemas/table-checksum.md` and `crates/table-parser` define.
  - A consumer MUST NOT treat an unprivileged inventory as evidence that an undeclared state is absent. Where the published reach does not cover a state, that state is determined by the privileged re-discovery HLP-002 already requires before the first write, and the privileged determination governs. An unprivileged layer MUST NOT refuse a write on the ground that its own contract cannot reach a state, and MUST NOT represent its inability as a determination either way. SAFE-005 applies as it always has, to what that re-discovery finds.
- **INV-004:** Detect partitions, free extents, alignment, partition types/flags, labels, UUIDs, volumes, file systems, encryption, mounts, and nested storage.
- **INV-005:** React to device attach, removal, mount, unmount, unlock, pool, and topology changes, driving plan invalidation per CONC-003.
- **INV-006:** Never auto-mount unknown or damaged media solely to inspect it, and never run repair tools during discovery.
- **INV-007:** Expose raw discovery evidence in a redacted diagnostic view.
- **INV-008:** Represent unsupported structures without flattening or discarding them.
- **INV-009:** Remain correct under rapid attach/detach churn: event storms MUST coalesce without dropping the terminal device state (stress-tested per Section 11.5).

### 7.2 Capability engine

- **CAP-001:** Calculate capabilities per exact device, partition, file system, encryption state, mount state, OS, dependency version, and boot role.
- **CAP-002:** Model detect, read, create, grow, shrink, move, copy, check, repair, label, UUID, encrypt, decrypt, and wipe separately.
- **CAP-003:** Return `supported`, `preview`, `unsupported`, or `blocked`, plus a reason and remediation. **Definitions:** `supported` — apply permitted, backed by matrix evidence (CAP-006); `preview` — planning and simulation permitted, apply refused pending qualification evidence, labeled as such in GUI and CLI; `unsupported` — the product does not implement the operation for this target; `blocked` — implemented, but a runtime precondition fails (missing tool, version, state). "Planning and simulation" means the pure planner surface: PLAN-001 planning and PLAN-002's simulated final topology. A PLAN-009 dry run is not a simulation — it is a rehearsal of the apply pipeline and belongs to the apply surface `preview` refuses; its behavior on a preview-backed plan is PLAN-009's to state. *(Definitional sentence added in 12.4.0 by ADR-0026, resolving SI-24; every pre-existing sentence stands verbatim.)*
- **CAP-004:** Confirm required native API/tool availability and version at runtime.
- **CAP-005:** Serve GUI, CLI, and planner from one capability engine so surfaces never disagree.
- **CAP-006:** Store tested capability fixtures for every advertised platform/file-system combination.
- **CAP-007:** Capability results shown by clients are advisory UX. The helper trusts only its own recomputation (HLP-002); a client cannot upgrade a capability by asserting it.

### 7.3 Core partition operations

- **PART-001:** Initialize blank media as GPT or MBR. Initialization proceeds only on the executing helper's own fresh, positively determined `Absent` at apply time — never on a plan-carried claim, never on media whose state is `Indeterminate` or undetermined, and never on the ground that a client could not distinguish the medium from blank. The plan's claim on unseparated media is "initialize this device, which the client could not distinguish from occupied" (SAFE-003, 7.0.0); the helper's determination is what makes it blank. *(Categorical invariant added in 8.0.0 with SI-35's resolution, per ADR-0014 and ADR-0015.)*
- **PART-002:** Create primary, logical, extended, and GPT partitions where applicable.
- **PART-003:** Delete partitions with protected-system-partition safeguards.
- **PART-004:** Grow and shrink partitions and their file systems in the safe order required by the direction of change.
- **PART-005:** Move partitions while preserving data and checking boot consequences. Moves MUST be implemented copy-then-commit where extents do not overlap, and via journaled chunk copy with a durable progress map where they do, so interruption never leaves an unrecoverable mapping (Section 11 invariants).
- **PART-006:** Copy partitions to compatible free extents or target devices.
- **PART-007:** Split and merge compatible partitions without claiming support for unsupported layouts.
- **PART-008:** Change label, partition name, type, flags, drive letter, mount point, UUID, and allocation-unit size where supported.
- **PART-009:** Align partitions to 1 MiB boundaries by default. Deviations occur only when the device's published geometry requires different alignment or the user explicitly overrides; both are recorded in the plan. A deviation is **authored**: a boundary whose byte offset the plan sets, placed off the default. A pre-existing boundary the plan does not move — byte-identical before and after — is an **inherited fact**, not a deviation: it demands no override, blocks no operation, and the plan MUST record it in its consequence text as a fact about the device, never as a grant by the user. An authored boundary placed coincident with a pre-existing structural edge (a neighbor's boundary, the device end) conforms to policy and is recorded as coincident. Every authored boundary therefore meets the default, is coincident with a named pre-existing edge, or carries one of the two recorded deviation causes — there is no fourth state. Section 11.2's preserved-alignment invariant reads accordingly: authored boundaries meet policy; inherited boundaries are byte-identical before and after. *(Amended in 12.1.0 by ADR-0023, resolving SI-15: the two pre-existing sentences stand verbatim; the authored/inherited scoping, the coincident-edge rule, and the inherited-fact recording obligation are additions.)*
- **PART-010:** Convert MBR to GPT and GPT to MBR only when all structural and boot requirements pass.
- **PART-011:** Provide clone-and-reformat migration when lossless in-place file-system conversion is unavailable.
- **PART-012:** Queue multiple operations and compute their combined final layout before apply.
- **PART-013:** Back up primary and secondary GPT or MBR/EBR metadata before the first table write. The obligation discharges by the helper's authored table state (ADR-0014), each arm journaled — no arm is silent: on `Present`, the parse-level backup above, verified, with failure → Failed (Section 8). On the helper's fresh positively determined `Absent` — the same determination PART-001 requires — the backup record is the journaled determination itself, a value rather than a skip (ADR-C4), demanding no user acknowledgement. On `Indeterminate`, ordinary operations remain disabled by SAFE-005 before this requirement is reached; a step of the typed REC-001 repair family instead backs up a raw capture of exactly the regions it will write, verified by re-read — preserving the unsound pre-state is the point, and a parsed backup would misrepresent it. Where that capture is impossible, the operation fails per Section 8 unless the plan carries an explicit journaled acknowledgement, recorded at plan creation and naming the uncapturable regions — the "separately supported recovery strategy" the Section 12 MUST-NOT clause requires, unavailable to any step outside the typed repair family. *(Amended in 12.2.0 by ADR-0024, resolving SI-16: the first sentence stands verbatim; the state-selected arms are additions.)*
- **PART-014:** Protect EFI System, Microsoft Reserved, Windows Recovery, Apple recovery, Apple sealed system volumes and signed system snapshots, Linux boot, active swap, and current boot/root volumes.
- **PART-015:** When a shrink is limited, report the true minimum size and its cause — including Windows unmovable files (pagefile, hibernation file, MFT zone, VSS store) — with per-cause remediation guidance. Remediation actions are separate, explicit plans; never silent side effects.
- **PART-016:** When a plan changes or regenerates identifiers (UUID, PARTUUID, disk GUID/MBR signature, volume serial, label), the planner MUST enumerate known dependent references — fstab, crypttab, BCD entries, bootloader configs, auto-unlock bindings — and either include supported update steps or attach an explicit warning listing exactly what the user must fix.

### 7.4 File-system operations

- **FS-001:** Windows first-class support: NTFS, FAT12/16/32, exFAT, and ReFS where documented native support permits.
- **FS-002:** macOS first-class support: APFS containers/volumes, HFS+, FAT32, and exFAT.
- **FS-003:** Linux first-class support: ext2/3/4, Btrfs, XFS, F2FS, FAT32, exFAT, NTFS, and swap according to validated tool support.
- **FS-004:** Detect APFS, HFS+, ReFS, ext, Btrfs, XFS, F2FS, NTFS, FAT, exFAT, UDF, LVM PV, Linux RAID, LUKS, BitLocker, ZFS pool members, Storage Spaces, LDM metadata, and common legacy file systems.
- **FS-005:** Check file-system health before shrink, move, copy, or conversion.
- **FS-006:** Never implement a novel production file-system resizer; use authoritative native APIs/tools through adapters.
- **FS-007:** Surface immutable technical limits, such as XFS not shrinking, as explicit blocked reasons.
- **FS-008:** Preserve file-system UUIDs only when safe; prevent accidental duplicate UUIDs after clone.
- **FS-009:** Verify file-system size, mountability, and reported free space after modification.
- **FS-010:** Operations of severity ≥ 3 (PLAN-004) touching an encrypted layer MUST require explicit user acknowledgment that recovery material (recovery key, passphrase, escrow) is available, and MUST link the platform's key-verification guidance.

### 7.5 Windows requirements

- **WIN-001:** Use Windows Storage Management API and documented storage/volume APIs as the primary interface.
- **WIN-002:** Treat VDS and DiskPart as isolated compatibility fallbacks, never the default abstraction.
- **WIN-003:** Discover and manage basic GPT/MBR disks, drive letters, mount folders, and VHD/VHDX. Storage Spaces are detected, represented, and protected only (Section 2.1). Protection's boundary is ADR-0018's: the pool, the spaces as structural objects, and member-disk substrates; a file system inside a space is an ordinary target within the space's provisioned block interface, through the documented API only, and mutation inside a space is `blocked` while the pool is degraded or a thin-provisioned space's allocation headroom cannot be verified.
- **WIN-004:** Detect legacy dynamic disks and support safe inspection and copy-based migration to basic disks; in-place LDM mutation is out of scope (Section 2.1).
- **WIN-005:** Detect BitLocker state, protect keys, explain suspend/unlock/decrypt requirements, and restore protection after success.
- **WIN-006:** Use VSS or documented volume-lock behavior where needed; never imply that VSS is a data backup.
- **WIN-007:** Support Windows OS migration to HDD, SATA SSD, and NVMe, including EFI/MSR/Recovery partitions.
- **WIN-008:** Validate UEFI/BCD boot configuration and provide repair actions.
- **WIN-009:** Reboot/offline operations must resume the same cryptographically bound plan.
- **WIN-010:** Ship signed installation, helper, updater, uninstaller, and rescue components.
- **WIN-011:** Detect hibernation, Fast Startup, active pagefiles, and dirty volumes. Operations blocked by these states MUST name the state and its remediation. The product MUST NOT silently delete or move `hiberfil.sys` or pagefiles; such remediation is its own explicit plan step.

### 7.6 Linux requirements

- **LIN-001:** Use UDisks2 for discovery/authorization and libblockdev or authoritative native tools for mutations.
- **LIN-002:** Support GPT/MBR, ext2/3/4, Btrfs, XFS, F2FS, FAT/exFAT, NTFS, and swap according to installed capabilities. The NTFS write stack (kernel `ntfs3` vs `ntfs-3g`) is selected and version-gated per ADR-L1.
- **LIN-003:** Support LUKS2 create/open/close/key management without exposing secrets.
- **LIN-004:** Support LVM2 PV, VG, LV, thin pool, snapshot, resize, activate, and deactivate workflows.
- **LIN-005:** Support mdraid discovery, create, assemble, stop, grow where safe, member replacement, status, and metadata cleanup.
- **LIN-006:** Detect device mapper, multipath, loop devices, Btrfs multi-device layouts, and active root/boot/swap dependencies. Multipath representation in v1 is detection-only (Section 2.1, ADR-0011): the inventory carries the kernel's own device-mapper multipath node and its member path devices, connected by the kernel-reported membership relation — whose edge kind is `platform-membership`, typed in 11.1.0 by ADR-0019 (platform-asserted composition, detection-only; the path-set encoding remains deferred exactly as stated below); the product infers no cross-path device sameness of its own; and a mutating operation on a multipath device or a recognized member reports CAP-003 `unsupported` with a multipath reason from CAP-003's reason vocabulary, a closed and versioned enum delivered with the capability engine (WP-050). The identity-record path-set encoding — including its body-versus-envelope placement — is deferred, behind a MODEL-003 schema version bump, to the specification change that first makes a multipath device a supported write target.
- **LIN-007:** Repair or regenerate GRUB and systemd-boot configuration through explicit plans.
- **LIN-008:** Package as a signed `.deb` and an Arch package with declared optional file-system dependencies.
- **LIN-009:** Use polkit rules scoped to validated plan execution, not broad command execution.
- **LIN-010:** After any identifier change (PART-016), verify that `/etc/fstab`, `/etc/crypttab`, and bootloader references still resolve; unresolved references produce explicit warnings and offered fix steps.

### 7.7 macOS requirements

- **MAC-001:** Use Disk Arbitration for device events and mount/unmount/eject coordination.
- **MAC-002:** Support GUID partition maps, APFS containers and volumes, HFS+, FAT32, exFAT, and disk images through documented mechanisms (APFS mutation surface per ADR-M1).
- **MAC-003:** Represent APFS physical stores, containers, volumes, roles, reserve/quota values, and snapshots.
- **MAC-004:** Detect FileVault and require a safe unlock/decrypt/offline path for affected operations.
- **MAC-005:** Distinguish Intel, Apple Silicon, startup, recovery, and external boot layouts.
- **MAC-006:** Treat Intel Boot Camp modification as a separate capability from inspection/migration.
- **MAC-007:** Ship a signed, notarized universal application and privileged helper.
- **MAC-008:** Do not target the Mac App Store if sandbox constraints prevent the declared functionality.
- **MAC-009:** Honor SIP and the sealed system volume: sealed volumes and signed system snapshots are protected objects (PART-014). Operations macOS permits only in Recovery (notably on Apple Silicon) MUST report `blocked` with that exact reason, never attempt a workaround. Whole-container destructive work that reaches the sealed objects only through substrate destruction is governed by this rule and Section 2.1's documented-paths clause, not by the sealed objects' own absolute refusal (ADR-0018, resolving SI-30); in v1 no such operation is implemented and the step family is empty.
- **MAC-010:** Detect Fusion Drives; all Fusion mutation is out of scope (Section 2.1).

### 7.8 Clone, migration, and image engine

- **IMG-001:** Clone a whole device or partition in used-block and sector-by-sector modes.
- **IMG-002:** Lock source and destination identity and reject reversal or destination changes.
- **IMG-003:** Support grow-to-fit and shrink-to-fit only after file-system and layout validation.
- **IMG-004:** Align partitions and preserve or intentionally regenerate identifiers as the plan declares (with PART-016 consistency handling).
- **IMG-005:** Verify copies with cryptographic checksums or authoritative file-system-aware verification.
- **IMG-006:** Create and restore raw sparse images with optional compression, checksums, and split archives (image format per ADR-I1).
- **IMG-007:** Resume interrupted image and clone jobs from a durable byte-range map.
- **IMG-008:** Provide damaged-media mode that records unreadable regions, retries conservatively, and never writes to the source.
- **IMG-009:** Mount or explore supported images read-only.
- **IMG-010:** Estimate required destination capacity before apply.
- **IMG-011:** Cross-sector-size operations (e.g., 512e ↔ 4Kn): cloning or restoring between devices with different logical sector sizes MUST recompute partition tables, alignment, and file-system geometry where the file system supports the target sector size, and MUST be `blocked` with a precise reason where it does not.

### 7.9 Recovery, rescue, and boot repair

- **REC-001:** Back up and restore partition-table metadata with device-identity validation.
- **REC-002:** Perform quick and deep lost-partition scans without writing to the source.
- **REC-003:** Preview candidate partitions, confidence, conflicts, and recoverable files before undelete.
- **REC-004:** Restore a lost partition only through a normal immutable plan.
- **REC-005:** Export recoverable files to a different device.
- **REC-006:** Provide Windows UEFI/BCD and Linux GRUB/systemd-boot repair plans.
- **REC-007:** Build a UEFI rescue environment using the same schemas, planner, journal, and helper validation (base image and Secure Boot chain per ADR-R1).
- **REC-008:** Verify rescue media after creation and before system-disk operations that may require it.
- **REC-009:** Surface the last durable checkpoint and valid roll-forward or recovery actions after interruption.
- **REC-010:** Never advertise rollback when the underlying data transformation is not reversible (enforced via PLAN-008).
- **REC-011:** Before mutating an encryption layer, create and verify a backup of its metadata (LUKS header, BitLocker metadata) with the same identity binding as PART-013. A failed or unverifiable backup blocks the operation (SAFE-005).

### 7.10 Diagnostics and erase

- **DIA-001:** Report SMART/NVMe health, temperature, wear, unsafe shutdowns, media errors, and critical warnings where supported.
- **DIA-002:** Run a read-only surface scan with progress, cancellation, and a bad-region map.
- **DIA-003:** Report TRIM/discard availability and alignment.
- **DIA-004:** Support controller/device secure erase or sanitize only after capability, power, frozen-state, and target-identity checks.
- **DIA-005:** Distinguish overwrite, crypto-erase, sanitize, format, discard, and file deletion; never call them equivalent.
- **DIA-006:** Use stronger confirmation for whole-device erase and display the immutable device identity.
- **DIA-007:** Produce a redacted diagnostic bundle that the user can inspect before export.

### 7.11 GUI and dark design

- **UI-001:** Default to a dark charcoal theme; also support system theme and accessible high contrast.
- **UI-002:** Main workspace contains a device rail, topology map, inspector, and pending-plan drawer.
- **UI-003:** Distinguish physical devices, partitions, containers, volumes, encryption, file systems, mounts, and free space visually and textually.
- **UI-004:** Show `Current → Planned` topology for every mutation.
- **UI-005:** Display exact target identity, operation order, risk severity and flags, offline/reboot needs, estimated affected range, and recovery action before Apply.
- **UI-006:** Provide Guided and Expert density modes without hiding risk or safety facts.
- **UI-007:** Never use color alone for identity, selection, file system, health, or risk.
- **UI-008:** Meet WCAG 2.2 AA, keyboard-only operation, screen-reader semantics, 200% zoom, reduced motion, and color-blind-safe palettes.
- **UI-009:** Require typed device-name confirmation for destructive whole-device actions.
- **UI-010:** Show actionable constraints and errors, including cause, unchanged state, safe next step, and diagnostic details.
- **UI-011:** Progress UI must distinguish planning, waiting for authorization, executing, verifying, reboot pending, recovering, failed, and complete.
- **UI-012:** The UI MUST not claim completion until postconditions pass.
- **UI-013:** v1 ships in English with all user-facing strings externalized for future localization. Exact byte values (MODEL-001) are always available in the inspector alongside IEC display units.

### 7.12 CLI and automation

- **CLI-001:** Provide human-readable output and stable versioned JSON.
- **CLI-002:** Support inventory, capabilities, plan, validate, dry-run, apply-plan, status, resume, cancel where safe, and export-diagnostics.
- **CLI-003:** The CLI MUST use the same planner, validator, helper, and plan schemas as the GUI.
- **CLI-004:** Raw destructive commands that bypass planning MUST NOT exist.
- **CLI-005:** Exit codes and error schemas MUST be documented and stable within a major version.
- **CLI-006:** Secret input MUST use protected prompt/descriptor mechanisms, not command-line arguments.
- **CLI-007:** Automation defaults to dry run unless an immutable reviewed plan is explicitly submitted.
- **CLI-008:** Honor `NO_COLOR` and non-TTY detection; `--json` output contains no ANSI sequences; long operations expose machine-readable progress as JSON Lines events.

### 7.13 Security, privacy, and supply chain

- **SEC-001:** Authenticate the GUI/CLI to the local helper and authorize only exact plan hashes.
- **SEC-002:** Reject replayed, expired, altered, cross-user, and cross-device plans.
- **SEC-003:** Isolate parsers for hostile/corrupt metadata and fuzz their boundaries (targets enumerated in Section 11.4).
- **SEC-004:** Sign applications, helpers, packages, updates, and rescue images (signing infrastructure per ADR-S1; rescue Secure Boot chain per ADR-R1).
- **SEC-005:** Publish an SBOM and dependency/license inventory for each release.
- **SEC-006:** Make telemetry opt-in and redact device serials, paths, labels, usernames, keys, and file names (recommended v1 posture: no telemetry, Section 2.1).
- **SEC-007:** Support fully offline use with no account for core functionality.
- **SEC-008:** Updates MUST be signature-verified, rollback-tested, and unable to downgrade security state silently (framework per ADR-U1).
- **SEC-009:** Persist audit logs locally with explicit retention and redaction controls.
- **SEC-010:** Supply chain: lockfiles are committed; `cargo audit` and `cargo deny` (advisories and licenses) gate CI; CI actions and builder images are pinned by digest; release builds SHOULD be reproducible, with deviations documented.

### 7.14 Packaging and documentation

- **PKG-001:** Produce signed Windows installer/uninstaller and update packages.
- **PKG-002:** Produce signed/notarized macOS application, helper, and uninstall procedure.
- **PKG-003:** Produce Debian and Arch packages with dependency metadata and clean removal behavior.
- **PKG-004:** Package capability data, schemas, licenses, notices, and recovery documentation with the product.
- **PKG-005:** Installation and removal MUST not modify disk layout or boot configuration unless an explicit user plan requests it.
- **DOC-001:** Document every capability, limitation, cancellation rule, and recovery path.
- **DOC-002:** Generate CLI and schema reference from source-controlled definitions where practical.
- **DOC-003:** Maintain a platform/file-system capability matrix tied to automated test evidence (generated, Section 11.7).

### 7.15 Execution environment

- **EXE-001:** System sleep and hibernation MUST be inhibited while a plan is in Protecting, Executing, or Verifying; inhibition is released afterward and its failure to engage is surfaced before apply.
- **EXE-002:** On battery power, plans with severity ≥ 3 or offline/system-disk steps MUST warn before apply; secure erase and sanitize SHOULD require external power where detectability permits.
- **EXE-003:** Progress reports step identity and byte counts where meaningful; ETAs are labeled as estimates; progress never moves backward except on a declared, journaled retry.
- **EXE-004:** Cancellable steps SHOULD acknowledge a cancel request within 2 seconds, even if safe unwinding then takes longer (PLAN-005 governs when cancel is offered).

## 8. Execution state machine

Top-level plan states:

`Draft, Validated, AwaitingAuthorization, Revalidating, Protecting, Executing, Verifying, Completed, Paused, RebootPending, RecoveryRequired, Failed, Cancelled`

**Terminal states:** `Completed`, `Failed`, `Cancelled`. Every terminal record includes an effect summary: `no-writes`, `partial`, or `complete`.

### Transition table

| From | To | Trigger |
| --- | --- | --- |
| Draft | Validated | Validator passes |
| Validated | Draft | User edit, or invalidation (CONC-003) |
| Validated | AwaitingAuthorization | User/CLI submits apply |
| AwaitingAuthorization | Revalidating | Authorization granted (HLP-003) |
| AwaitingAuthorization | Cancelled | User declines, or validity window expires (PLAN-007) — effect `no-writes` |
| Revalidating | Protecting | Helper revalidation passes (HLP-002, PLAN-006) |
| Revalidating | Failed | Identity/topology mismatch — effect `no-writes` (ACC-007) |
| Protecting | Executing | Metadata/encryption backups complete and verified (PART-013, REC-011) |
| Protecting | Failed | Backup failure (SAFE-005) — effect `no-writes` |
| Executing | Verifying | Final step complete |
| Executing | Paused | User pause at a cancellable or checkpoint boundary |
| Executing | RebootPending | Declared reboot step reached |
| Executing | RecoveryRequired | Step failure with recovery actions, or interruption detected on restart |
| Executing | Cancelled | Cancel honored at a safe point (PLAN-005) after journaled unwind — effect `no-writes` or `partial` |
| Paused | Executing | User resumes; topology re-verified first |
| Paused | Cancelled | User cancels — effect per journal |
| Paused | RecoveryRequired | Topology changed while paused |
| RebootPending | Revalidating | Same plan hash resumes after boot (WIN-009) |
| RebootPending | RecoveryRequired | Resume impossible or state divergent |
| Verifying | Completed | Postconditions pass (UI-012) |
| Verifying | RecoveryRequired | Postcondition failure |
| RecoveryRequired | Executing | User selects a valid roll-forward action (REC-009) |
| RecoveryRequired | Failed | User accepts failure; full report generated |

No other transitions exist. The transition table MUST be published machine-readably in `schemas/`, and property tests MUST prove undeclared transitions are unrepresentable (Section 11.6).

State transitions MUST be durable and idempotent (JRN-002, JRN-003). A restart MUST reconstruct status from the journal rather than guessing from UI state. `RecoveryRequired` persists across restarts until the user acts; recovery actions are themselves plans under this same contract.

The two `RecoveryRequired` exits are the two arms of that sentence (ADR-0027, resolving SI-20). A **roll-forward** action continues the original plan — same plan hash, same journal, execution resuming from the last durable checkpoint through the → Executing edge, its state derived from the journal plus fresh re-discovery (JRN-003) — and is the one recovery act that is not its own plan. Any **distinct** recovery action is its own `OperationPlan`, and selecting it is the acceptance the → Failed trigger names: the original terminates through that edge with its effect summary per journal, the full report, and a journaled linkage naming the recovery plan — one user act may drive both records, which remain two records. The original's Failed transition MUST be durable before the recovery plan enters its apply path (JRN-002's shape; on a shared bound device set, HLP-005 already makes the contrary order unexecutable). Cancelled's unwind semantics belong to the Executing era; after interrupted writes, the user-initiated terminal is Failed with its report. *(Added in 12.5.0 by ADR-0027; the table rows, terminal-state list, and "No other transitions exist" stand verbatim, and which acts on these edges require fresh authorization stays SI-21's open question.)*

## 9. Platform support floors

Initial floors; changeable only via ADR. The capability engine may narrow further at runtime (CAP-004); it may never widen below these floors.

| Tier | Platform | Floor | Guarantee |
| --- | --- | --- | --- |
| Primary | Windows 11 | 23H2 | Full advertised matrix |
| Compatibility | Windows 10 | 22H2 (build 19045) with ESU; LTSC 2021 | Read-only + core operations; advanced operations capability-gated |
| Primary | macOS | 13 (Ventura), Apple Silicon and Intel | Full advertised matrix per ADR-M1 limits |
| Primary | Debian / Ubuntu | Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15; UDisks2 ≥ 2.9 | Full advertised matrix |
| Primary | Arch Linux | Current rolling | Full advertised matrix, tool-version-gated |

Per-tool version floors live with the capability fixtures (CAP-006) in `docs/capabilities/`, not in this spec.

## 10. Required user acceptance scenarios

### ACC-001: Resize a Windows system volume

Given a healthy GPT/UEFI Windows disk with BitLocker, the product:

1. Identifies all EFI/MSR/Windows/Recovery relationships.
2. Reports whether BitLocker suspension or unlock is required.
3. Produces Current → Planned topology.
4. Saves partition metadata.
5. Executes online or resumes offline as declared.
6. Verifies NTFS and boot configuration.
7. Restores protection and reports the final exact sizes.

### ACC-002: Clone Windows to a smaller SSD

Given sufficient used space but a smaller destination, the product plans file-system shrink, partition resizing, clone, identifier handling, alignment, and boot repair. It refuses if the actual used data cannot fit.

### ACC-003: Prepare dual boot

Given a Windows device with available shrinkable space, the product creates a safe plan for Linux partitions without overwriting EFI/Recovery data and explains the bootloader implications.

### ACC-004: Recover a deleted partition

The product scans read-only, presents non-overlapping candidates and confidence, previews files, and restores metadata only after an explicit reviewed plan.

### ACC-005: Linux LUKS/LVM resize

The product orders file-system, LV, PV, encryption, and partition steps correctly for grow and shrink directions and blocks unsupported layer combinations.

### ACC-006: APFS external drive management

The product accurately displays the physical store, APFS container, volumes, roles, quotas, and free space and applies only operations supported by the current macOS environment.

### ACC-007: Stale device plan

If a target is removed and a similar-capacity device is connected, execution is rejected because immutable identity and snapshot checks fail.

### ACC-008: Interruption

At every declared checkpoint, forced termination or reboot results in a truthful journal state and either safe resume, safe roll-forward, or explicit recovery instructions.

### ACC-009: Missing tool

If a Linux file-system utility is absent or outside the tested version range, the capability becomes blocked with an installation/remediation message; the planner cannot create the affected write step.

### ACC-010: Destructive erase

The product shows immutable device identity and erase semantics, requires strong confirmation, verifies power/device state, and never targets the source or system disk through a selection race.

### ACC-011: Keyboard-only accessible apply

The full ACC-001 flow — device selection, plan review, typed confirmation, authorization, progress, completion — completes keyboard-only with correct screen-reader semantics (UI-008 evidence).

### ACC-012: Checkpoint cancellation

The user cancels mid-way through a long partition move. The product acknowledges promptly (EXE-004), stops at the declared checkpoint, the journal is truthful, the layout is consistent, and the user is offered valid resume or recovery actions (PLAN-005, REC-009).

### ACC-013: Damaged-media clone

Cloning a source with injected read errors, damaged-media mode maps unreadable regions, completes a best-effort image, never writes to the source, and the final report enumerates affected ranges (IMG-008).

### ACC-014: Weak-identity removable target

A USB enclosure exposes no serial/WWN. The product classifies identity as weak, requires the stronger confirmation path, refuses unattended apply without the recorded override, and re-probes immediately before apply (SAFE-003).

This scenario covers only the case where an identifier is **absent**. An enclosure that exposes its *own* identifier for removable media it does not identify — a card reader reporting the same serial for every card, and for an empty slot — produces a *Strong* record under SAFE-003 and is not exercised here. That case is open and MUST NOT be treated as covered by this scenario. *(Noted in 4.0.0. Confirmed on hardware; the absent-identifier case is the one SAFE-003 anticipated, and it is not the dangerous one.)*

### ACC-015: Cross-sector-size clone

Cloning a 512e source to a 4Kn destination either recomputes geometry end-to-end and verifies the result, or blocks with a precise file-system-level reason. Restore in the reverse direction behaves equivalently (IMG-011).

### ACC-016: Update rollback

A deliberately failed update rolls back to the previous working version without losing plans, journals, or audit logs, and cannot silently downgrade security state (SEC-008).
## 11. Testing contract

### 11.1 Per-work-package obligations

Every work package that can affect storage MUST include:

- Unit tests for domain and validation rules.
- Property tests for extent arithmetic and invariants.
- Golden-image integration tests.
- Negative tests for stale identity, corrupt metadata, missing dependencies, unsupported states, weak identity (SAFE-003), and permission denial.
- Fault-injection tests for every durable checkpoint it introduces.
- Schema compatibility tests for public JSON/RPC, including migration vectors for every schema version bump and rejection tests for incompatible versions.
- Redaction tests for errors, logs, plans, and diagnostics.

### 11.2 Invariants

At minimum, automated tests MUST prove:

- Partition extents do not overlap.
- Extents remain inside the bound device.
- Required alignment is preserved.
- Protected partitions cannot be modified without an explicit supported plan.
- Shrink order is file system before enclosing layers.
- Grow order is enclosing layers before file system.
- The source of clone/image/recovery is never written.
- Duplicate UUIDs are handled according to the plan.
- A helper cannot execute a plan with a different hash or topology.
- Completion cannot occur before postcondition verification.
- An interrupted move always leaves either the original mapping or a fully-copied, journal-recoverable state (PART-005).
- No undeclared state-machine transition is representable (Section 8).
- Two apply submissions racing for one device produce exactly one execution (CONC-005).

### 11.3 Test environment architecture

Three tiers; every tier is reproducible from the repository alone.

- **T1 — Unprivileged, everywhere:** synthetic disk images as regular files; pure planner/validator/model tests. Runs on any developer machine and every CI job with no elevation (SAFE-002).
- **T2 — Privileged, disposable VMs per OS:** loop/kpartx/LVM/mdraid and helper integration on Linux VMs; VHDX-backed disposable Windows VMs; disk-image-backed macOS VMs or hosted runners. Destructive suites run only here or in T3, gated by SAFE-007. Nested-virtualization limits per CI provider are documented in `docs/quality/`.
- **T3 — Physical lab:** explicitly provisioned, labeled fixture devices (SAFE-001) with SAFE-007 interlocks. The hardware matrix is versioned in `docs/quality/hardware-matrix.md` and MUST cover at minimum: NVMe 512e, NVMe 4Kn, SATA SSD, USB HDD, USB flash, SD via reader, plus a hot-plug rig for INV-009 and removal-during-write tests.

Fixture images are generated by scripts (deterministic, cached); binary disk images are never committed. A single task-runner entry point (e.g., `cargo xtask test --tier <n>`) runs identical commands locally and in CI (WP-000).

### 11.4 Fuzzing

`cargo-fuzz` targets MUST exist for every parser of on-disk or externally supplied bytes: GPT/MBR/APM headers, file-system probes, LVM/LUKS/mdraid metadata, plan/journal/RPC deserializers. Short fuzz smoke runs gate every PR touching a parser; scheduled long runs accumulate corpora; the release gate requires zero untriaged crashes or hangs.

### 11.5 Stress and environment tests

- Device-churn storms validating INV-009.
- VM hard-kill power-loss tests during image/clone jobs at randomized offsets, plus at every declared checkpoint class (extends fault injection).
- Sleep-inhibition and battery-state behavior (EXE-001/002) where the platform permits simulation.

### 11.6 State-machine conformance

Property tests generate transition sequences against the machine-readable table (Section 8); any undeclared transition or non-durable transition fails.

### 11.7 Traceability (automated)

CI generates `docs/traceability/` mapping every requirement ID to its tests and evidence artifacts. A work package claiming a requirement without linked evidence fails its gate. DOC-003's capability matrix is generated from CAP-006 fixtures plus test evidence — never hand-edited.

### 11.8 Coverage

Code coverage is measured per crate. Floors are set at M1 in repository configuration (not in this spec) and MUST NOT regress.

## 12. Definition of done

A requirement or work package is complete only when:

1. Production implementation exists; no no-op, fake, or test-only success path remains.
2. Relevant requirement IDs are linked in code/tests or task metadata, and the generated traceability map (11.7) shows the evidence.
3. Tests required by Section 11 pass.
4. Errors and unsupported cases are user-actionable.
5. GUI and CLI behavior use the same canonical capability and plan logic.
6. Public schemas and documentation are updated.
7. Logs and diagnostics pass redaction tests.
8. Platform-specific behavior is tested on its declared platform fixture.
9. No write test touched a host or non-disposable device.
10. The agent reports changed files, tests run, remaining limitations, and follow-up work, plus the spec version built against.
11. Any architectural deviation is captured in an ADR merged with the change.

## 13. Milestones and integration gates

Work packages alone do not force integration; milestones do. A milestone exits only when its criteria pass on all three platforms (or the milestone explicitly scopes fewer). Milestones are sequential; work packages within a milestone parallelize per Section 14.

| Milestone | Theme | Exit criteria |
| --- | --- | --- |
| **M0** | Foundations | CI green on Windows/macOS/Linux; schemas versioned with cross-language hash golden tests (MODEL-005); T1 fixture generator produces images; `xtask` single entry point works locally and in CI; accessibility harness runs; CODEOWNERS enforces ownership. |
| **M0.5** | Evidence | The read-only CLI chassis (WP-035) runs unprivileged on Windows, macOS, and Linux against WP-020 fixtures at Tier 1. Every gated surface refuses with a typed value naming the register issue or spec requirement that gates it — never merely an exit code, stderr string, or silent omission. `--json` output is ANSI-free and carries a schema version for every surface the package owns; domain payloads are absent and refuse. The redaction allowlist passes its tests; the dependency doctor resolves tools from trusted absolute paths only. The SI-33 media-change-counter liveness experiment is taken on real hardware and recorded in `docs/quality/observability.md`; the SI-35 loop-device measurement and its Windows partition-list equivalent are taken and recorded — operator-run and read-only, with the loop-backed portion gated on repository issue #94. Exiting M0.5 does not close SI-34's evidence list: macOS and real-partitioned-Linux observability rows remain outstanding. |
| **M1** | Trustworthy read-only product (internal alpha) | Inventory correct against fixtures on all platforms; capability engine honest (blocked/unsupported reasons verified); zero elevation anywhere; diagnostic bundle works; UI shows real topology read-only; CLI inventory/capabilities stable JSON; coverage floors set. |
| **M2** | Planning and dry run | Planner/validator/simulated topology complete; plan drawer and Current → Planned UI live; ACC-001/002/003/005 pass in planning-only mode (no writes, validator-level dry run); risk model and consequence text rendered; journal core merged. |
| **M3** | First safe writes (beta on disposable media) | Helpers ship on all platforms with per-apply authorization (HLP-003) demonstrated on each OS; basic GPT/MBR create/delete/format pass on T2; PLAN-009 full dry-run parity including helper revalidation; ACC-007 and ACC-008 pass; state-machine conformance tests pass; fault injection running in CI. |
| **M4** | Full storage operations | File systems, resize/move, encryption-aware flows, clone/image engine, recovery scans, diagnostics; ACC-001…006, 009, 012, 013, 015 pass on fixtures; PART-015 shrink-cause reporting and PART-016 identifier consistency demonstrated; usability targets recorded in `docs/quality/ux-targets.md`. |
| **M5** | Ship | Boot repair, rescue environment, secure erase, packaging/signing/updates on all platforms; ACC-010, 011, 014, 016 pass; Section 19 release gate satisfied. |

M1 and M3 are honest early ship points if scope must later narrow: a read-only inspector and a basic-operations tool are each independently valuable and safe.

## 14. Dependency-ordered work packages

Agents may work in parallel only when packages do not overlap owned files and all prerequisites are complete. Dependencies are explicit work-package or ADR IDs only.

| Package | Scope | Depends on | Milestone |
| --- | --- | --- | --- |
| WP-000 | Repository, CI (3 OS), xtask runner, CODEOWNERS, formatting, dependency policy (SEC-010), ADR template | None | M0 |
| WP-010 | Canonical domain model, schema versioning, canonical encoding + hashing (MODEL-005, ADR-C1) | WP-000 | M0 |
| WP-020 | Synthetic/golden disk image generator and T1/T2 destructive-test harness (SAFE-007 interlocks) | WP-000 | M0 |
| WP-030 | Design tokens, dark UI shell, accessibility harness | WP-000 | M0 |
| WP-035 | Read-only CLI chassis and evidence instrument: SAFE-004 structured argv; documented exit codes (CLI-005, provisional within major version 0); `NO_COLOR`/non-TTY and ANSI-free `--json` with schema-versioned package-owned surfaces (CLI-008, MODEL-003); JSON Lines progress; deny-by-default redaction allowlist (SAFE-006, INV-007); precursor observation records (toward MODEL-004); dependency doctor (CAP-004); technology-limit facts (FS-007 inputs only — the blocked-reason surface is WP-050's); redacted export-diagnostics (CLI-002, INV-007); fixture-backed replay over WP-020 images; unprivileged whole-device enumeration through each platform's own client-readable interfaces, reported as adapter-attributed observations under session-local selectors (SAFE-002, INV-006); and the INV-003 reach declaration for the contract this package itself reads — one answer per state in INV-003's list, per platform, derived from the contract rather than from any device, never omitted when the answer is "no", each cell citing the `docs/quality/observability.md` row it rests on. The enumeration opens no block device, mounts nothing, runs no repair, and does not widen its reads when run with privilege it did not need; where a platform has no adapter the answer stays the typed `not-implemented` value naming that platform's adapter package, never an empty list. This package's reach declaration describes its own contract and is not a claim about interfaces that contract does not read; the product discovery layer's INV-003 duty stays with WP-W100/WP-L100/WP-M100. Prints raw identifier strings labelled by reporting interface; computes no strength, table state, hash, verdict, or plan | WP-000, WP-020 | M0.5 |
| WP-040 | RPC schemas, transport per OS, handshake, helper authentication skeleton, redaction (RPC-001…006) | WP-010 | M0 |
| WP-050 | Capability engine interfaces and fixtures | WP-010, WP-020 | M1 |
| WP-W100 | Windows read-only inventory and health adapter | WP-010, WP-020, WP-050 | M1 |
| WP-L100 | Linux read-only inventory and capability adapter | WP-010, WP-020, WP-050 | M1 |
| WP-M100 | macOS read-only inventory, Disk Arbitration, APFS model | WP-010, WP-020, WP-050 | M1 |
| WP-080 | CLI inventory/capabilities/plan/dry-run | WP-040, WP-050 | M1–M2 |
| WP-060 | Pure planner, extent solver, risk model, simulated topology, reversal plans (PLAN-008) | WP-010, WP-050 | M2 |
| WP-070 | Journal (JRN-001…006) and execution state machine (Section 8) | WP-010, WP-040 | M2 |
| WP-090 | Current → Planned UI and plan drawer | WP-030, WP-060 | M2 |
| WP-S100 | Fuzzing harness, parser fuzz targets, corpora CI (11.4) | WP-010 | M2, continuous |
| WP-W110 | Windows helper, per-apply authorization (ADR-W1), basic GPT/MBR operations | WP-040, WP-060, WP-070, WP-W100 | M3 |
| WP-L110 | Linux helper, GPT/MBR, file systems, polkit | WP-040, WP-060, WP-070, WP-L100 | M3 |
| WP-M110 | macOS helper, GPT/APFS/HFS+ operations (ADR-M1) | WP-040, WP-060, WP-070, WP-M100 | M3 |
| WP-085 | CLI apply/resume/status/cancel against live helpers | WP-070, WP-080, and ≥1 of WP-W110/WP-L110/WP-M110 | M3 |
| WP-W120 | Windows file systems, BitLocker, unmovable-file reporting (PART-015, WIN-011), reboot/offline | WP-W110 | M4 |
| WP-L120 | LUKS2, LVM2, mdraid, boot repair, fstab/crypttab consistency (LIN-010) | WP-L110 | M4 |
| WP-M120 | FileVault, snapshots, SIP/SSV constraints (MAC-009), boot/recovery | WP-M110 | M4 |
| WP-I100 | Shared clone/image/verify/resume engine (ADR-I1, IMG-011) | WP-010, WP-020, WP-070 | M4 |
| WP-R100 | Partition-table + encryption-metadata backup (REC-011), lost-partition scan, preview | WP-010, WP-020, WP-060 | M4 |
| WP-D100 | SMART/NVMe, surface scan, TRIM | WP-W100, WP-L100, WP-M100 (per-platform delivery) | M4 |
| WP-095 | UI surfaces: diagnostics, recovery, erase, settings | WP-090, WP-D100, WP-R100 | M4 |
| WP-W130 | Windows clone/migrate and boot repair | WP-W120, WP-I100 | M5 |
| WP-R110 | Rescue environment and offline plan execution | WP-070, ADR-R1, and the helpers whose plans it executes (initially WP-L110, WP-W110) | M5 |
| WP-D110 | Secure erase/sanitize | WP-D100, WP-W110, WP-L110, WP-M110 | M5 |
| WP-DOC100 | Generated capability matrix, CLI/schema reference, traceability publishing (11.7, DOC-002/003) | WP-050, WP-080 | M4–M5 |
| WP-P100 | Windows packaging/signing/update (ADR-S1, ADR-U1) | WP-W130, WP-085, WP-095 | M5 |
| WP-P110 | Debian and Arch packaging | WP-L120, WP-085, WP-095 | M5 |
| WP-P120 | macOS signing/notarization/update (ADR-S1, ADR-U1) | WP-M120, WP-085, WP-095 | M5 |
| WP-Q100 | Cross-platform fault injection, model tests, release gates | All affected packages | M5 |

An orchestrating agent SHOULD split large platform packages into narrowly owned subtasks while preserving the same prerequisite gates.

## 15. Known hard problems and required ADRs

Each ADR MUST be accepted before its dependent work package starts. These are the questions most likely to sink the project if answered implicitly.

| ADR | Question | Blocks |
| --- | --- | --- |
| ADR-C1 | Canonical encoding and hash library for plans/snapshots; cross-language strategy | WP-010 |
| ADR-W1 | Windows per-apply authorization mechanism (HLP-003): consent broker design, token binding, secure-desktop use | WP-W110 |
| ADR-M1 | APFS mutation surface: `diskutil`-mediated operations, macOS version drift, absence of public partition-editing APIs; what is honestly supportable per macOS release | WP-M110 |
| ADR-L1 | Linux NTFS stack: kernel `ntfs3` vs `ntfs-3g`, version gating, capability mapping | WP-L110 |
| ADR-I1 | Image format: container layout, sparse/compression/split, resume map, format versioning | WP-I100 |
| ADR-R1 | Rescue base image and Secure Boot chain: Linux-based (shim/MOK) vs WinPE (ADK licensing) vs both; driver coverage; how rescue media get signed | WP-R110 |
| ADR-U1 | Per-OS update framework with verified rollback (SEC-008) | WP-P100/110/120 |
| ADR-S1 | Signing infrastructure: certificates, HSM/EV handling, notarization, CI secret isolation | WP-P100/110/120 |
| ADR-W2 | Long-term stance on Storage Spaces and dynamic-disk mutation (currently non-goals) | Future spec change only |

## 16. Prohibited shortcuts

Agents MUST NOT:

- Claim support based only on command availability.
- Parse human-localized command output when structured output or an API exists.
- Use the UI layer as the source of truth for topology or execution state.
- Put privileged code in the desktop renderer.
- Accept a device path alone as identity.
- Auto-select a replacement target after hot-plug.
- Continue after a failed metadata backup unless the user chooses a separately supported recovery strategy.
- Treat formatting as secure erase.
- Treat a snapshot, restore point, VSS snapshot, or partition-table backup as a full data backup.
- Log raw external-tool output without redaction and size limits.
- Add AI-generated "smart" recommendations that cannot be reproduced by deterministic rules.
- Mark a capability Stable without its matrix fixture and acceptance evidence.
- Renumber, reuse, or delete requirement IDs (Section 0.1).
- Ship or hide any flag, environment variable, or build option that bypasses helper validation in production builds.
- Weaken, skip, or fixture-out safety tests to make CI pass.
- Commit binary disk images; fixtures are generated by script (11.3).
- Push directly to the default branch; all changes go through PRs with required checks (Section 1).
- Swallow a cancel request without either honoring it or reporting why the current step is non-cancellable.

## 17. Prompt template for implementation agents

Use this template for every agent assignment:

```text
You are implementing one work package for the Cross-Platform Disk Partition Manager.

Read, in order:
1. Repository AGENTS.md
2. AGENT_BUILD_SPEC.md (note the spec version)
3. Relevant ADRs, schemas, and capability documents

Spec version: <4.x.x>
Work package: <WP-ID and title>
Requirement IDs: <explicit IDs>
Objective: <one bounded outcome>
Prerequisites already verified: <package/ADR IDs and evidence>
Owned paths: <exact files/directories the agent may edit>
Do not edit: <shared or agent-owned paths>
Test fixtures: <exact disposable images/VM profile and tier (T1/T2/T3)>
Required acceptance scenarios: <ACC IDs or task-specific cases>
Required commands/checks: <xtask targets, tests, formatters, schema validation>

Constraints:
- Never target host or user disks.
- Do not bypass the planner, capability engine, or helper validation.
- Do not change a public schema without an authorized versioned migration.
- Do not add fake/no-op support.
- Record all assumptions in the PR description; stop and report if the task
  requires an unsupported or unsafe assumption.

Deliver:
1. Implementation.
2. Automated tests.
3. Updated schemas/docs/capability fixtures and traceability evidence.
4. Summary of changed files.
5. Tests run and results.
6. Requirement IDs satisfied.
7. Remaining limitations and exact follow-up packages.
```

## 18. Prompt template for review agents

```text
Review work package <WP-ID> against AGENT_BUILD_SPEC.md <spec version>.

Check:
- Assigned requirement IDs and acceptance scenarios.
- Host-disk safety and disposable-test enforcement.
- Device identity, identity strength, and stale-plan rejection.
- Privilege boundary, per-apply authorization, and RPC authorization.
- Extent/file-system ordering invariants.
- Concurrency: locking, invalidation, racing applies (CONC-001…005).
- Cancellation, journaling, verification, and recovery behavior.
- Power/sleep behavior for long operations (EXE-001…004).
- Secret/log/telemetry redaction.
- Schema compatibility and canonical-hash stability.
- Platform capability honesty, including preview labeling.
- Accessibility on any UI surface touched (UI-007/008).
- Automated test completeness and traceability evidence (11.7).

Do not modify code unless explicitly assigned a fix task.
Report findings in severity order with exact file and line references.
State which requirements pass, fail, or lack evidence.
```

## 19. Release gate

The product is not releasable until:

- No open critical/high issue can select the wrong device, cause unexplained data loss, bypass privilege checks, leak secrets, or create an unbootable system outside a documented unsupported case.
- Every write operation has preconditions, postconditions, cancellation semantics, journal behavior, and recovery documentation.
- Fault-injected virtual/image tests complete with no unexplained topology divergence.
- The physical-device qualification suite passes on the versioned hardware matrix (11.3).
- The primary workflows — ACC-001, ACC-002, ACC-003, ACC-004, and ACC-010 — meet the task-completion and accessibility targets recorded in `docs/quality/ux-targets.md` (targets set no later than M4), and ACC-011 passes.
- All shipped packages, updates, helpers, and rescue images are signed and rollback-tested (ACC-016).
- The fuzz corpus has zero untriaged crashes or hangs (11.4).
- Supply-chain gates are green: SBOM published, advisory/license checks pass, pinned toolchains verified (SEC-005, SEC-010).
- The capability matrix matches actual automated evidence, generated per 11.7.

This gate has no calendar exception.

## 20. Glossary

- **Adapter:** platform-specific translation layer between canonical operations and native APIs/tools.
- **Capability:** the computed availability of one operation on one exact target in the current environment (CAP-001).
- **Checkpoint:** a journaled, durable point during execution from which resume or recovery is defined.
- **Disposable media:** storage that SAFE-001 permits tests to destroy.
- **Dry run:** a full-pipeline rehearsal of a plan that stops before the first write (PLAN-009).
- **Helper:** the signed, privileged per-OS process that revalidates and executes plans (Section 4.6).
- **Identity strength:** strong/weak classification of a device identity record (SAFE-003).
- **Journal:** the durable, append-only record of execution state (Section 4.7).
- **Plan (OperationPlan):** the immutable, hash-bound description of requested storage changes (Section 6).
- **Preview (capability):** planning allowed, apply refused pending qualification (CAP-003).
- **Protected object:** a partition/volume the product refuses to modify without an explicit supported plan (PART-014).
- **Reversal plan:** a generated plan that truthfully undoes another plan, where possible (PLAN-008).
- **Roll-forward:** completing remaining valid work from the last durable checkpoint after interruption (REC-009).
- **Aggregate:** one node type expressing every storage aggregation — LVM, mdraid, Storage Spaces, ZFS, APFS containers, LDM — distinguished by a technology discriminant rather than by separate types (MODEL-002, ADR-C5).
- **Backing signature:** on-disk evidence that an extent belongs to an aggregation or encryption technology, modelled as its own node so it can be represented even when the consumer it names is not observed (FS-004, INV-008, ADR-C5).
- **Snapshot (topology):** the immutable inventory state a plan was computed from, referenced by hash (PLAN-006). Distinct from `StorageSnapshot`, which is a storage-level object (APFS, LVM2, VSS, Btrfs, Apple signed system).
- **Tier (T1/T2/T3):** test environment classes defined in 11.3.
- **Weak identity:** a device identity record that is not Strong under SAFE-003 — most often one lacking any stable hardware identifier, but equally one whose partition-table state could not be determined, even when a serial or WWN is present. *(Corrected in 4.0.0. The 3.1.0 amendment changed SAFE-003 and left this restatement behind, so the glossary said a device with a serial and an unreadable table was not weak-identity, which is the case ADR-C3 deliberately added.)*
- **Work package (WP):** a bounded, dependency-gated unit of implementation (Section 14).
