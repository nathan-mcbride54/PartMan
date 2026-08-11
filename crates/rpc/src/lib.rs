//! The WP-040 RPC protocol layer's message layer (increment 1).
//!
//! Three rules carry RPC-002 through RPC-005 here, each held by
//! construction or by the strict validator rather than by convention:
//!
//! - **One validator for both ends** (RPC-003). The helper side is
//!   required to be strict; this library is the client side too, so the
//!   same decode path rejects unknown fields, mistyped values, and
//!   out-of-range sizes in both directions — laxness has nowhere to
//!   live.
//! - **Refuse, never degrade** (RPC-002). Version compatibility is a
//!   total function: compatible, or a typed refusal carrying a
//!   remediation message. There is no downgrade arm to reach.
//! - **Typed operations only** (RPC-005). A message body is the
//!   `pce/1` encoding of a `schemas/`-defined type, carried as bytes
//!   inside the envelope and bounded before anything parses
//!   (RPC-004). No field carries a path to execute, a command string,
//!   or dynamic code, and the vocabulary contains no type that could.
//!
//! The redaction boundary (SEC-006 at this edge) is the [`redaction`]
//! module's table: a schema-level classification of every field
//! position this package owns, whose allowlist — the positions that
//! may carry identifier-class bytes at all — is exactly the envelope
//! body and the resume token's execution handle, with the strict
//! validator as the mechanism holding every other position.
//!
//! The formats are documented in `schemas/rpc/envelope.md`,
//! `schemas/rpc/handshake.md`, `schemas/rpc/streams.md`, and
//! `schemas/rpc/redaction.md`, in the `schemas/domain` shape: the
//! documents record delivered formats and decide nothing.

use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};

pub mod redaction;
pub mod stream;

#[cfg(test)]
mod redaction_tests;
#[cfg(test)]
mod stream_tests;
#[cfg(test)]
mod tests;

/// The envelope's schema identity (MODEL-003).
pub const ENVELOPE_SCHEMA: &str = "partman.rpc.envelope";
/// The current envelope schema version. Version 2 added the event
/// stream's `sequence` field with per-channel presence rules — a
/// reviewed bump taken while no consumer existed, which is exactly
/// what version numbers are for.
pub const ENVELOPE_SCHEMA_VERSION: u64 = 2;
/// The handshake's schema identity (MODEL-003).
pub const HANDSHAKE_SCHEMA: &str = "partman.rpc.handshake";
/// The current handshake schema version. Version 2 constrained the
/// `build` field from free text to the build-version grammar
/// ([`is_build_version`]) — the redaction boundary's structural arm —
/// a reviewed bump taken while no consumer exists, the same posture as
/// the envelope's move to v2.
pub const HANDSHAKE_SCHEMA_VERSION: u64 = 2;
/// The protocol version the handshake negotiates. Bumped by reviewed
/// schema changes; RPC-002's compatibility rule compares exactly this.
pub const PROTOCOL_VERSION: u64 = 1;
/// RPC-004's size bound: no encoded message exceeds this, checked at
/// the decode entry before any parsing touches the bytes.
pub const MAX_MESSAGE_BYTES: usize = 1 << 20;
/// The build-version bound: a version token, not a payload.
pub const BUILD_VERSION_MAX_BYTES: usize = 64;

const fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-')
}

