# Project progress and strategy review — 2026-07-30

Feedback for the next agent, based on the repository at
`04ffc255c79e7a8109e45370e509e9a2a083f59c`, `origin/main` at
`b2800a57d59a20337e039a8257b7d04d56791747`, open PRs #65 and #66, and the
current issue register.

Read `AGENT_BUILD_SPEC.md` and `AGENTS.md` in full before acting. They are
normative. This review evaluates current progress and recommends direction; it
does not replace the specification, work-package assignments, ADRs, or issue
register.

## Overall assessment

Development is going well as safety-critical infrastructure work, but the
project is not yet meaningfully usable as a partitioning utility.

The foundation is unusually disciplined for this stage. Three-platform CI,
change ownership, dependency and action auditing, deterministic fixture
generation, cross-language canonical encoding, design-token policy, fuzzing,
and a deliberately unavailable destructive tier are all real. Recent work also
closed the governance defects found by the previous audit instead of merely
editing the claims around them.

Product progress is much earlier. There is no discovered storage inventory,
canonical topology, capability engine, operation planner, platform adapter,
CLI, desktop shell, privileged helper, or end-to-end storage flow. WP-010
increment 3 remains blocked, WP-040 has not started, and WP-030 has no shell.
The honest label is **pre-product foundation**, not an early partition manager.

That is not a failure. For software that can destroy data, refusing to build on
an unsound identity and authorization model is the right choice. The risk is
now one of balance: foundation and repository machinery are growing faster
than the user-visible vertical slice. After the current traceability work is
made evidence-preserving, the centre of gravity should move decisively toward
M1's honest, read-only product.

## Evidence checked

- The complete `AGENT_BUILD_SPEC.md` 4.0.0 and current repository instructions.
- Current README, work-package records, specification-issue register, ADRs,
  quality documentation, traceability files, prior audits, and the new handoff.
- Repository changes since the audit at `02ec952`, including PRs #54 through
  #64 and the open changes in PRs #65 and #66.
- WP-020's Unix and Windows target-acquisition code, regression tests, and
  documented residual risks.
- WP-030 integration and ownership decisions.
- Current GitHub pull requests, issues, commit trailers, and required checks.
- Local verification:
  - `cargo xtask ci` — passed, including 228 Rust tests and all Tier-1 policy
    gates;
  - `cargo xtask cross-language` — passed, including 28 TypeScript tests and an
    npm audit with no reported vulnerability;
  - `cargo xtask supply-chain` — passed for the root and fuzz dependency
    graphs;
  - `git diff --check origin/main...HEAD` — passed.

No production code was changed by this review. The only new repository content
is this feedback document.

## Current scorecard

| Area | Assessment | What is still missing |
| --- | --- | --- |
| Safety culture | Strong | Real Tier-2/Tier-3 execution evidence, power-loss testing, recovery drills, and the privileged-helper boundary |
| Repository governance | Strong and improving | Complete generated traceability and scheduled long-running security/fuzz jobs |
| WP-000 | Largely mature | Evidence-preserving trace generation and a small amount of M0 closure |
| WP-010 | Good encoding foundation; critical path blocked | Canonical domain model, identity rules, protection projection, topology naming, and storage plan types |
| WP-020 | Strong fixture/interlock foundation | A disposable-target lab and proof sufficient to enable Tier 2; Windows filesystem-provider classification remains incomplete |
| WP-030 | Good static token/a11y foundation | A real shell, rendered-state testing, keyboard/screen-reader evidence, and integration with actual topology data |
| WP-040 and adapters | Not started | Inventory API and all three operating-system implementations |
| User-facing product | Not started | Read-only inspection, planning, execution, recovery, and UI/CLI workflows |
| Documentation | Detailed and mostly honest | Less duplication, an authoritative generated status view, and correction of the current traceability handoff |

The quantity of tests is encouraging, but it should not be mistaken for product
completion. Most current tests prove foundation and policy behaviour. The next
meaningful progress metric is an end-to-end, read-only topology rendered from
real platform discovery through the canonical model.

## What is working especially well

### 1. Safety constraints are shaping the implementation

Tier 2 and Tier 3 still refuse to run. The project has not turned a test-only
interlock into a general authorization mechanism or used a green fixture suite
to imply that destructive operations are safe. This is exactly the discipline
the prime directive requires.

The confirmed interchangeable-media identity defect in SI-28 is being treated
as a data-loss defect, not an inconvenient edge case. That decision protects
the product's long-term credibility.

### 2. Audits lead to mechanisms, tests, and governance fixes

