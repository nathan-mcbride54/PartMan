# The helper's operations (WP-L110 increments 1–4a)

`partman.helper.request` v4 and `partman.helper.response` v4: the bodies
the Linux helper reads and writes inside `partman_rpc` envelopes
(`schemas/rpc/envelope.md`) over the Linux transport's frames
(`schemas/rpc/transport-linux.md`). One request per connection; the
connection closes after the response. Encoded with WP-010's `pce/1`
canonical codec; decoded strictly (RPC-003): unknown fields, unknown
operations, missing or out-of-place arguments and violated bounds refuse.

**Versions 1–3 are retired** (MODEL-003's explicit-migration
discipline): increment 2 added the validate-plan arguments, increment 3
added the computed authorization tier to the validated response,
increment 4a added the apply-plan argument and the journal-borne
outcomes — and no shipped client ever spoke any of them: the helper's
only consumers to date are its own suites and the Tier-2 instruments.
An older request is `refused` with a **remediation naming the version
this build speaks** (RPC-002: never degrade silently), not a debug
rendering.

**No request field carries an authorization tier, still, and that is
load-bearing**: a tier a client could name is exactly what CAP-007
makes unrepresentable. v4's one new request field is a plan hash — the
identity the helper itself computed and returned at validation, and
nothing a client could assert with: the body, the window, the tier and
the user all come from the helper's own journal and store.

## 1. The closed operation set (HLP-001)

| Wire name | Operation | Served by build | Else answered |
| --- | --- | --- | --- |
| `status` | The helper's own state | increment 1 (now) | — |
| `enumerate` | The adapter's client contract as root — **a proposal** | increment 1 (now) | — |
| `validate-plan` | HLP-002 re-discovery, then WP-060's `plan()` over the helper's own capture; journaled since 4a | increment 2 (now) | — |
| `apply-plan` | The two-phase apply (S2), to the authorization boundary | increment 4a (now) | — |
| `cancel` | Cancel an execution | increment 4 (its 4b half) | `not-yet-served { increment: 4 }` |
| `resume` | Resume an execution | increment 4 (its 4b half) | `not-yet-served { increment: 4 }` |
| `journal-query` | Query the journal | increment 4a (now) | — |

Cancel and resume move with everything past `AuthorizationGranted` to
increment 4's 4b half (the shape round,
`docs/reviews/WP-L110_INCREMENT_4_ROUND_2026-08-20.md` §9): a cancel of
an apply that can never execute would journal edges this build cannot
honestly take.

Nothing else exists: any other `operation` text — a path, a command, a
word — is `refused`, as is any field outside the vocabulary below
(RPC-005, CLI-004 at the transport layer). The set is closed by a test
that matches every variant.

## 2. Request

```
{ "schema": "partman.helper.request", "schema_version": 4, "operation": <text>, ...arguments }
```

The apply-plan argument exists on exactly the `apply-plan` operation —
out of place anywhere else, refused by name when absent:

