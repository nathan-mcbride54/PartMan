# Changelog

All notable implementation changes are recorded here. Specification changes
remain controlled by the changelog in `AGENT_BUILD_SPEC.md`.

## Unreleased

### Changed

- **The macOS second-reader readback is discharged** (2026-08-08). An
  independent reader session — not the session that produced any of the three
  records — retrieved both `partman-macos-sitting-2` transcripts and the M10
  transcript through their evidence-store locators and rehashed each to its
  recorded digest, all three matching; sitting 2's byte lengths matched as
  recorded, and the M10 capture's full 172-entry SHA-256 inventory was
  additionally rehashed with every entry matching. Each record's custody
  caveat travels into the discharge rather than being erased by it: sitting
  2's digests were computed after retention from an unmodified copy, so its
  matching rehash confirms the copy unchanged since 2026-08-05 and nothing
  stronger, while M10's digest was recorded at retention and its rehash
  carries the full property. The readback also surfaced and recorded an
  omission of sitting 2's class in the M10 record — a digest with no byte
  length, where the custody rule requires both; 23 516 bytes is now recorded
  as a readback-time measurement, stated as such. This discharge removes the
  custody caveat the SI-34 currency note and SI-39's dependency paragraph
  carry; it resolves neither issue and decides nothing on the register.

- WP-010 files **SI-39**, the SAFE-003 / INV-003 conflict, under Section 0.2's
  requirement to file rather than silently pick a side. SAFE-003 says "A blank
  device can therefore be Strong"; INV-003, as ADR-0013 amended it in spec
  6.0.0, forbids the unprivileged layer reporting "a medium as positively
  without a table" where its contract does not separate that case; and the
  macOS increment 6 matrix measured that non-separation directly — `blank-512`
  and media carrying ext4, an mdraid member, LUKS2 and LVM2 all project
  byte-identically. So on macOS a client may not report `Absent`, the state is
  not positively determined, and the device is Weak where SAFE-003 says it can
  be Strong.

  **The filing records that this repository created the conflict**, hours
  before finding it: INV-003's governing sentence is ADR-0013's, and that ADR's
  adversarial round did not reach SAFE-003. A register that files a conflict as
  discovered when it was introduced misleads the next round about where to look.

  Classified a direct blocker; the authoritative count moves to ten, seven of
  them direct. Four resolution options are recorded and none is recommended,
  including the one with a recorded data-loss path (amending INV-003 so a
  medium indistinguishable from blank is reportable as `Absent`, when PART-001
  initializes blank media). The `Present` face of the same INV-003 sentence
  reaches all three platforms and overlaps SI-35's open axis question; it is
  recorded as adjacent and **deliberately not decided**. The macOS rows carry an
  outstanding second-reader readback, and the filing says so. No requirement is
  amended and no specification version changes.

- **spec-change 6.1.0: WP-035 gains unprivileged whole-device enumeration and
  the INV-003 reach declaration.** The read-only CLI may now report real
  attached devices — raw identifier strings labelled by the interface that
  reported them, under session-local selectors — so the project has a working
  read-only alpha while the register decisions proceed in parallel.

  **Minor, an addition.** No requirement in Sections 2, 3, 5, 6 or 7 is
  retexted. INV-003 is *implemented in part*, not amended: 6.0.0 created the
  reach obligation hours earlier and nothing implemented it. WP-035's charter
  sentence survives verbatim and governs the new scope in full, and the change
  adds prohibitions rather than relaxing any — no strength, no ADR-C3 state or
  checksum, no typed Section 5 node, no artifact hash, no stable handle, no
  same-device claim, no protection or CAP-003 verdict, and the standing gated
  list still travels in every answer.

  **WP-W100/WP-L100/WP-M100 are untouched.** Narrowing them would remove scope
  from existing text and would be a major bump; this enumerator is interim and
  defers to them.

  M0.5's gate is **extended, not rewritten**, in the shape 4.2.0 used when it
  created M0.5. It deliberately does not require three live adapters: a
  platform whose access route is an open structural question ships its reach
  declaration and a typed `not-implemented` answer naming the recorded
  decision that defers it. Coupling M0.5's exit to that question would gate
  every sequential milestone after it on a structural argument this change
  exists to avoid importing.

- **SI-33's liveness precondition is recorded as discharged** (decided
  2026-08-05). Its filing said "until that passes on real hardware, this is a
  hypothesis"; the 2026-08-04 sittings passed it — immediate re-read and
  sixty-second idle gap both moved, close-before-event/reopen survived 3/3
  across true no-handle windows. **SI-33 stays Open**: what the pass
  discharges is the precondition, not the issue.

  The entry records three limits the protocol declared before any data
  existed, because the headline sentence would otherwise be read as removing
  them: the positive cannot be attributed to exchange-synchronous detection (a
  background poll explains it equally well, so the ceiling is "no staleness
  observed under these conditions"); it is bounded to the slot-exchange family
  on one reader, one bridge, one build; and the exposed reading is **not
  globally monotone**, a measured decrease across a PnP-arrival boundary making
  an equality-only witness unsafe, so a design must characterize the counter's
  epoch or use another witness. No axis, design, or placement is decided, and
  **SI-28's interim conservative floor is not relaxed** — that route is the
  design, not the liveness pass.

  Two stale statements are cleared in the same pass: Part 6's precondition 1
  still described M10 as untaken hours after it was taken, and SI-38's
  Dependencies paragraph still asserted a gate on SI-35 directly beneath its
  own Resolved banner. Both are now historical rather than contradictory, and
  the M10 sentence carries the outstanding second-reader readback.

- **spec-change 6.0.0: SI-38 is resolved by ADR-0013.** INV-003's detection
  duty is scoped by privilege, and the unprivileged discovery layer must
  publish the reach of its platform contract. Accepted by Nate McBride on
  2026-08-05 after an adversarial round that changed the recommendation.

  The conflict was real and measured: INV-003 required the discovery layer to
  detect hybrid and inconsistent partition tables, SAFE-002 places that layer
  at no elevation, and the enumerated client projections on Linux
  (2026-08-03), Windows (2026-08-04) and macOS (2026-08-05) all fail to
  separate a healthy GPT from one whose two tables describe different
  partitions. M10 (2026-08-05) located the separating fact in the backup
  table, behind a read the unprivileged client is denied on the same
  attachment.

  **Major under §0.1** because it narrows an existing MUST rather than adding
  one. The full detection set survives on the privileged path; the
  unprivileged layer is no longer required to do what it measurably cannot,
  and is instead required to say so.

  Two rejections are recorded rather than omitted. Reporting the remainder as
  undetermined is **unimplementable** — the client cannot identify the
  remainder, a conflicting table presenting as an ordinary valid GPT, so the
  rule would either never fire or mark every GPT undetermined; this was the
  first recommendation and review killed it. Qualifying **SAFE-002** was
  rejected on precedence: a Section 3 constraint may not be bent to satisfy a
  Section 7 functional requirement without inverting Section 0.2's ordering.

  Where the reach does not cover a state, the privileged re-discovery HLP-002
  already requires before the first write determines it; the unprivileged
  layer neither refuses on the ground of its own blindness nor represents that
  blindness as a determination. SAFE-005 is unchanged. SI-38 moves to
  Resolved, the register's count returns to nine, the transitive-blocker class
  returns to empty, and **SI-35 is unblocked and remains a direct blocker,
  undecided**.

- **M10, the privileged comparison leg, is taken** (2026-08-05), and with it
  **no preregistered cell on any platform is `not yet taken`**. It ran in a
  GitHub-hosted `macos-15` runner — `RELEASE_ARM64_VMAPPLE`, macOS 15.7.7, an
  ephemeral Apple Virtualization Framework guest destroyed at job end, which
  the cell's environment rule admits as a hosted macOS test environment. The
  harness captures both halves of every attachment itself, because this is a
  different machine from the M1–M8 sitting and M10 asks about the *same*
  attachment.

  On all seven fixtures the unprivileged client's raw read was denied `EACCES`
  while root read the device, and every helper byte-range digest equals the
  source image's. **The decisive pair separates for the helper**: identical
  first-64-KiB digests and differing last-64-KiB digests, placing the two
  tables' disagreement in the backup, which no client interface on any
  measured platform reports — and this sitting's own client half reproduces
  that blindness. The four signatures the client called byte-identical to
  blank each carry a distinct helper head digest.

  Two incidental observations are recorded because they refine the earlier
  sitting: the denial is `EACCES` here where the physical host gave the
  unexplained `EPERM`, making that a contrast rather than a lone oddity
  without attributing a cause; and the node modes differ. The client half also
  reproduced the decisive-pair byte-identity on a different macOS major
  version, weakening the earlier record's "one host, one build" limitation for
  that finding only. Limitations recorded: shared-infrastructure runner, no
  checksum-pinned image, first and last 64 KiB only, `dd` through
  `/dev/rdiskN` and no other privileged interface. The second-reader readback
  is outstanding. No register disposition changes, no SI-34 or SI-35 option is
  decided, and SI-34's freshness-projection element is **not** satisfied — it
  names a projection that does not yet exist.

- WP-010's third evidence-currency pass records the macOS observability record
  on the two register surfaces that understated it (issue #155). Precondition
  1 now says all three platforms have the non-elevated record it asks for, and
  SI-34's evidence clause gains a bounded status note: its **observability
  element is satisfied; every other element remains unsatisfied**. The note
  states the reading it rests on — precondition 1 defines the record as
  "established empirically and non-elevated", so macOS's untaken privileged
  leg does not hold that element open — and records the narrower reading as
  **rejected rather than ignored**, with what changes if it is preferred. It
  also states what is missing on macOS under either reading: M10 is untaken,
  so there is no privileged comparison leg, and SI-34's separate both-views
  freshness requirement is unmeasured there and unmeasurable until M10 exists.
  SI-34 is not resolved, its evidence clause is not discharged, no option is
  decided or ranked, and no state, class, dependency, or metadata changes.

- The increment 6 **macOS matrix is executed** (2026-08-05, Apple Silicon,
  console session, macOS 26.3.2 build 25D2140, SIP enabled), valid on its
  second sitting. Sitting 1 is **void on two harness defects** and is retained
  with the amendments it produced: tool versions were captured through
  `diskutil version` and `hdiutil version`, neither of which is a real verb,
  so no version was recorded — now replaced by a SHA-256 over every declared
  binary; and the post phase ran without the reboot M5 depends on, detected
  only afterwards by disk-numbering inference — now a hard `kern.boottime`
  gate that voids M5 in-transcript, tested by forcing each outcome before the
  amended harness shipped. The favourable reading of defect 1, that OS-bundled
  tools have no version beyond the recorded OS build, is recorded as
  considered and rejected.

  Substantive results: **macOS is the third platform whose enumerated
  unprivileged projection does not separate the decisive SI-35 pair** —
  `diskutil`'s structured output is byte-identical and unnormalized between a
  healthy GPT and one whose two tables describe different partitions. **Every
  non-native signature projects byte-identically to a blank disk** (live ext4
  with a stale mdraid superblock, an mdraid member, a LUKS2 container, an LVM2
  orphan), while GPT, MBR and APM are each named distinctly. APFS container
  membership and its UUID are client-readable, the UUID is carried by both
  interfaces and is stable across a verified reboot, and the unprivileged raw
  device read is denied `EPERM` — recorded as observed and unexplained, since
  `EACCES` was the expected errno for a user outside the owning group. M9 is
  `not established` (Apple Silicon has no Fusion Drive) and **M10, the
  privileged comparison leg, stays `not yet taken`** for want of a disposable
  macOS VM. No separation claim is made from the `ioreg` interface, whose
  capture is a whole-registry dump with no normalizer declared before it. The
  second-reader readback is outstanding. No register disposition changes, no
  SI-35 option is decided, and no existential hypothesis is refuted.

- WP-010 files **SI-38**, the INV-003 / SAFE-002 conflict, under Section 0.2's
  requirement to file rather than silently pick a side. INV-003 requires the
  discovery layer to detect hybrid and inconsistent partition tables; SAFE-002
  places that layer at no elevation; and the 2026-08-03 Linux and 2026-08-04
  Windows runs establish that the unprivileged projection distinguishes
  neither. HLP-002's independent re-discovery does not dissolve it, being
  scoped "before the first write" — plan time, not inventory time — so at
  inventory the unprivileged layer is the sole observer. Classified a
  transitive blocker to SI-35 rather than an input, because an ADR may not
  amend a MUST and every available resolution is a normative amendment; the
  register's count moves to ten and the previously empty transitive class is
  repopulated. Four resolution options are recorded and none is recommended.
  No requirement is amended, no specification version changes, and SI-35 is
  neither decided nor pre-empted.
- WP-010's bounded evidence-currency pass stops SI-35's register row
  understating evidence WP-035 has already established (issue #142). Two of
  its three acceptance-evidence categories are now recorded as discharged —
  the loop category by the descriptor-bound non-WSL run of 2026-08-03, valid
  on its third sitting, and the Windows category by the completion rerun of
  2026-08-04 — and the third, the chosen-option refusal demonstration, is
  recorded as blocked on a decision nobody has taken rather than on a
  measurement anyone can schedule. The round-four observability-status
  sentence now names macOS as the only platform still unestablished. SI-35
  stays **Open**; no option is decided or ranked, no existential hypothesis
  becomes refuted, no finding extends past the projection its run covered,
  and no class, dependency, hash-visible value, requirement, or evidence
  record changes.
- The increment 6 real-partitioned-Linux matrix is executed in full
  (sitting of 2026-08-04): disposable Proxmox VM from the digest-verified
  jammy image, two explicitly authorized SanDisk fixture media passed
  through, six declared layouts provisioned by the separated setup actor
  with per-layout digest brackets, and rows L1–L10 all `observed`. The
  substantive results: the ordinary-client baseline truly lacks raw block
  access while `disk`-group membership alone flips it; the client/helper
  signature asymmetry is measured in both directions — an empty cached
  client projection over live ZFS labels (event-time-cache mechanism
  recorded), and the live-plus-stale L-F pair answered with exactly the
  stale signature by both single-answer interfaces while only the
  enumerating probe reveals both; LVM2's member-independent designator
  (VG id) is helper-only; identity facts and designators survive replug
  and reboot; and two byte-identical media collide as silent
  last-writer-wins on `by-uuid`/`by-partuuid`/`by-label` with no
  duplicate signal, only the bus-serial `by-id` staying distinct. The
  sitting's six instrument corrections and incidents are recorded, each
  caught by a declared gate before any cell was derived, including a USB2
  passthrough wedge recovered by an XHCI reattach. The matrix's section
  heading, deleted accidentally by the 2026-08-03 mechanism-amendment
  commit, is restored, and the Method section's setup-write sentence is
  widened exactly as the protocol pre-filed. Custody is complete with an
  independent second-reader rehash; the VM is destroyed with post-destroy
  verification; the status header, README M0.5 prose, and WP-035 row are
  trued.

- The SI-33/SI-28 successor protocol's S1, S2, S2b, and S3 arms are
  executed on the reattached parent-record reader (sitting of
  2026-08-04), completing every arm of the protocol. S1: `moved` in all
  three trials — `count Δ=+1` per genuine exchange, read from a fresh
  handle after a true process-local no-handle window, every empty-slot
  assertion and arrival bracket clean; the fail-open `unchanged` outcome
  never appeared, establishing the register's close-before-event/reopen
  survival sequence on this reader at three trials. S3: L4's trials 2
  and 3 both `count Δ=+1`, final at first re-read and stable through the
  idle re-reads, bringing the leg to its requested three trials. S2: the
  boundary-1 counter reset is measured (five events above the epoch
  floor before a reader re-arrival, at the floor after), the storage-node
  PDO name qualifies as an unprivileged epoch signal — changed across
  every induced boundary and the S2b reboot, stable between, counter-
  independent — and ContainerId and the USB-node PDO name are refuted;
  the qualifying token is recorded with its limit (an allocation name,
  not a globally unique epoch id). Turn-based operation and its sample
  latency are declared in the record; custody is complete with an
  independent second-reader rehash; the status header, README M0.5
  prose, and WP-035 row are trued.

- The S4 card-move rider is executed (sitting 3, 2026-08-04), completing
  every S4 arm. The operator exchanged the cards between the attached
  units — the swap declared as the executed form of the preregistered
  one-card move — and the storage-layer record is invariant under the
  exchange: the exchange is visible only as media facts (sizes and
  volumes travelling with the cards). On a pair sharing one constant at
  every layer, follows-card versus follows-reader is undecidable by
  value, with the reader attribution resting on the empty-slot form,
  which held again on both units. Recorded at full strength: both units
  re-arrived inside the exchange window despite the operator reporting
  the readers stayed attached (cause not attributable unprivileged), and
  across those re-arrivals the serial-derived instance identity migrated
  ports — a first-arrival artifact, not a unit identity — so physical-
  unit continuity across the exchange is unverifiable from any read
  surface. Custody is complete with an independent second-reader rehash;
  the status header, README M0.5 prose, and WP-035 row are trued.

- The SI-33/SI-28 successor protocol's S4 collision test is executed on a
  same-model pair (sitting 2, 2026-08-04): both NS1081-model units report
  one identical 15-character placeholder serial at the USB-descriptor and
  storage layers on all four LUNs — the preregistered collision — with
  Windows re-keying the second-arrived unit by port and clearing its
  `CM_DEVCAP_UNIQUEID` bit; the empty-slot rider is observed on both
  units; the card-move rider stays `not yet taken`. The named distinctness
  instrument (distinct USB serials) was unavailable for the reason under
  test; distinctness is established by simultaneous distinct-port
  enumeration and declared as a substitution. Custody is complete with an
  independent second-reader rehash, and the sitting's three instrument
  failures — a mis-dated transcript header, a recalled constant, and a
  decode contradicting its own raw value — are recorded as corrections
  caught before any cell moved. Additionally: the SI-35 loop and
  Windows-rerun records' outstanding second-reader obligations are
  discharged by an independent 2026-08-04 retrieve-and-rehash of all six
  named artifacts (every digest matching), recorded beside the designated
  readbacks on their pull requests with archive locators and custodian now
  named in both records; the rerun subsection's status line still read
  `not yet taken` under its taken-and-valid heading and now states its
  executed status; and the observability status header, README's M0.5
  prose, and the WP-035 row are trued to all of the above.

- **The SI-35 Windows completion rerun was taken on 2026-08-04 and is valid**,
  closing the second of SI-35's three acceptance-evidence categories. All
  three added gates are satisfied: R1 total retention (a 48 KB property-bag
  record, not a summary — the 2026-08-02 query-and-discard defect does not
  recur), R2's restored digest bracket (all seven fixtures' post-detach VHD
  digests equal their pre-attach pair, so read-only is measured rather than
  documented), and R3's mandatory index fallback (both `MSFT_Disk`-absent
  fixtures probed at every `Win32_DiskDrive`-supplied index). Two consoles
  with opposite, separately recorded privilege assertions, the measurement one
  launched ordinarily and asserting both non-elevation and absence of the
  Administrators group.
  **W-H1, W-H2, and W-H3 are all refuted.** The refutation was evaluated
  mechanically: 76 retained `MSFT_Disk`/`MSFT_Partition` fields compared
  field-by-field between the healthy control and each of the conflicting,
  damaged-primary, and missing-backup fixtures, excluding a named list of
  session-local addressing fields — **exactly one field differs in each, and
  it is `Location`, the backing file's path**, which is by-name provenance
  rather than disk state. Every named status surface is equal, the partition
  rows are identical (so under W-H1's own wording the primary table was
  parsed and presented without complaint), and the layout IOCTL agrees.
  **W-Q4 is answered `other (verbatim)`**: neither `hybrid-mbr-gpt-512` nor
  its `mbr-basic-512` control reports any scheme — both are absent from
  `MSFT_Disk` and their layout IOCTL fails `ERROR_IO_DEVICE` through a
  succeeding zero-access open, with nothing flagging the aliasing.
  A surface the parent run discarded is now retained and carries a new fact:
  `MSFT_PhysicalDisk` reports a row for **all seven** fixtures including the
  two `MSFT_Disk` omits, so the enumeration gap is specific to `MSFT_Disk`.
  Recorded rather than smoothed over: R1 and this file's no-operator-paths
  rule genuinely conflict at `MSFT_Disk.Location`, resolved by elision as the
  protocol treats embedded paths elsewhere — a redacted citation copy is
  cited and the raw copy stays local — and the measurement script now elides
  at capture time. The sitting decides no SI-35 option, supplies no
  chosen-option refusal proof, and claims nothing about interfaces it did not
  enumerate. Second-reader readback is required before reliance.
- **The SI-35 hardened non-WSL confirmation protocol was taken and passed as
  valid** on its third 2026-08-03 sitting, in fresh disposable VM 9423 at
  `b231e0f`, after two void sittings whose instrument amendments are
  recorded. Every validity gate passed: all ten sessions' descriptor-bound
  bindings and byte-continuity hashes, the distinct-inode and repeat-attach
  controls, projection stability, trial coherence, the preallocated-node
  event shape, and the unprivileged reader's negative assertions (uid 1001,
  no `disk` group, zero `CapEff`, denied direct loop reads). The result: the
  named candidate client projection is **`non-separating`** for the decisive
  healthy/conflicting GPT pair — byte-identical in every valid trial — and
  the labelled privileged `blkid -p`/`wipefs -n` comparison was likewise
  non-differing. The WSL2 promotion hold is lifted and **M0.5's loop
  criterion is satisfied**, pending the protocol's second-reader rehash of
  the retained transcript (SHA-256 `76bbd9e1…52b994`, raw capture
  `8af58b26…c7700a`). The run chooses no SI-35 option, supplies no
  chosen-option refusal demonstration, does not substitute for SI-34's macOS
  and real-partitioned-Linux rows, and refutes no existential H-separation
  hypothesis. VM 9423 was destroyed with post-destroy verification; the two
  host-attached USB devices were never referenced and are unchanged.
  A post-result review of the instrument against gate 7 found and closed a
  latent false-pass path: two *successfully captured but empty* udev entries
  would have compared equal and printed `non-separating`, where gate 7
  requires `inconclusive (udev coverage gate)`. The instrument now counts
  retained properties per subject and reports `observed(absent)`, failing the
  run. Adding a gate after seeing output is the move the protocol distrusts,
  so the claim is bounded: the gate is strictly stricter — it can only turn a
  pass into inconclusive — and the verdict was **re-derived**, not assumed, by
  re-running the amended projection over the digest-verified retained capture
  off the sitting host on a different unprivileged Linux machine, with an
  identical verdict and every gate passing. Coverage was 12 retained
  properties for the disk and 23 per partition in every session.

