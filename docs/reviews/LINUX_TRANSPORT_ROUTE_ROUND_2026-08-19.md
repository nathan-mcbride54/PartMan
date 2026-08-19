# The Linux transport route round — how the helper's Unix socket is reached, verified, and admitted

**Date:** 2026-08-19. **Base:** `4477000` (main), spec 18.0.0.
**Directive:** Nate — "draft the Linux transport round".
**Question:** WP-040 delivered RPC-002…006 as a pure, endpoint-less
protocol library and gated every RPC-001 transport behind "a recorded
choice among its named routes, never by drift", on the WP-035
increment-10 precedent: no transport is simultaneously dependency-free,
`unsafe`-free, and clean against the workspace rules. The Linux transport
is the one every Linux arc now waits on — WP-L110 (the helper) cannot be
assigned without a way to be reached, so 3b/gitea#1003 (the table-role
route), LIN-001's authorization half (ADR-0054), HLP-002's re-discovery
and the UDisks2 tool floor all queue behind it. This round costs the
named routes for the Linux Unix-domain-socket transport — how the
endpoint is created, how the peer's credentials are read, how the
unprivileged client is admitted, what Tier-1 may test — and surfaces one
thing the reading found and the measurement confirmed: **RPC-001's Linux
clause, read literally, excludes the client it exists to serve.**

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner. The
> conflict in §1.4 is filed as **SI-41** in `docs/spec-issues/README.md`
> in the same commit, because Section 0.2 requires filing rather than
> picking a side; the recommendation below is priced as its resolution.
>
> **Decided 2026-08-19 (Nate): T1 + A1 as recommended, flat per-user
> nodes, per-message credentials later, `crates/transport-linux` under
> WP-040.** Recorded as ADR-0055 and spec 19.0.0; SI-41 resolved.

## 0. The premise, and the texts the round works under

- **RPC-001** (`AGENT_BUILD_SPEC.md:302`): "Transports: Windows — named
  pipe with an SDDL restricting access to SYSTEM and the authorizing
  interactive user; Linux — Unix domain socket (0700, root-owned
  directory) with peer-credential verification; macOS — XPC with
  code-signing requirement checks, or an equivalently verified Unix
  domain socket."
- **SAFE-002** (`:151-157`): "The GUI, CLI, discovery layer, and default
  test suites MUST run without elevation. Privileged behavior is confined
  to exactly two contexts: 1. The platform helper executing a validated
  plan after fresh, explicit user authorization (HLP-003). 2. Privileged
  or destructive test suites …".
- **HLP-003**: every apply requires "a fresh, explicit **floor**
  authorization act: performed by the RPC-001-authenticated user, naming
  the exact plan hash, single-use …". **HLP-007**: "The helper performs
  no work on behalf of non-local or cross-session callers (SEC-002) and
  verifies caller identity via RPC-001 before processing any request."
  **HLP-005**: the helper "idles locked-down when no work exists, and MAY
  exit when idle."
