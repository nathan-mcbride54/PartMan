# The helper-authentication skeleton

- Spec version: 11.2.0
- Requirement IDs: RPC-001
- Owner: WP-040 (`docs/work-packages/WP-040.md`)
- Underlying byte profile: none — this document records a type
  vocabulary, not a wire format. No bytes cross a connection for these
  claims; they name verifications a transport performs out-of-band,
  and any claim a future route decision makes wire-visible gets its
  own format document then.

This document records a delivered vocabulary. It decides nothing: the
types live in `crates/rpc`'s `identity` module, and the closure test
is the authority wherever a sentence could be read two ways.

## 1. The claims: what a peer proves, verified by nobody here

RPC-001 names three transports, and each implies one identity claim —
what a peer must prove it *is* before the helper processes anything
from it (HLP-007's caller-identity rule consumes this; enforcing it is
WP-070's). The vocabulary is closed at exactly these three:

| Claim | Transport | The peer proves |
| --- | --- | --- |
| `WindowsPipeSddl` | Windows named pipe | The connection was admitted by an SDDL restricting the pipe to SYSTEM and the authorizing interactive user. The authorizing user is runtime data the transport learns at endpoint creation; the claim names the restriction, not a value. |
| `UnixPeerCredentials` | Linux Unix domain socket (0700, root-owned directory) | The kernel-reported credentials of the connecting process match the peer the socket was created to serve. |
| `MacosCodeSigning` | macOS XPC, or an equivalently verified Unix domain socket | The connecting binary satisfies the helper's code-signing requirement. |

## 2. Which claim waits on which route

**No claim has a verifier in this package.** Every RPC-001 transport
is route-decision-gated — no route is simultaneously dependency-free,
`unsafe`-free, and clean against the workspace rules (the WP-035
increment-10 triangle, three times over) — and a claim's verifier
needs exactly the platform reach its route decision exists to cost
out. Each claim therefore waits, by name:

| Claim | Waits on | The reach a verifier needs |
| --- | --- | --- |
| `WindowsPipeSddl` | The Windows transport route decision (unrecorded) | Win32 security APIs to build and apply the SDDL. |
| `UnixPeerCredentials` | The Linux transport route decision (unrecorded) | `SO_PEERCRED` beyond std's surface. |
| `MacosCodeSigning` | The macOS transport route decision (unrecorded) | Platform frameworks for XPC and code-signing evaluation. |

Until a decision is recorded, the protocol layer is complete and
endpoint-less — a truthful state, not a gap — and the skeleton's
`waits_on` says so per claim rather than letting absence read as
oversight.

## 3. No authorization vocabulary (a standing decision since ADR-0021)

This section's posture began as SI-18's gate and is now permanent:
SI-18 resolved 2026-08-11 in spec 11.2.0 by ADR-0021. Authorization is
a two-tier ladder whose enforced tier the helper derives from its own
recomputed severity and flags (HLP-002); **no authorization-requirement
field enters the plan**, and a client-assertable authorization is
unrepresentable (CAP-007). This vocabulary therefore names **identity
facts only** — what a peer proves it is — and contains nothing about
what a peer may do, when a human must approve, or when an approval
expires, as a standing rule rather than a wait on the register.
HLP-003's two-tier binding is WP-070's to implement under ADR-0021,
with the helper-computed tier carried as validate-plan response data,
and the closure test pins the vocabulary to exactly the three identity
claims so anything more is a visible reviewed edit against that rule.
