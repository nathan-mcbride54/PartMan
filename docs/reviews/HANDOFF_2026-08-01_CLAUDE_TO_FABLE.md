# Handoff — 2026-08-01, Claude (Opus 5) to Fable 5

Written to resume from, not to summarise. Sits alongside
`HANDOFF_2026-07-31_CODEX_LEAD.md`, which remains accurate for everything it
covers and is **not** superseded — read it first for the Slint arc and the
worktree inventory.

**Commit status of this file is Nate's call.** Its predecessor is deliberately
uncommitted at his request; this one follows that precedent until told
otherwise.

---

## 0. How to read the claims in this document

This project's characteristic failure is a document that states something more
strongly than its evidence supports. So every substantive claim below is tagged:

- **[measured]** — I ran it and saw the output, in this session.
- **[verified]** — I read the authoritative file or code and confirmed it.
- **[relayed]** — produced by a subagent and *not* independently re-checked by
  me. Treat as a strong lead, not as established. Re-verify before acting.

Three claims I made confidently this session turned out to be **wrong**, and all
three were caught by measuring rather than reviewing. They are listed in §9.
Assume this document contains a fourth.

---

## 1. Live state

- `origin/main` at **`62c92b6`** [measured].
- The working checkout `D:\PartMan` is on branch
  **`work/wp-000-adr-svelte-ui`** at `8e13e5d` — one commit ahead of main,
  **not pushed**, holding the draft ADR-0010. [measured]
- Only untracked file: `docs/reviews/HANDOFF_2026-07-31_CODEX_LEAD.md`
  (deliberate), plus `.claude/` (user-owned). [measured]
- Open issues: **#35** only (scheduled CI runs / split workflows). [measured]
- Open PRs: none. [measured]

### Landed this session

| PR | What | Result |
|---|---|---|
| #92 | WP-030: README said a completed retirement was still pending | merged, 12/12 |
| #85 | Tauri comparison baseline | **closed without merge**, body rewritten to historical wording, branch retained |

Earlier in the same session arc (already on main before this handoff's window):
#60 (closes #51, Windows containment), #62/#65 (issue #39, generated
traceability), #63 (closes #61), #64 (governance), #66.

### Not pushed, awaiting a decision

**`work/wp-000-adr-svelte-ui` → `docs/adr/0010-svelte-instead-of-react-for-the-desktop-ui.md`.**
Status `Proposed`. Both gates pass [measured:
`verify-change-ownership` 1 path WP-000; `cargo xtask ci` green]. Held back
because the restructuring below may change how it is framed.

---

## 2. ADR-0010 — Svelte instead of React

**Decision proposed:** §4.1's UI line becomes "Svelte and TypeScript". Vite as
the build tool. **SvelteKit explicitly rejected.**

**Why now:** nothing on main is React-specific [verified] — the React code only
existed on the branch #85 closed; `packages/canonical` is framework-independent
TypeScript; design tokens are data. The cost is one line of spec today and rises
the moment anyone writes a component.

**Why React is not the safe default:** ADR-0009 put Slint through 41 gates and
it failed two. **Tauri with React has been through none of them** [verified
against `docs/quality/slint-feasibility.md`]. Keeping React is an un-exercised
default, not a survived one.

**Why not SvelteKit** — the substantive judgement in the ADR. Its idioms are
server-side (load functions, server routes, form actions) in a product that must
run fully offline (SEC-007) and keep network I/O out of the privileged path
(SAFE-008). The hazard is not dependency count; it is that every example the
next contributor reads uses a pattern the product forbids.

**Two things the ADR refuses to claim**, and which must not be quietly dropped
if it is edited:

1. It does **not** approve a desktop shell. Tauri never earned the gates Slint
   was held to. PR #91's retirement stands.
2. It closes **none** of the ten inconclusive `G-AX-*` accessibility gates. A
   compiler warning catches static markup, not keyboard flow or screen-reader
   behaviour.

**Verification is deferred deliberately.** Supply-chain evidence has a shelf
life — ADR-0009's findings are pinned to exact versions on a stated date.
Auditing Svelte's graph today says nothing about the graph built when a shell is
authorized. Until that evidence exists this is an *intended* stack, not a
validated one. The obligations are enumerated in the ADR's Verification section.

---

## 3. Why Slint failed — the short version

[verified against `docs/quality/slint-feasibility.md`]