- The SI-35 instrument's second sitting (fresh VM 9422 at `bc922f0`) is
  recorded as **void (gate 7)** and the instrument is amended a second time,
  recorded before any subsequent run's output. The first amendment held:
  every control and trial-coherence comparison passed with `DISKSEQ`
  dropped. Two narrower evaluation defects voided the run: the stability
  comparison covered udevadm's whole rendering, and the `S:` symlink block
  renders its set in varying order — the same nondeterminism `DEVLINKS`
  shows, in a section that is not part of gate 6's projection; and the event
  requirement demanded a disk `add` that a preallocated loop node never
  emits — this kernel pre-creates `loop0`–`loop7`, so attach produces disk
  `change` plus one `add` per partition, which the captured streams show
  exactly. Amendments: stability now compares the gate 6 projection (the
  `E:` sequence with the declared `DEVLINKS` canonicalization), and the
  event gate requires adds ≥ partitions plus at least one disk `change`.
  The void run's decisive-pair line remains unusable and unquoted. Gate
  texts, schedule, trials, hypotheses, and outcome rules untouched;
  transcript retained externally, SHA-256 `f435cf5b…be5974`.

- The SI-35 instrument's first sitting is recorded as **void (gates 4 and
  7)** and the instrument is amended, with the amendment recorded before any
  subsequent run's output. The capture half completed all ten sessions
  cleanly in fresh disposable VM 9421 at `491d10f`; the unprivileged
  projection half then voided the run on three instrument defects the first
  real kernel exposed: `DISKSEQ` — a kernel-assigned monotone attach counter
  not among the six preregistered droppable keys — varied per attach and
  failed every control comparison; `udevadm info` rendered `DEVLINKS`'s
  symlink set in varying order between back-to-back captures, failing
  byte-stability; and the passive monitor's netlink subscription raced the
  attach's first event burst, capturing 2 of 3 required add events. The
  decisive-pair output of the void run is deliberately unused and unquoted.
  Amendments, justified by the validity failures alone: `DISKSEQ` joins the
  droppable keys (session plumbing, same class as `USEC_INITIALIZED`); one
  declared token-order canonicalization for `DEVLINKS` that preserves every
  token; and the monitor now waits for its readiness banner and refuses
  rather than racing. Gates, schedule, trials, hypotheses, and outcome rules
  are untouched. Raw capture and transcript retained externally, transcript
  SHA-256 `2c2528d7…a71d067`; no hypothesis row moves.

- WP-035 delivers the SI-35 hardened-protocol instrument as the second and
  only other registered higher-tier selector,
  `cargo xtask test --tier 2 --profile destructive --acceptance
  si35-loop-capture`, plus its unprivileged counterpart
  `cargo xtask si35-project --raw <file>`. The capture half runs, under the
  2e acceptance's exact native-Linux/no-WSL/explicit-elevation gate, the
  **compiled preregistered schedule** — gate 5's negative controls (two
  generation roots for distinct inodes, plus a repeat attach), six
  order-balanced alternating trials of the healthy and conflicting fixtures,
  and a closing healthy control — each entry one crate-owned
  `run_probed_session` inside a fresh 0700 scratch that is created-or-refused
  and cleaned exactly, with passive `udevadm monitor` block-event capture
  around each session and an environment record digesting the instrument
  binary, kernel config, udev ruleset, tool versions, and fixture manifest
  before the first attach. It emits raw JSON-line records and registers no
  destructive suite. The projection half **refuses elevation**, records the
  measurement user's negative assertions (no disk group, empty capabilities,
  denied direct loop reads), applies the frozen normalizer — exactly the six
  preregistered plumbing keys dropped, everything else retained — and
  evaluates the stability, control, trial-coherence, event, and decisive-pair
  gates, exiting nonzero if any gate voids the run. Monitor output is
  evidence only and is never parsed for addressing. No measurement is taken
  by this change and every observability result cell stays `not yet taken`;
  the instrument exists to be run in a fresh disposable VM sitting.

- WP-020 increment 2f's session takes the in-process verification reads its
  amended boundary names for the SI-35 instrument's gates 3, 6, and 7: the
  attached device's complete logical contents are hashed through the held
  loop descriptor before the first and after the last external launch and
  must equal the compiled catalogue digest; the named sysfs projection facts
  (disk size, read-only flag, logical block size; partition index, start,
  size, read-only flag) are read from the retained-rdev root and released
  with the report; the mount table must contain no session device number and
  every session node must report read-only, both checked during the window;
  and the predeclared udev query now runs twice per node for the
  byte-stability gate. Each check refuses with a typed value
  (`LoopDeviceHashMismatch`, `SessionNodeMounted`, `SessionNodeWritable`)
  and still reaches the unconditional detach. These reads launch nothing and
  accept no caller input; the public surface grows by integer-only facts
  types and one pinned slice getter. Tier-1 evidence: 62 Linux / 39 Windows
  crate tests, clippy clean on both platforms; no real loop device ran.

- The SI-35 hardened non-WSL protocol gains its **mechanism amendment,
  recorded before any output exists**, as WP-020's increment 2f boundary
  requires: the preregistered setup and privileged comparison actors merge
  into the one crate-owned session (`run_probed_session`), and the live
  client-projection capture moves into that session's predeclared `udevadm`
  launches, because the 2f boundary forbids disclosing the live device
  identity to any caller while the device is bound — a lent descriptor or
  name cannot be confined, per the recorded `try_clone_to_owned`
  measurement. The udev database and named sysfs attributes are
  world-readable state whose content does not depend on reader privilege,
  so the capture's privilege changes who reads, not what exists to be read;
  the unprivileged measurement shell keeps its negative environment
  assertions and all post-release analysis over the quarantine-released
  records. One preregistered mitigation is substituted and the substitute
  is recorded as weaker: gate 2's authority-drop is replaced by the
  crate-enforced bracket around every launch, which detects a rebind that
  happened rather than preventing one. No measurement is taken, no gate,
  normalizer, trial, or outcome rule changes, and every result cell stays
  `not yet taken`.
- WP-020 increment 2f is implemented: `crates/ffi-linux-loop` gains
  `run_probed_session`, the hold-open loop session its merged authorization
  boundary permits. The session consumes a SAFE-007 `Authorization` selecting
  exactly one registered fixture, configures the same read-only
  autoclear/partscan mapping 2e uses from the held verified descriptor, and
  then — the genuinely new capability — launches the predeclared probers
  itself: `udevadm settle`, then `udevadm info --query=all`,
  `blkid -p -o udev`, and `wipefs -n` against the disk and each enumerated
  partition, every launch under compiled absolute paths, structured argv, no
  shell, no `PATH` search, a cleared environment with one fixed locale pin,
  bounded capture that refuses overflow rather than truncating evidence, and
  a kill deadline. Node identity is re-statted by `lstat` (a planted symlink
  is seen, not followed) and the full `LOOP_GET_STATUS64` binding re-verified
  immediately before and after every launch, as protocol control flow a
  caller cannot skip. Captured output is quarantined in the session gate and
  released only after `ENXIO`-confirmed detach and partition teardown; every
  refusal path drops the bytes unpublished, and no public signature returns a
  descriptor, `File`, name, path, or device number — the design that lent the
  caller a descriptor was rejected in the authorization on the
  `try_clone_to_owned` escape measurement, and the structural tests now pin
  the five-function public surface. Partition enumeration is exact-or-refuse:
  a child under the session's own retained-rdev sysfs root whose name,
  `partition`, or `dev` attribute disagrees refuses the session rather than
  being skipped or guessed at. Tier-1 evidence lands on both platforms (58
  Linux / 37 Windows crate tests, two compile-fail doctests); no real loop
  device ran, Tier 1 still opens no block device, and the session's first
  real-kernel exercise is intentionally the WP-035 SI-35 instrument. The 2e
  entry point, its protocol, and its proven behavior are unchanged; shared
  configure/verify/detach logic was extracted verbatim rather than
  duplicated or altered. Increment 2f remains **weaker than 2e by
  construction** — across the open window the bracketing detects a rebind
  that happened rather than preventing one — and nothing here registers a
  destructive suite.