Recent changes closed real holes in action dependency discovery, per-commit
trailers, base-revision ownership, generated lockfile handling, workspace lint
inheritance, and WP-030 integration choreography. Adversarial regression tests
were added rather than relying on prose alone.

This should remain the standard: a policy claim is not closed until the
repository can reject a representative violation.

### 3. Cross-language and visual sources of truth are centralized

Rust and TypeScript share canonical encoding vectors, and UI implementations
are expected to consume one token source. This avoids the common failure mode
where each implementation passes against data it owns.

### 4. The roadmap has a sound staged-release shape

M1's read-only product is a valid early ship point. M2 adds planning and dry
run. Only M3 introduces writes to disposable media. That sequence is much
safer—and more likely to produce useful feedback—than waiting for a broad
multi-filesystem mutation engine before exposing any product.

## Findings that should change the immediate plan

### F-01 — High — WP-010 is the real product and safety critical path

WP-010 increment 3 is still blocked by the authoritative issue register:
directly by SI-11, SI-27, SI-28, SI-31, SI-33, SI-34, and SI-35, and
transitively by SI-12. These are not naming cleanups:

- SI-28 is confirmed on hardware: a reader serial can identify the transport,
  not the removable medium. Two indistinguishable cards can therefore bind a
  destructive plan to the wrong card.
- SI-33's continuity-witness direction still needs real-hardware liveness proof
  across media swap, idle time, and reopen.
- SI-34 needs a protection/freshness projection that is observable across
  privilege boundaries and cannot weaken a helper's refusal.
- SI-35 needs platform measurements for partition-table states that the current
  file-based `libblkid` probe cannot observe.
- SI-12 and SI-27 determine multipath representation and stable node naming.
- SI-31 still needs the agreed collection ordering and nested depth-budget
  semantics landed normatively.

Do not implement a placeholder domain graph and let the UI or adapters harden
around it. Resolve these issues with focused experiments and ADR/spec changes,
then implement the smallest coherent canonical model needed for read-only M1.

### F-02 — High — Product delivery needs a vertical-slice constraint

Since the previous reviewed point, the repository added substantial xtask,
interlock, and documentation code but still added no product path. More
foundation work is justified where it is required by the specification, but
new policy machinery should now face a higher bar:

1. Is it required to unblock the next read-only vertical slice?
2. Does it close a demonstrated safety, supply-chain, or governance failure?
3. Is there an executable violation test?

If all three answers are no, defer it. Track milestone outcomes—real inventory,
canonical topology, capability explanation, rendered inspection—not test count
or documentation volume.

### F-03 — High before Windows Tier 2 — “Local volume” is not enforced as claimed

The Windows interlock refuses UNC and verbatim-UNC roots and holds the root open
with a restrictive share mode. That is meaningful protection. However,
`root_namespace_is_local` classifies by path prefix only. The code itself says
this does not identify WinFsp, Dokan, sshfs-win, or a mapped drive that
canonicalizes to a drive letter.

Consequently, the WP-020 wording “delivered—on locally served volumes only” is
stronger than the executable predicate. It currently means “not expressed as a
UNC path,” not “proved to be a local Microsoft filesystem.”

This is not an immediate user-data exposure because Tier 2 remains unavailable.
Before Windows Tier 2 can be enabled, choose and prove one of these boundaries:

- open descendants relative to an authenticated root handle through a narrow,
  reviewed Windows implementation; or
- make the harness create and verify its own known VHDX/NTFS/ReFS disposable
  target and refuse every unrecognized provider.

Until then, describe the result as UNC refusal plus a known third-party
drive-letter residual. Do not close it as general local-volume containment.

### F-04 — Medium — PR #65 is correctly held, but the handoff's metric is wrong

PR #65 turns WP-000's 31 hand-written evidence rows into 13 generated evidence
rows. The earlier “33-row” count included the Markdown header and separator;
the evidence-row difference is therefore 18. More importantly, requiring the
final generated table to have 31 rows is also incorrect. Generation may
legitimately consolidate or relocate evidence.

Before merging the handoff, replace the arithmetic and row-count criterion with
a row-by-row migration ledger. Every previous relationship needs one explicit
disposition:

- generated equivalent;
- intentionally consolidated;
- moved to a named narrative/ADR evidence record;
- superseded or invalid, with rationale; or
- unsupported, which remains a blocker.

The merge condition should be **zero unexplained evidence loss**, not equal row
counts.

The generator also needs a durable way to carry non-test evidence such as
configuration files, ownership policy, pinned toolchains, and verification
commands. A machine-validated structured evidence block in the owning work
package is preferable to a second hand-maintained output table: validate
requirement identifiers, paths, ownership, and optional check commands, then
generate the presentation from those inputs.