- **SEC-007** at this layer (WP-040's reading): local IPC only; nothing
  opens a network socket. **SAFE-004**: tools through structured argv,
  fixed allow-list, verified identity. **SAFE-009**: "`unsafe` Rust is
  forbidden (enforced by lint in CI) in the domain, planner, validator,
  journal, and rpc crates. It is permitted only in adapter, FFI, and
  helper crates inside reviewed, documented modules."
- **WP-040's gate** (`docs/work-packages/WP-040.md`, Boundary): "Linux
  peer-credential verification needs `SO_PEERCRED` beyond std's surface
  … each transport increment opens only after a recorded choice among
  its named routes, with the route's dependency, `unsafe`, review, and
  Tier-1-testability costs stated and accepted in review." Its test
  tier: "Transport increments, when their routes are decided, state
  their own test posture in the route decision — an IPC endpoint cannot
  be tested without creating one, and what Tier-1 may create is part of
  what each route decision must record." And: "A transport route
  decision that needs a specification change … arrives as its own
  governance grant first."
- **The claim vocabulary** (`crates/rpc/src/identity.rs`,
  `IdentityClaim::UnixPeerCredentials`): proves "the kernel-reported
  credentials of the connecting process match the peer the socket was
  created to serve"; waits on "the linux transport route decision
  (SO_PEERCRED beyond std; unrecorded)".
- **ADR-0021** (SI-18): the floor act is "by the RPC-001-authenticated
  user"; no authorization field in any message; identity claims name
  what a peer proves, never what it may do. **ADR-0054**: UDisks2 is not
  the client's discovery interface; the helper's authorization/mutation
  route is WP-L110's — this round decides the *transport*, not that.
- **The precedents**: WP-035's increment-10 deferral ("the route that
  builds nothing", three routes costed); `crates/ffi-linux-loop`, the
  workspace's one Linux FFI crate — `rustix` (`fs`) and `linux-raw-sys`
  as Linux-only dependencies, **`[lints] workspace = true`, so no
  `unsafe` in first-party code**, reviewed under WP-020.

## 1. What is measured

1. **`std` cannot read peer credentials on the pinned toolchain.**
   `std::os::unix::net::UnixStream::peer_cred()` is
   `#![feature(peer_credentials_unix_socket)]` (rust-lang/rust#42839);
   compiled today against `rustc 1.96.0` (the workspace's pin):
   `error[E0658]: use of unstable library feature`. WP-040's "beyond
   std's surface" holds.
2. **`rustix` already in the workspace wraps `SO_PEERCRED` safely.**
   `rustix 1.1.4` (the version `ffi-linux-loop` pins) exports
   `rustix::net::sockopt::socket_peercred(fd) -> io::Result<UCred>`
   (`UCred { pid, uid, gid }`) under the `net` feature, `cfg(linux_kernel)`
   — a safe function over the syscall; the `unsafe` lives in `rustix`,
   which the workspace already carries and `deny.toml` already admits
   (Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT). Reading the
   registry source, not the docs.
3. **`std` does everything else.** `UnixListener::bind`, `accept`,
   `UnixStream::connect`, `pair`, `set_read_timeout` are stable;
   `std::os::unix::fs::PermissionsExt` reads and sets the directory and
   socket modes; `MetadataExt::uid()` reads the owner. No dependency for
   the endpoint, the bounds, or the directory checks.
4. **A 0700 root-owned directory refuses the unprivileged client.**
   Measured on the Proxmox node (kernel `7.0.14-11-pve`), as root, with a
   forked child that dropped to uid 65534 **with supplementary groups
   cleared** (a first run without `setgroups([])` passed through root's
   group and was discarded): directory `0700 root:root`, socket `0666` →
   `connect` **EACCES**, nothing arrives at the listener. Directory
   `0711` (or `0755`) root-owned, socket `0666` → connect OK, and the
   listener's `SO_PEERCRED` reads `uid=65534 gid=65534 pid=<child>`.
   Directory `0711`, socket `0660 root:root` → EACCES; directory
   `0750 root:root` → EACCES. So: the directory's **search** bit for the
   connecting user and the socket inode's **write** bit for that user
   are both required, and each is an admission gate the kernel enforces
   before the helper sees anything; the directory's listing and write
   bits are not required (0711 suffices, and keeps the path
   root-created, so no other user can squat or symlink it). `SO_PEERCRED`
   delivers the connecting process's pid/uid/gid at accept time.
5. **The transport has no consumer yet**, on either end: no helper
   (WP-L110 unassigned; WP-070 is the journal and state machine), no
   client surface that connects (WP-080's CLI apply/resume is M3). The
   first real endpoint is this increment's, and its only tests are its
   own.

## 2. The options, each against the texts

Two axes, decided together: **how the endpoint reads and verifies peer
credentials** (T-routes) and **how the unprivileged client is admitted
to a root-created socket** (A-routes — the SI-41 axis).

### T — reading and verifying the peer

**T1. `std` sockets + `rustix` `net` for `SO_PEERCRED`.** Endpoint,
bounds, directory and socket modes from `std`; one safe call to
`socket_peercred` on the accepted stream; the verifier compares `uid` to
the authorizing user the endpoint was created to serve (the claim's own
words) and refuses otherwise before any byte is parsed (HLP-007).
*Costs:* one Linux-only dependency feature (`rustix` with `net`, plus
the `linux-raw-sys` it already pulls) on a crate that ships on Linux
only; **no `unsafe` in first-party code** (lints inherited, like
`ffi-linux-loop`); SAFE-009 is satisfied by construction (the crate is
neither domain/planner/validator/journal/rpc *nor* does it contain
`unsafe`); `deny.toml` already admits it; review surface is the one
function's semantics (documented: the credentials are those at
`connect` time, which is what HLP-007 wants — the identity of the
caller that opened the connection). *Tier-1:* a test may create a
`UnixListener` under a temp directory and a `UnixStream::pair`,
unprivileged, no network, no process; `socket_peercred` on a pair
returns the test's own uid, which is exactly the verifier's positive
case, and a second uid is not constructible unprivileged — the refusal
arm is tested by injecting the credential value through the verifier's
seam, stated as such.

**T2. `libc` + `unsafe getsockopt` in a reviewed FFI module.** What the
triangle originally priced. *Costs:* first-party `unsafe` (permitted in
an FFI/helper crate under SAFE-009, with review), a `libc` dependency,
a hand-written struct layout. Strictly worse than T1 now that `rustix`
is in the graph: the same syscall, with the `unsafe` moved into our
code. Rejected.

**T3. `nix`.** `libc`-based safe wrapper; duplicates what `rustix`
provides, adds a second FFI crate family to audit. Rejected.

**T4. `SCM_CREDENTIALS` per message** (ancillary credentials on every
`sendmsg`, `rustix` supports it). *For:* re-asserts identity per
message rather than per connection. *Against:* HLP-007's rule is "before
processing any request" on a connection the helper accepted; the
connection's credentials at accept are the connecting process's, and a
credential that could change mid-connection (a client `exec`ing into
another uid) is not a threat the spec prices; the cost is ancillary
parsing on both ends. Not taken now; recorded as the hardening route if
a later requirement wants per-message identity.

**T5. systemd socket activation** (`LISTEN_FDS`): the listener is
created by systemd with unit-declared mode/owner and handed to the
helper. *For:* directory and mode management leave the helper. *Against:*
couples the endpoint's existence to a unit file (a packaging decision,
LIN-008's), does nothing for peer verification (still T1), and would
make the Tier-1 posture depend on an init system. Not a transport
route; a deployment option WP-L110/LIN-008 may take later, compatible
with T1.

**T6. Inherited `socketpair` through a privileged launcher** (the client
spawns the helper via `pkexec`/polkit and inherits a pre-connected
pair; no filesystem path). *For:* no directory, no path squatting, the
launch *is* the admission. *Against:* RPC-001 names a socket in a
root-owned directory; `pkexec` does not pass arbitrary descriptors, so
the pair would ride stdin/stdout; the helper's lifetime becomes the
client's (HLP-005's idle-exit becomes moot but reattach — RPC-006 — gets
harder: a crashed client cannot reconnect to a pair); and it decides the
helper's *launch*, which is WP-L110's. Rejected as the transport;
recorded as a launch option that would still speak this transport's
protocol over its pair.

**T7. Abstract-namespace socket** (`@partman`): no filesystem node, no
directory permissions — any uid connects, and `SO_PEERCRED` is the only
gate. *Against:* RPC-001 wants the directory precisely so the kernel
admits before the helper parses; and an abstract name is first-come —
any process can bind it first and impersonate the helper to clients.
Rejected.

### A — admitting the unprivileged client (SI-41)

**A0. Literal RPC-001: directory 0700, root-owned.** Measured
unsatisfiable with SAFE-002: the client gets EACCES. Not an option; the
conflict is filed.

**A1. Directory 0711 root-owned; socket node owned by the authorizing
user, mode 0600; `SO_PEERCRED` uid must equal the node's owner.** The
root-created directory keeps the path unsquattable (no other user can
create or replace the node); the node's owner and mode make the kernel
admit *exactly one uid* — the Linux analog of Windows' "SDDL restricting
access to SYSTEM and the authorizing interactive user" — and the
peer-credential check then *verifies* what the kernel admitted rather
than being the only gate. Two gates, independently enforced. The
"authorizing user" is runtime data the helper learns at endpoint
creation (as the Windows claim already says of its SDDL). Per-user
endpoints (`/run/partman/<uid>/helper.sock` or a per-user node under one
0711 directory) fall out naturally. *Costs:* a spec change — RPC-001's
"0700" becomes "root-owned directory, search-only to others, socket
node owned by the authorizing user and writable by that user alone" —
which is the SI-41 resolution.

**A2. Directory 0711 root-owned; socket 0666; `SO_PEERCRED` the sole
gate.** Simplest; every local process can connect and be refused after
the credential check. *Against:* it removes the kernel's admission gate
RPC-001 clearly wants (the Windows clause admits by ACL; "0700" is the
same intent, mis-specified), so the helper parses *nothing* from
strangers only because its own first check holds — one gate where the
spec wrote two. Rejected for the transport, though the verifier is the
same code.

**A3. Directory 0750 root:`partman`; a group the user is added to.**
Admission by group membership. *Against:* group membership is a
persistent, administrator-granted standing permission — the opposite of
HLP-003's per-apply fresh act — and it admits every member, not "the
authorizing user". Rejected.

**A4. Reverse roles: the client listens in its own 0700 directory and
the root helper connects.** Satisfies "0700" by moving it to the user's
side. *Against:* RPC-001's peer-credential clause is about the
*connecting* process, which would now be root — proving nothing about
the caller — and the helper must discover the client's path, which
needs a rendezvous (a root-owned directory again) or a launch protocol
(T6). Rejected.

## 3. What is genuinely open, and the adversarial pass

1. **Is SI-41 a reading or a conflict?** Measured: the literal clause
   denies the connection (EACCES) to any non-root uid, and SAFE-002
   forbids the client being root. The pass tried to save the clause:
   "0700 means the *helper's* private directory and the socket lives
   elsewhere" (the text says the socket is in it); "the client is
   launched by the helper" (no: HLP-005, RPC-006 reattach, and the
   Windows analog all presuppose an independent client connecting);
   "the directory is 0700 and the client is admitted through a
   bind-mount or a capability" (SAFE-002 again). None survive. Filed.
2. **Does A1 weaken RPC-001?** No — it tightens it: admission to
   exactly the authorizing user by the kernel, plus the credential check
   the clause already requires. What it drops is a mode bit that was
   never satisfiable. Pricing: a MUST's sentence changes meaning
   (`0700` → root-owned, search-only, user-owned node) — major, 19.0.0,
   with an ADR (the rule this repository uses: a sentence becomes false).
3. **Does T1 make the rpc crate `unsafe`-bearing or dependency-laden?**
   Not if the transport is its own crate. WP-040's protocol layer is
   "pure: no test opens a socket"; a transport crate's tests must. The
   clean shape is `crates/transport-linux` (`partman-transport-linux`,
   Linux-only build, `rustix` `net` as a Linux-only dependency, lints
   inherited, depending on `partman-rpc` for the handshake and message
   types), reserved to WP-040 by a Governance PR (the package's "arrives
   as its own governance grant first"). The pass considered putting it
   under `crates/rpc` behind `cfg(target_os = "linux")` and rejected it:
   it would make the protocol crate's stated test posture false and
   bind a platform dependency into every consumer of the schema types.
4. **Is the Tier-1 posture honest?** What an unprivileged test can
   create: a listener under a temp directory it owns (so the "root-owned
   directory" rule is tested against the running euid — the check is
   "directory owner == the helper's euid, mode has no group/other write,
   others search-only" — and in production euid is 0); a `UnixStream::
   pair` whose `socket_peercred` is the test's own uid/gid/pid (the
   verifier's accept arm); the refusal arm through the verifier's seam
   with an injected credential, because a second uid is not
   constructible unprivileged. What it cannot create: a root-owned
   directory, a foreign uid, a cross-user connect — those are the Tier-2
   acceptance this increment owes (a disposable guest, the r-series
   shape: the helper stub as root, the client as `muser1`, the EACCES on
   a wrong-uid node and the refusal on a spoofed-uid credential seam),
   stated in the increment's record as what Tier-1 did not reach.
5. **Does this decide anything for WP-L110?** Only that the helper is
   reached over this transport, verified by this claim, and admitted by
   the kernel for one uid. How the helper is launched (systemd unit, T5;
   `pkexec`, T6; polkit under LIN-009), what it does with the identity
   (HLP-003's floor act names the RPC-001-authenticated user), and
   whether it speaks UDisks2 (ADR-0054) stay WP-L110's.
6. **Open, the decision owner's:** (a) A1's per-user node: one directory
   `/run/partman/` 0711 root with `helper-<uid>.sock` nodes, or
   `/run/partman/<uid>/` subdirectories — the round prefers the flat
   form (one root-created directory, one node per authorizing user, no
   second directory to get wrong); (b) whether T4's per-message
   credentials are wanted from the start (the round says no: one gate
   more than the spec asks, for a threat it does not price).

## 4. The recommendation

**T1 + A1, in three acts.** (1) File **SI-41** (this commit) and resolve
it by **ADR-0055** with spec **19.0.0**: RPC-001's Linux clause becomes
"Linux — Unix domain socket in a root-owned directory searchable but not
writable by others (0711), the socket node owned by the authorizing user
and accessible to that user alone (0600), with peer-credential
verification of the connecting process against that user"; the Windows
and macOS clauses untouched. (2) A **Governance PR** reserving
`crates/transport-linux/**` to WP-040 (and the `Cargo.toml` member
line). (3) **WP-040 increment 5, the Linux transport**: endpoint
creation with the directory and node checks fail-closed (wrong owner,
wrong mode, a pre-existing non-socket node, a symlink → refuse to serve,
never chmod a stranger's path); accept → `socket_peercred` → refuse
unless `uid` equals the endpoint's authorizing user, before any byte is
read; the RPC-002 handshake then RPC-004's bounds from `crates/rpc`;
`IdentityClaim::UnixPeerCredentials::waits_on` flips to "ADR-0055,
increment 5"; Tier-1 as §3.4 states; a Tier-2 acceptance in a
disposable guest for the arms Tier-1 cannot reach; owes its WP-020
sitting like every Rust path.

Why T1+A1 in one sentence: `std` plus one safe call the workspace
already ships gives the kernel-reported identity with no first-party
`unsafe`, and a user-owned node in a root-created directory gives the
Linux transport the same two-gate shape the Windows clause has — the
kernel admits the one user, the helper verifies it — where the spec's
literal mode bit admitted no one.

## 5. Open questions for the decision owner

1. T1+A1 as recommended, or A2 (credential check as the sole gate) with
   the same code and a smaller spec edit?
2. Flat per-user nodes under one `/run/partman/` (preferred) or per-uid
   subdirectories?
3. Per-message `SCM_CREDENTIALS` now (T4) or later?
4. The crate shape: `crates/transport-linux` under WP-040 (preferred) or
   inside `crates/rpc` behind `cfg`?

## 6. What would change this round's mind

- `peer_cred` stabilising in `std` on a toolchain the workspace moves to
  — T1's dependency disappears and the route is `std` alone.
- A text under which the Linux client runs with elevation — none; SAFE-002
  is Section 3 and precedence runs the other way (SI-38's own
  precedent).
- A decision that the helper is launched by the client (T6) — then the
  path, the directory and SI-41 dissolve together, and the protocol
  rides a pair; the round would still recommend T1's verifier on that
  pair.

## 7. Next acts, in order

1. This round + SI-41 (WP-000). Decision.
2. ADR-0055 + spec 19.0.0 (RPC-001's Linux clause), `spec-change`,
   WP-000.
3. Governance PR: `crates/transport-linux/**` → WP-040.
4. WP-040 increment 5 under `Work-Package: WP-040`: the transport crate,
   Tier-1 tests, the Tier-2 acceptance record, `identity.rs`'s
   `waits_on` updated (a Rust path in `crates/rpc`: r-sitting), package
   record sweep.
