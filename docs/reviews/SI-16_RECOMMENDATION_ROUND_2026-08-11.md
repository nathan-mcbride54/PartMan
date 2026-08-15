# SI-16 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-16 stays Later (WP-060) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-16, an early filing
that sketches three postures — an absent or corrupt prior table satisfies
PART-013 vacuously, requires a journaled acknowledgement, or blocks — and
names the stake: whether the operations intended to repair a table are
fail-closed against themselves.

---

## The conflict, made precise

> **PART-013:** Back up primary and secondary GPT or MBR/EBR metadata
> before the first table write.

> **Section 8:** Protecting → Executing on "Metadata/encryption backups
> complete and verified (PART-013, REC-011)"; Protecting → Failed on
> "Backup failure (SAFE-005) — effect `no-writes`."

> **SAFE-005:** […] corrupt metadata […] and failed backups MUST disable
> the affected write operation.

> **Agents MUST NOT:** Continue after a failed metadata backup unless the
> user chooses a separately supported recovery strategy.

Two media classes break the uniform reading:

1. **Blank media** — the helper's fresh determination is `Absent`
   (positively observed, ADR-0014's sole-author architecture). There is
   positively nothing to back up. A uniform "backup must complete" reads
   as Protecting → Failed on every blank device, which makes PART-001
   initialization — the operation whose entire population this is —
   unrunnable by construction.
2. **Corrupt media** — the determination is `Indeterminate` (unreadable
   or ambiguous). The backup source is precisely what is unsound. A
   uniform reading blocks the REC-001 restore family — the operations
   built to fix damaged tables — on exactly the media they exist for:
   fail-closed against themselves, the filing's own words.

What the register's later resolutions have already given this round: the
table state is helper-authored, three-valued, and fresh at the moment it
matters (ADR-0014; PART-001 since 8.0.0 initializes only on the helper's
fresh positively determined `Absent`); and a positively observed absence
is a **value**, not an unavailability (ADR-C4) — the principle that
already stopped PART-001 from initializing a blank device and an
unreadable one alike.

## Recommendation: PART-013 discharges by the helper's authored table state — each filed option is right somewhere, and the error is choosing one for all cases

**The backup obligation is scoped by what the helper's fresh
determination says exists.** Concretely, per arm:

1. **`Present` — parse-level backup, required, verified.** The existing
   behavior, untouched: primary and secondary metadata backed up before
   the first table write; failure → Failed (`no-writes`). Nothing in
   this round weakens the arm that carries almost every plan.
2. **`Absent` (the helper's fresh positive determination) — the
   obligation discharges as a journaled determination.** The backup
   record *is* the positively determined absence: a value, not a skip
   (ADR-C4's principle reaching the journal). Protecting → Executing's
   "complete and verified" is satisfied by the journaled determination —
   the same fresh determination PART-001 already requires for
   initialization, one fact with two consumers, so no new observation
   and no new state-machine edge. **No user acknowledgement**: the fact
   is the helper's own positive observation, and asking the user to
   acknowledge it is the rubber-stamp shape the SI-39/SI-18 rounds both
   sustained as a real cost — ceremony spent where it cannot inform.
3. **`Indeterminate`, ordinary operations — already answered upstream,
   no new rule.** SAFE-005 disables the affected write operation on
   corrupt metadata before PART-013 is ever reached. This round changes
   nothing about that arm and says so, because leaving it implicit would
   read as license.
4. **`Indeterminate`, the repair family — backup is a verified raw
   capture of exactly the regions the plan will write.** For an unsound
   source, the raw bytes *are* the honest backup: a parsed backup would
   launder corruption into a clean-looking artifact, and preserving the
   damaged pre-state — for reversal substrate and forensics — is the
   entire point of backing up before a repair. The capture is verified
   by re-read, and REC-001's restore-with-identity-validation can put
   raw bytes back. The repair family is scoped as a **typed step class**
   (REC-001's, owned by WP-R100), never an intent flag on an ordinary
   operation — a closed set, not a creep surface.
5. **`Indeterminate`, the repair family, capture impossible — Failed
   stands, with the exit the spec already carved.** If even the raw
   capture fails (unreadable sectors inside the region the plan will
   write), Protecting → Failed per Section 8 — a region whose pre-state
   cannot be preserved is refused, not blindly overwritten. The one exit
   is the existing MUST-NOT's own clause: the user chooses a separately
   supported recovery strategy, and that choice is an **explicit
   journaled acknowledgement recorded at plan creation** (the SAFE-003
   weak-identity-override shape: recorded before apply, never a
   mid-flight prompt), naming the exact uncapturable regions.

So the filing's three options land as: vacuous-but-journaled for
`Absent`; capture-and-verify (not vacuous, not blocking) for readable
`Indeterminate` repairs; block-with-recorded-exit for unreadable ones;
and the plain block stays exactly where SAFE-005 already put it, on
ordinary operations against corrupt media.

## What a consumer and a plan may rely on

- Every Protecting → Executing transition carries a journaled protection
  record: a verified parse-level backup (`Present`), a positively
  determined absence (`Absent`), or a verified raw capture of the
  write-target regions (`Indeterminate` repair). No arm is silent.
- No write ever proceeds over a region whose pre-state is neither
  preserved nor explicitly acknowledged as unpreservable by the user, at
  plan creation, by name.
- A blank device and an unreadable one never take the same arm — the
  distinction ADR-C4 built into the record vocabulary now reaches the
  protection step.
- Ordinary operations against corrupt media remain SAFE-005-disabled;
  nothing here opens them.

## The adversarial round

**Attack 1 — "the `Absent` arm is a TOCTOU hole: blank at Protecting,
occupied at write."** Refuted by the existing machinery, cited rather
than extended: Protecting follows Revalidating (HLP-002 re-discovery,
PLAN-006 body-hash equality) inside one execution, and the arm-selecting
determination is the same fresh one PART-001 requires. A device that
gained a table between validation and Protecting fails revalidation; one
that gains it after Protecting is inside the window every plan already
has between backup and write, unchanged by this round.

**Attack 2 — "a raw sector capture is not a 'backup' in PART-013's
sense — no parse path can restore it."** Refuted by locating the
purpose. PART-013 protects the pre-write state; for an unsound source
the raw bytes are the only truthful representation of that state, and
REC-001's restore-with-identity-validation puts raw bytes back. The
attack sharpened point 4's wording: the capture covers *exactly the
regions the plan will write*, verified by re-read — a defined artifact,
not a best-effort gesture.

**Attack 3 — "the acknowledgement exit will be rubber-stamped on every
dying disk."** Partly sustained, and it drove the scoping: the exit
exists only for the typed repair family, only when a verified capture
failed, recorded at plan creation naming the exact regions — never a
mid-flight prompt, never available to ordinary operations. The
alternative is worse on both sides: no exit makes repairs on degraded
media impossible (the filing's fail-closed-against-itself), and a
uniform acknowledgement arm would spend the ceremony everywhere,
training the stamp.

**Attack 4 — "'repair-declared' is an intent flag any plan can wear."**
Refuted by construction: the family is REC-001's typed step class, a
closed set owned by WP-R100. An ordinary create/delete/format step
cannot claim the arm — the arm attaches to the step type, not to a
declaration, per the safety-is-computed-never-declared discipline
(ADR-0012/0018's shape).

**Attack 5 — "this decides REC-011's encryption-metadata twin."**
Refuted by scope, with the boundary stated: REC-011 triggers before
*mutating an encryption layer* — a layer that exists to be mutated has
metadata to back up, so the blank arm is vacuous there, and the
corrupt-header restore case belongs to WP-R100's design under this
ADR's shape *when it is designed*, as its own reviewed round. Nothing
here amends REC-011.

**Attack 6 — "the journaled-determination arm invents a new journal
record class — schema surface."** Sustained as a fact, priced as small,
and bounded: the protection record's vocabulary lands with the journal's
own schema (JRN-006, WP-070's), where the three arms are three variants
of one record. This round fixes the semantics; the encoding lands with
the package that owns the journal format, jointly sequenced exactly as
the SI-19 linkage encoding was.

## Rejected, and why — to be recorded with the decision

- **(a) Uniform vacuous satisfaction.** Fail-open on the corrupt arm:
  repairs would proceed with no pre-state preservation even where a
  verified capture was possible, and a blank device and an unreadable
  one take the same silent arm — the exact conflation ADR-C4 exists to
  prevent.
- **(b) Uniform journaled acknowledgement.** Spends user ceremony on a
  helper-determined fact it cannot inform (`Absent`), trains the stamp
  that later approves a real risk, and still fails to say what backup
  *means* for a readable-but-corrupt source.
- **(c) Uniform block.** The filing's own reductio: PART-001 unrunnable
  on its entire population, the REC-001 family fail-closed against
  itself, and the fail-closed posture spent where there is no
  preservable state to protect.
- **(d) is the recommendation** — state-selected discharge.

## Deliberately not decided

REC-011's corrupt-encryption-header case (WP-R100's, under this shape
when designed); the protection record's journal encoding (JRN-006,
WP-070, jointly sequenced); SI-17, SI-24; any recovery-scan or
lost-partition behavior (REC-002's, untouched).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0024 is the next free number; reservation PR
before resolution PR, the established shape), amends **PART-013 only** —
its sentence stands verbatim, gaining the state-selected discharge arms,
the typed-repair-family scoping, and the capture-impossible exit's
plan-creation acknowledgement — bumps **minor** (12.2.0: additions; the
uniform-block reading was never text, Section 8's triggers and the
MUST-NOT's clause read naturally under every arm, and SAFE-005 is
untouched), and moves SI-16 to Resolved. The major counter-argument is
recorded for the decision to overrule with. WP-060's re-attribution
follows (the #261/#264/#267 shape): the backup step family stays
unbuilt — this ADR decides semantics, and the family lands with its own
increment when WP-060 or WP-R100 builds it.

Verification obligations for the ADR, owned by the packages that build
the arms: the blank-device fixture (journaled determination, no
acknowledgement demanded, PART-001 proceeding); the corrupt-readable
repair fixture (raw capture byte-verified, restore round-trip through
REC-001); the corrupt-unreadable fixture (Failed without the recorded
acknowledgement, proceeding with it, the acknowledgement naming the
regions); and the ordinary-operation-on-corrupt fixture (SAFE-005
refusal unchanged, PART-013 never reached).
