# The Linux transport (WP-040 increment 5, ADR-0055, spec 19.0.0)

RPC-001's Linux clause as an endpoint: `crates/transport-linux`
(`partman-transport-linux`). This document records the wire frame, the
endpoint rules and the admission order the crate enforces, in the shape of
the sibling `schemas/rpc/*.md` documents. It documents the delivered code;
it does not widen it.

## 1. The endpoint (RPC-001, 19.0.0)

| Rule | Enforced by | Refusal |
| --- | --- | --- |
| The directory is a directory reached without a symlink | `linux::check_directory` (`symlink_metadata`) | `DirectoryNotADirectory` |
| The directory is owned by the endpoint's own effective uid (root in production) | `check_directory` against `geteuid` | `DirectoryNotOwnedByEndpoint { expected_uid, found_uid }` |
| The directory mode is **exactly** `0711` (`SOCKET_DIRECTORY_MODE`): searchable, not listable, not writable by others | `check_directory` | `DirectoryMode { found }` |
| No node of any kind pre-exists at the socket path; nothing is replaced or re-moded | `Endpoint::create` (`symlink_metadata` before `bind`) | `NodeAlreadyExists` |
| The node is `helper-<uid>.sock` under the directory — flat per-user nodes (ADR-0055 decision 3) | `node_name` | — |
| The node is `0600` (`SOCKET_NODE_MODE`) and owned by the authorizing user before the endpoint is returned; on drop the endpoint removes the node it made and nothing else | `Endpoint::create` / `Drop` | `Io { operation: "set node mode" / "set node owner" }` |

Between `bind` and the mode/owner change the node briefly carries the
process umask's default; a connection admitted in that window is still
refused by the credential check below — the second gate is what makes the
first's window harmless.

## 2. Admission, in order (HLP-007: before processing any request)

1. The kernel admits or refuses the `connect` at the node: search on the
   directory, write on the `0600` node — so only the authorizing user (and
   root, which bypasses mode bits) reaches `accept`. Measured in the round:
   a `0700` directory refuses every non-root uid with `EACCES` (SI-41); a
   `0711` directory with a `0660 root:root` node refuses a non-root uid
   likewise.
2. The endpoint reads `SO_PEERCRED` through `rustix::net::sockopt::socket_peercred`
   — the connecting process's `pid/uid/gid` at connect time — as
   `PeerCredentials`.
3. `verify_peer` admits iff `uid` equals the endpoint's `AuthorizingUser`;
   otherwise `PeerNotAuthorizingUser { expected_uid, found_uid }` **before
   any byte is read** from the stream (`linux::admit`; a test proves the
   peer's bytes are still unread after the refusal). This is the verifier
   `IdentityClaim::UnixPeerCredentials` waited on; root connecting to
   another user's node is refused here.
4. The RPC-002 handshake under `Timeouts::handshake_ms`, then
   `Timeouts::request_ms` for the stream.

## 3. The frame (RPC-004)

```
+----------------+-----------------------+
| length: u32 BE | payload: length bytes |
+----------------+-----------------------+
```

- `length` ≤ `partman_rpc::MAX_MESSAGE_BYTES` (1 MiB), checked on the
  header **before any allocation**; over the bound → `FrameOverBound {
  declared, bound }`.
- A peer closing mid-frame → `FrameTruncated`.
- `write_frame` refuses a payload over the bound before writing a byte —
  one rule for both ends, so this side cannot emit what the peer refuses.
- The payload is an encoded `partman_rpc` message (`Envelope` or
  `Handshake`): the strict decode is theirs, not this crate's.
- Timeouts arrive as `Io { operation, kind }` with the kind named.

## 4. The handshake over the frame (RPC-002)

`exchange_handshake`: both sides send their `Handshake` first, then read
the peer's through `Handshake::decode` (strict), then apply
`compatible_with`; an incompatible pair refuses with the `VersionRefusal`
and its remediation, and the caller closes. No downgrade arm exists.

## 5. What is not here

No authorization vocabulary (ADR-0021); no helper behaviour or launch
(WP-L110); no journal (WP-070); no network socket type (SEC-007); no
process launch; no per-message credentials (`SCM_CREDENTIALS`, deferred by
ADR-0055 decision 4); no events stream multiplexing beyond the envelope's
own `Channel` (a consumer that wants a separate event connection opens a
second one to the same node).

## 6. Tier-1 and Tier-2 (ADR-0055 decision 6)

Tier-1 (unprivileged, every platform for the pure seams, Linux for the
sockets): a listener under a temporary directory the test owns, checked
against the running euid; a `pair` whose credentials are the test's own;
the refusal arm through `linux::admit` with injected credentials, proving
the bytes stay unread. Tier-2 (a disposable guest, root and two
unprivileged users, the `endpoint-probe` example as the instrument): a
root-owned `0711` directory, the authorizing user admitted end to end, a
second user refused by the kernel at the node (`EACCES`), root refused by
the verifier before any byte is read, a `0700` directory refused at
endpoint creation. Recorded with the increment in `docs/work-packages/WP-040.md`.
