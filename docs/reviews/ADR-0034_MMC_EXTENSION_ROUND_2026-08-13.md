# The mmc-class designation extension — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. Everything
load-bearing is restated in the ADR that lands the decision.

**What is being decided.** ADR-0034's revisit condition fired in its
sanctioned direction the day it was written: "a qualifying sitting
measures a serial or WWN source for a currently undesignated Linux
attachment class — the table extension lands with its rows." The S5
sitting is that sitting: on a native `sdhci-pci` controller the SD
card's CID register is world-readable, byte-stable across suspend,
reinsertion, and reboot, and per-medium distinct where the
host-assigned RCA is not. The extension adds the (Linux, native
MMC-attached block device) cell to the designation table. The open
question is which attribute is the designated serial source.

## The attachment class

A block device whose sysfs `device` link resolves under an
`mmc_host/*` node — the structural resolution S5c measured
(`/sys/block/mmcblk0/device → …/mmc_host/mmc0/mmc0:aaaa`), the
analogue of ADR-0034's nearest-USB-ancestor rule. The CID-bearing
node is the resolution target itself, so this class's rule is simpler
than USB's: no ancestor search, the linked node is the source's home.

## Routes

- **(a) Designate the `cid` attribute** (recommended): the full
  128-bit register as the node's `cid` sysfs file returns it, bytes
  verbatim, trailing newline included. The serial identifier for this
  class is the register itself.
- **(b) Designate the `serial` attribute**: the kernel-parsed PSN
  sub-field (32 bits of the register).
- **(c) Designate nothing yet; wait for eMMC and second-host
  measurements.**

## The adversarial pass on route (a)

1. **"The hex text is a rendering, not the identifier."** The sysfs
   file is the named source and its returned bytes are what the
   contract reads — the identical posture ADR-0034 takes for the USB
   `serial` attribute, itself a text rendering of a descriptor. A
   raw-register byte fetch would require an interface no qualifying
   record has measured. Sustained as consistent, not sustained as a
   defect.
2. **"PSN is the serial; the CID smuggles non-serial fields into
   naming."** The S5 pair answers this concretely: same `manfid`,
   same `oemid`, same manufacture date — the non-PSN fields are not
   discriminating padding, they are the context that makes a PSN
   collision survivable. Manufacturers reuse PSNs across product
   lines; the full register is strictly more collision-resistant than
   any sub-field, and ADR-0019 prices weak names in collisions. Route
   (b) chooses a weaker name to honor a word ("serial"), which is the
   naming-by-label mistake the register keeps refusing.
3. **"The kernel already parsed `serial` out; using `cid` re-derives
   it."** Backwards: `serial` is the kernel's *projection* of the
   register; `cid` is the source. Designating the projection would
   put a transformation between the medium and the name.
4. **"eMMC is unmeasured."** True: S5 measured removable SD cards on
   one host. The interface contract (`cid` on the mmc device node) is
   identical for eMMC, but the designation's evidence is SD's — the
   ADR records eMMC's first measurement as an evidence obligation and
   does not scope it out, because scoping by card type would make the
   class boundary depend on a field (`type`) no naming rule reads.
   Recorded as a priced judgment, reviewable.
5. **"The RCA rode along in the bus address — is the linkage rule
   stable?"** S5 measured the RCA reassigned identically across
   reinsertion and reboot on this host, and measured it identical
   across two different cards — it is host-assigned and appears in
   the *path*, never in the name. The designated source is the linked
   node's `cid` file content; the path spelling is resolution, not
   naming input, exactly as ADR-0034's traversal is.
6. **One host, one controller.** The class has a single-host evidence
   base (sdhci-pci, kernel 6.12). A second-host capture is filed as
   an evidence obligation beside eMMC's; the designation does not
   wait for it, on ADR-0034's own precedent (one topology instance,
   the structural claim measured).

## Rejected routes, recorded

- **(b) `serial`/PSN**: a projection of the source with strictly
  weaker collision resistance, chosen only to match a field's label.
- **(c) waiting**: undesignated cells are fail-closed already, but
  this class's measurement exists, is the strongest identity evidence
  in the register, and is the one SI-28's round five will lean on;
  withholding the designation buys no safety and delays the only
  medium-attributable naming route Linux has.

## Decision carried forward

Route (a): the (Linux, native MMC-attached block device) serial cell
designates the linked mmc node's `cid` attribute, bytes verbatim,
newline included; WWN stays undesignated for the class (no WWN-class
source exists on it); ADR-0034's `ObservedAbsent`, failed-read, and
bytes-path rules apply unchanged. Evidence obligations: first eMMC
capture; a second-host/second-controller capture. Spec version minor:
an addition to an undesignated cell, no existing text narrows.
