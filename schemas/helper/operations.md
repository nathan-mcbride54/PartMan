# The helper's operations (WP-L110 increments 1–2)

`partman.helper.request` v2 and `partman.helper.response` v2: the bodies
the Linux helper reads and writes inside `partman_rpc` envelopes
(`schemas/rpc/envelope.md`) over the Linux transport's frames
(`schemas/rpc/transport-linux.md`). One request per connection; the
connection closes after the response. Encoded with WP-010's `pce/1`
canonical codec; decoded strictly (RPC-003): unknown fields, unknown
operations, missing or out-of-place arguments and violated bounds refuse.

**Version 1 is retired** (MODEL-003's explicit-migration discipline):
increment 2 added the validate-plan arguments and serves the operation,
and no shipped client ever spoke v1 — the helper's only consumers to date
are its own suites and the Tier-2 instruments. A v1 request answers
`refused` (wrong schema version).

## 1. The closed operation set (HLP-001)

| Wire name | Operation | Served by build | Else answered |
| --- | --- | --- | --- |
| `status` | The helper's own state | increment 1 (now) | — |
| `enumerate` | The adapter's client contract as root — **a proposal** | increment 1 (now) | — |
| `validate-plan` | HLP-002 re-discovery, then WP-060's `plan()` over the helper's own capture | increment 2 (now) | — |
| `apply-plan` | Apply by plan hash under HLP-003's act | increment 3 | `not-yet-served { increment: 3 }` |
| `cancel` | Cancel an execution | increment 4 | `not-yet-served { increment: 4 }` |
| `resume` | Resume an execution | increment 4 | `not-yet-served { increment: 4 }` |
| `journal-query` | Query the journal | increment 4 | `not-yet-served { increment: 4 }` |

Nothing else exists: any other `operation` text — a path, a command, a
word — is `refused`, as is any field outside the vocabulary below
(RPC-005, CLI-004 at the transport layer). The set is closed by a test
that matches every variant.

## 2. Request

```
{ "schema": "partman.helper.request", "schema_version": 2, "operation": <text>, ...validate-plan arguments }
```

The validate-plan arguments exist on exactly the `validate-plan`
operation — present on any other operation they refuse as out of place,
and absent there they refuse by name:

| Field | Type | Rule |
| --- | --- | --- |
| `target_kind` | text | `physical-device` or `partition-table` — **the closed target vocabulary of this increment's capture**. The target is spelled as its ADR-0019 naming fields, never as a raw address digest: the helper derives every address itself (the recompute-at-decode discipline), and a kind outside this vocabulary — an `Aggregate` above all — has **no spelling** (SI-13's structural interim) |
| `target_serial` | bytes ≤ 256 | optional: the device's designated serial bytes, verbatim |
| `target_wwn` | bytes ≤ 256 | optional: the device's designated WWN bytes, verbatim |
| `target_total_bytes` | unsigned | the device's total size in bytes |
| `target_role` | text | required iff `target_kind` is `partition-table`: `gpt`, `mbr`, `apm`, `hybrid-mbr` — closed; out of place on a device target |
| `requested_operation` | text | one of CAP-002's fourteen kebab names (the store's spelling); anything else refuses |
| `plan_id` | bytes ≤ 64 | the client's plan identifier — correlation, never authority |
| `validity_seconds` | unsigned | PLAN-007's window: `0` takes the 24-hour default; over 604 800 (7 days) refuses |

## 3. Response

```
{ "schema": "partman.helper.response", "schema_version": 2, "outcome": <text>, ... }
```

| `outcome` | Further fields |
| --- | --- |
| `status` | `build` (RPC-002's build-version grammar), `authorizing_uid` (unsigned), `served` (array of operation names this build serves) |
| `enumeration` | `proposal` (bool, `true` — the adapter's client contract run as root, not HLP-002's re-discovery), `enumeration` (the adapter's arm: `listed`, `over-limit`, `unavailable`, `failed`), `devices` (array of `{selector, kind, transport, properties}`: the session-local selector, the kind name `plain` / `host-assembled:<kind>` / `indeterminate`, the transport class name from ADR-0018's closed list, the observed property **count** — never identifier bytes) |
| `validated` | `plan` (the plan body's canonical bytes — the helper re-planned over its own capture; nothing in it is client-authored except the plan identifier), `plan_hash` (32 bytes — what HLP-003's act names), `snapshot_hash` (32 bytes — the helper's own capture, PLAN-006's binding), `severity` (the helper-computed name: `informational`, `reversible`, `disruptive`, `data-moving`, `destructive`), `flags` (array of PLAN-004's flag names, the helper-computed union), `not_after` (unsigned, PLAN-007) |
| `validation-refused` | `arm` (`capture`, `target`, `aggregate-target`, `validity-over-maximum`, `planner`, `encoding`), `detail` (the ground, verbatim from the refusing layer — the capability engine's and the closure's refusals travel unparaphrased) |
| `not-yet-served` | `operation`, `increment` — the operation exists, this build does not serve it, and the increment that does is named; fail-closed, never a stub success |
| `refused` | `reason` — this crate's own words for the envelope or request refusal |

On a real host today every mutating `validate-plan` answers
`validation-refused` at the capability arm: the capture's transport class
is `Unrecognized` on every device (ADR-0018's fabric-versus-local rows
are outstanding, and privilege changes nothing about that), so the
device's own protection arm is `Indeterminate` and the engine refuses.
That is the fail-closed answer, stated here so nobody reads it as a
defect.

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
`refused`), `captured devices classified` (HLP-002's capture — counts
only), `idle-exit idle_seconds`. No field can carry a serial, device
path, label or username (SAFE-006 by construction; held by a test).

## 6. What is not here

No authorization vocabulary and no tier field (ADR-0021, CAP-007; the
computed tier enters the validate-plan response in increment 3, as
response data and never plan body); no apply, no journal, no network; no
write — the only device access behind any of this is the byte layer's
two bounded read-only windows per device. `SEC-002`'s admission arms
(replayed, cross-user, altered, stale, cross-device, expired) are
delivered as the typed checking function increment 3's apply consumes;
no wire operation exposes them yet.
