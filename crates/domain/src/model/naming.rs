//! Node naming per ADR-0019 (WP-010 increment 3a).
//!
//! A node's identifier is a **derived, document-local address**: the SHA-256
//! digest of a canonical `pce/1` value built from the node's own naming
//! fields, one of which is its parent's identifier — a positional address
//! rooted at physical devices. Identifiers are never stored; a decoder
//! recomputes them and rejects unknown referents (that boundary lands with
//! the snapshot schema in a later slice).
//!
//! Two rules from ADR-0019 are load-bearing here:
//!
//! - **Canonicalization by source, not transformation.** Identifier bytes
//!   (serial, WWN, designators) are the verbatim bytes of the one source the
//!   evidence contract names per platform. This module never folds case,
//!   strips prefixes, or re-encodes; callers hand it contract bytes.
//! - **A collision produces an artifact.** Same-kind nodes deriving equal
//!   addresses are absorbed — before any encoding — into a counted, flagged
//!   collision group ([`NodeEntry::Group`]). Absorption is total, so no
//!   on-disk byte content can make a node set unrepresentable, which is the
//!   register's governing finding ("fail-closed-by-unencodability is not
//!   fail-closed") discharged at the naming layer.

use std::collections::BTreeMap;
use std::fmt;

use crate::canonical::{self, Hash, Value};

/// A derived, document-local node address (ADR-0019).
///
/// Addresses are not identities: SAFE-003's identity record carries device
/// identity separately, with its own strength and match rules. An address
/// exists so edges in one body can reference nodes in that same body.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(Hash);

impl NodeId {
    /// The address digest's raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// The table scheme a table node or table view belongs to.
///
/// Every closed enum over externally observed values carries an explicit
/// unrecognized variant (MODEL-002, ADR-C5); the raw discriminant bytes ride
/// along so two distinct unrecognized schemes never share an address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableRole {
    /// A GUID partition table view.
    Gpt,
    /// A master boot record view.
    Mbr,
    /// An Apple Partition Map view.
    Apm,
    /// The non-protective MBR view beside a valid GPT (a hybrid table's
    /// second description of the same bytes).
    HybridMbr,
    /// A scheme this build does not recognize, carrying the platform's raw
    /// discriminant bytes verbatim.
    Unrecognized {
        /// The reporting interface's own discriminant bytes.
        raw: Vec<u8>,
    },
}

/// A non-file-system signature family the byte layer parses (ADR-0018).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureFamily {
    /// A ZFS vdev label (any of its four positions).
    Zfs,
    /// An mdraid 0.90 superblock (end-anchored).
    Mdraid09,
    /// An mdraid 1.x superblock.
    Mdraid1x,
    /// A LUKS1 header.
    Luks1,
    /// A LUKS2 header (either copy).
    Luks2,
    /// An LVM2 label and metadata area.
    Lvm2,
    /// A Storage Spaces pool marker.
    StorageSpaces,
    /// An LDM (dynamic disk) marker.
    Ldm,
    /// `BitLocker` volume metadata.
    BitLocker,
    /// An APFS container superblock acting as aggregation evidence.
    ApfsContainer,
    /// A family this build does not recognize, carrying its raw magic bytes.
    Unrecognized {
        /// The parser's raw family discriminant bytes.
        raw: Vec<u8>,
    },
}

/// A file-system kind (FS-004's detection list).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileSystemKind {
    /// ext2.
    Ext2,
    /// ext3.
    Ext3,
    /// ext4.
    Ext4,
    /// Btrfs (single- or multi-device; cardinality does not change shape).
    Btrfs,
    /// XFS.
    Xfs,
    /// F2FS.
    F2fs,
    /// FAT12.
    Fat12,
    /// FAT16.
    Fat16,
    /// FAT32.
    Fat32,
    /// exFAT.
    Exfat,
    /// NTFS.
    Ntfs,
    /// `ReFS`.
    Refs,
    /// HFS+.
    HfsPlus,
    /// An APFS volume's file system.
    Apfs,
    /// UDF.
    Udf,
    /// Linux swap.
    Swap,
    /// A kind this build does not recognize, carrying its raw probe bytes.
    Unrecognized {
        /// The prober's raw kind discriminant bytes.
        raw: Vec<u8>,
    },
}

