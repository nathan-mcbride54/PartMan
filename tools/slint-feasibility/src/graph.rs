use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::str::FromStr;

use cargo_platform::{Cfg, Platform};

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
const EXPECTED_RUNTIME_COMPILER_FEATURES: [&str; 4] =
    ["default", "display-diagnostics", "proc_macro_span", "rust"];
const SLINT_PACKAGE: &str = "slint";
const SLINT_MACROS_PACKAGE: &str = "slint-macros";
const CORE_PACKAGE: &str = "i-slint-core";
const BACKEND_SELECTOR_PACKAGE: &str = "i-slint-backend-selector";
const WINIT_BACKEND_PACKAGE: &str = "i-slint-backend-winit";
const FEMTOVG_RENDERER_PACKAGE: &str = "i-slint-renderer-femtovg";
const SOFTWARE_RENDERER_PACKAGE: &str = "i-slint-renderer-software";
const IMAGE_PACKAGE: &str = "image";
const IMAGE_VERSION: &str = "0.25.10";
const RESVG_PACKAGE: &str = "resvg";
const RESVG_VERSION: &str = "0.47.0";
const UNICODE_SEGMENTATION_PACKAGE: &str = "unicode-segmentation";
const UNICODE_SEGMENTATION_VERSION: &str = "1.13.3";

/// The proof phase represented by one locked metadata graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPhase {
    /// Build-host compiler adapter only; no Slint runtime is present or proven.
    CompilerOnly,
    /// Public runtime graph for one renderer or the marked combined control.
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

/// Exact desktop feature configuration represented by one metadata graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphConfiguration {
    /// Build-host compiler adapter with no public Slint runtime feature.
    CompilerOnly,
    /// Adoption-eligible Winit/FemtoVG graph.
    RendererFemtoVg,
    /// Adoption-eligible Winit/software graph.
    RendererSoftware,
    /// Deliberately non-shipping two-renderer comparison graph.
    ComparisonCombined,
}

impl GraphConfiguration {
    /// Whether this configuration contains exactly one candidate renderer.
    #[must_use]
    pub const fn is_single_renderer(self) -> bool {
        matches!(self, Self::RendererFemtoVg | Self::RendererSoftware)
    }
}

impl FromStr for GraphConfiguration {
    type Err = CheckError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "compiler-only" => Ok(Self::CompilerOnly),
            "renderer-femtovg" => Ok(Self::RendererFemtoVg),
            "renderer-software" => Ok(Self::RendererSoftware),
            "comparison-combined" => Ok(Self::ComparisonCombined),
            _ => Err(CheckError::new(format!(
                "unknown graph configuration {value:?}; expected compiler-only, renderer-femtovg, renderer-software, or comparison-combined"
            ))),
        }
    }
}

impl fmt::Display for GraphConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CompilerOnly => "compiler-only",
            Self::RendererFemtoVg => "renderer-femtovg",
            Self::RendererSoftware => "renderer-software",
            Self::ComparisonCombined => "comparison-combined",
        })
    }
}

/// Authenticated native target and compiler cfg values used for Cargo edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetContext {
    name: String,
    cfgs: Vec<Cfg>,
}