- WP-035's status surfaces are corrected for the closure of repository issue
  #94, and the correction is deliberately narrow: **no measurement changes and
  no register disposition moves.** The loop-backed half of SI-35 was blocked
  until #94 closed; it closed on 2026-08-03 when WP-020 increment 2e's
  descriptor-bound mechanism landed and passed a real acceptance, so the
  preregistered hardened non-WSL protocol is now **runnable rather than
  blocked**. What lifted is stated exactly rather than generally: the gate was
  the absence of a descriptor-bound attach, so a loop-backed measurement that
  configures the device *from a verified descriptor* no longer carries the gap,
  while one reaching the device by pathname — anything built on plain
  `losetup <file>` — still does, and the recording rule still applies to it.
  Nothing already measured improves. The 2026-08-02 WSL2 run stays
  non-qualifying, its decisive-pair negative stays unavailable to a register
  decision, and **M0.5's loop criterion stays unsatisfied** — takeable is not
  met, and a closed gate is not a result. Historical statements that a
  measurement was taken while #94 was open are left exactly as written,
  including the run's `Binding status | open at run time` row and the quoted
  increment-5 rule, because they were true when recorded. Touched surfaces are
  WP-035's own: the README M0.5 section and WP-035 row, the observability
  status header and its two loop-gate paragraphs, and WP-035's increment-5
  scope. No new authorization was needed — WP-035's existing share already
  names its own status rows and the M0.5 roadmap section.

- WP-020 increment 2e is **Delivered** and repository issue #94 is **closed**:
  it introduces the sole runnable
  higher-tier acceptance,
  `cargo xtask test --tier 2 --profile destructive --acceptance
  linux-loop-read-only`, while every generic destructive Tier-2 request and
  every Tier-3 request continues to refuse. The Linux-only acceptance runs with
  explicit privilege in a disposable non-WSL VM and applies SAFE-001, SAFE-002,
  and all SAFE-007 factors even though it is non-destructive and
  logical-content-read-only. Its verified backing descriptor remains held;
  backing, loop-control, and loop-device descriptors are `O_RDWR`-capable for
  mapping control; the mapping carries `LO_FLAGS_READ_ONLY`; the probe is
  in-process through the held loop descriptor; and no external storage tool or
  logical write, discard, or zero operation occurs. Linux's configure and
  rebind paths may `fsync` and write back already-dirty data or metadata, so the
  acceptance deliberately makes no zero-physical-write claim. Both authorized
  fixture objects are hashed before any attach; each initial digest must match
  the compiled fixture catalogue before any loop configuration, and
  both are hashed again after the ordinary and adversarial legs confirm detach
  and partition teardown. No observation is accepted until the sampled backing,
  loop configuration, and held-node identities match and those hashes are
  unchanged. External run evidence must exclude every other actor able to
  modify either fixture and every other actor able to administer or rebind loop
  devices. Ordinary kernel/udev read/open discovery is allowed and handled by
  bounded cleanup, but a loop-configuration `EBUSY` refuses immediately because
  isolated loop state was not established. Hash and status sampling cannot
  defeat an ABA change between samples; VM isolation bounds consequences but
  does not prove the exclusions, and this acceptance is not a continuous-binding
  guarantee. This exception grants SAFE-007
  coverage only to this named acceptance: it does not change Tier 1 or the
  product inspector's read-path boundary and it does not register a destructive
  suite.

  The acceptance **passed** on 2026-08-03 in a disposable
  Proxmox VE 9.2.4 guest — stock Ubuntu 22.04.5, kernel 5.15.0-186-generic,
  base image verified against Canonical's published `SHA256SUMS`, no USB or PCI
  passthrough, a `pre-acceptance` snapshot as the revert boundary. Two legs were
  configured and detached, the adversarial `LOOP_CHANGE_FD` rebind was detected
  and its observation discarded, partition teardown was confirmed, both
  fixtures' initial digests matched the compiled catalogue, and both were
  unchanged afterwards with `losetup -a` empty and no loop device holding a
  backing file. Four negative controls refused in the same session, including a
  generic destructive Tier 2 that authorized 13 targets and still refused
  because no suite is registered. It was run four times in that guest with
  identical harness results and identical fixture digests: three on the
  implementation commit `2dbf601`, and once more on the merged commit
  `c75b340` after main's `apps/cli` changes arrived. That last run was taken
  rather than argued: none of the changed files is on the acceptance's code
  path, which is exactly the reasoning a proof against a superseded tree
  invites and this package declines.
  The exclusions were established rather than asserted: `snapd` — which held
  four squashfs loop devices at first boot — and `udisks2` were purged, **a
  deliberate deviation from a stock image** without which the no-other-loop-
  administrator condition cannot hold; `/root` and the fixture directory are
  `drwx------` root-owned; and every non-root process's `CapEff` was read and
  none holds `CAP_SYS_ADMIN`. Two limits are recorded rather than smoothed
  over: **the guest was not network-isolated** — it held a DHCP address and a
  default route throughout, which the transcript records as a fact — and the
  digest and status checks remain discrete samples that cannot defeat an ABA
  change. Closing #94 registers no destructive suite; increment 2's own scope
  is unblocked and still unbuilt. The full record, including what the run does
  not establish, is in `docs/work-packages/WP-020.md`.

  Recorded because it will recur: this acceptance must run as root over a
  direct login with no `sudo` in the chain and no injected environment
  variables. WP-035's redaction sweep compares every environment value of six
  characters or more against CLI output, so `SUDO_USER=partman` — or any name
  colliding with product text — fails a Tier-1 gate before the acceptance is
  reached. That is the tripwire working correctly; the fix belongs in the
  environment, never in an exemption to the commit under proof.
