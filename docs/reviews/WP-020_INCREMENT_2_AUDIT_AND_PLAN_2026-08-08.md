# WP-020 increment 2: preconditions audit and a proposed shape — 2026-08-08

Untracked session artifact (`docs/reviews/**`, WP-000). Increment 2's scope
is "loopback and virtual-machine harness so a Tier-2 suite can exist," and
its status row is exact: unblocked, and still not delivered. This document
measures the ground before anyone builds on it (the audit), then proposes a
shape (the plan). The plan is a proposal; nothing here authorizes code.

---

## Audit: the four carried preconditions, measured against current state

**1. Authorization bound to the file object — CLOSED, with a platform
asymmetry that increment 2 inherits.** Unix: held root directory object,
`openat` direct-child with `NOFOLLOW`, no intermediate component to
redirect (2c). Windows: closed by a *different mechanism* — the filesystem
driver refuses the swap while the root handle excludes `FILE_SHARE_DELETE`
(2d) — so it holds exactly as far as the driver does: measured on NTFS,
`ReFS`, and Windows SMB server; broken on the WSL 9p redirector (measured);
non-local roots refused outright. **Consequence for increment 2: the
destructive harness is Linux-VM-first by construction**, matching where the
loop closure lives anyway.

**2. Third factor — CLOSED by ADR-0007's Option C, with two revisit
conditions that increment 2 can trip.** The token is an operator-intent
proof, not an independent factor. Live constraints: (i) **the destructive
suite must keep an operator in the trigger path** — "a T2 or T3 harness
gains an unattended trigger" is a named revisit condition that would force
re-deriving the factor; (ii) if increment 3's lab architecture ever gives
privileged-test state a home outside the source tree and fixture root,
Option B (a real nonce) becomes cheap and should be taken. Increment 2
should not pre-empt (ii) and must not violate (i).

**3. Windows other-name refusal — CLOSED (2d), snapshot residual stands.**
The link count is read on both platforms through the handle; a new alias
created after authorization is not prevented, and the tolerability argument
(the bound object's bytes were verified and its own name is pinned) is an
argument, not a measurement. A destructive suite writes through exactly
this handle; the argument carries it unchanged.

**4. Loop binding — CLOSED (2e), and increment 2 must use 2e's closure,
not 2f's.** The descriptor-bound configure/verify/detach with the
adversarial rebind leg passed in the VM (2026-08-03). 2f's hold-open
session is **weaker than 2e by construction** — built so an external
prober can run mid-session — and is the wrong primitive for a destructive
attach. (Aside, flagged separately: the 2f status row still says no real
kernel has run it, which predates the 2026-08-03 SI-35 sittings that
consumed `run_probed_session` in the VM. Stale row, its own small WP-020
fix.)

**The gate in front of everything: issue #175.** The 2e acceptance's
reproducibility record pins `c75b340` and requires markdown-only diffs
since; fifteen non-markdown paths have landed, including
`crates/ffi-linux-loop` and `tools/xtask` — the acceptance's own code path.
By the record's own terms the proof is stale and must not be relied on.
**The re-take belongs at the start of increment 2's first VM sitting**
(same guest, same operational preconditions: root over direct login, no
`sudo`, no injected variables), which both discharges #175 and gives
increment 2 a fresh baseline commit to pin.

## Residuals a destructive suite must respect (recorded, not new)

- Digest/status checks are **discrete samples**; no continuous-binding
  claim. The 2e record says a destructive path "needs a separately proven
  pre-write discipline and may not inherit this conclusion" — increment 2
  must state its own pre-write discipline, not cite 2e's.
- VM isolation bounds consequences; exclusions rest on enumerated facts
  (single-actor guest, no other loop administrators), re-asserted per
  sitting, never inherited.
- ADS/attribute writes and post-authorization aliases remain possible on
  Windows — irrelevant to a Linux-VM suite but they bound any future
  Windows leg.
- A pass over no suite remains forbidden; registering the first suite makes
  the generic destructive refusal's tests change meaning. Every existing
  refusal test must be re-read, not just re-run.

## Proposed shape — two slices, both operator-accepted in the VM

**2g — the suite registry becomes real.** Today "no destructive suite is
registered" is load-bearing prose backed by refusal tests. 2g gives it a
type: a compiled registry (the ADR-0007 catalogue pattern — nothing read
from the fixture root), where a suite names its fixture set, its verified
target class, its per-fixture *intended-change contract* — which byte
ranges may change, everything else pinned by digest bracket — and its
teardown proof obligations. SAFE-007's three factors gate execution
unchanged. Tier-1 deliverables: the registry type, the refusal semantics
with one fake suite in tests (never compiled into the shipped registry —
the 2f compile-fail/structural-guard pattern), and mutation-verified tests
that an unregistered request still refuses and that a registered suite with
an unmet factor still refuses. **The trap this slice must dodge: the
guard-that-cannot-fail.** Every new refusal test gets mutated (flip the
gate, watch it fail by name) before it counts.

**2h — the first destructive suite, and the smallest honest one.**
Loop-backed via 2e's descriptor-bound closure, single fixture
(`gpt-basic-512` regenerated in-sitting), one mutation the harness itself
performs through the held descriptor — no external storage tool, keeping
ADR-0006's GPL-tool boundary and SAFE-004 untouched: overwrite the primary
GPT header's signature bytes, a write whose intended-change contract is a
named 8-byte range. Post-conditions: the changed range differs exactly as
contracted, every other sampled range's digest is unchanged, detach
confirmed, backing file regenerated and re-digested to the catalogue.
Acceptance registered like 2e's: named, operator-run, in the disposable VM,
with the adversarial leg being a rebind attempt mid-suite that must void
the run.

**Sequencing:** #175 re-take → 2g (Tier-1, mergeable on green) → 2h
implementation → one VM sitting running the 2e re-take, 2g/2h acceptance,
in that order, recorded in WP-020 with the same sitting discipline the
observability records use.

**What increment 2 must not contain,** so review has a bright line: no
product write path, no storage-tool invocation, no domain types, no plan or
hash surfaces — the mutation is harness-owned bytes through an authorized
handle, and the deliverable is that a Tier-2 destructive *test* can exist,
not that the product can write.