Section references need stable anchors or durable process requirement IDs.
Bare section ordinals are too fragile to become the only generated key.

### F-05 — Medium — Documentation is faithful but too distributed

The work-package records are valuable engineering journals, yet current status
must be reconstructed from README rows, long work-package narratives,
traceability tables, issue registers, ADRs, review files, and GitHub state.
That creates recurring drift and makes old audits look current.

Keep the detailed history, but establish one generated current-state dashboard
with:

- milestone exit criteria;
- work-package increment status;
- blocking issue IDs;
- latest evidence command/run;
- supported/preview/blocked/unsupported capability counts by platform; and
- the next dependency-ready deliverable.

Mark review documents as dated snapshots and index or archive superseded ones.
Keep experiments and rejected designs in ADR/evidence records; keep the
work-package front matter and status section short enough to audit quickly.

## Recommended execution order

### P0 — Finish the current governance work without losing evidence

1. Correct PR #66's evidence-row count and acceptance criterion.
2. Build the traceability migration ledger.
3. Extend the structured source model for non-test evidence and stable section
   references.
4. Regenerate and prove zero unexplained evidence loss.
5. Merge PR #65 only after that proof; roll generated traceability into other
   packages as those packages next change rather than starting an unrelated,
   repository-wide rewrite.
6. Land the scheduled advisory and long-fuzz work tracked by issue #35.

This is a bounded final investment in the foundation, not a new phase of
meta-tooling.

### P1 — Resolve WP-010 with experiments before abstractions

Work in this risk order:

1. **Identity and continuity:** SI-28 and SI-33, using an actual matrix of
   identical readers/media, swap/reopen/idle cycles, and deliberately ambiguous
   devices.
2. **Protection observability:** SI-34 and SI-35, measuring unprivileged and
   privileged views on Windows, Linux loop devices, and macOS disk images/VMs.
3. **Topology identity:** SI-12 and SI-27, with multipath and aggregation
   fixtures.
4. **Canonical semantics:** SI-31 and the remaining depth-budget question.
5. **Protection representation:** SI-11 after the closure rules are testable.

Each decision should end in a small normative change plus conformance fixtures,
not another broad speculative model.

### P2 — Deliver M1 as a thin, real, read-only product

Build one end-to-end path:

```text
platform discovery
    -> normalized evidence
    -> canonical topology snapshot
    -> capability explanations
    -> stable JSON/CLI
    -> desktop topology view
```

Windows can be the first live adapter because it is the primary platform, but
the interface and fixtures must be exercised immediately by Linux and macOS
implementations. “Cross-platform” should mean one canonical semantic graph with
platform-specific evidence and capability reasons, not a least-common-
denominator API.

The first shell should render actual canonical fixture graphs and then actual
read-only discovery. A brief layout-only scaffold is useful for integration
proof, but a long-lived shell backed by fabricated product data will create
model and accessibility churn.

M1 should let a user:

- identify every disk and why PartMan believes it is that disk;
- inspect partition tables, partitions, filesystems, mount/use state,
  encryption, pools/containers, and health evidence;
- see ambiguity, conflict, stale data, and unsupported topology explicitly;
- export a stable redacted diagnostic snapshot; and
- understand why an operation would be supported, preview-only, blocked, or
  unsupported on this exact host.

That is already a valuable cross-platform product and the right substrate for
planning.

### P3 — Build the disposable-target lab before mutation features

Create a reproducible Tier-2 matrix:

- Windows VM plus programmatically created VHDX targets with verified
  filesystem/provider identity;
- Linux VM plus loop, device-mapper, LUKS, LVM, mdraid, and multipath fixtures;
- macOS VM or dedicated runner plus disk images, GPT, APFS containers, and
  encrypted-volume cases.

Add hotplug, media replacement, stale-plan, process crash, timeout, partial
write, full disk, and power-loss/fault-injection scenarios before M3 writes are
enabled. Recovery and journal replay must be tested as product behaviours, not
only internal state transitions.

### P4 — Add planning, then the narrowest useful write wedge

For M2, make the plan an inspectable, immutable object:

- exact before/after graph;
- byte ranges and alignment;
- capability evidence and tool/backend version;
- risk, consequence, reversibility, prerequisites, and recovery path;
- fresh identity/protection verdict;
- canonical plan hash used by UI, CLI, journal, and helper authorization.

For M3, start only with basic GPT/MBR create/delete and a very small set of
format operations on disposable non-system media. Defer system-disk resize,
encryption changes, cloning, complex pool mutation, and recovery tooling until
the journal, privileged helper, rescue workflow, and failure injection prove
them safe.