- The SI-33/SI-28 successor protocol's two S4 rows move from `not yet
  taken` to `not established` on a custody-complete 2026-08-03 sitting:
  with both readers attached simultaneously, the delivered second unit
  measured as a different bridge model than the parent-record reader
  (VID:PID 2537:1081, NORELSYS 1081CSx, against 0BDA:0306), and S4's own
  rule forbids approximating the comparison with a different model. No
  collision hypothesis is evaluated and no register disposition changes;
  the arm remains takeable as preregistered once a same-model second unit
  exists. The sitting's capture script and complete transcript are
  archived outside the repository with digests recorded before the first
  device query and an independent second-reader rehash; the observability
  record carries the custody fields. Recorded as context only: the second
  unit reports one ascending-hexadecimal placeholder serial at both the
  USB-descriptor and storage layers, identical across its LUNs — the
  SI-28 bridge-constant form on a second bridge family.
- The SI-35 Windows completion rerun is preregistered in the observability
  record, all cells `not yet taken`: three hardened gates — total retention
  (query-and-discard, the recorded defect that made W-H1/H2/H3 unevaluable,
  now voids the sitting), the restored before/after digest bracket
  (wrapper prose is not a digest), and a mandatory layout-IOCTL
  index-fallback probe for `MSFT_Disk`-invisible fixtures — plus the W-Q4
  hybrid answer against its MBR control. The parent protocol's hypotheses,
  mechanics, and scope limits are inherited unchanged; W-H2's
  which-GPT-copy question is recorded as needing a WP-020 discriminating
  fixture and is not preregistered across that package boundary. No
  measurement is taken and no disposition changes.
- The SI-33/SI-28 successor protocol is preregistered in the observability
  record, all cells `not yet taken`: the close-before-event/reopen survival
  arm with a true process-local no-handle window and epoch-boundary
  bracketing, epoch-signal characterization over a fixed candidate list with
  the counter itself excluded as circular, completion of L4's originally
  requested three trials, and the second-reader storage-layer serial
  collision test — previously pre-registered only in conversation, which
  confers no standing — with its hypothesis, refutation condition,
  live-comparison requirement, and enumeration-failure-is-data rule now in
  the repository. README's M0.5 section and WP-035 row note the
  preregistered instruments. No measurement is taken and no disposition
  changes.
- ADR-0010 is accepted: Section 4.1's required UI layer is Svelte and
  TypeScript, SvelteKit excluded, Vite as the build tool — spec **5.0.0**, a
  major bump because §0.1 makes a semantic change to an existing requirement
  major regardless of implementation state. No code changes: `main` carries
  no UI and nothing React-specific, re-verified at acceptance. No desktop
  shell is approved — Tauri 2 stays the named shell with no new authority,
  PR #91's retirement stands, and the ten `G-AX-*` accessibility gates
  remain inconclusive. The ADR's verification obligations are deferred to
  shell authorization; until then the stack is intended, not validated.
  Current-version pointers (README, CONTRIBUTING, PR template, test tiers)
  move to 5.0.0. Substance is in the spec's own §0.3 changelog, which
  controls specification changes.
- WP-035 increment 6's measurement matrices are preregistered in the
  observability record with every cell `not yet taken`: a macOS matrix (raw
  read policy, IOMedia/diskutil projections, APFS membership and container
  UUID, foreign-signature fixtures through read-only no-mount attach, the
  SI-34 stale-signature freshness row, and a hardware-conditional Fusion
  cell) and a real-partitioned-Linux matrix (six declared layouts on
  disposable passthrough fixture media, baseline/`disk`-group/helper
  projections kept separately labelled, Part 6 precondition 2's native
  designator, stability, and collision rows, and the SI-34 freshness row on
  a real device tree). Both define closed result vocabularies, validity
  gates whose failure voids rather than refutes, and transcript custody
  with second-reader readback. The Linux provisioning-versus-Method wording
  conflict is filed inside the protocol rather than left to surface later.
  No measurement is taken, no register disposition changes, and no cell
  asserts platform behavior.
- WP-035's portability and fidelity pass gives the Linux replay open flags
  their per-ABI-family values: generic, MIPS, and SPARC targets each pin the
  reviewed `O_NONBLOCK | O_NOCTTY` encoding (`0x900`, `0x880`, `0xc000`), an
  unreviewed Linux ABI now refuses to compile rather than inheriting generic
  values, and Tier-1 asserts every family constant plus the selected target's
  wiring into the one replay open call. The dependency doctor's output bound
  is stated as it is enforced — 4096 bytes per child stream, at most 8192
  aggregate retained — in the constant names, the over-limit diagnostic, and
  a new Tier-1 boundary test; the enforced limits themselves are unchanged.
  The CLI manifest comment names the Tier-1 non-`Hash` ambiguity assertion
  rather than the removed compile-fail doctest. No limit value, requirement,
  schema version, or exit code changes.
- WP-010's authorized metadata and historical-fidelity repair marks SI-11 as
  hash-visible and ties its retained plan/snapshot obligations to MODEL-005 and
  Section 6; narrows SI-12 and spec 4.3.0 to the evidence actually retained for
  unequal multipath identifiers; and limits SI-34's stale-signature result to
  the named finite projection it measured. No option, issue state, class,
  dependency, normative requirement, specification version, or evidence status
  changes.
- WP-010's post-acceptance integrity pass reconciles four accepted ADRs with
  the authoritative register without changing a normative decision. ADR-0002
  and ADR-0005 now label their old blocker lists as acceptance-time snapshots;
  ADR-0011 distinguishes its absolute multipath policy from the cases its
  accepted detection rules currently cover and records its completed WP-035
  follow-up; ADR-0012 states the client-visible boundary of type-level
  protection and the helper's operative role for invisible facts that actually
  tighten or refuse. The register now classifies withdrawn
  SI-36 instead of leaving it outside the supposedly exhaustive class table,
  records all three SI-35 acceptance-evidence categories as unsatisfied, and
  removes claims of backup recovery and universal interface absence that the
  retained measurements cannot prove. The unequal-identifier, unassembled
  multipath residual is filed as hash-visible SI-37, an input resolved through
  SI-11 rather than an independent direct blocker; SI-11 and SI-27 retain their
  separate closure and naming responsibilities. No open option is selected, no
  evidence is promoted, and no requirement text is changed by this correction.
- The WP-035 audit closes the chassis honesty and evidence gaps it found.
  `inventory`, `topology`,
  and `capabilities` are now recognized reserved commands that return exact
  schema-versioned typed refusals at exit 3, naming the spec issues or CAP-005
  requirement that prevents each domain payload; they are no longer parser
  usage errors. The dependency doctor now preserves a real tool's nonzero exit
  as `nonzero-exit` failure rather than allowing its output to become version
  evidence, with Git supplying both deterministic success and failure launcher
  proofs at reviewed absolute paths. Human output uses an injective visible
  encoding for caller controls and backslashes, JSON escapes every Unicode
  control, and tests pin record boundaries against C0, DEL, and C1 injection.
  Replay opens now use the reviewed target ABI values — Linux
  `O_NONBLOCK|O_NOCTTY=0x900`, Darwin `0x20004` — with a source-use guard that
  keeps the constant wired into the actual open. The advertised non-`Hash`
  boundary is now a regular Tier-1 compile-time assertion rather than a doctest
  the tier did not execute, and generated traceability now carries direct
  INV-006 evidence without claiming WP-050's CAP-005 capability engine.
- The SI-33/SI-35 evidence record is corrected to the retained evidence's
  actual reach. SI-33's close-before-event/reopen arm remains unmeasured; its H
  matrix and L6a have no retained transcript, L4 reached only one of three
  trials, and the lower later reading does not characterize a counter epoch or
  prove an actual fail-open comparison. Windows SI-35 discarded queried
  PhysicalDisk property values, so equal retained fields do not satisfy the
  pre-registered existential refutation conditions. The historical WSL2 loop
  output is non-qualifying under issue #94 and a post-hoc normalizer. A new
  all-`not yet taken` protocol specifies descriptor-bound, replicated,
  order-balanced confirmation in a disposable non-WSL VM; it records no new
  measurement.
- The inspect chassis's standing gated list re-attributes its
  `same-device-claims` entry from SI-12 to ADR-0011, completing the follow-up
  the ADR's Consequences named: SI-12 resolved in spec 4.3.0, and a live
  surface citing a resolved issue as an open gate is the count-drift the
  register's sole-authority rule exists to prevent. The prohibition itself is
  unchanged — the inspector still never claims two paths are or are not the
  same device — but its state renders as `never-inferred` under the deciding
  ADR rather than `not-established` under a question that no longer exists,
  in both the JSON and human renderings, with the pinned tests moved to the
  new strings and the JSON state assertion now per-surface. README's two
  restatements of the gated list and WP-035's two boundary citations move
  with it; the `partman.cli.envelope/0` schema is provisional within major
  version 0, and this is the kind of deliberate change that provisionality
  exists for.

### Added

- The SI-35 **loop-device measurement** was taken read-only on 2026-08-02,
  six fixtures plus the 4Kn annex attached `--read-only --partscan` one at a
  time, read-only read back from `/sys` rather than trusted from the flag, and
  the measuring identity asserted unprivileged — uid non-zero, not in `disk`,
  and a direct read of the loop device **denied**, where the denial is the
  pass. **Repository issue #94 was open, so this measurement was taken across
  a block that had not lifted.** WP-035 says loop-backed work is *blocked*
  until #94 closes and adds a rule for what to record if a read-only
  measurement is taken anyway — a contingency, not a permission — and #94
  itself disclaims proposing "a manual, out-of-tier loop attach". The run was
  performed at the operator's explicit instruction after the gate was raised,
  though it was raised with an over-favourable reading since withdrawn.
  **M0.5's loop-backed exit criterion is not satisfied by this run.** The
  contingency was honoured: the binding-gap line travels beneath every table
  filled, WP-020's increment-2 row stays Blocked, and test-tiers.md's
  sentences remain true because nothing in the repository opened a device,
  though the environment was not §11.3's either — T2 is defined as disposable
  VMs and this ran on the operator's working WSL2 instance. The two
  authorities characterize the activity differently — WP-035 calling these
  operator-run experiments and not tier work, #94 calling the same probe
  Tier-2 work that cannot yet be made — and §11.3, cited by neither, places
  `loop` under T2 while restricting only *destructive suites* to T2 and T3.
  That is a documentation inconsistency, not a conflict between requirements,
  so it is **not** a §1.11 filing; an earlier version of this entry said it
  was, and filing it would have inflated the register's counts with a
  miscategorization. It is recorded on issue #94 instead. A post-attach
  run-time content check retained `matched` for all seven fixtures, but it was
  not part of the pre-registered phases and occurred after attach, partition
  scanning, and udev settlement; it cannot bind pathname resolution, those
  earlier events, or later rebinding. On this WSL2 run, the post-hoc-normalized
  client projections of conflict and healthy were identical, and the retained
  `blkid` properties and `wipefs` offsets did not separate them. This is
  historical, non-qualifying evidence while #94 remains open. The negative is
  also withheld pending a descriptor-bound non-WSL rerun. Every named retained
  reading taken so far — Windows CIM/layout IOCTL and Linux file/loop
  projections — failed to separate the decisive pair, but the Linux readings
  share libblkid 2.41 and are not independent. This gives option (b) no
  support; it does not decide viability. None of the register's three evidence
  categories is decision-complete: the Windows procedure ran but retained an
  incomplete surface, the loop run is non-qualifying, and the chosen option's
  refusal demonstration cannot exist before an option is implemented.
  Separately, this record requires the non-WSL confirmation, and M0.5 requires
  #94's closure. The historical WSL2 run recorded H-4Kn
  supported: a loop configured with a 4096-byte logical sector size exposed
  the GPT where file probing reported `PMBR`, consistent with the file result
  being a probing artifact. Because #94 was open, it does not qualify as the
  confirmation. The damaged-primary client projection differed from healthy
  by materializing no partitions while its udev entry still said `gpt`; that
  is a content difference, not an explicit damaged-state marker. Retained
  `wipefs` offsets exposed the missing backup while the client projection did
  not. The client projection carried no hybrid trace; helper-side hybrid
  classification was unmeasured because only offsets, not signature types,
  were retained. The initial diffs exposed backing filename,
  encoded-filename, inode, and device plumbing. The recomputation dropped all
  four `ID_LOOP_BACKING_*` keys post hoc; the retained summary does not show
  that every key differed in every pair. The first computation is void, and
  post-hoc normalizer extension is not a qualifying method; the future
  protocol freezes normalization and voids on undeclared plumbing. Equality of
  this finite projection is not a refutation of the existential claim that
  some other client-readable fact may separate the pair.

- The SI-35 **Windows partition-list measurement** was taken on 2026-08-02,
  seven WP-020 fixtures attached one at a time as read-only fixed VHDs by an
  elevated console while an ordinary non-elevated console did the measuring,
  with the executed wrapper reporting every post-detach digest `UNCHANGED`.
  The underlying pairs were not retained, so that verdict is not independently
  auditable and does not prove the bytes never changed transiently. No retained
  field from `MSFT_Disk`/`MSFT_Partition`, `MSFT_PhysicalDisk` row presence, or
  the layout IOCTL separated conflicting, damaged-primary, or missing-backup
  from healthy GPT. The wrapper retained `IsReadOnly=True` and PhysicalDisk
  `rows=1` for all five CIM-visible fixtures, but discarded queried
  PhysicalDisk property values; the declared W-H1/W-H2/W-H3 refutation
  conditions are therefore unevaluable and their broader existential
  hypotheses remain inconclusive. The damaged-primary rows cannot distinguish
  backup recovery from parsing the primary without CRC validation. The run
  gives option (b) no support on the retained Windows projection; it does not
  establish that Windows has no separating client-readable surface.
  `blank-512` remains distinct. Two fixtures — precisely those whose MBR has a
  non-protective entry — produced no `MSFT_Disk` row while `Win32_DiskDrive`
  and `Get-DiskImage` enumerated the attachment. That absence reproduced for
  each fixture in two runs about three minutes apart, but the host device set
  changed between runs. Both 800 ms and 3 s waits produced no row; longer
  settling remains untested. Because the executed wrapper returned on zero
  `MSFT_Disk` candidates, it never queried `MSFT_PhysicalDisk` or
  `MSFT_Partition` for those fixtures, and the reachable layout IOCTL was not
  run. The MBR-entry relationship is a correlation, not a mechanism, and the
  hybrid question is **not attempted**, not unanswerable.

- The SI-33 media-change-counter liveness experiment was **taken** on
  2026-08-02, on a card reader and two identical flash drives, entirely
  unprivileged and read-only, and its results are recorded in
  `docs/quality/observability.md`. L1 immediate exchanges and L2
  sixty-second-idle exchanges moved in 3/3 trials, but the full register
  sequence did not pass: L5b kept the original handle open across the event
  before opening a fresh handle, and retained L5a was floor-to-floor and
  uninterpretable. The close-before-event/reopen arm is unmeasured. L4 moved in
  its sole retained trial (1/1 taken of 3 requested), so its replication
  requirement is unmet. Prompt movement still cannot be attributed to
  exchange-synchronous detection. A later reading was lower across an interval
  containing a timestamped PnP arrival, so global monotonicity cannot be
  assumed; LastArrival is an event marker, not a driver-incarnation token, and
  reset causation was not established. The fail-open example remains explicitly
  constructed: the run establishes a decrease, not a repeated plan/apply floor
  equality or an actual witness failure. No incarnation signal was
  characterized: the run-2 same-instance Boolean was derived from the counter,
  PNPDeviceID persisted across L7 replug, and LastArrival identifies an arrival
  only. The H matrix and L6a survive only as operator reports without a retained
  transcript or group-membership record; durable evidence is limited to the
  narrower retained legs. L3's V1 step followed filesystem I/O in the same
  round instead of replacing it, so that arm also needs the specified rerun.
  An earlier L4 attempt whose physical action never happened was discarded and
  occupies no cell. The later SI-35 run records and their current limits are
  recorded in the newer entries above.
- At the time WP-035 increment 5 first landed — before the later operator runs
  recorded above — it recorded the register measurement instruments:
  protocols and recording formats for the SI-33 media-change-counter
  liveness experiment and the SI-35 loop-device and Windows partition-list
  measurements, spliced into `docs/quality/observability.md` beside the
  established rows they extend. The experiments are operator-run and
  read-only — not tests, not repository commands — and no measurement had yet
  been taken: every cell a run could fill initially read `not yet taken`. Each
  section extends the file's rule of use so such a cell cannot be relied on,
  cited, or paraphrased as a finding, and each hypothesis carried a
  preregistered outcome rule — with the degraded-state questions named as
  questions, not hypotheses — so no conclusion could be fitted to data
  afterward. The later review withdrew H-separation's finite-projection
  equality rule because it could not refute an existential hypothesis. The
  SI-33 protocol probes both CHECK_VERIFY variants across an
  access matrix from zero-access up, quarantines the device-reaching variant
  from the liveness legs, and records counter movement only as deltas and signs
  because an absolute count is session history of real hardware (SEC-006);
  the register's staleness worry — an answer the class driver already
  holds — is named as the hypothesis under test, not asserted in either
  direction. The SI-35 Windows protocol reaches the storage stack through
  a byte-deterministic fixed-VHD conversion (payload untouched, every free
  footer field pinned, with an intended digest bracket on both sides of a
  read-only attach), splits the privileged setup from the unprivileged
  measurement
  with both elevation states asserted, and makes the content-versus-state
  distinction load-bearing: the conflicting fixture's primary table is
  row-identical to the healthy one's, so no partition-list shape can be
  recorded as state separation. The loop protocol is blocked on issue #94
  and carries the gap structurally — the binding-status line must travel
  beneath every table a run fills. Every embedded script that can run
  without an attach — the VHD conversion, the CIM measurement query, the
  layout probe, and the SI-33 block — was executed on 2026-08-02 as
  validation only (a signed-hex-literal defect that killed the SI-33 block
  on paste was found this way, by running, not reading); the privileged
  attach and detach blocks and the loop-protocol shell phases were not
  executed, the latter because running the attach is itself the #94-gated
  act, and nothing from any validation run fills any cell. The README's
  M0.5 heading now reports the milestone in progress rather than not
  started, its WP-035 status row counts increments 1–4 delivered and
  increment 5's instruments recorded while stating in the same sentence
  that the package's objective — the measurements themselves — remains
  open, and WP-035's README share names the M0.5 roadmap section
  explicitly, per the WP-030 precedent that a milestone surface is
  distinct from a status row.
- WP-035 increment 4 delivers the observation records, and `partman
  inspect` answers for the first time. Every observation carries its
  attribution — adapter name, version, method — the precursor toward
  MODEL-004, whose hashed envelope remains WP-010's to deliver. The outcome
  vocabulary is ADR-C4's as the ADR wrote it, after adversarial review
  refused a paraphrase that folded read errors into unavailability:
  `observed` carries bytes or a positively determined absence — absence is
  a value, known from the opened handle's own length, and the state word
  says so; `unavailable` is the platform not exposing an answer; `failed`
  is the read itself erroring, kept distinct because collapsing the two is
  the conflation the ADR condemns. A probe straddling the object's end
  never claims absence of bytes that exist: the existing prefix is
  observed under an accurate subject and only the truly missing tail is an
  absence. `--replay` runs the fixture-replay adapter over one
  caller-named regular file: a pre-open look refuses devices and
  directories before any open in the common case, the authority is fstat
  through the opened handle, and on Unix the open is non-blocking so a
  FIFO is refused rather than hanging — a device named by path is refused
  unread, opened read-only under a rebinding race at most long enough for
  the handle to identify itself; no command reads a block device, and
  nothing opens one with write intent. A following flag is never swallowed
  as `--replay`'s value. Bytes are reported as lowercase hex at compiled
  probe offsets and classified by nobody: a mechanical test bans a named
  list of interpretation words from the observation renderings — the
  list's reach is exactly those words, with the residue held by review —
  and the standing gated list — identity-strength (SI-28),
  partition-table-state (SI-35), same-device-claims (SI-12) — travels
  in-band in every inspect answer, refusals included, because what the
  inspector will not say is stated, never inferred from silence. Without
  `--replay`, the answer is a typed no-adapter statement naming the
  platform adapter package rather than a plausible empty machine. The
  output echoes no path and no file name — a session-local selector
  stands in, and the tests pin that in both renderings. Replay over one
  of WP-020's deterministic images is pinned byte-reproducible through
  the fixtures crate as a dev-dependency, which the empty-shipped-closure
  guard keeps out of the binary.

- WP-035 increment 3 delivers the dependency doctor and the technology
  facts. `partman doctor` reports each roster tool as facts with provenance —
  present or absent with every candidate path checked, the sanitized version
  banner, the parsed version, and whether it falls inside the one recorded
  tested family (util-linux 2.41, the version the fixture prober measured) —
  under SAFE-004-derived controls: compiled absolute candidate paths as the
  executable allow-list, no PATH search, structured argument arrays, a cleared
  child environment, bounded output, and a time limit. Executable identity is
  not verified beyond the absolute path. Mapping an out-of-range version to a
  `blocked` capability is SAFE-004's own last clause and stays with WP-050's
  capability engine; a test refuses the
  CAP-003 status vocabulary in every doctor and facts rendering. The Windows
  and macOS rosters are deliberately empty and render as a typed
  not-implemented statement naming the adapter package that will populate
  them — never as "all dependencies satisfied". `partman facts` ships
  FS-007's inputs: five immutable technology limits (led by the
  specification's own example, XFS not shrinking), each carrying the basis a
  reader can check, pinned as a literal contract. The doctor's I/O goes
  through an injected launcher seam, so WP-035's Tier-1 tests launch no roster tool.
  WP-035's direct test subprocess classes are Git and the compile-time-selected
  Cargo; Git supplies the real launcher's success and nonzero-exit subjects.
  The shipped
  binary's I/O statement grew with the increment and says so: existence
  checks and version probes of roster tools, and nothing else. A banner that
  does not parse stays unrecognized with its raw line preserved, never
  guessed; the launcher's time-limit path is exercised by review and manual
  probe rather than by a Tier-1 test, a stated trade rather than a hidden
  one.

- WP-035 increment 2 delivers the deny-by-default redaction allowlist and the
  redacted `export-diagnostics` command. The allowlist is the bundle
  builder's type, not a filter: a closed field enum is the only route into
  the bundle, every variant renders compile-time data, and no API accepts a
  caller-supplied key or value — so SEC-006's field list (device serials,
  paths, labels, usernames, keys, file names) is a deny-floor the increment
  cannot fall below because no runtime value exists to leak. The bundle
  carries the build's identity, target, command surface states, exit-code
  contract, and the missing discovery evidence as an in-band typed refusal
  naming WP-035 increment 4 — never an omission a reader could mistake for a
  clean bill. Redaction tests gate the tier from four sides: the bundle's
  JSON key set is pinned as literals so widening the allowlist is a visible
  reviewed edit; the human rendering is pinned byte-for-byte so a human-only
  disclosure fails like a smuggled key; no output in any mode may carry the
  host's username, home path, computer name, or any environment value six
  bytes or longer that is not byte-equal to a compile-time constant the
  bundle renders by definition (WSL's HOSTTYPE=x86_64 equals the build
  target and forced that exemption to be named); and a source guard refuses
  any `env::var`, `env::vars`, or `var_os` spelling in the shipped sources,
  which is what makes environment-independence a tested property rather
  than a review promise — the tripwire only sees variables the test host
  sets. The bundle's command-surface states are additionally required to
  agree with dispatch behavior, so the diagnostics cannot claim an
  unimplemented surface answers. The command emits to stdout only; the
  shipped binary still opens no socket, reads no file, and reads no
  environment variable — the source guard and the empty dependency closure
  are what hold that sentence, with the glob-import bypass refused by the
  workspace's pedantic clippy gate.
- WP-035 increment 1 delivers the `partman` CLI chassis: structured argument
  parsing that owns the non-Unicode seam as a typed usage refusal instead of
  `std::env::args()`'s panic; a documented exit-code contract (0 answered, 2
  usage refusal, 3 typed refusal) whose literal values are pinned by test; an
  ANSI-free, schema-versioned JSON envelope (`partman.cli.envelope/0`,
  provisional within major version 0) wrapping every JSON emission including
  usage refusals; and the typed refusal vocabulary — `partman inspect`
  refuses with state, reference, and detail on stdout rather than printing a
  plausible empty inspection. Domain payloads are absent from every surface,
  not emitted unversioned. Two structural guards land with their reach
  stated exactly: the shipped dependency closure is empty (asserted through
  `cargo metadata`; dev-dependencies cannot reach the binary) and the output
  type carries a Tier-1 compile-time ambiguity proof that it does not implement
  `Hash`; `std`'s
  own hashers used deliberately in-crate remain a named review obligation.
  The increment was adversarially reviewed before push; the review found and
  forced closure of a self-referential exit-code test, the argv panic seam,
  and guard prose that claimed more than the mechanics establish. WP-035's
  traceability converts to generated in the same change. The register
  measurements, observation records, redaction allowlist, and dependency
  doctor are later increments and are not claimed.

- WP-030 publishes the evidence-only result of ADR-0009's bounded Slint 1.17.1
  evaluation without merging the rejected desktop runtime. A strict normalized
  manifest records immutable source/lock/artifact identities, structured
  commands, exact renderer-graph observations, and supply-chain findings but
  cannot carry a `pass` or `result` field. The generator parses the ADR's exact
  41-gate registry, rejects duplicate JSON keys and missing/duplicate/unknown
  gates, hashes integer-only input with PartMan's shared `pce/1` implementation,
  and emits byte-stable Markdown. It derives one pass, hard failures at
  `G-CFG-08` and `G-SC-01`, and 38 inconclusive rows. Windows unstripped
  executable measurements remain explicitly non-decisive; no Slint runtime,
  renderer dependency, licence exception, or supply-chain waiver lands.
- WP-000 closes issue #39 after all four current package traceability documents
  completed package-owned, zero-loss conversions. Every document is now
  generated from validated source annotations and typed evidence; declarations
  are checked in both directions, hand edits fail CI, and each predecessor is
  frozen by an exact revision/blob migration ledger. WP-000 remains in progress
  for its other documented gaps rather than being rounded up by this one
  completion.
- WP-020 increment 1g converts its traceability document to generated evidence.
  Source-local annotations cover deterministic fixture construction,
  purpose-binding mutations, external-prober expectations, and the
  disposable-target interlock, while typed package evidence covers the shared
  runner, Linux-only prober command, workspace lint boundary, and zero-loss
  migration. A frozen source-revision/blob ledger accounts for every former
  row, correction, limitation, and residual risk. The conversion also removes
  a stale operator-provenance claim: the public, build-derived token proves only
  exact value presentation and supplies accident friction; it does not prove
  that an operator ran the generator or intended an operation. Tier 2 and Tier
  3 remain unavailable.
- WP-000 scheduled maintenance: every Monday and on manual request, dependency
  policy is re-evaluated on Windows, Linux, and macOS even when the repository
  has not changed, while both current fuzz targets receive 15 minutes each and
  carry a GitHub-hosted corpus forward from earlier successful runs. The
  pull-request workflow and all eleven branch-protection check names remain
  intact; a structural test holds the schedule, corpus key, duration, OS
  matrix, and existing gate roster to those claims. Its first full run exposed
  libFuzzer's 2 GiB aggregate RSS default as too low for 15 minutes of the
  allocation-heavy structured-value target despite only about 26 MiB of live
  heap. Fuzzing now pins a 4 GiB process ceiling while retaining a separate
  256 MiB single-allocation ceiling, a 4,096-byte input limit, and the existing
  25-second per-input timeout.
- WP-010 increment 2a resolves SI-31 with schema-level canonical set semantics
  in spec 4.1.0 and ADR-C6. Rust and TypeScript now sort set elements by an
  unsigned lexicographic comparison of each element's full canonical bytes,
  reject duplicates without deduplicating, validate strict order at the schema
  boundary, and inherit the enclosing artifact's depth budget when producing
  element sort keys. One shared, deliberately unsorted fixture pins exact
  bytes, hashes, comparator disagreement, and the accepted/rejected depth
  boundary. Semantic arrays and the `pce/1` profile remain unchanged. Both
  existing fuzz targets exercise the new producer/validator paths, and WP-010's
  traceability is now generated with a zero-loss migration ledger. The
  TypeScript authorization boundary also snapshots generic arrays, schema sets,
  and raw hash bytes without consulting caller-controlled constructor species
  or mutable prototype methods; regression tests cover element substitution,
  dropping, reordering, and wider typed-array views.
- WP-030 increment 1c converts its traceability document to generated evidence
  without treating row-count similarity as proof of preservation. Thirty-one
  live Rust tests now carry requirement-bound claims, non-test evidence is
  typed in the work-package source, and an exact source-revision/blob ledger
  accounts for every former evidence row and narrative section. The conversion
  also corrects vocabulary findings so entity, severity, and progress roles
  report UI-003, PLAN-004, and UI-011 respectively, with a hostile progress-role
  deletion proving the UI-011 path can fail.
- WP-010 increment 1: the `pce/1` canonical encoding. `schemas/canonical-encoding.md`
  specifies it normatively, and `crates/domain` implements the encoder, a strict
  validating decoder, and SHA-256 hashing over canonical bytes (MODEL-005).
  Golden vectors pin the encoding byte-for-byte, including the `2^53` and
  `2^64 - 1` boundaries that RFC 8785 could not have carried as JSON numbers.
  The decoder rejects rather than repairs: non-shortest arguments, floats, tags,
  indefinite lengths, non-text or misordered map keys, duplicate keys,
  ill-formed UTF-8, lengths beyond the input, nesting past a fixed depth limit,
  and trailing bytes.
- WP-010 increment 2: `packages/canonical`, the TypeScript half of MODEL-005.
  Both languages now read one shared fixture,
  `schemas/canonical-encoding-vectors.json`, so parity is proven against a
  single source rather than two per-language tables that could drift.
  `cargo xtask cross-language` runs the proof and gates CI as its own job. The
  package has no runtime dependencies: hashing uses Web Crypto and testing uses
  `node:test`.
- WP-020 increment 1: `crates/fixtures`, the deterministic disk-image generator,
  and the SAFE-007 disposable-target interlock. `cargo xtask fixtures` writes
  synthetic disk images — GPT, 4Kn GPT, MBR, blank, damaged-primary,
  conflicting-tables, hybrid MBR/GPT, APM, and on-disk signatures for LUKS2,
  LVM2 and mdraid — into the gitignored `tests/generated/`, each
  a pure function of the code that builds it, so two machines produce identical
  bytes and nothing binary is ever committed. The interlock requires all three of
  SAFE-007's proofs and computes disposability from a target's own bytes rather
  than accepting an assertion: a block device cannot pass, because its bytes will
  never equal a generated fixture. Tier 2 and Tier 3 still refuse, now for the
  honest reason that no destructive suite exists to run.
- WP-020 increment 1d: `crates/fixtures/src/evidence.rs`, which binds every
  fixture's bytes to the rationale recorded beside it. Until now nothing did:
  every layout and signature test rebuilt its own image from its own literals,
  so the catalogue was free to produce something else. Measured before it was
  fixed — with the LUKS2 builder emptied to a blank image and the
  multi-signature builder stripped of the stale mdraid superblock the Part 5
  asymmetry finding rests on, **all 64 tests passed**. Each catalogue entry now
  has a claim computed from its bytes; the set is exhaustive in both directions;
  each claim is paired with a mutation it must reject, so a check that cannot
  fail is caught; and `generate` refuses to write an image that no longer serves
  its purpose, naming what was lost. The oracles reimplement CRC-32, LVM2's CRC
  and mdraid's folded sum by different methods from the writers they check, each
  anchored outside the repository — to the published IEEE check value, and to
  three checksum fields `libblkid` 2.41 accepted, pinned with their provenance.

  The module was put through an adversarial pass before being proposed, and it
  found the first version repeating the defect it was written to end. The gate in
  `generate` was load-bearing on nothing — deleting it kept all 74 tests green,
  because every test fed it the real catalogue, which passes. Its
  "anchored outside this repository" claim held for one checksum of three:
  changing an initial constant in both writer and oracle kept everything green
  while making every fixture undetectable. And ten claims accepted mutations that
  destroyed a fixture's purpose while leaving its checksums valid — most sharply,
  "two tables that disagree" was proven by comparing entry-array CRCs, which one
  character of a partition *name* satisfies while both copies describe identical
  extents. All are closed, and the details are in
  `docs/work-packages/WP-020.md`.
- WP-020 increment 1e: `cargo xtask probe`, which re-runs `blkid` and `wipefs`
  over every generated fixture and compares against the expectations recorded in
  `crates/fixtures/src/prober.rs`. This closes the project review's open finding
  that real-prober acceptance was "manual, not regression-protected" — the last
  place in the package where an important property rested on someone having
  looked once. It needs Linux, so CI runs it as its own job; both tools are
  read-only and are handed regular files, never a device.

  A verbatim capture of `libblkid` 2.41's output is embedded in the tests and the
  recorded table is compared against it, so the table is checked on machines with
  no prober too — otherwise a transcription slip between the measurement and the
  table would look exactly like a passing test. The comparison is proved capable
  of failing in all four directions: a format no longer detected, a changed
  answer, a lost signature, and an added one.

  **Its first run falsified a claim increment 1 had recorded.** On util-linux
  2.39.3, which stock `ubuntu-24.04` ships, `blkid -p` reports nothing at all for
  `mdraid-1.2-member-512.img`, while `wipefs` still lists the superblock and both
  tools agree about every other fixture — including the 0.90 superblock in the
  stale-pair image. Increment 1 checked the signature writers by hand against
  2.41 on one machine and recorded the result as unconditional, so FS-004 Linux
  RAID and LIN-005 are **not** established on that platform. The expectation is
  now version-keyed rather than relaxed: below 2.41 the recorded answer is
  silence, so a prober that starts naming the fixture fails just as one that
  stops does. Which condition 2.39.3 rejects is unestablished — both versions'
  checksum routines are arithmetically identical, and the fixture satisfies the
  magic, `major_version` and `super_offset` checks — and it is recorded as
  unestablished rather than guessed at.
- WP-030 increment 1: `schemas/design-tokens.json` and `crates/tokens`, the
  design tokens and the accessibility harness that computes UI-001, UI-007 and
  UI-008 from them. The token file is the single source of truth and lives in
  `schemas/` for the reason `AGENTS.md` already records for the canonical
  vectors: when the front end arrives it must read *this* file, because an
  implementation checked against a table it also owns proves only
  self-consistency. `cargo xtask tokens` runs the audit and is part of
  `cargo xtask ci`.

  **The first palette failed its own harness on ten counts.** Chosen by eye and
  entirely reasonable-looking, it put `severity.reversible` — PLAN-004's "fully
  undoable" — at delta-E 10.1 from `severity.destructive` — "data is
  intentionally destroyed" — under deuteranopia, against a floor of 12. In the
  high-contrast theme, the one a low-vision user is most likely to choose, the
  same pair measured 4.8. Three further risk pairs collapsed the same way and
  three borders sat below WCAG's 3:1 floor for interface components. The floor
  was not lowered: the severity ramp now varies in lightness as well as hue,
  because lightness survives every colour-vision deficiency and the red-green
  axis does not, and the light theme's `reversible` is teal-leaning so it keeps
  a blue component deuteranopia preserves. Closest surviving pair is 21.9.

  Every check is paired with a mutation it must reject, and each was confirmed
  by deleting the check it targets and watching the table go red — the deletion
  sweep WP-020 established after finding a gate that was load-bearing on
  nothing. The colour maths is anchored outside the repository (black on white
  is WCAG's published 21:1; black against white is delta-E 100 because CIELAB
  lightness runs 0..=100), and the colour-vision matrices are checked by their
  defining property — red and green converge under protanopia and deuteranopia
  but not tritanopia, and greys are untouched by all three — rather than by
  trusting transcribed digits.

  What it does **not** establish is recorded in `docs/work-packages/WP-030.md`
  and repeated in the harness output on every run: it renders nothing, so the
  keyboard, screen-reader, zoom and reduced-motion halves of UI-008 are
  untouched; only declared pairings are checked, so a combination the front end
  invents is invisible to it; and the colour-vision check is a model, not a
  proof — UI-007's redundant channels are the guarantee. M0's "accessibility
  harness runs" criterion is therefore **partially** met.
- `cargo xtask verify-ownership`, closing the mechanically decidable half of
  Section 1.10. Every `docs/work-packages/WP-*.md` now carries an `owned-paths`
  block, which is the same text a reviewer reads, so the prose and the enforced
  data are one thing rather than two that drift. The check refuses a tracked
  file no package claims and a claim matching no file — both mutations were run
  — and reports overlaps rather than forbidding them, because `tools/xtask/**`
  is genuinely shared by three packages and forbidding that would push the
  sharing into prose where nothing can see it. All 100 tracked files are
  claimed. It runs inside `cargo xtask ci`.

  Only exact paths and `directory/**` are understood, and anything else is an
  error rather than a pattern silently matching nothing — the failure mode the
  action scanner was audited for twice.

  WP-030's increment-2 assignment is reserved *ahead of the work*, in an
  `owned-paths-reserved` block the checker reports rather than requires to
  match. Both audits observed that WP-030's assignment did not authorize
  creating a Tauri shell and that widening scope in a pull-request description
  afterwards is the pattern ownership exists to prevent. The reservation also
  records the audits' design constraint: the front end consumes a generated
  typed accessor, never a copy of the palette.

  What this does **not** do is decide whether a given change came from the
  package owning the path — that needs a pull-request-to-package mapping this
  repository does not carry, and it is the remaining half of issue #39.
- ADR-0007, accepted, deciding what SAFE-007's disposable-test token proves.
  WP-020 carried "decide a genuinely independent token factor" as an open
  precondition since increment 1, and both audits repeated it. The queued
  answer — add an entropy source — turned out to be wrong in an instructive
  way: `authorize` trusts nothing inside the directory it verifies, because
  accepting a caller-supplied manifest was a defect that let a hand-written one
  authorize an arbitrary target, and a per-generation random token cannot be
  compiled in. The interlock would have to read it from the fixture root,
  re-creating that exact trust dependency, so randomness would have added a
  dependency and a writable-file trust while defeating nobody.

  Read exactly, SAFE-007 requires the three factors to be *present* and forbids
  one environment variable from standing in for all of them; both hold, and it
  does not require independence. The token is an operator-intent proof, the
  documents already said so, and the precondition is closed by decision rather
  than by code. A real third factor needs state outside both the source tree and
  the fixture root, which is a T2/T3 lab-architecture question recorded as the
  ADR's revisit condition.
- SI-36 filed: SAFE-009 neither permits nor forbids reviewed `unsafe` in a
  test-fixture crate. WP-020's Windows other-name check needs link count by
  handle, `MetadataExt::number_of_links` is unstable behind `windows_by_handle`
  on the pinned 1.96.0 toolchain, and the FFI alternative runs into SAFE-009's
  two lists naming `crates/fixtures` in neither. An enumeration is not a rule,
  so per Section 0.2 it is filed rather than guessed. The residual is recorded
  and narrow: while an authorization is held the Windows share mode refuses
  writes through any name for the object.
- ADR-C1, accepted, fixing the canonical encoding and hash strategy.
- ADR-C5, accepted, fixing the aggregation vocabulary: one `Aggregate` node in
  place of three undefined Section 5 names, on-disk signatures as their own
  nodes, and `StorageSnapshot`. Landed as spec 4.0.0. It resolves four of the
  conflicts blocking WP-010 increment 3 and does not unblock it.

- WP-000 repository foundation: pinned Rust workspace, Tier-1 task runner,
  cross-platform CI, formatting/lint policy, dependency policy, and ADR
  template.
- `cargo xtask verify-actions` enforces SEC-010 digest pinning for GitHub
  Actions. It runs inside `cargo xtask ci` and as a Tier-1 test, and fails
  closed when no workflow can be read.
- `.gitattributes` normalizes line endings to LF in every working tree.
- `SECURITY.md` defines a private disclosure channel and reporting scope.
- Job timeouts on both CI jobs.

### Changed

- `cargo xtask verify-actions` enforces the rule it reports. SEC-010 and
  `AGENTS.md` require every action pinned to a full commit SHA **with the release
  tag in a trailing comment**, and the error message said exactly that — but the
  scanner stripped the comment before checking and `is_pinned` validated only the
  SHA, so a bare 40-character digest passed a gate that claimed to require a tag.
  The comment is now carried through and must name a version. Without one a
  reviewer cannot tell which release a digest corresponds to, so a bump becomes
  40 hex characters to resolve by hand. The repository's own workflows already
  complied, so nothing had to change to pass it — which is why the gap survived.
- `xtask` separates command parsing from execution, so every documented task,
  rejected task, and tier decision is unit-tested without launching a
  subprocess.
- Removed `[build] rustflags = ["-Dwarnings"]` from `.cargo/config.toml`. Cargo
  discovers that file from the working directory, so the flag applied to every
  crate compiled from anywhere inside the repository: third-party dependencies,
  out-of-workspace manifests, and the supply-chain job's
  `cargo install cargo-deny`, which built its entire dependency tree under
  `-D warnings`. That tree compiles warning-free today, so nothing was failing
  yet, but the exposure grows with every dependency added and every rustc
  release that introduces a lint. Workspace lint scope now comes from
  `[workspace.lints]`, and `cargo xtask ci` still fails on any warning in
  workspace code through `cargo clippy -- -D warnings`.

- ~~The project is deliberately unlicensed until it is complete.~~ Superseded
  below. PartMan is now `MIT OR Apache-2.0`.

- The project is licensed `MIT OR Apache-2.0` at the recipient's choice
  (ADR-0006), the Rust and Tauri ecosystem standard. `LICENSE-MIT` and
  `LICENSE-APACHE` carry the texts; every workspace member, the out-of-workspace
  `fuzz` crate, and `packages/canonical/package.json` declare the expression.
  Apache-2.0 supplies the explicit patent grant that MIT lacks — worth having
  for code that drives NTFS, exFAT, and APFS paths — while the MIT arm keeps the
  result usable by GPL-2.0-only projects, which Apache-2.0 alone would not.
  Both arms were already on `deny.toml`'s allow-list, so no supply-chain rule
  was relaxed to accommodate the choice.

  `[licenses.private]` is now `ignore = false`. That exemption existed only to
  stop `cargo deny` reporting `error[unlicensed]` against the unlicensed
  workspace; with the cause gone it is removed rather than left dormant, so the
  project's own crates are checked by the gate that checks every dependency.
  This closes the WP-000 known gap that recorded SEC-005's license inventory as
  unsatisfiable. Two manifests remain outside the gate and are recorded as gaps
  rather than counted: `fuzz/Cargo.toml` is outside cargo-deny's graph, and no
  license gate reads `packages/canonical/package.json`.

  ADR-0006 also makes the GPL boundary binding, which the unlicensed state had
  made moot: PartMan invokes GPL storage tools as separate processes under
  SAFE-004 and reaches UDisks2 over D-Bus, may link LGPL libraries such as
  `libblkid` and `libblockdev` dynamically, and MUST NOT link a GPL library.
  `libparted` is named specifically — it is the obvious dependency for a
  partition editor, and `cargo deny` cannot catch it, because a `-sys` crate
  declares its own license and not that of the C library it links.

- Contributions are accepted, inbound=outbound under Apache-2.0 §5, with no CLA.
  `CONTRIBUTING.md` previously barred outside contributions because the rights
  in one were undefined for both sides; that reason no longer holds.

- `cross-language` and `supply-chain` run on all three operating systems instead
  of Linux alone. Both were narrowed to save metered private-repository runner
  minutes, where Windows bills 2x and macOS 10x, and that constraint is gone.
  The widening is not symmetric bookkeeping: the MODEL-005 parity proof can fail
  on CRLF translation of the shared vector file or on a platform-specific Node
  build, and cargo-deny resolves a per-target graph, so once the platform
  helpers add `windows-sys` or `core-foundation` a Linux-only run would be blind
  to advisories reachable only from Windows or macOS. `prober-acceptance` and
  `fuzz-smoke` stay Linux-only for reasons that were never cost — `blkid` and
  `wipefs` have no Windows or macOS counterpart, and cargo-fuzz has no supported
  Windows target while the decoder under test is byte-oriented and
  endian-independent. Both reasons are now comments in the workflow, so neither
  job gets widened later for the appearance of consistency.

### Fixed

- `cargo xtask verify-change-ownership` closes the half of Section 1.10 that the
  inventory check deliberately left open, and that an audit then caught in
  practice: PR #47 was a nominal WP-000 change that also edited WP-010, WP-020
  and WP-030 documents, and `verify-ownership` passed it because every path was
  claimed by *someone*.

  Every commit now carries a `Work-Package: WP-0NN` trailer, and every changed
  path must fall inside that assignment. A trailer rather than a branch name or a
  label because this repository's branch names are inconsistent — keying on them
  would have been a guess dressed as a rule — and because trailers are already
  used here, need no API call, and stay in the log permanently.

  The assignment is read from the **base revision**, never the working tree, so
  widening your own `owned-paths` block in the same change buys nothing. That was
  the audit's specific criticism, and a deletion sweep confirms it: switching the
  read back to the working tree fails the self-widening test by name.

  `Governance: <reason>` permits editing the assignments themselves, and then
  **only** `docs/work-packages/WP-*.md` may change — otherwise the trailer would
  become a universal bypass for the check it sits beside. That restriction is
  swept too.

  Wired into CI as a *step* in the existing Tier-1 job rather than a new job, on
  Linux and pull requests only. A new job would need a new required-status-check
  name, and adding one without updating branch protection in the same change
  leaves every pull request waiting forever on a check that never reports.

- **The npm advisory check audits every package, not one named directory.** It
  ran in `packages/canonical` by name, because that was the only npm package
  there was. WP-030 reserves `packages/ui/`, `packages/design-tokens/` and
  `apps/desktop/`, and a Tauri front end normally brings its own `package.json` —
  each of which would have been audited by nobody while `cargo xtask
  cross-language` went on reporting success. Discovery is a tree walk now, for
  the same reason the action scanner's is: a gate that checks a hard-coded path
  stops covering the repository the moment the repository grows, and says
  nothing when it does. A package without a committed `package-lock.json` is a
  violation rather than a skip — `npm audit` without one reports a verdict about
  a tree that install time decides. Coverage is unchanged today (one package,
  the same one) and correct when the shell lands.

- **The Dockerfile half of `verify-actions` fails closed.** Structural YAML
  parsing closed the workflow half and left this separate line scanner alone; a
  project audit found three fail-open paths in it and an adversarial pass found
  six more. All nine are permanent regressions, and each was confirmed against
  the old scanner first — the gate exited successfully on every one of them.

  Four needed no unusual syntax at all. **A tab after `FROM`** hid the
  instruction completely, because the matcher was `strip_prefix("FROM ")` and
  BuildKit splits on `[\t\v\f\r ]+` — the cheapest bypass in the file. **A UTF-8
  BOM** on the first line hid it too, which is what a Windows editor produces by
  accident. **`COPY --from=<image>`** and **`RUN --mount=…,from=<image>`** pull
  images that never appear in any `FROM` and were never looked at. And
  **`FROM alpine AS alpine`** shadowed itself, because the stage was registered
  from the same line before the base was tested against it.

  The audit's three: a `$`-prefixed base was skipped outright, so
  `ARG BASE=alpine:3.20` + `FROM ${BASE}` passed while pulling a mutable tag —
  it is refused now rather than resolved, because resolving would have to prove
  no `--build-arg` overrides it; `FROM` matched case-sensitively though
  Dockerfile instructions are not; and `# syntax=docker/dockerfile:1`, which
  makes BuildKit fetch a frontend image and **run it as the builder**, was
  discarded as a comment.

  Two things are deliberately not violations, and are tested so they stay that
  way: `FROM scratch` is not a pull, and `# check=`/`# escape=` name no image,
  so only `syntax=` is treated as a dependency.

- **`unsafe_code = "deny"` was opt-in.** `[workspace.lints]` reaches a crate only
  if that crate's manifest says `[lints] workspace = true`, and nothing checked
  that it did. Measured: a workspace member without the stanza, containing an
  `unsafe fn`, produced **zero** diagnostics and `cargo xtask ci` stayed green.
  A safety property resting on a line somebody has to remember is the shape this
  repository rejects everywhere else. `cargo xtask ci` now asks `cargo metadata`
  for the member list and reads each manifest's text — metadata resolves the
  inheritance away, so by the time it reports a package, a crate that opted in
  and one that did not look identical. Deleting `[lints]` from `crates/tokens`
  now fails the gate by name.

- `verify-change-ownership` enforces the rule it claimed to. A project audit and
  an adversarial pass over the gate found five ways a change could travel without
  belonging to anything, and all five are closed with regressions and a deletion
  sweep each:

  - **One trailered commit laundered every untrailered commit beside it.** The
    trailers of a whole range were folded into one set, and the set had to hold
    exactly one package — so a two-commit pull request passed with a trailer only
    on the second. Each non-merge commit is now asked for its own.
  - **The parse was a line scan, not a trailer parse.** Any line beginning with
    the key after trimming counted, including a fenced example inside a commit
    body, while a genuine lowercase `work-package:` trailer — which git accepts —
    was refused. Git's own parser answers now, through `%(trailers:…)` in the
    same `git log` call: no house dialect to keep in step with git's.
  - **Merge commits are exempt deliberately, and that is now written down.** The
    documents said "every commit", which could never have been enforced: `strict:
    true` branch protection makes `gh pr update-branch` write untrailered merges,
    CI judges GitHub's generated `refs/pull/N/merge`, and `main` carries 51 merge
    commits of which none has a trailer. A literal rule would have failed every
    pull request the day it landed. The prose was corrected rather than the code
    tightened to match a sentence nobody could satisfy.
  - **An empty `Governance:` reason was accepted** and printed as an empty
    parenthesis — an audit record of nothing. And a commit declaring both modes
    was silently judged as governance, so the work package beside it was never
    checked against anything. Both are refusals.
  - **A rename was judged only at its destination.** Detection is on by default
    and `--name-only` prints only where a file landed, so `git mv` carried a file
    out of another package's territory unseen — and a `Governance:` change could
    delete *any* file in the repository by renaming it to a
    `docs/work-packages/WP-*.md` name, because every path the check could see was
    then an assignment document. `--no-renames` makes the source a deletion.

  Two more defects lived in that same expression. `-z`, because `--name-only`
  C-quotes a non-ASCII path, so `crates/tokens/src/café.rs` inside owned
  territory was refused as a stray — a gate rejecting work it should permit costs
  trust as fast as a bypass does. And no `trim`, because git does not quote a
  leading space, so ` crates/tokens/src/lib.rs` was silently normalised onto the
  owned path. A path is a byte string.

- The inventory and the change gate **agree about a reservation**, which they did
  not, and the disagreement deadlocked WP-030. A package may write inside its own
  `owned-paths-reserved` block — `verify-change-ownership` always allowed it — but
  `verify-ownership` did not count a matching reservation as coverage, so the
  first commit to create those files passed the change gate and then failed
  `cargo xtask ci` with "claimed by no work package" about a path the package had
  claimed in advance, in the document, precisely so this could not happen. The
  promotion that would have resolved it has no route that is both green and
  permitted: a governance change moving the paths early leaves `main` red on a
  stale claim, and moving them alongside the files is an assignment edit under a
  `Work-Package:` trailer, which `AGENTS.md` forbids. A reservation counts once
  it matches something; one that matches nothing is still reported, not counted.

- `verify-change-ownership` understands a **generated** file, which it had to
  before any package could add a crate. The gate as first landed made the next
  scheduled piece of work impossible, and not only that piece: `Cargo.lock` is
  claimed by WP-000 alone, and every package that adds a crate or a dependency
  rewrites it.

  Measured against `02ec952` rather than reasoned about. A minimal
  `apps/desktop/src-tauri` plus its workspace member line, committed as
  `Work-Package: WP-030`, was refused for `Cargo.lock` and `Cargo.toml`; the
  identical tree committed as `Work-Package: WP-000` was refused for the crate it
  would have had to create, because `apps/desktop/**` is WP-030's reservation.
  Neither package could take the first step. Landing the member line *before* the
  crate is not a way out either — Cargo fails to load a workspace whose member
  has no manifest, and a glob does not help: `apps/*/src-tauri` matching nothing
  falls back to the literal path and fails the same way, so `cargo xtask ci`
  would have been red for everyone in between.

  A `derived-paths` block declares a path generated rather than authored. Any
  package may then carry it — **but only alongside a manifest that lockfile
  actually resolves.** A lockfile moving on its own is refused with its own
  explanation, because nothing in such a change asks the resolver for a different
  answer, and a transitive dependency quietly re-pinned to a different version
  with a valid checksum satisfies `--locked` perfectly well.

  **The predicate took three attempts, and the first two were lexical.** The
  first accepted any `Cargo.toml` anywhere: `fuzz/` is excluded from the
  workspace, so editing `fuzz/Cargo.toml` cannot change the root `Cargo.lock` —
  yet it unlocked it. The second matched a manifest to the nearest lockfile above
  it, which an adversarial pass broke twice over: a file merely *named*
  `Cargo.toml` — a note, a fixture, a symlink — anywhere a package already owned
  was accepted as a manifest, and deleting `fuzz/Cargo.lock` in one pull request
  let `fuzz/Cargo.toml` vouch for the root lock in the next while `fuzz` stayed
  excluded.

  A fourth lexical predicate standing in for a semantic fact was not worth
  writing. `cargo metadata` is asked which manifests belong to the workspace that
  lockfile locks, so membership is answered by the tool that defines it, and the
  virtual root manifest is included explicitly because adding a member to it is
  the most legitimate reason of all for the lockfile to move. Both earlier holes
  are permanent regressions.

  **The three-OS gate earned its keep on the first run.** The membership lookup
  relativized `cargo metadata`'s absolute manifest paths with
  `if let Ok(relative) = path.strip_prefix(root)`, silently discarding anything
  that did not strip. On macOS nothing did: `std::env::temp_dir()` is
  `/var/folders/…`, `/var` is a symlink to `/private/var`, and cargo answers with
  the resolved path. The set shrank to the workspace root alone and a legitimate
  change was refused — with a message helpfully listing the manifests that would
  have worked, none of which was the one the author had just edited. Linux and
  Windows were green. Both spellings of the root are tried now, and a path that
  matches neither is a refusal rather than a quiet omission.

  A document may also only declare a path generated if it **owns** that path.
  Generatedness is a property of the file rather than a privilege of one
  assignment, and that argument stands — but a document asserting it about a file
  it does not answer for was a unilateral grant to every package, made in a
  change that edits nothing but assignment documents.

  Declaring a path generated is not claiming it: the inventory check still
  requires an `owned-paths` claim, or "this is generated" would be a way to make
  a file belong to nobody while the inventory read as complete. And a derived
  path whose derivation this tool cannot check is refused rather than exempted —
  an exemption nobody can verify is a hole with a comment beside it.

  Four deletion sweeps confirm every part is load-bearing: dropping the manifest
  requirement, accepting any manifest anywhere, accepting any derived pattern,
  and letting a derived declaration count as inventory coverage each fail a
  named test.

  **What it does not establish:** a re-pin travelling alongside a genuine
  manifest change passes. Telling the two apart needs the resolver's answer at
  both revisions — the base tree and a full resolution on every pull request.
  That residual risk is the one the repository has always carried; this does not
  widen it, and it is recorded in `docs/quality/dependency-policy.md` rather than
  implied to be covered.

- WP-020 increment 2d: the Windows halves of issue #51, and one of them was a
  live defect rather than a missing check. A file outside the fixture root
  holding a fixture's exact bytes, hard-linked in at that fixture's name, passed
  name, location, type, length and digest, authorized, and was destroyed through
  the authorized handle — reproduced before it was fixed, with the link count
  reading 2 through the handle the whole time and read only under `cfg(unix)`.
  The comment claiming the Windows share mode closed that hole is deleted: the
  share mode refuses *other* openers, and the write that reaches every name is
  this interlock's own.

  **Neither half needed FFI, so the crate-placement question issue #51 asks to
  be decided is not decided — the premise did not survive measurement.**
  Containment needs a held *directory* handle, not a handle-relative open, and
  `std` opens directories given `FILE_FLAG_BACKUP_SEMANTICS`; the link count
  needs `GetFileInformationByHandle`, which a safe wrapper exposes. So
  `unsafe_code = "deny"` still holds across the whole workspace and no new crate
  exists. The rejected options, including the ntdll route that would have been
  stronger, are recorded in `docs/work-packages/WP-020.md` rather than dropped.

  The root is now held with a share mode excluding `FILE_SHARE_DELETE`, which
  makes the filesystem refuse to rename or delete it during the one window the
  target handles cannot cover — between the root open and the first child open.
  **Windows containment is therefore enforcement by the filesystem, not
  resolution from a handle, and it holds only as far as the driver does.**
  Measured: NTFS, `ReFS` and the Windows SMB server refuse the swap; the WSL 9p
  redirector does not, and a swap staged from the Linux side redirected the open
  to a decoy with the root handle held. Roots that are not locally served are
  refused outright, which over-refuses SMB to a Windows server and is the
  deliberate direction under SAFE-005.

  Six mutations, each caught by the test named for it — and two of the tests
  were decoration until the mutations said so. The namespace test exercised only
  its classifier, so deleting the call site left the suite green; it now goes
  through `RootDirectory::hold`. The reparse-point guard proved unreachable —
  `is_file()` through the handle already refuses a file symlink and a junction,
  measured both ways, contradicting a review finding — so it was removed rather
  than shipped as an untestable branch. A drafted test asserting that the root
  cannot be renamed while an authorization is alive was also rejected: that
  refusal comes from the *target* handle and predates this increment, so the
  test passed with the fix removed.

  What it does not close is listed in `docs/work-packages/WP-020.md`: a
  third-party filesystem presenting as a drive letter is indistinguishable from
  NTFS by a path-prefix check; the link count is a snapshot and a later alias is
  not prevented; `ReFS` file identity is unproven; and the symlink test is
  environment-gated. Tier 2 stays unavailable on every platform.

- WP-020 increment 2c: containment now starts from a held directory object, on
  Unix. Increment 2b bound every check to the target's handle and still opened
  that handle by absolute pathname, and `O_NOFOLLOW` constrains only the final
  path component — so renaming the fixture root aside and leaving a symlink at
  its name redirected the open to an out-of-root file whose length, digest, type
  and link count all matched. No check on the object could have caught it: a
  user's ordinary file may hold a fixture's exact bytes, which is what
  `object_verification_alone_cannot_prove_root_membership` already recorded.

  `Authorization` holds the fixture directory open and targets are opened
  relative to that handle by catalogue basename, via `rustix::fs::openat` with
  `NOFOLLOW`. There are no intermediate components left to redirect.

  *Corrected 2026-07-30: this entry also said the directory handle "outlives the
  target handles because one value owns both". It does not —
  `Authorization::into_targets(self)` moves the targets out and drops the root
  field before the caller uses them. The implementation is unaffected, because
  containment is established at `openat` time and is a property of the returned
  descriptor rather than something the directory handle maintains afterwards; the
  root is worth holding for a different reason, that it denies a consumer a root
  path to reopen by name. The false rationale is corrected here; the same
  sentence in `docs/work-packages/WP-020.md` and the comment in
  `crates/fixtures/src/interlock.rs` are WP-020's to correct, and are recorded in
  the progress notes rather than edited from this package.*
  `rustix` is a safe wrapper, so no `unsafe` appears in this crate and SAFE-009
  needs no exception — the adapter crate F-03 contemplated is not required for
  the Unix half. The regression stages the audit's exact attack through the
  pre-open seam and compares the authorized handle's **inode** to the real
  fixture's, because the decoy holds identical bytes and content cannot tell
  them apart.

  **Windows is not closed.** The standard library exposes no safe
  handle-relative open, and the `NtCreateFile` route needs FFI that SAFE-009
  permits only in an adapter/FFI/helper crate. That platform still opens by
  pathname and the full finding stands there; the code says so at
  `RootDirectory::open_child`, and Tier 2 must not be enabled on Windows until
  it is closed.

- Containers are executable dependencies, and the scanner now sees them. It
  collected only `uses` keys, so a job container
  (`jobs.<id>.container.image`), the documented `container: <image>` shorthand, a
  service container (`jobs.<id>.services.<name>.image`), and a Docker action's
  `runs.image` were all invisible — every one of them code GitHub pulls and
  runs. An `image:` value must now be pinned by content digest, because a tag can
  be repointed exactly like a mutable action tag. `image: Dockerfile` is followed
  to that file's `FROM` lines, with multi-stage builds understood so an internal
  stage reference is not mistaken for a pull.

- A release-tag comment can no longer be borrowed from another step. The check
  searched the whole file for the reference and returned the first comment it
  found, so two steps sharing one SHA — one tagged, one bare — both passed on the
  tagged one's comment, and a reviewer reading the bare step saw no version at
  all. Every occurrence must now carry its own tag.

- A symlinked `action.yml` can no longer escape the repository. Containment was
  checked on the local action's *directory* and then inferred for its contents,
  so metadata linked to a file outside the tree would have been read and treated
  as first-party code. The metadata file is canonicalized and re-checked. A
  deletion sweep found this fix had **no test** — the check could be removed with
  everything still green, which was the audit's criticism of the traversal
  coverage repeated — so a Unix regression now exercises a real symlink.

- **WP-020's status table said containment was closed while the prose beneath it
  said reopened.** The corrections had gone into the deep prose and not the table
  a reader actually consults, which could have authorized Tier-2 work on a
  reopened precondition — the most dangerous kind of documentation drift this
  project can have. The table now carries increment 2c (not started, precondition
  1 reopened) and states that Tier 2 stays unavailable per platform until it
  lands. *Superseded 2026-07-30: 2c has since landed and closed precondition 1 on
  Unix; Windows is still open.* The token's "proves the operator ran the
  generator" wording is corrected
  in all three places it survived, and precondition 3 no longer cites SI-36 as a
  live blocker.

- WP-000 traceability cited three tests that no longer exist, having named the
  text-scanner suite the YAML parse replaced. Traceability naming absent evidence
  is worse than naming none. The rows now cite the tests that exist, including
  the container, Dockerfile, comment-binding and local-resolution regressions.

- Recorded, not hidden: PR #47 was a nominal WP-000 change that also edited
  WP-010, WP-020 and WP-030 documents. The ownership *inventory* passed because
  every path was claimed by someone; only reading caught it. Audit-driven
  corrections to another package's records are a legitimate need with no route in
  the current model, and the fix is a governance route under issue #39 — **not**
  widening WP-000's claims, which is the move that would make the checker
  complicit.

- Action discovery is a structural YAML parse, reversing a decision this project
  defended twice. Three text-based attempts were each defeated by valid YAML,
  and every one of them reported *success with one fewer reference* — silence
  shaped like a pass. The third attempt, a sweep for `owner/repo@ref` tokens
  described here as "syntax-independent" and "unbypassable", fell three ways at
  once: `"actions/checkout@v7"` hides the `@` behind a YAML escape no text
  search decodes; `docker://alpine:3.20` is a documented, mutable step reference
  containing no `@` at all; and a local action outside `.github/actions/` was
  never recursed into, so its own remote references went unread.

  `yaml-rust2` now parses each workflow and every `uses` mapping key in the tree
  is a reference with its value decoded — context-free, so a position GitHub
  adds later cannot be missed. Containers must be pinned by `@sha256:` digest.
  Local references are resolved wherever they live, must carry action metadata
  if they name a directory, must stay inside the repository, and are recursed
  into with a visited set that survives cycles. Unparseable YAML is a violation
  rather than a skip, and the release-tag comment survives as a separate textual
  auditability layer rather than as discovery.

  All three bypasses are permanent regressions. A deletion sweep also caught one
  of the *new* tests not being load-bearing: removing the container-digest
  branch still refused `docker://alpine:3.20`, because `is_pinned` reports it as
  "not pinned to a full commit SHA" — true, but it tells a reader to look for a
  git SHA on a Docker image. The test now asserts the container-specific
  guidance the branch exists to produce.

