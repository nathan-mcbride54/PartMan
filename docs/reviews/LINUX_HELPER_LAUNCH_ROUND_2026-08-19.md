# The Linux helper launch round — how the helper comes to exist, and who makes the directory and the node

**Date:** 2026-08-19. **Base:** `0f18971` (main), spec 19.0.0; DR20 taken
the same day (gitea#1012).
**Directive:** the WP-L110 assignment (created 2026-08-19 on its arc
plan): the launch and endpoint-ownership route is claimed
increment-gated, decided before increment 1.
**Question:** ADR-0055 decided *how the helper is reached* — a
root-created `0711` directory, a `0600` node owned by the authorizing
user, the peer verified before any byte — and left to WP-L110 *how the
helper comes to exist*: who creates `/run/partman/` and the per-user node,
what starts the process, when it exits (HLP-005), and how that interacts
with HLP-003's interactive ceremony (polkit `auth_admin` without retained
grants, ADR-0021) and LIN-009 (polkit rules scoped to validated plan
execution, not broad command execution). The options were named by
ADR-0055 (T5 systemd socket activation; T6 an inherited pair through
`pkexec`) and the assignment (polkit-mediated start). This round costs
them on what DR20 measured across the three pinned tier images.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner. No
> spec text moves under any option here; the decision is recorded in
> WP-L110's record (its increment 1) and, where a package artifact results,
> in `packaging/`'s later work.

## 0. The premise, and the texts the round works under

- **RPC-001** (19.0.0): the Linux transport's directory and node rules.
  **ADR-0055** decisions 3 (flat per-user nodes under one root-created
  directory), 6 (Tier-1 posture) and 7 ("how the helper is launched … is
  WP-L110's, with T5/T6 recorded as compatible options"); its revisit
  condition: a launch that hands the helper a pre-connected pair makes
  the directory rules inapplicable on that path.
- **HLP-005**: "idles locked-down when no work exists, and MAY exit when
  idle." **HLP-007**: no work for non-local or cross-session callers;
  identity via RPC-001. **HLP-003 / ADR-0021**: every apply needs a floor
  act by the RPC-001-authenticated user; ≥ Disruptive or any flag needs
  "polkit `auth_admin` without retained grants"; "no apply at any severity
  proceeds from connection standing". **LIN-009**: "polkit rules scoped to
  validated plan execution, not broad command execution." **RPC-006**:
  clients reattach after disconnect or crash and reconstruct from the
  journal. **SAFE-002**: the GUI and CLI run without elevation; the
  helper is context 1. **SAFE-004**: any tool the client or helper
  launches goes through the structured-argv allow-list launcher (WP-035's
  `SystemLauncher` today). **LIN-008**: signed `.deb` and Arch packages
  (`packaging/`, not this round's, but the launch shape is what those
  packages install).
- **WP-L110's assignment**: the launch route decided before increment 1;
  the toolset and launcher-home routes before increment 4; the helper
  integrates and re-implements nothing.

## 1. What is measured (DR20, `docs/quality/observability.md`)

1. **polkit is on one default tier out of three.** jammy ships
   `policykit-1`/`polkitd`/`pkexec` **0.105** (the pre-JavaScript line:
   `.pkla`/`rules.d` under `/usr/share`, no `/etc/polkit-1/rules.d`),
   `polkit.service` always active, `pkexec` setuid `4755`. Debian 12 ships
   `polkitd` **122** (`libpolkit-gobject`, `pkcheck`, `pkaction`, the
   setuid `polkit-agent-helper-1`) but **not `pkexec`** (bookworm splits
   it into its own package); the daemon is **D-Bus-activated on first
   use** — `inactive` before the client's `pkaction` query, `active`
   after; its `rules.d` directories are `0700 polkitd`, unreadable to a
   client. Arch's cloud image ships **no polkit at all** (no binaries,
   no `/etc/polkit-1`, `polkit.service` `not-found`, `dbus-broker` as the
   bus).
2. **The ceremony with no agent is a clean refusal where it exists.**
   `pkexec --disable-internal-agent /bin/true` as the client on jammy:
   rc 127, `Error executing command as another user: No authentication
   agent found.` — not a hang, not a grant. A `runuser` client has **no
   logind session and no `/run/user/<uid>`** on any tier, which is the
   shape a non-interactive caller has and the shape no agent can answer
   in (ADR-0021's ceremony therefore needs an *interactive* caller —
   which is exactly what "fresh interactive authorization" means).
3. **systemd's launch substrate is on every tier**: `systemd` 249/252/261,
   socket units loaded (21/12/46), `systemd-run`, `systemd-tmpfiles`,
   client-readable `tmpfiles.d` (23/21/36 conf files, 7–9 naming `/run`
   entries in exactly the `d /run/<name> <mode> <user> <group> -` shape),
   `/run` a `tmpfs` `mode=755`, `dbus.service` active.
