//! SAFE-003's device identity record (WP-010 increment 3d; ADR-C3,
//! ADR-C4's guard, ADR-0014, ADR-0015, ADR-0017).
//!
//! The record binds a write target immutably: all available identifiers,
//! geometry, the three-valued partition-table state, and — where the
//! apparatus is qualified — the continuity witness. Two disciplines are
//! structural here:
//!
//! - **Strength is derived, never stored.** [`DeviceIdentity::strength`]
//!   computes SAFE-003's classification from the record alone; no field
//!   carries it, so a record claiming `Strong` while lacking a stable
//!   identifier is unrepresentable, and a forged `strength` key in body
//!   bytes refuses as an undeclared field.
//! - **ADR-C4's guard holds in bytes**: a positively absent partition
//!   table and an unreadable one produce different body values, held by
//!   test against the encoded record.
//!
//! The table state is one of MODEL-005's two authoring-set entries: the
//! privileged helper stamps it at validation from its own raw-sector
//! parser (ADR-0014), and a client-authored value never validates. That
//! enforcement lives at the plan boundary in a later slice; this module
//! defines the vocabulary both sides share.

use std::collections::BTreeMap;
use std::fmt;

use crate::canonical::{Hash, Value};

/// Why a table state is `Indeterminate` (spec 8.0.0's two arms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndeterminateCause {
    /// Two independently valid tables describe different partitions.
    Ambiguous,
    /// No copy could be read and validated.
    Unreadable,
}

/// ADR-C3's three-valued partition-table state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableState {
    /// A table was read and hashed; the checksum is over the scheme's
    /// copy-invariant content per `schemas/table-checksum.md`, so two
    /// agreeing copies produce one checksum from either copy.
    Present {
        /// The copy-invariant content checksum.
        checksum: Hash,
    },
    /// Positively observed to have no table.
    Absent,
    /// Unreadable or ambiguous. Never positively determined; a device in
    /// this state cannot be `Strong` under any contract, and PART-001's
    /// categorical invariant never initializes it.
    Indeterminate {
        /// Which arm.
        cause: IndeterminateCause,
    },
}

impl TableState {
    /// Whether the state is positively determined (`Present` or `Absent`).
    #[must_use]
    pub const fn positively_determined(&self) -> bool {
        matches!(self, Self::Present { .. } | Self::Absent)
    }
}

/// ADR-0017's continuity witness: an epoch token and a media-event
/// counter reading, present only where the target is exchange-capable and
/// the platform's apparatus is qualified. Absence is the status quo, never
/// a regression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContinuityWitness {
    /// The epoch token — the storage-node identity whose re-arrival marks
    /// a comparability boundary. Contract-source-verbatim bytes.
    pub epoch_token: Vec<u8>,
    /// The media-event counter reading within that epoch.
    pub counter: u64,
}

/// The closed witness-comparison vocabulary (SAFE-003, 10.0.0).
///
/// Deliberately, no `continuous` or `verified` value exists to reach for:
/// the strongest word is the liveness ceiling's own, and a consumer MUST
/// NOT treat [`WitnessOutcome::NoExchangeObserved`] as evidence of
/// continuity. The witness is a refusal input, never an assurance, and it
/// relaxes no confirmation, floor, or policy anywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessOutcome {
    /// Same epoch, value unchanged — the vocabulary's strongest word.
    NoExchangeObserved,
    /// Same epoch, value moved: an exchange may have occurred. For covered
    /// targets this is SAFE-003's identity change, and the plan rejects.
    ExchangeObserved,
    /// The epoch token changed, or the value decreased within a token — a
    /// reset the token failed to witness. For covered targets, reject.
    Incomparable,
    /// No witness on one side or both (unqualified apparatus).
    Unavailable,
}

/// Compare a bound witness against a fresh reading (SAFE-003, 10.0.0).
///
/// Readings are comparable only within an unchanged epoch token and never
/// when the value decreased; the unmeasured-hardware failure mode is a
/// spurious refusal, never a false pass.
#[must_use]
pub fn compare_witness(
    bound: Option<&ContinuityWitness>,
    fresh: Option<&ContinuityWitness>,
) -> WitnessOutcome {
    let (Some(bound), Some(fresh)) = (bound, fresh) else {
        return WitnessOutcome::Unavailable;
    };
    if bound.epoch_token != fresh.epoch_token {
        return WitnessOutcome::Incomparable;
    }
    match fresh.counter.cmp(&bound.counter) {
        std::cmp::Ordering::Less => WitnessOutcome::Incomparable,
        std::cmp::Ordering::Equal => WitnessOutcome::NoExchangeObserved,
        std::cmp::Ordering::Greater => WitnessOutcome::ExchangeObserved,
    }
}

/// SAFE-003's identity strength — a property of one record, computable
/// without a counterpart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityStrength {
    /// A stable hardware identifier, total size, both sector sizes, and a
    /// positively determined table state.
    Strong,
    /// Anything less.
    Weak,
}

