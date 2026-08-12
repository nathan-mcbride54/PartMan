# ADR-0024: PART-013 discharges by the helper's authored table state

- Status: Accepted
- Date: 2026-08-11. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-16_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.2.0 (minor under §0.1 — additions only; argued in
  Decision, with the major counter-argument recorded)
- Work packages blocked: the backup step family (WP-060/WP-R100's when
  built; SI-16 resolved; SI-17 and SI-24 unchanged and still gating
  their own increments)
- Requirement IDs: PART-013, PART-001, REC-001, REC-011, SAFE-005,
  SAFE-003, INV-003, JRN-006, Section 8, Section 12, ADR-C4, ADR-0014
- Decision owners: Nate McBride

## Context

PART-013 requires backing up primary and secondary table metadata before
the first table write. Section 8 gates Protecting → Executing on
"backups complete and verified" and routes backup failure to Failed with
no writes (SAFE-005: failed backups disable the affected write
operation). Section 12's MUST-NOT list forbids continuing after a failed
metadata backup "unless the user chooses a separately supported recovery
strategy."

SI-16 filed the two media classes that break a uniform reading. On blank
media there is positively nothing to back up, so uniform
backup-must-complete makes PART-001 initialization — the operation whose
entire population this is — unrunnable by construction. On corrupt media
the backup source is precisely what is unsound, so the uniform reading
blocks the REC-001 restore family on exactly the media it exists for:
fail-closed against itself. The filing sketched three postures —
vacuous satisfaction, journaled acknowledgement, or block — and asked
which one holds.

Later resolutions gave this round its instruments: the table state is
helper-authored, three-valued, and fresh at the moment it matters
(ADR-0014; PART-001 since 8.0.0 initializes only on the helper's fresh
positively determined `Absent`), and a positively observed absence is a
value, not an unavailability (ADR-C4) — the principle that already
stopped PART-001 from initializing a blank device and an unreadable one
alike.

## Safety analysis

**The answer is state-selected: each filed option is right somewhere,
and the error was choosing one for all cases.**

**`Present` — the parse-level backup stands untouched.** Primary and
secondary metadata backed up and verified before the first table write;
failure → Failed, `no-writes`. The arm that carries almost every plan is
not weakened by anything in this decision.

**`Absent` — the obligation discharges as a journaled determination.**
The backup record is the helper's fresh positively determined absence: a
value, not a skip — ADR-C4's principle reaching the journal. It is the
same fresh determination PART-001 already requires for initialization:
one fact, two consumers, no new observation, no new state-machine edge —
Protecting → Executing's "complete and verified" is satisfied by the
journaled determination. **No user acknowledgement**: the fact is the
helper's own positive observation, and ceremony spent where it cannot
inform is the rubber-stamp shape the SI-39 and SI-18 rounds each
sustained as a real cost.

**`Indeterminate`, ordinary operations — already answered upstream, no
new rule.** SAFE-005 disables the affected write operation on corrupt
metadata before PART-013 is ever reached. This decision changes nothing
there, and says so because silence would read as license.

