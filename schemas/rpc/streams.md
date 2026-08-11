# The event stream, sequencing, and reattach anchor

- Spec version: 11.1.0
- Requirement IDs: RPC-004, RPC-006, MODEL-003
- Owner: WP-040 (`docs/work-packages/WP-040.md`)
- Underlying byte profile: `pce/1` (unchanged)

This document records delivered formats and rules. It decides nothing.

## 1. Stream separation (RPC-004)

Progress and events flow on the `event` channel, separate from
`request`/`response`. The separation is held in the envelope's shape
(`schemas/rpc/envelope.md` §1): an event carries exactly one `sequence`
field, a request or response carries none, and a violation refuses
naming the channel and the presence found.

## 2. Sequencing and loss tolerance

The producer's sequence is monotone from 1 with no gaps. The consumer
classifies every arrival against the last sequence number it processed
(zero before the first):

| Arrival | Meaning | Action |
| --- | --- | --- |
| exactly last + 1 | in order | process |
| at or before last | replay or duplicate | discard — replay after reattach is expected and harmless |
| beyond last + 1 | events were lost | the classification names the missing closed range; recover by resynchronizing from the journal |

Loss-tolerant means loss is **detected, classified, and recovered
from** — never papered over. The journal a client resynchronizes from
is WP-070's; this layer ships the anchor and the classification, and
nothing that pretends to replay.

## 3. The resume token (RPC-006's protocol half)

`partman.rpc.resume-token` version 1:

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.rpc.resume-token`. |
| `schema_version` | Unsigned | `1`. |
| `execution` | Bytes | The helper-assigned execution identifier, opaque. |
| `last_sequence` | Unsigned | The last event sequence the client processed; zero if none. |

Strict decode, as everywhere: unknown fields refuse by name. The token
is the protocol's statement of where to anchor; what a client
reconstructs from is journal plus event replay, and reattach *behavior*
binds when the journal exists.

## 4. Timeouts (RPC-004)

Timeout values are typed configuration the consumer supplies and
enforces (`Timeouts { request_ms, handshake_ms }`): this pure layer has
no clock, so its honest contribution is the vocabulary, and the values
are deployment policy at the surfaces that own connections.
