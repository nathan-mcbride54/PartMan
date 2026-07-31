use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant};

use cargo_platform::Cfg;

use crate::graph::validate_phase_configuration;
use crate::{CargoMetadata, CheckError, GraphConfiguration, GraphPhase, TargetContext};

const STDOUT_LIMIT: usize = 16 * 1024 * 1024;
const STDERR_LIMIT: usize = 1024 * 1024;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(30);
const CARGO_BANNER: &str = "cargo 1.96.0 (30a34c682 2026-05-25)";
const CARGO_RELEASE: &str = "1.96.0";
const CARGO_COMMIT_HASH: &str = "30a34c6821b57de0aaec83a901aca39f88f6778c";
const CARGO_COMMIT_DATE: &str = "2026-05-25";
const RUSTC_BANNER: &str = "rustc 1.96.0 (ac68faa20 2026-05-25)";
const RUSTC_RELEASE: &str = "1.96.0";
const RUSTC_COMMIT_HASH: &str = "ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96";
const RUSTC_COMMIT_DATE: &str = "2026-05-25";

/// Load reviewed metadata bytes or collect locked offline metadata with Cargo.
///
/// Exactly one of `metadata_path` and `manifest_path` must be supplied. Replay
/// reads are bounded. Live collection uses only the Cargo executable selected
/// when this pinned verifier was compiled, authenticates its exact release and
/// commit, requires an absolute manifest path, invokes fixed structured
/// `cargo metadata` arguments, clears the environment before restoring a
/// minimal allow-list, bounds both output streams, and kills a command that
/// exceeds 30 seconds.
///
/// # Errors
///
/// Rejects ambiguous input modes, an untrusted Cargo/rustc identity, a
/// non-absolute manifest, unreadable/oversized replay input, incompatible
/// phase/configuration selection, process timeout/failure/output overflow, or
/// malformed metadata.
pub fn load_or_collect_metadata(
    metadata_path: Option<&Path>,
    manifest_path: Option<&Path>,
    phase: GraphPhase,
    configuration: GraphConfiguration,
) -> Result<(CargoMetadata, TargetContext), CheckError> {
    let target = load_native_target_context()?;
    let metadata = match (metadata_path, manifest_path) {
        (Some(path), None) => {
            let bytes = read_bounded_file(path, STDOUT_LIMIT)?;
            CargoMetadata::parse(&bytes)
        }
        (None, Some(manifest)) => {
            let bytes = collect_metadata(manifest, phase, configuration, &target)?;
            CargoMetadata::parse(&bytes)
        }
        _ => Err(CheckError::new(
            "choose exactly --metadata FILE or --manifest ABSOLUTE",
        )),
    }?;
    Ok((metadata, target))
}

fn collect_metadata(
    manifest_path: &Path,
    phase: GraphPhase,
    configuration: GraphConfiguration,
    target: &TargetContext,
) -> Result<Vec<u8>, CheckError> {
    validate_phase_configuration(phase, configuration)?;
    let cargo_path = trusted_cargo_path(Path::new(env!("CARGO")))?;
    let manifest_path = absolute_regular_file(manifest_path, "workspace manifest")?;
    if manifest_path.file_name() != Some(OsStr::new("Cargo.toml")) {
        return Err(CheckError::new(format!(
            "workspace manifest is not named Cargo.toml: {}",
            manifest_path.display()
        )));
    }
    let working_directory = manifest_path
        .parent()
        .ok_or_else(|| CheckError::new("workspace manifest has no parent directory"))?;
    let cargo_host = verify_cargo_identity(&cargo_path, working_directory)?;
    if cargo_host != target.name() {
        return Err(CheckError::new(format!(
            "Cargo host {cargo_host:?} differs from authenticated rustc target {:?}",
            target.name()
        )));
    }
    let mut cargo_arguments = vec![
        OsString::from("metadata"),
        OsString::from("--locked"),
        OsString::from("--offline"),
        OsString::from("--format-version"),
        OsString::from("1"),
        OsString::from("--filter-platform"),
        OsString::from(target.name()),
        OsString::from("--no-default-features"),
    ];
    if let Some(feature) = runtime_cargo_feature(configuration) {
        cargo_arguments.push(OsString::from("--features"));
        cargo_arguments.push(OsString::from(feature));
    }
    cargo_arguments.push(OsString::from("--manifest-path"));
    cargo_arguments.push(manifest_path.as_os_str().to_owned());

    let mut command = Command::new(&cargo_path);
    command
        .args(cargo_arguments)
        .current_dir(working_directory)
        .env_clear()
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    copy_minimal_environment(&mut command, &cargo_path);
    let output = run_bounded(command, "locked offline Cargo metadata")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckError::new(format!(
            "locked offline cargo metadata failed with {}: {}",
            output.status,
            stderr.trim()
        )));
    }
    Ok(output.stdout)
}

