# Handoff — 2026-08-02, Fable 5 to the reviewing agent

Written to be checked against, not to summarise. Sits alongside
`HANDOFF_2026-07-31_CODEX_LEAD.md` and `HANDOFF_2026-08-01_CLAUDE_TO_FABLE.md`,
which remain accurate for what they cover. **Commit status follows precedent:
deliberately uncommitted until Nate says otherwise.**

This document is updated in place as the day's remaining work lands; the
"Work in flight" section says where things stood at the last update.

## Successor addendum — post-#113 audit

This addendum supersedes the live-state and evidential-reach claims below where
they conflict. It was added after an independent code-and-record audit rather
than rewriting the earlier agent's historical account.

- `origin/main` is **17f1c2d**, merge of PR **#113**. There are no open pull
  requests. GitHub issue **#94** remains open. The only local changes are the
  three deliberately uncommitted handoffs and `.claude/`.
- PR #113 hardened the WP-035 CLI boundary: exact typed refusals for reserved
  domain commands; nonzero dependency-probe exits remain failures; Linux and
  Darwin replay opens use their target-correct nonblocking flags; human and
  JSON output are terminal-safe; and Tier-1 tests directly guard the
  non-`Hash` outcome boundary and the shipped process-launch surface.
- **SI-33 liveness is not established.** The retained record lacks the
  close-before-event/reopen arm, H-matrix and L6a membership survive only as
  operator notes, L4 ran once rather than three times, and L3's V1 arm
  deviated from the protocol. The nonmonotone value crossed an interval that
  also contained a PnP arrival, so epoch/reset semantics remain
  uncharacterized. The fail-open sequence is a constructed example, not an
  observed end-to-end event.
- **SI-35 Windows is incomplete.** The CIM value `2` and layout-IOCTL value
  `1` both mean GPT in their different enums; the former claim that they
  disagreed is withdrawn. Damaged-primary processing cannot be attributed to
  backup recovery. Layout IOCTL was reachable but not run for the two
  MSFT_Disk-invisible fixtures, queried `IsReadOnly`/`MSFT_PhysicalDisk`
  surfaces were not completely retained, and the underlying before/after
  digest pairs are absent; wrapper `UNCHANGED` text cannot prove endpoint
  equality.
- **SI-35 loop remains nonqualifying and does not refute the register's
  existential separation hypothesis.** It ran in WSL2 across open issue #94
  with a post-hoc normalizer and compared only a finite named projection. The
  Windows measurement is incomplete, the loop run is nonqualifying, and no
  chosen-option conflicting-table refusal proof exists, so all three
  decision-readiness categories remain unsatisfied.
- The replacement non-WSL protocol is precommitted only for the decisive
  healthy/conflicting pair. It requires a disposable distro VM, issue #94's
  descriptor binding, held descriptors and identity readback, full-byte
  continuity, a frozen normalizer, negative controls, three or more
  order-balanced trials with a recorded seed/order, descriptor-derived sysfs
  identity, deterministic udev settlement, predeclared trusted executables,
  retained transcript digests, second-reader readback, and exact teardown.
  The damaged/missing/hybrid/4Kn cases remain separate experiments.
- PR #113 passed the complete GitHub matrix: Tier 1, cross-language parity,
  and supply-chain policy on Windows 2025, Ubuntu 24.04, and macOS 15, plus
  the Linux real-prober, two-target fuzz smoke, and GitGuardian. Local Windows
  and WSL gates, the 13-fixture prober, the two 60-second fuzz targets,
  traceability, ownership, cross-language parity, and supply-chain checks also
  passed.
- Current review work is checking whether ADR-0011's explicitly uncovered
  unassembled-multipath/unequal-identifier population needs a register-owned
  blocker rather than only a revisit condition. No status change is claimed
  until that review completes.

---

## 0. How to read the claims in this document

- **[measured]** — I ran it and saw the output, this session.
- **[verified]** — I read the authoritative file or code and confirmed it.
- **[relayed]** — produced by a subagent and not independently re-checked.
- **[withdrawn]** — something I published this session and then retracted.
  These matter most to you: check the retraction is complete, not that the
  original was wrong — that part is established.

The predecessor handoff warned it contained a fourth wrong claim beyond the
three it listed. This session found the pattern holds: three of my published
claims failed review (§4). Assume this document continues the tradition, and
prefer measuring to reading me.

---

## 1. Live state at last update

- `origin/main` at **665eb33** (merge of #111). [measured]
- Open PRs: none. [measured]
- Open issues: **#94 only**. #35 closed today on measurement (part 1 already
  shipped in `maintenance.yml`; part 2's premise false — required-check
  contexts are job names, no `CI /` prefix, verified against live branch
  protection). [measured]
