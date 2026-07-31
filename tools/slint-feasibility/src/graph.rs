use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::str::FromStr;

use crate::CheckError;
use crate::metadata::{CargoMetadata, Dependency, DependencyKind, NodeDependencyKind, Package};

const DESKTOP_PACKAGE: &str = "partman-desktop";
const COMPILER_PACKAGE: &str = "i-slint-compiler";
const SPIN_ON_PACKAGE: &str = "spin_on";
const REGISTRY_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";
const COMPILER_VERSION: &str = "1.17.1";
const SPIN_ON_VERSION: &str = "0.1.1";
const TYPED_INDEX_PACKAGE: &str = "typed-index-collections";
const TYPED_INDEX_VERSION: &str = "3.5.0";
const INACTIVE_BINCODE_PACKAGE: &str = "bincode";
const INACTIVE_BINCODE_VERSION: &str = "2.0.1";
const INACTIVE_BINCODE_ADVISORY: &str = "RUSTSEC-2025-0141";
const FORBIDDEN_COMPILER_FEATURES: [&str; 3] =
    ["software-renderer", "bundle-translations", "sdf-fonts"];
const EXPECTED_COMPILER_FEATURES: [&str; 2] = ["display-diagnostics", "rust"];

/// The proof phase represented by one locked metadata graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPhase {
    /// Build-host compiler adapter only; no Slint runtime is present or proven.
    CompilerOnly,
    /// Reserved final runtime phase, deliberately rejected by this checkpoint.
    FinalRuntime,
}

impl FromStr for GraphPhase {
    type Err = CheckError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "compiler-only" => Ok(Self::CompilerOnly),
            "final-runtime" => Ok(Self::FinalRuntime),
            _ => Err(CheckError::new(format!(
                "unknown graph phase {value:?}; expected compiler-only or final-runtime"
            ))),
        }
    }
}

impl fmt::Display for GraphPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CompilerOnly => "compiler-only",
            Self::FinalRuntime => "final-runtime",
        })
    }
}

/// Successful conservative host/target graph judgment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphReport {
    /// The only phase this report proves.
    pub phase: GraphPhase,
    /// Reachable build-host package count.
    pub host_package_count: usize,
    /// Reachable target package count.
    pub target_package_count: usize,
    /// Exact direct capability roots enabling `i-slint-compiler`.
    pub compiler_capability_roots: BTreeSet<String>,
    /// Always false for the bounded compiler-only checkpoint.
    pub final_runtime_proven: bool,
    /// Number of non-development conditional edges conservatively included.
    pub conservatively_included_predicates: usize,
    /// Exact cargo-audit warning proven confined to an inactive optional edge.
    pub lockfile_only_advisories: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Realm {
    Host,
    Target,
}