fn runtime_cargo_feature(configuration: GraphConfiguration) -> Option<&'static str> {
    match configuration {
        GraphConfiguration::CompilerOnly => None,
        GraphConfiguration::RendererFemtoVg => Some("partman-desktop/renderer-femtovg"),
        GraphConfiguration::RendererSoftware => Some("partman-desktop/renderer-software"),
        GraphConfiguration::ComparisonCombined => Some("partman-desktop/comparison-combined"),
    }
}

fn trusted_cargo_path(path: &Path) -> Result<PathBuf, CheckError> {
    trusted_cargo_path_against(path, Path::new(env!("CARGO")))
}

fn trusted_cargo_path_against(path: &Path, selected: &Path) -> Result<PathBuf, CheckError> {
    if path != selected {
        return Err(CheckError::new(format!(
            "Cargo executable differs from the compile-time selection: {}",
            path.display()
        )));
    }
    if !path.is_absolute() {
        return Err(CheckError::new(format!(
            "Cargo executable path must be absolute: {}",
            path.display()
        )));
    }
    let expected_names = if cfg!(windows) {
        ["cargo.exe", "cargo"]
    } else {
        ["cargo", "cargo"]
    };
    let name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        CheckError::new(format!(
            "Cargo executable name is not UTF-8: {}",
            path.display()
        ))
    })?;
    if !expected_names.contains(&name) {
        return Err(CheckError::new(format!(
            "explicit executable is not named cargo: {}",
            path.display()
        )));
    }
    let metadata = std::fs::metadata(path)
        .map_err(|error| CheckError::new(format!("cannot inspect {}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(CheckError::new(format!(
            "Cargo executable is not a regular file: {}",
            path.display()
        )));
    }
    // Preserve the supplied path. Rustup commonly exposes `cargo` as a symlink
    // to its proxy binary, whose dispatch depends on argv[0] remaining cargo.
    Ok(path.to_path_buf())
}