/// SAFE-003's immutable target identity record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// Serial bytes from the contract's named source, or absent.
    pub serial: Option<Vec<u8>>,
    /// WWN bytes from the contract's named source, or absent.
    pub wwn: Option<Vec<u8>>,
    /// The OS device instance identifier, or absent.
    pub os_instance_id: Option<Vec<u8>>,
    /// The connection/location path, or absent.
    pub connection_path: Option<Vec<u8>>,
    /// Total device size in bytes (MODEL-001).
    pub total_bytes: u64,
    /// Logical sector size in bytes, where the contract reports it.
    pub logical_sector_size: Option<u64>,
    /// Physical sector size in bytes, where the contract reports it.
    pub physical_sector_size: Option<u64>,
    /// ADR-C3's table state — helper-authored at validation (ADR-0014).
    pub table: TableState,
    /// ADR-0017's continuity witness, where the apparatus is qualified.
    pub witness: Option<ContinuityWitness>,
}

impl DeviceIdentity {
    /// Derive SAFE-003's strength from the record alone.
    ///
    /// Strong requires at least one stable hardware identifier (serial or
    /// WWN), total size, both sector sizes, and a positively determined
    /// table state. A device whose table failed to parse cannot be Strong
    /// under any contract — ADR-C3's deliberate tightening, which the
    /// glossary's 4.0.0 correction restates.
    #[must_use]
    pub fn strength(&self) -> IdentityStrength {
        let has_stable_identifier = self.serial.is_some() || self.wwn.is_some();
        let has_geometry =
            self.logical_sector_size.is_some() && self.physical_sector_size.is_some();
        if has_stable_identifier && has_geometry && self.table.positively_determined() {
            IdentityStrength::Strong
        } else {
            IdentityStrength::Weak
        }
    }

    /// The record as a canonical body value (MODEL-005). Strength is not
    /// in the bytes — it is derived — so a forged `strength` key refuses
    /// at [`identity_from_map`] as an undeclared field.
    #[must_use]
    pub fn body_value(&self) -> Value {
        let mut map = BTreeMap::new();
        insert_optional(&mut map, "serial", self.serial.as_deref());
        insert_optional(&mut map, "wwn", self.wwn.as_deref());
        insert_optional(&mut map, "os_instance_id", self.os_instance_id.as_deref());
        insert_optional(&mut map, "connection_path", self.connection_path.as_deref());
        map.insert("total_bytes".to_owned(), Value::Unsigned(self.total_bytes));
        if let Some(size) = self.logical_sector_size {
            map.insert("logical_sector_size".to_owned(), Value::Unsigned(size));
        }
        if let Some(size) = self.physical_sector_size {
            map.insert("physical_sector_size".to_owned(), Value::Unsigned(size));
        }
        map.insert("table".to_owned(), table_value(&self.table));
        if let Some(witness) = &self.witness {
            let mut witness_map = BTreeMap::new();
            witness_map.insert(
                "epoch_token".to_owned(),
                Value::Bytes(witness.epoch_token.clone()),
            );
            witness_map.insert("counter".to_owned(), Value::Unsigned(witness.counter));
            map.insert("witness".to_owned(), Value::Map(witness_map));
        }
        Value::Map(map)
    }
}

fn insert_optional(map: &mut BTreeMap<String, Value>, key: &str, bytes: Option<&[u8]>) {
    if let Some(bytes) = bytes {
        map.insert(key.to_owned(), Value::Bytes(bytes.to_vec()));
    }
}

fn table_value(table: &TableState) -> Value {
    let mut map = BTreeMap::new();
    match table {
        TableState::Present { checksum } => {
            map.insert("state".to_owned(), Value::Text("present".to_owned()));
            map.insert(
                "checksum".to_owned(),
                Value::Bytes(checksum.as_bytes().to_vec()),
            );
        }
        TableState::Absent => {
            map.insert("state".to_owned(), Value::Text("absent".to_owned()));
        }
        TableState::Indeterminate { cause } => {
            map.insert("state".to_owned(), Value::Text("indeterminate".to_owned()));
            let cause = match cause {
                IndeterminateCause::Ambiguous => "ambiguous",
                IndeterminateCause::Unreadable => "unreadable",
            };
            map.insert("cause".to_owned(), Value::Text(cause.to_owned()));
        }
    }
    Value::Map(map)
}