| Field | Type | Rule |
| --- | --- | --- |
| `plan_hash` | bytes = 32 | the plan body hash the helper returned at validation — exactly 32 bytes, anything else refuses. An apply names its plan by hash and nothing else |

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
{ "schema": "partman.helper.response", "schema_version": 4, "outcome": <text>, ... }
```

| `outcome` | Further fields |
| --- | --- |
| `status` | `build` (RPC-002's build-version grammar), `authorizing_uid` (unsigned), `served` (array of operation names this build serves) |
| `enumeration` | `proposal` (bool, `true` — the adapter's client contract run as root, not HLP-002's re-discovery), `enumeration` (the adapter's arm: `listed`, `over-limit`, `unavailable`, `failed`), `devices` (array of `{selector, kind, transport, properties}`: the session-local selector, the kind name `plain` / `host-assembled:<kind>` / `indeterminate`, the transport class name from ADR-0018's closed list, the observed property **count** — never identifier bytes) |
| `validated` | `plan` (the plan body's canonical bytes — the helper re-planned over its own capture; nothing in it is client-authored except the plan identifier), `plan_hash` (32 bytes — what HLP-003's act names), `snapshot_hash` (32 bytes — the helper's own capture, PLAN-006's binding), `severity` (the helper-computed name: `informational`, `reversible`, `disruptive`, `data-moving`, `destructive`), `flags` (array of PLAN-004's flag names, the helper-computed union), **`tier`** (v3: the helper-computed authorization tier, `floor-act` or `interactive-ceremony` — HLP-003's one authorized reporting site and UI-011's reason for it; **response data, never plan body**, so MODEL-005's authoring set stays closed at two), `not_after` (unsigned, PLAN-007) |
| `validation-refused` | `arm` (`capture`, `target`, `aggregate-target`, `validity-over-maximum`, `planner`, `encoding`; since 4a also `clock-behind-journal`, `journal-decode`, `durability`), `detail` (the ground, verbatim from the refusing layer — the capability engine's and the closure's refusals travel unparaphrased) |
| `awaiting-authorization` | 4a, S2's phase one: `plan_hash` (echoed for correlation), `tier` (`floor-act` or `interactive-ceremony` — what the authorization will require), `not_after` (past it the apply terminates on the published `DeclinedOrExpired` edge). `ApplySubmitted` is journaled durably **before** this answer leaves; a second `apply-plan` for the same hash is the completion request |
| `apply-refused` | 4a: `arm` (`clock`, `clock-behind-journal`, `journal-decode`, `chain-broken`, `not-validated`, `replayed`, `cross-user`, `hash-mismatch`, `stale`, `cross-device`, `altered`, `expired`, `declined-or-expired`, `ceremony-unavailable`, `grant-not-served`, `beyond-authorization`, `validation-store`, `durability`, `capture`, `audit`, `runtime`), `detail` (the helper's own words; no client content echoed). `stale` journals `EditOrInvalidation` (CONC-003's published edge); `declined-or-expired` journals the published terminal, `NoWrites` |
| `journal` | 4a, journal-query's answer: `high_water_instant` (absent on an empty journal), `records` (count), `plans` (array of `{plan_hash, state, instant}` — the last journaled state under Section 8's own names, all helper-authored) |
| `not-yet-served` | `operation`, `increment` — the operation exists, this build does not serve it, and the increment that does is named; fail-closed, never a stub success |
| `refused` | `reason` — this crate's own words for the envelope or request refusal |

On a real host today every mutating `validate-plan` answers
`validation-refused` at the capability arm: the capture's transport class
is `Unrecognized` on every device (ADR-0018's fabric-versus-local rows
are outstanding, and privilege changes nothing about that), so the
device's own protection arm is `Indeterminate` and the engine refuses.
That is the fail-closed answer, stated here so nobody reads it as a
defect.

## 4. The journal's on-disk home and the validation store (increment 4a)

JRN-004's location clause, discharged for Linux: the state directory is
`/var/lib/partman` (root-owned `0700`, overridable with
`--state-directory`), holding **one pair of files per authorizing uid**
— `journal-<uid>.log` (the Section 8 journal,
`schemas/journal/records.md` v2 payloads in `schemas/journal/framing.md`
frames) and `validations-<uid>.log` (the validation store), both `0600`.
One helper per uid means one writer per file, which is what keeps 4a
honest before CONC-001's locking lands in 4b. Both files go through the
helper's real durability seam — append then `fsync` — and **no answer
leaves ahead of its record**; a torn tail truncates at recovery
(JRN-001) and the truncation is made physical before appending resumes.

The validation store is a sibling append-only log in ADR-0030's
sibling-store shape (never inside the journal: the journal's record
vocabulary is closed, and plan bodies are bulk its budget must not
carry). Its entries are `partman.helper.validation` version 1, each one
`pce/1` map:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | Text | always `partman.helper.validation` |
| `schema_version` | Unsigned | always `1` |
| `kind` | Text | `recorded` or `consumed` |
| `plan_hash` | Bytes(32) | the validated plan's body hash |
| `uid` | Unsigned, `recorded` only | the RPC-001-authenticated user the validation answered |
| `tier` | Text, `recorded` only | the helper-computed tier, written at validation (HLP-003) |
| `not_after` | Unsigned, `recorded` only | PLAN-007's window end |
| `body` | Bytes, `recorded` only | the plan body's canonical bytes — what SEC-002's arms re-check at presentation; a client presents a hash, never bytes |

Consumption is an appended `consumed` entry, never a flipped bit, so a
spent validation refuses re-presentation across a restart. **This store
holds plan bodies, which carry naming fields**: it is not a log in
SEC-006's sense — it is the helper's own working state, root-owned like
the protection-artifact store, and nothing from it is echoed onto a
refusal or an audit line.

**The backward-clock bound**: `ValidatorPasses` is journaled at
validation carrying schema v2's recorded instant, so the journal's
high-water instant is populated from the first validation onward, and
any operation whose clock reads below it refuses
(`clock-behind-journal`) — the monotonicity debt `clock.rs` recorded,
paid with the journal's own mark, covering exactly the
validation-to-presentation window it named.

## 5. Launch (the launch round, L2)

`pkexec /usr/libexec/partman/partman-helper-linux --serve <uid>` under the
polkit action `org.partman.helper.serve` (`services/helper-linux/polkit/`,
`allow_active` yes — nothing asked to start; installed by `packaging/`).
The helper refuses unless `PKEXEC_UID` equals `<uid>`; ensures the
runtime directory (`/run/partman`, `0711`, root); if a node for the uid
already exists it exits 0 saying so (a second launch connects to the first);
otherwise creates the `0600` node through the transport's endpoint, serves
one request per connection, and exits after `--idle-seconds` (default 120)
without a connection, removing the node it made (HLP-005).

## 6. Audit (HLP-006, SEC-009)

One line per event, `ts=<secs> event=<name> <k=v>…`, appended to `--audit
<file>` (`0600`) or stderr. The vocabulary is closed: `started uid`,
`admitted uid pid`, `connection-refused reason` (the transport's typed
refusal), `operation name outcome` (`served` / `not-yet-served` /
`refused`), `captured devices classified` (HLP-002's capture — counts
only), **`authorization tier outcome`** (v3: the computed tier's own wire
name and this build's single outcome word `computed` — **no plan hash**,
because a 64-character digest in a log line is an identifier by another
name), **`journaled transition`** (4a: the appended transition's wire tag
from the journal schema's closed 23-member vocabulary — no plan hash,
same reason), `idle-exit idle_seconds`. No field can carry a serial,
device path, label or username (SAFE-006 by construction; held by a
test).

**Fail-closed since increment 3:** a line that cannot be written refuses
the operation. An operation served without its record would make SEC-009
a wish; increments 1–2 discarded the write's result, and that is fixed.

## 7. What is not here

**Nothing past the authorization boundary.** 4a's `apply-plan` runs S2's
two phases and stops at `AwaitingAuthorization`: phase two refuses
exactly where increment 3 refuses (the interactive ceremony's own arm —
R8's seam still ships refusing, and even a completed ceremony reaches no
grant, `grant-not-served`), and a closed window terminates on the
published `DeclinedOrExpired → Cancelled` edge. The
`AuthorizationGranted` edge and everything after it — Revalidating,
Protecting/PART-013, Executing, the table writer, CONC-001's locking,
cancel and resume — are increment 4b's, behind the toolset and
launcher-home rounds (the shape round §9).

**No interactive authorization on any route.** The apply-ceremony round
(`docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md`) took R8: the
ceremony is a seam whose completion value is unconstructible in a shipped
build, so a plan whose computed tier is `interactive-ceremony` cannot be
authorized here at all. A client sees the tier on its validated response
and, at apply time, sees the refusal — named without reporting any host
fact, because a refusal that distinguished "no route decided" from
"polkit absent" would tell an unprivileged caller about the host.

No bus client, no tool launch, no polkit action shipped, no network, and
**no write** — the only device access behind any of this remains the
byte layer's two bounded read-only windows per device. SEC-002's
admission arms have their production caller since 4a: phase one checks
the stored body against the fresh capture, the presenting peer, the
helper's clock and the durable record.
