# SI-25 recommendation round — 2026-08-12

**Status: a recommendation adversarially reviewed, then filed as Accepted
on Nate's directive** ("finish SI-25 and SI-26"), following ten identical
delegated arcs in this session pair. Untracked session artifact under
`docs/reviews/**` (WP-000); the register's own text is not modified by
this round.

The register entry is `docs/spec-issues/README.md` §SI-25, an early
filing with no options recorded.

---

## The conflict, made precise

> **CAP-002:** Model detect, read, create, grow, shrink, move, copy,
> check, repair, label, UUID, encrypt, decrypt, and wipe separately.

> **DIA-005:** Distinguish overwrite, crypto-erase, sanitize, format,
> discard, and file deletion; never call them equivalent.

CAP-002 enumerates fourteen operations including a single `wipe`;
DIA-005 requires six erase kinds distinguished and never called
equivalent. One `wipe` cannot be six never-equivalent things. Separately,
PART-007 (split/merge), PART-010 (MBR↔GPT convert), and PART-011
(clone-and-reformat migration) map to no CAP-002 operation at all.
Whether the list is a closed enumeration or a required minimum is
unstated — and WP-050's delivered engine already carries an `Operation`
enum spelled from CAP-002's names, so the answer also decides what that
enum's extension discipline is.

## Recommendation: a required minimum over a closed-and-versioned vocabulary

1. **CAP-002's list is a floor, not a ceiling**: the operations that
   MUST be modeled separately wherever they exist. It was never a claim
   that no other operation may exist — PART-007/010/011 are existing
   normative operations the list simply predates.
2. **The operation vocabulary is closed and versioned at every moment**
   (MODEL-003's discipline, the WP-050 reason-enum precedent): additions
   arrive only through reviewed spec changes with schema versioning,
   never by drift — which is what keeps CAP-005's one-engine promise
   stable while the floor reading keeps DIA-005 implementable.
3. **`wipe` is a family, and DIA-005's kinds are its members.** When
   erase surfaces are built, the six kinds are modeled as separate
   operations — capability genuinely differs per kind (sanitize needs
   device support and DIA-004's checks; discard needs TRIM; crypto-erase
   needs an encrypted layer) — making never-equivalent structural rather
   than behavioral. Split/merge, convert, and migrate join as named
   operations when their packages build them, for the same reason:
   their feasibility is not derivable from member operations.
4. **The delivered enum stands until WP-050's next increment.**
   `crates/capability`'s `Operation` is Rust; extension rides the next
   reviewed increment under the versioned discipline this decision
   fixes — the standing debt pattern, recorded rather than tripped.

## The adversarial round

**Attack 1 — "the closed-enumeration reading is safer: a fixed list
cannot creep."** Rejected on its own consequence: it makes DIA-005
unimplementable (one operation cannot carry six never-equivalent
semantics) and leaves PART-007/010/011 permanently unmodelable — the
fail-closed posture spent making required features unrepresentable.

**Attack 2 — "a minimum without versioning lets surfaces drift apart,
breaking CAP-005."** Sustained and absorbed as point 2: the vocabulary
is closed *at every moment* and moves only by reviewed versioned
change — minimum-over-time, closed-in-the-instant.

**Attack 3 — "kind-discriminated wipe (one operation, a kind field) is
lighter than six operations."** Rejected: a discriminant invites exactly
the equivalence DIA-005 forbids — one operation with kinds *is* "calling
them equivalent" at the modeling layer — and capability answers differ
per kind anyway, so separate modeling is CAP-002's own principle
applied.

**Attack 4 — "this decides SI-13/SI-14's territory."** Refuted by
scope: nothing here touches identity binding or confidence; the
vocabulary rule is about operation names only.

## Rejected, and why

- **(a) Closed enumeration.** Attack 1: DIA-005 unimplementable,
  required operations unrepresentable.
- **(b) Open minimum without versioning.** Attack 2: surface drift
  against CAP-005.
- **(c) is the recommendation** — minimum over a closed-and-versioned
  vocabulary, wipe as a family.
- **(d) Kind-discriminated wipe.** Attack 3: structural equivalence of
  the never-equivalent.

## If accepted, the mechanics

ADR-0031 (reservation then resolution, the established shape), amending
**CAP-002 only** — its sentence verbatim plus the floor/versioning/
family additions — **minor** (12.9.0). SI-25 → Resolved. No
re-attribution: WP-050's assignment cites no SI-25 gate; the enum
extension rides its next increment. Verification obligations for the
ADR: the vocabulary-closure test extended per version (the WP-040
claim-closure shape); the six erase kinds as distinct operations with
distinct capability answers when built; a drift test that no surface
carries an operation the versioned vocabulary does not.
