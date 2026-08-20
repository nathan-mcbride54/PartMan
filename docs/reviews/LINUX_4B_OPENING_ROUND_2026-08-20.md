# The increment-4b opening round — the artifact store's home, the CONC-001 mechanism, EXE-001/EXE-003

**Date:** 2026-08-20. **Base:** main at the r58 re-pin (`0eba70d` the
WP-020 pin; spec 20.0.0).
**Directive:** Nate — complete the three items WP-L110's 4b delivery row
names as owed within 4b: the `Governance:` act for PART-013's artifact
store, the CONC-001 mechanism decision, and the EXE-001/EXE-003
decision.
**Question:** the three decisions, each on measured or delivered
substrate, so increment 4b's first code opens against a settled ground.

> Committed session record. `docs/reviews/**` is in WP-000's
> `owned-paths` block and lands in its own `Work-Package: WP-000`
> commit, never bundled with code. Nothing below is decided; §5 is for
> the decision owner.
>
> **Decided 2026-08-20 (Nate): all three as recommended — (a) the store
> to WP-070 as `crates/artifact-store` with ADR-0030's obligations
> imported; (A) CONC-001 as journal-first arbitration with `flock` on
> every bind-set member's handle; EXE-001/EXE-003 in §4's shapes, the
> inhibitor mechanism joined to the ceremony follow-up's bus-vs-binary
> decision.** Increment 4b's opening ground is settled; what stands
> before its `Protecting` is the WP-070 store increment.

## 0. The texts

