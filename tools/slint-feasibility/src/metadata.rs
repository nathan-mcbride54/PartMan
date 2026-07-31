use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::CheckError;

/// The locked Cargo metadata fields used by the feasibility judges.
#[derive(Debug, Clone)]
pub struct CargoMetadata {
    pub(crate) packages: BTreeMap<String, Package>,
    pub(crate) nodes: BTreeMap<String, Node>,
    pub(crate) workspace_members: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Package {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) license: Option<String>,
    pub(crate) manifest_path: PathBuf,
    pub(crate) targets: Vec<Target>,
    pub(crate) features: BTreeMap<String, Vec<String>>,
    pub(crate) dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub(crate) struct Target {
    pub(crate) kinds: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Dependency {
    pub(crate) name: String,
    pub(crate) rename: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) requirement: String,
    pub(crate) kind: DependencyKind,
    pub(crate) target: Option<String>,
    pub(crate) optional: bool,
    pub(crate) uses_default_features: bool,
    pub(crate) features: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyKind {
    Normal,
    Build,
    Development,
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) features: BTreeSet<String>,
    pub(crate) dependencies: Vec<NodeDependency>,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeDependency {
    pub(crate) name: String,
    pub(crate) package_id: String,
    pub(crate) kinds: Vec<NodeDependencyKind>,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeDependencyKind {
    pub(crate) kind: DependencyKind,
    pub(crate) target: Option<String>,
}

impl CargoMetadata {
    /// Parse Cargo metadata format 1 without trusting absent identity fields.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, a missing required field, duplicate
    /// package/node IDs, an unknown dependency kind, or a dangling graph edge.
    pub fn parse(bytes: &[u8]) -> Result<Self, CheckError> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| CheckError::new(format!("invalid Cargo metadata JSON: {error}")))?;
        let root = object(&value, "Cargo metadata root")?;
        if integer(root, "version")? != 1 {
            return Err(CheckError::new("Cargo metadata format version is not 1"));
        }

        let mut packages = BTreeMap::new();
        for value in array(root, "packages")? {
            let package = parse_package(value)?;
            let id = package.id.clone();
            if packages.insert(id.clone(), package).is_some() {
                return Err(CheckError::new(format!(
                    "duplicate Cargo metadata package ID {id:?}"
                )));
            }
        }

        let resolve = object(required(root, "resolve")?, "resolve")?;
        let mut nodes = BTreeMap::new();
        for value in array(resolve, "nodes")? {
            let (id, node) = parse_node(value)?;
            if nodes.insert(id.clone(), node).is_some() {
                return Err(CheckError::new(format!(
                    "duplicate Cargo metadata node ID {id:?}"
                )));
            }
        }
        for (id, node) in &nodes {
            if !packages.contains_key(id) {
                return Err(CheckError::new(format!(
                    "resolve node {id:?} has no package record"
                )));
            }
            for dependency in &node.dependencies {
                if !packages.contains_key(&dependency.package_id) {
                    return Err(CheckError::new(format!(
                        "resolve edge from {id:?} points to unknown package {:?}",
                        dependency.package_id
                    )));
                }
            }
        }

        let workspace_members = string_array(root, "workspace_members")?
            .into_iter()
            .collect();
        Ok(Self {
            packages,
            nodes,
            workspace_members,
        })
    }

    pub(crate) fn exact_package(&self, name: &str) -> Result<&Package, CheckError> {
        let matches = self
            .packages
            .values()
            .filter(|package| package.name == name)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [package] => Ok(package),
            [] => Err(CheckError::new(format!(
                "locked Cargo metadata does not contain {name}"
            ))),
            _ => Err(CheckError::new(format!(
                "locked Cargo metadata contains multiple {name} packages"
            ))),
        }
    }
}

fn parse_package(value: &Value) -> Result<Package, CheckError> {
    let package = object(value, "package")?;
    let id = string(package, "id")?;
    let targets = array(package, "targets")?
        .iter()
        .map(parse_target)
        .collect::<Result<Vec<_>, _>>()?;
    let features = object(required(package, "features")?, "package features")?
        .iter()
        .map(|(name, values)| Ok((name.clone(), value_string_array(values, "feature members")?)))
        .collect::<Result<_, CheckError>>()?;
    let dependencies = array(package, "dependencies")?
        .iter()
        .map(parse_dependency)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Package {
        id,
        name: string(package, "name")?,
        version: string(package, "version")?,
        source: optional_string(package, "source")?,
        license: optional_string(package, "license")?,
        manifest_path: PathBuf::from(string(package, "manifest_path")?),
        targets,
        features,
        dependencies,
    })
}