/// Judge locked Cargo metadata without parsing `cargo tree` presentation text.
///
/// The compiler-only phase starts at the desktop target, transitions build and
/// proc-macro dependencies to the host realm, excludes development edges, and
/// conservatively includes every non-development target-predicate edge. That
/// over-approximation may reject a graph but cannot hide a forbidden package.
/// It proves only the compiler checkpoint and sets `final_runtime_proven` to
/// false by construction.
///
/// # Errors
///
/// Rejects a missing/ambiguous root, a dangling or declaration-less edge,
/// `slint-build`, target-reachable Slint runtime packages, missing or target-
/// reachable exact compiler/`spin_on` pins, compiler feature/capability drift,
/// host `image` reachability, and all requests for the unimplemented final
/// runtime phase.
pub fn verify_graph(
    metadata: &CargoMetadata,
    phase: GraphPhase,
) -> Result<GraphReport, CheckError> {
    if phase == GraphPhase::FinalRuntime {
        return Err(CheckError::new(
            "final-runtime graph proof is not implemented by this checkpoint; compiler-only evidence cannot satisfy it",
        ));
    }
    let root = metadata.exact_package(DESKTOP_PACKAGE)?;
    if !metadata.workspace_members.contains(&root.id) {
        return Err(CheckError::new(
            "partman-desktop is not a Cargo workspace member",
        ));
    }

    let Reachability {
        states,
        incoming,
        conditional_edges,
    } = reachable_states(metadata, &root.id)?;
    reject_forbidden_packages(metadata, &states)?;
    let all_workspace_states = all_workspace_states(metadata)?;
    let lockfile_only_advisories = verify_inactive_bincode(metadata, &all_workspace_states)?;

    let compiler = metadata.exact_package(COMPILER_PACKAGE)?;
    let spin_on = metadata.exact_package(SPIN_ON_PACKAGE)?;
    require_registry_identity(compiler, COMPILER_PACKAGE, COMPILER_VERSION)?;
    require_registry_identity(spin_on, SPIN_ON_PACKAGE, SPIN_ON_VERSION)?;
    require_desktop_build_pin(
        root,
        COMPILER_PACKAGE,
        &format!("={COMPILER_VERSION}"),
        false,
        &EXPECTED_COMPILER_FEATURES,
    )?;
    require_desktop_build_pin(
        root,
        SPIN_ON_PACKAGE,
        &format!("={SPIN_ON_VERSION}"),
        true,
        &[],
    )?;
    require_host_only(&states, compiler, COMPILER_PACKAGE)?;
    require_host_only(&states, spin_on, SPIN_ON_PACKAGE)?;

    let compiler_node = metadata
        .nodes
        .get(&compiler.id)
        .ok_or_else(|| CheckError::new("i-slint-compiler has no locked resolve node"))?;
    let expected_roots = EXPECTED_COMPILER_FEATURES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected_resolved_features = local_feature_closure(compiler, &expected_roots)?;
    if compiler_node.features != expected_resolved_features {
        return Err(CheckError::new(format!(
            "compiler-only i-slint-compiler feature closure drifted: expected {expected_resolved_features:?} from roots {expected_roots:?}, found {:?}",
            compiler_node.features
        )));
    }
    for forbidden in FORBIDDEN_COMPILER_FEATURES {
        if compiler_node.features.contains(forbidden) {
            return Err(CheckError::new(format!(
                "forbidden i-slint-compiler feature {forbidden:?} is enabled"
            )));
        }
    }

    let compiler_roots = capability_roots(metadata, compiler, &incoming)?;
    if compiler_roots != expected_roots {
        return Err(CheckError::new(format!(
            "compiler capability roots drifted: expected {expected_roots:?}, found {compiler_roots:?}"
        )));
    }
    for feature in &compiler_roots {
        if !compiler.features.contains_key(feature) {
            return Err(CheckError::new(format!(
                "compiler capability root {feature:?} is absent from its manifest feature table"
            )));
        }
    }

    let host_package_count = states
        .iter()
        .filter(|(_, realm)| *realm == Realm::Host)
        .count();
    let target_package_count = states
        .iter()
        .filter(|(_, realm)| *realm == Realm::Target)
        .count();
    Ok(GraphReport {
        phase,
        host_package_count,
        target_package_count,
        compiler_capability_roots: compiler_roots,
        final_runtime_proven: false,
        conservatively_included_predicates: conditional_edges,
        lockfile_only_advisories,
    })
}

fn all_workspace_states(metadata: &CargoMetadata) -> Result<BTreeSet<(String, Realm)>, CheckError> {
    let mut all_states = BTreeSet::new();
    for root_id in &metadata.workspace_members {
        if !metadata.packages.contains_key(root_id) {
            return Err(CheckError::new(format!(
                "workspace member {root_id:?} has no package record"
            )));
        }
        let Reachability { states, .. } = reachable_states(metadata, root_id)?;
        all_states.extend(states);
    }
    Ok(all_states)
}

