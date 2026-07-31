use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::{CargoMetadata, CheckError, GraphPhase};

const STDOUT_LIMIT: usize = 16 * 1024 * 1024;
const STDERR_LIMIT: usize = 1024 * 1024;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(30);
const CARGO_BANNER: &str = "cargo 1.96.0 (30a34c682 2026-05-25)";
const CARGO_RELEASE: &str = "1.96.0";
const CARGO_COMMIT_HASH: &str = "30a34c6821b57de0aaec83a901aca39f88f6778c";
const CARGO_COMMIT_DATE: &str = "2026-05-25";

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
/// Rejects ambiguous input modes, an untrusted Cargo identity, a non-absolute
/// manifest, unreadable/oversized replay input, unsupported final-runtime
/// collection, process timeout/failure/output overflow, or malformed metadata.
pub fn load_or_collect_metadata(
    metadata_path: Option<&Path>,
    manifest_path: Option<&Path>,
    phase: GraphPhase,
) -> Result<CargoMetadata, CheckError> {
    match (metadata_path, manifest_path) {
        (Some(path), None) => {
            let bytes = read_bounded_file(path, STDOUT_LIMIT)?;
            CargoMetadata::parse(&bytes)
        }
        (None, Some(manifest)) => {
            let bytes = collect_metadata(manifest, phase)?;
            CargoMetadata::parse(&bytes)
        }
        _ => Err(CheckError::new(
            "choose exactly --metadata FILE or --manifest ABSOLUTE",
        )),
    }
}

fn collect_metadata(manifest_path: &Path, phase: GraphPhase) -> Result<Vec<u8>, CheckError> {
    if phase == GraphPhase::FinalRuntime {
        return Err(CheckError::new(
            "this checkpoint cannot collect or prove a final-runtime graph",
        ));
    }
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
    verify_cargo_identity(&cargo_path, working_directory)?;
    let mut command = Command::new(&cargo_path);
    command
        .args([
            OsString::from("metadata"),
            OsString::from("--locked"),
            OsString::from("--offline"),
            OsString::from("--format-version"),
            OsString::from("1"),
            OsString::from("--no-default-features"),
            OsString::from("--manifest-path"),
            manifest_path.as_os_str().to_owned(),
        ])
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

fn verify_cargo_identity(cargo_path: &Path, working_directory: &Path) -> Result<(), CheckError> {
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

fn verify_cargo_identity_output(output: &[u8]) -> Result<(), CheckError> {
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
    for line in lines {
        if let Some(value) = line.strip_prefix("release: ") {
            set_identity_field(&mut release, value, "release")?;
        } else if let Some(value) = line.strip_prefix("commit-hash: ") {
            set_identity_field(&mut commit_hash, value, "commit-hash")?;
        } else if let Some(value) = line.strip_prefix("commit-date: ") {
            set_identity_field(&mut commit_date, value, "commit-date")?;
        }
    }
    require_identity_field(release, CARGO_RELEASE, "release")?;
    require_identity_field(commit_hash, CARGO_COMMIT_HASH, "commit-hash")?;
    require_identity_field(commit_date, CARGO_COMMIT_DATE, "commit-date")
}

fn set_identity_field<'a>(
    slot: &mut Option<&'a str>,
    value: &'a str,
    name: &str,
) -> Result<(), CheckError> {
    if slot.replace(value).is_some() {
        return Err(CheckError::new(format!(
            "Cargo identity contains duplicate {name} fields"
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
            "Cargo identity {name} differs from the pinned toolchain"
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
