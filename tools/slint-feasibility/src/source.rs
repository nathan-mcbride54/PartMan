use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::CheckError;
use crate::metadata::{CargoMetadata, Package, package_root};

const SOURCE_INVENTORY: &str = include_str!("../inventory/compiler-source-1.17.1.json");
const BUFFER_SIZE: usize = 64 * 1024;
const SLINT_LICENSE_EXPRESSION: &str =
    "GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0";

/// Successful compiler-source verification details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReport {
    /// Exact compiler package version.
    pub compiler_version: String,
    /// Verified upstream tag commit from `.cargo_vcs_info.json`.
    pub tag_commit: String,
    /// Verified deterministic published-tree digest.
    pub published_tree_sha256: String,
    /// Number of regular files included in the tree digest.
    pub file_count: usize,
}

#[derive(Debug)]
struct SourceInventory {
    compiler: PackageIdentity,
    spin_on: PackageIdentity,
    tag_commit: String,
    path_in_vcs: String,
    tree_hash: String,
    license_packages: Vec<LicensePackage>,
    critical_files: Vec<CriticalFile>,
}

#[derive(Debug)]
struct PackageIdentity {
    name: String,
    version: String,
    source: String,
    checksum: String,
}

#[derive(Debug)]
struct CriticalFile {
    path: String,
    sha256: String,
}

#[derive(Debug)]
struct LicensePackage {
    identity: PackageIdentity,
    license_expression: String,
    cargo_toml_sha256: String,
    files: Vec<LicenseFile>,
}

#[derive(Debug)]
struct LicenseFile {
    path: String,
    length: u64,
    sha256: String,
}

/// Verify locked compiler/`spin_on` identity and the exact published compiler tree.
///
/// # Errors
///
/// Fails closed on malformed committed inventory, package/source/checksum drift,
/// a missing registry tree, VCS identity drift, an unexpected filesystem entry,
/// a tree digest mismatch, or any critical-file mismatch.
pub fn verify_source(metadata: &CargoMetadata) -> Result<SourceReport, CheckError> {
    let inventory = parse_inventory(SOURCE_INVENTORY.as_bytes())?;
    let compiler = metadata.exact_package(&inventory.compiler.name)?;
    verify_package_identity(compiler, &inventory.compiler)?;
    let spin_on = metadata.exact_package(&inventory.spin_on.name)?;
    verify_package_identity(spin_on, &inventory.spin_on)?;
    verify_license_packages(metadata, &inventory.license_packages)?;

    let root = package_root(compiler)?;
    verify_vcs_info(root, &inventory)?;
    let (tree_hash, file_count) = published_tree_hash(root)?;
    if tree_hash != inventory.tree_hash {
        return Err(CheckError::new(format!(
            "i-slint-compiler published tree hash drifted: expected {}, found {tree_hash}",
            inventory.tree_hash
        )));
    }
    verify_critical_files(root, &inventory.critical_files)?;
    Ok(SourceReport {
        compiler_version: inventory.compiler.version,
        tag_commit: inventory.tag_commit,
        published_tree_sha256: tree_hash,
        file_count,
    })
}

fn verify_package_identity(
    package: &Package,
    expected: &PackageIdentity,
) -> Result<(), CheckError> {
    let actual_source = package
        .source
        .as_deref()
        .ok_or_else(|| CheckError::new(format!("{} has no registry source", expected.name)))?;
    if package.version != expected.version || actual_source != expected.source {
        return Err(CheckError::new(format!(
            "locked {} identity drifted: expected {} from {}, found {} from {}",
            expected.name, expected.version, expected.source, package.version, actual_source
        )));
    }
    let root = package_root(package)?;
    let archive = registry_archive_path(root, package)?;
    let actual_checksum = hash_file(&archive)?;
    if actual_checksum != expected.checksum {
        return Err(CheckError::new(format!(
            "{} .crate checksum drifted: expected {}, found {actual_checksum} at {}",
            expected.name,
            expected.checksum,
            archive.display()
        )));
    }
    Ok(())
}

