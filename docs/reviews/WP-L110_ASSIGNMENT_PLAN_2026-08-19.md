# WP-L110 — the arc plan, written before the assignment

**Session:** Nate — "Draft the WP-L110 assignment plan", after the Linux
transport landed (WP-040 increment 5 on ADR-0055, spec 19.0.0) and the
floor finished (ADR-0054, spec 18.0.0). **Base:** `670a3ca` (main), spec
19.0.0, nothing in flight, WP-020 pinned at `80f4f93`.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §7 is for the decision owner, and
> the assignment itself is a Governance PR that follows this plan.

## 0. Why this package, and what it unblocks

Every Linux layer below the helper is delivered and has no privileged
consumer. WP-L100's adapter emits the client's **proposal** snapshot and
the engine's runtime facts; WP-050's engine judges; WP-060's planner
plans over a snapshot and refuses on the engine's word; WP-070's journal
admits one apply per floor act and its state machine owns the thirteen
states; WP-040's protocol is complete and, since increment 5, **reachable
on Linux** — a root-created `0711` directory, a `0600` node owned by the
authorizing user, the peer verified before any byte is read. The Section
14 row — `WP-L110 | Linux helper, GPT/MBR, file systems, polkit | WP-040,
WP-060, WP-070, WP-L100 | M3` — has every prerequisite met as far as a
helper can be assigned against it.

