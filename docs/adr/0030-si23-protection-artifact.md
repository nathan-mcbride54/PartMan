# ADR-0030: The REC-011 backup is a first-class protection artifact — helper-owned store, hash-only references, liveness retention, consequence-stated end of life

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-23_RECOMMENDATION_ROUND_2026-08-12.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.8.0 (minor under §0.1 — additions; REC-011's two
  sentences stand verbatim; argued in Decision, with the major
  counter-argument recorded)
- Work packages blocked: none newly — no WP-R100 or WP-070 assignment
  exists; this ADR records the obligations their creation must carry
  (the ADR-0027/0028/0029 precedent)
- Requirement IDs: REC-011 (amended); SAFE-005, SAFE-006, SAFE-008,
  JRN-004, JRN-005, SEC-009, REC-001, PART-013, Section 6, UI-005,
  UI-010, ADR-0024, ADR-0029 (read, none amended)
- Decision owners: Nate McBride

## Context

REC-011 mandates creating and verifying a backup of encryption-layer
metadata — explicitly the LUKS header, which contains the key slots —
before mutating that layer, with a failed backup blocking the operation.
SAFE-006 and JRN-005 forbid key material in logs, telemetry, crash
dumps, plans, journals, command histories, and UI state. SI-23 filed the
gap: nothing says where this artifact lives, how it is protected,
whether it inherits JRN-004's admin-protected location, or how it ends —
and it cannot simply be discarded, because RecoveryAction must reach it.

The object is dangerous in a specific, well-known way: a header backup
freezes the key-slot state at backup time, so a passphrase revoked
*after* the backup remains usable *with* the backup. The spec required
creating this object and then said nothing about it, while forbidding
key material from every surface that might otherwise have housed it.

ADR-0029's revisit condition named this round in advance: SI-23 either
routes the artifact's lifecycle through the journal (inheriting the
liveness rule) or files its own answer. This ADR answers the fork.

## Safety analysis

**Rule 1 — Home.** The artifact lives in a dedicated **protection-
artifact store**, helper-owned, inheriting JRN-004's location clause —
admin/root-protected, documented per OS — sibling to and distinct from
the journal. Its bytes never enter journal records: JRN-005's bounds
exist precisely to keep bulk out, a multi-MiB LUKS2 header would break
the budget discipline ADR-0029 built, and journal replay must not drag
key-slot material through every replay consumer. ADR-0029's fork is
answered in terms: the lifecycle does **not** route through the
journal, and the store adopts the liveness rule by this ADR's own text
(Rule 3).

