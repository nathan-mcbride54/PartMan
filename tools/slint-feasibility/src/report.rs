//! Typed, byte-reproducible reporting for ADR-0009's decision gates.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;

use partman_domain::canonical::{Value as CanonicalValue, hash as canonical_hash};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;

use crate::CheckError;

const ADR_PATH: &str = "docs/adr/0009-bounded-slint-desktop-feasibility.md";
const EVIDENCE_PATH: &str = "docs/quality/slint-feasibility-data/evidence.json";
const REPORT_PATH: &str = "docs/quality/slint-feasibility.md";
const EXPECTED_GATE_IDS: [&str; 41] = [
    "G-CFG-01",
    "G-CFG-02",
    "G-CFG-03",
    "G-CFG-04",
    "G-CFG-05",
    "G-CFG-06",
    "G-CFG-07",
    "G-CFG-08",
    "G-PF-01",
    "G-PF-02",
    "G-PF-03",
    "G-PF-04",
    "G-PF-05",
    "G-PF-06",
    "G-PF-07",
    "G-PF-08",
    "C-PF-01",
    "G-AX-01",
    "G-AX-02",
    "G-AX-03",
    "G-AX-04",
    "G-AX-05",
    "G-AX-06",
    "G-AX-07",
    "G-AX-08",
    "G-AX-09",
    "G-AX-10",
    "G-PKG-01",
    "G-PKG-02",
    "G-PKG-03",
    "G-PKG-04",
    "G-INT-01",
    "G-SC-01",
    "G-SC-02",
    "G-LIC-01",
    "G-PERF-01",
    "G-PERF-02",
    "G-PERF-03",
    "G-PERF-04",
    "G-PERF-05",
    "C-PERF-01",
];