/// An aggregation technology (ADR-C5's closed discriminant).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AggregateTechnology {
    /// LVM2 volume group.
    Lvm2,
    /// mdraid array.
    Mdraid,
    /// Windows Storage Spaces pool.
    StorageSpaces,
    /// ZFS pool.
    Zfs,
    /// APFS container (Fusion is the member-count-2 instance).
    Apfs,
    /// Windows dynamic-disk (LDM) group.
    Ldm,
    /// A technology this build does not recognize.
    Unrecognized {
        /// The reporting interface's raw technology bytes.
        raw: Vec<u8>,
    },
}

/// Where a host-backed virtual device's bytes live (ADR-0019's
/// `BackingExtent`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtentLocator {
    /// A file within the host file system, by canonicalized path bytes —
    /// the platform-reported backing path verbatim, not re-encoded.
    Path {
        /// The backing path's bytes as the named platform source reports
        /// them.
        bytes: Vec<u8>,
    },
    /// A byte range within the host node's own address space.
    Range {
        /// First byte of the range.
        start: u64,
        /// Length of the range in bytes.
        length: u64,
    },
}

/// A node's naming fields — exactly ADR-0019's per-kind naming maps.
///
/// Everything on the exclusion list stays out: connection path, OS instance
/// id, positional indices, table state and checksum, length, regenerable
/// identifiers, partition type, adapter-formatted text, and both sector
/// sizes. Identifier bytes are contract-source-verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamingFields {
    /// A physical device: canonicalized stable identifiers and total bytes,
    /// nothing else.
    PhysicalDevice {
        /// Serial bytes from the contract's named source, or absent.
        serial: Option<Vec<u8>>,
        /// WWN bytes from the contract's named source, or absent.
        wwn: Option<Vec<u8>>,
        /// Total device size in bytes (MODEL-001).
        total_bytes: u64,
    },
    /// A partition table, one per device, at MODEL-002's chain position.
    PartitionTable {
        /// The device the table describes.
        parent: NodeId,
        /// The table's scheme role.
        role: TableRole,
    },
    /// A partition, re-parented onto its table (ADR-0019): the role
    /// discriminant on the parent is what keeps a hybrid view's aliased
    /// entry a distinct address.
    Partition {
        /// The table view the entry belongs to.
        parent_table: NodeId,
        /// The entry's start offset in the containment root's address space.
        start_offset: u64,
    },
    /// A non-file-system signature (FS-004, materialized per ADR-C5).
    BackingSignature {
        /// The node whose bytes carry the signature.
        host: NodeId,
        /// The parsed family.
        family: SignatureFamily,
        /// The primary signature offset the parser fixed.
        primary_offset: u64,
    },
    /// A file system.
    FileSystem {
        /// The node whose bytes carry the file system.
        host: NodeId,
        /// The parsed kind.
        kind: FileSystemKind,
        /// The primary superblock offset the parser fixed.
        superblock_offset: u64,
    },
    /// An encryption layer, named from the signature that evidences it.
    EncryptionLayer {
        /// The LUKS/BitLocker signature node.
        backing_signature: NodeId,
    },
    /// An aggregate, named from its technology's own designator — never
    /// from its members (the withdrawn round-three rule stays withdrawn).
    Aggregate {
        /// The aggregation technology.
        technology: AggregateTechnology,
        /// The native designator's bytes from the contract's named source,
        /// or absent — a designator-absent aggregate is representable and
        /// is an `Indeterminate` non-operand under ADR-0018's closure.
        designator: Option<Vec<u8>>,
    },
    /// A volume or produced virtual device, named from its producer and the
    /// technology's own name — never a regenerable UUID.
    Volume {
        /// The producing aggregate or encryption layer.
        producer: NodeId,
        /// The technology's own volume name bytes.
        name: Vec<u8>,
        /// The technology's role bytes, where it defines one.
        role: Option<Vec<u8>>,
    },
    /// The file or byte range carrying a host-backed virtual device's bytes.
    BackingExtent {
        /// The file system or node hosting the extent.
        host: NodeId,
        /// Where within the host.
        locator: ExtentLocator,
    },
    /// A platform-assembled multipath node.
    MultipathNode {
        /// The platform-reported LUN designator bytes, contract-verbatim.
        lun_designator: Vec<u8>,
    },
    /// A partition-table entry that aliases or contradicts across views,
    /// held verbatim and marked indeterminate (INV-008, REC-003).
    ConflictingTableEntry {
        /// The table node whose views conflict.
        table: NodeId,
        /// The view the entry appears in.
        view_role: TableRole,
        /// The entry's start offset.
        entry_start: u64,
    },
}