fn verify_inactive_bincode(
    metadata: &CargoMetadata,
    states: &BTreeSet<(String, Realm)>,
) -> Result<BTreeSet<String>, CheckError> {
    let owner = metadata.exact_package(TYPED_INDEX_PACKAGE)?;
    let bincode = metadata.exact_package(INACTIVE_BINCODE_PACKAGE)?;
    require_registry_identity(owner, TYPED_INDEX_PACKAGE, TYPED_INDEX_VERSION)?;
    require_registry_identity(bincode, INACTIVE_BINCODE_PACKAGE, INACTIVE_BINCODE_VERSION)?;
    require_host_only(states, owner, TYPED_INDEX_PACKAGE)?;

    let expected_manifest_features = BTreeMap::from([
        (
            "alloc".to_owned(),
            vec!["serde?/alloc".to_owned(), "bincode?/alloc".to_owned()],
        ),
        ("bincode".to_owned(), vec!["dep:bincode".to_owned()]),
        (
            "default".to_owned(),
            vec!["alloc".to_owned(), "std".to_owned()],
        ),
        ("serde".to_owned(), vec!["dep:serde".to_owned()]),
        (
            "serde-alloc".to_owned(),
            vec!["alloc".to_owned(), "serde".to_owned()],
        ),
        (
            "serde-std".to_owned(),
            vec!["std".to_owned(), "serde".to_owned()],
        ),
        (
            "std".to_owned(),
            vec![
                "alloc".to_owned(),
                "serde?/std".to_owned(),
                "bincode?/std".to_owned(),
            ],
        ),
    ]);
    if owner.features != expected_manifest_features {
        return Err(CheckError::new(format!(
            "{TYPED_INDEX_PACKAGE} feature table drifted; the inactive {INACTIVE_BINCODE_ADVISORY} proof must be requalified"
        )));
    }
    let owner_node = metadata.nodes.get(&owner.id).ok_or_else(|| {
        CheckError::new(format!("{TYPED_INDEX_PACKAGE} has no locked resolve node"))
    })?;
    let expected_enabled = ["alloc", "default", "std"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if owner_node.features != expected_enabled {
        return Err(CheckError::new(format!(
            "{TYPED_INDEX_PACKAGE} enabled features drifted: expected {expected_enabled:?}, found {:?}",
            owner_node.features
        )));
    }

    let declarations = owner
        .dependencies
        .iter()
        .filter(|dependency| dependency.name == INACTIVE_BINCODE_PACKAGE)
        .collect::<Vec<_>>();
    let [declaration] = declarations.as_slice() else {
        return Err(CheckError::new(format!(
            "{TYPED_INDEX_PACKAGE} must declare exactly one optional {INACTIVE_BINCODE_PACKAGE} edge; found {}",
            declarations.len()
        )));
    };
    if declaration.rename.is_some()
        || declaration.source.as_deref() != Some(REGISTRY_SOURCE)
        || declaration.requirement != "^2.0.1"
        || declaration.kind != DependencyKind::Normal
        || declaration.target.is_some()
        || !declaration.optional
        || declaration.uses_default_features
        || !declaration.features.is_empty()
    {
        return Err(CheckError::new(format!(
            "{TYPED_INDEX_PACKAGE}'s {INACTIVE_BINCODE_PACKAGE} declaration drifted; {INACTIVE_BINCODE_ADVISORY} may be ignored only for the exact inactive optional edge"
        )));
    }

    let incoming = states
        .iter()
        .filter_map(|(package_id, _)| {
            metadata
                .nodes
                .get(package_id)
                .map(|node| (package_id, node))
        })
        .flat_map(|(package_id, node)| {
            node.dependencies
                .iter()
                .filter(move |dependency| dependency.package_id == bincode.id)
                .map(move |_| package_id.clone())
        })
        .collect::<BTreeSet<_>>();
    if incoming != BTreeSet::from([owner.id.clone()]) {
        return Err(CheckError::new(format!(
            "{INACTIVE_BINCODE_PACKAGE} has unexpected reachable declarers {incoming:?}; {INACTIVE_BINCODE_ADVISORY} cannot be treated as lockfile-only"
        )));
    }

    Ok(BTreeSet::from([INACTIVE_BINCODE_ADVISORY.to_owned()]))
}

fn local_feature_closure(
    package: &Package,
    roots: &BTreeSet<String>,
) -> Result<BTreeSet<String>, CheckError> {
    let mut closure = roots.clone();
    let mut queue = roots.iter().cloned().collect::<VecDeque<_>>();
    while let Some(feature) = queue.pop_front() {
        let members = package.features.get(&feature).ok_or_else(|| {
            CheckError::new(format!(
                "compiler feature {feature:?} is absent from its manifest feature table"
            ))
        })?;
        for member in members {
            if !member.starts_with("dep:")
                && !member.contains('/')
                && package.features.contains_key(member)
                && closure.insert(member.clone())
            {
                queue.push_back(member.clone());
            }
        }
    }
    Ok(closure)
}

fn require_registry_identity(
    package: &Package,
    name: &str,
    version: &str,
) -> Result<(), CheckError> {
    if package.version != version || package.source.as_deref() != Some(REGISTRY_SOURCE) {
        return Err(CheckError::new(format!(
            "{name} registry identity drifted: expected version {version} from {REGISTRY_SOURCE}; found version {} from {:?}",
            package.version, package.source
        )));
    }
    Ok(())
}

fn require_desktop_build_pin(
    desktop: &Package,
    package_name: &str,
    requirement: &str,
    uses_default_features: bool,
    features: &[&str],
) -> Result<(), CheckError> {
    let declarations = desktop
        .dependencies
        .iter()
        .filter(|dependency| {
            dependency.name == package_name
                && dependency.rename.is_none()
                && dependency.kind == DependencyKind::Build
                && dependency.target.is_none()
        })
        .collect::<Vec<_>>();
    let [declaration] = declarations.as_slice() else {
        return Err(CheckError::new(format!(
            "partman-desktop must have exactly one unconditional unaliased build dependency on {package_name}; found {}",
            declarations.len()
        )));
    };
    let expected_features = features.iter().map(|value| (*value).to_owned()).collect();
    if declaration.source.as_deref() != Some(REGISTRY_SOURCE)
        || declaration.requirement != requirement
        || declaration.uses_default_features != uses_default_features
        || declaration.features != expected_features
    {
        return Err(CheckError::new(format!(
            "partman-desktop build dependency {package_name} drifted: expected source {REGISTRY_SOURCE}, requirement {requirement}, default-features={uses_default_features}, features {expected_features:?}; found source {:?}, requirement {}, default-features={}, features {:?}",
            declaration.source,
            declaration.requirement,
            declaration.uses_default_features,
            declaration.features
        )));
    }
    Ok(())
}

struct Reachability {
    states: BTreeSet<(String, Realm)>,
    incoming: BTreeMap<String, Vec<IncomingEdge>>,
    conditional_edges: usize,
}

#[derive(Debug, Clone)]
struct IncomingEdge {
    from_package_id: String,
    dependency_name: String,
    kind: DependencyKind,
    target: Option<String>,
}

fn reachable_states(metadata: &CargoMetadata, root_id: &str) -> Result<Reachability, CheckError> {
    let mut states = BTreeSet::new();
    let mut incoming = BTreeMap::<String, Vec<IncomingEdge>>::new();
    let mut queue = VecDeque::from([(root_id.to_owned(), Realm::Target)]);
    let mut conditional_edges = 0_usize;
    while let Some((package_id, realm)) = queue.pop_front() {
        if !states.insert((package_id.clone(), realm)) {
            continue;
        }
        let package = metadata.packages.get(&package_id).ok_or_else(|| {
            CheckError::new(format!("reachable package ID {package_id:?} is missing"))
        })?;
        let node = metadata.nodes.get(&package_id).ok_or_else(|| {
            CheckError::new(format!(
                "reachable package {} has no resolve node",
                package.name
            ))
        })?;
        for dependency in &node.dependencies {
            if dependency.kinds.is_empty() {
                return Err(CheckError::new(format!(
                    "resolve edge {} -> {} has no dependency kind",
                    package.name, dependency.name
                )));
            }
            let target_package =
                metadata
                    .packages
                    .get(&dependency.package_id)
                    .ok_or_else(|| {
                        CheckError::new(format!(
                            "resolve edge {} -> {} has no package",
                            package.name, dependency.package_id
                        ))
                    })?;
            for dep_kind in &dependency.kinds {
                if dep_kind.kind == DependencyKind::Development {
                    continue;
                }
                if dep_kind.target.is_some() {
                    conditional_edges = conditional_edges.checked_add(1).ok_or_else(|| {
                        CheckError::new("conditional dependency edge count overflowed")
                    })?;
                }
                let declaration =
                    match_dependency_declaration(package, dependency.name.as_str(), dep_kind)?;
                let target_realm = dependency_realm(realm, dep_kind.kind, target_package);
                incoming
                    .entry(target_package.id.clone())
                    .or_default()
                    .push(IncomingEdge {
                        from_package_id: package_id.clone(),
                        dependency_name: dependency.name.clone(),
                        kind: declaration.kind,
                        target: declaration.target.clone(),
                    });
                queue.push_back((target_package.id.clone(), target_realm));
            }
        }
    }
    Ok(Reachability {
        states,
        incoming,
        conditional_edges,
    })
}

fn dependency_realm(realm: Realm, kind: DependencyKind, package: &Package) -> Realm {
    if realm == Realm::Host || kind == DependencyKind::Build || is_proc_macro(package) {
        Realm::Host
    } else {
        Realm::Target
    }
}

fn is_proc_macro(package: &Package) -> bool {
    package
        .targets
        .iter()
        .any(|target| target.kinds.contains("proc-macro"))
}

fn match_dependency_declaration<'a>(
    package: &'a Package,
    resolved_name: &str,
    resolved_kind: &NodeDependencyKind,
) -> Result<&'a Dependency, CheckError> {
    let matches = package
        .dependencies
        .iter()
        .filter(|dependency| {
            declaration_crate_name(dependency) == resolved_name
                && dependency.kind == resolved_kind.kind
                && dependency.target == resolved_kind.target
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [dependency] => Ok(dependency),
        [] => Err(CheckError::new(format!(
            "resolve edge {} -> {resolved_name} has no exact manifest declaration for kind {:?} and target {:?}",
            package.name, resolved_kind.kind, resolved_kind.target
        ))),
        _ => Err(CheckError::new(format!(
            "resolve edge {} -> {resolved_name} has ambiguous manifest declarations",
            package.name
        ))),
    }
}