/// Whether `build` is a build version the handshake admits — the
/// redaction boundary's structural arm for the protocol's one
/// free-entry text position (SEC-006, `schemas/rpc/redaction.md`).
///
/// RPC-002 calls the field a build *version*, and the grammar holds it
/// to that word: `digits '.' digits '.' digits`, optionally followed
/// by one `+` or `-` and a nonempty suffix over `[A-Za-z0-9._+-]`,
/// ASCII throughout, nonempty, at most [`BUILD_VERSION_MAX_BYTES`]
/// bytes. The identifier classes that carry structure — paths, file
/// names, spaced labels, armored keys — cannot fit; what a grammar
/// cannot refuse (a token deliberately shaped like a version) is a
/// peer's schema violation, named in the schema doc rather than
/// silently accepted as preventable here.
#[must_use]
pub fn is_build_version(build: &str) -> bool {
    if build.is_empty() || build.len() > BUILD_VERSION_MAX_BYTES {
        return false;
    }
    let bytes = build.as_bytes();
    if !bytes.iter().all(|&byte| is_token_byte(byte)) {
        return false;
    }
    let mut rest = bytes;
    for expect_dot in [true, true, false] {
        let digits = rest.iter().take_while(|byte| byte.is_ascii_digit()).count();
        if digits == 0 {
            return false;
        }
        rest = &rest[digits..];
        if expect_dot {
            match rest.first() {
                Some(b'.') => rest = &rest[1..],
                _ => return false,
            }
        }
    }
    match rest.first() {
        None => true,
        Some(b'+' | b'-') => rest.len() > 1,
        Some(_) => false,
    }
}

/// The three message classes (RPC-004's stream separation, typed).
/// Sequence numbering and resume tokens for the event stream arrive in
/// increment 2; the class vocabulary is closed now so the envelope's
/// shape does not move under them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// A request awaiting exactly one response.
    Request,
    /// The response to a request.
    Response,
    /// A loss-tolerant event on the separate stream.
    Event,
}

impl Channel {
    const fn tag(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Response => "response",
            Self::Event => "event",
        }
    }

    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "request" => Some(Self::Request),
            "response" => Some(Self::Response),
            "event" => Some(Self::Event),
            _ => None,
        }
    }
}

/// One protocol message: a channel, a `pce/1`-encoded body of a
/// `schemas/`-defined type, and — on the event channel only — the
/// monotone sequence number resynchronization anchors on. Fields are
/// private; the per-channel constructors hold the presence rules, and
/// decode re-proves them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    channel: Channel,
    body: Vec<u8>,
    sequence: Option<u64>,
}

/// Why the strict validator refused — both directions, one vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeRefusal {
    /// The encoded message exceeds RPC-004's bound. Checked before any
    /// parsing touches the bytes.
    OversizedMessage {
        /// The encoded length presented.
        presented: usize,
        /// The bound it exceeds.
        bound: usize,
    },
    /// The bytes are not canonical `pce/1`.
    NotCanonical,
    /// The message is not the expected map shape.
    NotAMessage,
    /// The schema identity or version is not the expected one.
    WrongSchema,
    /// A field the schema does not declare (RPC-003's strict arm).
    UnknownField {
        /// The undeclared key.
        key: String,
    },
    /// A declared field is absent or mistyped.
    BadField {
        /// The field.
        key: &'static str,
    },
    /// The body bytes inside the envelope are not canonical `pce/1`.
    BodyNotCanonical,
    /// The sequence field's presence violates its channel rule: events
    /// carry exactly one, requests and responses carry none (RPC-004's
    /// stream separation held in the shape itself).
    SequenceMisplaced {
        /// The channel presented.
        channel: Channel,
        /// Whether a sequence was present.
        present: bool,
    },
    /// The build field is not a build version ([`is_build_version`]) —
    /// the redaction boundary's refusal, both directions. It names the
    /// rule and deliberately never echoes the presented value: a
    /// refusal that quoted the bytes would itself carry what the
    /// boundary exists to keep out (SEC-006).
    NotABuildVersion,
}

impl Envelope {
    fn wrap(channel: Channel, body: Vec<u8>, sequence: Option<u64>) -> Result<Self, DecodeRefusal> {
        if body.len() > MAX_MESSAGE_BYTES {
            return Err(DecodeRefusal::OversizedMessage {
                presented: body.len(),
                bound: MAX_MESSAGE_BYTES,
            });
        }
        if canonical::decode(&body).is_err() {
            return Err(DecodeRefusal::BodyNotCanonical);
        }
        Ok(Self {
            channel,
            body,
            sequence,
        })
    }