/// A naming failure, surfaced as a value rather than a panic.
///
/// The register's governing finding — a refusal must be an artifact — holds
/// at this layer too: nothing here aborts, and absorption is total.
#[derive(Debug, PartialEq, Eq)]
pub enum NamingError {
    /// The canonical encoder refused the naming preimage. Preimages are
    /// flat, shallow maps of the fields above, so this indicates a
    /// programming error, not hostile input; it is reported rather than
    /// panicked.
    Encoding(canonical::Error),
    /// Two distinct naming-field sets produced one digest — a SHA-256
    /// collision. Cryptographically unreachable; refused rather than
    /// silently merged if it ever occurs.
    AddressCollision,
}

impl fmt::Display for NamingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding(error) => {
                write!(formatter, "naming preimage refused by encoder: {error}")
            }
            Self::AddressCollision => {
                formatter.write_str("two distinct naming-field sets produced one address digest")
            }
        }
    }
}

impl std::error::Error for NamingError {}

/// One entry in an absorbed node set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeEntry {
    /// A uniquely addressed node.
    Single {
        /// The derived address.
        id: NodeId,
        /// The naming fields the address was derived from.
        fields: NamingFields,
    },
    /// ADR-0019's collision group: two or more same-kind nodes derived one
    /// address and are jointly represented — counted, flagged, and
    /// indeterminate. The group carries the shared address, so children
    /// named under it keep their addresses when a colliding member arrives.
    Group {
        /// The shared derived address.
        id: NodeId,
        /// How many nodes derived this address. Always at least two.
        count: u32,
        /// True when the collision is an aggregate pair sharing a present
        /// native designator — the cloned-pool case, flagged without
        /// re-designating anything.
        duplicate_designator: bool,
        /// The shared naming fields.
        fields: NamingFields,
    },
}

impl NodeEntry {
    /// The entry's address.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        match self {
            Self::Single { id, .. } | Self::Group { id, .. } => *id,
        }
    }
}

/// Derive a node's address from its naming fields (ADR-0019).
///
/// The address is the SHA-256 of a domain-separated canonical value
/// (`schema: "partman.node-id"`, version 1) holding the kind tag and the
/// per-kind fields, with parent addresses embedded as digest bytes. The
/// derivation reads nothing but the fields — a node's address depends only
/// on the node and its ancestors, never on the presence of other nodes.
///
/// # Errors
///
/// [`NamingError::Encoding`] if the canonical encoder refuses the preimage,
/// which no well-formed field set can cause.
pub fn derive_id(fields: &NamingFields) -> Result<NodeId, NamingError> {
    let mut preimage = BTreeMap::new();
    preimage.insert(
        "schema".to_owned(),
        Value::Text("partman.node-id".to_owned()),
    );
    preimage.insert("schema_version".to_owned(), Value::Unsigned(1));
    preimage.insert("kind".to_owned(), Value::Text(kind_tag(fields).to_owned()));
    insert_fields(&mut preimage, fields);
    canonical::hash(&Value::Map(preimage))
        .map(NodeId)
        .map_err(NamingError::Encoding)
}

/// Rebuild an address from its recorded 32 bytes, or `None` when the
/// length is wrong. The rebuilt address asserts nothing: every consumer
/// recomputes and compares (the decode-recompute rule).
pub(crate) fn id_from_bytes(bytes: &[u8]) -> Option<NodeId> {
    let digest: [u8; 32] = bytes.try_into().ok()?;
    Some(NodeId(Hash::from_bytes(digest)))
}

/// The kind-tagged field map for a node — the snapshot body's per-node
/// content, and the same fields the address preimage carries (without the
/// preimage's schema keys).
pub(crate) fn fields_value(fields: &NamingFields) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "kind".to_owned(),
        Value::Text(fields.kind_name().to_owned()),
    );
    insert_fields(&mut map, fields);
    Value::Map(map)
}