fn registry_archive_path(root: &Path, package: &Package) -> Result<PathBuf, CheckError> {
    let expected_directory = format!("{}-{}", package.name, package.version);
    if root.file_name().and_then(std::ffi::OsStr::to_str) != Some(expected_directory.as_str()) {
        return Err(CheckError::new(format!(
            "registry source root does not match package identity: expected {expected_directory:?}, found {}",
            root.display()
        )));
    }
    let registry_index = root
        .parent()
        .ok_or_else(|| CheckError::new("registry package root has no index directory"))?;
    let registry_source = registry_index
        .parent()
        .ok_or_else(|| CheckError::new("registry index has no source directory"))?;
    if registry_source.file_name() != Some(std::ffi::OsStr::new("src")) {
        return Err(CheckError::new(format!(
            "registry source package is not beneath a registry/src directory: {}",
            root.display()
        )));
    }
    let registry_root = registry_source
        .parent()
        .ok_or_else(|| CheckError::new("registry source directory has no registry root"))?;
    let index_name = registry_index
        .file_name()
        .ok_or_else(|| CheckError::new("registry index directory has no name"))?;
    Ok(registry_root
        .join("cache")
        .join(index_name)
        .join(format!("{expected_directory}.crate")))
}

fn verify_license_packages(
    metadata: &CargoMetadata,
    expected_packages: &[LicensePackage],
) -> Result<(), CheckError> {
    for expected in expected_packages {
        let package = metadata.exact_package(&expected.identity.name)?;
        verify_package_identity(package, &expected.identity)?;
        if package.license.as_deref() != Some(expected.license_expression.as_str()) {
            return Err(CheckError::new(format!(
                "{} license expression drifted: expected {:?}, found {:?}",
                expected.identity.name, expected.license_expression, package.license
            )));
        }
        let root = package_root(package)?;
        let cargo_toml_hash = hash_file(&root.join("Cargo.toml"))?;
        if cargo_toml_hash != expected.cargo_toml_sha256 {
            return Err(CheckError::new(format!(
                "{} normalized Cargo.toml drifted: expected {}, found {cargo_toml_hash}",
                expected.identity.name, expected.cargo_toml_sha256
            )));
        }
        verify_license_file_roster(root, expected)?;
    }
    Ok(())
}