    /// A request message. The body is re-proved canonical, so an
    /// envelope cannot launder bytes the codec would refuse.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::BodyNotCanonical`] or
    /// [`DecodeRefusal::OversizedMessage`].
    pub fn request(body: Vec<u8>) -> Result<Self, DecodeRefusal> {
        Self::wrap(Channel::Request, body, None)
    }

    /// A response message. Same proofs as [`Envelope::request`].
    ///
    /// # Errors
    ///
    /// As [`Envelope::request`].
    pub fn response(body: Vec<u8>) -> Result<Self, DecodeRefusal> {
        Self::wrap(Channel::Response, body, None)
    }

    /// An event message carrying its monotone sequence number — the
    /// anchor a disconnected client resynchronizes from (RPC-006's
    /// protocol half; the journal it re-anchors against is WP-070's).
    ///
    /// # Errors
    ///
    /// As [`Envelope::request`].
    pub fn event(sequence: u64, body: Vec<u8>) -> Result<Self, DecodeRefusal> {
        Self::wrap(Channel::Event, body, Some(sequence))
    }

    /// The event sequence number, present exactly on the event channel.
    #[must_use]
    pub const fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    /// The message class.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// The body's canonical bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Encode the envelope to its canonical wire bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::OversizedMessage`] if the whole encoded message
    /// would exceed the bound — the bound binds the wire, not just the
    /// body.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        let mut map = BTreeMap::new();
        map.insert("schema".to_owned(), Value::Text(ENVELOPE_SCHEMA.to_owned()));
        map.insert(
            "schema_version".to_owned(),
            Value::Unsigned(ENVELOPE_SCHEMA_VERSION),
        );
        map.insert(
            "channel".to_owned(),
            Value::Text(self.channel.tag().to_owned()),
        );
        map.insert("body".to_owned(), Value::Bytes(self.body.clone()));
        if let Some(sequence) = self.sequence {
            map.insert("sequence".to_owned(), Value::Unsigned(sequence));
        }
        let bytes = canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(DecodeRefusal::OversizedMessage {
                presented: bytes.len(),
                bound: MAX_MESSAGE_BYTES,
            });
        }
        Ok(bytes)
    }

    /// The strict decode path, both directions (RPC-003): size bound
    /// first, canonical `pce/1` second, exact schema third, no unknown
    /// fields, every declared field well-typed, and the body re-proved
    /// canonical.
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
                "schema" | "schema_version" | "channel" | "body" | "sequence"
            ) {
                return Err(DecodeRefusal::UnknownField { key: key.clone() });
            }
        }
        match map.get("schema") {
            Some(Value::Text(text)) if text == ENVELOPE_SCHEMA => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == ENVELOPE_SCHEMA_VERSION => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        let channel = match map.get("channel") {
            Some(Value::Text(tag)) => {
                Channel::from_tag(tag).ok_or(DecodeRefusal::BadField { key: "channel" })?
            }
            _ => return Err(DecodeRefusal::BadField { key: "channel" }),
        };
        let body = match map.get("body") {
            Some(Value::Bytes(bytes)) => bytes.clone(),
            _ => return Err(DecodeRefusal::BadField { key: "body" }),
        };
        let sequence = match map.get("sequence") {
            None => None,
            Some(Value::Unsigned(sequence)) => Some(*sequence),
            Some(_) => return Err(DecodeRefusal::BadField { key: "sequence" }),
        };
        match (channel, sequence.is_some()) {
            (Channel::Event, true) | (Channel::Request | Channel::Response, false) => {}
            (channel, present) => {
                return Err(DecodeRefusal::SequenceMisplaced { channel, present });
            }
        }
        Self::wrap(channel, body, sequence)
    }
}

