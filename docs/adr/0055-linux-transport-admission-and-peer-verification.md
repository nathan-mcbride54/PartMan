# ADR-0055: The Linux transport admits the authorizing user by the kernel and verifies the peer by its credentials

- Status: Accepted
- Date: 2026-08-19. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/LINUX_TRANSPORT_ROUTE_ROUND_2026-08-19.md`, a committed
  session record; T1 + A1 taken by the decision owner: flat per-user
  socket nodes under one root-owned directory, per-message credentials
  deferred, the transport in its own crate), resolving **SI-41**
  (`docs/spec-issues/README.md`), which the round filed from a
  measurement. Recorded before its first consumer is written — merging is
  not acceptance.
- Spec version: 19.0.0 (major under §0.1 — RPC-001's Linux clause, a
  MUST, changes meaning: the socket directory's mode and the socket
  node's ownership)
- Work packages blocked: none (the first consumer is WP-040's Linux
  transport increment, which this ADR and a Governance grant for
  `crates/transport-linux/**` open; WP-L110's launch and authorization
  route stay its own)
- Requirement IDs: RPC-001, SAFE-002, HLP-003, HLP-005, HLP-007, RPC-006,
  SEC-007, SAFE-009, ADR-0021, ADR-0054
- Decision owners: Nate McBride

## Context

RPC-001 read, since 2.0.0: "Linux — Unix domain socket (0700, root-owned
directory) with peer-credential verification." WP-040 delivered the
protocol layer (RPC-002…006) pure and endpoint-less and gated each
transport behind a recorded route decision, because no transport is
"simultaneously dependency-free, `unsafe`-free, and clean against the
workspace rules" (the WP-035 increment-10 triangle); its boundary named
the Linux cost exactly: "peer-credential verification needs `SO_PEERCRED`
beyond std's surface." Its claim vocabulary (`IdentityClaim::
UnixPeerCredentials`) says what the peer proves — "the kernel-reported
credentials of the connecting process match the peer the socket was
created to serve" — and that its verifier waits on this decision.

The round measured the two things that decide the route. **First**, on the
pinned toolchain (`rustc 1.96.0`) `std`'s `UnixStream::peer_cred` is
unstable (`peer_credentials_unix_socket`, rust-lang/rust#42839), so the
credential read is indeed beyond `std`; but `rustix 1.1.4` — already a
Linux-only dependency of `crates/ffi-linux-loop`, a crate that inherits
the workspace's `unsafe_code = "deny"` and carries no first-party
`unsafe` — exports `rustix::net::sockopt::socket_peercred(fd) ->
UCred { pid, uid, gid }` as a safe function under its `net` feature, and
`deny.toml` already admits it. **Second**, the literal clause does not
work: a Unix socket is connected to by path, and the kernel requires
search permission on the directory and write permission on the socket
inode of the connecting process; measured on the Proxmox node with a
child dropped to uid 65534 and its supplementary groups cleared, a
`0700 root:root` directory refuses the connection (`EACCES`) before the
listener sees anything, while a `0711` root-owned directory admits it and
`SO_PEERCRED` then reports the child's credentials; a `0660 root:root`
node in a `0711` directory refuses again. SAFE-002 places the GUI and CLI
at no elevation, so RPC-001's Linux transport, as written, excludes the
only client it exists to serve — a requirement-versus-requirement
conflict (RPC-001 against SAFE-002, with HLP-003 and HLP-007 left with no
caller to identify), filed as SI-41 under Section 0.2 rather than read
around. The Windows clause has no such defect: its SDDL admits "the
authorizing interactive user" by name; the Linux clause's "0700" was the
same intent with the wrong bit.

## The decision

1. **RPC-001's Linux clause is revised** (19.0.0): "Linux — Unix domain
   socket in a root-owned directory searchable but not writable by
   others (mode 0711), the socket node owned by the authorizing user and
   accessible to that user alone (mode 0600), with peer-credential
   verification of the connecting process against that user." Two
   gates, both enforced by the kernel before the helper parses a byte:
   the root-created directory keeps the path unsquattable (no other user
   can create, replace or symlink the node), and the node's owner and
   mode admit exactly one uid — the Linux analog of the Windows SDDL.
   The peer-credential check then *verifies* what the kernel admitted.
   The Windows and macOS clauses are untouched.
2. **The credential route is `std` plus `rustix`'s safe `socket_peercred`**
   (T1): the endpoint, its bounds and timeouts, the directory and node
   checks from `std`; one safe call for the credentials; **no first-party
   `unsafe`**, no new FFI family; the dependency is Linux-only and
   already in the workspace. The verifier refuses a connection whose
   `uid` is not the endpoint's authorizing user **before any byte is
   read** (HLP-007), and the refusal is a typed, redactable reason, not a
   silent drop.
3. **The layout is flat per-user nodes under one directory**: one
   root-created `0711` directory (`/run/partman/` is the intended place;
   the path is the helper's runtime configuration, not this ADR's), one
   node per authorizing user, owned by that user, `0600`. Not per-uid
   subdirectories (a second directory to get wrong), not an abstract
   name (first-come, impersonable), not a group-owned directory (standing
   membership is the opposite of HLP-003's per-apply act).
4. **Per-message credentials (`SCM_CREDENTIALS`) are deferred**: the
   connecting process's credentials at `accept` are the caller's
   identity HLP-007 names; a mid-connection identity change is not a
   threat the spec prices. Recorded as the hardening route if one is
   ever added.
5. **The transport lives in its own crate**, `crates/transport-linux`
   (`partman-transport-linux`), Linux-only, depending on `partman-rpc`
   for the handshake and envelope types and on `rustix` (`net`) for the
   one call, lints inherited, reserved to WP-040 by a Governance grant.
   Not inside `crates/rpc`: that crate's stated posture ("no test opens
   a socket") stays true, and no platform dependency enters the schema
   types' consumers.
6. **The Tier-1 posture, stated:** tests may create a listener under a
   temporary directory the test owns (the directory rule is checked
   against the running euid — owner must be the endpoint's own euid,
   mode `0711`; in production that euid is 0), a `UnixStream::pair` whose
   `socket_peercred` is the test's own uid (the verifier's accept arm),
   and the refusal arm through the verifier's seam with an injected
   credential, since a second uid is not constructible unprivileged. No
   network socket, no process launch, no elevation. A root-owned
   directory, a foreign uid and a cross-user connect are the Tier-2
   acceptance the increment owes in a disposable guest, stated in its
   record as what Tier-1 did not reach.
7. **What this does not decide:** how the helper is launched (a systemd
   unit, `pkexec`, polkit under LIN-009 — WP-L110's, with T5/T6 recorded
   as compatible options), what the helper does with the identity
   (HLP-003's floor act names the RPC-001-authenticated user; WP-070/
   WP-L110), and the helper's discovery/mutation route (ADR-0054).

## Options considered

### T2 — `libc` and `unsafe getsockopt` in a reviewed FFI module

The triangle's original price: first-party `unsafe` (permitted in an FFI
or helper crate under SAFE-009, with review), a `libc` dependency, a
hand-written struct. Strictly worse than T1 now that `rustix` is in the
graph — the same syscall with the `unsafe` moved into our code. Rejected.

### T3 — `nix`

A second `libc`-based wrapper family beside `rustix`. Rejected.

### T4 — `SCM_CREDENTIALS` per message

Deferred; see decision 4.

### T5 — systemd socket activation; T6 — an inherited `socketpair` via `pkexec`

Not transport routes: the first is a deployment choice that still needs
T1's verifier and would bind Tier-1 to an init system; the second decides
the helper's launch (WP-L110's), rides stdin/stdout, and makes RPC-006's
reattach after a client crash impossible on a pair. Both recorded as
compatible with this transport's protocol.

### T7 — an abstract-namespace socket

No directory, no node permissions: every uid connects, and the name is
first-come — any process can bind it and impersonate the helper to
clients. Rejected.

### A0 — the literal 0700 directory

Measured unsatisfiable with SAFE-002 (SI-41). Not an option.

### A2 — 0711 directory, 0666 node, the credential check as the sole gate

Works, with the same verifier; one gate where the clause wrote two, and
the helper then parses nothing from strangers only because its own first
check holds. Rejected for the transport.

### A3 — a group-owned 0750 directory

Admission by standing administrator-granted membership, admitting every
member rather than the authorizing user. Rejected.

### A4 — reversed roles (the root helper connects to a client-owned directory)

The peer-credential clause is about the connecting process, which would be
root, proving nothing about the caller; and the helper would need a
rendezvous, which is a root-owned directory again. Rejected.

## Consequences

- **Positive:** the Linux transport is buildable with no first-party
  `unsafe` and no new dependency family; RPC-001's Linux clause is
  satisfiable by a SAFE-002 client and has the same two-gate shape as
  the Windows clause; WP-L110 can be assigned against a reachable
  endpoint; SI-41 is resolved on a measurement.
- **Negative, accepted knowingly:** a major spec bump for one clause;
  `rustix` enters a second crate (Linux-only) and its `net` feature's
  review surface is the one function's semantics; the endpoint's
  per-user node needs root to create it for the user, which ties endpoint
  creation to the helper's launch (WP-L110's) — the transport crate
  creates and checks, it does not decide who runs it; and per-message
  identity is not asserted (decision 4).
- **Evidence obligations:** (1) the Tier-2 acceptance of decision 6 — a
  root-owned directory, a foreign uid refused by the kernel at the node,
  a spoofed credential refused by the verifier — in a disposable guest,
  recorded with the increment; (2) `IdentityClaim::UnixPeerCredentials::
  waits_on` updated to name this ADR and the increment; (3) the
  `SO_PEERCRED` semantics the verifier relies on (credentials at
  `connect`) stated in the crate's doc with the measurement that showed
  them.

## Verification

- When WP-040's Linux transport lands on this ADR: an endpoint refuses
  to serve on a directory not owned by its euid, not `0711`, or holding a
  pre-existing non-socket or symlinked node, and never changes the mode
  of a path it did not create; the node it creates is owned by the
  authorizing user and `0600`; an accepted connection whose credentials'
  `uid` differs from the authorizing user is refused before any byte is
  read, with a typed reason; a matching `uid` proceeds to the RPC-002
  handshake; every read is bounded by `MAX_MESSAGE_BYTES` and a timeout;
  no network socket type exists in the crate. Each with a test, each with
  a mutation killed.
- `crates/rpc` stays free of sockets in tests; `partman-transport-linux`
  carries no `unsafe` (workspace lint) and builds on Linux only.

## Revisit conditions

- `peer_cred` stabilises in `std` on a toolchain the workspace moves to
  — the `rustix` dependency leaves and the route is `std` alone; nothing
  else changes.
- WP-L110 chooses a launch that hands the helper a pre-connected pair
  (T6) — the directory and node rules become inapplicable on that path
  and the verifier applies to the pair; this ADR's clause still governs
  the path-based endpoint.
- A requirement arrives that prices mid-connection identity change —
  T4's per-message credentials are added on top, not instead.