- **WP-020 precondition 1 is reopened.** *Superseded 2026-07-30 by increment 2c,
  which closed this form of the attack on Unix by opening a direct child relative
  to a held root-directory object. The Windows residual stands, and Tier 2 stays
  unavailable on every platform because no destructive suite exists. The entry is
  kept rather than deleted: the reasoning below is why 2c had to exist.*
  `O_NOFOLLOW` constrains only the final
  path component, which `open(2)` documents plainly and increment 2b overlooked.
  Renaming the fixture root aside and putting a symlink in its place redirects
  the open to an out-of-root file, and matching length, digest, type and link
  count then all pass — the same lesson as
  `object_verification_alone_cannot_prove_root_membership`, one directory up.
  Closing it needs a held root-directory object and an `openat`-style
  direct-child open; more `canonicalize` calls cannot. Tier 2 stays unavailable
  on every platform until that lands.

- SI-36 is **withdrawn the day it was filed.** SAFE-009 permits `unsafe` *only*
  in adapter, FFI, and helper crates, which forbids it in `crates/fixtures` and
  names the route in the same clause. Reading the omission of that crate from
  both lists as ambiguity was using the §0.2 process to convert an
  implementation-location constraint into permission by omission. Precondition 3
  is ordinary work with a known route, not a blocked decision.