/// Reverse of [`fields_value`] over an already-decoded map: rebuild the
/// naming fields a body entry declares, refusing unknown kinds, missing or
/// mistyped fields, and unknown keys. Owned by the schema-validation pass,
/// which is the sole decode boundary; the generic codec never sees this.
pub(crate) fn fields_from_map(
    map: &BTreeMap<String, Value>,
) -> Result<NamingFields, FieldParseError> {
    let kind = require_text(map, "kind")?;
    let fields = match kind {
        "physical-device" => NamingFields::PhysicalDevice {
            serial: optional_bytes(map, "serial"),
            wwn: optional_bytes(map, "wwn"),
            total_bytes: require_unsigned(map, "total_bytes")?,
        },
        "partition-table" => NamingFields::PartitionTable {
            parent: require_id(map, "parent")?,
            role: role_from(require_value(map, "role")?)?,
        },
        "partition" => NamingFields::Partition {
            parent_table: require_id(map, "parent_table")?,
            start_offset: require_unsigned(map, "start_offset")?,
        },
        "backing-signature" => NamingFields::BackingSignature {
            host: require_id(map, "host")?,
            family: family_from(require_value(map, "family")?)?,
            primary_offset: require_unsigned(map, "primary_offset")?,
        },
        "file-system" => NamingFields::FileSystem {
            host: require_id(map, "host")?,
            kind: fs_kind_from(require_value(map, "fs_kind")?)?,
            superblock_offset: require_unsigned(map, "superblock_offset")?,
        },
        "encryption-layer" => NamingFields::EncryptionLayer {
            backing_signature: require_id(map, "backing_signature")?,
        },
        "aggregate" => NamingFields::Aggregate {
            technology: technology_from(require_value(map, "technology")?)?,
            designator: optional_bytes(map, "designator"),
        },
        "volume" => NamingFields::Volume {
            producer: require_id(map, "producer")?,
            name: require_bytes(map, "name")?,
            role: optional_bytes(map, "volume_role"),
        },
        "backing-extent" => NamingFields::BackingExtent {
            host: require_id(map, "host")?,
            locator: if map.contains_key("path") {
                ExtentLocator::Path {
                    bytes: require_bytes(map, "path")?,
                }
            } else {
                ExtentLocator::Range {
                    start: require_unsigned(map, "range_start")?,
                    length: require_unsigned(map, "range_length")?,
                }
            },
        },
        "multipath-node" => NamingFields::MultipathNode {
            lun_designator: require_bytes(map, "lun_designator")?,
        },
        "conflicting-table-entry" => NamingFields::ConflictingTableEntry {
            table: require_id(map, "table")?,
            view_role: role_from(require_value(map, "view_role")?)?,
            entry_start: require_unsigned(map, "entry_start")?,
        },
        other => {
            return Err(FieldParseError::UnknownKind {
                kind: other.to_owned(),
            });
        }
    };
    let Value::Map(mut expected) = fields_value(&fields) else {
        return Err(FieldParseError::Internal);
    };
    for key in map.keys() {
        if expected.remove(key).is_none() {
            return Err(FieldParseError::UnknownField { key: key.clone() });
        }
    }
    Ok(fields)
}

/// A body entry that does not parse back into naming fields.
///
/// Public because [`super::snapshot::SnapshotSchemaError`] carries it; only
/// the schema-validation pass constructs it.
#[derive(Debug, PartialEq, Eq)]
pub enum FieldParseError {
    /// The `kind` tag names no kind this build knows.
    UnknownKind {
        /// The unrecognized tag.
        kind: String,
    },
    /// A required field is missing or carries the wrong value shape.
    BadField {
        /// The field's key.
        key: &'static str,
    },
    /// The entry carries a key its kind does not declare.
    UnknownField {
        /// The undeclared key.
        key: String,
    },
    /// An internal invariant failed; unreachable for well-formed input.
    Internal,
}

impl fmt::Display for FieldParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind { kind } => write!(formatter, "unknown node kind `{kind}`"),
            Self::BadField { key } => write!(formatter, "missing or mistyped field `{key}`"),
            Self::UnknownField { key } => write!(formatter, "undeclared field `{key}`"),
            Self::Internal => formatter.write_str("internal parse invariant failed"),
        }
    }
}

fn require_value<'map>(
    map: &'map BTreeMap<String, Value>,
    key: &'static str,
) -> Result<&'map Value, FieldParseError> {
    map.get(key).ok_or(FieldParseError::BadField { key })
}

fn require_text<'map>(
    map: &'map BTreeMap<String, Value>,
    key: &'static str,
) -> Result<&'map str, FieldParseError> {
    match require_value(map, key)? {
        Value::Text(text) => Ok(text),
        _ => Err(FieldParseError::BadField { key }),
    }
}