**Rule 2 — Reference by identity, everywhere.** The journal, the plan
(Section 6's existing backup-and-recovery-actions body item), and every
SAFE-006 surface carry only the artifact's content hash and store
identity — never its bytes. SAFE-006's list stands verbatim, and the
reading is stated so it cannot drift: SAFE-006 forbids key material
*in* the named surfaces, and a hash reference is not the material. Only
the helper reads the store — SAFE-008's existing discipline, not new
policy. A restore is an ordinary plan through REC-001's
identity-validated path (REC-011's PART-013-shaped binding), at its own
authorization tier, journaled.

**Rule 3 — Retention by the liveness rule, adopted.** The artifact is
retention-exempt while its creating apply — or any apply whose linkage
closure (ADR-0027) references it — is non-terminal. This is the
filing's "RecoveryAction must reach it," made structural: nothing may
reclaim the artifact while anything live depends on it, and SAFE-005's
blocked-backup rule plus this exemption together mean the mutating
operation never starts without the artifact and never outlives its
reachability. The rule is adopted in REC-011's own normative text, not
claimed by analogy from ADR-0029's journal-record wording — the
drafting precision the adversarial round demanded.

**Rule 4 — End of life: explicit, consequence-stated, never silent.**
After the closure terminates, the artifact falls under explicit
user-controlled retention in SEC-009's shape. The deciding surface MUST
state both costs in terms: retaining the backup preserves the key-slot
state at backup time, so a passphrase revoked since then remains usable
with it; deleting it forfeits the disaster-recovery asset — a header
corrupted next month is restorable only from a backup that still
exists. Displayed, changeable defaults are permitted; silence is
forbidden in both directions. This is not a completion-time modal: it
is standing retention policy surfaced where retention is managed, the
UI-005/UI-010 honesty shape applied to the one object whose retention
is simultaneously a security liability and a recovery asset.

**The liability framing, answered.** The adversarial round's sharpest
attack — retention *is* the hazard — was partly sustained: the
liability is inherent to REC-011's mandate, since the artifact must
exist at least through the apply or the mutation is uninsured. The only
free variable is what happens after, where both arms carry real costs;
this decision refuses to pick silently in either direction.

**What a consumer and a plan may rely on:**

- The artifact exists in exactly one admin-protected, helper-read
  place; its bytes appear on no SAFE-006 surface, in no journal record,
  in no plan body — references are hashes.
- Recovery can always reach the artifact while anything live depends
  on it.
- A restore is an ordinary identity-validated plan at its own
  authorization tier.
- No key-slot state outlives the user's explicit, consequence-informed
  decision that it should — in either direction.

## Options considered

### Option (a) — embed the artifact in the journal

Rejected: inherits JRN-004's protection automatically but breaks
JRN-005's bounds, bloats ADR-0029's budget with bulk bytes, and drags
key-slot material through every replay consumer — three disciplines
traded for one inheritance the store gets by citation anyway.

### Option (b) — user-chosen arbitrary location

Rejected: an unprotected-by-default home for key-slot material, and
recovery cannot rely on reaching it — the filing's own requirement
broken by design.

### Option (c) — the protection-artifact store with its four rules (accepted)

Accepted, scoped as above.

### Option (d) — auto-delete on completion (and its silent-retention mirror)

Rejected together: auto-delete unilaterally forfeits the user's
disaster-recovery asset while REC-001 contemplates backup/restore as a
product surface; silent retention forever resurrects revoked
credentials silently. Both silences are rejected as one by Rule 4.

## Decision

Option (c), landed as spec 12.8.0's amendment to REC-011 and only
REC-011. **SI-23 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** REC-011's two
sentences stand verbatim; the four rules are additions; SAFE-006,
JRN-004, JRN-005, SEC-009, REC-001, and Section 6 are untouched and
read naturally under the rules. The counter-argument (the additions fix
semantics other text depended on — the 3.1.0 caution) was weighed and
is recorded so the numbering is auditable; it was not taken because
§0.1's rule turns on what happens to existing requirement text, and
none changes.

## Consequences

- **Positive.** The mandated-then-orphaned object has an owner, a home,
  a reference discipline, a structural reachability guarantee, and an
  honest end of life; SAFE-006's list survives unamended; ADR-0029's
  named fork closes with its rule adopted rather than stretched.
- **Negative, accepted knowingly.** A retained artifact is a standing
  security liability the user must manage — by explicit, stated-
  consequence policy rather than by the product's silent choice. And
  the store is one more admin-protected surface per OS to implement,
  document, and test.
- **Scope preserved.** ADR-0024's corrupt-source discharge stays
  WP-R100's under that ADR's shape; BitLocker/FileVault artifact
  specifics beyond the class rule land with their platform packages
  under this shape.
- **For WP-R100/WP-070, when their assignments are created.** The
  verification obligations below are this ADR's record; the
  assignments' creation MUST import them. The store's layout, encoding,
  and per-OS paths land under their own grants, jointly sequenced like
  the SI-16 protection record, the SI-19 linkage, and the SI-22
  compaction record.
- Nothing here is hash-visible beyond Section 6's existing
  backup-and-recovery-actions item, whose byte encoding lands with the
  jointly-sequenced schema change when built.

## Verification

Owned by the packages that build the store, recorded here so their
assignments' creation cannot omit them:

1. The artifact's bytes are absent from every SAFE-006 surface, every
   journal record, and every plan body — a sweep test in the WP-035
   redaction-gate shape, with the hash reference present where the
   bytes are not.
2. A retention pass reclaims no artifact whose creating apply or
   referencing closure is non-terminal — the ADR-0029 exemption test
   extended to the store.
3. A restore constructs only as an identity-validated plan (REC-001) at
   its own authorization tier; a raw store read outside the helper is
   structurally impossible on each platform's route.
4. The end-of-life surface states both consequences and is never silent
   in either direction — the default-displayed-as-policy test.

## Revisit conditions

- ADR-0024's corrupt-source discharge round (WP-R100) lands; if the
  unsound-source arm needs a different artifact class (a raw capture
  rather than a parsed backup), it takes these four rules or amends
  this ADR first.
- A platform's protected-location primitive cannot satisfy the
  inherited JRN-004 clause; the route decision that discovers this
  files the gap rather than weakening the clause.
- Key-management features (re-encryption, slot rotation as product
  operations) arrive; each new artifact they mint is presumptively a
  protection artifact under these rules, and one that cannot be files
  its own round.