/// Rebuild an identity record from a decoded body map, refusing unknown
/// keys, mistyped fields, and malformed table or witness values. Owned by
/// the schema-validation pass; the generic codec never sees this.
///
/// # Errors
///
/// [`IdentityParseError`] naming the first rule violated.
pub fn identity_from_map(
    map: &BTreeMap<String, Value>,
) -> Result<DeviceIdentity, IdentityParseError> {
    for key in map.keys() {
        if !matches!(
            key.as_str(),
            "serial"
                | "wwn"
                | "os_instance_id"
                | "connection_path"
                | "total_bytes"
                | "logical_sector_size"
                | "physical_sector_size"
                | "table"
                | "witness"
        ) {
            return Err(IdentityParseError::UnknownField { key: key.clone() });
        }
    }
    let total_bytes = match map.get("total_bytes") {
        Some(Value::Unsigned(value)) => *value,
        _ => return Err(IdentityParseError::BadField { key: "total_bytes" }),
    };
    let table = match map.get("table") {
        Some(Value::Map(table_map)) => table_from_map(table_map)?,
        _ => return Err(IdentityParseError::BadField { key: "table" }),
    };
    let witness = match map.get("witness") {
        None => None,
        Some(Value::Map(witness_map)) => Some(witness_from_map(witness_map)?),
        Some(_) => return Err(IdentityParseError::BadField { key: "witness" }),
    };
    Ok(DeviceIdentity {
        serial: optional_bytes(map, "serial")?,
        wwn: optional_bytes(map, "wwn")?,
        os_instance_id: optional_bytes(map, "os_instance_id")?,
        connection_path: optional_bytes(map, "connection_path")?,
        total_bytes,
        logical_sector_size: optional_unsigned(map, "logical_sector_size")?,
        physical_sector_size: optional_unsigned(map, "physical_sector_size")?,
        table,
        witness,
    })
}

fn optional_bytes(
    map: &BTreeMap<String, Value>,
    key: &'static str,
) -> Result<Option<Vec<u8>>, IdentityParseError> {
    match map.get(key) {
        None => Ok(None),
        Some(Value::Bytes(bytes)) => Ok(Some(bytes.clone())),
        Some(_) => Err(IdentityParseError::BadField { key }),
    }
}

fn optional_unsigned(
    map: &BTreeMap<String, Value>,
    key: &'static str,
) -> Result<Option<u64>, IdentityParseError> {
    match map.get(key) {
        None => Ok(None),
        Some(Value::Unsigned(value)) => Ok(Some(*value)),
        Some(_) => Err(IdentityParseError::BadField { key }),
    }
}

fn table_from_map(map: &BTreeMap<String, Value>) -> Result<TableState, IdentityParseError> {
    let state = match map.get("state") {
        Some(Value::Text(text)) => text.as_str(),
        _ => return Err(IdentityParseError::BadField { key: "table" }),
    };
    let expected_keys: &[&str] = match state {
        "present" => &["state", "checksum"],
        "absent" => &["state"],
        "indeterminate" => &["state", "cause"],
        _ => return Err(IdentityParseError::BadField { key: "table" }),
    };
    for key in map.keys() {
        if !expected_keys.contains(&key.as_str()) {
            return Err(IdentityParseError::UnknownField { key: key.clone() });
        }
    }
    Ok(match state {
        "present" => match map.get("checksum") {
            Some(Value::Bytes(bytes)) => {
                let digest: [u8; 32] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| IdentityParseError::BadField { key: "table" })?;
                TableState::Present {
                    checksum: Hash::from_bytes(digest),
                }
            }
            _ => return Err(IdentityParseError::BadField { key: "table" }),
        },
        "absent" => TableState::Absent,
        _ => match map.get("cause") {
            Some(Value::Text(cause)) => match cause.as_str() {
                "ambiguous" => TableState::Indeterminate {
                    cause: IndeterminateCause::Ambiguous,
                },
                "unreadable" => TableState::Indeterminate {
                    cause: IndeterminateCause::Unreadable,
                },
                _ => return Err(IdentityParseError::BadField { key: "table" }),
            },
            _ => return Err(IdentityParseError::BadField { key: "table" }),
        },
    })
}

fn witness_from_map(
    map: &BTreeMap<String, Value>,
) -> Result<ContinuityWitness, IdentityParseError> {
    for key in map.keys() {
        if !matches!(key.as_str(), "epoch_token" | "counter") {
            return Err(IdentityParseError::UnknownField { key: key.clone() });
        }
    }
    let epoch_token = match map.get("epoch_token") {
        Some(Value::Bytes(bytes)) => bytes.clone(),
        _ => return Err(IdentityParseError::BadField { key: "witness" }),
    };
    let counter = match map.get("counter") {
        Some(Value::Unsigned(value)) => *value,
        _ => return Err(IdentityParseError::BadField { key: "witness" }),
    };
    Ok(ContinuityWitness {
        epoch_token,
        counter,
    })
}

/// An identity record that does not parse from its body map.
#[derive(Debug, PartialEq, Eq)]
pub enum IdentityParseError {
    /// The map carries a key the record does not declare — including a
    /// forged `strength`, which is derived and never stored.
    UnknownField {
        /// The undeclared key.
        key: String,
    },
    /// A required field is missing or carries the wrong value shape.
    BadField {
        /// The field's key.
        key: &'static str,
    },
}

impl fmt::Display for IdentityParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownField { key } => write!(formatter, "undeclared field `{key}`"),
            Self::BadField { key } => write!(formatter, "missing or mistyped field `{key}`"),
        }
    }
}

impl std::error::Error for IdentityParseError {}