fn require_unsigned(
    map: &BTreeMap<String, Value>,
    key: &'static str,
) -> Result<u64, FieldParseError> {
    match require_value(map, key)? {
        Value::Unsigned(value) => Ok(*value),
        _ => Err(FieldParseError::BadField { key }),
    }
}

fn require_bytes(
    map: &BTreeMap<String, Value>,
    key: &'static str,
) -> Result<Vec<u8>, FieldParseError> {
    match require_value(map, key)? {
        Value::Bytes(bytes) => Ok(bytes.clone()),
        _ => Err(FieldParseError::BadField { key }),
    }
}

fn require_id(map: &BTreeMap<String, Value>, key: &'static str) -> Result<NodeId, FieldParseError> {
    match require_value(map, key)? {
        Value::Bytes(bytes) => {
            let digest: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| FieldParseError::BadField { key })?;
            Ok(NodeId(Hash::from_bytes(digest)))
        }
        _ => Err(FieldParseError::BadField { key }),
    }
}

fn optional_bytes(map: &BTreeMap<String, Value>, key: &str) -> Option<Vec<u8>> {
    match map.get(key) {
        Some(Value::Bytes(bytes)) => Some(bytes.clone()),
        _ => None,
    }
}

fn role_from(value: &Value) -> Result<TableRole, FieldParseError> {
    Ok(match value {
        Value::Text(text) => match text.as_str() {
            "gpt" => TableRole::Gpt,
            "mbr" => TableRole::Mbr,
            "apm" => TableRole::Apm,
            "hybrid-mbr" => TableRole::HybridMbr,
            _ => return Err(FieldParseError::BadField { key: "role" }),
        },
        Value::Bytes(raw) => TableRole::Unrecognized { raw: raw.clone() },
        _ => return Err(FieldParseError::BadField { key: "role" }),
    })
}

fn family_from(value: &Value) -> Result<SignatureFamily, FieldParseError> {
    Ok(match value {
        Value::Text(text) => match text.as_str() {
            "zfs" => SignatureFamily::Zfs,
            "mdraid-0.90" => SignatureFamily::Mdraid09,
            "mdraid-1.x" => SignatureFamily::Mdraid1x,
            "luks1" => SignatureFamily::Luks1,
            "luks2" => SignatureFamily::Luks2,
            "lvm2" => SignatureFamily::Lvm2,
            "storage-spaces" => SignatureFamily::StorageSpaces,
            "ldm" => SignatureFamily::Ldm,
            "bitlocker" => SignatureFamily::BitLocker,
            "apfs-container" => SignatureFamily::ApfsContainer,
            _ => return Err(FieldParseError::BadField { key: "family" }),
        },
        Value::Bytes(raw) => SignatureFamily::Unrecognized { raw: raw.clone() },
        _ => return Err(FieldParseError::BadField { key: "family" }),
    })
}

fn fs_kind_from(value: &Value) -> Result<FileSystemKind, FieldParseError> {
    Ok(match value {
        Value::Text(text) => match text.as_str() {
            "ext2" => FileSystemKind::Ext2,
            "ext3" => FileSystemKind::Ext3,
            "ext4" => FileSystemKind::Ext4,
            "btrfs" => FileSystemKind::Btrfs,
            "xfs" => FileSystemKind::Xfs,
            "f2fs" => FileSystemKind::F2fs,
            "fat12" => FileSystemKind::Fat12,
            "fat16" => FileSystemKind::Fat16,
            "fat32" => FileSystemKind::Fat32,
            "exfat" => FileSystemKind::Exfat,
            "ntfs" => FileSystemKind::Ntfs,
            "refs" => FileSystemKind::Refs,
            "hfsplus" => FileSystemKind::HfsPlus,
            "apfs" => FileSystemKind::Apfs,
            "udf" => FileSystemKind::Udf,
            "swap" => FileSystemKind::Swap,
            _ => return Err(FieldParseError::BadField { key: "fs_kind" }),
        },
        Value::Bytes(raw) => FileSystemKind::Unrecognized { raw: raw.clone() },
        _ => return Err(FieldParseError::BadField { key: "fs_kind" }),
    })
}

