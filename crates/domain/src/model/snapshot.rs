//! The topology snapshot's body, envelope, and typed boundary
//! (WP-010 increment 3c; MODEL-003, MODEL-005, MODEL-006, ADR-C2, CONC-004).
//!
//! The body is everything the hash covers: the schema identity (captured
//! and simulated topologies carry **two different schema strings**, so a
//! simulated topology is structurally incapable of being accepted where a
//! captured one is required — canonical-encoding §5 domain separation),
//! the CONC-004 transitional marking (body deliberately, so a transitional
//! snapshot can never be hash-equal to a stable one), and the absorbed
//! entries and validated edges as MODEL-006 sets. The envelope — capture
//! timestamp and MODEL-004 provenance — is carried beside the body and
//! never enters the bytes, which is what keeps PLAN-006 satisfiable.
//!
//! [`TopologySnapshot::from_canonical_body`] is the typed
//! decode/validate/hash boundary WP-010's codec-remediation section
//! mandates: it decodes with the strict `pce/1` decoder, parses under the
//! schema, **rebuilds the topology from the parsed content, and requires
//! the rebuilt body to reproduce the input bytes exactly** — the
//! decode-recompute rule, discharging address recomputation, collision
//! grouping, set ordering, and the edge pair table in one equality.
//! Authorization-adjacent callers use this boundary; the generic
//! `hash`/`hash_encoded` primitives prove `pce/1` canonicality only.

use std::collections::BTreeMap;
use std::fmt;

use crate::canonical::{self, Hash, Value};

use super::naming::{self, FieldParseError, NamingFields, NodeEntry};
use super::protection::{Facts, HostRange, ProtectionRefusal, StepRanges, TransportClass};
use super::provenance::PropertyObservations;
use super::topology::{Edge, EdgeKind, Topology, TopologyError};

/// Which world a snapshot describes (ADR-0019's two schema identifiers).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotKind {
    /// A discovery capture of observed hardware.
    Captured,
    /// A planner-simulated final topology (PLAN-002). Never a planning
    /// base and never accepted where PLAN-006 requires a capture — the
    /// schema string enforces that structurally.
    Simulated,
}

impl SnapshotKind {
    const fn schema(self) -> &'static str {
        match self {
            Self::Captured => "partman.topology-snapshot.captured",
            Self::Simulated => "partman.topology-snapshot.simulated",
        }
    }
}

/// The current body schema version (MODEL-003).
pub const SCHEMA_VERSION: u64 = 1;

/// The unhashed envelope carried beside a snapshot body (ADR-C2).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SnapshotEnvelope {
    /// When the capture was taken, if recorded. Envelope, or two probes of
    /// unchanged hardware could never compare equal (PLAN-006).
    pub capture_timestamp: Option<u64>,
    /// MODEL-004 provenance, keyed by the property it explains.
    pub provenance: Vec<(String, PropertyObservations)>,
}

/// A topology snapshot: hashed body content plus its envelope.
#[derive(Debug, PartialEq, Eq)]
pub struct TopologySnapshot {
    kind: SnapshotKind,
    transitional: bool,
    topology: Topology,
    facts: Facts,
    /// The unhashed envelope. Public: editing it must never move the body
    /// hash, and a test holds that property.
    pub envelope: SnapshotEnvelope,
}

impl TopologySnapshot {
    /// Assemble a snapshot from observed nodes, edges, and the
    /// evidence-contract facts the protection layer consumes.
    ///
    /// The facts are body content (increment 3f): they are verdict
    /// inputs, and ADR-0016's logic extends to them — what the verdict
    /// reads, the authorization must commit to.
    ///
    /// # Errors
    ///
    /// [`SnapshotError::Topology`] if the topology refuses construction.
    pub fn assemble(
        kind: SnapshotKind,
        transitional: bool,
        nodes: Vec<NamingFields>,
        edges: Vec<Edge>,
        facts: Facts,
    ) -> Result<Self, SnapshotError> {
        let topology = Topology::build(nodes, edges).map_err(SnapshotError::Topology)?;
        Ok(Self {
            kind,
            transitional,
            topology,
            facts,
            envelope: SnapshotEnvelope::default(),
        })
    }

    /// The snapshot's kind.
    #[must_use]
    pub const fn kind(&self) -> SnapshotKind {
        self.kind
    }

    /// Whether the capture is CONC-004-transitional.
    #[must_use]
    pub const fn transitional(&self) -> bool {
        self.transitional
    }

