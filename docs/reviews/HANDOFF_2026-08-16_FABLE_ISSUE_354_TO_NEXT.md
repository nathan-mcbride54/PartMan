# Handoff — 2026-08-16, issue #354's kind half (ADR-0045) and the r26 re-pin

**From:** Claude (Fable 5), the session Nate directed with "take the next
slice: #354's kind half" (the same session took #360 earlier the same
day; that handoff is `HANDOFF_2026-08-16_FABLE_ISSUE_360_TO_NEXT.md`).
**To:** whoever picks this up next.

> `docs/reviews` artifact, committed under WP-000 in its own pull request
> after the WP-020 r26 re-pin merged.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`e5448cb`** — the merge of PR #399 (the r26 re-pin), on top of `ee12af2` |
| Spec | **15.1.0** (ADR-0045) |
| `cargo xtask ci` | **exit 0** on the act — 630 annotations, 50 evidence rows, 85 requirements, 666 live tests; workspace 673 passed |
| WP-020 pin | **`ee12af2`** — `git diff --name-only ee12af2 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **8** — #319, #333, #365, #366, #370, #371, #392, **#397** (new; **#354 closed** by ADR-0045) |
| Proxmox | no `partman-wp020-*` guest; VMID **9451** is next; the `-r26` script set is current |

**Nothing is owed.** The next Rust merge owes r27.

---

## 1. What landed

| PR | Package | What |
| --- | --- | --- |
| #396 | Governance | ADR-0045's path reserved under WP-010. |
| #398 | WP-010 | **ADR-0045: names are admitted where edges are.** The pair-table-derived naming kind check (`naming_referent_rule` — a map from field to *relation*; `naming_referent_kind_allowed`; `TopologyError::ForbiddenNamingReferent`, at construction and therefore at decode and in the planner's rebuild), and the three `multipath-node → {backing-signature, file-system, partition-table}` rows, so content on a multipath node inherits its detection-only refusal. Spec **15.1.0**. Closes #354; files #397. |
| #399 | WP-020 | r26 re-pin at `ee12af2` (VMID 9450, 2026-08-16 UTC, custody run 36, transcript `3dd4468c…`). |
| this | WP-000 | this handoff and `ISSUE-354_KIND_HALF_ROUND_2026-08-16.md`. |

Five non-Markdown paths in the act:
`crates/domain/src/model/{topology,topology_tests,snapshot_tests,protection,protection_tests}.rs`
(`protection.rs` doc comments only). No consumer package moved.

---

## 2. What was learned

### 2.1 The multipath omission was a fail-open, and reading would not have found it

The previous handoff framed the question as "is `multipath-node →
file-system`'s absence ADR-0011's intent?" — a question about a document.
The answer came from `device_scope_verdict`: content whose host has no
row can carry no edge, finds itself its own root, and inherits nothing;
an xfs on `/dev/mapper/mpatha` gated `Clear` ×10 at HEAD. Reading
ADR-0011 alone would have supported "deliberate". Ask the closure what
it does with the population before deciding whether its absence is
intended.

### 2.2 The rule is a relation, not a roster

The panel's objection to every kind-list design was "a second authored
list". `naming_referent_rule` authors one thing per field — *which
relation the field names* — and the admissible kinds are the pair
table's at the moment of the check. The pin test names all eight; an
unclassified field admits nothing and reds twenty tests (M7). If you add
a naming field, classify it there first.

### 2.3 Enumerate what a lens did not run

The fixed-kind round recorded that two of five adversarial lenses
produced no output. The answer to a lens that did not run is an
enumeration: `naming_admits_exactly_what_the_pair_table_admits` builds
every (field, kind) pairing — 7 × 11 relation-bound plus the open field
× 11 — and asserts the full error value on every refusal. Direction
reversal (M5) killed 73 tests; a sample would have caught it too, but the
enumeration is what makes "admits exactly" a measurement.

### 2.4 Two things I did wrong

- I ran `git checkout -- <file>` as a mutation cleanup with the whole
  implementation uncommitted in that file. It reverted everything. The
  splice script was idempotent from HEAD so nothing was lost, but that
  was luck; the memory note about this exists and I still did it. Revert
  a mutation with a second explicit edit, never with checkout.
- Bash heredocs holding backticks failed three more times this session.
  Write generator scripts with the Write tool; do not type them inline.

### 2.5 The guest took the ThinkPad's address

VMID 9450 came up on `10.7.7.67`, which the ThinkPad-apparatus memory
records as `nate@10.7.7.67`; the ThinkPad was evidently off, DHCP reused
it, and I ran `ssh-keygen -R 10.7.7.67` on the Proxmox host to bootstrap
the guest. If the ThinkPad comes back on that address, the host's
`known_hosts` for it is gone (the workstation's is untouched). Not a
defect in anything; worth knowing before the next ThinkPad sitting.

---

## 3. What is next

The chain **#347 → #360 → #354 → #333** has three links closed; **#333's
enforcement is the head**, unblocked. Its round inherits ADR-0037
verbatim: derive-and-**compare** form only; the front-runner is the
naming-field-derived frame predicate (measured to survive
`the_guard_stands_with_every_containment_edge_removed`); the golden
vector and `plan_tests.rs` regenerated in the same act with its MODEL-003
discharge. Comment left on #333.

- **#333** — the enforcement. Start from ADR-0037's "Enforcement — held"
  section and the ADR-0045 check, which is its precondition; the
  derivation path `partition → parent_table → table → parent → root` now
  has both hops kind-checked.
- **#397** — device scope by name (adjacent to #333, lands before or
  after; fail-closed candidate in the filing).
- **#392** — the extentless-target limit, with its measured candidate.
- **#319's authorization half** — unmeasured since #338 closed.
- **The per-kind `canonical_ranges` entry** — ADR-0042's revisit condition,
  unfiled.
- **#365**, **#366** — small / parallelizable. #365's open question
  (what hosts a `BackingExtent`) is exactly the open field ADR-0045 left.
- WP-050: `multipath_scoped` widening by containment root, so content on
  a multipath node reports `Reason::MultipathDetectionOnly` rather than
  the (truthful) inherited device-scope ground.

Any of these that ships Rust owes r27.

---

## 4. Operational

`-r26` → `-r27`, VMID 9451; the sequence has run void-free six times.
Creating the guest and running `settle` while the act's CI runs, then
provisioning on the merge commit, brings a sitting to ~35 minutes wall
clock from merge to teardown.
