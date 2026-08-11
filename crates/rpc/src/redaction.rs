//! The redaction boundary (WP-040 increment 3): SEC-006's deny-floor
//! applied at the protocol edge, held the WP-035 way.
//!
//! SEC-006 names the identifier classes — device serials, paths,
//! labels, usernames, keys, and file names — and this layer's honest
//! contribution is **a schema-level rule for which protocol field
//! positions may carry identifier-class bytes at all**: an allowlist
//! that needs no knowledge of what the denied classes are, because
//! every position outside it is structurally incapable of carrying
//! them. A pinned schema constant refuses any other value; an unsigned
//! number cannot hold bytes; a closed tag vocabulary refuses anything
//! outside itself; the build field is held to RPC-002's own word for
//! it — a *version* — by a grammar the structured identifier classes
//! cannot fit. The strict validator is therefore the redaction
//! mechanism: there is no API that accepts a caller-supplied key, an
//! unknown field refuses by name, and a raw identifier planted in any
//! non-allowlisted position refuses before it can cross.
//!
//! The allowlist is exactly two positions, each with its governing
//! authority named in [`FieldRule::authority`]:
//!
//! - the envelope's `body`, which carries `schemas/`-defined operation
//!   types whose field classes are governed by their own schemas —
//!   SEC-006's floor applies there, at the schema that defines each
//!   field, not here where the bytes are opaque; and
//! - the resume token's `execution`, an opaque handle the helper
//!   mints. Nothing at this layer can verify opacity; the obligation
//!   to mint non-identifying handles is WP-070's, and the schema doc
//!   says so rather than pretending a check exists.
//!
//! What a grammar cannot do is stated rather than hidden: a bare
//! alphanumeric token deliberately shaped like a version cannot be
//! distinguished from one, so the boundary's reach is *raw*
//! identifier-class values — every exemplar class the gate test
//! plants refuses — while deliberate smuggling inside an admitted
//! alphabet remains a schema violation by the peer, named as such in
//! `schemas/rpc/redaction.md`.

/// How one protocol field position relates to identifier-class bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldClass {
    /// Pinned to a schema constant: any other value refuses
    /// [`super::DecodeRefusal::WrongSchema`]. No identifier can stand
    /// where only one value is admitted.
    PinnedConstant,
    /// An unsigned number: bytes cannot live here at all.
    Unsigned,
    /// A closed tag vocabulary: anything outside it refuses by field.
    ClosedTag,
    /// A constrained build version ([`super::is_build_version`]):
    /// digits `.` digits `.` digits, an optional `+`/`-` suffix over
    /// the token alphabet, ASCII, bounded. The identifier classes that
    /// carry structure — paths, file names, spaced labels, armored
    /// keys — cannot fit, and the refusal names the rule, never the
    /// value.
    BuildVersion,
    /// The allowlist: a position identifier-class bytes may cross,
    /// with the authority governing those bytes named on the rule.
    IdentifierCapable,
}

/// One row of the boundary table: a format, a field, its class, and —
/// for the allowlist — the authority governing what may flow there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldRule {
    /// The schema identity of the format the field belongs to.
    pub format: &'static str,
    /// The field's key.
    pub field: &'static str,
    /// The position's relation to identifier-class bytes.
    pub class: FieldClass,
    /// For [`FieldClass::IdentifierCapable`] positions, who governs
    /// the bytes; empty for positions that structurally cannot carry
    /// them.
    pub authority: &'static str,
}

const fn structural(format: &'static str, field: &'static str, class: FieldClass) -> FieldRule {
    FieldRule {
        format,
        field,
        class,
        authority: "",
    }
}

/// The complete boundary table: every field of every message format
/// this package owns, classified. The closure test pins each format's
/// field set to the validator's accepted keys as literals, so widening
/// the allowlist — or adding a field without classifying it — is a
/// visible reviewed edit that fails the suite until the table and the
/// validator agree.
pub const FIELD_RULES: [FieldRule; 13] = [
    structural(super::ENVELOPE_SCHEMA, "schema", FieldClass::PinnedConstant),
    structural(
        super::ENVELOPE_SCHEMA,
        "schema_version",
        FieldClass::PinnedConstant,
    ),
    structural(super::ENVELOPE_SCHEMA, "channel", FieldClass::ClosedTag),
    FieldRule {
        format: super::ENVELOPE_SCHEMA,
        field: "body",
        class: FieldClass::IdentifierCapable,
        authority: "the `schemas/`-defined operation type the bytes encode; SEC-006's \
                    field classes apply at the schema defining each field",
    },
    structural(super::ENVELOPE_SCHEMA, "sequence", FieldClass::Unsigned),
    structural(
        super::HANDSHAKE_SCHEMA,
        "schema",
        FieldClass::PinnedConstant,
    ),
    structural(
        super::HANDSHAKE_SCHEMA,
        "schema_version",
        FieldClass::PinnedConstant,
    ),
    structural(
        super::HANDSHAKE_SCHEMA,
        "protocol_version",
        FieldClass::Unsigned,
    ),
    structural(super::HANDSHAKE_SCHEMA, "build", FieldClass::BuildVersion),
    structural(
        super::stream::RESUME_TOKEN_SCHEMA,
        "schema",
        FieldClass::PinnedConstant,
    ),
    structural(
        super::stream::RESUME_TOKEN_SCHEMA,
        "schema_version",
        FieldClass::PinnedConstant,
    ),
    FieldRule {
        format: super::stream::RESUME_TOKEN_SCHEMA,
        field: "execution",
        class: FieldClass::IdentifierCapable,
        authority: "WP-070: the helper mints the handle and owes its opacity; nothing at \
                    this layer can verify it, and the schema doc says so",
    },
    structural(
        super::stream::RESUME_TOKEN_SCHEMA,
        "last_sequence",
        FieldClass::Unsigned,
    ),
];