    /// The validated topology.
    #[must_use]
    pub const fn topology(&self) -> &Topology {
        &self.topology
    }

    /// The body-carried evidence facts.
    #[must_use]
    pub const fn facts(&self) -> &Facts {
        &self.facts
    }

    /// Whether a step constructs against this snapshot's own
    /// authenticated facts (ADR-0018, over the body's topology and
    /// facts).
    ///
    /// # Errors
    ///
    /// [`ProtectionRefusal`] naming the reached node and its verdict.
    pub fn step_constructs(
        &self,
        target: super::naming::NodeId,
        ranges: &StepRanges,
    ) -> Result<std::collections::BTreeSet<super::naming::NodeId>, ProtectionRefusal> {
        super::protection::step_constructs(&self.topology, &self.facts, target, ranges)
    }

    /// The body as a canonical value (MODEL-005's hashed side).
    ///
    /// # Errors
    ///
    /// [`SnapshotError::Encoding`] if an element cannot be encoded for
    /// set ordering — unreachable for a snapshot this module assembled.
    pub fn body_value(&self) -> Result<Value, SnapshotError> {
        let mut body = BTreeMap::new();
        body.insert(
            "schema".to_owned(),
            Value::Text(self.kind.schema().to_owned()),
        );
        body.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
        body.insert("transitional".to_owned(), Value::Bool(self.transitional));
        body.insert(
            "nodes".to_owned(),
            sorted_set(
                self.topology
                    .entries()
                    .iter()
                    .map(|entry| entry_value(entry, &self.facts))
                    .collect::<Vec<_>>(),
            )?,
        );
        body.insert(
            "edges".to_owned(),
            sorted_set(self.topology.edges().iter().map(edge_value).collect())?,
        );
        Ok(Value::Map(body))
    }

    /// The body hash (MODEL-005): SHA-256 over the body's canonical bytes.
    ///
    /// # Errors
    ///
    /// As [`Self::body_value`], plus encoding failure.
    pub fn body_hash(&self) -> Result<Hash, SnapshotError> {
        canonical::hash(&self.body_value()?).map_err(SnapshotError::Encoding)
    }

    /// The typed decode/validate boundary: rebuild a snapshot from body
    /// bytes, refusing anything the schema does not declare, and requiring
    /// the recomputed body to reproduce the input bytes exactly.
    ///
    /// The returned snapshot carries an empty envelope: envelope content
    /// never lives in body bytes.
    ///
    /// # Errors
    ///
    /// [`SnapshotSchemaError`] naming the first rule violated.
    pub fn from_canonical_body(bytes: &[u8]) -> Result<Self, SnapshotSchemaError> {
        let value = canonical::decode(bytes).map_err(SnapshotSchemaError::Codec)?;
        let Value::Map(map) = value else {
            return Err(SnapshotSchemaError::NotABodyMap);
        };
        let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
        keys.retain(|key| {
            !matches!(
                *key,
                "schema" | "schema_version" | "transitional" | "nodes" | "edges"
            )
        });
        if let Some(unknown) = keys.first() {
            return Err(SnapshotSchemaError::UnknownField {
                key: (*unknown).to_owned(),
            });
        }
        let kind = match map.get("schema") {
            Some(Value::Text(text)) if text == SnapshotKind::Captured.schema() => {
                SnapshotKind::Captured
            }
            Some(Value::Text(text)) if text == SnapshotKind::Simulated.schema() => {
                SnapshotKind::Simulated
            }
            _ => return Err(SnapshotSchemaError::WrongSchema),
        };
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == SCHEMA_VERSION => {}
            _ => return Err(SnapshotSchemaError::WrongSchemaVersion),
        }
        let transitional = match map.get("transitional") {
            Some(Value::Bool(value)) => *value,
            _ => {
                return Err(SnapshotSchemaError::MissingField {
                    key: "transitional",
                });
            }
        };
        let (nodes, facts) = parse_nodes(&map)?;
        let edges = parse_edges(&map)?;
        let rebuilt = Self::assemble(kind, transitional, nodes, edges, facts)
            .map_err(SnapshotSchemaError::Rebuild)?;
        let recomputed = rebuilt
            .body_value()
            .and_then(|body| canonical::encode(&body).map_err(SnapshotError::Encoding))
            .map_err(SnapshotSchemaError::Rebuild)?;
        if recomputed != bytes {
            return Err(SnapshotSchemaError::RecomputationMismatch);
        }
        Ok(rebuilt)
    }
}