/// The stable facts returned after checking or writing the generated report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSummary {
    /// Mechanical decision derived from the gate outcomes.
    pub decision: String,
    /// Number of ADR gate rows rendered.
    pub gate_count: usize,
    /// `pce/1` SHA-256 of the complete normalized evidence input.
    pub raw_evidence_manifest_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvidenceManifest {
    schema_version: u64,
    observed_date: String,
    candidate: CandidateEvidence,
    hosts: Vec<HostEvidence>,
    sources: Vec<SourceObservation>,
    command_runs: Vec<CommandRun>,
    configurations: Vec<ConfigurationObservation>,
    supply_chain: SupplyChainEvidence,
    artifact_comparison: ArtifactComparison,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CandidateEvidence {
    source_commit: String,
    slint_version: String,
    cargo_lock_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HostEvidence {
    id: String,
    os_family: String,
    registry_product_name: String,
    display_version: String,
    build: String,
    architecture: String,
    cpu_identifier: String,
    logical_processors: u64,
    rustc: String,
    cargo: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceObservation {
    id: String,
    kind: String,
    url: String,
    observed_value: String,
    archive_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommandRun {
    id: String,
    host_id: String,
    source_commit: String,
    argv: Vec<String>,
    exit_code: i64,
    observation: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigurationObservation {
    id: String,
    shipping_eligible: bool,
    host_package_count: u64,
    target_package_count: u64,
    evaluated_target_predicates: u64,
    final_runtime_proven: bool,
    renderer_features: Vec<String>,
    runtime_closures: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SupplyChainEvidence {
    run_id: String,
    findings: Vec<SupplyChainFinding>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SupplyChainFinding {
    tool: String,
    code: String,
    package: String,
    version: String,
    classification: String,
    solution_available: bool,
    detail: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactComparison {
    host_id: String,
    scope: String,
    tauri_cargo_lock_sha256: String,
    tauri_package_lock_sha256: String,
    tauri_embedded_frontend_bytes: u64,
    artifacts: Vec<ArtifactObservation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactObservation {
    id: String,
    stack: String,
    configuration: String,
    source_commit: String,
    kind: String,
    bytes: u64,
    sha256: String,
    argv: Vec<String>,
    retention: String,
}

struct StrictJson(JsonValue);

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(JsonValue::Number)
            .map(StrictJson)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJson(JsonValue::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrictJson::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(StrictJson(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(StrictJson(JsonValue::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = serde_json::Map::new();
        while let Some((key, StrictJson(value))) = map.next_entry::<String, StrictJson>()? {
            if values.insert(key.clone(), value).is_some() {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key {key:?}"
                )));
            }
        }
        Ok(StrictJson(JsonValue::Object(values)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GateDefinition {
    id: String,
    objective: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateResult {
    Pass,
    Fail,
    Inconclusive,
}

impl GateResult {
    const fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GateEvaluation {
    result: GateResult,
    evidence: String,
    limitation: String,
}

/// Verify the fixed ADR/input/report paths, optionally replacing the generated
/// Markdown with the only rendering accepted by this tool.
///
/// # Errors
///
/// Returns an error for malformed or authority-bearing evidence, gate-registry
/// drift, a stale report, a failed write, or any referenced schema invariant.
pub fn verify_or_write_report(root: &Path, write: bool) -> Result<ReportSummary, CheckError> {
    if !root.is_absolute() {
        return Err(CheckError::new("report root must be an absolute path"));
    }
    let adr_path = root.join(ADR_PATH);
    let evidence_path = root.join(EVIDENCE_PATH);
    let report_path = root.join(REPORT_PATH);
    let adr = read_utf8(&adr_path)?;
    let evidence = read_utf8(&evidence_path)?;
    let gates = parse_gate_registry(&adr)?;
    let (manifest, raw_value) = parse_manifest(&evidence)?;
    validate_manifest(&manifest)?;
    let hash = canonical_manifest_hash(&raw_value)?;
    let (rendered, decision) = render_report(&gates, &manifest, &hash)?;

    if write {
        fs::write(&report_path, rendered.as_bytes()).map_err(|error| {
            CheckError::new(format!("cannot write {}: {error}", report_path.display()))
        })?;
    } else {
        let current = read_utf8(&report_path)?;
        if current != rendered {
            return Err(CheckError::new(format!(
                "{} is stale; run `cargo xtask slint-report --write`",
                report_path.display()
            )));
        }
    }

    Ok(ReportSummary {
        decision,
        gate_count: gates.len(),
        raw_evidence_manifest_hash: hash,
    })
}

fn read_utf8(path: &Path) -> Result<String, CheckError> {
    fs::read_to_string(path)
        .map_err(|error| CheckError::new(format!("cannot read {}: {error}", path.display())))
}

fn parse_manifest(text: &str) -> Result<(EvidenceManifest, JsonValue), CheckError> {
    let StrictJson(value) = serde_json::from_str(text).map_err(|error| {
        CheckError::new(format!("normalized evidence is invalid JSON: {error}"))
    })?;
    reject_authority_fields(&value, "$")?;
    let manifest = serde_json::from_value(value.clone()).map_err(|error| {
        CheckError::new(format!(
            "normalized evidence violates its strict schema: {error}"
        ))
    })?;
    Ok((manifest, value))
}

fn reject_authority_fields(value: &JsonValue, path: &str) -> Result<(), CheckError> {
    match value {
        JsonValue::Object(entries) => {
            for (key, child) in entries {
                if key.eq_ignore_ascii_case("pass") || key.eq_ignore_ascii_case("result") {
                    return Err(CheckError::new(format!(
                        "normalized evidence cannot own gate authority at {path}.{key}"
                    )));
                }
                reject_authority_fields(child, &format!("{path}.{key}"))?;
            }
        }
        JsonValue::Array(entries) => {
            for (index, child) in entries.iter().enumerate() {
                reject_authority_fields(child, &format!("{path}[{index}]"))?;
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }
    Ok(())
}

fn canonical_manifest_hash(value: &JsonValue) -> Result<String, CheckError> {
    let canonical = canonical_value(value)?;
    canonical_hash(&canonical)
        .map(|hash| hash.to_hex())
        .map_err(|error| CheckError::new(format!("cannot canonically hash evidence: {error}")))
}

fn canonical_value(value: &JsonValue) -> Result<CanonicalValue, CheckError> {
    match value {
        JsonValue::Null => Ok(CanonicalValue::Null),
        JsonValue::Bool(value) => Ok(CanonicalValue::Bool(*value)),
        JsonValue::String(value) => Ok(CanonicalValue::Text(value.clone())),
        JsonValue::Array(values) => values
            .iter()
            .map(canonical_value)
            .collect::<Result<Vec<_>, _>>()
            .map(CanonicalValue::Array),
        JsonValue::Object(values) => values
            .iter()
            .map(|(key, value)| Ok((key.clone(), canonical_value(value)?)))
            .collect::<Result<BTreeMap<_, _>, CheckError>>()
            .map(CanonicalValue::Map),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_u64() {
                Ok(CanonicalValue::Unsigned(value))
            } else if let Some(value) = number.as_i64().filter(|value| *value < 0) {
                Ok(CanonicalValue::Negative(value))
            } else {
                Err(CheckError::new(
                    "normalized evidence may contain only integer JSON numbers",
                ))
            }
        }
    }
}

fn parse_gate_registry(adr: &str) -> Result<Vec<GateDefinition>, CheckError> {
    let expected = EXPECTED_GATE_IDS.iter().copied().collect::<BTreeSet<_>>();
    let mut observed = BTreeMap::new();
    let mut duplicates = BTreeSet::new();

    for line in adr.lines() {
        let Some(cells) = line.strip_prefix('|') else {
            continue;
        };
        let cells = cells.split('|').map(str::trim).collect::<Vec<_>>();
        let Some(id) = cells.first().copied().filter(|id| is_gate_id(id)) else {
            continue;
        };
        let objective = cells.get(1).copied().unwrap_or_default();
        if objective.is_empty() {
            return Err(CheckError::new(format!(
                "ADR gate {id} has no eligibility assertion"
            )));
        }
        if observed
            .insert(id.to_owned(), objective.to_owned())
            .is_some()
        {
            duplicates.insert(id.to_owned());
        }
    }

    if !duplicates.is_empty() {
        return Err(CheckError::new(format!(
            "ADR gate registry contains duplicate IDs: {}",
            duplicates.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    let observed_ids = observed.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let missing = expected
        .difference(&observed_ids)
        .copied()
        .collect::<Vec<_>>();
    let unknown = observed_ids
        .difference(&expected)
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() || !unknown.is_empty() {
        return Err(CheckError::new(format!(
            "ADR gate registry drift: missing=[{}] unknown=[{}]",
            missing.join(", "),
            unknown.join(", ")
        )));
    }

    EXPECTED_GATE_IDS
        .iter()
        .map(|id| {
            observed
                .remove(*id)
                .map(|objective| GateDefinition {
                    id: (*id).to_owned(),
                    objective,
                })
                .ok_or_else(|| CheckError::new(format!("ADR gate {id} disappeared")))
        })
        .collect()
}

fn is_gate_id(value: &str) -> bool {
    (value.starts_with("G-") || value.starts_with("C-"))
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_manifest(manifest: &EvidenceManifest) -> Result<(), CheckError> {
    if manifest.schema_version != 1 {
        return Err(CheckError::new(format!(
            "unsupported evidence schema version {}",
            manifest.schema_version
        )));
    }
    validate_date(&manifest.observed_date)?;
    validate_commit(&manifest.candidate.source_commit, "candidate source commit")?;
    if manifest.candidate.slint_version != "1.17.1" {
        return Err(CheckError::new(format!(
            "ADR-0009 pins Slint 1.17.1, not {}",
            manifest.candidate.slint_version
        )));
    }
    validate_sha256(
        &manifest.candidate.cargo_lock_sha256,
        "candidate Cargo.lock",
    )?;

    let host_ids = validate_hosts(&manifest.hosts)?;
    validate_sources(&manifest.sources)?;
    validate_command_runs(&manifest.command_runs, &host_ids)?;
    validate_configurations(&manifest.configurations)?;
    validate_supply_chain(manifest)?;
    validate_artifacts(manifest, &host_ids)
}

fn validate_date(value: &str) -> Result<(), CheckError> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(CheckError::new(format!(
            "observed date {value:?} is not YYYY-MM-DD"
        )))
    }
}

fn validate_hosts(hosts: &[HostEvidence]) -> Result<BTreeSet<String>, CheckError> {
    let mut ids = BTreeSet::new();
    for host in hosts {
        require_nonempty(&host.id, "host id")?;
        if !ids.insert(host.id.clone()) {
            return Err(CheckError::new(format!("duplicate host id {}", host.id)));
        }
        for (value, label) in [
            (&host.os_family, "host OS family"),
            (&host.registry_product_name, "host registry product name"),
            (&host.display_version, "host display version"),
            (&host.build, "host build"),
            (&host.architecture, "host architecture"),
            (&host.cpu_identifier, "host CPU identifier"),
            (&host.rustc, "host rustc"),
            (&host.cargo, "host cargo"),
        ] {
            require_nonempty(value, label)?;
        }
        if host.logical_processors == 0 {
            return Err(CheckError::new(format!(
                "host {} has zero logical processors",
                host.id
            )));
        }
    }
    if ids.is_empty() {
        return Err(CheckError::new("normalized evidence contains no hosts"));
    }
    Ok(ids)
}

fn validate_sources(sources: &[SourceObservation]) -> Result<(), CheckError> {
    let mut ids = BTreeSet::new();
    for source in sources {
        require_nonempty(&source.id, "source id")?;
        if !ids.insert(source.id.clone()) {
            return Err(CheckError::new(format!(
                "duplicate source id {}",
                source.id
            )));
        }
        require_nonempty(&source.kind, "source kind")?;
        require_nonempty(&source.observed_value, "source observed value")?;
        require_nonempty(&source.archive_status, "source archive status")?;
        if !source.url.starts_with("https://") {
            return Err(CheckError::new(format!(
                "source {} does not use an HTTPS URL",
                source.id
            )));
        }
    }
    if ids.is_empty() {
        return Err(CheckError::new("normalized evidence contains no sources"));
    }
    Ok(())
}

fn validate_command_runs(
    runs: &[CommandRun],
    host_ids: &BTreeSet<String>,
) -> Result<(), CheckError> {
    let mut ids = BTreeSet::new();
    for run in runs {
        require_nonempty(&run.id, "command-run id")?;
        if !ids.insert(run.id.clone()) {
            return Err(CheckError::new(format!(
                "duplicate command-run id {}",
                run.id
            )));
        }
        if !host_ids.contains(&run.host_id) {
            return Err(CheckError::new(format!(
                "command run {} references unknown host {}",
                run.id, run.host_id
            )));
        }
        validate_commit(&run.source_commit, "command-run source commit")?;
        validate_argv(&run.argv, &format!("command run {}", run.id))?;
        require_nonempty(&run.observation, "command-run observation")?;
    }
    Ok(())
}

fn validate_configurations(configurations: &[ConfigurationObservation]) -> Result<(), CheckError> {
    let mut ids = BTreeSet::new();
    for configuration in configurations {
        if !ids.insert(configuration.id.clone()) {
            return Err(CheckError::new(format!(
                "duplicate configuration {}",
                configuration.id
            )));
        }
        let (shipping, features, runtime_proven) = match configuration.id.as_str() {
            "renderer-femtovg" => (true, ["renderer-femtovg"].as_slice(), true),
            "renderer-software" => (true, ["renderer-software"].as_slice(), true),
            "comparison-combined" => (
                false,
                ["renderer-femtovg", "renderer-software"].as_slice(),
                false,
            ),
            other => {
                return Err(CheckError::new(format!(
                    "unknown Slint configuration {other}"
                )));
            }
        };
        let actual_features = configuration
            .renderer_features
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        if configuration.shipping_eligible != shipping
            || actual_features != features
            || configuration.final_runtime_proven != runtime_proven
        {
            return Err(CheckError::new(format!(
                "configuration {} contradicts ADR-0009's closed renderer contract",
                configuration.id
            )));
        }
        if configuration.host_package_count == 0
            || configuration.target_package_count == 0
            || configuration.evaluated_target_predicates == 0
            || configuration.runtime_closures.is_empty()
            || configuration
                .runtime_closures
                .iter()
                .any(|value| value.trim().is_empty())
        {
            return Err(CheckError::new(format!(
                "configuration {} has an empty graph observation",
                configuration.id
            )));
        }
    }
    Ok(())
}

fn validate_supply_chain(manifest: &EvidenceManifest) -> Result<(), CheckError> {
    let run = manifest
        .command_runs
        .iter()
        .find(|run| run.id == manifest.supply_chain.run_id)
        .ok_or_else(|| {
            CheckError::new(format!(
                "supply-chain evidence references missing command run {}",
                manifest.supply_chain.run_id
            ))
        })?;
    if run.argv != ["cargo", "xtask", "supply-chain"] {
        return Err(CheckError::new(
            "supply-chain evidence does not reference `cargo xtask supply-chain`",
        ));
    }
    let mut findings = BTreeSet::new();
    for finding in &manifest.supply_chain.findings {
        for (value, label) in [
            (&finding.tool, "finding tool"),
            (&finding.code, "finding code"),
            (&finding.package, "finding package"),
            (&finding.version, "finding version"),
            (&finding.classification, "finding classification"),
            (&finding.detail, "finding detail"),
        ] {
            require_nonempty(value, label)?;
        }
        let identity = format!(
            "{}:{}:{}:{}",
            finding.tool, finding.code, finding.package, finding.version
        );
        if !findings.insert(identity.clone()) {
            return Err(CheckError::new(format!(
                "duplicate supply-chain finding {identity}"
            )));
        }
    }
    if run.exit_code == 0 && !manifest.supply_chain.findings.is_empty() {
        return Err(CheckError::new(
            "a successful supply-chain observation cannot carry blocking findings",
        ));
    }
    if run.exit_code != 0 && manifest.supply_chain.findings.is_empty() {
        return Err(CheckError::new(
            "a failed supply-chain observation must retain normalized findings",
        ));
    }
    Ok(())
}

fn validate_artifacts(
    manifest: &EvidenceManifest,
    host_ids: &BTreeSet<String>,
) -> Result<(), CheckError> {
    let comparison = &manifest.artifact_comparison;
    if !host_ids.contains(&comparison.host_id) {
        return Err(CheckError::new(format!(
            "artifact comparison references unknown host {}",
            comparison.host_id
        )));
    }
    if comparison.scope != "windows-release-executable-only" {
        return Err(CheckError::new(
            "artifact comparison scope must remain executable-only",
        ));
    }
    validate_sha256(&comparison.tauri_cargo_lock_sha256, "Tauri Cargo.lock")?;
    validate_sha256(
        &comparison.tauri_package_lock_sha256,
        "Tauri package-lock.json",
    )?;
    if comparison.tauri_embedded_frontend_bytes == 0 {
        return Err(CheckError::new(
            "Tauri embedded frontend byte observation is zero",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    for artifact in &comparison.artifacts {
        if !ids.insert(artifact.id.clone()) {
            return Err(CheckError::new(format!(
                "duplicate artifact id {}",
                artifact.id
            )));
        }
        for (value, label) in [
            (&artifact.stack, "artifact stack"),
            (&artifact.configuration, "artifact configuration"),
            (&artifact.kind, "artifact kind"),
        ] {
            require_nonempty(value, label)?;
        }
        validate_commit(&artifact.source_commit, "artifact source commit")?;
        validate_sha256(&artifact.sha256, "artifact")?;
        if !hashes.insert(artifact.sha256.clone()) {
            return Err(CheckError::new(format!(
                "two artifact records reuse SHA-256 {}",
                artifact.sha256
            )));
        }
        if artifact.bytes == 0 {
            return Err(CheckError::new(format!(
                "artifact {} has zero bytes",
                artifact.id
            )));
        }
        validate_argv(&artifact.argv, &format!("artifact {}", artifact.id))?;
        if artifact.retention != "local-uncommitted-measurement" {
            return Err(CheckError::new(format!(
                "artifact {} has unsupported retention {}; binary artifacts must not be committed",
                artifact.id, artifact.retention
            )));
        }
    }
    Ok(())
}

fn validate_argv(argv: &[String], label: &str) -> Result<(), CheckError> {
    if argv.is_empty() || argv.iter().any(String::is_empty) {
        Err(CheckError::new(format!(
            "{label} has an empty structured command"
        )))
    } else {
        Ok(())
    }
}

fn validate_commit(value: &str, label: &str) -> Result<(), CheckError> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CheckError::new(format!(
            "{label} is not a lowercase full Git object ID: {value}"
        )))
    }
}

fn validate_sha256(value: &str, label: &str) -> Result<(), CheckError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CheckError::new(format!(
            "{label} SHA-256 is not 64 lowercase hexadecimal digits: {value}"
        )))
    }
}

fn require_nonempty(value: &str, label: &str) -> Result<(), CheckError> {
    if value.trim().is_empty() {
        Err(CheckError::new(format!("{label} is empty")))
    } else {
        Ok(())
    }
}

fn render_report(
    gates: &[GateDefinition],
    manifest: &EvidenceManifest,
    manifest_hash: &str,
) -> Result<(String, String), CheckError> {
    let evaluations = gates
        .iter()
        .map(|gate| (gate, evaluate_gate(&gate.id, manifest, manifest_hash)))
        .collect::<Vec<_>>();
    let failed = evaluations
        .iter()
        .filter(|(gate, evaluation)| {
            gate.id.starts_with("G-") && evaluation.result == GateResult::Fail
        })
        .map(|(gate, _)| gate.id.as_str())
        .collect::<Vec<_>>();
    let inconclusive = evaluations
        .iter()
        .filter(|(gate, evaluation)| {
            gate.id.starts_with("G-") && evaluation.result == GateResult::Inconclusive
        })
        .map(|(gate, _)| gate.id.as_str())
        .collect::<Vec<_>>();
    let decision = if failed.is_empty() {
        if inconclusive.is_empty() {
            "eligible-for-adoption-decision"
        } else {
            "blocked-inconclusive"
        }
    } else {
        "rejected"
    };

    let mut out = String::new();
    render_report_header(&mut out, gates, manifest, manifest_hash, decision, &failed);
    render_hosts(&mut out, manifest);
    render_sources(&mut out, manifest);
    render_commands(&mut out, manifest);
    render_configurations(&mut out, manifest);
    render_supply_chain(&mut out, manifest);
    render_artifacts(&mut out, manifest)?;
    render_gate_table(&mut out, &evaluations);

    Ok((out, decision.to_owned()))
}

fn render_report_header(
    out: &mut String,
    gates: &[GateDefinition],
    manifest: &EvidenceManifest,
    manifest_hash: &str,
    decision: &str,
    failed: &[&str],
) {
    writeln!(
        out,
        "<!-- Generated by `cargo xtask slint-report`. Edit normalized evidence or the ADR, not this file. -->"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "# Slint feasibility decision report\n")
        .expect("writing to a String cannot fail");
    writeln!(out, "- Decision: **{}**", decision.replace('-', " "))
        .expect("writing to a String cannot fail");
    writeln!(
        out,
        "- Candidate checkpoint: `{}`",
        manifest.candidate.source_commit
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "- Slint pin: `{}`", manifest.candidate.slint_version)
        .expect("writing to a String cannot fail");
    writeln!(out, "- Evidence observed: `{}`", manifest.observed_date)
        .expect("writing to a String cannot fail");
    writeln!(out, "- Raw evidence manifest: `pce/1:{manifest_hash}`")
        .expect("writing to a String cannot fail");
    writeln!(
        out,
        "- Gate inventory: {} hard (`G-*`) and {} comparison-only (`C-*`) rows\n",
        gates
            .iter()
            .filter(|gate| gate.id.starts_with("G-"))
            .count(),
        gates
            .iter()
            .filter(|gate| gate.id.starts_with("C-"))
            .count()
    )
    .expect("writing to a String cannot fail");

    if failed.is_empty() {
        writeln!(
            out,
            "No hard gate has a recorded failure. Adoption is nevertheless blocked while any hard gate is inconclusive.\n"
        )
        .expect("writing to a String cannot fail");
    } else {
        writeln!(
            out,
            "ADR-0009 rejects the candidate because hard gate(s) {} fail. The remaining inconclusive gates are retained rather than silently skipped; no adoption claim is made.\n",
            failed.join(", ")
        )
        .expect("writing to a String cannot fail");
    }
    writeln!(
        out,
        "The report generator owns the outcome calculation. The normalized JSON contains observations and exit codes, but it is rejected if it contains a `pass` or `result` field.\n"
    )
    .expect("writing to a String cannot fail");
}

fn render_gate_table(out: &mut String, evaluations: &[(&GateDefinition, GateEvaluation)]) {
    writeln!(out, "## Exact ADR gate registry\n").expect("writing to a String cannot fail");
    writeln!(
        out,
        "| ID | Eligibility assertion | Evidence | Result | Limitation |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | --- | --- |").expect("writing to a String cannot fail");
    for (gate, evaluation) in evaluations {
        writeln!(
            out,
            "| {} | {} | {} | **{}** | {} |",
            gate.id,
            markdown_cell(&gate.objective),
            markdown_cell(&evaluation.evidence),
            evaluation.result.label(),
            markdown_cell(&evaluation.limitation)
        )
        .expect("writing to a String cannot fail");
    }
}

fn evaluate_gate(id: &str, manifest: &EvidenceManifest, manifest_hash: &str) -> GateEvaluation {
    let common = format!(
        "source={}; manifest=pce/1:{}",
        manifest.candidate.source_commit, manifest_hash
    );
    evaluate_decisive_gate(id, manifest, &common)
        .or_else(|| evaluate_partial_gate(id, manifest, &common))
        .unwrap_or_else(|| evaluate_uncollected_gate(id, manifest, &common))
}

fn evaluate_decisive_gate(
    id: &str,
    manifest: &EvidenceManifest,
    common: &str,
) -> Option<GateEvaluation> {
    match id {
        "G-CFG-02" if graph_checkpoint_is_complete(manifest) => GateEvaluation {
            result: GateResult::Pass,
            evidence: format!(
                "{common}; host={}; command={}; artifacts=none",
                command_host(manifest, "desktop"),
                command_text(manifest, "desktop")
            ),
            limitation: "Pass is limited to the locked Windows checkpoint and the exact resolver/source replay; any pin, feature, target, or lockfile drift reruns the gate.".to_owned(),
        }
        .into(),
        "G-CFG-08" | "G-SC-01" if supply_chain_failed(manifest) => GateEvaluation {
            result: GateResult::Fail,
            evidence: format!(
                "{common}; host={}; command={}; artifacts=none; findings={}",
                command_host(manifest, &manifest.supply_chain.run_id),
                command_text(manifest, &manifest.supply_chain.run_id),
                manifest
                    .supply_chain
                    .findings
                    .iter()
                    .map(|finding| format!("{}:{}@{}", finding.code, finding.package, finding.version))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            limitation: if id == "G-SC-01" {
                "The Windows policy run fails; a single required-platform failure is decisive even though macOS and Linux were not run.".to_owned()
            } else {
                "The workspace/lint/release checks pass, but this gate also requires the existing supply-chain policy checks to pass; they do not.".to_owned()
            },
        }
        .into(),
        _ => None,
    }
}

fn evaluate_partial_gate(
    id: &str,
    manifest: &EvidenceManifest,
    common: &str,
) -> Option<GateEvaluation> {
    match id {
        "G-PERF-02" | "C-PERF-01" => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host={}; command=separate release builds; artifacts={}",
                manifest.artifact_comparison.host_id,
                artifact_hashes(&manifest.artifact_comparison.artifacts)
            ),
            limitation: "Only Windows executable bytes were measured. They are not installer/package bytes, clean-system incremental runtime dependencies, paired runtime samples, or all-platform evidence, so no performance threshold is decided.".to_owned(),
        }
        .into(),
        "G-CFG-01" => inconclusive(
            common,
            manifest,
            "supply-chain",
            "The latest-release observations were not archived with the pinned security policy and advisory responses, and the policy run fails.",
        )
        .into(),
        "G-CFG-03" => inconclusive(
            common,
            manifest,
            "desktop",
            "AOT/source/environment checks pass, but the required clean minimal-environment and Qt-varied build matrix plus the complete hostile launch matrix was not normalized and retained.",
        )
        .into(),
        "G-CFG-04" => inconclusive(
            common,
            manifest,
            "desktop",
            "Single-renderer build graphs are proven, but no presented-frame and platform accessibility-root initialization evidence was retained.",
        )
        .into(),
        "G-CFG-05" => inconclusive(
            common,
            manifest,
            "ci",
            "Generated-token, catalogue, source, and lowered-IR checks pass, but separate high-contrast and rendered platform evidence is absent.",
        )
        .into(),
        "G-CFG-06" => inconclusive(
            common,
            manifest,
            "tier-1",
            "The renderer-neutral hostile identifier tests pass, but this normalized checkpoint does not retain the full corpus and per-case replay output required for a standalone gate pass.",
        )
        .into(),
        "G-CFG-07" => inconclusive(
            common,
            manifest,
            "ci",
            "Static boundaries and synthetic-only behavior are tested, but required runtime process/socket tracing and accessibility-IPC separation were not collected.",
        )
        .into(),
        "G-SC-02" => inconclusive(
            common,
            manifest,
            "desktop",
            "The resolver graph is replayed, but no complete SBOM, artifact inspection, or clean offline rebuild evidence exists for every required platform.",
        )
        .into(),
        "G-LIC-01" => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host=none; command=not run; artifacts=none uploaded"
            ),
            limitation: "No candidate binary was uploaded, so packaged notices and the required captured public attribution-badge readback were not evaluated.".to_owned(),
        }
        .into(),
        _ => None,
    }
}

fn evaluate_uncollected_gate(
    id: &str,
    manifest: &EvidenceManifest,
    common: &str,
) -> GateEvaluation {
    match id {
        gate if gate.starts_with("G-PF-") || gate == "C-PF-01" => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host=required platform image not observed; command=not run; artifacts=none"
            ),
            limitation: "The exact OS/architecture runtime, accessibility, scaling, renderer, launch, and stability qualification required by this row was not run.".to_owned(),
        },
        gate if gate.starts_with("G-AX-") => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host=required assistive-technology matrix not observed; command=not run; artifacts=none"
            ),
            limitation: "No complete platform-tree capture, assistive-technology transcript, rendered-state matrix, or renderer-qualified manual observation exists for this row.".to_owned(),
        },
        gate if gate.starts_with("G-PKG-") => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host=required clean installer VM not observed; command=not run; artifacts=none"
            ),
            limitation: "No installer, bundle, native package, clean-VM causal write trace, uninstall proof, or pre/post storage inventory was produced.".to_owned(),
        },
        "G-INT-01" => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host=required cross-platform dialog controls not observed; command=not run; artifacts=none"
            ),
            limitation: "Neither immutable candidate carries the bounded equivalent file-dialog control and path-race evidence required for this comparison.".to_owned(),
        },
        gate if gate.starts_with("G-PERF-") => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!(
                "{common}; host={}; command=required paired harness not run; artifacts={}",
                manifest.artifact_comparison.host_id,
                artifact_hashes(&manifest.artifact_comparison.artifacts)
            ),
            limitation: "The required all-platform paired samples, content-addressed harness, statistics, memory/process rows, and soak observations are absent; executable bytes alone cannot substitute for them.".to_owned(),
        },
        _ => GateEvaluation {
            result: GateResult::Inconclusive,
            evidence: format!("{common}; host=none; command=not run; artifacts=none"),
            limitation: "No complete normalized evidence was collected for this gate.".to_owned(),
        },
    }
}

fn inconclusive(
    common: &str,
    manifest: &EvidenceManifest,
    command_id: &str,
    limitation: &str,
) -> GateEvaluation {
    GateEvaluation {
        result: GateResult::Inconclusive,
        evidence: format!(
            "{common}; host={}; command={}; artifacts=none",
            command_host(manifest, command_id),
            command_text(manifest, command_id)
        ),
        limitation: limitation.to_owned(),
    }
}

fn command_run<'a>(manifest: &'a EvidenceManifest, id: &str) -> Option<&'a CommandRun> {
    manifest.command_runs.iter().find(|run| run.id == id)
}

fn command_host(manifest: &EvidenceManifest, id: &str) -> String {
    command_run(manifest, id).map_or_else(|| "not observed".to_owned(), |run| run.host_id.clone())
}

fn command_text(manifest: &EvidenceManifest, id: &str) -> String {
    command_run(manifest, id).map_or_else(
        || "not run".to_owned(),
        |run| format!("{} (exit {})", run.argv.join(" "), run.exit_code),
    )
}

fn graph_checkpoint_is_complete(manifest: &EvidenceManifest) -> bool {
    let expected = [
        "comparison-combined",
        "renderer-femtovg",
        "renderer-software",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = manifest
        .configurations
        .iter()
        .map(|configuration| configuration.id.as_str())
        .collect::<BTreeSet<_>>();
    actual == expected && command_run(manifest, "desktop").is_some_and(|run| run.exit_code == 0)
}

fn supply_chain_failed(manifest: &EvidenceManifest) -> bool {
    command_run(manifest, &manifest.supply_chain.run_id).is_some_and(|run| run.exit_code != 0)
}

fn artifact_hashes(artifacts: &[ArtifactObservation]) -> String {
    if artifacts.is_empty() {
        return "none".to_owned();
    }
    artifacts
        .iter()
        .map(|artifact| format!("{}:{}", artifact.id, artifact.sha256))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_hosts(out: &mut String, manifest: &EvidenceManifest) {
    writeln!(out, "## Normalized host and toolchain\n").expect("writing to a String cannot fail");
    writeln!(
        out,
        "The Windows registry product name is reproduced as observed; it is not treated as proof of a named ADR platform floor.\n"
    )
    .expect("writing to a String cannot fail");
    writeln!(
        out,
        "| ID | OS observation | Architecture / CPU | Toolchain |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | --- |").expect("writing to a String cannot fail");
    for host in &manifest.hosts {
        writeln!(
            out,
            "| {} | {} `{}` `{}` build `{}` | `{}`; `{}`; {} logical processors | `{}`; `{}` |",
            markdown_cell(&host.id),
            markdown_cell(&host.os_family),
            markdown_cell(&host.registry_product_name),
            markdown_cell(&host.display_version),
            markdown_cell(&host.build),
            markdown_cell(&host.architecture),
            markdown_cell(&host.cpu_identifier),
            host.logical_processors,
            markdown_cell(&host.rustc),
            markdown_cell(&host.cargo)
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
}

fn render_sources(out: &mut String, manifest: &EvidenceManifest) {
    writeln!(out, "## Upstream observations\n").expect("writing to a String cannot fail");
    writeln!(out, "| ID | Kind | Observation | Archive status |")
        .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | --- |").expect("writing to a String cannot fail");
    for source in &manifest.sources {
        writeln!(
            out,
            "| [{}]({}) | {} | {} | {} |",
            markdown_cell(&source.id),
            source.url,
            markdown_cell(&source.kind),
            markdown_cell(&source.observed_value),
            markdown_cell(&source.archive_status)
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
}

fn render_commands(out: &mut String, manifest: &EvidenceManifest) {
    writeln!(out, "## Checkpoint command observations\n").expect("writing to a String cannot fail");
    writeln!(
        out,
        "| ID | Host | Source | Structured command | Exit | Observation |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | --- | ---: | --- |")
        .expect("writing to a String cannot fail");
    for run in &manifest.command_runs {
        writeln!(
            out,
            "| {} | {} | `{}` | `{}` | {} | {} |",
            markdown_cell(&run.id),
            markdown_cell(&run.host_id),
            run.source_commit,
            markdown_cell(&run.argv.join(" ")),
            run.exit_code,
            markdown_cell(&run.observation)
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
}

fn render_configurations(out: &mut String, manifest: &EvidenceManifest) {
    writeln!(out, "## Locked renderer graph observations\n")
        .expect("writing to a String cannot fail");
    writeln!(
        out,
        "| Configuration | Shipping eligible | Host packages | Target packages | Target predicates | Final runtime proven | Renderer features | Required closures |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | ---: | ---: | ---: | --- | --- | --- |")
        .expect("writing to a String cannot fail");
    for configuration in &manifest.configurations {
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_cell(&configuration.id),
            configuration.shipping_eligible,
            configuration.host_package_count,
            configuration.target_package_count,
            configuration.evaluated_target_predicates,
            configuration.final_runtime_proven,
            markdown_cell(&configuration.renderer_features.join(", ")),
            markdown_cell(&configuration.runtime_closures.join(", "))
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
}

fn render_supply_chain(out: &mut String, manifest: &EvidenceManifest) {
    writeln!(out, "## Supply-chain findings\n").expect("writing to a String cannot fail");
    writeln!(
        out,
        "| Tool | Code | Package | Classification | Safe solution | Detail |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | --- | --- | --- |")
        .expect("writing to a String cannot fail");
    for finding in &manifest.supply_chain.findings {
        writeln!(
            out,
            "| {} | `{}` | `{}` `{}` | {} | {} | {} |",
            markdown_cell(&finding.tool),
            markdown_cell(&finding.code),
            markdown_cell(&finding.package),
            markdown_cell(&finding.version),
            markdown_cell(&finding.classification),
            finding.solution_available,
            markdown_cell(&finding.detail)
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
}

fn render_artifacts(out: &mut String, manifest: &EvidenceManifest) -> Result<(), CheckError> {
    writeln!(
        out,
        "## Windows executable-size observation (non-decisive)\n"
    )
    .expect("writing to a String cannot fail");
    writeln!(
        out,
        "Scope is `{}` on host `{}`. These are unstripped release executable bytes, not installer/package bytes or clean-system runtime-dependency bytes. The Tauri Vite output was {} bytes embedded in its executable and is not added a second time. Binary artifacts remain local and uncommitted; hashes identify the exact measured files.\n",
        manifest.artifact_comparison.scope,
        manifest.artifact_comparison.host_id,
        manifest.artifact_comparison.tauri_embedded_frontend_bytes
    )
    .expect("writing to a String cannot fail");
    writeln!(
        out,
        "Tauri lock hashes: Cargo.lock `{}`; package-lock.json `{}`. Slint candidate Cargo.lock `{}`.\n",
        manifest.artifact_comparison.tauri_cargo_lock_sha256,
        manifest.artifact_comparison.tauri_package_lock_sha256,
        manifest.candidate.cargo_lock_sha256
    )
    .expect("writing to a String cannot fail");
    let baseline = manifest
        .artifact_comparison
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "tauri-baseline")
        .ok_or_else(|| CheckError::new("artifact comparison has no tauri-baseline"))?;
    writeln!(
        out,
        "| Artifact | Stack / configuration | Source | Bytes | SHA-256 | Tauri ratio | Structured build command |"
    )
    .expect("writing to a String cannot fail");
    writeln!(out, "| --- | --- | --- | ---: | --- | ---: | --- |")
        .expect("writing to a String cannot fail");
    for artifact in &manifest.artifact_comparison.artifacts {
        let basis_points = ratio_basis_points(artifact.bytes, baseline.bytes)?;
        writeln!(
            out,
            "| {} | {} / {} | `{}` | {} | `{}` | {} bp (`{}.{:04}x`) | `{}` |",
            markdown_cell(&artifact.id),
            markdown_cell(&artifact.stack),
            markdown_cell(&artifact.configuration),
            artifact.source_commit,
            artifact.bytes,
            artifact.sha256,
            basis_points,
            basis_points / 10_000,
            basis_points % 10_000,
            markdown_cell(&artifact.argv.join(" "))
        )
        .expect("writing to a String cannot fail");
    }
    out.push('\n');
    Ok(())
}

fn ratio_basis_points(value: u64, baseline: u64) -> Result<u64, CheckError> {
    if baseline == 0 {
        return Err(CheckError::new("artifact ratio baseline is zero"));
    }
    value
        .checked_mul(10_000)
        .and_then(|scaled| scaled.checked_add(baseline / 2))
        .map(|rounded| rounded / baseline)
        .ok_or_else(|| CheckError::new("artifact ratio overflows u64"))
}

fn markdown_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::{
        EXPECTED_GATE_IDS, GateResult, canonical_manifest_hash, evaluate_gate, parse_gate_registry,
        parse_manifest, ratio_basis_points, validate_manifest, verify_or_write_report,
    };

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn evidence_text() -> String {
        std::fs::read_to_string(root().join("docs/quality/slint-feasibility-data/evidence.json"))
            .expect("read committed normalized evidence")
    }

    fn manifest() -> super::EvidenceManifest {
        let (manifest, _) = parse_manifest(&evidence_text()).expect("evidence parses");
        validate_manifest(&manifest).expect("evidence validates");
        manifest
    }

    // Requirements: SEC-010, Section 12
    //   The report registry is the ADR's exact closed set; missing, duplicate,
    //   and unknown gates cannot disappear behind a successful generation.
    // Work-Package: WP-030
    // Evidence: adr_gate_registry_is_exact_and_duplicate_closed
    #[test]
    fn adr_gate_registry_is_exact_and_duplicate_closed() {
        let synthetic = EXPECTED_GATE_IDS
            .iter()
            .map(|id| format!("| {id} | objective |"))
            .collect::<Vec<_>>()
            .join("\n");
        let gates = parse_gate_registry(&synthetic).expect("exact registry parses");
        assert_eq!(gates.len(), 41);

        let duplicate = format!("{synthetic}\n| G-CFG-01 | duplicate |");
        assert!(parse_gate_registry(&duplicate).is_err());
        let missing = synthetic.replace("| G-CFG-01 | objective |\n", "");
        assert!(parse_gate_registry(&missing).is_err());
        let unknown = format!("{synthetic}\n| G-NEW-01 | unknown |");
        assert!(parse_gate_registry(&unknown).is_err());
    }

    // Requirements: SEC-010, Section 12
    //   Evidence is duplicate-free, can describe observations but cannot inject
    //   the gate verdict, and uses PartMan's shared pce/1 implementation.
    // Work-Package: WP-030
    // Evidence: normalized_manifest_rejects_outcome_authority_and_hashes_canonically
    #[test]
    fn normalized_manifest_rejects_outcome_authority_and_hashes_canonically() {
        let (_, value) = parse_manifest(&evidence_text()).expect("evidence parses");
        let first = canonical_manifest_hash(&value).expect("hash evidence");
        let reordered = serde_json::from_str::<serde_json::Value>(
            &serde_json::to_string(&value).expect("serialize evidence"),
        )
        .expect("reparse evidence");
        assert_eq!(first, canonical_manifest_hash(&reordered).expect("rehash"));

        let mut authority = value;
        authority
            .as_object_mut()
            .expect("manifest is an object")
            .insert("result".to_owned(), json!("pass"));
        assert!(
            parse_manifest(&serde_json::to_string(&authority).expect("serialize authority"))
                .is_err()
        );

        let duplicate = evidence_text().replacen(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1,\n  \"schemaVersion\": 1,",
            1,
        );
        assert!(parse_manifest(&duplicate).is_err());
    }

    // Requirements: PKG-005, SEC-010, Section 12
    //   A failed required supply-chain command mechanically rejects the Slint
    //   candidate; normalized prose cannot downgrade it to a warning.
    // Work-Package: WP-030
    // Evidence: supply_chain_failure_mechanically_rejects_candidate
    #[test]
    fn supply_chain_failure_mechanically_rejects_candidate() {
        let manifest = manifest();
        for id in ["G-CFG-08", "G-SC-01"] {
            assert_eq!(
                evaluate_gate(id, &manifest, "0".repeat(64).as_str()).result,
                GateResult::Fail
            );
        }
    }

    // Requirements: SEC-010, Section 12
    //   Missing proof is inconclusive and can never be promoted to a pass.
    // Work-Package: WP-030
    // Evidence: missing_evidence_stays_inconclusive
    #[test]
    fn missing_evidence_stays_inconclusive() {
        let mut manifest = manifest();
        manifest.command_runs.retain(|run| run.id != "desktop");
        assert_eq!(
            evaluate_gate("G-CFG-02", &manifest, "0".repeat(64).as_str()).result,
            GateResult::Inconclusive
        );
        assert_eq!(
            evaluate_gate("G-PF-01", &manifest, "0".repeat(64).as_str()).result,
            GateResult::Inconclusive
        );
    }

    // Requirements: PKG-005, Section 12
    //   Footprint observations use exact integer bytes, deterministic rounding,
    //   and independent rows for all three renderer configurations.
    // Work-Package: WP-030
    // Evidence: executable_ratios_are_integer_and_renderer_distinct
    #[test]
    fn executable_ratios_are_integer_and_renderer_distinct() {
        let manifest = manifest();
        let baseline = 7_745_536;
        let observed = manifest
            .artifact_comparison
            .artifacts
            .iter()
            .filter(|artifact| artifact.stack == "slint")
            .map(|artifact| {
                (
                    artifact.configuration.as_str(),
                    ratio_basis_points(artifact.bytes, baseline).expect("ratio"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            observed,
            [
                ("renderer-femtovg", 14_276),
                ("renderer-software", 14_965),
                ("comparison-combined", 15_784),
            ]
        );
    }

    // Requirements: SEC-010, Section 12
    //   The committed Markdown is byte-identical to the only rendering derived
    //   from the current ADR and normalized observations.
    // Work-Package: WP-030
    // Evidence: committed_feasibility_report_is_byte_fresh
    #[test]
    fn committed_feasibility_report_is_byte_fresh() {
        let summary = verify_or_write_report(
            &std::fs::canonicalize(root()).expect("canonical repository root"),
            false,
        )
        .expect("committed report is fresh");
        assert_eq!(summary.decision, "rejected");
        assert_eq!(summary.gate_count, 41);
    }
}