- ADR-0007's justification is corrected. It said the token proves the operator
  ran the generator; a pure function of public source cannot prove that history,
  since anyone with the repository can compute the value. It proves only that
  the invocation presented the exact build-derived value — accident friction,
  which is what the decision actually rests on. The decision stands.

- Stale documentation corrected across the review set: `HANDOFF`'s execution
  order, `DECISION_NOTES`' disproved claims, the progress report, the audit
  response, README's WP-000/WP-020/WP-030 rows, WP-000's "only filesystem reads"
  sentence, and four traceability headers that named fewer increments or
  requirements than their own evidence tables contained. The token-mutation
  count said 12 where the table holds 26, and the ownership count said 100 where
  the tree holds 101 — that one is now left to the tool to print rather than
  restated in prose that goes stale on the next file added.

- WP-020 increment 2b: the object binding now starts at the open. Increment 2a
  bound every check to the handle but opened the target by path a second time,
  and the 2026-07-29 follow-up audit showed what lives in that gap: replace
  `root/name` with a symlink to an out-of-root file holding the fixture's exact
  bytes, and the handle is outside the fixture tree while every handle-based
  check — regular file, link count, length, digest — accepts it. Increment 2a
  had claimed a raced symlink was harmless *because* the object is verified
  after opening; `object_verification_alone_cannot_prove_root_membership` now
  records why that was wrong, by demonstrating the object checks accepting an
  out-of-root file. They establish fixture shape; containment is not a property
  of content, and a user's ordinary file may hold those bytes.

  The open refuses to leave the directory: `O_NOFOLLOW` on Unix, taken from
  `libc` because the value differs across Linux, macOS and the BSDs, and
  `FILE_FLAG_OPEN_REPARSE_POINT` on Windows. A test seam fires between
  canonicalization and open, so the race is scheduled rather than sampled —
  `a_symlink_swapped_in_before_open_is_refused` performs the audit's exact
  substitution, and a portable companion covers the seam on Windows, where
  creating a symlink needs a privilege CI cannot be relied on to hold. Removing
  the seam fails both by name.

  Still not claimed closed: the Windows hard-link vector. A hard link is not a
  reparse point, and stable Rust exposes link counts on Windows only behind an
  unstable feature — that is WP-020 precondition 3, and the reason precondition
  1 is narrowed rather than finished.