fn technology_from(value: &Value) -> Result<AggregateTechnology, FieldParseError> {
    Ok(match value {
        Value::Text(text) => match text.as_str() {
            "lvm2" => AggregateTechnology::Lvm2,
            "mdraid" => AggregateTechnology::Mdraid,
            "storage-spaces" => AggregateTechnology::StorageSpaces,
            "zfs" => AggregateTechnology::Zfs,
            "apfs" => AggregateTechnology::Apfs,
            "ldm" => AggregateTechnology::Ldm,
            _ => return Err(FieldParseError::BadField { key: "technology" }),
        },
        Value::Bytes(raw) => AggregateTechnology::Unrecognized { raw: raw.clone() },
        _ => return Err(FieldParseError::BadField { key: "technology" }),
    })
}

/// Absorb a set of observed nodes into addressed entries (ADR-0019).
///
/// Nodes deriving distinct addresses become [`NodeEntry::Single`]; nodes
/// deriving equal addresses become one counted [`NodeEntry::Group`]. The
/// result is sorted by address bytes, so it is a deterministic function of
/// the observed multiset, independent of enumeration order. Absorption is
/// total: every multiset of well-formed fields produces entries.
///
/// # Errors
///
/// [`NamingError::Encoding`] as for [`derive_id`];
/// [`NamingError::AddressCollision`] if distinct field sets share a digest.
pub fn absorb(nodes: Vec<NamingFields>) -> Result<Vec<NodeEntry>, NamingError> {
    let mut by_id: BTreeMap<NodeId, (NamingFields, u32)> = BTreeMap::new();
    for fields in nodes {
        let id = derive_id(&fields)?;
        match by_id.get_mut(&id) {
            None => {
                by_id.insert(id, (fields, 1));
            }
            Some((existing, count)) => {
                if *existing != fields {
                    return Err(NamingError::AddressCollision);
                }
                *count += 1;
            }
        }
    }
    Ok(by_id
        .into_iter()
        .map(|(id, (fields, count))| {
            if count == 1 {
                NodeEntry::Single { id, fields }
            } else {
                let duplicate_designator = matches!(
                    fields,
                    NamingFields::Aggregate {
                        designator: Some(_),
                        ..
                    }
                );
                NodeEntry::Group {
                    id,
                    count,
                    duplicate_designator,
                    fields,
                }
            }
        })
        .collect())
}

impl NamingFields {
    /// The node's kind name — the same tag the address preimage carries,
    /// and the vocabulary the edge endpoint-pair table speaks.
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::PhysicalDevice { .. } => "physical-device",
            Self::PartitionTable { .. } => "partition-table",
            Self::Partition { .. } => "partition",
            Self::BackingSignature { .. } => "backing-signature",
            Self::FileSystem { .. } => "file-system",
            Self::EncryptionLayer { .. } => "encryption-layer",
            Self::Aggregate { .. } => "aggregate",
            Self::Volume { .. } => "volume",
            Self::BackingExtent { .. } => "backing-extent",
            Self::MultipathNode { .. } => "multipath-node",
            Self::ConflictingTableEntry { .. } => "conflicting-table-entry",
        }
    }

    /// The addresses this node's own name embeds, each paired with the
    /// field that carries it.
    ///
    /// Eight kinds name themselves relative to another node, so their
    /// address is a function of that node's address. This is the one
    /// roster of those fields: `Topology::build` sweeps it to refuse a
    /// referent no absorbed entry carries, and the planner's destruction
    /// closure walks it to remove everything named relative to a removed
    /// node. Those two readings are a matched pair — "a swept capture
    /// stays swept across a simulated rebuild" is a theorem only while
    /// both read the same list, and it stops being one the moment a
    /// second copy drifts.
    ///
    /// The naming enum is closed today; a variant added later fails this
    /// match at compile time, which is the intended review point.
    #[must_use]
    pub fn naming_referents(&self) -> Vec<(&'static str, NodeId)> {
        match self {
            Self::PhysicalDevice { .. } | Self::Aggregate { .. } | Self::MultipathNode { .. } => {
                vec![]
            }
            Self::PartitionTable { parent, .. } => vec![("parent", *parent)],
            Self::Partition { parent_table, .. } => vec![("parent_table", *parent_table)],
            Self::BackingSignature { host, .. }
            | Self::FileSystem { host, .. }
            | Self::BackingExtent { host, .. } => vec![("host", *host)],
            Self::EncryptionLayer { backing_signature } => {
                vec![("backing_signature", *backing_signature)]
            }
            Self::Volume { producer, .. } => vec![("producer", *producer)],
            Self::ConflictingTableEntry { table, .. } => vec![("table", *table)],
        }
    }

    /// The partition table whose destruction releases this node, if its
    /// own name says one describes it (issue #347, ADR-0043).
    ///
    /// A `Partition` names its table in `parent_table`, so it cannot be
    /// represented without saying which table describes it — the release
    /// is read here rather than off a containment edge, which a body may
    /// omit. A `ConflictingTableEntry` also names a table, and is **not**
    /// released by it: ADR-0019 holds it verbatim as a record inside the
    /// table's own bytes, and ADR-0036 decided it is not an occupant of
    /// the region it names; destroying its table destroys the record,
    /// which the ordinary geometry already reaches, and releases nothing
    /// beyond it. Every other kind names no table.
    #[must_use]
    pub const fn released_by_table(&self) -> Option<NodeId> {
        match self {
            Self::Partition { parent_table, .. } => Some(*parent_table),
            _ => None,
        }
    }

    /// Whether a node of this kind may carry an extent fact.
    ///
    /// A produced node has no position in anyone's address space: an
    /// aggregate, a volume, an encryption layer and a multipath node are
    /// named by what produces them, not by where they sit. The decode
    /// path refuses an extent on these kinds, and the protection closure
    /// reads the same predicate rather than a second copy of the list —
    /// a fact the body format rejects must not be able to steer reach.
    #[must_use]
    pub const fn may_carry_extent(&self) -> bool {
        !matches!(
            self,
            Self::Aggregate { .. }
                | Self::MultipathNode { .. }
                | Self::EncryptionLayer { .. }
                | Self::Volume { .. }
        )
    }
}

