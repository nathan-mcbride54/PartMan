# SI-23 recommendation round — 2026-08-12

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-23 stays Later (WP-R100) until a decision is
recorded through a WP-010 spec change with an ADR, the established shape.
This is an untracked session artifact under `docs/reviews/**` (WP-000);
the register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-23, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it. ADR-0029's revisit conditions named this round in
advance: SI-23's resolution assigns the artifact its owner, and either
routes its lifecycle through the journal (inheriting the liveness rule) or
files its own — this round answers that fork in terms.

---

## The conflict, made precise

> **REC-011:** Before mutating an encryption layer, create and verify a
> backup of its metadata (LUKS header, BitLocker metadata) with the same
> identity binding as PART-013. A failed or unverifiable backup blocks
> the operation (SAFE-005).

> **SAFE-006:** BitLocker, FileVault, LUKS, recovery keys, passphrases,
> and key files MUST NOT appear in logs, telemetry, crash dumps, plan
> files, command histories, or UI state snapshots.

> **JRN-004/JRN-005:** journals live in an admin/root-protected
> documented location, bounded; embedded tool output is bounded and
> redacted; journals never contain secrets.

> **SI-23's filing:** nothing says where this artifact lives, how it is
> protected, or whether it inherits JRN-004's admin-protected location.
> It cannot be discarded, because RecoveryAction must reach it.

The artifact REC-011 mandates is dangerous in a specific, well-known
way: a LUKS header backup freezes the key-slot state at backup time, so
a passphrase revoked *after* the backup remains usable *with* the
backup. The spec requires creating this object and then says nothing
about its home, its protection, its reference discipline, or its end of
life — while forbidding key material from every surface that might
otherwise have housed or described it.

## Recommendation: the artifact is a first-class protection artifact — helper-owned store, referenced by identity, retained by the liveness rule, ended by explicit user decision with the consequence stated

**Four rules, one named object.** Concretely:

1. **Home: a dedicated protection-artifact store, helper-owned,
   inheriting JRN-004's location clause** — admin/root-protected,
   documented per OS — sibling to and distinct from the journal. The
   artifact's bytes never enter journal records: JRN-005's bounds exist
   precisely to keep bulk out, a multi-MiB LUKS2 header would break the
   budget discipline ADR-0029 just built, and journal replay must not
   drag key-slot material through every replay consumer. This answers
   ADR-0029's fork: the lifecycle does **not** route through the
   journal, and the store adopts the liveness rule by its own text
   (rule 3).