- The verified handle is handed over rewound. Hashing the contents left the
  cursor at EOF, so a consumer assuming a fresh file would have appended;
  the replace-after-authorization test having to seek explicitly was the smell.

- The action-pin gate no longer depends on recognising the `uses` key. Two
  audits in a row defeated the key-shaped reader with valid YAML it could not
  parse — a quoted key, then an anchored one, `&pin uses: actions/checkout@v7` —
  and each time it reported success having counted one *fewer* reference. A
  mutable tag was invisible rather than rejected, which is the worst failure
  mode a gate has: silence that looks like a pass. Discovery is now
  syntax-independent. An action reference must contain `owner/repo@ref`
  verbatim, whatever surrounds it, so a sweep for that shape finds every
  reference and anything the reader could not attribute to a `uses:` key is a
  violation. Anchors, tags, flow mappings, and every future spelling are
  covered by the same property, rather than by extending a subset one
  demonstrated bypass at a time. Verified against four spellings including both
  the audit's bypass and the tag variant it named as the same class.

- `verify-licenses` is semantic rather than lexical. It matched trimmed lines,
  so the follow-up audit moved the JSON property under `metadata`: the line
  still read `"license": "MIT OR Apache-2.0"` while the document's root
  `license` was `undefined`, and nine artefacts passed. `package.json` is now
  parsed as JSON with the property required at the root, Cargo licences come
  from `cargo metadata --locked --no-deps` (which resolves
  `license.workspace` inheritance the way the toolchain does), and a Cargo
  manifest that neither workspace includes is a violation because no gate
  resolves it. The blanket skip for directories named `generated` is gone.