fn verify_cargo_identity(
    cargo_path: &Path,
    working_directory: &Path,
) -> Result<String, CheckError> {
    let mut command = Command::new(cargo_path);
    command
        .arg("-vV")
        .current_dir(working_directory)
        .env_clear()
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    copy_minimal_environment(&mut command, cargo_path);
    let output = run_bounded(command, "Cargo identity check")?;
    if !output.status.success() {
        return Err(CheckError::new(format!(
            "Cargo identity check failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    if !output.stderr.is_empty() {
        return Err(CheckError::new(format!(
            "Cargo identity check wrote unexpected stderr: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    verify_cargo_identity_output(&output.stdout)
}

fn verify_cargo_identity_output(output: &[u8]) -> Result<String, CheckError> {
    let text = std::str::from_utf8(output)
        .map_err(|error| CheckError::new(format!("Cargo identity is not UTF-8: {error}")))?;
    let mut lines = text.lines();
    if lines.next() != Some(CARGO_BANNER) {
        return Err(CheckError::new(
            "Cargo identity banner differs from the pinned toolchain",
        ));
    }

    let mut release = None;
    let mut commit_hash = None;
    let mut commit_date = None;
    let mut host = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("release: ") {
            set_identity_field(&mut release, value, "release")?;
        } else if let Some(value) = line.strip_prefix("commit-hash: ") {
            set_identity_field(&mut commit_hash, value, "commit-hash")?;
        } else if let Some(value) = line.strip_prefix("commit-date: ") {
            set_identity_field(&mut commit_date, value, "commit-date")?;
        } else if let Some(value) = line.strip_prefix("host: ") {
            set_identity_field(&mut host, value, "host")?;
        }
    }
    require_identity_field(release, CARGO_RELEASE, "release")?;
    require_identity_field(commit_hash, CARGO_COMMIT_HASH, "commit-hash")?;
    require_identity_field(commit_date, CARGO_COMMIT_DATE, "commit-date")?;
    let host = host
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CheckError::new("Cargo identity has no non-empty host field"))?;
    Ok(host.to_owned())
}

fn load_native_target_context() -> Result<TargetContext, CheckError> {
    let selected_rustc = selected_rustc_path();
    let rustc_path = trusted_rustc_path(&selected_rustc)?;
    let mut identity_command = Command::new(&rustc_path);
    identity_command
        .arg("-vV")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    copy_minimal_environment(&mut identity_command, &rustc_path);
    let identity_output = run_bounded(identity_command, "rustc identity check")?;
    require_success_without_stderr(&identity_output, "rustc identity check")?;
    let target_name = verify_rustc_identity_output(&identity_output.stdout)?;

    let mut cfg_command = Command::new(&rustc_path);
    cfg_command
        .arg("--print=cfg")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    copy_minimal_environment(&mut cfg_command, &rustc_path);
    let cfg_output = run_bounded(cfg_command, "rustc target-cfg inventory")?;
    require_success_without_stderr(&cfg_output, "rustc target-cfg inventory")?;
    let cfg_text = std::str::from_utf8(&cfg_output.stdout)
        .map_err(|error| CheckError::new(format!("rustc target cfg is not UTF-8: {error}")))?;
    let cfgs = cfg_text
        .lines()
        .map(|line| {
            Cfg::from_str(line).map_err(|error| {
                CheckError::new(format!("cannot parse rustc target cfg {line:?}: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if cfgs.is_empty() {
        return Err(CheckError::new("rustc target-cfg inventory is empty"));
    }
    TargetContext::new(target_name, cfgs)
}

fn trusted_rustc_path(path: &Path) -> Result<PathBuf, CheckError> {
    if path != selected_rustc_path() {
        return Err(CheckError::new(format!(
            "rustc executable differs from the compile-time selection: {}",
            path.display()
        )));
    }
    if !path.is_absolute() {
        return Err(CheckError::new(format!(
            "rustc executable path must be absolute: {}",
            path.display()
        )));
    }
    let expected_names = if cfg!(windows) {
        ["rustc.exe", "rustc"]
    } else {
        ["rustc", "rustc"]
    };
    let name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        CheckError::new(format!(
            "rustc executable name is not UTF-8: {}",
            path.display()
        ))
    })?;
    if !expected_names.contains(&name) {
        return Err(CheckError::new(format!(
            "compile-time executable is not named rustc: {}",
            path.display()
        )));
    }
    let metadata = std::fs::metadata(path)
        .map_err(|error| CheckError::new(format!("cannot inspect {}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(CheckError::new(format!(
            "rustc executable is not a regular file: {}",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn selected_rustc_path() -> PathBuf {
    let filename = if cfg!(windows) { "rustc.exe" } else { "rustc" };
    Path::new(env!("CARGO")).with_file_name(filename)
}

fn verify_rustc_identity_output(output: &[u8]) -> Result<String, CheckError> {
    let text = std::str::from_utf8(output)
        .map_err(|error| CheckError::new(format!("rustc identity is not UTF-8: {error}")))?;
    let mut lines = text.lines();
    if lines.next() != Some(RUSTC_BANNER) {
        return Err(CheckError::new(
            "rustc identity banner differs from the pinned toolchain",
        ));
    }
    let mut release = None;
    let mut commit_hash = None;
    let mut commit_date = None;
    let mut host = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("release: ") {
            set_identity_field(&mut release, value, "release")?;
        } else if let Some(value) = line.strip_prefix("commit-hash: ") {
            set_identity_field(&mut commit_hash, value, "commit-hash")?;
        } else if let Some(value) = line.strip_prefix("commit-date: ") {
            set_identity_field(&mut commit_date, value, "commit-date")?;
        } else if let Some(value) = line.strip_prefix("host: ") {
            set_identity_field(&mut host, value, "host")?;
        }
    }
    require_identity_field(release, RUSTC_RELEASE, "release")?;
    require_identity_field(commit_hash, RUSTC_COMMIT_HASH, "commit-hash")?;
    require_identity_field(commit_date, RUSTC_COMMIT_DATE, "commit-date")?;
    let host = host
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CheckError::new("rustc identity has no non-empty host field"))?;
    Ok(host.to_owned())
}

fn require_success_without_stderr(
    output: &BoundedOutput,
    operation: &str,
) -> Result<(), CheckError> {
    if !output.status.success() {
        return Err(CheckError::new(format!(
            "{operation} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    if !output.stderr.is_empty() {
        return Err(CheckError::new(format!(
            "{operation} wrote unexpected stderr: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

fn set_identity_field<'a>(
    slot: &mut Option<&'a str>,
    value: &'a str,
    name: &str,
) -> Result<(), CheckError> {
    if slot.replace(value).is_some() {
        return Err(CheckError::new(format!(
            "toolchain identity contains duplicate {name} fields"
        )));
    }
    Ok(())
}

fn require_identity_field(
    actual: Option<&str>,
    expected: &str,
    name: &str,
) -> Result<(), CheckError> {
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(CheckError::new(format!(
            "toolchain identity {name} differs from the pinned toolchain"
        )))
    }
}

fn absolute_regular_file(path: &Path, kind: &str) -> Result<PathBuf, CheckError> {
    if !path.is_absolute() {
        return Err(CheckError::new(format!(
            "{kind} path must be absolute: {}",
            path.display()
        )));
    }
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| CheckError::new(format!("cannot resolve {}: {error}", path.display())))?;
    let metadata = std::fs::metadata(&canonical).map_err(|error| {
        CheckError::new(format!("cannot inspect {}: {error}", canonical.display()))
    })?;
    if !metadata.is_file() {
        return Err(CheckError::new(format!(
            "{kind} is not a regular file: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn copy_minimal_environment(command: &mut Command, cargo_path: &Path) {
    for name in [
        "CARGO_HOME",
        "HOME",
        "LOCALAPPDATA",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "SYSTEMDRIVE",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
        "TMPDIR",
        "USERPROFILE",
        "WINDIR",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    if let Some(parent) = cargo_path.parent() {
        command.env("PATH", parent.as_os_str());
    }
}

struct BoundedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_bounded(mut command: Command, operation: &str) -> Result<BoundedOutput, CheckError> {
    let mut child = command
        .spawn()
        .map_err(|error| CheckError::new(format!("cannot launch {operation}: {error}")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CheckError::new("Cargo stdout pipe is missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CheckError::new("Cargo stderr pipe is missing"))?;
    let stdout_reader = std::thread::spawn(move || drain_bounded(stdout, STDOUT_LIMIT));
    let stderr_reader = std::thread::spawn(move || drain_bounded(stderr, STDERR_LIMIT));

    let deadline = Instant::now() + PROCESS_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => {
                let kill_result = child.kill();
                let wait_result = child.wait();
                let stdout_result = join_reader(stdout_reader, "stdout");
                let stderr_result = join_reader(stderr_reader, "stderr");
                return Err(CheckError::new(format!(
                    "cannot poll {operation}: {error}; kill={kill_result:?}; wait={wait_result:?}; stdout={stdout_result:?}; stderr={stderr_result:?}"
                )));
            }
        }
        if Instant::now() >= deadline {
            let kill_result = child.kill();
            let wait_result = child.wait();
            let stdout_result = join_reader(stdout_reader, "stdout");
            let stderr_result = join_reader(stderr_reader, "stderr");
            return Err(CheckError::new(format!(
                "{operation} exceeded the 30-second timeout; kill={kill_result:?}; wait={wait_result:?}; stdout={stdout_result:?}; stderr={stderr_result:?}"
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let (stdout, stdout_overflow) = join_reader(stdout_reader, "stdout")?;
    let (stderr, stderr_overflow) = join_reader(stderr_reader, "stderr")?;
    if stdout_overflow || stderr_overflow {
        return Err(CheckError::new(format!(
            "{operation} output exceeded bounds: stdout>{STDOUT_LIMIT}={stdout_overflow}, stderr>{STDERR_LIMIT}={stderr_overflow}"
        )));
    }
    Ok(BoundedOutput {
        status,
        stdout,
        stderr,
    })
}

fn drain_bounded(mut reader: impl Read, limit: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut retained = Vec::new();
    let mut overflow = false;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(retained.len());
        let keep = remaining.min(count);
        retained.extend_from_slice(&buffer[..keep]);
        overflow |= keep != count;
    }
    Ok((retained, overflow))
}

fn join_reader(
    handle: std::thread::JoinHandle<std::io::Result<(Vec<u8>, bool)>>,
    stream: &str,
) -> Result<(Vec<u8>, bool), CheckError> {
    handle
        .join()
        .map_err(|_| CheckError::new(format!("Cargo {stream} reader panicked")))?
        .map_err(|error| CheckError::new(format!("cannot read Cargo {stream}: {error}")))
}

fn read_bounded_file(path: &Path, limit: usize) -> Result<Vec<u8>, CheckError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| CheckError::new(format!("cannot inspect {}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(CheckError::new(format!(
            "metadata replay input is not a regular file: {}",
            path.display()
        )));
    }
    if metadata.len() > u64::try_from(limit).expect("output limit fits u64") {
        return Err(CheckError::new(format!(
            "metadata replay input exceeds {limit} bytes: {}",
            path.display()
        )));
    }
    std::fs::read(path)
        .map_err(|error| CheckError::new(format!("cannot read {}: {error}", path.display())))
}

#[cfg(test)]
mod tests;
