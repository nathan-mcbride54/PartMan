# The helper's operations (WP-L110 increment 1)

`partman.helper.request` v1 and `partman.helper.response` v1: the bodies
the Linux helper reads and writes inside `partman_rpc` envelopes
(`schemas/rpc/envelope.md`) over the Linux transport's frames
(`schemas/rpc/transport-linux.md`). One request per connection in this
increment; the connection closes after the response. Encoded with
WP-010's `pce/1` canonical codec; decoded strictly (RPC-003): unknown
fields and unknown operations refuse.

## 1. The closed operation set (HLP-001)

| Wire name | Operation | Served by build | Else answered |
| --- | --- | --- | --- |
| `status` | The helper's own state | increment 1 (now) | — |
| `enumerate` | The adapter's client contract as root — **a proposal** | increment 1 (now) | — |
| `validate-plan` | HLP-002 re-discovery and validation | increment 2 | `not-yet-served { increment: 2 }` |
| `apply-plan` | Apply by plan hash under HLP-003's act | increment 3 | `not-yet-served { increment: 3 }` |
| `cancel` | Cancel an execution | increment 4 | `not-yet-served { increment: 4 }` |
| `resume` | Resume an execution | increment 4 | `not-yet-served { increment: 4 }` |
| `journal-query` | Query the journal | increment 4 | `not-yet-served { increment: 4 }` |

Nothing else exists: any other `operation` text — a path, a command, a
word — is `refused`, as is any field outside `schema`, `schema_version`,
`operation` (RPC-005, CLI-004 at the transport layer). The set is closed
by a test that matches every variant.

## 2. Request

```
{ "schema": "partman.helper.request", "schema_version": 1, "operation": <text> }
```

## 3. Response

```
{ "schema": "partman.helper.response", "schema_version": 1, "outcome": <text>, ... }
```

| `outcome` | Further fields |
| --- | --- |
| `status` | `build` (RPC-002's build-version grammar), `authorizing_uid` (unsigned), `served` (array of operation names this build serves) |
| `enumeration` | `proposal` (bool, `true` — the adapter's client contract run as root, not HLP-002's re-discovery), `enumeration` (the adapter's arm: `listed`, `over-limit`, `unavailable`, `failed`), `devices` (array of `{selector, kind, transport, properties}`: the session-local selector, the kind name `plain` / `host-assembled:<kind>` / `indeterminate`, the transport class name from ADR-0018's closed list, the observed property **count** — never identifier bytes; those stay in the adapter's observation set for increment 2's snapshot body) |
| `not-yet-served` | `operation`, `increment` — the operation exists, this build does not serve it, and the increment that does is named; fail-closed, never a stub success |
| `refused` | `reason` — this crate's own words for the envelope or request refusal |

## 4. Launch (the launch round, L2)

`pkexec /usr/libexec/partman/partman-helper-linux --serve <uid>` under the
polkit action `org.partman.helper.serve` (`services/helper-linux/polkit/`,
`allow_active` yes — nothing asked to start; installed by `packaging/`).
The helper refuses unless `PKEXEC_UID` equals `<uid>`; ensures the
runtime directory (`/run/partman`, `0711`, root); if a node for the uid
already exists it exits 0 saying so (a second launch connects to the first);
otherwise creates the `0600` node through the transport's endpoint, serves
one request per connection, and exits after `--idle-seconds` (default 120)
without a connection, removing the node it made (HLP-005).

## 5. Audit (HLP-006, SEC-009)

One line per event, `ts=<secs> event=<name> <k=v>…`, appended to `--audit
<file>` (`0600`) or stderr. The vocabulary is closed: `started uid`,
`admitted uid pid`, `connection-refused reason` (the transport's typed
refusal), `operation name outcome` (`served` / `not-yet-served` /
`refused`), `idle-exit idle_seconds`. No field can carry a serial, device
path, label or username (SAFE-006 by construction; held by a test).

## 6. What is not here

No authorization vocabulary (ADR-0021; increment 3); no device read
(increment 2); no tool; no journal; no network; no second schema version
until a later increment adds fields, which bumps `schema_version` (MODEL-003).