fn verify_license_file_roster(root: &Path, expected: &LicensePackage) -> Result<(), CheckError> {
    let license_root = root.join("LICENSES");
    let actual_paths = std::fs::read_dir(&license_root)
        .map_err(|error| {
            CheckError::new(format!(
                "cannot enumerate {}: {error}",
                license_root.display()
            ))
        })?
        .map(|entry| {
            let entry = entry.map_err(|error| {
                CheckError::new(format!(
                    "cannot enumerate {}: {error}",
                    license_root.display()
                ))
            })?;
            if !entry
                .file_type()
                .map_err(|error| {
                    CheckError::new(format!(
                        "cannot inspect {}: {error}",
                        entry.path().display()
                    ))
                })?
                .is_file()
            {
                return Err(CheckError::new(format!(
                    "license entry is not a regular file: {}",
                    entry.path().display()
                )));
            }
            normalized_relative_path(root, &entry.path())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected_paths = expected
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    if actual_paths != expected_paths {
        return Err(CheckError::new(format!(
            "{} packaged license roster drifted: expected {expected_paths:?}, found {actual_paths:?}",
            expected.identity.name
        )));
    }
    for file in &expected.files {
        let path = root.join(&file.path);
        let metadata = std::fs::metadata(&path).map_err(|error| {
            CheckError::new(format!("cannot inspect {}: {error}", path.display()))
        })?;
        if metadata.len() != file.length {
            return Err(CheckError::new(format!(
                "{} length drifted: expected {}, found {}",
                file.path,
                file.length,
                metadata.len()
            )));
        }
        let actual_hash = hash_file(&path)?;
        if actual_hash != file.sha256 {
            return Err(CheckError::new(format!(
                "{} hash drifted: expected {}, found {actual_hash}",
                file.path, file.sha256
            )));
        }
    }
    Ok(())
}

fn verify_vcs_info(root: &Path, inventory: &SourceInventory) -> Result<(), CheckError> {
    let path = root.join(".cargo_vcs_info.json");
    let bytes = std::fs::read(&path)
        .map_err(|error| CheckError::new(format!("cannot read {}: {error}", path.display())))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| CheckError::new(format!("cannot parse {}: {error}", path.display())))?;
    let root = value
        .as_object()
        .ok_or_else(|| CheckError::new(".cargo_vcs_info.json root is not an object"))?;
    let git = field(root, "git")?
        .as_object()
        .ok_or_else(|| CheckError::new(".cargo_vcs_info.json git field is not an object"))?;
    let commit = string_field(git, "sha1")?;
    let path_in_vcs = string_field(root, "path_in_vcs")?;
    if commit != inventory.tag_commit || path_in_vcs != inventory.path_in_vcs {
        return Err(CheckError::new(format!(
            "compiler VCS identity drifted: expected {} at {}, found {commit} at {path_in_vcs}",
            inventory.tag_commit, inventory.path_in_vcs
        )));
    }
    Ok(())
}

fn published_tree_hash(root: &Path) -> Result<(String, usize), CheckError> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

    let mut seen = BTreeSet::new();
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_SIZE];
    for (relative, absolute) in &files {
        if !seen.insert(relative.clone()) {
            return Err(CheckError::new(format!(
                "duplicate normalized source-tree path {relative:?}"
            )));
        }
        let metadata = std::fs::symlink_metadata(absolute).map_err(|error| {
            CheckError::new(format!("cannot inspect {}: {error}", absolute.display()))
        })?;
        if !metadata.file_type().is_file() {
            return Err(CheckError::new(format!(
                "source-tree entry is not a regular file: {}",
                absolute.display()
            )));
        }
        let path_bytes = relative.as_bytes();
        let path_length = u64::try_from(path_bytes.len())
            .map_err(|_| CheckError::new("source-tree path length does not fit u64"))?;
        hasher.update(path_length.to_be_bytes());
        hasher.update(path_bytes);
        hasher.update(metadata.len().to_be_bytes());

        let mut file = BufReader::new(File::open(absolute).map_err(|error| {
            CheckError::new(format!("cannot open {}: {error}", absolute.display()))
        })?);
        let mut bytes_read = 0_u64;
        loop {
            let count = file.read(&mut buffer).map_err(|error| {
                CheckError::new(format!("cannot read {}: {error}", absolute.display()))
            })?;
            if count == 0 {
                break;
            }
            bytes_read = bytes_read
                .checked_add(u64::try_from(count).expect("buffer count always fits u64"))
                .ok_or_else(|| CheckError::new("source file byte count overflowed u64"))?;
            hasher.update(&buffer[..count]);
        }
        if bytes_read != metadata.len() {
            return Err(CheckError::new(format!(
                "source file changed while hashing: {}",
                absolute.display()
            )));
        }
    }
    Ok((hex_digest(hasher.finalize()), files.len()))
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), CheckError> {
    let entries = std::fs::read_dir(directory).map_err(|error| {
        CheckError::new(format!("cannot enumerate {}: {error}", directory.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            CheckError::new(format!("cannot enumerate {}: {error}", directory.display()))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            CheckError::new(format!("cannot inspect {}: {error}", path.display()))
        })?;
        if file_type.is_symlink() {
            return Err(CheckError::new(format!(
                "compiler source tree contains a symbolic link: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            collect_files(root, &path, files)?;
        } else if file_type.is_file() {
            let relative = normalized_relative_path(root, &path)?;
            if relative != ".cargo-ok" {
                files.push((relative, path));
            }
        } else {
            return Err(CheckError::new(format!(
                "compiler source tree contains an unsupported entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn normalized_relative_path(root: &Path, path: &Path) -> Result<String, CheckError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| CheckError::new(format!("{} escaped source root", path.display())))?;
    let mut components = Vec::new();
    for component in relative.components() {
        let value = component.as_os_str().to_str().ok_or_else(|| {
            CheckError::new(format!("source-tree path is not UTF-8: {}", path.display()))
        })?;
        if value.is_empty() || value == "." || value == ".." || value.contains(['/', '\\']) {
            return Err(CheckError::new(format!(
                "source-tree path has an invalid component: {}",
                path.display()
            )));
        }
        components.push(value);
    }
    if components.is_empty() {
        return Err(CheckError::new(
            "source-tree file has an empty relative path",
        ));
    }
    Ok(components.join("/"))
}

fn verify_critical_files(root: &Path, files: &[CriticalFile]) -> Result<(), CheckError> {
    for expected in files {
        let path = root.join(&expected.path);
        let actual = hash_file(&path)?;
        if actual != expected.sha256 {
            return Err(CheckError::new(format!(
                "critical compiler file {} drifted: expected {}, found {actual}",
                expected.path, expected.sha256
            )));
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, CheckError> {
    let bytes = std::fs::read(path)
        .map_err(|error| CheckError::new(format!("cannot read {}: {error}", path.display())))?;
    Ok(hex_digest(Sha256::digest(bytes)))
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        use std::fmt::Write as _;
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn parse_inventory(bytes: &[u8]) -> Result<SourceInventory, CheckError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| CheckError::new(format!("invalid source inventory JSON: {error}")))?;
    let root = exact_object(
        &value,
        "source inventory",
        &[
            "schemaVersion",
            "compiler",
            "spinOn",
            "licensePackages",
            "criticalFiles",
        ],
    )?;
    if unsigned_field(root, "schemaVersion")? != 1 {
        return Err(CheckError::new(
            "unsupported source inventory schemaVersion",
        ));
    }
    let compiler = exact_object(
        field(root, "compiler")?,
        "compiler inventory",
        &[
            "package",
            "version",
            "registrySource",
            "crateSha256",
            "tagCommit",
            "pathInVcs",
            "publishedTreeSha256",
            "treeHashFraming",
        ],
    )?;
    let framing = string_field(compiler, "treeHashFraming")?;
    if framing
        != "For every regular file except root .cargo-ok, sort UTF-8 forward-slash relative-path bytes lexicographically, then hash u64be(path byte length), path bytes, u64be(content byte length), and raw content bytes; no prefix, count, or root name."
    {
        return Err(CheckError::new(
            "source inventory tree framing text drifted",
        ));
    }
    let spin_on = exact_object(
        field(root, "spinOn")?,
        "spin_on inventory",
        &["package", "version", "registrySource", "crateSha256"],
    )?;
    let critical_values = field(root, "criticalFiles")?
        .as_array()
        .ok_or_else(|| CheckError::new("criticalFiles is not an array"))?;
    let mut critical_files = Vec::new();
    let mut seen = BTreeSet::new();
    for value in critical_values {
        let object = exact_object(value, "critical file", &["path", "sha256"])?;
        let path = string_field(object, "path")?.to_owned();
        validate_relative_inventory_path(&path)?;
        if !seen.insert(path.clone()) {
            return Err(CheckError::new(format!(
                "duplicate critical compiler file {path:?}"
            )));
        }
        let sha256 = validated_sha256(string_field(object, "sha256")?)?;
        critical_files.push(CriticalFile { path, sha256 });
    }
    if critical_files.len() != 9 {
        return Err(CheckError::new(format!(
            "critical compiler file roster must contain exactly 9 entries, found {}",
            critical_files.len()
        )));
    }
    let license_packages = parse_license_packages(field(root, "licensePackages")?)?;
    Ok(SourceInventory {
        compiler: package_identity(compiler)?,
        spin_on: package_identity(spin_on)?,
        tag_commit: validated_sha1(string_field(compiler, "tagCommit")?)?,
        path_in_vcs: string_field(compiler, "pathInVcs")?.to_owned(),
        tree_hash: validated_sha256(string_field(compiler, "publishedTreeSha256")?)?,
        license_packages,
        critical_files,
    })
}

fn parse_license_packages(value: &Value) -> Result<Vec<LicensePackage>, CheckError> {
    let values = value
        .as_array()
        .ok_or_else(|| CheckError::new("licensePackages is not an array"))?;
    let mut packages = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let object = exact_object(
            value,
            "license package",
            &[
                "package",
                "version",
                "registrySource",
                "crateSha256",
                "licenseExpression",
                "normalizedCargoTomlSha256",
                "licenseFiles",
            ],
        )?;
        let identity = package_identity(object)?;
        if !seen.insert(identity.name.clone()) {
            return Err(CheckError::new(format!(
                "duplicate license package {:?}",
                identity.name
            )));
        }
        let files = parse_license_files(field(object, "licenseFiles")?)?;
        let license_expression = string_field(object, "licenseExpression")?.to_owned();
        if license_expression != SLINT_LICENSE_EXPRESSION {
            return Err(CheckError::new(format!(
                "license package {:?} expression drifted from the exact Royalty-free exception boundary",
                identity.name
            )));
        }
        packages.push(LicensePackage {
            identity,
            license_expression,
            cargo_toml_sha256: validated_sha256(string_field(
                object,
                "normalizedCargoTomlSha256",
            )?)?,
            files,
        });
    }
    let expected = [
        "i-slint-backend-selector",
        "i-slint-backend-winit",
        "i-slint-common",
        "i-slint-compiler",
        "i-slint-core",
        "i-slint-core-macros",
        "i-slint-renderer-femtovg",
        "i-slint-renderer-software",
        "slint",
        "slint-macros",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    if seen != expected {
        return Err(CheckError::new(format!(
            "license package roster drifted: expected {expected:?}, found {seen:?}"
        )));
    }
    Ok(packages)
}

fn parse_license_files(value: &Value) -> Result<Vec<LicenseFile>, CheckError> {
    let values = value
        .as_array()
        .ok_or_else(|| CheckError::new("licenseFiles is not an array"))?;
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let object = exact_object(value, "license file", &["path", "length", "sha256"])?;
        let path = string_field(object, "path")?.to_owned();
        validate_relative_inventory_path(&path)?;
        if !seen.insert(path.clone()) {
            return Err(CheckError::new(format!("duplicate license file {path:?}")));
        }
        files.push(LicenseFile {
            path,
            length: unsigned_field(object, "length")?,
            sha256: validated_sha256(string_field(object, "sha256")?)?,
        });
    }
    if files.is_empty() {
        return Err(CheckError::new("license file roster is empty"));
    }
    Ok(files)
}

fn package_identity(object: &Map<String, Value>) -> Result<PackageIdentity, CheckError> {
    Ok(PackageIdentity {
        name: string_field(object, "package")?.to_owned(),
        version: string_field(object, "version")?.to_owned(),
        source: string_field(object, "registrySource")?.to_owned(),
        checksum: validated_sha256(string_field(object, "crateSha256")?)?,
    })
}

fn validate_relative_inventory_path(path: &str) -> Result<(), CheckError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(CheckError::new(format!(
            "invalid relative inventory path {path:?}"
        )));
    }
    Ok(())
}

fn validated_sha256(value: &str) -> Result<String, CheckError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(CheckError::new(format!(
            "invalid lowercase SHA-256 value {value:?}"
        )));
    }
    Ok(value.to_owned())
}

fn validated_sha1(value: &str) -> Result<String, CheckError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(CheckError::new(format!(
            "invalid lowercase SHA-1 value {value:?}"
        )));
    }
    Ok(value.to_owned())
}

fn exact_object<'a>(
    value: &'a Value,
    context: &str,
    expected_keys: &[&str],
) -> Result<&'a Map<String, Value>, CheckError> {
    let object = value
        .as_object()
        .ok_or_else(|| CheckError::new(format!("{context} is not an object")))?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected_keys.iter().copied().collect::<BTreeSet<_>>();
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
        .ok_or_else(|| CheckError::new(format!("missing inventory field {name:?}")))
}

fn string_field<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, CheckError> {
    field(object, name)?
        .as_str()
        .ok_or_else(|| CheckError::new(format!("inventory field {name:?} is not a string")))
}

fn unsigned_field(object: &Map<String, Value>, name: &str) -> Result<u64, CheckError> {
    field(object, name)?
        .as_u64()
        .ok_or_else(|| CheckError::new(format!("inventory field {name:?} is not an integer")))
}

#[cfg(test)]
mod tests;
