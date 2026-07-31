use std::collections::BTreeSet;
use std::str::FromStr;

use cargo_platform::Cfg;

use super::{GraphConfiguration, GraphPhase, Realm, TargetContext, reachable_states, verify_graph};
use crate::CargoMetadata;

fn metadata(
    compiler_features: &str,
    extra_package: &str,
    target_slint_edge: &str,
) -> CargoMetadata {
    let input = format!(
        r#"{{
  "version": 1,
  "workspace_members": ["desktop 0.0.0 (path+file:///desktop)"],
  "packages": [
    {{"name":"partman-desktop","version":"0.0.0","id":"desktop 0.0.0 (path+file:///desktop)","license":"MIT OR Apache-2.0","source":null,"checksum":null,"manifest_path":"/desktop/Cargo.toml","targets":[{{"kind":["lib"]}}],"features":{{}},"dependencies":[
      {{"name":"i-slint-compiler","source":"registry+https://github.com/rust-lang/crates.io-index","req":"=1.17.1","kind":"build","rename":null,"optional":false,"uses_default_features":false,"features":["display-diagnostics","rust"],"target":null}},
      {{"name":"spin_on","source":"registry+https://github.com/rust-lang/crates.io-index","req":"=0.1.1","kind":"build","rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null}}
    ]}},
    {{"name":"i-slint-compiler","version":"1.17.1","id":"compiler 1.17.1","license":"GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"45ea275b15a425c7f2f77481151e1f5f8f1ea83feae580273090ef6b9e192218","manifest_path":"/registry/i-slint-compiler-1.17.1/Cargo.toml","targets":[{{"kind":["lib"]}}],"features":{{"default":[],"display-diagnostics":[],"rust":["quote","proc-macro2"],"quote":["dep:quote"],"proc-macro2":["dep:proc-macro2"],"software-renderer":[],"bundle-translations":[],"sdf-fonts":[]}},"dependencies":[{{"name":"typed-index-collections","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^3.2","kind":null,"rename":null,"optional":false,"uses_default_features":true,"features":[],"target":null}}]}},
    {{"name":"spin_on","version":"0.1.1","id":"spin 0.1.1","license":"MIT","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"076e103ed41b9864aa838287efe5f4e3a7a0362dd00671ae62a212e5e4612da2","manifest_path":"/registry/spin_on-0.1.1/Cargo.toml","targets":[{{"kind":["lib"]}}],"features":{{}},"dependencies":[]}},
    {{"name":"typed-index-collections","version":"3.5.0","id":"typed-index 3.5.0","license":"MIT OR Apache-2.0","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"898160f1dfd383b4e92e17f0512a7d62f3c51c44937b23b6ffc3a1614a8eaccd","manifest_path":"/registry/typed-index-collections-3.5.0/Cargo.toml","targets":[{{"kind":["lib"]}}],"features":{{"alloc":["serde?/alloc","bincode?/alloc"],"bincode":["dep:bincode"],"default":["alloc","std"],"serde":["dep:serde"],"serde-alloc":["alloc","serde"],"serde-std":["std","serde"],"std":["alloc","serde?/std","bincode?/std"]}},"dependencies":[{{"name":"bincode","source":"registry+https://github.com/rust-lang/crates.io-index","req":"^2.0.1","kind":null,"rename":null,"optional":true,"uses_default_features":false,"features":[],"target":null}}]}},
    {{"name":"bincode","version":"2.0.1","id":"bincode 2.0.1","license":"MIT","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"36eaf5d7b090263e8150820482d5d93cd964a81e4019913c972f4edcc6edb740","manifest_path":"/registry/bincode-2.0.1/Cargo.toml","targets":[{{"kind":["lib"]}}],"features":{{}},"dependencies":[]}}
    {extra_package}
  ],
  "resolve": {{"nodes": [
    {{"id":"desktop 0.0.0 (path+file:///desktop)","features":[],"deps":[
      {{"name":"i_slint_compiler","pkg":"compiler 1.17.1","dep_kinds":[{{"kind":"build","target":null}}]}},
      {{"name":"spin_on","pkg":"spin 0.1.1","dep_kinds":[{{"kind":"build","target":null}}]}}
      {target_slint_edge}
    ]}},
    {{"id":"compiler 1.17.1","features":[{compiler_features}],"deps":[{{"name":"typed_index_collections","pkg":"typed-index 3.5.0","dep_kinds":[{{"kind":null,"target":null}}]}}]}},
    {{"id":"spin 0.1.1","features":[],"deps":[]}},
    {{"id":"typed-index 3.5.0","features":["alloc","default","std"],"deps":[{{"name":"bincode","pkg":"bincode 2.0.1","dep_kinds":[{{"kind":null,"target":null}}]}}]}},
    {{"id":"bincode 2.0.1","features":["alloc","std"],"deps":[]}}
  ]}}
}}"#
    );
    CargoMetadata::parse(input.as_bytes()).expect("fixture metadata parses")
}

fn target_context() -> TargetContext {
    TargetContext::new("test-target".to_owned(), Vec::new()).expect("test target is valid")
}

// Requirements: SEC-010
//   Resolver-3 host and target capabilities are propagated independently, and foreign target predicates cannot make a package reachable
// Work-Package: WP-030
// Evidence: realm_features_and_target_predicates_are_independent
#[test]
fn realm_features_and_target_predicates_are_independent() {
    let input = br#"{
      "version":1,
      "workspace_members":["root"],
      "packages":[
        {"name":"root","version":"0.0.0","id":"root","license":"MIT","source":null,"checksum":null,"manifest_path":"/root/Cargo.toml","targets":[{"kind":["lib"]}],"features":{},"dependencies":[
          {"name":"shared","source":"registry+https://github.com/rust-lang/crates.io-index","req":"=1.0.0","kind":"build","rename":null,"optional":false,"uses_default_features":false,"features":["host"],"target":null},
          {"name":"shared","source":"registry+https://github.com/rust-lang/crates.io-index","req":"=1.0.0","kind":null,"rename":null,"optional":false,"uses_default_features":false,"features":["target"],"target":null},
          {"name":"foreign","source":"registry+https://github.com/rust-lang/crates.io-index","req":"=1.0.0","kind":null,"rename":null,"optional":false,"uses_default_features":false,"features":[],"target":"cfg(target_os = \"linux\")"}
        ]},
        {"name":"shared","version":"1.0.0","id":"shared","license":"MIT","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"x","manifest_path":"/shared/Cargo.toml","targets":[{"kind":["lib"]}],"features":{"host":[],"target":[]},"dependencies":[]},
        {"name":"foreign","version":"1.0.0","id":"foreign","license":"MIT","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"x","manifest_path":"/foreign/Cargo.toml","targets":[{"kind":["lib"]}],"features":{},"dependencies":[]}
      ],
      "resolve":{"nodes":[
        {"id":"root","features":[],"deps":[
          {"name":"shared","pkg":"shared","dep_kinds":[{"kind":"build","target":null},{"kind":null,"target":null}]},
          {"name":"foreign","pkg":"foreign","dep_kinds":[{"kind":null,"target":"cfg(target_os = \"linux\")"}]}
        ]},
        {"id":"shared","features":["host","target"],"deps":[]},
        {"id":"foreign","features":[],"deps":[]}
      ]}
    }"#;
    let metadata = CargoMetadata::parse(input).expect("realm fixture parses");
    let target = TargetContext::new(
        "x86_64-pc-windows-msvc".to_owned(),
        vec![Cfg::from_str("target_os=\"windows\"").expect("cfg parses")],
    )
    .expect("target context is valid");
    let reachability = reachable_states(&metadata, "root", &target, &BTreeSet::new())
        .expect("feature graph resolves");

    assert_eq!(
        reachability
            .features
            .get(&("shared".to_owned(), Realm::Host)),
        Some(&BTreeSet::from(["host".to_owned()]))
    );
    assert_eq!(
        reachability
            .features
            .get(&("shared".to_owned(), Realm::Target)),
        Some(&BTreeSet::from(["target".to_owned()]))
    );
    assert!(
        !reachability
            .states
            .contains(&("foreign".to_owned(), Realm::Target))
    );
    assert_eq!(reachability.evaluated_target_predicates.len(), 1);
}

// Requirements: SEC-010
//   Compiler-only metadata proves only exact build-host compiler capabilities and explicitly cannot become final runtime evidence
// Evidence: compiler_only_graph_is_host_separated_and_scope_limited
#[test]
fn compiler_only_graph_is_host_separated_and_scope_limited() {
    let metadata = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        "",
        "",
    );
    let report = verify_graph(
        &metadata,
        &target_context(),
        GraphPhase::CompilerOnly,
        GraphConfiguration::CompilerOnly,
    )
    .expect("clean graph passes");
    assert!(!report.final_runtime_proven);
    assert_eq!(report.host_package_count, 3);
    assert_eq!(report.target_package_count, 1);
    assert_eq!(
        report.lockfile_only_advisories,
        ["RUSTSEC-2025-0141".to_owned()].into_iter().collect()
    );
    assert!(
        verify_graph(
            &metadata,
            &target_context(),
            GraphPhase::FinalRuntime,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );
}

// Requirements: SEC-010
//   The exact bincode warning remains ignorable only while typed-index-collections declares one inactive optional edge with its reviewed feature table and no other reachable declarer
// Evidence: lockfile_only_bincode_advisory_fails_on_feature_or_declaration_drift
#[test]
fn lockfile_only_bincode_advisory_fails_on_feature_or_declaration_drift() {
    let clean = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        "",
        "",
    );

    let mut enabled = clean.clone();
    enabled
        .nodes
        .get_mut("typed-index 3.5.0")
        .expect("typed index node exists")
        .features
        .insert("bincode".to_owned());
    assert!(
        verify_graph(
            &enabled,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );

    let mut required = clean;
    required
        .packages
        .get_mut("typed-index 3.5.0")
        .expect("typed index package exists")
        .dependencies[0]
        .optional = false;
    assert!(
        verify_graph(
            &required,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );
}

// Requirements: SEC-010
//   A root cargo-audit exception is permitted only when every workspace member proves the advisory package unreachable, not merely the desktop member
// Work-Package: WP-030
// Evidence: lockfile_only_advisory_is_proven_from_every_workspace_member
#[test]
fn lockfile_only_advisory_is_proven_from_every_workspace_member() {
    let extra = r#",{
      "name":"partman-other",
      "version":"0.0.0",
      "id":"other 0.0.0 (path+file:///other)",
      "license":"MIT OR Apache-2.0",
      "source":null,
      "checksum":null,
      "manifest_path":"/other/Cargo.toml",
      "targets":[{"kind":["lib"]}],
      "features":{},
      "dependencies":[{
        "name":"bincode",
        "source":"registry+https://github.com/rust-lang/crates.io-index",
        "req":"^2.0.1",
        "kind":null,
        "rename":null,
        "optional":false,
        "uses_default_features":false,
        "features":[],
        "target":null
      }]
    }"#;
    let mut metadata = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        extra,
        "",
    );
    let other_id = "other 0.0.0 (path+file:///other)".to_owned();
    metadata.workspace_members.insert(other_id.clone());
    let mut other_node = metadata
        .nodes
        .get("spin 0.1.1")
        .expect("node fixture exists")
        .clone();
    other_node.dependencies = vec![
        metadata
            .nodes
            .get("typed-index 3.5.0")
            .expect("typed-index node exists")
            .dependencies[0]
            .clone(),
    ];
    metadata.nodes.insert(other_id, other_node);

    assert!(
        verify_graph(
            &metadata,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err(),
        "a second workspace root that reaches bincode must invalidate the root-wide audit exception"
    );
}

// Requirements: SEC-010
//   Compiler software rendering and translation bundling fail even when every package pin remains exact
// Evidence: compiler_capability_uplift_fails_closed
#[test]
fn compiler_capability_uplift_fails_closed() {
    for features in [
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\",\"software-renderer\"",
        "\"bundle-translations\",\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
    ] {
        let metadata = metadata(features, "", "");
        assert!(
            verify_graph(
                &metadata,
                &target_context(),
                GraphPhase::CompilerOnly,
                GraphConfiguration::CompilerOnly,
            )
            .is_err()
        );
    }
}

// Requirements: SEC-010
//   slint-build, build-host image codecs, and target runtime Slint packages cannot hide in Cargo's package list or dependency-kind presentation
// Evidence: forbidden_reachability_fails_closed
#[test]
fn forbidden_reachability_fails_closed() {
    let extra = r#",{"name":"slint","version":"1.17.1","id":"slint 1.17.1","license":"x","source":"registry+https://github.com/rust-lang/crates.io-index","checksum":"x","manifest_path":"/registry/slint/Cargo.toml","targets":[{"kind":["lib"]}],"features":{},"dependencies":[]}"#;
    let edge =
        r#",{"name":"slint","pkg":"slint 1.17.1","dep_kinds":[{"kind":null,"target":null}]}"#;
    let metadata = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        extra,
        edge,
    );
    assert!(
        verify_graph(
            &metadata,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );
}

// Requirements: SEC-010
//   Cargo's implicit hyphen-to-underscore crate naming is accepted while wrong or ambiguous explicit aliases remain failures
// Evidence: dependency_name_normalization_preserves_alias_strictness
#[test]
fn dependency_name_normalization_preserves_alias_strictness() {
    let clean = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        "",
        "",
    );
    assert!(
        verify_graph(
            &clean,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_ok()
    );

    let mut wrong = clean.clone();
    let desktop = wrong
        .packages
        .get_mut("desktop 0.0.0 (path+file:///desktop)")
        .expect("desktop package exists");
    desktop.dependencies[0].rename = Some("wrong_alias".to_owned());
    assert!(
        verify_graph(
            &wrong,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );

    let mut ambiguous = clean;
    let desktop = ambiguous
        .packages
        .get_mut("desktop 0.0.0 (path+file:///desktop)")
        .expect("desktop package exists");
    desktop.dependencies.push(desktop.dependencies[0].clone());
    assert!(
        verify_graph(
            &ambiguous,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );
}

// Requirements: SEC-010
//   Locked identities and manifest requirements are both exact, so lockfile state cannot conceal a widened or substituted compiler pin
// Evidence: compiler_and_executor_identity_drift_fails_closed
#[test]
fn compiler_and_executor_identity_drift_fails_closed() {
    let clean = metadata(
        "\"display-diagnostics\",\"proc-macro2\",\"quote\",\"rust\"",
        "",
        "",
    );

    let mut wrong_version = clean.clone();
    wrong_version
        .packages
        .get_mut("compiler 1.17.1")
        .expect("compiler package exists")
        .version = "1.17.2".to_owned();
    assert!(
        verify_graph(
            &wrong_version,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );

    let mut widened_requirement = clean;
    widened_requirement
        .packages
        .get_mut("desktop 0.0.0 (path+file:///desktop)")
        .expect("desktop package exists")
        .dependencies[0]
        .requirement = "^1.17.1".to_owned();
    assert!(
        verify_graph(
            &widened_requirement,
            &target_context(),
            GraphPhase::CompilerOnly,
            GraphConfiguration::CompilerOnly,
        )
        .is_err()
    );
}