fn sorted_set(mut elements: Vec<Value>) -> Result<Value, SnapshotError> {
    let mut keyed = Vec::with_capacity(elements.len());
    for element in elements.drain(..) {
        let bytes = canonical::encode(&element).map_err(SnapshotError::Encoding)?;
        keyed.push((bytes, element));
    }
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(Value::Array(
        keyed.into_iter().map(|(_, element)| element).collect(),
    ))
}

fn entry_value(entry: &NodeEntry, facts: &Facts) -> Value {
    let Value::Map(mut map) = (match entry {
        NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => {
            naming::fields_value(fields)
        }
    }) else {
        unreachable!("fields_value always builds a map");
    };
    if let NodeEntry::Group {
        count,
        duplicate_designator,
        ..
    } = entry
    {
        map.insert(
            "collision_count".to_owned(),
            Value::Unsigned(u64::from(*count)),
        );
        map.insert(
            "duplicate_designator".to_owned(),
            Value::Bool(*duplicate_designator),
        );
    }
    let id = entry.id();
    if let Some(extent) = facts.extents.get(&id) {
        map.insert(
            "extent_host".to_owned(),
            Value::Bytes(extent.host.as_bytes().to_vec()),
        );
        map.insert("extent_start".to_owned(), Value::Unsigned(extent.start));
        map.insert("extent_length".to_owned(), Value::Unsigned(extent.length));
    }
    if let Some(transport) = facts.transports.get(&id) {
        map.insert(
            "transport".to_owned(),
            Value::Text(transport_tag(*transport).to_owned()),
        );
    }
    if let Some(count) = facts.member_counts.get(&id) {
        map.insert("member_count".to_owned(), Value::Unsigned(*count));
    }
    Value::Map(map)
}

const fn transport_tag(transport: TransportClass) -> &'static str {
    match transport {
        TransportClass::NvmePcie => "nvme-pcie",
        TransportClass::Sata => "sata",
        TransportClass::SasDirect => "sas-direct",
        TransportClass::Usb => "usb",
        TransportClass::SdMmc => "sd-mmc",
        TransportClass::ParavirtualLocal => "paravirtual-local",
        TransportClass::RecognizedRemote => "recognized-remote",
        TransportClass::Unrecognized => "unrecognized",
    }
}

fn transport_from_tag(tag: &str) -> Option<TransportClass> {
    Some(match tag {
        "nvme-pcie" => TransportClass::NvmePcie,
        "sata" => TransportClass::Sata,
        "sas-direct" => TransportClass::SasDirect,
        "usb" => TransportClass::Usb,
        "sd-mmc" => TransportClass::SdMmc,
        "paravirtual-local" => TransportClass::ParavirtualLocal,
        "recognized-remote" => TransportClass::RecognizedRemote,
        "unrecognized" => TransportClass::Unrecognized,
        _ => return None,
    })
}

fn edge_value(edge: &Edge) -> Value {
    let mut map = BTreeMap::new();
    let kind = match edge.kind {
        EdgeKind::Containment => "containment",
        EdgeKind::Backing => "backing",
        EdgeKind::Production => "production",
        EdgeKind::HostBacking => "host-backing",
        EdgeKind::PlatformMembership => "platform-membership",
    };
    map.insert("kind".to_owned(), Value::Text(kind.to_owned()));
    map.insert(
        "source".to_owned(),
        Value::Bytes(edge.source.as_bytes().to_vec()),
    );
    map.insert(
        "target".to_owned(),
        Value::Bytes(edge.target.as_bytes().to_vec()),
    );
    Value::Map(map)
}