- `supply-chain` no longer repairs the fuzz lock before auditing it. The
  preflight lived only in `fuzz()`, but `cargo deny` resolves the manifest to
  build its graph, so `supply-chain` silently restored a deleted
  `fuzz/Cargo.lock` entry and audited the repaired copy — the policy tool
  committing the fail-open shape it exists to catch, and leaving a later `fuzz`
  preflight nothing to refuse. `verify_fuzz_lock` is now shared and runs first
  in both entry points; the same mutation now refuses and the lock stays stale.

- `tokenSetVersion` is validated instead of merely present. It was only
  required to be non-empty, so `"not-a-version"` passed while WP-030 and the
  audit response both described parsing as "versioned" — a field nothing
  compares against is documentation. It is now compared against
  `REQUIRED_TOKEN_SET_VERSION` in `policy.rs`, alongside `specVersion`.

- WP-020 increment 2a: authorization holds the object it verified, not the
  name it found it under. The 2026-07-29 audit ranked this the most important
  precondition before any Tier-2 write: `Authorization` carried a
  `Vec<PathBuf>`, and a name can be rebound between verification and
  destructive use. It now carries open `File` handles — `fstat`, the hard-link
  count, the length, and every content byte are read through the handle, and
  the handle itself is what a destructive consumer receives, so renaming or
  swapping the path afterwards changes what the *name* means, never what the
  authorization holds. On Windows the handle's share mode refuses concurrent
  writes, deletion, and renames — through any name, hard links included — for
  as long as the authorization lives; the replace-after-authorization test
  asserts those refusals there and asserts write-through-handle reaches the
  verified object on POSIX. The proof is non-cloneable (a `compile_fail`
  doctest pins it) and consumed once.

  The first version of this fix repeated the defect it was written to end,
  and only planting regressions found it: downgrading the handle `fstat` to a
  by-path `stat` kept every test green, because the difference only shows
  during a race no unit test can stage. `verify_object` now takes no usable
  path, and its test deletes the path before verifying — handle-purity proven
  deterministically. Both planted regressions (`stat`-by-path,
  `fs::read`-by-path) fail that test by name.

  Deliberately not done here: platform no-follow open flags (hardcoding
  `O_NOFOLLOW` values without `libc` is its own defect factory; the by-name
  symlink refusal stays as hygiene and post-open object verification makes a
  raced symlink harmless), and the independent-token decision, which needs an
  entropy source and is a dependency-policy change — still recorded open in
  `docs/work-packages/WP-020.md`.

- The gate can no longer repair the lockfile it claims to enforce. The
  2026-07-29 audit deleted a package entry from `Cargo.lock` and ran
  `cargo xtask ci`: Cargo silently regenerated the entry while building `xtask`
  itself, and all 160 tests passed against a lockfile the repository had never
  committed — the internal `--locked` flags bind only once the binary is
  built. `--locked` now sits in the `xtask` alias, the boundary that builds the
  gate; the same mutation now refuses with "cannot update the lock file". A
  Tier-1 test fails by name if the alias loses the flag.

- `cargo xtask verify-actions` no longer goes blind on valid YAML. The audit
  rewrote one pinned step as `"uses": actions/checkout@v7` — the same YAML key,
  and GitHub executes it — and the scanner reported *success with one fewer
  reference*: the mutable tag was invisible rather than rejected. The scanner
  now enforces a deliberately small YAML subset and **refuses what it cannot
  positively read**: quoted keys are recognized and checked, while flow
  mappings, block scalars, aliases, anchors, escaped quoted keys, explicit-key
  syntax, and values continuing on the next line are each a named violation.
  Local composite actions under `.github/actions/` are scanned too — exempting
  `./` references is safe only if their own metadata is read. What remains
  manual is recorded: nothing verifies a tag comment resolves to its pinned
  SHA, and that is a review obligation, not an automated check.

- The fuzz crate's dependency graph is no longer outside every gate.
  `fuzz/Cargo.lock` was gitignored and the crate is excluded from the
  workspace, so every fresh CI run resolved `libfuzzer-sys` and `arbitrary` to
  whatever the registry served that day and ran their build scripts — on the
  job that executes hostile-byte parser tests, checked by no advisory, licence,
  or source policy. The lock is committed; `cargo xtask fuzz` refuses a stale
  lock before the nightly toolchain is even involved; `cargo xtask
  supply-chain` checks the fuzz graph as a second graph under the same
  `deny.toml` (which required allowing NCSA — `libfuzzer-sys` is
  `(MIT OR Apache-2.0) AND NCSA`, so the permissive NCSA arm is mandatory, and
  the addition is commented in `deny.toml`); and a `/fuzz` Dependabot entry
  updates what nothing previously watched.

- `cargo xtask verify-licenses` closes the recorded WP-000 gap: it walks every
  `Cargo.toml` and `package.json`, fails unless each declares
  `MIT OR Apache-2.0` and both licence texts exist, and runs inside
  `cargo xtask ci`. Previously `fuzz/Cargo.toml` and
  `packages/canonical/package.json` could lose their licence keys with CI
  green.

- WP-000 is reclassified from Complete to in progress. Section 12 defines done
  as generated traceability showing a package's evidence, and
  `docs/traceability/` is hand-maintained; the audit also demonstrated two
  fail-open evidence paths in what Complete claimed to cover. The README row
  now says what is delivered and what is not, and the hosted-runner deviation
  from SEC-010's builder-image digest rule is documented with its residual risk
  in `docs/quality/dependency-policy.md` instead of being silently absorbed.

- WP-030 increment 1a: the accessibility harness no longer takes its standards
  from the file it audits. The 2026-07-29 project audit demonstrated two live
  bypasses through the whole Tier-1 gate: lowering the token file's own `text`
  threshold from 4.5 to 3.0 let normal-size text pass at **3.33:1**, and
  deleting `entity.container` from every theme, pairing and channel table at
  once passed with six fewer checks — a coordinated omission indistinguishable
  from a smaller product, while UI-003 requires containers to be represented.
  Both are the self-consistency failure `AGENTS.md` records for the canonical
  vectors, committed inside the harness written to enforce that rule on
  colours: increment 1's mutation table mutated colours thoroughly and never
  mutated the policy.

  The WCAG floors, the colour-separation floor, the required themes and the
  full UI-003/PLAN-004/UI-011 role vocabulary now live in
  `crates/tokens/src/policy.rs`, outside the audited file. The JSON restates
  the floors for a front end to read, and the audit requires the restatement to
  agree exactly — a lowered value is a finding, not a new setting. Twelve
  mutations were added (threshold lowering with and without a hidden colour,
  threshold removal, role deletion, pairing removal, risk-pair removal, role
  invention, version mismatches); re-running the audit's own reproductions now
  yields 3 and 2 findings where both yielded none. The reader is genuinely
  strict too: `deny_unknown_fields` throughout, so a misspelled
  `nonColorChannels` key can no longer silently disable the UI-007 check.

- `docs/quality/test-tiers.md` overstated the SAFE-007 token: it said the token
  "cannot be known without having generated that fixture set", but the token is
  a pure function of the source, identical on every machine building the same
  commit. The file now carries the honest account `docs/work-packages/WP-020.md`
  always had — the factor is weak, three factors are effectively two, and the
  interlock's strength rests on targets byte-equalling generated images.

- The TypeScript encoder had no `default` arm, so an unrecognized value kind fell
  through the switch, `encode` returned zero bytes, and `hash` published SHA-256
  of the empty string as a well-formed digest over an artifact with no encoding.
  Rust cannot reach this, because its `match` is exhaustive at compile time.
  Payload runtime types are now checked too: `TextEncoder` coerces a non-string
  rather than failing, and `Uint8Array.from` truncates modulo 256. `fromHex`
  refused nothing outside hex, where `Number.parseInt` yields NaN and stores as
  0, so two distinct textual digests decoded to identical bytes.

- Three signature fixtures wrote fields at the wrong offsets, found by an audit
  of the project review. The mdraid 0.90 set UUID occupied `utime`, `state` and
  `active_disks` rather than words 13 to 15, leaving the array identity three
  quarters zero — `blkid` reported it as `fb2871eb-0000-0000-0000-000000000000`.
  LUKS2 wrote its checksum algorithm and UUID inside the 48-byte label field,
  leaving the fixture with no UUID at all. ext4 declared 8 MiB of blocks on a
  4 MiB device, having reused a sector count as a block count. Each correction is
  confirmed against `libblkid` rather than against the struct definition.

- The TypeScript encoder authenticated a forged boolean as the **opposite**
  logical value. The `bool` arm used JavaScript truthiness, so a runtime-forged
  `{ kind: 'bool', value: 'false' }` encoded as `f5` — canonical `true` — and
  `hash` published a digest over the other value, on the MODEL-005 and SEC-001
  authorization boundary. TypeScript types do not protect an object that arrived
  as JSON, over RPC, from a plugin, or as `unknown`, which is why `text` and
  `bytes` already validated at runtime; the reasoning had simply not been carried
  to the rest. Every variant now validates, including map keys — where
  `requireWellFormed` iterated `.length`, `undefined` on a number, so its loop
  never ran and `utf8` then coerced `1` to `"1"`, silently turning a map keyed by
  a forged number into one keyed by text. Rust was unaffected: its `match` is
  exhaustive over a real enum.

  An adversarial pass on that fix found the guards checked fields while a payload
  can lie *between two reads*: `kind` was read twice, containers declared a count
  and then wrote a body from a second read, `bytes` trusted a `Symbol.iterator`
  that `Uint8Array.from` truncated modulo 256, and `instanceof` let a
  prototype-only fake through to a native `TypeError`. Each field is now read
  exactly once and containers are snapshotted before being measured. The tests
  were vacuous the same way — the `array` case used a string, which has a
  `.length`, so the guard never ran and deleting it left the suite green. Every
  case now names the phrase its refusal must contain.
- Raw-byte hashing is no longer a way around strict decode. Both languages
  exported a function that hashed whatever it was handed, documented as "use this
  only for bytes produced by `encode` or accepted by `decode`" — an instruction,
  not a guarantee. `hash_canonical_bytes` and `hashCanonicalBytes` are replaced by
  `hash_encoded` / `hashEncoded`, which decode first, so canonicality is proven
  rather than asserted. The TypeScript version validated a *prefix* and hashed a
  *buffer* until the same pass caught it: `decode` walks the array through its
  `length` property while `crypto.subtle.digest` reads the underlying buffer. No
  digest changed; both languages still reproduce every recorded `sha256`.
- Two fail-open edges and a flaky test harness, found by a progress review. The
  sharpest is one the prober increment introduced in the module the evidence
  increment wrote about: both prober parsers **discarded what they could not
  read**, so an unreadable row was not an "unexpected signature" the comparison
  would report — it was no observation at all, and on the fixture whose
  expectation is *nothing*, an entirely changed output shape parsed as empty and
  passed. The module documentation claimed the signature set was compared in both
  directions. Both parsers now return `Result` and refuse a missing `=`, an empty
  or repeated key, a bad offset, a typeless row, a misplaced header, and a
  repeated row; `probe_output` no longer uses `from_utf8_lossy`.
- Fixture-directory pruning inferred ownership from a filename:
  `root.join(MANIFEST_FILE).is_file()` establishes nothing about who wrote the
  file and follows a symlink besides, so any directory holding an unrelated file
  or link named `MANIFEST` was treated as ours and could lose its other regular
  files. Ownership is now computed — a regular file reached without following a
  link, parsing as one of our manifests, with the token recomputed from its own
  entries. Every failure is a refusal to prune.
- Test sandboxes used fixed paths and deleted them at setup and drop, so two
  concurrent `cargo test` runs of the fixtures crate erased each other's trees —
  in the suite that gates destructive execution. Names now carry the process id
  and a per-process counter.
- Two documented claims that had outrun their code, corrected in opposite
  directions. `corrupt_primary_header_crc` still described itself as producing
  ADR-C3's `Indeterminate` state, contradicting `write_conflicting_backup` in the
  same file, which had already been corrected; the layout test repeated it. Both
  now say *recoverable*. And the review response's note that
  `authorization_cannot_be_forged_outside_this_module` "reportedly stays green
  with `verify_target`'s body short-circuited" — read as evidence the interlock
  suite was blind to that mutation — is false. It was run: **ten tests fail**,
  covering traversal, subdirectory copies, modified bytes, wrong names, missing
  targets and mixed requests. The named test does stay green, because it asserts
  a compile-time property rather than target verification. "Reportedly" marked a
  claim that had never been executed, in a document written about that exact
  failure.
- `every_generated_fixture_authorizes` asserted `targets.len() >= 8`. A floor
  lets catalogue entries be deleted silently while the test still reads as
  coverage; it is now an equality against the catalogue's own length.
- `gpt-missing-backup-512.img` had a backup. It zeroed the last sector only,
  leaving 16 KiB of byte-identical backup entry array at LBAs 8159 to 8190 —
  which any recovery tool that scans rather than seeking to the last LBA would
  find, on a fixture named for having no backup. The whole backup copy is now
  erased.
- `gpt-basic-512.img` and `gpt-basic-4kn.img` shared a disk GUID, both deriving
  it from the literal `"gpt-basic"`. Two different media with one identity is a
  manufactured instance of the collision SI-27 is trying to reason about. Found
  by a new catalogue-wide identity check that no single-image claim could see.

- A subdirectory bypass in the SAFE-007 interlock, introduced by the fix for the
  forged-manifest defect and missed by its own new tests. Containment was a path
  prefix while the name came from `file_name()`, so a byte-identical copy at
  `<root>/sub/blank-512.img` passed the root, name, length and digest checks at
  once. The resolved path must now equal the exact location that fixture is
  generated at.

- The TypeScript encoder could emit bytes its own decoder rejects, violating
  `schemas/canonical-encoding.md` §6.1. `TextEncoder` substitutes U+FFFD for an
  unpaired surrogate rather than failing, so two distinct values encoded
  identically and `encode` was not injective; a map holding both keys emitted a
  declared size of two with byte-identical keys, which §3 makes invalid. The
  encoder now refuses an ill-formed string instead of repairing it, and validates
  map keys before sorting so the refusal cannot depend on insertion order. Rust
  needed no change — `String` is validated UTF-8 — which is the point: the two
  implementations had disagreed about what was *encodable*. Reachable without an
  attacker, since NTFS permits unpaired surrogates in volume labels and INV-008
  requires such structures be represented rather than discarded.

- Windows could not pass `cargo fmt --check`. Git for Windows sets
  `core.autocrlf=true` in its system configuration by default, and the
  GitHub-hosted `windows-*` runner images do not override it, so checkout
  produced CRLF working-tree files that `newline_style = "Unix"` rejects.
  `.gitattributes` now pins LF in every working tree.


- WP-010 increment 4: `cargo-fuzz` targets for the canonical codec (Section
  11.4), plus `crates/domain/tests/canonicality.rs`, which asserts the same
  canonicality property on stable over every single-bit flip, truncation, and
  boundary substitution of every known-good encoding, and every one- and
  two-byte input exhaustively. `cargo xtask fuzz` runs the smoke pass and gates
  CI as its own job. `fuzz/` is excluded from the workspace and is the only
  place a nightly toolchain is permitted; it is pinned by exact date.