const fn kind_tag(fields: &NamingFields) -> &'static str {
    fields.kind_name()
}

fn insert_fields(preimage: &mut BTreeMap<String, Value>, fields: &NamingFields) {
    match fields {
        NamingFields::PhysicalDevice {
            serial,
            wwn,
            total_bytes,
        } => {
            insert_optional_bytes(preimage, "serial", serial.as_deref());
            insert_optional_bytes(preimage, "wwn", wwn.as_deref());
            preimage.insert("total_bytes".to_owned(), Value::Unsigned(*total_bytes));
        }
        NamingFields::PartitionTable { parent, role } => {
            insert_id(preimage, "parent", *parent);
            preimage.insert("role".to_owned(), role_value(role));
        }
        NamingFields::Partition {
            parent_table,
            start_offset,
        } => {
            insert_id(preimage, "parent_table", *parent_table);
            preimage.insert("start_offset".to_owned(), Value::Unsigned(*start_offset));
        }
        NamingFields::BackingSignature {
            host,
            family,
            primary_offset,
        } => {
            insert_id(preimage, "host", *host);
            preimage.insert("family".to_owned(), family_value(family));
            preimage.insert(
                "primary_offset".to_owned(),
                Value::Unsigned(*primary_offset),
            );
        }
        NamingFields::FileSystem {
            host,
            kind,
            superblock_offset,
        } => {
            insert_id(preimage, "host", *host);
            preimage.insert("fs_kind".to_owned(), fs_kind_value(kind));
            preimage.insert(
                "superblock_offset".to_owned(),
                Value::Unsigned(*superblock_offset),
            );
        }
        NamingFields::EncryptionLayer { backing_signature } => {
            insert_id(preimage, "backing_signature", *backing_signature);
        }
        NamingFields::Aggregate {
            technology,
            designator,
        } => {
            preimage.insert("technology".to_owned(), technology_value(technology));
            insert_optional_bytes(preimage, "designator", designator.as_deref());
        }
        NamingFields::Volume {
            producer,
            name,
            role,
        } => {
            insert_id(preimage, "producer", *producer);
            preimage.insert("name".to_owned(), Value::Bytes(name.clone()));
            insert_optional_bytes(preimage, "volume_role", role.as_deref());
        }
        NamingFields::BackingExtent { host, locator } => {
            insert_id(preimage, "host", *host);
            match locator {
                ExtentLocator::Path { bytes } => {
                    preimage.insert("path".to_owned(), Value::Bytes(bytes.clone()));
                }
                ExtentLocator::Range { start, length } => {
                    preimage.insert("range_start".to_owned(), Value::Unsigned(*start));
                    preimage.insert("range_length".to_owned(), Value::Unsigned(*length));
                }
            }
        }
        NamingFields::MultipathNode { lun_designator } => {
            preimage.insert(
                "lun_designator".to_owned(),
                Value::Bytes(lun_designator.clone()),
            );
        }
        NamingFields::ConflictingTableEntry {
            table,
            view_role,
            entry_start,
        } => {
            insert_id(preimage, "table", *table);
            preimage.insert("view_role".to_owned(), role_value(view_role));
            preimage.insert("entry_start".to_owned(), Value::Unsigned(*entry_start));
        }
    }
}