impl TargetContext {
    /// Construct a target context from authenticated `rustc` output.
    ///
    /// # Errors
    ///
    /// Rejects an empty target name or duplicate cfg values.
    pub fn new(name: String, cfgs: Vec<Cfg>) -> Result<Self, CheckError> {
        if name.is_empty() {
            return Err(CheckError::new("native target name is empty"));
        }
        let unique = cfgs.iter().collect::<BTreeSet<_>>();
        if unique.len() != cfgs.len() {
            return Err(CheckError::new(
                "native target cfg output contains duplicate values",
            ));
        }
        Ok(Self { name, cfgs })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, predicate: &str) -> Result<bool, CheckError> {
        let platform = Platform::from_str(predicate).map_err(|error| {
            CheckError::new(format!(
                "cannot parse Cargo target predicate {predicate:?}: {error}"
            ))
        })?;
        Ok(platform.matches(&self.name, &self.cfgs))
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
    /// Exact feature configuration this report judged.
    pub configuration: GraphConfiguration,
    /// Reachable build-host package count.
    pub host_package_count: usize,
    /// Reachable target package count.
    pub target_package_count: usize,
    /// Exact direct capability roots enabling `i-slint-compiler`.
    pub compiler_capability_roots: BTreeSet<String>,
    /// True only for a qualified, exactly-one-renderer final runtime graph.
    pub final_runtime_proven: bool,
    /// Number of non-development target predicates evaluated for this target.
    pub evaluated_target_predicates: usize,
    /// Exact cargo-audit warning proven confined to an inactive optional edge.
    pub lockfile_only_advisories: BTreeSet<String>,
    /// Exact reachable Slint package names whose source policy is in scope.
    pub reachable_slint_packages: BTreeSet<String>,
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
/// evaluates every non-development target predicate with Cargo's own platform
/// parser against an authenticated native `rustc --print=cfg` context.
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
    target: &TargetContext,
    phase: GraphPhase,
    configuration: GraphConfiguration,
) -> Result<GraphReport, CheckError> {
    validate_phase_configuration(phase, configuration)?;
    let root = metadata.exact_package(DESKTOP_PACKAGE)?;
    if !metadata.workspace_members.contains(&root.id) {
        return Err(CheckError::new(
            "partman-desktop is not a Cargo workspace member",
        ));
    }

    let root_features = root_feature_roots(configuration);
    let reachability = reachable_states(metadata, &root.id, target, &root_features)?;
    let states = &reachability.states;
    reject_forbidden_packages(metadata, states, phase, configuration)?;
    validate_reachable_features_are_resolved(metadata, &reachability)?;
    let all_workspace_states = all_workspace_states(metadata, target, configuration)?;
    let lockfile_only_advisories = verify_inactive_bincode(metadata, &all_workspace_states)?;

    let compiler_roots =
        verify_compiler_graph(metadata, root, &reachability, phase, configuration)?;

    if phase == GraphPhase::FinalRuntime {
        verify_runtime_graph(metadata, root, &reachability, configuration)?;
    }

    let host_package_count = states
        .iter()
        .filter(|(_, realm)| *realm == Realm::Host)
        .count();
    let target_package_count = states
        .iter()
        .filter(|(_, realm)| *realm == Realm::Target)
        .count();
    let reachable_slint_packages = states
        .iter()
        .filter_map(|(package_id, _)| {
            let name = &metadata.packages[package_id].name;
            is_slint_package(name).then(|| name.clone())
        })
        .collect();
    Ok(GraphReport {
        phase,
        configuration,
        host_package_count,
        target_package_count,
        compiler_capability_roots: compiler_roots,
        final_runtime_proven: phase == GraphPhase::FinalRuntime
            && configuration.is_single_renderer(),
        evaluated_target_predicates: reachability.evaluated_target_predicates.len(),
        lockfile_only_advisories,
        reachable_slint_packages,
    })
}

pub(crate) fn validate_phase_configuration(
    phase: GraphPhase,
    configuration: GraphConfiguration,
) -> Result<(), CheckError> {
    match (phase, configuration) {
        (GraphPhase::CompilerOnly, GraphConfiguration::CompilerOnly)
        | (
            GraphPhase::FinalRuntime,
            GraphConfiguration::RendererFemtoVg
            | GraphConfiguration::RendererSoftware
            | GraphConfiguration::ComparisonCombined,
        ) => Ok(()),
        _ => Err(CheckError::new(format!(
            "graph phase {phase} is incompatible with configuration {configuration}"
        ))),
    }
}

fn verify_compiler_graph(
    metadata: &CargoMetadata,
    desktop: &Package,
    reachability: &Reachability,
    phase: GraphPhase,
    configuration: GraphConfiguration,
) -> Result<BTreeSet<String>, CheckError> {
    let compiler = metadata.exact_package(COMPILER_PACKAGE)?;
    let spin_on = metadata.exact_package(SPIN_ON_PACKAGE)?;
    require_registry_identity(compiler, COMPILER_PACKAGE, COMPILER_VERSION)?;
    require_registry_identity(spin_on, SPIN_ON_PACKAGE, SPIN_ON_VERSION)?;
    require_desktop_build_pin(
        desktop,
        COMPILER_PACKAGE,
        &format!("={COMPILER_VERSION}"),
        false,
        &EXPECTED_COMPILER_FEATURES,
    )?;
    require_desktop_build_pin(
        desktop,
        SPIN_ON_PACKAGE,
        &format!("={SPIN_ON_VERSION}"),
        true,
        &[],
    )?;
    require_host_only(&reachability.states, compiler, COMPILER_PACKAGE)?;
    require_host_only(&reachability.states, spin_on, SPIN_ON_PACKAGE)?;

    let compiler_feature_roots = if phase == GraphPhase::CompilerOnly {
        EXPECTED_COMPILER_FEATURES.as_slice()
    } else {
        EXPECTED_RUNTIME_COMPILER_FEATURES.as_slice()
    };
    let expected_roots = compiler_feature_roots
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    let expected_resolved_features = local_feature_closure(compiler, &expected_roots)?;
    let compiler_features = reachability.features_for(compiler, Realm::Host)?;
    let compiler_global_features = &metadata
        .nodes
        .get(&compiler.id)
        .ok_or_else(|| CheckError::new("i-slint-compiler has no locked resolve node"))?
        .features;
    if compiler_features != &expected_resolved_features
        || compiler_global_features != &expected_resolved_features
    {
        return Err(CheckError::new(format!(
            "i-slint-compiler feature closure drifted in {configuration}: expected {expected_resolved_features:?} from roots {expected_roots:?}, found realm {compiler_features:?} and workspace {compiler_global_features:?}"
        )));
    }
    for forbidden in FORBIDDEN_COMPILER_FEATURES {
        if compiler_features.contains(forbidden) {
            return Err(CheckError::new(format!(
                "forbidden i-slint-compiler feature {forbidden:?} is enabled"
            )));
        }
    }

    let compiler_roots = capability_roots(metadata, compiler, &reachability.incoming)?;
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
    Ok(compiler_roots)
}

fn verify_runtime_graph(
    metadata: &CargoMetadata,
    desktop: &Package,
    reachability: &Reachability,
    configuration: GraphConfiguration,
) -> Result<(), CheckError> {
    verify_runtime_manifest_and_packages(metadata, desktop, reachability, configuration)?;
    verify_renderer_reachability(metadata, reachability, configuration)?;
    verify_runtime_assets(metadata, reachability)?;
    reject_forbidden_runtime_features(metadata, reachability)
}

fn verify_runtime_manifest_and_packages(
    metadata: &CargoMetadata,
    desktop: &Package,
    reachability: &Reachability,
    configuration: GraphConfiguration,
) -> Result<(), CheckError> {
    let states = &reachability.states;
    let expected_manifest_features = BTreeMap::from([
        (
            "comparison-combined".to_owned(),
            vec![
                "renderer-femtovg".to_owned(),
                "renderer-software".to_owned(),
            ],
        ),
        ("default".to_owned(), vec!["renderer-femtovg".to_owned()]),
        (
            "renderer-femtovg".to_owned(),
            vec!["slint/renderer-femtovg".to_owned()],
        ),
        (
            "renderer-software".to_owned(),
            vec!["slint/renderer-software".to_owned()],
        ),
    ]);
    if desktop.features != expected_manifest_features {
        return Err(CheckError::new(
            "partman-desktop renderer feature table drifted from ADR-0009",
        ));
    }
    require_desktop_runtime_pin(
        desktop,
        SLINT_PACKAGE,
        &format!("={COMPILER_VERSION}"),
        false,
        &["accessibility", "backend-winit", "compat-1-2", "std"],
    )?;
    require_desktop_runtime_pin(
        desktop,
        UNICODE_SEGMENTATION_PACKAGE,
        &format!("={UNICODE_SEGMENTATION_VERSION}"),
        true,
        &[],
    )?;

    let expected_desktop_features = match configuration {
        GraphConfiguration::RendererFemtoVg => BTreeSet::from(["renderer-femtovg".to_owned()]),
        GraphConfiguration::RendererSoftware => BTreeSet::from(["renderer-software".to_owned()]),
        GraphConfiguration::ComparisonCombined => BTreeSet::from([
            "comparison-combined".to_owned(),
            "renderer-femtovg".to_owned(),
            "renderer-software".to_owned(),
        ]),
        GraphConfiguration::CompilerOnly => unreachable!("phase/configuration checked above"),
    };
    let desktop_features = reachability.features_for(desktop, Realm::Target)?;
    if desktop_features != &expected_desktop_features {
        return Err(CheckError::new(format!(
            "partman-desktop enabled features drifted in {configuration}: expected {expected_desktop_features:?}, found {desktop_features:?}"
        )));
    }

    let slint = require_exact_runtime_package(metadata, SLINT_PACKAGE)?;
    let slint_macros = require_exact_runtime_package(metadata, SLINT_MACROS_PACKAGE)?;
    let common = require_exact_runtime_package(metadata, "i-slint-common")?;
    let core = require_exact_runtime_package(metadata, CORE_PACKAGE)?;
    let selector = require_exact_runtime_package(metadata, BACKEND_SELECTOR_PACKAGE)?;
    let winit = require_exact_runtime_package(metadata, WINIT_BACKEND_PACKAGE)?;
    require_target_only(states, slint, SLINT_PACKAGE)?;
    require_host_only(states, slint_macros, SLINT_MACROS_PACKAGE)?;
    require_target_reachable(states, common, "i-slint-common")?;
    require_target_only(states, core, CORE_PACKAGE)?;
    require_target_only(states, selector, BACKEND_SELECTOR_PACKAGE)?;
    require_target_only(states, winit, WINIT_BACKEND_PACKAGE)?;

    let mut expected_slint_features = ["accessibility", "backend-winit", "compat-1-2", "std"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    match configuration {
        GraphConfiguration::RendererFemtoVg => {
            expected_slint_features.insert("i-slint-renderer-femtovg".to_owned());
            expected_slint_features.insert("renderer-femtovg".to_owned());
        }
        GraphConfiguration::RendererSoftware => {
            expected_slint_features.insert("renderer-software".to_owned());
        }
        GraphConfiguration::ComparisonCombined => {
            expected_slint_features.insert("i-slint-renderer-femtovg".to_owned());
            expected_slint_features.insert("renderer-femtovg".to_owned());
            expected_slint_features.insert("renderer-software".to_owned());
        }
        GraphConfiguration::CompilerOnly => unreachable!("phase/configuration checked above"),
    }
    require_realm_features(reachability, slint, Realm::Target, &expected_slint_features)?;
    Ok(())
}

fn verify_renderer_reachability(
    metadata: &CargoMetadata,
    reachability: &Reachability,
    configuration: GraphConfiguration,
) -> Result<(), CheckError> {
    let states = &reachability.states;
    let femtovg = optional_exact_runtime_package(metadata, FEMTOVG_RENDERER_PACKAGE)?;
    let software = optional_exact_runtime_package(metadata, SOFTWARE_RENDERER_PACKAGE)?;
    let femtovg_reachable =
        femtovg.is_some_and(|package| states.contains(&(package.id.clone(), Realm::Target)));
    let software_reachable =
        software.is_some_and(|package| states.contains(&(package.id.clone(), Realm::Target)));
    let found_renderer_reachability = (femtovg_reachable, software_reachable);
    let expected_renderer_reachability = match configuration {
        GraphConfiguration::RendererFemtoVg => (true, false),
        GraphConfiguration::RendererSoftware => (false, true),
        GraphConfiguration::ComparisonCombined => (true, true),
        GraphConfiguration::CompilerOnly => unreachable!("phase/configuration checked above"),
    };
    if found_renderer_reachability != expected_renderer_reachability {
        return Err(CheckError::new(format!(
            "renderer reachability drifted in {configuration}: expected FemtoVG/software {expected_renderer_reachability:?}, found {found_renderer_reachability:?}"
        )));
    }
    Ok(())
}

fn verify_runtime_assets(
    metadata: &CargoMetadata,
    reachability: &Reachability,
) -> Result<(), CheckError> {
    let states = &reachability.states;
    let image = metadata.exact_package(IMAGE_PACKAGE)?;
    let resvg = metadata.exact_package(RESVG_PACKAGE)?;
    require_registry_identity(image, IMAGE_PACKAGE, IMAGE_VERSION)?;
    require_registry_identity(resvg, RESVG_PACKAGE, RESVG_VERSION)?;
    require_target_only(states, image, IMAGE_PACKAGE)?;
    require_target_reachable(states, resvg, RESVG_PACKAGE)?;
    require_realm_features(
        reachability,
        image,
        Realm::Target,
        &["jpeg", "png"].into_iter().map(str::to_owned).collect(),
    )?;
    require_realm_features(
        reachability,
        resvg,
        Realm::Target,
        &["gif", "image-webp", "raster-images", "text"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )?;
    if states.contains(&(resvg.id.clone(), Realm::Host)) {
        require_realm_features(reachability, resvg, Realm::Host, &BTreeSet::new())?;
    }
    Ok(())
}

fn require_exact_runtime_package<'a>(
    metadata: &'a CargoMetadata,
    name: &str,
) -> Result<&'a Package, CheckError> {
    let package = metadata.exact_package(name)?;
    require_registry_identity(package, name, COMPILER_VERSION)?;
    Ok(package)
}

fn optional_exact_runtime_package<'a>(
    metadata: &'a CargoMetadata,
    name: &str,
) -> Result<Option<&'a Package>, CheckError> {
    let matches = metadata
        .packages
        .values()
        .filter(|package| package.name == name)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [package] => {
            require_registry_identity(package, name, COMPILER_VERSION)?;
            Ok(Some(package))
        }
        _ => Err(CheckError::new(format!(
            "locked Cargo metadata contains multiple {name} packages"
        ))),
    }
}

