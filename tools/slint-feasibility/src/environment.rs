use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::CheckError;
use crate::metadata::{CargoMetadata, package_root};

#[allow(
    dead_code,
    reason = "the verifier deliberately imports the exact shared policy file but needs only its inventory constants"
)]
#[path = "../../../apps/desktop/build_support/environment.rs"]
mod app_environment_policy;

const ENVIRONMENT_INVENTORY: &str = include_str!("../inventory/environment-1.17.1.json");
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;

/// Successful source-derived environment inventory verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentInventory {
    /// Exact environment names observed in currently resolved production source.
    pub resolved_names: BTreeSet<String>,
    /// Exact ambient names that the shared guard must reject and rerun on.
    pub rejected_rerun_names: BTreeSet<String>,
    /// Exact upstream-created names that are not downstream ambient inputs.
    pub upstream_controlled_names: BTreeSet<String>,
}

#[derive(Debug)]
struct Inventory {
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    name: String,
    classification: String,
    rerun_input: bool,
    rationale: String,
    occurrences: BTreeSet<Occurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Occurrence {
    package: String,
    path: String,
    access: Access,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Access {
    RuntimeRead,
    CompileTimeRead,
    RuntimeWrite,
    CargoRustcEnvWrite,
    CargoRerunInput,
}

/// Verify exact environment accesses in every currently resolved Slint source package.
///
/// Production Rust source includes package-root modules and build scripts but
/// excludes `tests/`, `benches/`, and `examples/`. Actual environment accesses
/// are lexed from `std::env::{var,var_os,set_var,remove_var}`, `env!`,
/// `option_env!`, and Cargo rustc-env/rerun directives; comments and unrelated
/// identifiers do not count. Exact `(name, package, path, access)` sets must
/// equal the committed inventory subset for the resolved packages.
///
/// # Errors
///
/// Rejects malformed inventory, unknown classifications/access kinds,
/// duplicate names/occurrences, missing rationale or rerun coverage, unreadable
/// or oversized source, invalid UTF-8 source, a new environment access, or an
/// inventoried access that disappeared.
pub fn verify_environment_inventory(
    metadata: &CargoMetadata,
) -> Result<EnvironmentInventory, CheckError> {
    let inventory = parse_inventory(ENVIRONMENT_INVENTORY.as_bytes())?;
    if inventory
        .entries
        .iter()
        .any(|entry| entry.rationale.trim().is_empty())
    {
        return Err(CheckError::new(
            "environment inventory contains an empty rationale after parsing",
        ));
    }
    let entries_by_name = inventory
        .entries
        .iter()
        .map(|entry| (entry.name.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut resolved_packages = BTreeSet::new();
    let mut actual = BTreeSet::new();
    for package in metadata.packages.values() {
        if metadata.nodes.contains_key(&package.id) && is_slint_source_package(&package.name) {
            if package.version != "1.17.1" {
                return Err(CheckError::new(format!(
                    "resolved Slint source package {} has unreviewed version {}",
                    package.name, package.version
                )));
            }
            resolved_packages.insert(package.name.clone());
            let root = package_root(package)?;
            actual.extend(scan_package(&package.name, root)?);
        }
    }

    let expected = inventory
        .entries
        .iter()
        .flat_map(|entry| {
            entry
                .occurrences
                .iter()
                .filter(|occurrence| resolved_packages.contains(&occurrence.package))
                .cloned()
                .map(|occurrence| (entry.name.clone(), occurrence))
        })
        .collect::<BTreeSet<_>>();
    if actual != expected {
        let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
        let unexpected = actual.difference(&expected).cloned().collect::<Vec<_>>();
        return Err(CheckError::new(format!(
            "resolved Slint environment source inventory drifted; missing {missing:?}; unexpected {unexpected:?}"
        )));
    }

    let resolved_names = actual
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<String>>();
    let rejected_rerun_names = inventory
        .entries
        .iter()
        .filter(|entry| entry.rerun_input)
        .map(|entry| entry.name.clone())
        .collect::<BTreeSet<_>>();
    let mut shared_policy_names = app_environment_policy::KNOWN_SLINT_ENVIRONMENT_NAMES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    shared_policy_names.insert(app_environment_policy::DEP_MCU_EMBED_TEXTURES.to_owned());
    if rejected_rerun_names != shared_policy_names {
        return Err(CheckError::new(format!(
            "source inventory rerun set differs from the shared app policy: inventory {rejected_rerun_names:?}, app {shared_policy_names:?}"
        )));
    }
    let upstream_controlled_names = inventory
        .entries
        .iter()
        .filter(|entry| entry.classification == "upstream-controlled")
        .map(|entry| entry.name.clone())
        .collect();
    for name in &resolved_names {
        if !entries_by_name.contains_key(name.as_str()) {
            return Err(CheckError::new(format!(
                "resolved environment name {name:?} has no policy entry"
            )));
        }
    }
    Ok(EnvironmentInventory {
        resolved_names,
        rejected_rerun_names,
        upstream_controlled_names,
    })
}

fn is_slint_source_package(name: &str) -> bool {
    name == "slint" || name.starts_with("slint-") || name.starts_with("i-slint-")
}

fn scan_package(package: &str, root: &Path) -> Result<BTreeSet<(String, Occurrence)>, CheckError> {
    let mut paths = Vec::new();
    collect_rust_sources(root, root, &mut paths)?;
    let mut output = BTreeSet::new();
    for (relative, absolute) in paths {
        let metadata = std::fs::metadata(&absolute).map_err(|error| {
            CheckError::new(format!("cannot inspect {}: {error}", absolute.display()))
        })?;
        if metadata.len() > MAX_SOURCE_BYTES {
            return Err(CheckError::new(format!(
                "Slint source exceeds {} bytes: {}",
                MAX_SOURCE_BYTES,
                absolute.display()
            )));
        }
        let bytes = std::fs::read(&absolute).map_err(|error| {
            CheckError::new(format!("cannot read {}: {error}", absolute.display()))
        })?;
        let source = std::str::from_utf8(&bytes).map_err(|error| {
            CheckError::new(format!(
                "{} is not UTF-8 Rust source: {error}",
                absolute.display()
            ))
        })?;
        for (name, access) in discover_accesses(source)? {
            if name.starts_with("SLINT_") || name == "DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES" {
                output.insert((
                    name,
                    Occurrence {
                        package: package.to_owned(),
                        path: relative.clone(),
                        access,
                    },
                ));
            }
        }
    }
    Ok(output)
}

fn collect_rust_sources(
    root: &Path,
    directory: &Path,
    output: &mut Vec<(String, PathBuf)>,
) -> Result<(), CheckError> {
    for entry in std::fs::read_dir(directory).map_err(|error| {
        CheckError::new(format!("cannot enumerate {}: {error}", directory.display()))
    })? {
        let entry = entry.map_err(|error| {
            CheckError::new(format!("cannot enumerate {}: {error}", directory.display()))
        })?;
        let path = entry.path();
        let relative = normalized_path(root, &path)?;
        let first = relative.split('/').next().unwrap_or_default();
        if ["tests", "benches", "examples"].contains(&first) {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            CheckError::new(format!("cannot inspect {}: {error}", path.display()))
        })?;
        if file_type.is_symlink() {
            return Err(CheckError::new(format!(
                "Slint source contains a symbolic link: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_rust_sources(root, &path, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "rs") {
            output.push((relative, path));
        }
    }
    Ok(())
}

fn normalized_path(root: &Path, path: &Path) -> Result<String, CheckError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| CheckError::new(format!("{} escaped package root", path.display())))?;
    let components = relative
        .components()
        .map(|component| {
            component.as_os_str().to_str().ok_or_else(|| {
                CheckError::new(format!("non-UTF-8 Slint source path: {}", path.display()))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(components.join("/"))
}

#[derive(Debug)]
struct Literal {
    context: String,
    value: String,
}

fn discover_accesses(source: &str) -> Result<BTreeSet<(String, Access)>, CheckError> {
    let literals = rust_string_literals(source)?;
    let mut output = BTreeSet::new();
    for literal in literals {
        let value = &literal.value;
        if is_environment_name(value)
            && let Some(access) = access_from_context(&literal.context)
        {
            output.insert((value.clone(), access));
        }
        if let Some(rest) = value.strip_prefix("cargo:rustc-env=")
            && let Some(name) = rest.split('=').next()
            && is_environment_name(name)
        {
            output.insert((name.to_owned(), Access::CargoRustcEnvWrite));
        }
        if let Some(name) = value.strip_prefix("cargo:rerun-if-env-changed=")
            && is_environment_name(name)
        {
            output.insert((name.to_owned(), Access::CargoRerunInput));
        }
    }
    Ok(output)
}

fn is_environment_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn access_from_context(context: &str) -> Option<Access> {
    let compact = context
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.ends_with("option_env!(") || compact.ends_with("env!(") {
        Some(Access::CompileTimeRead)
    } else if compact.ends_with("std::env::var(")
        || compact.ends_with("std::env::var_os(")
        || compact.ends_with("env::var(")
        || compact.ends_with("env::var_os(")
    {
        Some(Access::RuntimeRead)
    } else if compact.ends_with("std::env::set_var(")
        || compact.ends_with("std::env::remove_var(")
        || compact.ends_with("env::set_var(")
        || compact.ends_with("env::remove_var(")
    {
        Some(Access::RuntimeWrite)
    } else {
        None
    }
}

fn rust_string_literals(source: &str) -> Result<Vec<Literal>, CheckError> {
    let bytes = source.as_bytes();
    let mut output = Vec::new();
    let mut code = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            code.push('\n');
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index)?;
            code.push(' ');
        } else if bytes[index] == b'\''
            && let Some(next) = skip_character_literal(source, index)
        {
            code.push_str("''");
            index = next;
        } else if bytes[index] == b'"' {
            let context = trailing_context(&code);
            let (value, next) = parse_quoted_string(bytes, index)?;
            output.push(Literal { context, value });
            code.push_str("\"\"");
            index = next;
        } else if let Some((hashes, quote)) = raw_string_start(bytes, index) {
            let context = trailing_context(&code);
            let (value, next) = parse_raw_string(bytes, quote, hashes)?;
            output.push(Literal { context, value });
            code.push_str("\"\"");
            index = next;
        } else {
            code.push(char::from(bytes[index]));
            index += 1;
        }
    }
    Ok(output)
}

fn skip_character_literal(source: &str, quote: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let content = quote.checked_add(1)?;
    let next = if *bytes.get(content)? == b'\\' {
        match *bytes.get(content + 1)? {
            b'x' => content.checked_add(4)?,
            b'u' if bytes.get(content + 2) == Some(&b'{') => {
                let closing_brace = bytes[content + 3..].iter().position(|byte| *byte == b'}')?;
                content + 4 + closing_brace
            }
            _ => content.checked_add(2)?,
        }
    } else {
        let character = source[content..].chars().next()?;
        content.checked_add(character.len_utf8())?
    };
    (bytes.get(next) == Some(&b'\'')).then_some(next + 1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> Result<usize, CheckError> {
    let mut depth = 1_u32;
    let mut index = start + 2;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"/*") {
            depth = depth
                .checked_add(1)
                .ok_or_else(|| CheckError::new("Rust block-comment depth overflowed"))?;
            index += 2;
        } else if bytes[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Ok(index);
            }
        } else {
            index += 1;
        }
    }
    Err(CheckError::new("unterminated Rust block comment"))
}

fn parse_quoted_string(bytes: &[u8], quote: usize) -> Result<(String, usize), CheckError> {
    let mut output = Vec::new();
    let mut index = quote + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                return String::from_utf8(output)
                    .map(|value| (value, index + 1))
                    .map_err(|_| CheckError::new("Rust string literal is not UTF-8"));
            }
            b'\\' => {
                let escaped = *bytes
                    .get(index + 1)
                    .ok_or_else(|| CheckError::new("unterminated Rust string escape"))?;
                match escaped {
                    b'"' | b'\\' => output.push(escaped),
                    b'n' => output.push(b'\n'),
                    b'r' => output.push(b'\r'),
                    b't' => output.push(b'\t'),
                    _ => {
                        output.push(b'\\');
                        output.push(escaped);
                    }
                }
                index += 2;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    Err(CheckError::new("unterminated Rust string literal"))
}

fn raw_string_start(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if bytes.get(start) != Some(&b'r') {
        return None;
    }
    let mut index = start + 1;
    while bytes.get(index) == Some(&b'#') {
        index += 1;
    }
    (bytes.get(index) == Some(&b'"')).then_some((index - start - 1, index))
}

fn parse_raw_string(
    bytes: &[u8],
    quote: usize,
    hashes: usize,
) -> Result<(String, usize), CheckError> {
    let content_start = quote + 1;
    let mut index = content_start;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes.get(index + 1..index + 1 + hashes) == Some(&bytes[quote - hashes..quote])
        {
            let value = std::str::from_utf8(&bytes[content_start..index])
                .map_err(|_| CheckError::new("raw Rust string literal is not UTF-8"))?
                .to_owned();
            return Ok((value, index + 1 + hashes));
        }
        index += 1;
    }
    Err(CheckError::new("unterminated raw Rust string literal"))
}

fn trailing_context(code: &str) -> String {
    let start = code
        .char_indices()
        .rev()
        .nth(127)
        .map_or(0, |(index, _)| index);
    code[start..].to_owned()
}

fn parse_inventory(bytes: &[u8]) -> Result<Inventory, CheckError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| CheckError::new(format!("invalid environment inventory JSON: {error}")))?;
    let root = exact_object(
        &value,
        "environment inventory",
        &["schemaVersion", "slintVersion", "entries"],
    )?;
    if unsigned(root, "schemaVersion")? != 1 || string(root, "slintVersion")? != "1.17.1" {
        return Err(CheckError::new(
            "unsupported environment inventory schema or Slint version",
        ));
    }
    let values = field(root, "entries")?
        .as_array()
        .ok_or_else(|| CheckError::new("environment entries is not an array"))?;
    let allowed_classifications = [
        "build-aot-rejected",
        "build-and-runtime-rejected",
        "forbidden-auxiliary",
        "forbidden-package",
        "resolved-excluded-path",
        "runtime-rejected",
        "software-runtime-rejected",
        "upstream-controlled",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut entries = Vec::new();
    let mut names = BTreeSet::new();
    for value in values {
        let object = exact_object(
            value,
            "environment entry",
            &[
                "name",
                "classification",
                "rerunInput",
                "rationale",
                "occurrences",
            ],
        )?;
        let name = string(object, "name")?.to_owned();
        if !(name.starts_with("SLINT_") || name == "DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES")
            || !is_environment_name(&name)
        {
            return Err(CheckError::new(format!(
                "invalid guarded environment name {name:?}"
            )));
        }
        if !names.insert(name.clone()) {
            return Err(CheckError::new(format!(
                "duplicate environment inventory name {name:?}"
            )));
        }
        let classification = string(object, "classification")?.to_owned();
        if !allowed_classifications.contains(classification.as_str()) {
            return Err(CheckError::new(format!(
                "unknown environment classification {classification:?}"
            )));
        }
        let rerun_input = boolean(object, "rerunInput")?;
        if classification == "upstream-controlled" {
            if rerun_input {
                return Err(CheckError::new(
                    "upstream-controlled environment value cannot be a downstream rerun input",
                ));
            }
        } else if !rerun_input {
            return Err(CheckError::new(format!(
                "ambient environment name {name:?} is missing rerun coverage"
            )));
        }
        let rationale = string(object, "rationale")?.to_owned();
        if rationale.trim().is_empty() {
            return Err(CheckError::new(format!(
                "environment name {name:?} has no rationale"
            )));
        }
        let occurrences = parse_occurrences(field(object, "occurrences")?)?;
        entries.push(Entry {
            name,
            classification,
            rerun_input,
            rationale,
            occurrences,
        });
    }
    if !names.contains("SLINT_WIDGETS_LIBRARY")
        || !names.contains("DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES")
    {
        return Err(CheckError::new(
            "environment inventory lacks the upstream widget value or forbidden MCU auxiliary",
        ));
    }
    Ok(Inventory { entries })
}

fn parse_occurrences(value: &Value) -> Result<BTreeSet<Occurrence>, CheckError> {
    let values = value
        .as_array()
        .ok_or_else(|| CheckError::new("environment occurrences is not an array"))?;
    let mut output = BTreeSet::new();
    for value in values {
        let object = exact_object(
            value,
            "environment occurrence",
            &["package", "path", "access"],
        )?;
        let occurrence = Occurrence {
            package: string(object, "package")?.to_owned(),
            path: string(object, "path")?.to_owned(),
            access: match string(object, "access")? {
                "runtime-read" => Access::RuntimeRead,
                "compile-time-read" => Access::CompileTimeRead,
                "runtime-write" => Access::RuntimeWrite,
                "cargo-rustc-env-write" => Access::CargoRustcEnvWrite,
                "cargo-rerun-input" => Access::CargoRerunInput,
                other => {
                    return Err(CheckError::new(format!(
                        "unknown environment access {other:?}"
                    )));
                }
            },
        };
        if occurrence.package.is_empty()
            || occurrence.path.is_empty()
            || occurrence.path.contains('\\')
            || occurrence
                .path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(CheckError::new(format!(
                "invalid environment occurrence {occurrence:?}"
            )));
        }
        if !output.insert(occurrence.clone()) {
            return Err(CheckError::new(format!(
                "duplicate environment occurrence {occurrence:?}"
            )));
        }
    }
    if output.is_empty() {
        return Err(CheckError::new("environment occurrence list is empty"));
    }
    Ok(output)
}

fn exact_object<'a>(
    value: &'a Value,
    context: &str,
    keys: &[&str],
) -> Result<&'a Map<String, Value>, CheckError> {
    let object = value
        .as_object()
        .ok_or_else(|| CheckError::new(format!("{context} is not an object")))?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = keys.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(CheckError::new(format!(
            "{context} keys differ: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(object)
}

fn field<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a Value, CheckError> {
    object
        .get(name)
        .ok_or_else(|| CheckError::new(format!("missing environment field {name:?}")))
}

fn string<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, CheckError> {
    field(object, name)?
        .as_str()
        .ok_or_else(|| CheckError::new(format!("environment field {name:?} is not a string")))
}

fn unsigned(object: &Map<String, Value>, name: &str) -> Result<u64, CheckError> {
    field(object, name)?
        .as_u64()
        .ok_or_else(|| CheckError::new(format!("environment field {name:?} is not an integer")))
}

fn boolean(object: &Map<String, Value>, name: &str) -> Result<bool, CheckError> {
    field(object, name)?
        .as_bool()
        .ok_or_else(|| CheckError::new(format!("environment field {name:?} is not a boolean")))
}

#[cfg(test)]
mod tests;