fn parse_nodes(
    map: &BTreeMap<String, Value>,
) -> Result<(Vec<NamingFields>, Facts), SnapshotSchemaError> {
    let Some(Value::Array(entries)) = map.get("nodes") else {
        return Err(SnapshotSchemaError::MissingField { key: "nodes" });
    };
    canonical::set::validate_array(entries, 1).map_err(SnapshotSchemaError::SetOrder)?;
    let mut nodes = Vec::new();
    let mut facts = Facts::default();
    for entry in entries {
        let Value::Map(entry_map) = entry else {
            return Err(SnapshotSchemaError::NotAnEntryMap);
        };
        let mut fields_only = entry_map.clone();
        let count = match fields_only.remove("collision_count") {
            None => 1,
            Some(Value::Unsigned(count)) if count >= 2 => count,
            Some(_) => return Err(SnapshotSchemaError::BadCollisionCount),
        };
        let flagged = fields_only.remove("duplicate_designator");
        if count == 1 && flagged.is_some() {
            return Err(SnapshotSchemaError::BadCollisionCount);
        }
        if count >= 2 && !matches!(flagged, Some(Value::Bool(_))) {
            return Err(SnapshotSchemaError::BadCollisionCount);
        }
        let extent = extract_extent(&mut fields_only)?;
        let transport = match fields_only.remove("transport") {
            None => None,
            Some(Value::Text(tag)) => Some(
                transport_from_tag(&tag)
                    .ok_or(SnapshotSchemaError::BadFact { key: "transport" })?,
            ),
            Some(_) => return Err(SnapshotSchemaError::BadFact { key: "transport" }),
        };
        let member_count = match fields_only.remove("member_count") {
            None => None,
            Some(Value::Unsigned(count)) => Some(count),
            Some(_) => {
                return Err(SnapshotSchemaError::BadFact {
                    key: "member_count",
                });
            }
        };
        let fields = naming::fields_from_map(&fields_only).map_err(SnapshotSchemaError::Field)?;
        if transport.is_some() && !matches!(fields, NamingFields::PhysicalDevice { .. }) {
            return Err(SnapshotSchemaError::MisplacedFact { key: "transport" });
        }
        if member_count.is_some() && !matches!(fields, NamingFields::Aggregate { .. }) {
            return Err(SnapshotSchemaError::MisplacedFact {
                key: "member_count",
            });
        }
        if extent.is_some()
            && matches!(
                fields,
                NamingFields::Aggregate { .. }
                    | NamingFields::MultipathNode { .. }
                    | NamingFields::EncryptionLayer { .. }
                    | NamingFields::Volume { .. }
            )
        {
            return Err(SnapshotSchemaError::MisplacedFact { key: "extent_host" });
        }
        let id = naming::derive_id(&fields).map_err(|_| SnapshotSchemaError::NotAnEntryMap)?;
        if let Some(extent) = extent {
            facts.extents.insert(id, extent);
        }
        if let Some(transport) = transport {
            facts.transports.insert(id, transport);
        }
        if let Some(count) = member_count {
            facts.member_counts.insert(id, count);
        }
        let copies = usize::try_from(count).map_err(|_| SnapshotSchemaError::BadCollisionCount)?;
        for _ in 0..copies {
            nodes.push(fields.clone());
        }
    }
    Ok((nodes, facts))
}

fn extract_extent(
    fields_only: &mut BTreeMap<String, Value>,
) -> Result<Option<HostRange>, SnapshotSchemaError> {
    let host = fields_only.remove("extent_host");
    let start = fields_only.remove("extent_start");
    let length = fields_only.remove("extent_length");
    match (host, start, length) {
        (None, None, None) => Ok(None),
        (Some(Value::Bytes(host)), Some(Value::Unsigned(start)), Some(Value::Unsigned(length))) => {
            let host = naming::id_from_bytes(&host)
                .ok_or(SnapshotSchemaError::BadFact { key: "extent_host" })?;
            Ok(Some(HostRange {
                host,
                start,
                length,
            }))
        }
        _ => Err(SnapshotSchemaError::BadFact { key: "extent_host" }),
    }
}

fn parse_edges(map: &BTreeMap<String, Value>) -> Result<Vec<Edge>, SnapshotSchemaError> {
    let Some(Value::Array(entries)) = map.get("edges") else {
        return Err(SnapshotSchemaError::MissingField { key: "edges" });
    };
    canonical::set::validate_array(entries, 1).map_err(SnapshotSchemaError::SetOrder)?;
    let mut edges = Vec::new();
    for entry in entries {
        let Value::Map(entry_map) = entry else {
            return Err(SnapshotSchemaError::NotAnEntryMap);
        };
        if entry_map.len() != 3 {
            return Err(SnapshotSchemaError::NotAnEntryMap);
        }
        let kind = match entry_map.get("kind") {
            Some(Value::Text(text)) => match text.as_str() {
                "containment" => EdgeKind::Containment,
                "backing" => EdgeKind::Backing,
                "production" => EdgeKind::Production,
                "host-backing" => EdgeKind::HostBacking,
                "platform-membership" => EdgeKind::PlatformMembership,
                _ => return Err(SnapshotSchemaError::UnknownEdgeKind),
            },
            _ => return Err(SnapshotSchemaError::UnknownEdgeKind),
        };
        let source = parse_edge_id(entry_map.get("source"))?;
        let target = parse_edge_id(entry_map.get("target"))?;
        edges.push(Edge {
            kind,
            source,
            target,
        });
    }
    Ok(edges)
}

