//! Streams and reattach vocabulary (WP-040 increment 2): RPC-004's
//! loss-tolerant event stream and RPC-006's protocol half, pure.
//!
//! Events are loss-tolerant **by design**, which means loss is
//! detected, classified, and recovered from — never papered over. The
//! producer's sequence is monotone from 1 with no gaps
//! ([`EventSequencer`]); the consumer classifies every arrival against
//! its last seen number ([`classify`]): in order, a duplicate or replay
//! (already seen, discard), or a gap — and a gap's recovery is
//! **resynchronization from the journal**, which is WP-070's to
//! provide. This layer ships the anchor ([`ResumeToken`]) and the
//! classification; it ships nothing that pretends to replay.
//!
//! Timeouts are typed configuration the consumer supplies
//! ([`Timeouts`]): this layer has no clock, exactly like the planner,
//! so a timeout here is vocabulary for the caller that enforces it.

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};

use super::{DecodeRefusal, MAX_MESSAGE_BYTES};

/// The resume token's schema identity (MODEL-003).
pub const RESUME_TOKEN_SCHEMA: &str = "partman.rpc.resume-token";
/// The current resume token schema version.
pub const RESUME_TOKEN_SCHEMA_VERSION: u64 = 1;

/// The producer's monotone event sequence: starts at 1, increments by
/// exactly 1, and cannot be constructed mid-stream — a fresh stream
/// starts fresh, and reattachment is the *consumer's* re-anchoring,
/// not the producer renumbering.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct EventSequencer {
    last: u64,
}

impl EventSequencer {
    /// A fresh stream's sequencer.
    #[must_use]
    pub const fn new() -> Self {
        Self { last: 0 }
    }

    /// The next sequence number: monotone, gap-free, starting at 1.
    pub const fn next(&mut self) -> u64 {
        self.last += 1;
        self.last
    }

    /// The last number issued, zero before the first.
    #[must_use]
    pub const fn last(&self) -> u64 {
        self.last
    }
}

/// The consumer-side classification of one arriving event against the
/// last sequence number seen (zero before the first).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrival {
    /// Exactly the next number: process it.
    InOrder,
    /// At or before the last seen: already processed, discard — replay
    /// after reattach is expected and harmless.
    AlreadySeen,
    /// Beyond the next number: events were lost. The classification
    /// names the missing closed range, and recovery is
    /// resynchronization from the journal (WP-070's) — never guessing,
    /// never skipping silently.
    Gap {
        /// The first missing sequence number.
        from: u64,
        /// The last missing sequence number.
        to: u64,
    },
}

/// Classify an arriving event's sequence number against the last seen.
#[must_use]
pub const fn classify(last_seen: u64, arrived: u64) -> Arrival {
    if arrived <= last_seen {
        Arrival::AlreadySeen
    } else if arrived == last_seen + 1 {
        Arrival::InOrder
    } else {
        Arrival::Gap {
            from: last_seen + 1,
            to: arrived - 1,
        }
    }
}

/// The reattach anchor (RPC-006's protocol half): which execution, and
/// the last event sequence number the client processed. What the client
/// re-anchors *against* is the journal plus event replay — WP-070's to
/// provide; this token is the protocol's statement of where to anchor,
/// nothing more.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumeToken {
    /// The helper-assigned execution identifier, opaque bytes.
    pub execution: Vec<u8>,
    /// The last sequence number the client processed; zero if none.
    pub last_sequence: u64,
}

impl ResumeToken {
    /// Encode the token to canonical bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::NotCanonical`] if encoding refuses —
    /// unreachable for this flat map, reported rather than panicked.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        let mut map = BTreeMap::new();
        map.insert(
            "schema".to_owned(),
            Value::Text(RESUME_TOKEN_SCHEMA.to_owned()),
        );
        map.insert(
            "schema_version".to_owned(),
            Value::Unsigned(RESUME_TOKEN_SCHEMA_VERSION),
        );
        map.insert("execution".to_owned(), Value::Bytes(self.execution.clone()));
        map.insert(
            "last_sequence".to_owned(),
            Value::Unsigned(self.last_sequence),
        );
        canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)
    }

    /// The strict decode path — the same rules as every message here,
    /// the size bound first: the token also travels standalone, so it
    /// cannot borrow the envelope's gate.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal`], the first rule violated.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeRefusal> {
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(DecodeRefusal::OversizedMessage {
                presented: bytes.len(),
                bound: MAX_MESSAGE_BYTES,
            });
        }
        let value = canonical::decode(bytes).map_err(|_| DecodeRefusal::NotCanonical)?;
        let Value::Map(map) = value else {
            return Err(DecodeRefusal::NotAMessage);
        };
        for key in map.keys() {
            if !matches!(
                key.as_str(),
                "schema" | "schema_version" | "execution" | "last_sequence"
            ) {
                return Err(DecodeRefusal::UnknownField { key: key.clone() });
            }
        }
        match map.get("schema") {
            Some(Value::Text(text)) if text == RESUME_TOKEN_SCHEMA => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == RESUME_TOKEN_SCHEMA_VERSION => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        let execution = match map.get("execution") {
            Some(Value::Bytes(bytes)) => bytes.clone(),
            _ => return Err(DecodeRefusal::BadField { key: "execution" }),
        };
        let last_sequence = match map.get("last_sequence") {
            Some(Value::Unsigned(sequence)) => *sequence,
            _ => {
                return Err(DecodeRefusal::BadField {
                    key: "last_sequence",
                });
            }
        };
        Ok(Self {
            execution,
            last_sequence,
        })
    }
}

/// Timeout vocabulary, as typed configuration the consumer supplies and
/// enforces — this layer has no clock. RPC-004 requires messages to
/// have timeouts; the values are deployment policy, and a pure protocol
/// library's honest contribution is the type that carries them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timeouts {
    /// Milliseconds a request may await its response.
    pub request_ms: u64,
    /// Milliseconds the handshake exchange may take.
    pub handshake_ms: u64,
}
