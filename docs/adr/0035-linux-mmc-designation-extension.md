# ADR-0035: The Linux mmc-class designation extension

- Status: Accepted
- Date: 2026-08-13. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/ADR-0034_MMC_EXTENSION_ROUND_2026-08-13.md`, an
  untracked session artifact; everything load-bearing is restated
  here), under ADR-0034's own revisit condition, fired in its
  sanctioned direction the day that ADR was accepted: a qualifying
  sitting measured a serial source for a currently undesignated Linux
  attachment class, and the table extension lands with its rows.
  Recorded and implemented in one autonomous arc — merging is not
  acceptance.
- Spec version: 12.12.0 (minor under §0.1 — one previously
  undesignated cell gains a designation; no existing text narrows)
- Work packages blocked: none (the cell's first consumer is WP-L100's
  future mmc enumeration, unbuilt; nothing waits on it today)
- Requirement IDs: Section 5, MODEL-004, SAFE-003, SAFE-005, ADR-C4,
  ADR-0018, ADR-0019, ADR-0034
- Decision owners: Nate McBride

## Context

The S5 sitting (`docs/quality/observability.md`, the SI-33/SI-28
successor protocol's fifth arm, 2026-08-13) measured what no
qualifying record had before: on a native `sdhci-pci` controller, the
SD card's own CID register is world-readable to an unprivileged
client — the full 128 bits, carrying the medium's PSN beside its
manufacturer, OEM, product name, revision, and manufacture date —
byte-stable across suspend, reinsertion, and reboot, and distinct
across two same-manufacturer, same-date media whose host-assigned RCA
was identical. The same sitting measured the medium identity-invisible
behind three USB bridges, two of them a same-model pair sharing one
placeholder constant. The CID is the only medium-attributable
identifier any Linux record has ever measured, and ADR-0034 left its
attachment class undesignated because no rows existed. They exist now.

## The extension

ADR-0034's designation table gains one cell:

| Platform | Attachment class | Identifier | Designated source |
| --- | --- | --- | --- |
| Linux | native MMC-attached block device | serial | The `cid` attribute of the mmc device node the block device's sysfs `device` link resolves to, bytes verbatim as read — trailing newline included |
| Linux | native MMC-attached block device | WWN | **Undesignated** |

**The attachment class** is a block device whose sysfs `device` link
resolves under an `mmc_host/*` node — the structural rule S5c
measured (`/sys/block/mmcblk0/device → …/mmc_host/mmc0/mmc0:aaaa`).
Unlike ADR-0034's USB rule there is no ancestor search: the linked
node is the source's home.

**The source is the register, not its projection.** The kernel also
parses a `serial` attribute — the PSN sub-field — out of the same
register. Designating it was the round's recorded rejection: it is a
transformation between the medium and its name, and it is strictly
less collision-resistant — the S5 pair shares `manfid`, `oemid`, and
manufacture date, so the non-PSN fields are the context that makes a
PSN collision survivable, not padding. Choosing the smaller field to
honor the label "serial" would be naming by label, which this
register's history keeps refusing.

**Everything ADR-0034 established applies unchanged**: verbatim
includes the trailing newline; a measured absence (`ObservedAbsent`)
yields an operand-eligible weak name; a failed read yields an
indeterminate non-operand still present in the body; naming inputs
flow through a bytes-preserving path, which the delivered
`read_attribute` is not; the host-assigned bus address (the RCA,
measured identical across two media and reassigned identically across
reinsertion and reboot) is path material and never a naming input.

## Options considered

### The kernel-parsed `serial` (PSN) attribute

Rejected, as above: a projection with strictly weaker collision
resistance, chosen only to match a field label.

### Waiting for eMMC and second-host measurements

Rejected: undesignated cells are already fail-closed, the measured
class is the strongest identity evidence in the register, and it is
the route SI-28's round five will lean on. Withholding buys no safety.

## Decision

The cell above is normative, versioned with the evidence contract
exactly as ADR-0034's cells are; changing it is hash-visible and lands
only with a spec change. WWN stays undesignated for the class. SI-28,
its floor, and its round five are untouched — these rows enable round
five and do not decide it.

## Consequences

- **Positive:** Linux gains its first medium-attributable naming
  route: a card in a native slot names from its own silicon, with the
  bridge-attached population remaining exactly as fail-closed as
  ADR-0034 left it. The S4/S5-measured collision families and this
  designation now sit on opposite sides of the same table, which is
  the shape SI-28's round five needs.
- **Negative, accepted knowingly:** the evidence base is SD cards on
  one host and one controller; eMMC (same interface contract,
  different population — soldered, non-removable, ubiquitous) is
  designated by interface without its own rows yet, a priced judgment
  the round recorded rather than a scope-out, because scoping by card
  type would make the class boundary read a field no naming rule
  consumes.
- **Evidence obligations:** (1) the first eMMC capture — the same
  cells S5b/S5c measured, on an eMMC device; (2) a second-host,
  second-controller capture of the same cells; (3) these ride
  whatever Linux sitting next runs, alongside ADR-0034's outstanding
  canonical-path obligation, now discharged by FR4.

## Verification

- When an mmc enumeration first lands (WP-L100's future work): the
  serial bytes in a `NamingFields::PhysicalDevice` for this class
  equal the linked node's `cid` file content verbatim; no naming
  input flows through the text-decoding read path; the `serial`
  attribute is not read for naming.
- Any text implying the PSN sub-field feeds naming, or that the RCA
  or bus address is identity, is an error against this ADR.

## Revisit conditions

- The eMMC or second-host capture measures a different register
  shape, a constant CID population (the S4 shared-constant story
  reappearing in silicon), or an unstable read — the cell deserves
  re-examination on those rows.
- A kernel change moves or reformats the `cid` attribute —
  hash-visible by construction, re-examined rather than shimmed.
- ADR-0034's own revisit conditions, inherited unchanged.