fn parse_edge_id(value: Option<&Value>) -> Result<super::naming::NodeId, SnapshotSchemaError> {
    match value {
        Some(Value::Bytes(bytes)) => {
            naming::id_from_bytes(bytes).ok_or(SnapshotSchemaError::BadEdgeEndpoint)
        }
        _ => Err(SnapshotSchemaError::BadEdgeEndpoint),
    }
}

/// A snapshot assembly or encoding failure.
#[derive(Debug, PartialEq, Eq)]
pub enum SnapshotError {
    /// Topology construction refused (see [`TopologyError`]).
    Topology(TopologyError),
    /// The canonical encoder refused body content — unreachable for a
    /// snapshot this module assembled; surfaced rather than panicked.
    Encoding(canonical::Error),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Topology(error) => write!(formatter, "topology refused: {error}"),
            Self::Encoding(error) => write!(formatter, "body not encodable: {error}"),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// The schema-validation pass's error type — deliberately distinct from
/// the generic `pce/1` error enum, per round three's placement finding.
#[derive(Debug, PartialEq, Eq)]
pub enum SnapshotSchemaError {
    /// The bytes are not canonical `pce/1`.
    Codec(canonical::Error),
    /// The decoded value is not a body map.
    NotABodyMap,
    /// The body carries a key the schema does not declare.
    UnknownField {
        /// The undeclared key.
        key: String,
    },
    /// The schema string is neither snapshot identifier.
    WrongSchema,
    /// The schema version is not this build's.
    WrongSchemaVersion,
    /// A required field is missing or mistyped.
    MissingField {
        /// The field's key.
        key: &'static str,
    },
    /// A declared set is not in strict canonical order (MODEL-006).
    SetOrder(canonical::set::Error),
    /// A node entry is not a map, or an edge entry is malformed.
    NotAnEntryMap,
    /// A collision count below two, or group flags on a singleton.
    BadCollisionCount,
    /// A fact field with the wrong shape or an unknown tag.
    BadFact {
        /// The fact's key.
        key: &'static str,
    },
    /// A fact on a kind that does not carry it (a transport on a
    /// partition, a member count on a device, an extent on an
    /// aggregate).
    MisplacedFact {
        /// The fact's key.
        key: &'static str,
    },
    /// A node entry does not parse as naming fields.
    Field(FieldParseError),
    /// An edge kind tag this build does not know.
    UnknownEdgeKind,
    /// An edge endpoint is not a 32-byte address.
    BadEdgeEndpoint,
    /// Rebuilding the parsed content refused.
    Rebuild(SnapshotError),
    /// The rebuilt body does not reproduce the input bytes.
    RecomputationMismatch,
}

impl fmt::Display for SnapshotSchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codec(error) => write!(formatter, "not canonical pce/1: {error}"),
            Self::NotABodyMap => formatter.write_str("body is not a map"),
            Self::UnknownField { key } => write!(formatter, "undeclared body field `{key}`"),
            Self::WrongSchema => formatter.write_str("unknown snapshot schema"),
            Self::WrongSchemaVersion => formatter.write_str("unsupported schema version"),
            Self::MissingField { key } => write!(formatter, "missing body field `{key}`"),
            Self::SetOrder(error) => write!(formatter, "set order violated: {error:?}"),
            Self::NotAnEntryMap => formatter.write_str("malformed entry"),
            Self::BadCollisionCount => formatter.write_str("invalid collision-group fields"),
            Self::BadFact { key } => write!(formatter, "malformed fact `{key}`"),
            Self::MisplacedFact { key } => {
                write!(formatter, "fact `{key}` on a kind that does not carry it")
            }
            Self::Field(error) => write!(formatter, "node entry: {error}"),
            Self::UnknownEdgeKind => formatter.write_str("unknown edge kind"),
            Self::BadEdgeEndpoint => formatter.write_str("edge endpoint is not an address"),
            Self::Rebuild(error) => write!(formatter, "rebuild refused: {error}"),
            Self::RecomputationMismatch => {
                formatter.write_str("rebuilt body does not reproduce the input bytes")
            }
        }
    }
}

impl std::error::Error for SnapshotSchemaError {}