2. **Reference by identity, everywhere.** The journal, the plan
   (Section 6's existing backup-and-recovery-actions body item), and
   every SAFE-006 surface carry only the artifact's content hash and
   store identity — never its bytes. SAFE-006's list stands verbatim,
   and the artifact joins none of its surfaces: the reading, stated so
   it cannot drift, is that SAFE-006 forbids key material *in* the
   named surfaces, and a hash reference is not the material. Only the
   helper reads the store (SAFE-008's discipline); clients receive
   typed identity and metadata; a restore is a plan through REC-001's
   identity-validated path with full ceremony.
3. **Retention: the ADR-0029 liveness rule, adopted.** The artifact is
   retention-exempt while its creating apply — or any apply whose
   linkage closure references it — is non-terminal. This is the
   filing's "RecoveryAction must reach it," made structural: recovery
   can always reach the artifact because nothing may reclaim it while
   anything live depends on it.
4. **End of life: an explicit user decision with the consequence stated
   at the decision point.** After the closure is terminal, the artifact
   falls under explicit user-controlled retention in SEC-009's shape —
   never silently kept forever, never silently discarded. The surface
   offering the decision MUST state both costs in terms: retaining the
   backup preserves the key-slot state at backup time, so a passphrase
   revoked since then remains usable with it; deleting it forfeits the
   disaster-recovery asset (a header corrupted next month is restorable
   only from a backup that still exists). The honest-disclosure shape
   UI-005/UI-010 already carry, applied to the one object whose
   retention is simultaneously a security liability and a recovery
   asset.

## What a consumer and a plan may rely on

- The artifact exists in exactly one place, admin-protected, helper-
  read; its bytes appear on no SAFE-006 surface, in no journal record,
  in no plan body — references are hashes.
- Recovery can always reach the artifact while anything live depends on
  it; the SAFE-005 blocked-backup rule and the liveness exemption
  together mean the mutating operation never starts without the
  artifact and never outlives its reachability.
- A restore is an ordinary plan: identity-validated (REC-001, REC-011's
  PART-013-shaped binding), authorized at its own tier, journaled.
- No key-slot state outlives the user's explicit, consequence-informed
  decision that it should — in either direction.

## The adversarial round

**Attack 1 — "a header backup resurrects revoked passphrases; retaining
it at all is the hazard, and the round launders a liability as a
feature."** Partly sustained, and it shaped rule 4. The liability is
inherent to REC-011's mandate — the artifact must exist at least
through the apply, or the mutation is uninsured — so the only free
variable is what happens after, where both arms carry real costs:
silent retention resurrects revoked credentials, silent deletion
forfeits the only copy of a recovery asset. The resolution refuses to
pick silently in either direction and puts the stated trade in front of
the user, which is this register's honesty pattern everywhere else
consequences land on users.

**Attack 2 — "a new store is new attack surface and new schema, decided
in a register round."** Refuted by the precedent stack: semantics
decided here, layout and encoding landed with WP-R100/WP-070 under
their own grants (the SI-16 protection record, SI-19 linkage, and
SI-22 compaction-record pattern, now four times over). The location
clause is inherited from JRN-004 verbatim rather than invented, and
helper-only read access is SAFE-008's existing discipline, not new
policy.

**Attack 3 — "reference-by-hash makes the artifact hash-visible in the
plan body — schema creep into WP-010's territory."** Refuted by the
body's own contents: Section 6 already lists "Backup and recovery
actions" as a body item; the artifact's identity is that item's
natural content, and its byte encoding lands as the jointly-sequenced
schema change when the item is built — nothing new enters the body by
this decision.

**Attack 4 — "ADR-0029's liveness rule is written for journal records;
adopting it for a non-journal store is analogy, not inheritance."**
Sustained as a drafting precision and answered in rule 3's wording: the
store adopts the rule *by its own normative text* in REC-011's
amendment — not by claiming ADR-0029's text covers it. ADR-0029's
revisit condition asked exactly this question; the answer is recorded
where that condition said it should be.

**Attack 5 — "the user-decision arm will be dialog fatigue: every
completed encryption operation ends in a security quiz."** Refuted by
scope and shape: the decision point is not a modal at completion — it
is explicit retention policy in SEC-009's existing shape, surfaced
where retention is managed, with sensible defaults permitted so long
as neither arm is silent (a default MUST be displayed as the standing
policy it is, with the stated consequence, and changeable). What is
forbidden is silence, not defaults.

**Attack 6 — "this decides the corrupt-header restore discharge ADR-0024
reserved for WP-R100."** Refuted by scope, stated: ADR-0024's revisit
condition — how REC-011's backup obligation discharges against a
corrupt source — is untouched. This round houses the artifact; what
PART-013-shaped arm applies when the source is unsound stays WP-R100's
under ADR-0024's shape.

## Rejected, and why — to be recorded with the decision

- **(a) Embed the artifact in the journal.** Inherits JRN-004's
  protection automatically but breaks JRN-005's bounds, bloats the
  ADR-0029 budget with bulk bytes, and drags key-slot material through
  every replay consumer — three disciplines traded for one inheritance
  the store gets by citation anyway.
- **(b) User-chosen arbitrary location.** An unprotected-by-default
  home for key-slot material, and recovery cannot rely on reaching it —
  the filing's own requirement broken by design.
- **(c) is the recommendation** — the protection-artifact store with
  its four rules.
- **(d) Auto-delete on completion.** Unilaterally forfeits the user's
  disaster-recovery asset — the header corrupted next month is
  restorable only from the backup this arm just deleted — and REC-001
  contemplates backup/restore as a product surface. Its mirror
  (silent retention forever) resurrects revoked credentials silently;
  both silences are rejected together by rule 4.

## Deliberately not decided

ADR-0024's corrupt-source discharge (WP-R100's, under that ADR's
shape); the store's layout, encoding, and per-OS paths (WP-R100/WP-070,
jointly sequenced); BitLocker/FileVault artifact specifics beyond the
class rule (each platform's package, under this shape); SEC-009's
broader audit retention; any UI wording (UI-010's).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0030 is the next free number; reservation PR
before resolution PR, the established shape), amends **REC-011 only** —
its two sentences stand verbatim, gaining the four rules: the
helper-owned store inheriting JRN-004's location clause, the
reference-by-identity discipline with SAFE-006's list unamended, the
adopted liveness retention, and the explicit consequence-stated
end-of-life — bumps **minor** (12.8.0: additions; SAFE-006, JRN-004,
JRN-005, SEC-009, REC-001, and Section 6 all stand verbatim), and
moves SI-23 to Resolved. The major counter-argument is recorded for
the decision to overrule with. **No re-attribution PR follows** — no
WP-R100 or WP-070 assignment exists; the ADR records the verification
obligations so those assignments' creation cannot omit them (the
ADR-0027/0028/0029 precedent).

Verification obligations for the ADR, owned by the packages that build
the store:

1. The artifact's bytes are absent from every SAFE-006 surface, every
   journal record, and every plan body — a sweep test in the WP-035
   redaction-gate shape, with the hash reference present where the
   bytes are not.
2. A retention pass reclaims no artifact whose creating apply or
   referencing closure is non-terminal — the ADR-0029 exemption test
   extended to the store.
3. A restore constructs only as an identity-validated plan (REC-001)
   at its own authorization tier; a raw store read outside the helper
   is structurally impossible on each platform's route.
4. The end-of-life surface states both consequences and is never
   silent in either direction — the default-displayed-as-policy test.