fn parse_target(value: &Value) -> Result<Target, CheckError> {
    let target = object(value, "target")?;
    Ok(Target {
        kinds: string_array(target, "kind")?.into_iter().collect(),
    })
}

fn parse_dependency(value: &Value) -> Result<Dependency, CheckError> {
    let dependency = object(value, "dependency")?;
    Ok(Dependency {
        name: string(dependency, "name")?,
        rename: optional_string(dependency, "rename")?,
        source: optional_string(dependency, "source")?,
        requirement: string(dependency, "req")?,
        kind: dependency_kind(optional_string(dependency, "kind")?.as_deref())?,
        target: optional_string(dependency, "target")?,
        optional: boolean(dependency, "optional")?,
        uses_default_features: boolean(dependency, "uses_default_features")?,
        features: string_array(dependency, "features")?.into_iter().collect(),
    })
}

fn parse_node(value: &Value) -> Result<(String, Node), CheckError> {
    let node = object(value, "resolve node")?;
    let dependencies = array(node, "deps")?
        .iter()
        .map(parse_node_dependency)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        string(node, "id")?,
        Node {
            features: string_array(node, "features")?.into_iter().collect(),
            dependencies,
        },
    ))
}

fn parse_node_dependency(value: &Value) -> Result<NodeDependency, CheckError> {
    let dependency = object(value, "resolve dependency")?;
    let kinds = array(dependency, "dep_kinds")?
        .iter()
        .map(|value| {
            let kind = object(value, "resolve dependency kind")?;
            Ok(NodeDependencyKind {
                kind: dependency_kind(optional_string(kind, "kind")?.as_deref())?,
                target: optional_string(kind, "target")?,
            })
        })
        .collect::<Result<Vec<_>, CheckError>>()?;
    Ok(NodeDependency {
        name: string(dependency, "name")?,
        package_id: string(dependency, "pkg")?,
        kinds,
    })
}

fn dependency_kind(value: Option<&str>) -> Result<DependencyKind, CheckError> {
    match value {
        None | Some("normal") => Ok(DependencyKind::Normal),
        Some("build") => Ok(DependencyKind::Build),
        Some("dev") => Ok(DependencyKind::Development),
        Some(other) => Err(CheckError::new(format!(
            "unknown Cargo dependency kind {other:?}"
        ))),
    }
}

fn object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, CheckError> {
    value
        .as_object()
        .ok_or_else(|| CheckError::new(format!("{context} is not an object")))
}

fn required<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a Value, CheckError> {
    object
        .get(key)
        .ok_or_else(|| CheckError::new(format!("missing Cargo metadata field {key:?}")))
}

fn array<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a Vec<Value>, CheckError> {
    required(object, key)?
        .as_array()
        .ok_or_else(|| CheckError::new(format!("Cargo metadata field {key:?} is not an array")))
}

fn string(object: &Map<String, Value>, key: &str) -> Result<String, CheckError> {
    required(object, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| CheckError::new(format!("Cargo metadata field {key:?} is not a string")))
}

fn optional_string(object: &Map<String, Value>, key: &str) -> Result<Option<String>, CheckError> {
    match required(object, key)? {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value.clone())),
        _ => Err(CheckError::new(format!(
            "Cargo metadata field {key:?} is neither a string nor null"
        ))),
    }
}

fn boolean(object: &Map<String, Value>, key: &str) -> Result<bool, CheckError> {
    required(object, key)?
        .as_bool()
        .ok_or_else(|| CheckError::new(format!("Cargo metadata field {key:?} is not a boolean")))
}

fn integer(object: &Map<String, Value>, key: &str) -> Result<u64, CheckError> {
    required(object, key)?
        .as_u64()
        .ok_or_else(|| CheckError::new(format!("Cargo metadata field {key:?} is not an integer")))
}

fn string_array(object: &Map<String, Value>, key: &str) -> Result<Vec<String>, CheckError> {
    value_string_array(required(object, key)?, key)
}

fn value_string_array(value: &Value, context: &str) -> Result<Vec<String>, CheckError> {
    value
        .as_array()
        .ok_or_else(|| CheckError::new(format!("{context} is not an array")))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| CheckError::new(format!("{context} contains a non-string")))
        })
        .collect()
}

pub(crate) fn package_root(package: &Package) -> Result<&Path, CheckError> {
    package.manifest_path.parent().ok_or_else(|| {
        CheckError::new(format!(
            "package manifest has no parent: {}",
            package.manifest_path.display()
        ))
    })
}