## Platform direction

### Windows

- Treat basic disks and removable media as the first supported substrate.
- Model BitLocker, Recovery, drive-letter/mount-point state, BCD sensitivity,
  sector geometry, and system-disk protections explicitly.
- Detect Storage Spaces and dynamic disks early, but report them as protected or
  unsupported before attempting mutation.
- Prefer documented storage APIs and handle-based identity. Never infer safety
  from a drive letter alone.
- Test 512e, 4Kn, USB bridges, identical card readers, VHDX, ReFS, hotplug, and
  disks above common size boundaries.

### Linux

- Normalize kernel/udev evidence while preserving connection paths and
  multipath identity.
- Make backend versions part of capability decisions for `libblkid`,
  `wipefs`, filesystem tools, LUKS, LVM, mdraid, and device mapper.
- Use structured process invocation with the existing allow-list, environment,
  timeout, and output rules.
- Do not treat `/dev` names as durable device identity.

### macOS

- Build discovery around Disk Arbitration and explicit APFS
  physical-store/container/volume relationships.
- Represent FileVault, Recovery, SIP, and the sealed system volume as
  protection facts, not generic partitions.
- Use disk images for early conformance and a VM/dedicated host for behaviour
  that images cannot prove.
- Prefer honest read-only reporting to mutation through undocumented or
  release-fragile behaviour.

## Product qualities that could make PartMan best in class

Breadth alone will not distinguish this project. The strongest differentiator
is **capability honesty**:

- explain what is known, inferred, ambiguous, stale, or unsupported;
- show why an operation is blocked on this device and host;
- show current and planned topology side by side;
- make identity and destructive target selection visually unmistakable;
- provide guided and expert views over the same plan;
- make every plan exportable, reviewable, and reproducible;
- treat recovery instructions and evidence as part of the operation, not an
  afterthought.

Accessibility must be exercised through real workflows: full keyboard
navigation, focus order/restoration, screen-reader names and announcements,
zoom/reflow, reduced motion, high contrast, non-colour status cues, and
large/complex topology navigation. Static token contrast is necessary but not
the rendered UI-008 proof.

Use scenario-based research with beginners, administrators, dual-boot users,
repair technicians, and accessibility users. Measure wrong-disk selection,
ability to explain the planned result, completion time, recovery success, and
confidence calibration—not visual preference alone.

## Architecture and security guardrails for later packages

- Threat-model the UI/CLI-to-helper boundary before implementing the privileged
  helper.
- Use strict versioned request schemas; never pass shell command strings or
  ambient `PATH` decisions across the boundary.
- Authorize one immutable, freshly revalidated plan hash per apply.
- Make refusal monotone: additional privileged evidence may block an operation
  but must not silently weaken a client-visible refusal.
- Keep inventory and planning unprivileged wherever possible.
- Plan packaging/signing, update integrity, diagnostic redaction, and crash
  handling before the first public mutation build.
- Continue fuzzing parsers and canonical encoders; use mutation testing
  selectively on safety and policy gates where surviving mutants would matter.

## Concrete definition of success for the next phase

The next phase is successful when all of these are true:

1. PR #65 has no unexplained evidence loss and the handoff states the criterion
   accurately.
2. WP-010's destructive identity and protection questions have measured,
   normative answers.
3. A real Windows, Linux, and macOS discovery sample normalizes into the same
   canonical fixture vocabulary.
4. The CLI emits stable, redacted JSON for a read-only topology snapshot.
5. The desktop shell renders that snapshot with capability explanations and
   passes keyboard, screen-reader, zoom, contrast, and reduced-motion checks.
6. Unsupported or ambiguous storage is visible and safe by default.
7. Tier 2 still refuses everywhere unless the disposable-target lab and all
   interlock predicates are proved on that platform.

Do not judge this phase by how many filesystem operations are listed. Judge it
by whether a user can inspect a real machine on all three operating systems and
trust both what PartMan says and what it refuses to claim.

## Handoff to the next agent

Start with P0, but keep it bounded. The best next product move is not another
general repository framework and not an empty UI shell. It is the smallest
measured WP-010 decision that unlocks a real, read-only topology slice.

When updating progress documentation:

- distinguish merged, open, experimental, and proposed work;
- cite the command or test that proves each “delivered” claim;
- state residuals beside the claim they narrow;
- never use issue closure as proof when executable enforcement is weaker;
- record historical audits as snapshots rather than current authority; and
- prefer “zero unexplained evidence loss” over table-size or test-count proxies.