fn require_desktop_runtime_pin(
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
                && dependency.kind == DependencyKind::Normal
                && dependency.target.is_none()
        })
        .collect::<Vec<_>>();
    let [declaration] = declarations.as_slice() else {
        return Err(CheckError::new(format!(
            "partman-desktop must have exactly one unconditional unaliased runtime dependency on {package_name}; found {}",
            declarations.len()
        )));
    };
    let expected_features = features.iter().map(|value| (*value).to_owned()).collect();
    if declaration.source.as_deref() != Some(REGISTRY_SOURCE)
        || declaration.requirement != requirement
        || declaration.optional
        || declaration.uses_default_features != uses_default_features
        || declaration.features != expected_features
    {
        return Err(CheckError::new(format!(
            "partman-desktop runtime dependency {package_name} drifted from its exact ADR-0009 pin"
        )));
    }
    Ok(())
}

fn require_realm_features(
    reachability: &Reachability,
    package: &Package,
    realm: Realm,
    expected: &BTreeSet<String>,
) -> Result<(), CheckError> {
    let found = reachability.features_for(package, realm)?;
    if found != expected {
        return Err(CheckError::new(format!(
            "{} resolved feature set drifted in the {realm:?} realm: expected {expected:?}, found {found:?}",
            package.name
        )));
    }
    Ok(())
}