fn declaration_crate_name(dependency: &Dependency) -> String {
    dependency
        .rename
        .as_deref()
        .unwrap_or(dependency.name.as_str())
        .replace('-', "_")
}

fn reject_forbidden_packages(
    metadata: &CargoMetadata,
    states: &BTreeSet<(String, Realm)>,
) -> Result<(), CheckError> {
    for (id, realm) in states {
        let package = &metadata.packages[id];
        if package.name == "slint-build" {
            return Err(CheckError::new(format!(
                "slint-build is reachable in the {realm:?} graph"
            )));
        }
        if *realm == Realm::Host
            && is_slint_package(&package.name)
            && ![COMPILER_PACKAGE, "i-slint-common"].contains(&package.name.as_str())
        {
            return Err(CheckError::new(format!(
                "compiler-only phase contains build-host Slint runtime package {}",
                package.name
            )));
        }
        if *realm == Realm::Host && package.name == "image" {
            return Err(CheckError::new(
                "image is build-host reachable; the compiler-only graph must not carry default image codecs",
            ));
        }
        if *realm == Realm::Target
            && (package.name == "slint" || package.name.starts_with("i-slint-"))
        {
            return Err(CheckError::new(format!(
                "compiler-only phase contains target-reachable Slint runtime package {}",
                package.name
            )));
        }
    }
    Ok(())
}