- **PART-013** (`AGENT_BUILD_SPEC.md:586`, as ADR-0024 shaped it): back
  up primary and secondary table metadata before the first table write,
  each arm journaled — on `Present` the parse-level backup, verified,
  failure → Failed; on fresh positively determined `Absent` the
  journaled determination *is* the record; on `Indeterminate` the typed
  REC-001 repair family's raw capture. Section 8:759: `Protecting →
  Executing` requires "Metadata/encryption backups complete and
  verified (PART-013, REC-011)".
- **ADR-0030** (REC-011's protection artifact): the store's shape is
  **already decided** — a dedicated helper-owned protection-artifact
  store inheriting JRN-004's admin-protected documented-location
  clause, **sibling to and never inside the journal**; hash-only
  references on every surface; the ADR-0029 liveness retention rule;
  the consequence-stated end of life. What it deliberately left open:
  *"no re-attribution follows — neither assignment exists, and the ADR
  records the verification obligations so their creation cannot omit
  them. The store's layout, encoding, and per-OS paths land with
  WP-R100/WP-070, jointly sequenced."* Four verification obligations
  are recorded there for the assignment that takes the store.
- **CONC-001** (`:296`): "At most one plan may execute against a
  physical device at a time. An executing plan locks every device it
  binds for its full execution, **including reboot-resumed phases**."
  **CONC-005** (`:300`): two racing submissions — exactly one wins, the
  loser receives a deterministic, explained rejection. **HLP-005**: at
  most one plan per bound device set. The bind set traverses
  host-backing (`:410`), so a plan on a virtual device binds through to
  the physical device beneath it.
- **EXE-001** (`:735`): sleep and hibernation inhibited during
  Protecting/Executing/Verifying; released after; **"its failure to
  engage is surfaced before apply"** — the requirement's own text
  contemplates an engage-failure that is surfaced rather than silently
  proceeded past. **EXE-003** (`:737`): progress reports step identity
  and byte counts **where meaningful**; ETAs labeled as estimates;
  never backward except on a declared, journaled retry.

## 1. What is delivered or measured

1. **The journal discipline the store inherits is real code**: 4a
   delivered `/var/lib/partman` under JRN-004's clause (root `0700`,
   per-uid `0600` logs), `FileSeam` (append + fsync, poisoned), torn
   tails truncated physically. The store is the *sibling* of exactly
   this.
2. **One helper process per authorizing uid** (increment 1's launch
   rule; JRN-004's per-uid logs). CONC-001 is per *device*, across
   uids — so any mechanism that lives inside one helper's memory
   cannot serialize two users' helpers.
3. **The byte layer is seam-shaped** (`bytes.rs` reads through injected
   handles; the write path's read-write sibling arrives with 4b) — a
   lock that attaches to the opened device handle has a natural home.
4. **The journal already arbitrates one-winner questions**: 4a's
   consume-before-submit ordering and the fresh-lifecycle guard are
   delivered; a non-terminal journaled apply is durable, per-uid.
5. **The event stream exists**: envelope v2's `event` channel carries a
   monotone `sequence` with per-channel presence rules (WP-040
   increment 2) — EXE-003's transport is delivered, unconsumed.
6. **DR22/DR24 measured the bus substrate** (busctl present on all
   tiers, logind answering on the polkit tiers) — the same
   bus-vs-launched-binary route the apply-ceremony round's follow-up
   already owns for `pkcheck` vs `CheckAuthorization`. `systemd-inhibit`
   presence is unmeasured (flagged; it ships with systemd on every
   measured tier, but that is archive knowledge, not a row).
7. **Archive knowledge, flagged:** util-linux tools and systemd-udevd
   take BSD `flock` on whole-disk nodes as a cooperative convention
   (udev skips probing a flocked disk). Not a row; deliberately **not
   load-bearing** in §3's recommendation.

## 2. Decision 1 — the artifact store's home (the Governance act)

The store class serves PART-013's parse-level table backup (4b's
immediate need), ADR-0024's raw captures, and REC-011's encryption
headers (later packages). Its shape and rules are ADR-0030's; only the
owning assignment is open.

**(a) WP-070 takes it — a reserved `crates/artifact-store/**` and
`schemas/artifact-store.md`, the four ADR-0030 obligations imported
into WP-070's assignment (recommended).** *For:* ADR-0030 names
WP-070 (with WP-R100, which still has no assignment); the store
inherits the journal package's own JRN-004/JRN-005 discipline and is
consumed by every platform helper exactly as the journal is — a shared
crate under the package that owns the sibling discipline, with each
helper's per-OS on-disk path landing under that helper's own grant
(4b's is a `/var/lib/partman`-sibling directory). *Cost:* a
`Governance:` PR, then a WP-070 store increment (schema + crate — Rust,
its own sitting) sequenced before 4b's `Protecting`.

**(b) WP-L110-internal** (a helper module, schema under
`schemas/helper/**`): no new paths, fastest for 4b. *Against:* the
store class is cross-platform by ADR-0030's own text; parking it in the
Linux helper re-attributes it later — the exact churn the ADR declined
to create, recreated one package over.

**(c) A module inside `crates/journal`:** zero governance. *Against:*
"sibling to and never inside the journal" is a runtime rule, but
housing bulk-byte store code inside the crate whose ADR-0029 budget
discipline exists to keep bulk bytes *out* invites the confusion the
sibling rule was written against; and the journal crate's dependency
story stays cleanest carrying nothing but records.

## 3. Decision 2 — the CONC-001 mechanism

Nothing in the repository names one (measured for the shape round §7.3
and unchanged). The candidates:

**(A) `flock(LOCK_EX | LOCK_NB)` on the opened whole-device handle, for
every device in the bind set, journal-first (recommended).** Two
layers, each doing the job only it can do:
- **The journal is the CONC-005 arbiter.** A submission's phase two
  refuses if any journaled apply on an intersecting bind set is
  non-terminal — deterministic, explained, and durable across restarts,
  which is how *"including reboot-resumed phases"* is honestly read:
  the journal holds the logical lock from `ApplySubmitted` to the
  terminal record; the physical lock is re-established at recovery
  before any resumed byte. One journal per uid cannot arbitrate across
  users — which is exactly why the second layer exists.
- **The kernel's `flock` is the cross-process exclusion.** Taken
  `LOCK_EX | LOCK_NB` on the read-write whole-device handle at
  execution entry (each bind-set member, host-backing descended), held
  for the execution, released with the handle. It serializes the
  per-uid helpers against each other *and* against every non-PartMan
  writer that honors the convention; a failed `LOCK_NB` is a typed
  refusal ("device locked by another process") — the loser explained,
  never a hang. The udev-cooperation bonus (§1.7) is flagged, not
  load-bearing: correctness rests on the kernel's arbitration between
  claimants, not on udev's manners.
*Against, stated:* `flock` cannot name the foreign holder (the refusal
names the device and the fact, not the pid); and a third-party writer
that ignores the convention is excluded by nothing — which is true of
every candidate short of exclusive-open device claiming, and is why
the plan-hash freshness checks (CONC-002's revalidation) stand in
front of every byte regardless.

**(B) Lock files (`O_EXCL`) under `/run/partman`:** stale-lock liveness
questions, per-device identity keying to invent, no kernel arbitration
against anything that is not PartMan, and nothing `flock` does not do
better on the handle the helper already holds. **(C) A helper-held
in-memory table:** fails the cross-uid population outright (§1.2).

## 4. Decision 3 — EXE-001 and EXE-003 in 4b

**EXE-001 — the seam in 4b, the mechanism with the ceremony
follow-up (recommended).** On Linux the inhibitor is logind — reached
either as a D-Bus client (`org.freedesktop.login1` `Inhibit`, a held
fd) or as a launched binary (`systemd-inhibit`, awkward for a
long-lived hold; presence unmeasured). That is the same
bus-client-versus-launched-binary route the apply-ceremony round's
follow-up already owns on DR22–DR24's rows for `pkcheck` vs
`CheckAuthorization` — one route decision, two consumers, decided once
in that round. 4b ships the inhibition **seam** and EXE-001's own
surfacing sentence: the pre-apply report states whether inhibition
engaged, and on this build it honestly reports *not engaged — no
inhibitor route decided* until the follow-up lands. The Tier-2
destructive suite runs in disposable guests where sleep is not a
factor; the surfaced-before-apply answer is the spec's own shape for
an engage failure.

**EXE-003 — owed by 4b, in the delivered stream's shape
(recommended).** Progress rides envelope v2's `event` channel (§1.5):
step identity always; byte counts exactly where a step measures bytes —
the product's own table writer knows its bytes, a launched `mkfs` gives
none, and "where meaningful" is the requirement's own carve-out, stated
per step rather than fabricated; monotone never-backward pinned by test
on the stream's existing sequence discipline, the journaled-retry
exception typed; **no ETA surface in 4b** — none can be estimated
honestly yet, and EXE-003 constrains ETAs' labeling, not their
existence. Emitting none is recorded as the deliberate reading.

## 5. The decisions for the owner

1. **The store's home: option (a)** — WP-070 reserves
   `crates/artifact-store/**` and `schemas/artifact-store.md`, imports
   ADR-0030's four obligations, and its store increment (Rust, own
   sitting) sequences before 4b's `Protecting`?
2. **CONC-001: option (A)** — journal-first arbitration with
   `flock(LOCK_EX | LOCK_NB)` on every bind-set member's read-write
   handle, recovery re-acquiring from the journal before any resumed
   byte, both refusal arms typed?
3. **EXE-001/EXE-003: as §4** — the inhibition seam and honest
   pre-apply surfacing in 4b with the mechanism joined to the ceremony
   follow-up's bus-vs-binary decision; progress in 4b at step
   granularity with byte counts where a step measures them and no ETA
   surface?

## 6. Next acts, in order

1. This round (WP-000). Decisions.
2. The `Governance:` PR per decision 1 (WP-070's assignment edit).
3. WP-L110's consequential edit: the 4b row's owed-within list resolves
   to its remaining sequencing (the WP-070 store increment; the ceremony
   follow-up round).
4. The WP-070 store increment, then 4b.
