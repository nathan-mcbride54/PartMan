# The Linux apply-ceremony route round

- Date: 2026-08-19
- Package: WP-L110 (increment 3, the authorization ladder)
- Decision owner: Nate McBride
- Evidence: `docs/quality/observability.md` — DR20 (the polkit-and-launch
  cell) and **DR22–DR24 (the apply-ceremony cells, taken this day on all
  three tiers, gitea#1014)**
- Governing texts: ADR-0021 (spec 11.2.0), HLP-003, HLP-004, CAP-007,
  SAFE-002, SAFE-004, SAFE-005, LIN-009, SEC-009, RPC-002, ADR-0028

## 0. What this round decides, and what it does not

**Decides:** how the root helper obtains polkit's `auth_admin` answer
**about its client** for an apply at severity ≥ Disruptive or carrying any
step flag — and whether that mechanism ships in increment 3 or is deferred
behind a structurally-unconstructible seam.

**Does not decide:** the *mechanism* in the abstract — ADR-0021 already
fixed it ("Linux — polkit `auth_admin` without retained grants") — nor
anything about the floor act, which needs no polkit at all and is
programmatic by ADR-0021's own text.

**Not in this round, recorded so the boundary is visible:** the mutation
toolset (route b) and the launcher's home (route c) remain WP-L110's own
gates before increment 4. This round *touches* route (c) and says how.

## 1. What the record now establishes

DR20 measured what a *client* sees. It did not ask what a **daemon** can
learn, which is the whole ceremony. DR22–DR24 asked, on all three pinned
tiers, in two identical captures.

1. **The helper can ask polkit about its client, and the instrument is
   present on both polkit tiers.** `pkcheck --action-id … --process <pid>`
   resolved every client subject and returned **rc 0 for a `yes` control
   action** — the subject is evaluated, not rejected. `pkcheck` is 0.105 on
   jammy and 122 on Debian 12, and on **both** it comes from the `polkitd`
   package, so `pkexec`'s absence on Debian 12 does not remove the
   ceremony's instrument there.
2. **`auth_admin` for a client refuses, bounded and typed.** rc 2,
   `Authorization requires authentication and -u wasn't passed.`; with
   `--allow-user-interaction`, rc 2, `Authorization requires
   authentication but no agent is available.` Never a hang, never a grant.
3. **`auth_admin` for a *root* subject returns rc 0, with no agent and no
   prompt**, on both tiers. This is the measured shape of the fail-open:
   a ceremony implemented with the helper as its own subject authorizes
   everything and proves nothing.
4. **No daemon or script can register an authentication agent for a
   client.** `pkttyagent --process <pid>` returns rc 127, `Error creating
   textual authentication agent: Error opening current controlling
   terminal for the process ('/dev/tty'): No such device or address`.
   Measured for both a session-less and an `ssh`-session subject, because
   the *registering context* had no terminal. The agent belongs to the
   client's own terminal.
5. **The D-Bus authority answers a root caller**, and on Debian 12
   **activates `polkitd` on demand** (inactive before the call, running
   after) — the launch round's second unmeasured item.
   `CheckAuthorization`'s signature, verbatim:
   `(sa{sv})sa{ss}us` → `(bba{ss})`, beside
   `RegisterAuthenticationAgent (sa{sv})ss`.
6. **A remote `ssh` login session is `Active` to logind** —
   `Class=user Active=yes State=active Remote=yes Seat=` on both polkit
   tiers. Increment 1's H2a arm measured a `runuser` client with **no
   session at all**; the generalisation drawn from it, that an `ssh` CLI
   user is not "active" either, is **measured false** and is corrected on
   `docs/work-packages/WP-L110.md` with this increment.
7. **On a default Arch image there is nothing to ask**: no `pkcheck`, no
   `pkttyagent`, no agent helper, no activation file; the bus call fails
   `The name is not activatable`. That is a packaging fact — LIN-008's
   Arch package declares `polkit` — not a helper defect.

**What is still unmeasured, and named rather than assumed:** no
`auth_admin` authorization has ever *succeeded* for a client anywhere in
this record, because succeeding needs a human at a terminal typing an
administrator password, which no headless sitting can produce. Every
grant-capable route below therefore rests, at its last step, on an
unmeasured belief. That asymmetry is the round's central fact.

## 2. The options, each against the texts

**R1 — `pkcheck` through a SAFE-004 launcher.** The helper launches
`pkcheck --action-id <action> --process <pid>,<start-time>,<uid>
--allow-user-interaction` and reads the exit status.
*For:* the instrument is present and measured on both polkit tiers (§1.1);
the subject is the client, not the helper; the exit statuses are already
known and bounded (§1.2).
*Against, and each of these is verified, not feared:*
- It **launches a tool**, which SAFE-004 routes through the one reviewed
  launcher — and the only implementation, `ToolLauncher`/`SystemLauncher`,
  lives in `apps/cli/src/doctor.rs:163-200`. **A helper cannot depend on
  an app.** So R1 pulls route (c), the launcher-home decision, ahead of
  its stated gate.
- That launcher's `LAUNCH_TIME_LIMIT` is a **private 5-second constant**
  (`apps/cli/src/doctor.rs:118`) and, unlike the output limit, is *not* a
  caller-stated bound. **A human typing a password outlives five seconds.**
  Reusing the launcher as delivered would kill every real ceremony; the
  launcher must therefore change, in whichever package ends up owning it.
- CAP-006 wants a floor row for a first-invoked tool, and
  `docs/capabilities/**` is **WP-050's reserved path** — a second package's
  act. A single floor is also incoherent across polkit `0.105` and `122`.

**R2 — the D-Bus authority directly.** `CheckAuthorization` with a
`unix-process` subject, `AllowUserInteraction`, and the plan hash in the
details map.
*For:* no tool launch — so no launcher-home gate, no 5-second kill, no
CAP-006 floor; the details map and the interaction flag are the only
*named* homes for the hash binding and for "no retained grant"; measured
reachable from a root caller on both tiers, with on-demand activation
(§1.5). ADR-0054 closed the bus for *discovery* and the launch round's L5
explicitly left it open for the ceremony.
*Against:* the product's first D-Bus client, inside the privileged
process — a supply-chain edge into root, or a hand-rolled bus client.
Needs explicit text that a system-bus Unix socket is neither SEC-007
network I/O nor "a transport of its own".

**R3 — the client performs the ceremony and reports the result.**
**Rejected:** a direct CAP-007 violation. polkit issues no bearer token the
helper can verify, so a client that skipped the ceremony presents the
identical nothing.

**R4 — the helper asks polkit with *itself* as the subject.**
**Rejected, and named so no later patch arrives at it by accident.** §1.3
measures it: root's `auth_admin` returns rc 0 with no agent and no prompt.
It would authorize every apply, record nothing about any human, and pass
every test.

**R5 — a per-apply `pkexec` re-launch of a second privileged worker.**
**Rejected:** puts a second privileged process in the apply path against
SAFE-002's two contexts, and `pkexec` is absent by default on Debian 12.

**R6 — ship a `.rules`/`.pkla` grant so the action passes.**
**Rejected:** it *is* the caching complement ADR-0021 rejected, written as
configuration — a standing YES is a remembered approval HLP-003 forbids at
these severities.

**R7 — the helper spawns `pkttyagent` itself.**
**Rejected:** the helper would both ask and answer. §1.4 also shows the
agent needs a controlling terminal, which a socket-activated or idling
helper has not got; but the principled ground stands on its own.

**R8 — defer the ceremony behind a seam whose completion value is
unconstructible in shipped builds.** One trait, one token type with no
public constructor, and a typed `CeremonyUnavailable` refusal. Everything
else in increment 3 ships.
*For:* it costs nothing the measured substrate has not already lost — Arch
has no polkit, Debian 12 has no `pkexec`, and no client `auth_admin` has
ever succeeded anywhere in this record. It keeps the fail-closed answer
that WP-L110's own evidence rule prescribes where no evidence exists.
*Against:* the interactive tier is declared and not served, which must be
said plainly in the increment table, the schema and the PR body.

## 3. The protocol shape, which R1 and R2 both need decided

A ceremony that waits on a human cannot sit inside a single blocking
privileged call: the delivered launcher would kill it at five seconds
(§2/R1), the helper's idle watchdog would kill the *process* at 120, and
RPC-004's bounded response has nothing to say about a prompt. Two shapes
exist and the round records both:

- **S1, one round trip.** `apply-plan` blocks while polkit prompts. Needs a
  caller-stated launcher bound, a watchdog that knows an operation is in
  flight, and a response deadline longer than a human.
- **S2, two-phase.** `apply-plan` answers **`awaiting-authorization`**
  immediately — Section 8's own entry edge — and a second `apply-plan` for
  the same plan hash completes once the client's agent has answered. This
  keeps a human-length prompt out of every blocking privileged call,
  survives the idle watchdog, and is the shape RPC-006's reattach already
  assumes. HLP-001 closes the *operation set*, not the round-trip count,
  so S2 adds no operation.

**S2 is the recommendation** whichever of R1/R2 wins, and it is why
`apply-plan` should not be served before the ladder is complete (§5).

## 4. Recommendation

**R8 for increment 3, with R2 as the target route and R1 alive, decided in
a follow-up round once a ceremony has been observed succeeding once.**

**The one decisive ground:** every route that can *grant* rests, at its
final step, on a fact this record does not contain — a client's
`auth_admin` succeeding. What the record does contain is the fail-open's
exact shape, measured: root's own `auth_admin` returns rc 0 (§1.3). WP-L110's
evidence rule decides this by rule rather than preference: *"Where none
exists the increment delivers the fail-closed answer and says so."*

**Two constraints worth binding now, whichever route wins**, because they
cost nothing and cannot be measured headlessly: any shipped ceremony
action declares `auth_admin` in **all three** implicit values and never a
`*_keep` variant; and any runtime call passes no keep-implying flag.
Together they are "without retained grants" made structural.

## 5. What does not depend on this decision

The whole of increment 3 except the body of one trait method: the
helper-computed tier (`required_tier`, structural over severity and the
flags-nonempty rule, so a sixth flag escalates without an edit); the tier
reported on the validate-plan response (schema version 3) for UI-011; the
provenance typing that makes a client-claimed tier unrepresentable
(CAP-007); the floor act's own arm, which needs no polkit; the typed
`CeremonyUnavailable` refusal; the audit event; and the correction that
`apply-plan` is served in increment **4**, not 3 — because single-use
cannot be held on a served path whose consumption record does not exist
yet.

## 6. Three delivered fail-opens this round found, which increment 3 fixes

An adversarial pass over increments 1–2 found three, all in code this
package shipped, all of which the ladder would stand on:

1. **`now_secs()` returns `0` when the clock is before the epoch**
   (`services/helper-linux/src/linux.rs`, `map_or(0, …)`). Then
   `admit_presented_plan`'s `not_after < now` is never true and **HLP-004's
   expiry fails open**. Fixed by a fallible clock whose failure refuses.
2. **The idle watchdog exits on wall-clock idle regardless of work in
   flight**, so a long capture can be killed mid-operation and its node
   removed under a connected client — HLP-005 says exit when *idle*.
   Fixed by a serving guard over a pure predicate.
3. **A failed audit write is discarded** (`let _ = writeln!`), so an
   operation can proceed unaudited against SEC-009. Fixed by making the
   sink fallible and refusing the operation when the record cannot be
   written.

None was introduced by this increment; all three are named here rather
than fixed quietly, because the ladder's guarantees are exactly what they
would have undermined.

## 7. Next acts, in order

1. This round (WP-000). **Decision.**
2. WP-L110 increment 3 under `Work-Package: WP-L110`: the tier, the wire,
   the floor arm, the seam, the three fixes, the H2a correction; its
   Tier-1 suite and mutations; its WP-020 sitting; its Tier-2 acceptance.
3. The ceremony route's own follow-up round when a client `auth_admin` has
   been observed succeeding once — which needs an apparatus with a
   terminal and an administrator password, and is a row before it is a
   round.