/// One side's handshake declaration (RPC-002): schema and build
/// versions, exchanged first in both directions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handshake {
    /// The protocol version this side speaks.
    pub protocol_version: u64,
    /// The build version, for the refusal message's remediation —
    /// never for compatibility logic. Held to [`is_build_version`] at
    /// encode and decode: the wire is the redaction boundary, and a
    /// build that violates the grammar crosses in neither direction.
    pub build: String,
}

/// RPC-002's refusal: incompatible versions refuse with a remediation
/// message, never degrade silently. The remediation names a build to
/// update to; a peer's build reaches it only through the strict decode
/// that held it to the build-version grammar, so the message renders
/// no free peer text (SEC-006 at the one place this crate composes
/// human-facing prose from wire data).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionRefusal {
    /// The local protocol version.
    pub local: u64,
    /// The remote protocol version.
    pub remote: u64,
    /// The remediation, stated for a human: which side is older and
    /// what to update.
    pub remediation: String,
}

impl Handshake {
    /// This build's handshake.
    #[must_use]
    pub fn local(build: &str) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            build: build.to_owned(),
        }
    }

    /// Encode the handshake to canonical bytes. The build-version rule
    /// binds this direction too — one validator for both ends, so this
    /// side cannot emit what the peer's decode would refuse.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::NotABuildVersion`] if the build violates its
    /// grammar; [`DecodeRefusal::NotCanonical`] if encoding refuses —
    /// unreachable for the flat map this builds, reported rather than
    /// panicked.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        if !is_build_version(&self.build) {
            return Err(DecodeRefusal::NotABuildVersion);
        }
        let mut map = BTreeMap::new();
        map.insert(
            "schema".to_owned(),
            Value::Text(HANDSHAKE_SCHEMA.to_owned()),
        );
        map.insert(
            "schema_version".to_owned(),
            Value::Unsigned(HANDSHAKE_SCHEMA_VERSION),
        );
        map.insert(
            "protocol_version".to_owned(),
            Value::Unsigned(self.protocol_version),
        );
        map.insert("build".to_owned(), Value::Text(self.build.clone()));
        canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)
    }

    /// The strict decode path for a peer's handshake — the same rules
    /// as the envelope's.
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
                "schema" | "schema_version" | "protocol_version" | "build"
            ) {
                return Err(DecodeRefusal::UnknownField { key: key.clone() });
            }
        }
        match map.get("schema") {
            Some(Value::Text(text)) if text == HANDSHAKE_SCHEMA => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == HANDSHAKE_SCHEMA_VERSION => {}
            _ => return Err(DecodeRefusal::WrongSchema),
        }
        let protocol_version = match map.get("protocol_version") {
            Some(Value::Unsigned(version)) => *version,
            _ => {
                return Err(DecodeRefusal::BadField {
                    key: "protocol_version",
                });
            }
        };
        let build = match map.get("build") {
            Some(Value::Text(text)) => text.clone(),
            _ => return Err(DecodeRefusal::BadField { key: "build" }),
        };
        if !is_build_version(&build) {
            return Err(DecodeRefusal::NotABuildVersion);
        }
        Ok(Self {
            protocol_version,
            build,
        })
    }

    /// RPC-002's compatibility rule, total: compatible, or a typed
    /// refusal carrying a remediation message. There is no downgrade
    /// arm to reach.
    ///
    /// # Errors
    ///
    /// [`VersionRefusal`] naming both versions and the side to update.
    pub fn compatible_with(&self, remote: &Self) -> Result<(), VersionRefusal> {
        if self.protocol_version == remote.protocol_version {
            return Ok(());
        }
        let (older, newer) = if self.protocol_version < remote.protocol_version {
            ("this side", remote.build.as_str())
        } else {
            ("the peer", self.build.as_str())
        };
        Err(VersionRefusal {
            local: self.protocol_version,
            remote: remote.protocol_version,
            remediation: format!(
                "{older} speaks an older protocol; update it to match build {newer} — \
                 versions must be equal, and nothing degrades silently"
            ),
        })
    }
}