**It failed on supply chain, and only on supply chain.** The prototype worked:
`cargo xtask ci`, `test --tier 1`, `desktop` (31 tests + native release build
across three renderer configurations), `slint-controls`, and
`verify-change-ownership` all exited 0 at checkpoint `359e331`.

`cargo xtask supply-chain` exited 1. That is the whole rejection.

The report reads **1 pass, 2 fail, 38 inconclusive** — and the two failures are
one failure counted twice, because `G-CFG-08` *includes* "passes the existing
supply-chain policy checks" as a sub-condition of itself. Five findings:

| Finding | Why fatal |
|---|---|
| `RUSTSEC-2026-0206` `rustybuzz` 0.20.1 unmaintained | no safe upgrade; arrives through Slint's **required** text/SVG closure |
| `RUSTSEC-2026-0192` `ttf-parser` 0.25.1 unmaintained | same |
| `clipboard-win` 5.4.1 — BSL-1.0 | would require widening the licence allow-list |
| `error-code` 3.3.2 — BSL-1.0 | same |
| `i-slint-renderer-skia` 1.17.1 | Skia forbidden in shipping graphs |

The unmaintained pair is not an optional feature — upstream
[slint-ui/slint#8805](https://github.com/slint-ui/slint/issues/8805), asking to
make SVG optional, was open at the time.

**The decisive line in the record:** *"no advisory ignore, global licence
allowance, Skia exception, or warning downgrade was added."* Every finding had a
one-line workaround. None was taken.

Two qualifiers that must travel with any retelling: **only Windows was run**
(the record says so and calls a single required-platform failure decisive), and
**38 gates are inconclusive, not passed** — including all ten `G-AX-*`. Binary
size (11.06 MB vs Tauri's 7.75 MB, 1.43×) is explicitly recorded non-decisive.

This is *not* evidence that Slint is inaccessible or that Tauri is better.

---

## 4. The restructuring — recommendation

Nate asked for three things: CLI-first sequencing, "remap the gates to not
artificially constrain development where it's not needed", and replacing
`WP-XXX` with clearer project stages.

Produced by a 7-agent workflow (3 independent designs, 3 adversarial lenses, 1
synthesis). Everything in §4–§7 is **[relayed]** unless tagged otherwise.

### 4.1 Rename the presentation, keep the identifiers

**Recommendation: do not rename `WP-0NN`. Add a Stage column and a
`- Stage:` header bullet.**

The reasoning, which I find sound: the confusion is not caused by the
identifier. `docs/work-packages/WP-030.md` opens with spec version, objective,
requirement IDs, prerequisites, test tier — and **never names the milestone it
belongs to** [verified]. `README.md`'s table is flat and keyed on an opaque
three-digit number. A reader meets a number and no stage.

§13 already carries the stage names verbatim, ordered and complete [verified].

The full rename costs eight PRs, a dual-accept window in the ownership gate, two
new gate states born under migration pressure, a partitioned zero-loss ledger,
and ~321 orphaned references across 18 dated review documents — to buy
legibility a table column already delivers.

It also has a **four-way deadlock** the proposing agent only half-solved
[relayed, worth verifying]: renaming a package changes the generated
traceability document's *path*, its *H1 body text*, **and** its *table cells*,
because evidence rows declare `- path: docs/work-packages/WP-010.md` and
`validate_declared_evidence` requires that path tracked and owned by the
declaring package.

### 4.2 The stage scheme

| Stage | Name | Covers | Milestone |
|---|---|---|---|
| S0 | Foundations | WP-000, WP-010 inc. 1/2/2a/4, WP-020, tokens+a11y half of WP-030 | M0 |
| **S1** | **Evidence** | **new** — register measurements + read-only CLI chassis | **M0.5 (new)** |
| S2 | Domain model | WP-010 increment 3 | M0.5 |
| S3 | Read-only inspection | WP-050, WP-080 inventory half, WP-S100 | M1 |
| S4 | Planning and dry run | WP-060, WP-070, WP-080 plan half | M2 |
| S5 | First safe writes | WP-040, platform 110s, WP-085 | M3 |
| S6 | Full storage operations | platform 120s, WP-I100, WP-R100, WP-D100 | M4 |
| S7 | Ship | packaging, WP-DOC100, WP-Q100 | M5 |
| S8 | Desktop shell | WP-030 shell half, WP-090, WP-095 | deferred |

S3–S7 names are verbatim from §13's Theme column, minus parentheticals —
deliberately. A stage named "Honest read-only inspection" would assert the exact
property SI-28 and SI-35 are open on. **Nothing is named "bulletproof."**

S1/S2 between M0 and M1 is a §13 change (new milestone band) needing a version
bump and a §0.3 changelog row. **Filed, not adopted.**

### 4.3 The git trailer

`Work-Package: WP-0NN` survives verbatim. **No `Stage:` trailer.** A recorded
stage can disagree with the WP→stage mapping, and two registers drifting apart
is this repository's characteristic failure — the 2026-07-28 "five remain"
summary above seven names is the precedent. Derive the stage from the identifier
through one mapping file; never record it twice.

**One prose correction while `AGENTS.md` is open:** it says `WP-0NN`, but §14
contains `WP-W100`, `WP-DOC100`, `WP-S100`, `WP-Q100`, and the checker accepts
any stem starting `WP-` [verified at `main.rs:1724`]. The prose is narrower than
the code and would refuse a legitimate platform package — which is the next
thing S1 creates. Correct to `WP-<id>`.

### 4.4 Migration and the self-refusal problem

Three PRs, then two filings. Every intermediate state green. **Zero lines of
`tools/xtask/src/main.rs` change.**

**The trap:** a PR creating `apps/cli/**` refuses itself.
`ownership_claims_at(root, base)` reads the catalogue at *main's tip*, where no
package claims those paths, so the files are strays. Widening your own
`owned-paths` in the same PR does not help — that base-revision read is the
hardening that closed the PR #47 hole and **must not be relaxed**.

**The answer is `owned-paths-reserved`.** `OwnershipClaim::authored()` returns
true for `Reserved`, and a reservation counts as ownership the moment it matches
a real file. A plain `owned-paths` claim would turn main red immediately,
because a claim matching no tracked file reads as coverage.

1. **`Governance:`** — reserve `apps/cli/**`, `crates/inventory/**`,
   `crates/platform-*/**`, `docs/stages/**`; add `- Stage:` bullets.
2. **`Work-Package: WP-000`** — README stage table, `AGENTS.md` `WP-<id>` fix.
3. Remaining presentation edits.

**Rejected:** using `Governance:` for cosmetic-only edits (passes the checker
while being a false description of itself), and a single atomic cutover under
`WP-000` after widening WP-000 to claim all assignment documents — it passes
mechanically because Work-Package mode never consults `is_assignment_document`,
while performing an assignment edit under a work-package trailer. The governance
escape hatch running in reverse.

---

## 5. The finding that actually matters

**CLI-first and unblocking the domain model are the same programme.**

Every one of the six direct register blockers ends with an evidence clause
naming measurements nobody has taken: SI-34 requires Windows, real partitioned
Linux hardware and macOS observability established first; SI-35 requires a
loop-device measurement plus the Windows per-partition equivalent; SI-33
requires a media-change-counter liveness experiment on hardware; SI-28 was
itself only confirmed by a hardware measurement.
`docs/quality/observability.md` states its own status: **Windows established,
Linux partly, macOS not at all** [verified].

The instrument that produces that evidence is a read-only, unprivileged,
reproducible, cross-platform probe. **That is a CLI.**

Three design rounds have already failed by asserting an unprivileged projection
nobody had measured. A fourth run from spec text alone fails the same way.

So CLI-first is not a detour around the blocked model. It is the only route
through it.

---

## 6. CLI-first: the precise line

### Needs none of the nine blockers

1. **CLI chassis** — SAFE-004 structured argv; stable exit codes (CLI-005);
   `NO_COLOR` + non-TTY detection with ANSI-free `--json` (CLI-008); JSON Lines
   progress; secrets via protected descriptor, never argv (CLI-006);
   dry-run-by-default (CLI-007); CLI-004 satisfied structurally by there being
   no mutation argument surface to bolt `--force` onto.
2. **Deny-by-default redaction** (SEC-006, SAFE-006) as an allowlist — an
   allowlist need not know what the denied fields are, so it is model-independent.
3. **Adapter-attributed observation records** (MODEL-004) — what each interface
   returned, tagged with source adapter, version, method, outcome. SI-04 is
   **Resolved** by ADR-C4, so the one distinction that matters (positively
   observed absence is a value; unavailability is not) is already decided.
4. **Dependency doctor** (CAP-004) — tool present, version, in/out of tested
   range, resolved from a trusted absolute path.
5. **Technology-level capability facts** (FS-007) — "XFS does not shrink" is a
   property of a technology, not a verdict about a target.
6. **Redacted `export-diagnostics`** (CLI-002, INV-007).
7. **Fixture-backed replay** over WP-020's 13 deterministic images — Tier 1, no
   device access.
8. **The register's own outstanding measurements** (§5).

### Blocked, with the gate named

- **Any `IdentityStrength`** — SI-28, SI-35, SI-12. The Strong predicate is a
  conjunction and two conjuncts are open.
  **The line: recording what a device reported is not classifying it, and
  classifying is what SI-28 is about.** Print raw identifier strings each
  labelled with the interface that produced it; never print a strength.
- **Any ADR-C3 partition-table state** — SI-35. Emit no table checksum.
- **Any Section 5 typed node, `TopologySnapshot`, artifact hash, or plan** —
  SI-27, SI-34, SI-35.
- **Any stable device handle** — SI-27, blocked by SI-12. The selector must be a
  session-local index or the platform's own path, documented as not stable
  across runs. This is a real product cost and it is the cost of not guessing.
- **Any claim that two paths are or are not the same device** — SI-12.
- **Any protection verdict** for ZFS / Storage Spaces / LDM / Fusion / Apple
  sealed system — SI-11 (inputs SI-29, SI-30).
- **Any per-target capability verdict.**
- **`plan`, `validate`, `dry-run`, `apply-plan`, `status`, `resume`, `cancel`** —
  entirely. There is no honest partial `plan`.
- **CLI-001 "stable versioned JSON"** — stable means schema-versioned means
  MODEL-003, which is S2. Ship `--json` marked unstable and **version-refusing**:
  it declines to emit a schema version rather than emitting a provisional one.

### The smallest genuinely useful slice

**`partman inspect`** — cross-platform, unprivileged, read-only, offline. Per
device: which identifiers each OS interface reported and which interface
reported each; whether raw table sectors were readable at this privilege and
what was denied; the host dependency report; a redacted bundle for bug reports.

It answers the two questions a user has before repartitioning — *what does this
machine expose*, and *will PartMan work here* — and no mainstream tool prints
per-field provenance, because they all synthesize.

**Every refusal is a typed value carrying the register issue that gates it** —
`{"strength": {"state": "not-established", "gate": "SI-28"}}` — never an exit
code, never a stderr string, never a silent omission. The refusal list is the
first real feature.

**Two structural constraints, not documented ones:** no hash function reachable
from the inspector's output type; no code path from inspector output to a plan.

---

## 7. Gate remap

### Never relax

| Gate | Reason |
|---|---|
| SAFE-001…009 (§3) | §0.2 puts §3 above every other section **and above instructions to an agent**. "CLI-first" is not an argument that reaches them. |
| SAFE-007 interlock; T2/T3 refuse | Keep it a refusal, never a skip. A green destructive tier is the exact signal someone would trust. |
| `unsafe_code = "deny"` + `[lints] workspace = true` on every new member | `apps/cli/`, `crates/inventory/` and adapters are all new members. A member omitting the lints line inherits nothing and compiles clean. |
| INV-006, SAFE-004 from the inspector's first line | No auto-mount to inspect, no repair tools during discovery, structured argv, trusted absolute paths, bounded output, timeouts. §16 also forbids parsing human-localized output where structured output exists. |
| §11.4 fuzz targets **before** the parser exists | `fuzz/fuzz_targets/` holds two targets, both `pce/1`. All three proposals scheduled parser fuzzing *after* the device-reading inspector. Only safe while the inspector parses nothing. SI-35's measurement points at raw sectors. |
| `verify-change-ownership` base-revision read; `verify-ownership` | Caught PR #47 and WP-030 increment 1. CLI-first adds directories, which is when path discipline matters most. |
| Governance may edit only assignment documents; `--no-renames` | Without `--no-renames` a governance change could delete any file by renaming it into the assignment directory. |
| §11.7 generated traceability; `not established` as a printed state | The only mechanism enforcing that a package claiming a requirement without evidence fails. A CLI is the first thing that can print something plausible and look like coverage. |
| **`cargo xtask tokens` stays blocking** | `design-tokens.json` carries `severity.destructive` with a redundant **non-colour channel** rule — exactly what CLI-008's `NO_COLOR`/`--json` path depends on. Deferring it defers a check the CLI's own output relies on. |
| Supply chain: `verify-actions`, `cargo deny`, `verify-toolchain`, `verify-licenses` | All green, all free, none constrains a CLI. |
| **`cargo xtask cross-language` parity** | Tempting to delete since its consumer is a deferred GUI. Its value is that it was established *before* either side had reason to drift. Record the deferral as load-bearing, not residue. |
| §15 ADR-before-package for anything hash-visible | Split on hash-visibility, not convenience. |
| The nine register blockers | Not gates. Unanswered questions about what canonical bytes mean. Four hash-visible. |
| SI-28's interim conservative floor | A read-only CLI satisfies it **vacuously**. Vacuous satisfaction is not evidence it can be narrowed. Say so in the CLI's own docs or it gets recorded as closed at the next review. |
| §1.11 stop-and-file | The gate most likely to be quietly remapped under a "faster to a usable product" mandate, and the only one whose relaxation is unrecoverable. |

### Defer, but keep recorded

- All ten `G-AX-01..10` rows, UI-002, the rendered half of UI-008. Deferring a
  GUI does not make its accessibility evidence not-applicable; it makes it
  **deferred-and-still-unmet**. Dated and attributed, never deleted.
- §12 item 5 / CLI-003 GUI↔CLI agreement — vacuously satisfiable with one front
  end; record as not-yet-testable rather than met.

### Genuinely artificial for CLI work

- §14's ordering that places UI work parallel to or ahead of CLI work.
- M0's "Accessibility harness runs" as a *blocker on CLI progress* — the harness
  runs; the rendered half is what is unmet, and that belongs to S8.

---

## 8. Three things to file before any of this lands

**Two I verified myself.**

1. **[verified] The tier-boundary sentences go false the day `inspect` ships.**
   `docs/quality/test-tiers.md:48` — "No test opens a block device at any tier"
   — and `:117` — "No command in this repository enumerates, opens, or writes a
   block device, at any tier." That file already records that a boundary
   description lagging the code is worse than none, because it is the sentence a
   reader relies on to decide a tier is safe.
   **Narrow it before the first device-reading commit**, to the claim that
   survives and carries the weight: *no code in this repository opens a block
   device with write intent, at any tier* — enforceable by an open-flags
   assertion plus a test. State explicitly that **SAFE-007 provides zero
   coverage for the read path**, because a read-only inspector never calls
   `authorize`. Defensible as a stated decision; indefensible as a claim that
   silently expires.

2. **[verified] CAP-003 has no value meaning "the product has not decided."**
   `AGENT_BUILD_SPEC.md:468` — `supported` / `preview` / `unsupported` /
   `blocked`, with `unsupported` defined as "the product does not implement the
   operation for this target." Mapping a register-blocked target onto it
   converts an open decision into a product verdict. **File the missing fifth
   state under §1.11**; do not let whoever writes the `capabilities` command
   pick one.

3. **[relayed — verify before acting] Loop-device indirection in the
   interlock.** `crates/fixtures/src/interlock.rs` authorizes a *regular file*:
   canonicalized inside the fixture root, matched against the compiled catalogue
   by name and digest, returned as a held handle. The SI-35 loop-device
   measurement runs `blkid` against `/dev/loopN`, and **nothing binds
   `/dev/loopN` to the handle that was verified.** The 2026-07-29/30 hardening
   closed path-rebinding by holding the descriptor; the loop association is a
   second indirection the descriptor does not cover. Read-only today, so the
   blast radius is a wrong measurement — but WP-020 increment 2 turns the same
   mechanism into a destructive T2 suite. **Close it before a destructive suite
   is registered, not after.**

---

## 9. Three things I got wrong this session

Recorded because the pattern matters more than the instances: **all three were
found by measuring, and all three looked correct when read.**

1. **"`Work-Package:` is on every non-merge commit on main, so history can't be
   rewritten."** False. [measured] 117 non-merge commits on main, **36** carry
   `Work-Package:`, **18** carry `Governance:`, **63 carry neither** — and main
   is green. `read_declarations` asks git only for `{base}..HEAD`
   [verified, `main.rs:2126`], so the gate is forward-only and never re-parses
   history. I used this to argue a rename was near-impossible. It is not; it is
   merely not worth it, which is a different argument.
2. **"Generation converted the traceability cleanly."** It did not — 33
   hand-written rows became 13. Caught by counting rows before merging, not by
   reading the diff, which looked like a clean conversion. Nate held the PR
   until the gap closed; the final result carries **56** rows.
3. **`git add -A` on a branch with foreign untracked files.** Swept the Codex
   handoff and 10k build artifacts into a commit. Caught by
   `verify-change-ownership`, amended to README-only. **Stage explicit paths.**

Earlier in the same arc: a test named
`renaming_an_annotated_test_is_invisible_because_the_binding_is_positional`
documented a limit that a later change removed; three tests passed with their
own fix removed until the mutations were run; and a parser was defeated by the
document that explained its own syntax.

**The lesson to carry: run the mutation, count the rows, measure the claim.**

---

## 10. What must survive any restructuring

[relayed, but each is checkable in one command]

**Records that must not be merged, paraphrased, or converted to generated output:**

- `docs/work-packages/WP-000.md` §"Verification and known gaps", including the
  recorded fact that the first generated traceability **lost evidence**.
- `docs/work-packages/WP-030.md` §"What this does not establish" and §"Evidence
  discipline", including the deletion-sweep table where each row is marked
  **FAILED** — a gate proven capable of failing.
- `docs/quality/accessibility.md` §"ADR-0009 accessibility matrix" — all ten
  `G-AX-01..10` rows with their "Evidence still required" column, none omitted
  or rounded up.
- The four `docs/reviews/WP_0NN_TRACEABILITY_MIGRATION_*.md` ledgers.
- `docs/spec-issues/README.md` — its table as the **sole** authority on counts,
  and its lines 8–9: *"None of them proposes an answer as though it were
  decided."*
- `docs/quality/observability.md` — "Windows established. Linux partly
  established… macOS not established" — and its rule that an entry marked
  `not established` MUST NOT be relied on.

**Sentences that must survive verbatim:**

- `README.md` — **"Not a usable partition manager, and must not be represented
  as one."**
- "…recorded rather than rounded up: a milestone that exits on a criterion
  nobody verified is worse than one that exits late."
- "…reporting a pass for a run of nothing would be a fake success path."
- WP-020's "**unproven for roots that are not on a local volume**", and that the
  other-name refusal was "a **live defect**, not a missing check."
- WP-030's "no shell exists, UI-002 remains unimplemented, and the rendered half
  of UI-008 remains untested", and "reviving either off-main branch needs fresh
  governance rather than inertia."
- Every generated traceability document's closing "**Not established here:**"
  block.
- `main.rs` — "A new work package cannot be born hand-maintained by omission."

The recommended migration touches none of the above. That is the strongest
argument for it.

---

## 11. Decisions waiting on Nate

1. **ADR-0010** — push and open, or revise first? It is committed locally,
   unpushed, both gates green.
2. **Stage scheme** — accept "rename the presentation, keep identifiers"? This
   is the recommendation and it is cheap and reversible.
3. **S1 Evidence / M0.5** — adding a milestone band is a §13 spec change with a
   version bump. Filed, not adopted.
4. **Order of the three filings in §8.** My advice: #1 first, because it has a
   real deadline — it must land *before* the first device-reading commit.
5. **Issue #35** (scheduled CI / split workflows) is still open and untouched.

---

## 12. Environment

Everything CI runs is reproducible locally except macOS.

| Gate | Where |
|---|---|
| `cargo xtask ci` | Windows, and WSL Debian (pinned 1.96.0) |
| `cargo xtask cross-language` | Windows — Node 24.18 |
| `cargo xtask supply-chain` | Windows — cargo-deny 0.19.4, cargo-audit 0.22.2 |
| `cargo xtask probe` | WSL — util-linux 2.41, the version `prober.rs` records |
| `cargo xtask fuzz` | WSL — nightly-2026-07-01, cargo-fuzz 0.13.2, needs `g++` |

- WSL builds use `CARGO_TARGET_DIR=/tmp/partman-linux-target` (in `~/.bashrc`).
  The Linux build earns its keep — it caught a `-D warnings` clippy failure the
  Windows build had no opinion about.
- **Do not move the working copy onto the WSL filesystem.** `\\wsl.localhost\…`
  is the 9p path where the SAFE-007 containment break was reproduced, and the
  interlock now refuses roots there.
- `C:` is NTFS, `D:` is **ReFS**, and the repository is on `D:`. `%TEMP%` is
  therefore a different filesystem from the fixture root. Point the suite at the
  real volume with `PARTMAN_TEST_ROOT`, using **backslashes** — `mklink` is
  parsed by `cmd`, which reads a leading `/` as a switch.
- Worktrees: `D:\PartMan` plus several historical ones under
  `~/.codex/visualizations/...` and `%TEMP%`. I removed a stale clean `main`
  worktree in `%TEMP%` so `D:\PartMan` could hold `main`. Do not delete the
  others casually.