What queues behind it, by name: **WP-L100 3b / gitea#1003** (the solver
reserves nothing on a Linux client draft until HLP-002 re-discovery
supplies a table node — the helper is that re-discovery); **LIN-001's
authorization/mutation half** (ADR-0054 reserved UDisks2, libblockdev or
native tools for the helper's own route decision); **the UDisks2 tool
floor** (entered by the first package that invokes it); **HLP-003's floor
act** naming the RPC-001-authenticated user the transport now verifies;
**the held report's consumer** (increment 4b's third slice: consumed
versus released is decided in the helper's capture, reading (b)); and
WP-080's apply/resume/status/cancel surface, which needs "≥1 live
helper".

## 1. What the search established, and what changed because of it

Six findings, each cheaper to find now than mid-increment.

1. **The helper is an integrator, not a second implementation of anything
   below it.** What it calls is already delivered and typed:
   `partman_transport_linux::linux::Endpoint` (admission and the
   handshake), `partman_rpc` (envelopes, streams, redaction boundary,
   identity claims), `partman_adapter_linux` (the client draft, `held`,
   `floor`, `runtime` — the same library runs as root and reads the same
   files), `partman_table_parser::classify(head, tail, geometry)` (ADR-0018's
   byte layer: "the helper's own bounded parsers over raw device bytes"),
   `partman_capability` (the engine; `QualificationEvidence` still has no
   constructor), `partman_planner::plan(request, snapshot, limits,
   runtime, identity)`, `partman_journal::lifecycle::admit_apply` (one act,
   one apply — ADR-0028's obligation 7 as a pure function over the bytes)
   and `partman_statemachine`. The helper's own code is the glue, the
   device I/O, and the two things no library below it may author: the
   table state (ADR-0014) and the protection verdict (ADR-0016).
2. **HLP-002 is the first thing in the product that opens a block device,
   and it needs no `unsafe`.** Re-discovery is: the adapter's draft run as
   root (same contract, same files), plus `classify` over head and tail
   windows read from each whole device with `std::fs::File` opened
   read-only — SAFE-002's context 1, SAFE-009's "helper crate" permission
   unused. The byte layer's reach is the parser's (GPT/MBR; the windows it
   asks for); nothing else is read from a device before the first write.
   That authored table state is exactly the node 3b waits for, so **3b's
   block dissolves in this package's increment 2**, not in WP-L100.
3. **Three route decisions are this package's, each increment-gated,
   none decided by drift.** (a) **Launch and endpoint ownership** — who
   creates `/run/partman/` (`0711` root) and the per-user `0600` node, and
   how the helper comes to exist: a systemd unit with socket activation
   (ADR-0055 T5), `pkexec` on demand (T6, which rides a pair and
   complicates RPC-006 reattach), or polkit-mediated start under LIN-009;
   HLP-005's "MAY exit when idle" and HLP-007's "non-local or cross-session
   callers" both bear on it. (b) **The mutation toolset** — LIN-001's
   second half since ADR-0054: UDisks2 over D-Bus, libblockdev, or
   authoritative native tools through a SAFE-004 launcher (structured
   argv, fixed absolute allow-list, verified identity and version, bounded
   output, timeout, sanitized environment); the UDisks2 ≥ 2.9 tool floor
   enters the CAP-006 store with whichever package first invokes it.
   (c) **The launcher's home** — WP-035's `ToolLauncher`/`SystemLauncher`
   live in `apps/cli/src/doctor.rs`; a helper cannot depend on an app, and
   WP-L100 decided "no launcher in the adapter", so SAFE-004's one
   reviewed launcher needs a crate or the helper owns its own — a
   governance question the assignment records rather than answers.
4. **The authorization ladder is decided; its Linux mechanism is named
   and unmeasured.** ADR-0021 (spec 11.2.0): a floor act for every apply
   at every severity, by the RPC-001-authenticated user, naming the plan
   hash, single-use, journaled, never cached; the interactive ceremony at
   severity ≥ Disruptive or any step flag — "Linux — polkit `auth_admin`
   without retained grants". The tier is the helper's own computation
   from its recomputed severity and flags (HLP-002), carried as
   validate-plan response data; no message carries an authorization
   requirement (CAP-007). **No row in `docs/quality/observability.md`
   measures polkit** — whether `polkitd`/`pkexec` are present by default
   on the three tier images (the jammy, Debian 12 and Arch cloud images the
   DR apparatus already pins), which agent answers, whether
   `auth_admin` can be exercised headless — so the ceremony's increment is
   gated on a DR row the same way the floor's was.
5. **SI-13 bites this package by name, and has a structural interim.**
   "Identity binding for pool and array write targets … Later (WP-L110)":
   whether an mdraid grow binds the union of member identities, the pool
   UUID, or both. Until decided, validate-plan refuses any plan whose
   target is an `Aggregate` — the conservative answer, structural (no
   binding constructor for aggregates), stated in the assignment as the
   gate on the increment that first validates, not discovered there.
   SI-28 (Mitigated-open) and SI-37 (multipath detection-only) are
   inherited unchanged.
6. **The spec's layout names the path.** Section 4.3 reserves
   `services/helper-linux/` for this component; no `services/` tree exists
   yet, and `packaging/debian`, `packaging/arch` (LIN-008, signed packages)
   are separate paths this package does not claim. The spec also says
   "canonical crates MUST NOT depend on platform adapters" — the helper
   depends on the adapter, which is the permitted direction.

## 2. Imported obligations the creation cannot omit

| Source | Obligation | Increment |
| --- | --- | --- |
| HLP-001 | The closed operation set — status/enumeration, validate-plan, apply-plan by hash, cancel, resume, journal queries — and nothing else, held structurally (no path, no command, no dynamic code: RPC-005/CLI-004) | 1 |
| HLP-007, RPC-001, ADR-0055 | Caller identity verified by the transport before any byte; non-local or cross-session callers refused (SEC-002) | 1 |
| HLP-005 | One plan per bound device set (CONC-001); locked-down idle; may exit when idle | 1, 4 |
| HLP-006, SAFE-006, SEC-009 | Structured, redacted logging appended to a local audit log with retention | 1 |
| HLP-002, ADR-0014, ADR-0016, ADR-0018 | Independent re-discovery before the first write: the adapter's contract as root plus `classify` over device bytes; the table state and the protection verdict helper-authored; client output an untrusted hint | 2 |
| HLP-004, PLAN-006, PLAN-007, SEC-002 | Validity windows and snapshot-hash freshness enforced; replayed, expired, altered, cross-user, cross-device plans rejected | 2 |
| ADR-0013, ADR-0054 | The helper's own reach declaration for its privileged contract; discovery through the client-readable interfaces plus the byte layer — UDisks2 is not the discovery interface | 2 |
| SI-13 | Aggregate write targets refused structurally until decided | 2 |
| ADR-0021, HLP-003, LIN-009 | The floor act per apply (journal-admitted, ADR-0028's one-act-one-apply); the interactive ceremony via polkit `auth_admin` without retained grants at ≥ Disruptive or any flag; the tier helper-computed and reported in validate-plan | 3 |
| ADR-0028, JRN-001…006, Section 8 | The apply lifecycle through `crates/journal` and `crates/statemachine`; append-only journal; torn-tail recovery; resume and cancel (RPC-006 on the protocol side) | 4 |
| CONC-002…005 | Queued plans revalidated; external changes invalidate; discovery during execution transitional; two racing applies — one wins, one explained rejection | 4 |
| SAFE-004, SAFE-005, CAP-004/006 | Every tool through the launcher's allow-list with verified identity and version; tool floors in the CAP-006 store; absence fails closed | 4 (behind the toolset route) |
| ADR-0011, SI-37 | Multipath devices and recognized members: mutating capability `unsupported`; no cross-path sameness | 2, 4 |
| ADR-0053 / the held report | The consumed-member arm decided in the helper's capture (reading (b)) | 2 |
| LIN-002 | GPT/MBR and the listed file systems "according to installed capabilities" — capability-gated, never assumed | 4 |

## 3. Register gates recorded at creation

- **SI-13 (Later, before WP-L110's validate-plan surface)** — the
  structural refusal of aggregate write targets in increment 2; the
  decision itself is a round this package files before any aggregate
  operation is planned.
- **SI-28 (Mitigated-open)** — strength stays the adapter's `Weak` for
  client records; the helper's re-discovery may establish `Strong` only
  by SAFE-003's own terms (a positively determined table state, which
  increment 2 is the first to author) — the assignment says this is
  where SI-28's floor is first *met*, not relaxed.
- **SI-37 (Open, Later)** — unchanged.

## 4. The route decisions this package will face

All three (§1.3) are claimed increment-gated. The launch/endpoint
decision precedes increment 1 (the helper cannot be started without it);
the toolset and launcher-home decisions precede increment 4 (nothing
before it invokes a tool). Each is a round in `docs/reviews/` with the
routes costed against the texts, an adversarial pass, a Tier-1 posture and
the Tier-2 apparatus, then the decision owner's choice recorded by ADR
where a spec text moves and by the WP record where none does.

## 5. Sequence

1. **`Governance:` PR creating `docs/work-packages/WP-L110.md`** — only
   that file, reserving `services/helper-linux/**` (Section 4.3's path)
   and `schemas/helper/**` (the operation set's message formats under the
   `schemas/rpc` precedent), with `docs/traceability/WP-L110.md`, the
   `Cargo.toml` member share, README row, CHANGELOG share. Born
   `hand-maintained`; no Delivery status section until increment 1.
2. **The launch and endpoint round** (§1.3a), with the polkit presence row
   filed on WP-035 (DR20: `polkitd`/`pkexec` presence, version and agent
   on the three pinned images; the `/run` tmpfiles conventions each tier
   ships) — measured on the existing apparatus before the round closes.
3. **Increment 1 — the helper process and its closed surface.** A Linux
   binary under `services/helper-linux` over the transport's endpoint:
   HLP-001's six operations as `schemas/helper/` message types carried in
   `partman_rpc` envelopes, every other request a typed refusal; HLP-007
   by the transport; HLP-006's log; HLP-005's idle posture; status and
   enumeration answered from the adapter's contract run as root (the
   proposal, labelled as such). Tier-1: the operation set closed by
   construction and the refusals typed, over an in-process pair; Tier-2: a
   disposable guest, the r-series shape, the real endpoint as root with a
   client user. No device opened yet.
4. **Increment 2 — HLP-002 re-discovery and validate-plan.** The byte
   layer over read-only device handles; the helper-authored table state
   and protection verdict; the authoritative snapshot and its hash;
   `plan()` over it with the engine's facts; PLAN-006/007 and SEC-002
   refusals; the helper's own reach declaration; SI-13's structural
   refusal; 3b's table node handed to WP-L100. Tier-2 over WP-020's
   fixture images in a disposable guest.
5. **Increment 3 — the ladder.** The floor act consumed through
   `admit_apply`; the computed tier in the validate-plan response; the
   polkit `auth_admin` ceremony at ≥ Disruptive or any flag, behind the
   DR20 row and the launch round; audit-logged; nothing cached.
6. **The toolset and launcher-home rounds** (§1.3b–c).
7. **Increment 4 — apply.** The state machine driven over the journal;
   CONC-001 device locking; GPT/MBR table writes (the first writer in the
   product — reviewed, fuzzed where it parses, proven on WP-020's fixture
   images in disposable environments only, SAFE-001/007); file-system
   operations through the launcher per installed capability (LIN-002);
   cancel and resume; the first entries in `docs/capabilities/` for the
   tools invoked. Tier-2 destructive suites in the r-series shape, never
   on a workstation disk.
8. **Increment 5 — the record.** Reach, capabilities, README, CHANGELOG,
   generated traceability, the package's Tier-2 transcripts.

Every Rust increment owes its WP-020 sitting; every row the helper's
claims rest on is a WP-035 two-act filing; every spec text that moves is
an ADR with `spec-change`.

## 6. Evidence-sourcing rule the assignment sets

The WP-L100 rule, extended to the privileged layer: structural properties
(closure of the operation set, refusal arms, one-act-one-apply, bounds)
may be tested over authored inputs and in-process pairs; every
**representational** claim about a real Linux host — what polkit answers,
what a device's bytes classify to, what a tool prints — rests on a
recorded capture (a DR row or a Tier-2 transcript in a disposable guest),
and where none exists the increment delivers the fail-closed answer and
says so. Nothing in this package writes to a device outside SAFE-001's
disposable environments, and no Tier-1 test opens a device, launches a
process, or needs elevation.

## 7. Open for the decision owner

1. **The path:** `services/helper-linux/` as Section 4.3 proposes (the
   plan's preference — the spec named it, and a helper is a service, not a
   library), or `crates/helper-linux/` beside the other crates?
2. **Scope of LIN-003…007 and LIN-010** (LUKS, LVM, mdraid, dm/multipath
   operations, GRUB, fstab/crypttab verification): the Section 14 row
   names "GPT/MBR, file systems, polkit" — the plan reads the rest as
   later increments or rows *behind their own rounds* (LUKS needs a
   secrets posture; LVM/mdraid wait on SI-13), claimed nowhere yet. Agree,
   or claim them now?
3. **The launcher's home** (§1.3c): a small reviewed crate
   (`crates/launcher`, SAFE-004's one implementation, consumed by the CLI
   and the helper) versus a helper-owned copy — the plan prefers the
   crate, decided in its own round with WP-035 in the room.
4. **Whether increment 1 ships with a systemd unit or nothing** — the
   launch round decides; the plan's lean is T5 (socket activation creates
   the `0711` directory and the per-user nodes from a unit, the helper
   exits when idle) with `pkexec` as the interactive ceremony's vehicle
   rather than the transport's.