fn insert_id(preimage: &mut BTreeMap<String, Value>, key: &str, id: NodeId) {
    preimage.insert(key.to_owned(), Value::Bytes(id.as_bytes().to_vec()));
}

fn insert_optional_bytes(preimage: &mut BTreeMap<String, Value>, key: &str, bytes: Option<&[u8]>) {
    if let Some(bytes) = bytes {
        preimage.insert(key.to_owned(), Value::Bytes(bytes.to_vec()));
    }
}

fn role_value(role: &TableRole) -> Value {
    match role {
        TableRole::Gpt => Value::Text("gpt".to_owned()),
        TableRole::Mbr => Value::Text("mbr".to_owned()),
        TableRole::Apm => Value::Text("apm".to_owned()),
        TableRole::HybridMbr => Value::Text("hybrid-mbr".to_owned()),
        TableRole::Unrecognized { raw } => Value::Bytes(raw.clone()),
    }
}

fn family_value(family: &SignatureFamily) -> Value {
    match family {
        SignatureFamily::Zfs => Value::Text("zfs".to_owned()),
        SignatureFamily::Mdraid09 => Value::Text("mdraid-0.90".to_owned()),
        SignatureFamily::Mdraid1x => Value::Text("mdraid-1.x".to_owned()),
        SignatureFamily::Luks1 => Value::Text("luks1".to_owned()),
        SignatureFamily::Luks2 => Value::Text("luks2".to_owned()),
        SignatureFamily::Lvm2 => Value::Text("lvm2".to_owned()),
        SignatureFamily::StorageSpaces => Value::Text("storage-spaces".to_owned()),
        SignatureFamily::Ldm => Value::Text("ldm".to_owned()),
        SignatureFamily::BitLocker => Value::Text("bitlocker".to_owned()),
        SignatureFamily::ApfsContainer => Value::Text("apfs-container".to_owned()),
        SignatureFamily::Unrecognized { raw } => Value::Bytes(raw.clone()),
    }
}

fn fs_kind_value(kind: &FileSystemKind) -> Value {
    match kind {
        FileSystemKind::Ext2 => Value::Text("ext2".to_owned()),
        FileSystemKind::Ext3 => Value::Text("ext3".to_owned()),
        FileSystemKind::Ext4 => Value::Text("ext4".to_owned()),
        FileSystemKind::Btrfs => Value::Text("btrfs".to_owned()),
        FileSystemKind::Xfs => Value::Text("xfs".to_owned()),
        FileSystemKind::F2fs => Value::Text("f2fs".to_owned()),
        FileSystemKind::Fat12 => Value::Text("fat12".to_owned()),
        FileSystemKind::Fat16 => Value::Text("fat16".to_owned()),
        FileSystemKind::Fat32 => Value::Text("fat32".to_owned()),
        FileSystemKind::Exfat => Value::Text("exfat".to_owned()),
        FileSystemKind::Ntfs => Value::Text("ntfs".to_owned()),
        FileSystemKind::Refs => Value::Text("refs".to_owned()),
        FileSystemKind::HfsPlus => Value::Text("hfsplus".to_owned()),
        FileSystemKind::Apfs => Value::Text("apfs".to_owned()),
        FileSystemKind::Udf => Value::Text("udf".to_owned()),
        FileSystemKind::Swap => Value::Text("swap".to_owned()),
        FileSystemKind::Unrecognized { raw } => Value::Bytes(raw.clone()),
    }
}

fn technology_value(technology: &AggregateTechnology) -> Value {
    match technology {
        AggregateTechnology::Lvm2 => Value::Text("lvm2".to_owned()),
        AggregateTechnology::Mdraid => Value::Text("mdraid".to_owned()),
        AggregateTechnology::StorageSpaces => Value::Text("storage-spaces".to_owned()),
        AggregateTechnology::Zfs => Value::Text("zfs".to_owned()),
        AggregateTechnology::Apfs => Value::Text("apfs".to_owned()),
        AggregateTechnology::Ldm => Value::Text("ldm".to_owned()),
        AggregateTechnology::Unrecognized { raw } => Value::Bytes(raw.clone()),
    }
}