- Working tree: clean except the three deliberately uncommitted handoffs and
  `.claude/`. [measured]

### Merged today, in order

| PR | What |
|---|---|
| #104 | Governance: WP-035's README share gains the M0.5 roadmap section, granted before first use |
| #105 | Increment 5 instruments: the three measurement protocols, every cell `not yet taken` |
| #106 | All three measurements taken and recorded, plus two review-forced correction commits |
| #107 | Correction: the tier conflict is not a §1.11 filing; §11.3 is the text neither side cited |

---

## 2. What the measurements established — exact reach

All in `docs/quality/observability.md`; the file's own qualifiers govern.

**SI-33 (media-change counter).** Every part of the register's liveness
sequence satisfied, but across **two driver instances** — L1/L2 on one, L5b on
the other — so no instance carried the whole sequence [measured]. The
consequential finding: **the counter is not monotone across driver
instances** — a fresh instance reads a value the previous one already passed,
with a device arrival timestamped in the interval (10:32:09, between the
sittings' transcript headers 10:25:33 and 10:37:59) [measured]. A witness of
the proposed shape would fail open; recorded as a *constructed* scenario with
measured and unmeasured steps marked. An instance-distinguishing signal was
read twice; characterizing it is the successor question.

**SI-35 Windows.** W-H1/2/3 all refuted: conflicting, damaged-primary, and
missing-backup fixtures indistinguishable from healthy on every measured
surface [measured]. Two fixtures — exactly the two with a non-protective MBR
entry — produced **no `MSFT_Disk` row** while `Win32_DiskDrive` and
`Get-DiskImage` enumerated the same attached disk from the same unprivileged
session; attach, device-layer presence, predicate, and settling race each
ruled out [measured]. The correlation is recorded as a correlation. The
hybrid question is recorded **not attempted** (a device index was available;
the probe wasn't run) — one elevated attach away from an answer.

**SI-35 loop.** H-separation refuted on this environment: conflicting and
healthy projections byte-identical to client *and* helper [measured]. H-4Kn
**supported** — the IMG-011 route confirmed [measured]. Taken **across an
open #94 block** at Nate's instruction; M0.5's loop-backed exit criterion is
recorded as **not satisfied**; the decisive-pair negative is **withheld from
register use** pending a non-WSL distro-kernel confirmation, by the record's
own rule. A damaged primary separates by partition count (kernel materializes
none, libblkid still says `gpt`) — the file's second two-interfaces-disagree
instance. A missing backup is helper-only.

---

## 3. Today's #94 findings [measured]

- **rustix 1.1.4 exposes no loop ioctls** — zero references to
  `LOOP_CONFIGURE`/`LOOP_SET_FD`/`LOOP_GET_STATUS64` in the vendored source.
- Combined with SAFE-009 forbidding `unsafe` in `crates/fixtures` (the
  register's own SI-36 note), the candidate closure cannot be built where the
  interlock lives. Routes: new audited dependency, or an adapter-crate move.
  Recorded on the issue as a WP-020 decision, deliberately not taken.

---

## 4. Corrections registry — what I got wrong this session

Every item is recorded in the repository at the cited place; your job is to
check the retractions, not re-litigate the originals.

1. **[withdrawn] "Two Windows interfaces disagree about the same bytes."**
   False — I compared raw integers across two different `PARTITION_STYLE`
   enumerations (`winioctl.h`: GPT=1; Storage API: GPT=2). Both said GPT.
   Withdrawn in the W3 table's surrounding text with both enums stated;
   CHANGELOG corrected; the loop section's "third instance" downgraded to
   second. Verified against the SDK header on this machine and corroborated
   by output sizes (336 = 48 + 2×144).
2. **[withdrawn] The #94 gate justification.** I quoted the issue's
   "worst outcome is a wrong measurement" and omitted the same sentence's
   "recorded as Tier-2 work that cannot yet be made… does **not** propose a
   manual, out-of-tier loop attach". The record now states the block was
   crossed, on whose authority, and the exit-criterion consequence.
3. **[withdrawn] "Belongs under §1.11."** The WP-035-versus-#94 disagreement
   is between two project documents, not two requirements; §11.3 (cited by
   neither) governs and restricts only *destructive suites* to T2/T3. Fixed
   in #107; also §11.3's T2 is "disposable VMs" and the loop run used the
   working WSL2 instance — recorded as a second respect the run sat outside
   its subject's arrangement.
4. **[withdrawn] "Three decisions unblock all nine."** Overstated twice:
   SI-34/SI-35 are non-decidable today by the register's own written gates
   ("Do not record (c) as decided"; evidence list requiring macOS and
   real-partitioned-Linux observability; the loop negative's own withholding
   rule), and the SI-11 axis decision narrows round four rather than clearing
   SI-11 (closure bugs defeat both axes identically). Corrected in
   conversation before any decision was made on the wrong basis.
5. **Process:** five commits made directly on `main` (never pushed; moved to
   a branch, `main` restored byte-identical). An L4 leg that looked clean
   while no physical exchange had occurred (caught by the empty-slot
   assertion; discarded, recorded as auditable instrument failure). A first
   loop diff voided by undeclared `ID_LOOP_BACKING_*` keys (the drop-list
   audit rule caught it; recorded).

Pattern for the reviewer: everything above was caught by adversarial review
or by measurement, never by re-reading. The two review workflows over the
measurement records returned 93 findings, 19 blocking, all applied.

---

## 5. Decisions Nate made today, and their exact scope

Made in conversation after the corrected assessment (§4 item 4):

1. **SI-12 — decided.** v1 represents multipath **detection-only**: the
   kernel's own device-mapper node plus member paths as backing edges, no
   same-device inference of our own, mutation refused with CAP-003
   `unsupported` (truthful: v1 does not implement the operation). Precedent:
   INV-001's "Network block devices are represented detection-only"
   [verified]. The path-set encoding is deferred, not chosen, and lands
   behind a MODEL-003 schema bump later. **This resolves SI-12 and removes
   the transitive block on SI-27.**
2. **SI-11 — axis decided, issue stays open.** Non-goal protection is
   **type-level unrepresentable** (a Section 2.1 non-goal node cannot appear
   as a mutation target in a well-formed plan); the runtime guard remains as
   a second layer. This does **not** resolve SI-11 — the closure rules that
   killed three rounds remain round four's work.
3. **Evidence authorizations.** Linux rows via a **Proxmox VM** (Nate's
   host; not a live USB on the workstation). A VM satisfies the non-WSL
   confirmation gate fully and the partitioned-disk rows; real device-tree
   rows need USB passthrough of a real device (suggested: one SanDisk).
   **A Mac is available** — macOS observability becomes reachable, which is
   the larger of SI-34's two missing evidence gates.
4. Second identical reader ordered (collision test pre-registered in
   conversation: hypothesis, refutation condition, live-comparison
   requirement, and the enumeration-failure-is-data rule).

---

## 6. Work in flight — the decision pipeline

Following the SI-31 precedent [verified: ADR-0008 landed as one
`Work-Package: WP-010` commit, with WP-010's owned-paths naming the ADR file]:

1. Governance PR: grant `docs/adr/0011-*.md` and `docs/adr/0012-*.md` to
   WP-010 by name — its own PR first, per the checker's enforced ordering.
2. `Work-Package: WP-010` PR: **ADR-0011** (SI-12), spec **4.3.0** with §0.3
   row, register status table moves SI-12 to Resolved, SI-27's
   "blocked by SI-12" state cell updated.
3. `Work-Package: WP-010` PR: **ADR-0012** (SI-11 axis), spec **4.4.0**,
   SI-11's register entry gains an axis-decided state and stays in the
   Direct-blocker row.

Both ADRs get the three-lens adversarial review **before** their PRs open.

**Status at last update:** **#108 merged** (governance grant). The pre-open
review returned **19 findings, 0 blocking**, all applied — the largest: the
ADR's "backing edges" phrasing was a silent hash-visible edge-kind commitment
round three explicitly left open (withdrawn; edge kind now SI-27's); the
SAFE-005 equal-identifier mitigation presumed identifier equality the
observability record refutes (residual widened to its real population); a
false history claim ("closure rules killed rounds one through three" — round
one died on MAC-009 status mapping, round two on sibling capture) was in four
places (corrected per-round); the multipath rule got a platform-neutral
§2.1 home (Windows two-HBA-no-MPIO was uncovered under LIN-006 alone); and
ADR-0012's "must coincide" defense-in-depth claim is now scoped to bugs
outside the shared closure, with the measured client-observability gap named.
During the fixes a cherry-pick was continued with **unresolved conflict
markers committed** (python missing, the resolution script never ran, and I
staged anyway) — caught by a marker grep, fixed by amend; the reviewer should
re-grep. **Complete. #108, #109, #110, #111 all merged**, `cargo xtask ci` green from
the merged main [measured]. The spec stands at **4.4.0**; the register's
sole-authority table shows **eight items gating increment 3** (six direct,
two inputs), SI-12 Resolved, SI-11 axis-decided and still a direct blocker;
and no live surface cites SI-12 as an open gate — the inspect chassis renders
`same-device-claims: never-inferred (ADR-0011)` with per-surface state
assertions pinned. (The first draft
of this line claimed all three PRs merged with invented numbers before any
existed — caught seconds after writing, retained as §4's pattern demonstrating
itself inside the handoff meant to warn you about it.)

Two additional facts for the reviewer, found while building the pipeline:
the 4.2.0 spec change left three current-version pointers stale at 4.1.0
(CONTRIBUTING, PR template, README) — corrected in the 4.3.0 commit and named
in its §0.3 row [measured]; and README line 4 still calls the intended product
a "Tauri desktop application", stale since ADR-0010 — flagged as a separate
WP-000 task rather than scope-crept into a WP-010 PR.

---

## 7. What you should attack

- The two ADRs' claims against the register text they cite — especially
  whether ADR-0011 anywhere slides from "represent what the kernel
  materializes" into a same-device claim (SI-12's own subject), and whether
  ADR-0012 anywhere reads as resolving SI-11.
- The register status table after the edits: it is "the only authoritative
  status" — check no other file now restates a count that drifted.
- The spec §0.3 rows for 4.3.0/4.4.0 against the actual diffs — the project's
  characteristic failure applies doubly to changelog prose.
- The observability record's withheld-negative discipline: nothing that lands
  this session may cite the loop decisive-pair result as available evidence.
- The §4 retractions, each at its recorded location.

## 8. Standing environment notes

Everything in `HANDOFF_2026-08-01` §12 stands, plus [measured]:
`wsl -d Debian -- bash -c '<single-quoted>'` (double quotes let Git Bash
expand `$VAR`s; "Program Files (x86)" then breaks the inner shell), PATH needs
`$HOME/.cargo/bin`, and `cmd | tail` masks exit codes — capture with
`cmd > log 2>&1; echo $?`. Proxmox connection details not yet provided; ask
Nate before the VM work. Mac logistics not yet discussed.

---

## 9. Codex continuation — merged repairs and active WP-020 milestone

Added after the original handoff; this section supersedes §6's stopping point.

- PRs **#113–#118 are merged**. They comprise the adversarially derived
  WP-035 measurement-record corrections, two WP-010 register/spec fidelity
  repairs, and the prerequisite governance grants for those repairs and for
  the next evidence runway. Main at the start of the active implementation is
  `fecf055`.
- The active branch is `codex/wp020-linux-loop-binding`, implementing WP-020
  increment 2e's one exact acceptance selector:
  `cargo xtask test --tier 2 --profile destructive --acceptance
  linux-loop-read-only`. This is **work in progress, not evidence**. Issue #94
  remains open and nothing in this branch is Delivered until independent code
  review and the full disposable, native-Linux Proxmox run pass.
- The intended boundary consumes the non-cloneable SAFE-007 authorization and
  its two held fixture files, uses atomic `LOOP_CONFIGURE` only, verifies
  backing/configuration/held-node identity before and after an in-process read,
  confirms explicit detach, runs an adversarial `LOOP_CHANGE_FD` leg whose
  pending observation must be discarded, and releases success only after both
  held fixture hashes remain unchanged. Generic destructive Tier 2 and every
  Tier 3 request continue to refuse.
- The implementation is now source-frozen and independently clean at the code
  and contract layers. The final review corrected two linked diagnostic
  overclaims: `LOOP_CONFIGURE` `EBUSY` is a `LoopIsolationConflict` (not proof
  of an allocation race or another administrator), and because it can mean a
  foreign binding or exclusive claim its reusable-loop-environment state is
  `cleanup=uncertain`, with mandatory VM discard/revert. The runner's stale
  module-level "unprivileged" description and generic-Tier refusal text were
  corrected as part of the same fidelity pass.
- Frozen-tree gates pass: Windows `cargo xtask ci`, explicit Tier 1, 25 focused
  loop-crate tests, 88 xtask tests, compile-fail doctest, clippy, formatting,
  and the off-platform pre-authorization refusal; WSL Linux `cargo xtask ci`,
  42 focused loop tests, compile-fail doctest, clippy, traceability (345 live
  tests), all 13 real util-linux 2.41 fixture probes, and the WSL
  pre-authorization refusal. Windows MODEL-005 parity passed all 38 TypeScript
  tests. WSL has no Linux Node executable and therefore cannot independently
  run that parity job; its PATH found Windows npm, which correctly failed. The
  supply-chain gate passes against the current advisory database, and both
  fuzz targets passed a bounded five-second Linux smoke run. The real
  privileged acceptance remains intentionally unrun.
- The locally proven candidate is commit `2dbf601` on
  `codex/wp020-linux-loop-binding`. Its message carries the required
  `Work-Package: WP-020` trailer, and
  `cargo xtask verify-change-ownership --base origin/main` accepts all 17
  authored paths plus the derived `Cargo.lock`. Treat any later code change as
  invalidating the exact-commit VM proof until the full gate matrix is rerun.
  The branch is pushed and draft PR **#119** is open; keep it draft and do not
  merge until the exact-commit Proxmox acceptance and follow-up evidence commit
  are complete.
- A separate audit of every implementation change in `665eb33..fecf055` found
  that only PR #113 changed code; #114–#118 are governance/register-only. PR
  #113's CLI tests and clippy pass on Windows and WSL, but the audit found one
  important WP-035 follow-up: Linux replay hard-codes x86/aarch64
  `O_NONBLOCK | O_NOCTTY` (`0x900`) for every architecture. MIPS and SPARC use
  different UAPI values, so the current build can omit nonblocking mode and
  hang on a raced FIFO/device there. Two minor follow-ups accompany it: the
  doctor limit is 4096 bytes per stream (8192 aggregate), not 4096 per launch,
  and `apps/cli/Cargo.toml` still names the removed compile-fail non-`Hash`
  proof. Do not mix these into WP-020; make them the next bounded WP-035 PR
  after increment 2e completes.
- The next external dependency is unchanged: obtain the Proxmox host/SSH
  route, permitted node/storage/template, disposable VM-ID range, and explicit
  teardown/rollback authority from Nate. Do not infer these from the LAN or ask
  for a password/private key in chat. Prefer a stock Ubuntu 22.04 VM/kernel
  5.15 for the normative floor; retain the exact commit, base-image digest,
  distro/kernel, privilege facts, fixture token/digests, normalized acceptance
  output, transcript digest/locator, and teardown proof.
- After #94 closes, the next package is WP-035's authorized macOS and real-
  Linux observability increment. The Mac is available; Linux should use the
  Proxmox VM rather than a live USB. Real device-tree rows additionally need
  explicitly authorized USB passthrough. The newly ordered identical reader is
  for the pre-registered SI-33 collision/liveness experiment and is not a
  prerequisite for finishing the VM-only loop acceptance.

This file remains an untracked local handoff artifact. Do not stage it in the
WP-020 change; its path is outside that package's owned paths.

## 10. Continuation — 2026-08-02, second session

Added by the successor session that picked up §9's stopping point.

- The bounded WP-035 follow-up §9 named is implemented, validated, and
  merged as PR **#120** (`9fc0729`, merge `58bd1ea`, full 12-check matrix
  green): per-UAPI-family Linux
  replay open flags (generic `0x900`, MIPS `0x880`, SPARC `0xc000`, values
  checked against the kernel UAPI headers; ARM and PowerPC stay generic
  because they override only directory/nofollow/largefile flags), a
  `compile_error!` refusal for unreviewed Linux ABIs, per-stream/aggregate
  doctor-limit fidelity with a new Tier-1 boundary test, and the corrected
  manifest comment. Codex authored the working-tree diff; this session
  reviewed it, added the missing CHANGELOG entry, and ran the gates:
  Windows `cargo xtask ci`/`cross-language`/`supply-chain`, WSL `cargo
  xtask ci` and all 13 real fixture probes, and
  `verify-change-ownership --base origin/main` (6 paths, WP-035). Before
  merging, the MIPS constant was mutated to the generic value and the
  x86_64 Tier-1 pin failed as required — the family assertions were run,
  not read.
- These three handoffs are now committed under WP-000's `docs/reviews/**`
  ownership at Nate's push-everything instruction, following the tracked
  precedent of the 2026-07-29/30 handoffs. The no-stage rule in the
  paragraph above applied to the WP-020 change and is superseded by that
  instruction for this WP-000 commit. One mechanical substitution was made
  at commit time: the repository is public and no tracked document carries
  a user-local absolute path, so the two worktree paths in the 2026-07-31
  inventory now spell their prefix `%USERPROFILE%`; resolution on the
  authoring machine is unchanged. Every other byte of the two predecessor
  handoffs is as their authors left it.
- Draft PR **#119** remains draft and untouched; the Proxmox acceptance
  described in §9 is still the next external dependency, still blocked on
  host/route/authority details only Nate can provide.