4. **What the transport needs exists without polkit** — `0711` directory
   and `0600` per-user node are filesystem facts any root process can
   make (ADR-0055's Tier-2 acceptance made them from a script).
5. **Unmeasured, and said so**: `pkttyagent` presence per tier (a text
   agent a CLI could run for the ceremony); whether D-Bus activation of
   `polkitd` works from a root caller with no session; a systemd
   *template* socket unit's behaviour with `SocketUser=%i`. Each is a row
   if an option below needs it.

## 2. The options, each against the texts

**L1 — a systemd socket unit owns the rendezvous.** `partman-helper.socket`
with `ListenStream=/run/partman/helper.sock`, the service with
`RuntimeDirectory=partman`/`RuntimeDirectoryMode=0711`, the helper started
on first connection and exiting when idle (HLP-005 exactly). *For:* the
directory and the process come from the init system on every tier (§1.3),
no polkit needed to *start*, reattach (RPC-006) is free — the socket
outlives the process. *Against, and it is decisive:* a single listening
node cannot be "owned by the authorizing user and `0600`" for more than
one user — it would be `0666` (or group-owned), which is ADR-0055's
**A2** (credential check as the sole gate), rejected by that ADR; a
per-user *template* socket (`partman-helper@.socket`,
`ListenStream=/run/partman/helper-%i.sock`, `SocketUser=%i`,
`SocketMode=0600`) satisfies the node rule but needs *something* to start
the instance for uid `%i` — and an unprivileged user starting a unit is
systemd's own `manage-units` polkit action, which is back to polkit, on a
default Arch image that has none. L1 is the right *deployment* for a
distribution that ships polkit and a unit; it is not the route that
creates the node for the user who is asking.

**L2 — launched on demand by the client through `pkexec`, the helper
makes the directory and its own node.** The client (CLI or GUI) runs, via
its SAFE-004 launcher, `pkexec /usr/libexec/partman/helper-linux --serve
<uid>`; polkit authorizes a package-shipped action
(`org.partman.helper.serve`, whose policy lets an **active local** user
start a helper for **their own uid** — `allow_active` without a password;
the rule is scoped to that one action, LIN-009's spirit: a helper that
only serves status/validate needs no ceremony to *start*); the helper,
now root, creates `/run/partman` (`0711`) if absent, refuses to serve a
uid other than the one `pkexec` vouches for (`PKEXEC_UID`, set by `pkexec`
itself), creates `helper-<uid>.sock` `0600` owned by that uid, serves, and
exits when idle (HLP-005); the client connects to the node after launch.
*For:* per-user nodes fall out naturally for exactly the user asking;
ADR-0055's rules hold verbatim; the helper's lifetime is the user's
activity, not the boot; reattach works while the helper lives and a
re-launch reconnects to the journal after it exits (RPC-006 "reconstruct
from journal plus event replay" is the protocol's, not the process's);
the apply-time ceremony (HLP-003) is a *second*, separate polkit ask the
helper makes per plan — launch authorization is never apply
authorization, which is what keeps "no apply from connection standing"
true. *Against:* polkit becomes a **package dependency on every tier**
(`policykit-1` on jammy — present; `polkitd` **and `pkexec`** on Debian 12
— `pkexec` not by default; `polkit` on Arch — not by default), which
LIN-009 already implies and LIN-008's packages can declare; a headless
CLI with no agent cannot launch (DR20: rc 127) unless the CLI runs a text
agent (`pkttyagent`, unmeasured) or the action's default is
`allow_active`/`yes` for the launch step (then no agent is asked at all,
and only the apply ceremony needs one — the plan's preference: *starting
a read-only helper asks nothing; mutating asks every time*); a polkit
action file and (on 0.105) a `.pkla` or (on 122) a `.rules` ship with
the package — two policy dialects for one rule, measured, not a
surprise.

**L3 — inherited `socketpair` through `pkexec`** (ADR-0055 T6): the
client spawns the helper via `pkexec` with a pre-connected pair on
stdin/stdout; no directory, no node. *For:* the kernel's admission is the
launch itself. *Against:* RPC-006 reattach is impossible on a pair (a
crashed client's pair is gone), the helper's lifetime is the client's
(HLP-005's idle-exit is moot and CONC-005's "multiple clients through the
same helper" is unreachable — a second client would get a second
helper), and `pkexec` closes descriptors beyond 0/1/2 so the pair rides
the standard streams. Rejected; recorded by ADR-0055 as a compatible
shape for a single-client context, which the product is not.

**L4 — the helper runs always, as a boot service, creating per-user nodes
for every human account.** *Against:* a node for users who never ask,
none for users created later, a root daemon idling forever against
HLP-005's "MAY exit"; and it still needs polkit for the apply ceremony.
Rejected.

**L5 — a D-Bus-activated helper** (`org.partman.Helper` name, the bus
starts it). *Against:* a D-Bus client in the helper for its own
activation — ADR-0054 declined D-Bus for discovery and the round sees no
reason to bring it in through the side door for launch when `pkexec` and
systemd exist; and it does not answer who makes the per-user node.
Rejected for launch (the *ceremony* may still speak to polkit over the
bus or through `pkcheck` — the toolset round's).

**L6 — the client creates the directory and node and hands the helper a
path.** *Against:* SAFE-002 — the client cannot create a root-owned
directory; and ADR-0055's node is root-created precisely so no user can
squat it. Rejected.

## 3. What is genuinely open, and the adversarial pass

1. **Is L2 "polkit for broad command execution" (LIN-009's prohibition)?**
   No: the action authorizes one executable (the helper, at its packaged
   absolute path) with one argument shape (`--serve <uid>`, refused unless
   `<uid>` equals `PKEXEC_UID`); the helper itself accepts HLP-001's six
   operations and nothing else; and launch authorization authorizes no
   apply — HLP-003's act is separate and per plan. The pass checked the
   failure mode "a launch rule that `pkexec`s a shell" — structurally
   excluded: the action's `org.freedesktop.policykit.exec.path` annotation
   names the helper binary alone.
2. **Does L2 make the product unusable on a default Arch image?** Without
   the package's dependency on `polkit`, yes — and that is also true of
   HLP-003's ceremony under any option, because ADR-0021 names polkit.
   LIN-008's Arch package declares `polkit`; the read-only client still
   works without any helper (the M1 product). Recorded, not hidden.
3. **Headless CLI:** with the launch action at `allow_active`/`yes`, no
   agent is consulted to start; the apply ceremony for ≥ Disruptive still
   needs an agent — a text agent the CLI runs (`pkttyagent`, unmeasured —
   a row for increment 3) or a refusal with a remediation naming it.
   Severity-0 applies still need the floor act, which is the CLI's own
   `apply <hash>` — programmatic, per ADR-0021 — and no agent.
4. **Could L2's "the helper creates `/run/partman`" race two clients?**
   Two `pkexec` launches for two uids create two nodes under one
   directory; `mkdir` of the directory is idempotent and the transport
   refuses a pre-existing node for the *same* uid (`NodeAlreadyExists`) —
   the second launch for one uid finds the first helper's node and
   **connects to it instead of serving**, which the round makes an
   explicit rule of increment 1 (launch = "ensure a helper for my uid",
   not "start a new one").
5. **Does any of this move spec text?** No. RPC-001 stands; HLP-005 is
   honoured; LIN-009's scoping is met. The launch shape is a record in
   WP-L110 and an artifact in `packaging/`.
6. **Open, the decision owner's:** whether increment 1 ships the polkit
   action/policy files (two dialects) and a `.service`/`.socket` for the
   L1 deployment option beside the L2 launch, or only L2 — the round's
   lean is **L2 only in increment 1**, L1 as a deployment option filed for
   `packaging/`'s time.

## 4. The recommendation

**L2.** The client launches the helper through its SAFE-004 launcher via
`pkexec`, under a package-shipped polkit action that admits an active
local user to start a helper for their own uid only (`allow_active`;
nothing asked); the helper, as root, ensures `/run/partman` (`0711`),
refuses any uid but `PKEXEC_UID`, creates the `0600` node for that uid
through `partman-transport-linux::Endpoint::create` (never replacing an
existing node — a second launch for the same uid connects to the first),
serves, and exits when idle. The apply ceremony is a separate polkit ask
per plan (HLP-003, ADR-0021), decided in its mechanism by the toolset
round (`pkcheck` through the launcher, or the bus). polkit is a package
dependency on all three tiers — `policykit-1`, `polkitd`+`pkexec`,
`polkit` — declared by LIN-008's packages. Increment 1 ships the helper,
its `--serve` refusal logic (Tier-1 over injected `PKEXEC_UID`), and the
polkit action and policy files under `services/helper-linux/`; its Tier-2
acceptance launches it through `pkexec` in a disposable jammy guest (the
one tier where polkit is there by default), and in a Debian 12 and an
Arch guest **after** the package dependency is installed by the setup
actor, recorded as such.

Why L2 over L1 in one sentence: only a launch that knows *which user is
asking* can make that user's `0600` node, and `pkexec` tells the helper
exactly that, on every tier, once polkit is a dependency the ceremony
already makes it.

## 5. Open questions for the decision owner

1. L2 as recommended, or L1 with per-user template sockets (and its
   `manage-units` polkit dependency for starting them)?
2. The launch action's default: `allow_active` (no ask to start; the
   plan's lean) or `auth_admin` (a password to start — doubling the
   ceremony for nothing)?
3. Ship the L1 deployment files (`.socket`/`.service`) in increment 1
   beside L2, or leave them to `packaging/`?

## 6. What would change this round's mind

- A row showing polkit present and `polkitd` runnable by default on the
  Arch image — L2's dependency cost drops to zero and L1's per-user
  template sockets become easier to justify; the recommendation holds.
- A decision that the product ships a per-user session helper
  (user-owned, unprivileged, proxying to a root service) — a different
  architecture; not in any text today.
- `pkexec` measured absent and un-packageable on a tier — not the case
  (Debian 12 packages it separately; Arch's `polkit` includes it).

## 7. Next acts, in order

1. This round (WP-000). Decision.
2. WP-L110 increment 1 under `Work-Package: WP-L110` (`services/helper-linux`,
   `schemas/helper/`), its Tier-2 acceptance in three guests, its WP-020
   sitting; a row for `pkttyagent` filed on WP-035 for increment 3.