fn is_slint_package(name: &str) -> bool {
    name == "slint" || name.starts_with("slint-") || name.starts_with("i-slint-")
}

fn require_host_only(
    states: &BTreeSet<(String, Realm)>,
    package: &Package,
    name: &str,
) -> Result<(), CheckError> {
    if !states.contains(&(package.id.clone(), Realm::Host)) {
        return Err(CheckError::new(format!(
            "{name} is not reachable in the build-host graph"
        )));
    }
    if states.contains(&(package.id.clone(), Realm::Target)) {
        return Err(CheckError::new(format!(
            "{name} is unexpectedly target-reachable"
        )));
    }
    Ok(())
}

fn capability_roots(
    metadata: &CargoMetadata,
    compiler: &Package,
    incoming: &BTreeMap<String, Vec<IncomingEdge>>,
) -> Result<BTreeSet<String>, CheckError> {
    let mut roots = BTreeSet::new();
    let edges = incoming.get(&compiler.id).ok_or_else(|| {
        CheckError::new("i-slint-compiler has no reachable incoming dependency edge")
    })?;
    for edge in edges {
        let from = metadata
            .packages
            .get(&edge.from_package_id)
            .ok_or_else(|| {
                CheckError::new("compiler incoming edge references an unknown package")
            })?;
        let declaration = from
            .dependencies
            .iter()
            .find(|dependency| {
                declaration_crate_name(dependency) == edge.dependency_name
                    && dependency.kind == edge.kind
                    && dependency.target == edge.target
            })
            .ok_or_else(|| {
                CheckError::new(format!(
                    "cannot recover compiler capability declaration from {}",
                    from.name
                ))
            })?;
        if declaration.uses_default_features {
            roots.insert("default".to_owned());
        }
        roots.extend(declaration.features.iter().cloned());
    }
    Ok(roots)
}

#[cfg(test)]
mod tests;