**`Indeterminate`, the repair family — backup is a verified raw capture
of exactly the regions the plan will write.** For an unsound source the
raw bytes are the only truthful backup: a parsed backup would launder
corruption into a clean-looking artifact, and preserving the damaged
pre-state — reversal substrate and forensics — is the entire point of
backing up before a repair. The capture is verified by re-read, and
REC-001's restore-with-identity-validation puts raw bytes back. The
family is a **typed step class** (REC-001's, owned by WP-R100), never an
intent flag an ordinary operation can wear — the arm attaches to the
step type, per the safety-is-computed-never-declared discipline
(ADR-0012/0018).

**`Indeterminate`, capture impossible — Failed stands, with the exit the
spec already carved.** Unreadable sectors inside the region the plan
will write mean the pre-state cannot be preserved; the operation refuses
per Section 8 rather than blindly overwriting. The one exit is Section
12's own clause — the user's separately supported recovery strategy —
formalized as an **explicit journaled acknowledgement recorded at plan
creation, naming the exact uncapturable regions**: the SAFE-003
weak-identity-override shape, never a mid-flight prompt, never available
outside the typed repair family.

**The TOCTOU objection is answered by existing machinery.** Protecting
follows Revalidating (HLP-002 re-discovery, PLAN-006 body-hash equality)
inside one execution, and the arm-selecting determination is the same
fresh one PART-001 requires. A device that gained a table between
validation and Protecting fails revalidation; one that gains it after
Protecting sits inside the window every plan already has between backup
and write, unchanged by this decision.

**What a consumer and a plan may rely on:**

- Every Protecting → Executing transition carries a journaled protection
  record: a verified parse-level backup, a positively determined
  absence, or a verified raw capture of the write-target regions. No arm
  is silent.
- No write proceeds over a region whose pre-state is neither preserved
  nor explicitly acknowledged as unpreservable by the user, at plan
  creation, by name.
- A blank device and an unreadable one never take the same arm.
- Ordinary operations against corrupt media remain SAFE-005-disabled.

## Options considered

### Option (a) — uniform vacuous satisfaction

Rejected: fail-open on the corrupt arm — repairs would proceed with no
pre-state preservation even where a verified capture was possible — and
a blank device and an unreadable one take the same silent arm, the exact
conflation ADR-C4 exists to prevent.

### Option (b) — uniform journaled acknowledgement

Rejected: spends user ceremony on a helper-determined fact it cannot
inform (`Absent`), trains the stamp that later approves a real risk, and
still fails to say what backup *means* for a readable-but-corrupt
source.

### Option (c) — uniform block

Rejected: the filing's own reductio. PART-001 unrunnable on its entire
population, the REC-001 family fail-closed against itself, the
fail-closed posture spent where there is no preservable state to
protect.

### Option (d) — state-selected discharge (accepted)

Accepted, scoped as above: the three filed options each land on the arm
where they are correct.

## Decision

Option (d), landed as spec 12.2.0's amendment to PART-013 and only
PART-013. **SI-16 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** PART-013's sentence
stands verbatim; the state-selected arms are additions; SAFE-005,
Section 8's transition rows, REC-011, and Section 12's MUST-NOT clause
all read naturally under every arm and are untouched. The
counter-argument (disambiguation as semantic change, the 3.1.0 caution)
was weighed and is recorded so the numbering is auditable; it was not
taken because §0.1's rule turns on what happens to existing requirement
text, and none changes.

## Consequences

- **Positive.** PART-001 runs on its population; the repair family is no
  longer fail-closed against itself; the blank/unreadable distinction
  reaches the protection step; every arm leaves a journal record.
- **Negative, accepted knowingly.** A repair over acknowledged
  uncapturable regions proceeds with no pre-state to restore — by the
  user's recorded, region-naming choice, which is the entire content of
  Section 12's existing clause. And the raw-capture artifact is not a
  parseable backup; consumers of backup artifacts must handle the raw
  variant, which the artifact's own record states.
- **Hash- and journal-visible, jointly sequenced.** The protection
  record's three-variant vocabulary lands with the journal's schema
  (JRN-006, WP-070's), exactly as the SI-19 linkage encoding is
  sequenced; the plan-creation acknowledgement is plan-body content
  landing with the backup step family's own schema change.
- **For WP-060/WP-R100.** The backup step family remains unbuilt; this
  ADR decides its semantics so the family can be built without
  answering a register question in code.

## Verification

Owned by the packages that build the arms, recorded here so none is
discovered late:

1. The blank-device fixture: journaled determination, no acknowledgement
   demanded, PART-001 proceeding on the same fresh determination.
2. The corrupt-readable repair fixture: raw capture byte-verified by
   re-read, restore round-trip through REC-001's identity-validated
   path.
3. The corrupt-unreadable fixture: Failed without the recorded
   acknowledgement; proceeding with it; the acknowledgement naming the
   exact regions; the acknowledgement unconstructible on a step outside
   the typed repair family.
4. The ordinary-operation-on-corrupt fixture: SAFE-005 refusal
   unchanged, PART-013 never reached.

## Revisit conditions

- REC-011's corrupt-encryption-header case is designed (WP-R100): it
  should follow this ADR's shape — state-selected, raw capture, recorded
  exit — or amend this ADR first if it cannot.
- The journal schema (JRN-006) lands: if the three-variant protection
  record cannot be encoded as specified, the variants are the part to
  keep and the encoding the part to redesign.
- A recovery strategy vocabulary richer than the single acknowledgement
  is ever designed; the plan-creation, region-naming, family-scoped
  properties fixed here are floors, not ceilings.
