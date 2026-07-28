# ADR-C3: Identity strength is a record property; matching is a separate verdict

- Status: Accepted
- Date: 2026-07-28
- Spec version: 3.1.0
- Requirement IDs: SAFE-003, INV-002, INV-003, PART-001, UI-009, ACC-014,
  PLAN-004, Section 6
- Resolves: SI-01, SI-02
- Decision owners: repository CODEOWNERS

Acceptance basis: filed under Section 1.11, analysed with an adversarial review
of the proposal, and delegated to the implementer by the decision owner. The
review rejected part of the original proposal; see "What was rejected".

## Context

SAFE-003 defines Strong identity as "at least one stable hardware identifier
(serial or WWN) plus size, sector geometry, and partition-table checksum all
match", and also requires that "each identity record MUST be classified".
INV-002 requires reporting strength at discovery and Section 6 requires plans to
carry it. In all three there is no counterpart record to match against, so the
definition cannot be evaluated where the requirement demands a value (SI-01).

Read literally, Strong also requires a partition-table checksum match, which no
blank device can satisfy. A factory-fresh NVMe exposing both serial and WWN
would be Weak, so PART-001 first-initialization would always take the
weak-identity path: typed device-name confirmation (UI-009) and refusal of
unattended apply (SI-02).

## Decision

**Strength is a property of one record. Matching is a verdict over two.**

An identity record is **Strong** when it carries at least one stable hardware
identifier — a serial number or a WWN — together with total size, both logical
and physical sector size, and a *positively determined* partition-table state.
It is **Weak** otherwise. This is computable at discovery, at plan creation, and
at re-probe, because it asks only what the record contains.

**Identity match** is a separate, ordered-pair verdict produced only by the
helper when it compares a plan's bound record against its own freshly derived
one. SAFE-003's "all match" clause defines *this*, not strength. The two are
distinct domain types and MUST NOT be interchangeable.

**Partition-table state is three-valued**, replacing the previous optional
checksum:

- `Present { checksum }` — a table was read and hashed.
- `Absent` — positively observed to have no table.
- `Indeterminate` — the region could not be read, or parsed ambiguously.

`Absent` is a determined value, so blank media can be Strong.
`Indeterminate` is not, so a device whose table failed to parse is Weak and
falls under the weak-identity protections. That distinction is the whole point:
under the previous single optional field, a factory-blank disk and a disk whose
GPT failed to parse were the same value.

## What was rejected, and why

The original proposal also carved out destructiveness: *"initializing media
whose partition-table state is positively absent (PART-001) is not severity 4
and does not trigger UI-009 on destructiveness grounds."* **Rejected.**

**An absent partition table does not mean absent data.** `pvcreate /dev/sdb`,
`cryptsetup luksFormat /dev/sdb`, `mdadm --create`, `mkfs.ext4` on a whole
device, and any superfloppy-formatted exFAT stick all produce a device whose
table state is positively observed as `Absent` and whose entire content is user
data. MODEL-002 explicitly permits container, volume, and file-system layers
with no intervening partition table, and FS-004 requires detecting LVM PV, LUKS,
BitLocker, Linux RAID, and pool members — every one of which occurs whole-disk.

The carve-out would therefore have created a silent whole-device destruction
path for exactly the media most likely to hold an unpartitioned filesystem.

Severity and confirmation strength MUST continue to derive from **detected
content** (FS-004, INV-004), never from the absence of a partition table. A
device with `Absent` state and no detected content is a severity-4 destructive
target like any other; it simply is not additionally *weak-identity* if it
carries a serial or WWN.

The friction SI-02 complained about is real but was mis-attributed. It came from
strength, and this decision fixes it there. It did not come from severity, and
lowering severity would have bought convenience with data.

## Consequences

- Two domain types where the spec named one. `IdentityStrength` and
  `IdentityMatch` must not be substitutable; passing one where the other was
  meant is a bug the type system has to catch.
- Strength is hash-visible: it is body content under ADR-C2, so an adapter
  upgrade that begins reporting a WWN for a device that previously exposed none
  will invalidate outstanding plans bound to it. That is correct — the binding
  genuinely changed — but it is a rejection class that needs a user-actionable
  message under UI-010, not a bare mismatch.
- A strong-identity blank removable now qualifies for SAFE-003's replug
  path-change allowance, which the literal text denied it. The bullet's other
  conditions still apply.
- `Indeterminate` gives corrupt-table devices a weak-identity classification
  they did not previously have, which is a tightening.

## Verification

- Strength is computable from a single record with no counterpart, at all three
  points, proven by unit tests with no comparison input available.
- A blank device with a serial classifies Strong; the same device with an
  unreadable table classifies Weak.
- A device with `Absent` table state and a detected whole-disk LUKS header or
  LVM PV signature still plans at severity 4 and still triggers UI-009. This is
  the regression guard for the rejected carve-out.
- Golden vectors covering all three table states, cross-language (MODEL-005).
- Test tier: T1, unprivileged.

## Revisit conditions

- A platform appears where "positively observed absent" cannot be distinguished
  from "could not read", collapsing the three-state distinction.
- Strength gains a third level, at which point the ordering and its effect on
  UI-009 must be restated rather than inferred.