fn reject_forbidden_runtime_features(
    metadata: &CargoMetadata,
    reachability: &Reachability,
) -> Result<(), CheckError> {
    for (id, realm) in &reachability.states {
        let package = &metadata.packages[id];
        if is_slint_package(&package.name) {
            require_registry_identity(package, &package.name, COMPILER_VERSION)?;
        }
        for feature in reachability.features_for(package, *realm)? {
            let forbidden = feature == "backend-default"
                || feature == "backend-qt"
                || feature == "backend-testing"
                || feature == "image-default-formats"
                || feature == "live-preview"
                || feature == "mcp"
                || feature == "system-testing"
                || feature == "system-tray"
                || feature.starts_with("renderer-skia")
                || feature.starts_with("unstable-")
                || feature.contains("wgpu");
            if forbidden {
                return Err(CheckError::new(format!(
                    "forbidden runtime feature {feature:?} is reachable on {} in the {realm:?} realm",
                    package.name
                )));
            }
        }
    }
    Ok(())
}

fn all_workspace_states(
    metadata: &CargoMetadata,
    target: &TargetContext,
    configuration: GraphConfiguration,
) -> Result<BTreeSet<(String, Realm)>, CheckError> {
    let mut all_states = BTreeSet::new();
    for root_id in &metadata.workspace_members {
        let package = metadata.packages.get(root_id).ok_or_else(|| {
            CheckError::new(format!(
                "workspace member {root_id:?} has no package record"
            ))
        })?;
        let roots = if package.name == DESKTOP_PACKAGE {
            root_feature_roots(configuration)
        } else {
            metadata
                .nodes
                .get(root_id)
                .ok_or_else(|| {
                    CheckError::new(format!(
                        "workspace member {} has no resolve node",
                        package.name
                    ))
                })?
                .features
                .clone()
        };
        let Reachability { states, .. } = reachable_states(metadata, root_id, target, &roots)?;
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
    let mut closure = BTreeSet::new();
    let mut queue = VecDeque::new();
    for root in roots {
        if package.features.contains_key(root) {
            closure.insert(root.clone());
            queue.push_back(root.clone());
        } else if root != "default" {
            return Err(CheckError::new(format!(
                "feature {root:?} is absent from {}'s manifest feature table",
                package.name
            )));
        }
    }
    while let Some(feature) = queue.pop_front() {
        let members = &package.features[&feature];
        for member in members {
            if !member.starts_with("dep:")
                && !member.contains('/')
                && package.features.contains_key(member)
                && closure.insert(member.clone())
            {
                queue.push_back(member.clone());
            } else if let Some((dependency_feature, _)) = member.split_once('/')
                && !dependency_feature.ends_with('?')
                && package.features.contains_key(dependency_feature)
                && closure.insert(dependency_feature.to_owned())
            {
                queue.push_back(dependency_feature.to_owned());
            }
        }
    }
    Ok(closure)
}

fn root_feature_roots(configuration: GraphConfiguration) -> BTreeSet<String> {
    match configuration {
        GraphConfiguration::CompilerOnly => BTreeSet::new(),
        GraphConfiguration::RendererFemtoVg => BTreeSet::from(["renderer-femtovg".to_owned()]),
        GraphConfiguration::RendererSoftware => BTreeSet::from(["renderer-software".to_owned()]),
        GraphConfiguration::ComparisonCombined => {
            BTreeSet::from(["comparison-combined".to_owned()])
        }
    }
}

fn validate_reachable_features_are_resolved(
    metadata: &CargoMetadata,
    reachability: &Reachability,
) -> Result<(), CheckError> {
    let mut unions = BTreeMap::<String, BTreeSet<String>>::new();
    for ((package_id, _realm), features) in &reachability.features {
        unions
            .entry(package_id.clone())
            .or_default()
            .extend(features.iter().cloned());
    }
    for (package_id, computed) in unions {
        let package = &metadata.packages[&package_id];
        let resolved = &metadata
            .nodes
            .get(&package_id)
            .ok_or_else(|| CheckError::new(format!("{} has no locked resolve node", package.name)))?
            .features;
        if !computed.is_subset(resolved) {
            return Err(CheckError::new(format!(
                "realm-specific feature propagation produced features Cargo did not resolve for {}: computed union {computed:?}, Cargo reported {resolved:?}",
                package.name
            )));
        }
    }
    Ok(())
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

type RealmState = (String, Realm);
type RealmFeatures = BTreeMap<RealmState, BTreeSet<String>>;

struct Reachability {
    states: BTreeSet<RealmState>,
    features: RealmFeatures,
    incoming: BTreeMap<String, Vec<IncomingEdge>>,
    evaluated_target_predicates: BTreeSet<(String, String, String)>,
}

impl Reachability {
    fn features_for(
        &self,
        package: &Package,
        realm: Realm,
    ) -> Result<&BTreeSet<String>, CheckError> {
        self.features
            .get(&(package.id.clone(), realm))
            .ok_or_else(|| {
                CheckError::new(format!(
                    "{} is not reachable in the {realm:?} realm",
                    package.name
                ))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IncomingEdge {
    from_package_id: String,
    dependency_name: String,
    kind: DependencyKind,
    target: Option<String>,
}

fn reachable_states(
    metadata: &CargoMetadata,
    root_id: &str,
    target: &TargetContext,
    root_features: &BTreeSet<String>,
) -> Result<Reachability, CheckError> {
    let mut states = BTreeSet::new();
    let (mut features, mut queue) = initial_reachability(metadata, root_id, root_features)?;
    let mut incoming = BTreeMap::<String, Vec<IncomingEdge>>::new();
    let mut evaluated_target_predicates = BTreeSet::new();
    while let Some((package_id, realm)) = queue.pop_front() {
        states.insert((package_id.clone(), realm));
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
                if let Some(predicate) = dep_kind.target.as_deref() {
                    evaluated_target_predicates.insert((
                        package_id.clone(),
                        dependency.package_id.clone(),
                        format!("{:?}:{predicate}", dep_kind.kind),
                    ));
                    if !target.matches(predicate)? {
                        continue;
                    }
                }
                let declaration = match_dependency_declaration(
                    package,
                    target_package,
                    dependency.name.as_str(),
                    dep_kind,
                )?;
                let owner_features =
                    features.get(&(package_id.clone(), realm)).ok_or_else(|| {
                        CheckError::new(format!(
                            "reachable package {} has no realm feature set",
                            package.name
                        ))
                    })?;
                let Some(dependency_features) =
                    dependency_feature_roots(package, owner_features, declaration)
                else {
                    continue;
                };
                let target_realm = dependency_realm(realm, dep_kind.kind, target_package);
                let edge = IncomingEdge {
                    from_package_id: package_id.clone(),
                    dependency_name: declaration.name.clone(),
                    kind: declaration.kind,
                    target: declaration.target.clone(),
                };
                let edges = incoming.entry(target_package.id.clone()).or_default();
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
                let dependency_state = (target_package.id.clone(), target_realm);
                let enabled = local_feature_closure(target_package, &dependency_features)?;
                let state_features = features.entry(dependency_state.clone()).or_default();
                let previous_len = state_features.len();
                state_features.extend(enabled);
                if state_features.len() != previous_len || !states.contains(&dependency_state) {
                    queue.push_back(dependency_state);
                }
            }
        }
    }
    Ok(Reachability {
        states,
        features,
        incoming,
        evaluated_target_predicates,
    })
}

fn initial_reachability(
    metadata: &CargoMetadata,
    root_id: &str,
    root_features: &BTreeSet<String>,
) -> Result<(RealmFeatures, VecDeque<RealmState>), CheckError> {
    let root = metadata
        .packages
        .get(root_id)
        .ok_or_else(|| CheckError::new(format!("root package ID {root_id:?} is missing")))?;
    let root_state = (root_id.to_owned(), Realm::Target);
    let features = BTreeMap::from([(
        root_state.clone(),
        local_feature_closure(root, root_features)?,
    )]);
    Ok((features, VecDeque::from([root_state])))
}

fn dependency_feature_roots(
    package: &Package,
    owner_features: &BTreeSet<String>,
    declaration: &Dependency,
) -> Option<BTreeSet<String>> {
    let dependency_key = declaration.rename.as_deref().unwrap_or(&declaration.name);
    let explicit_activation = format!("dep:{dependency_key}");
    let strong_feature_prefix = format!("{dependency_key}/");
    let weak_feature_prefix = format!("{dependency_key}?/");
    let mut active = !declaration.optional || owner_features.contains(dependency_key);
    let mut strong_features = BTreeSet::new();
    let mut weak_features = BTreeSet::new();
    for feature in owner_features {
        let Some(members) = package.features.get(feature) else {
            continue;
        };
        for member in members {
            if member == &explicit_activation {
                active = true;
            } else if let Some(dependency_feature) = member.strip_prefix(&weak_feature_prefix) {
                weak_features.insert(dependency_feature.to_owned());
            } else if let Some(dependency_feature) = member.strip_prefix(&strong_feature_prefix) {
                active = true;
                strong_features.insert(dependency_feature.to_owned());
            }
        }
    }
    if !active {
        return None;
    }

    let mut roots = declaration.features.clone();
    if declaration.uses_default_features {
        roots.insert("default".to_owned());
    }
    roots.extend(strong_features);
    roots.extend(weak_features);
    Some(roots)
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
    target_package: &Package,
    resolved_name: &str,
    resolved_kind: &NodeDependencyKind,
) -> Result<&'a Dependency, CheckError> {
    let candidates = package
        .dependencies
        .iter()
        .filter(|dependency| {
            dependency.name == target_package.name
                && dependency.kind == resolved_kind.kind
                && dependency
                    .rename
                    .as_ref()
                    .is_none_or(|rename| rename.replace('-', "_") == resolved_name)
        })
        .collect::<Vec<_>>();
    if let [dependency] = candidates.as_slice()
        && dependency.target.is_some() == resolved_kind.target.is_some()
    {
        // Cargo may reorder semantically equivalent cfg(any(...)) operands
        // between the package declaration and resolve edge. With one candidate
        // there is no identity ambiguity, and this judge includes every
        // conditional edge conservatively rather than evaluating the text.
        return Ok(dependency);
    }
    let exact = candidates
        .iter()
        .copied()
        .filter(|dependency| dependency.target == resolved_kind.target)
        .collect::<Vec<_>>();
    match exact.as_slice() {
        [dependency] => Ok(dependency),
        [] if candidates.is_empty() => Err(CheckError::new(format!(
            "resolve edge {} -> {resolved_name} has no exact manifest declaration for kind {:?} and target {:?}",
            package.name, resolved_kind.kind, resolved_kind.target
        ))),
        [] => Err(CheckError::new(format!(
            "resolve edge {} -> {resolved_name} cannot be paired unambiguously with {} same-kind target declarations",
            package.name,
            candidates.len()
        ))),
        _ => Err(CheckError::new(format!(
            "resolve edge {} -> {resolved_name} has ambiguous exact-target manifest declarations",
            package.name
        ))),
    }
}

fn reject_forbidden_packages(
    metadata: &CargoMetadata,
    states: &BTreeSet<(String, Realm)>,
    phase: GraphPhase,
    _configuration: GraphConfiguration,
) -> Result<(), CheckError> {
    for (id, realm) in states {
        let package = &metadata.packages[id];
        if package.name == "slint-build" {
            return Err(CheckError::new(format!(
                "slint-build is reachable in the {realm:?} graph"
            )));
        }
        if *realm == Realm::Host && is_slint_package(&package.name) {
            let allowed_host = if phase == GraphPhase::CompilerOnly {
                [COMPILER_PACKAGE, "i-slint-common"].contains(&package.name.as_str())
            } else {
                [
                    COMPILER_PACKAGE,
                    "i-slint-common",
                    "i-slint-core-macros",
                    SLINT_MACROS_PACKAGE,
                ]
                .contains(&package.name.as_str())
            };
            if !allowed_host {
                return Err(CheckError::new(format!(
                    "unexpected build-host Slint package {} in {phase}",
                    package.name
                )));
            }
        }
        if *realm == Realm::Host && package.name == "image" {
            return Err(CheckError::new(
                "image is build-host reachable; the compiler graph must not carry image codecs",
            ));
        }
        if phase == GraphPhase::CompilerOnly
            && *realm == Realm::Target
            && (package.name == "slint" || package.name.starts_with("i-slint-"))
        {
            return Err(CheckError::new(format!(
                "compiler-only phase contains target-reachable Slint runtime package {}",
                package.name
            )));
        }
        if phase == GraphPhase::FinalRuntime
            && [
                "i-slint-backend-linuxkms",
                "i-slint-backend-qt",
                "i-slint-backend-testing",
                "i-slint-live-preview",
                "i-slint-renderer-skia",
                "skia-bindings",
                "skia-safe",
            ]
            .contains(&package.name.as_str())
        {
            return Err(CheckError::new(format!(
                "forbidden runtime package {} is reachable in the {realm:?} graph",
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

fn require_target_only(
    states: &BTreeSet<(String, Realm)>,
    package: &Package,
    name: &str,
) -> Result<(), CheckError> {
    require_target_reachable(states, package, name)?;
    if states.contains(&(package.id.clone(), Realm::Host)) {
        return Err(CheckError::new(format!(
            "{name} is unexpectedly build-host reachable"
        )));
    }
    Ok(())
}

fn require_target_reachable(
    states: &BTreeSet<(String, Realm)>,
    package: &Package,
    name: &str,
) -> Result<(), CheckError> {
    if states.contains(&(package.id.clone(), Realm::Target)) {
        Ok(())
    } else {
        Err(CheckError::new(format!(
            "{name} is not reachable in the target-runtime graph"
        )))
    }
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
                dependency.name == edge.dependency_name
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
