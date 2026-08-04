//! The SI-35 hardened-protocol instrument (WP-035).
//!
//! Two halves, matching the protocol's recorded mechanism amendment. The
//! privileged capture half runs one crate-owned `run_probed_session` per
//! preregistered schedule entry inside a fresh private scratch, with passive
//! `udevadm monitor` capture around each session, and emits raw JSON-line
//! records. The unprivileged projection half refuses elevation, records its
//! own negative environment assertions, applies the frozen normalizer — which
//! may drop exactly the six predeclared plumbing keys and nothing else — and
//! evaluates the protocol's control, stability, event, and decisive-pair
//! gates over the released records.
//!
//! The instrument is measurement logic only. It launches, under SAFE-004
//! controls at compiled absolute paths, exactly passive `udevadm monitor`
//! event capture and the three roster tools' `--version` probes for the
//! environment record; it addresses no device node by name, opens no block
//! device, and performs no storage mutation. Monitor output is captured as
//! evidence and never parsed for addressing.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use partman_fixtures::interlock::{self, Request};
use partman_fixtures::{catalogue, manifest};
use sha2::{Digest as _, Sha256};

use crate::TaskError;

const BASIC: &str = "gpt-basic-512.img";
const CONFLICTING: &str = "gpt-conflicting-tables-512.img";

/// Compiled absolute paths for the instrument's own launches. The roster is
/// the allow-list; nothing resolves through `PATH`.
const UDEVADM_PATH: &str = "/usr/bin/udevadm";
const BLKID_PATH: &str = "/usr/sbin/blkid";
const WIPEFS_PATH: &str = "/usr/sbin/wipefs";

/// Bound for one `--version` probe.
const VERSION_TIME_LIMIT: Duration = Duration::from_secs(5);

/// Bound for the passive monitor capture per session.
const MONITOR_CAPTURE_LIMIT: usize = 64 * 1024;

/// How long to let trailing uevents drain before stopping the monitor.
const MONITOR_DRAIN: Duration = Duration::from_millis(500);

/// The exact plumbing keys the frozen normalizer may drop: the six gate 4
/// preregistered, plus `DISKSEQ` per the recorded 2026-08-03 amendment — the
/// kernel-assigned monotone attach counter the first sitting discovered
/// varying per attach (22 → 26 across one root's controls), session plumbing
/// of exactly the class the original six name. Nothing else is ever dropped.
const NORMALIZER_DROPPED_KEYS: [&str; 7] = [
    "USEC_INITIALIZED",
    "ID_PART_ENTRY_DISK",
    "ID_LOOP_BACKING_FILENAME",
    "ID_LOOP_BACKING_FILENAME_ENC",
    "ID_LOOP_BACKING_INODE",
    "ID_LOOP_BACKING_DEVICE",
    "DISKSEQ",
];

/// Which generation root a schedule entry consumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Root {
    A,
    B,
}

/// The preregistered, compiled schedule: gate 5's negative controls first
/// (two distinct copies under different scratch roots and inodes, then a
/// repeat of the first), gate 8's order-balanced alternating trials, and a
/// closing healthy control. Compiled means committed before the first attach.
const SCHEDULE: [(&str, Root, &str); 10] = [
    ("NC1-healthy-a", Root::A, BASIC),
    ("NC2-healthy-b", Root::B, BASIC),
    ("NC3-healthy-a-repeat", Root::A, BASIC),
    ("T1-basic", Root::A, BASIC),
    ("T2-conflicting", Root::A, CONFLICTING),
    ("T3-basic", Root::A, BASIC),
    ("T4-conflicting", Root::A, CONFLICTING),
    ("T5-basic", Root::A, BASIC),
    ("T6-conflicting", Root::A, CONFLICTING),
    ("HC-close-a", Root::A, BASIC),
];

fn safety(message: String) -> TaskError {
    TaskError::Safety(message)
}

fn hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(rendered, "{byte:02x}");
    }
    rendered
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

// ---------------------------------------------------------------------------
// Privileged capture half
// ---------------------------------------------------------------------------

/// Run the capture half. The caller has already established native Linux,
/// absence of WSL markers, and effective UID zero; this function performs the
/// measurement mechanics and emits raw JSON-line records on stdout.
pub(crate) fn run_capture() -> Result<(), TaskError> {
    let token = std::env::var(interlock::TOKEN_VARIABLE).map_err(|_| {
        safety(format!(
            "no disposable-test token: set {} from the generated MANIFEST. Nothing was run",
            interlock::TOKEN_VARIABLE
        ))
    })?;

    for (label, path) in [
        ("udevadm", UDEVADM_PATH),
        ("blkid", BLKID_PATH),
        ("wipefs", WIPEFS_PATH),
    ] {
        if !std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
            return Err(safety(format!(
                "instrument tool {label} is absent from its compiled location. Nothing was run"
            )));
        }
    }

    // Gate 1: a private, fresh scratch with the exact required mode. A
    // pre-existing directory refuses rather than being reused.
    let scratch = std::env::temp_dir().join(format!("partman-si35-{}", std::process::id()));
    make_private_directory(&scratch)?;
    let root_a = scratch.join("a");
    let root_b = scratch.join("b");
    make_private_directory(&root_a)?;
    make_private_directory(&root_b)?;

    let manifest_a = catalogue::generate(&root_a)
        .map_err(|error| safety(format!("fixture generation failed in root a: {error}")))?;
    let manifest_b = catalogue::generate(&root_b)
        .map_err(|error| safety(format!("fixture generation failed in root b: {error}")))?;
    if manifest_a.token() != token || manifest_b.token() != token {
        return Err(safety(
            "the presented token does not match the generated manifests. Nothing was run"
                .to_owned(),
        ));
    }

    print_environment_record(&manifest_a)?;

    let mut sessions = 0_usize;
    for (label, root, fixture) in SCHEDULE {
        let root_path = match root {
            Root::A => &root_a,
            Root::B => &root_b,
        };
        let record = capture_one_session(label, root_path, fixture, &token)?;
        println!("{record}");
        sessions += 1;
    }

    // Gate-1 cleanup: only objects this run created inside its own scratch.
    std::fs::remove_dir_all(&scratch)
        .map_err(|error| safety(format!("scratch cleanup failed: {error}")))?;
    println!("si35-capture complete sessions={sessions}");
    Ok(())
}

/// Create one directory that must not already exist, with mode 0700, and
/// verify both facts back rather than assuming them.
fn make_private_directory(path: &Path) -> Result<(), TaskError> {
    use std::os::unix::fs::DirBuilderExt as _;
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| safety(format!("scratch directory not fresh: {error}")))?;
    let metadata = std::fs::metadata(path)
        .map_err(|error| safety(format!("scratch directory unreadable: {error}")))?;
    let mode = {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o777
    };
    if mode != 0o700 {
        return Err(safety(format!(
            "scratch directory mode {mode:o} is not the required 0700"
        )));
    }
    let mut entries =
        std::fs::read_dir(path).map_err(|error| safety(format!("scratch unreadable: {error}")))?;
    if entries.next().is_some() {
        return Err(safety("scratch directory is not empty".to_owned()));
    }
    Ok(())
}

/// Print the environment record the protocol requires, before the first
/// attach. Every digest here binds an input the run depends on.
fn print_environment_record(manifest_value: &manifest::Manifest) -> Result<(), TaskError> {
    println!("si35-capture environment record");
    println!("  schedule={SCHEDULE:?}");
    println!("  normalizer_dropped_keys={NORMALIZER_DROPPED_KEYS:?}");

    let exe = std::fs::read("/proc/self/exe")
        .map_err(|error| safety(format!("cannot read own binary for digest: {error}")))?;
    println!("  instrument_binary_sha256={}", sha256_hex(&exe));

    for (label, path) in [
        ("os_release", "/etc/os-release"),
        ("kernel_osrelease", "/proc/sys/kernel/osrelease"),
        ("kernel_version", "/proc/version"),
    ] {
        let content = std::fs::read_to_string(path)
            .map_err(|error| safety(format!("cannot read {path}: {error}")))?;
        let first = content.lines().next().unwrap_or("").trim();
        println!("  {label}={first}");
    }

    let osrelease = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map_err(|error| safety(format!("cannot read kernel osrelease: {error}")))?;
    let config = PathBuf::from(format!("/boot/config-{}", osrelease.trim()));
    match std::fs::read(&config) {
        Ok(bytes) => println!("  kernel_config_sha256={}", sha256_hex(&bytes)),
        Err(_) => println!("  kernel_config_sha256=absent"),
    }

    println!("  udev_rules_sha256={}", udev_rules_digest()?);

    for (label, path) in [
        ("udevadm", UDEVADM_PATH),
        ("blkid", BLKID_PATH),
        ("wipefs", WIPEFS_PATH),
    ] {
        let output = run_bounded(path, &["--version"], VERSION_TIME_LIMIT)?;
        let banner = String::from_utf8_lossy(&output);
        println!("  {label}_version={}", banner.lines().next().unwrap_or(""));
    }

    let mut names: Vec<&str> = manifest_value.names().collect();
    names.sort_unstable();
    for name in names {
        if let Some(entry) = manifest_value.entry(name) {
            println!("  fixture {name} sha256={}", entry.digest);
        }
    }
    Ok(())
}

/// Digest every udev rules file under the two canonical rules directories,
/// sorted by path, contents concatenated with their names.
fn udev_rules_digest() -> Result<String, TaskError> {
    let mut files = Vec::new();
    for base in ["/usr/lib/udev/rules.d", "/etc/udev/rules.d"] {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries {
            let entry = entry.map_err(|error| safety(format!("udev rules dir: {error}")))?;
            if entry.file_type().is_ok_and(|kind| kind.is_file()) {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.to_string_lossy().as_bytes());
        hasher.update([0]);
        let content =
            std::fs::read(&file).map_err(|error| safety(format!("udev rule read: {error}")))?;
        hasher.update(&content);
        hasher.update([0]);
    }
    Ok(hex(&hasher.finalize()))
}

/// Run one schedule entry: passive monitor on, one authorized crate session,
/// monitor off, one raw JSON record out.
fn capture_one_session(
    label: &str,
    root: &Path,
    fixture: &str,
    token: &str,
) -> Result<String, TaskError> {
    let mut monitor = spawn_monitor()?;

    let authorization = interlock::authorize(
        root,
        &Request {
            profile: Some(interlock::DESTRUCTIVE_PROFILE.to_owned()),
            token: Some(token.to_owned()),
            targets: vec![root.join(fixture)],
        },
    )
    .map_err(|refusal| {
        let _ = monitor.child.kill();
        safety(format!(
            "SAFE-007 refused schedule entry {label}: {refusal}. Nothing further was run"
        ))
    })?;

    let session = partman_ffi_linux_loop::run_probed_session(authorization);
    std::thread::sleep(MONITOR_DRAIN);
    let monitor_bytes = stop_monitor(monitor);

    let report = session.map_err(|refusal| {
        safety(format!(
            "schedule entry {label} refused: {refusal}; cleanup={}; fixtures={}; \
             remediation={}",
            refusal.cleanup_state(),
            refusal.fixture_state(),
            refusal.remediation()
        ))
    })?;

    let mut records = Vec::new();
    for record in report.records() {
        records.push(serde_json::json!({
            "tool": record.tool().label(),
            "subject": record.subject().to_string(),
            "exit_code": record.exit_code(),
            "stdout_hex": hex(record.stdout()),
            "stderr_hex": hex(record.stderr()),
        }));
    }
    let partitions: Vec<serde_json::Value> = report
        .partition_facts()
        .iter()
        .map(|facts| {
            serde_json::json!({
                "index": facts.index(),
                "start_sectors": facts.start_sectors(),
                "size_sectors": facts.size_sectors(),
                "read_only": facts.read_only(),
            })
        })
        .collect();

    let value = serde_json::json!({
        "entry": label,
        "fixture": fixture,
        "partitions_observed": report.partitions_observed(),
        "disk_facts": {
            "size_sectors": report.disk_facts().size_sectors(),
            "read_only": report.disk_facts().read_only(),
            "logical_block_size": report.disk_facts().logical_block_size(),
        },
        "partition_facts": partitions,
        "records": records,
        "monitor_hex": hex(&monitor_bytes),
    });
    Ok(value.to_string())
}

struct Monitor {
    child: std::process::Child,
    buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

/// Start the passive event listener and wait for its readiness banner before
/// returning, so the attach's first uevent burst cannot race the netlink
/// bind — the first sitting saw exactly that race (2 of 3 required add
/// events captured). Its output is evidence; nothing parses it for
/// addressing and no device is ever opened from it.
fn spawn_monitor() -> Result<Monitor, TaskError> {
    let mut command = std::process::Command::new(UDEVADM_PATH);
    command
        .args(["monitor", "--udev", "--kernel", "--subsystem-match=block"])
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| safety(format!("udevadm monitor failed to launch: {error}")))?;
    let stdout = child.stdout.take();
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = std::sync::Arc::clone(&buffer);
    std::thread::spawn(move || {
        if let Some(mut pipe) = stdout {
            let mut chunk = [0_u8; 4096];
            loop {
                match pipe.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        let mut collected = writer.lock().expect("monitor buffer lock");
                        if collected.len() < MONITOR_CAPTURE_LIMIT {
                            let remaining = MONITOR_CAPTURE_LIMIT - collected.len();
                            collected.extend_from_slice(&chunk[..read.min(remaining)]);
                        }
                    }
                }
            }
        }
    });

    // udevadm prints a banner once its sockets are bound; wait for it, so a
    // configure issued after this return cannot emit events the listener was
    // not yet subscribed to. A silent monitor refuses rather than racing.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        {
            let collected = buffer.lock().expect("monitor buffer lock");
            if !collected.is_empty() {
                break;
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(safety(
                "udevadm monitor produced no readiness output; refusing to race the \
                 first uevent"
                    .to_owned(),
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(Monitor { child, buffer })
}

/// Stop the listener and return what it captured.
fn stop_monitor(mut monitor: Monitor) -> Vec<u8> {
    let _ = monitor.child.kill();
    let _ = monitor.child.wait();
    let collected = monitor.buffer.lock().expect("monitor buffer lock");
    collected.clone()
}

/// Launch one bounded run-to-exit probe for the environment record.
fn run_bounded(path: &str, arguments: &[&str], limit: Duration) -> Result<Vec<u8>, TaskError> {
    let mut command = std::process::Command::new(path);
    command
        .args(arguments)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| safety(format!("version probe failed to launch: {error}")))?;
    let stdout = child.stdout.take();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut collected = Vec::new();
        if let Some(mut pipe) = stdout {
            let _ = pipe.read_to_end(&mut collected);
        }
        collected.truncate(4096);
        let _ = sender.send(collected);
    });
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(safety("version probe exceeded its time limit".to_owned()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                return Err(safety(format!("version probe wait failed: {error}")));
            }
        }
    }
    receiver
        .recv_timeout(limit)
        .map_err(|_| safety("version probe output was not delivered".to_owned()))
}

// ---------------------------------------------------------------------------
// Unprivileged projection half
// ---------------------------------------------------------------------------

/// One parsed session from the raw capture file.
struct Session {
    entry: String,
    fixture: String,
    partitions_observed: u64,
    facts_line: String,
    /// Per subject: the two udev info captures, in order.
    info_captures: Vec<(String, Vec<Vec<u8>>)>,
    privileged: Vec<(String, String, Vec<u8>)>,
    monitor: Vec<u8>,
}

/// Run the projection half over a raw capture file. Refuses elevation: this
/// half exists to be the unprivileged reader.
pub(crate) fn run_project(raw: &Path) -> Result<(), TaskError> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| safety(format!("cannot read own status: {error}")))?;
    let euid = crate::parse_effective_uid(&status)
        .ok_or_else(|| safety("cannot establish effective uid; failing closed".to_owned()))?;
    if euid == 0 {
        return Err(safety(
            "the projection half must run unprivileged; rerun as the measurement user".to_owned(),
        ));
    }

    println!("si35-project environment assertions");
    println!("  euid={euid}");
    print_negative_assertions(&status)?;

    let content = std::fs::read_to_string(raw)
        .map_err(|error| safety(format!("cannot read raw capture: {error}")))?;
    let sessions = parse_sessions(&content)?;
    if sessions.len() != SCHEDULE.len() {
        return Err(safety(format!(
            "raw capture has {} sessions; the schedule requires {}",
            sessions.len(),
            SCHEDULE.len()
        )));
    }

    let mut gates_passed = evaluate_stability_gate(&sessions);

    // The normalized projection per session: every subject's retained E:
    // lines plus the integer facts line.
    let projections: Vec<(String, String, String)> = sessions
        .iter()
        .map(|session| {
            (
                session.entry.clone(),
                session.fixture.clone(),
                normalize_session(session),
            )
        })
        .collect();

    // Gate 5 controls and the closing healthy control.
    let baseline = &projections[0].2;
    for index in [1_usize, 2, 9] {
        let (entry, _, projection) = &projections[index];
        let matches = projection == baseline;
        println!("gate control entry={entry} matches_baseline={matches}");
        if !matches {
            gates_passed = false;
        }
    }

    // Trial coherence: basic trials equal the baseline, conflicting trials
    // equal each other.
    for index in [3_usize, 5, 7] {
        let (entry, _, projection) = &projections[index];
        let matches = projection == baseline;
        println!("gate trial-basic entry={entry} matches_baseline={matches}");
        if !matches {
            gates_passed = false;
        }
    }
    let conflicting_baseline = &projections[4].2;
    for index in [6_usize, 8] {
        let (entry, _, projection) = &projections[index];
        let matches = projection == conflicting_baseline;
        println!("gate trial-conflicting entry={entry} coherent={matches}");
        if !matches {
            gates_passed = false;
        }
    }

    if !evaluate_event_gate(&sessions) {
        gates_passed = false;
    }

    // Gate 7's udev coverage gate, evaluated before the decisive pair is
    // allowed to mean anything. A successfully captured entry that retains no
    // property at all is a no-entry state, recorded `observed(absent)`; two
    // such projections compare equal, so without this the pair would report
    // `non-separating` on a coverage failure — a pass produced by measuring
    // nothing, which is exactly what the protocol forbids. Inability to
    // determine whether an entry exists cannot reach here: the capture half
    // refuses a udev query that exits outside its allowed set.
    let coverage = evaluate_coverage_gate(&sessions);

    // The decisive pair.
    let healthy = baseline;
    let conflicting = conflicting_baseline;
    if coverage.is_empty() {
        if healthy == conflicting {
            println!("decisive-pair candidate-projection=non-separating");
        } else {
            println!("decisive-pair candidate-projection=SEPARATES");
            for difference in projection_differences(healthy, conflicting) {
                println!("  differs: {difference}");
            }
        }
    } else {
        println!("decisive-pair candidate-projection=inconclusive (udev coverage gate)");
        for subject in &coverage {
            println!("  observed(absent): {subject}");
        }
        gates_passed = false;
    }

    // Privileged captures, labelled, never merged into the client projection.
    let healthy_privileged = privileged_rendering(&sessions[3]);
    let conflicting_privileged = privileged_rendering(&sessions[4]);
    println!(
        "privileged-comparison blkid-wipefs differ={}",
        healthy_privileged != conflicting_privileged
    );

    println!("gates all_pass={gates_passed}");
    if gates_passed {
        Ok(())
    } else {
        Err(safety(
            "one or more validity gates failed; every hypothesis row is void".to_owned(),
        ))
    }
}

/// Gate 7's stability check: each subject's two udev captures must be equal
/// after the one declared canonicalization — `DEVLINKS` value tokens sorted.
/// The first sitting measured udevadm rendering that property's symlink SET
/// in varying order between two back-to-back queries with identical token
/// sets; ordering is renderer nondeterminism, not device state, and the
/// canonicalization preserves every token.
fn evaluate_stability_gate(sessions: &[Session]) -> bool {
    let mut passed = true;
    for session in sessions {
        for (subject, captures) in &session.info_captures {
            // Stability is over the client projection gate 6 defines — the
            // `E:` property sequence — not over udevadm's whole rendering.
            // The second sitting measured the `S:` symlink block rendering
            // its set in varying order exactly as `DEVLINKS` does; those
            // lines duplicate the DEVLINKS set and are not projection.
            let stable = captures.len() == 2
                && projection_lines(&captures[0]) == projection_lines(&captures[1]);
            if !stable {
                println!(
                    "gate stability entry={} subject={subject} pass=false",
                    session.entry
                );
                passed = false;
            }
        }
    }
    println!("gate stability pass={passed}");
    passed
}

/// Gate 7 events, counting only; the capture never addresses anything. The
/// expected shape, measured on the second sitting: a preallocated loop node
/// emits **no disk add** — attach produces disk `change` events plus one
/// `add` per materialized partition. Require exactly that: udev adds at
/// least the partitions observed, and at least one udev `change` proving
/// the disk's configure event was processed.
fn evaluate_event_gate(sessions: &[Session]) -> bool {
    let mut passed = true;
    for session in sessions {
        let text = String::from_utf8_lossy(&session.monitor);
        let udev_adds = text
            .lines()
            .filter(|line| line.starts_with("UDEV") && line.contains(" add "))
            .count();
        let udev_changes = text
            .lines()
            .filter(|line| line.starts_with("UDEV") && line.contains(" change "))
            .count();
        let enough = udev_adds as u64 >= session.partitions_observed && udev_changes >= 1;
        println!(
            "gate events entry={} udev_adds={udev_adds} udev_changes={udev_changes} \
             required_adds={} pass={enough}",
            session.entry, session.partitions_observed
        );
        if !enough {
            passed = false;
        }
    }
    passed
}

/// Gate 7's udev coverage check. Returns every `entry/subject` whose
/// successfully captured udev entry retained no property at all — a no-entry
/// state, `observed(absent)`. A non-empty return makes the decisive pair
/// `inconclusive (udev coverage gate)` rather than allowing two empty
/// projections to compare equal and read as a negative result.
fn evaluate_coverage_gate(sessions: &[Session]) -> Vec<String> {
    let mut absent = Vec::new();
    for session in sessions {
        for (subject, captures) in &session.info_captures {
            let retained = captures
                .first()
                .map_or(0, |capture| projection_lines(capture).len());
            println!(
                "gate coverage entry={} subject={subject} retained_properties={retained}",
                session.entry
            );
            if retained == 0 {
                absent.push(format!("{}/{subject}", session.entry));
            }
        }
    }
    absent
}

/// The projection content of one capture: its `E: ` lines in order, with the
/// one declared `DEVLINKS` canonicalization applied. Everything else in
/// udevadm's rendering (`P:`, `N:`, `M:`, `S:` lines) is addressing or
/// duplicate-symlink presentation, not the udev property database.
fn projection_lines(capture: &[u8]) -> Vec<String> {
    canonicalize_info(capture)
        .lines()
        .filter(|line| line.starts_with("E: "))
        .map(str::to_owned)
        .collect()
}

/// The one declared rendering canonicalization: within an `E: DEVLINKS=`
/// line, sort the space-separated tokens. Every other byte is preserved.
fn canonicalize_info(capture: &[u8]) -> String {
    let text = String::from_utf8_lossy(capture);
    let mut canonical = String::with_capacity(text.len());
    for line in text.lines() {
        canonical.push_str(&canonicalize_devlinks_line(line));
        canonical.push('\n');
    }
    canonical
}

/// Sort the token set of one `DEVLINKS` property line; return others as-is.
fn canonicalize_devlinks_line(line: &str) -> String {
    let Some(value) = line.strip_prefix("E: DEVLINKS=") else {
        return line.to_owned();
    };
    let mut tokens: Vec<&str> = value.split_whitespace().collect();
    tokens.sort_unstable();
    format!("E: DEVLINKS={}", tokens.join(" "))
}

/// Record the unprivileged reader's negative assertions: no disk group, no
/// capabilities, and a denied direct loop read.
fn print_negative_assertions(status: &str) -> Result<(), TaskError> {
    let groups_line = status
        .lines()
        .find_map(|line| line.strip_prefix("Groups:"))
        .unwrap_or("")
        .trim()
        .to_owned();
    println!("  groups={groups_line}");
    let disk_gid = std::fs::read_to_string("/etc/group")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                let mut fields = line.split(':');
                if fields.next() == Some("disk") {
                    fields.nth(1).map(str::to_owned)
                } else {
                    None
                }
            })
        });
    match disk_gid {
        Some(gid) => {
            let in_disk = groups_line.split_whitespace().any(|value| value == gid);
            println!("  disk_gid={gid} member={in_disk}");
            if in_disk {
                return Err(safety(
                    "the measurement user is in the disk group; the baseline is not \
                     unprivileged"
                        .to_owned(),
                ));
            }
        }
        None => println!("  disk_gid=unresolved"),
    }

    let cap_eff = status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:"))
        .unwrap_or("")
        .trim();
    println!("  cap_eff={cap_eff}");
    if !matches!(u128::from_str_radix(cap_eff, 16), Ok(0)) {
        return Err(safety(
            "the measurement user holds effective capabilities; the baseline is not \
             unprivileged"
                .to_owned(),
        ));
    }

    for node in ["/dev/loop-control", "/dev/loop0"] {
        match File::open(node) {
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                println!("  direct_read {node}=denied");
            }
            Err(error) => {
                println!("  direct_read {node}=error({})", error.kind());
                return Err(safety(format!(
                    "direct-read denial not established for {node}"
                )));
            }
            Ok(_) => {
                return Err(safety(format!(
                    "direct read of {node} succeeded; the baseline is not unprivileged"
                )));
            }
        }
    }
    Ok(())
}

fn parse_sessions(content: &str) -> Result<Vec<Session>, TaskError> {
    let mut sessions = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|error| safety(format!("raw record is not valid JSON: {error}")))?;
        let entry = value["entry"]
            .as_str()
            .ok_or_else(|| safety("raw record has no entry".to_owned()))?
            .to_owned();
        let fixture = value["fixture"]
            .as_str()
            .ok_or_else(|| safety("raw record has no fixture".to_owned()))?
            .to_owned();
        let partitions_observed = value["partitions_observed"].as_u64().unwrap_or(0);
        let facts_line = format!(
            "disk={} partitions={}",
            value["disk_facts"], value["partition_facts"]
        );
        let mut info_captures: Vec<(String, Vec<Vec<u8>>)> = Vec::new();
        let mut privileged = Vec::new();
        if let Some(records) = value["records"].as_array() {
            for record in records {
                let tool = record["tool"].as_str().unwrap_or("");
                let subject = record["subject"].as_str().unwrap_or("").to_owned();
                let stdout = decode_hex(record["stdout_hex"].as_str().unwrap_or(""))?;
                match tool {
                    "udevadm-info" => {
                        if let Some(slot) = info_captures
                            .iter_mut()
                            .find(|(existing, _)| *existing == subject)
                        {
                            slot.1.push(stdout);
                        } else {
                            info_captures.push((subject, vec![stdout]));
                        }
                    }
                    "blkid-probe" | "wipefs-noact" => {
                        privileged.push((tool.to_owned(), subject, stdout));
                    }
                    _ => {}
                }
            }
        }
        let monitor = decode_hex(value["monitor_hex"].as_str().unwrap_or(""))?;
        sessions.push(Session {
            entry,
            fixture,
            partitions_observed,
            facts_line,
            info_captures,
            privileged,
            monitor,
        });
    }
    Ok(sessions)
}

fn decode_hex(text: &str) -> Result<Vec<u8>, TaskError> {
    if !text.len().is_multiple_of(2) {
        return Err(safety("raw record hex field has odd length".to_owned()));
    }
    let mut bytes = Vec::with_capacity(text.len() / 2);
    let mut characters = text.bytes();
    while let (Some(high), Some(low)) = (characters.next(), characters.next()) {
        let value = |character: u8| -> Result<u8, TaskError> {
            match character {
                b'0'..=b'9' => Ok(character - b'0'),
                b'a'..=b'f' => Ok(character - b'a' + 10),
                _ => Err(safety("raw record hex field is malformed".to_owned())),
            }
        };
        bytes.push((value(high)? << 4) | value(low)?);
    }
    Ok(bytes)
}

/// The frozen normalizer: keep every `E:` property except exactly the six
/// predeclared plumbing keys, preserving order; append the integer facts.
fn normalize_session(session: &Session) -> String {
    let mut normalized = String::new();
    for (subject, captures) in &session.info_captures {
        normalized.push_str("subject=");
        normalized.push_str(subject);
        normalized.push('\n');
        if let Some(first) = captures.first() {
            let text = canonicalize_info(first);
            for line in text.lines() {
                let Some(property) = line.strip_prefix("E: ") else {
                    continue;
                };
                let key = property.split('=').next().unwrap_or("");
                if NORMALIZER_DROPPED_KEYS.contains(&key) {
                    continue;
                }
                normalized.push_str(property);
                normalized.push('\n');
            }
        }
    }
    normalized.push_str(&session.facts_line);
    normalized
}

/// Which retained keys differ between two normalized projections.
fn projection_differences(healthy: &str, conflicting: &str) -> Vec<String> {
    let healthy_lines: std::collections::BTreeSet<&str> = healthy.lines().collect();
    let conflicting_lines: std::collections::BTreeSet<&str> = conflicting.lines().collect();
    let mut differences = Vec::new();
    for line in healthy_lines.symmetric_difference(&conflicting_lines) {
        let key = line.split('=').next().unwrap_or(line);
        let rendered = key.to_string();
        if !differences.contains(&rendered) {
            differences.push(rendered);
        }
    }
    differences
}

/// Render one session's privileged captures for the labelled comparison.
fn privileged_rendering(session: &Session) -> String {
    let mut rendered = String::new();
    for (tool, subject, stdout) in &session.privileged {
        rendered.push_str(tool);
        rendered.push(' ');
        rendered.push_str(subject);
        rendered.push('\n');
        rendered.push_str(&String::from_utf8_lossy(stdout));
        rendered.push('\n');
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    // Requirements: SAFE-006
    //   The frozen normalizer drops exactly the six predeclared plumbing keys
    //   and retains every other property in order.
    // Work-Package: WP-035
    // Evidence: normalizer_drops_exactly_the_declared_keys
    #[cfg(target_os = "linux")]
    #[test]
    fn normalizer_drops_exactly_the_declared_keys() {
        let info = b"P: /devices/virtual/block/loop0\n\
N: loop0\n\
E: DEVNAME=/dev/loop0\n\
E: USEC_INITIALIZED=123\n\
E: DISKSEQ=22\n\
E: ID_LOOP_BACKING_INODE=456\n\
E: ID_FS_TYPE=ext4\n\
E: ID_PART_ENTRY_DISK=7:0\n\
E: ID_LOOP_BACKING_FILENAME=/x\n\
E: ID_LOOP_BACKING_FILENAME_ENC=/x\n\
E: ID_LOOP_BACKING_DEVICE=8:1\n\
E: ID_PART_TABLE_TYPE=gpt\n"
            .to_vec();
        let session = Session {
            entry: "test".to_owned(),
            fixture: BASIC.to_owned(),
            partitions_observed: 0,
            facts_line: "disk={} partitions=[]".to_owned(),
            info_captures: vec![("disk".to_owned(), vec![info])],
            privileged: Vec::new(),
            monitor: Vec::new(),
        };
        let normalized = normalize_session(&session);
        assert!(normalized.contains("DEVNAME=/dev/loop0"));
        assert!(normalized.contains("ID_FS_TYPE=ext4"));
        assert!(normalized.contains("ID_PART_TABLE_TYPE=gpt"));
        for dropped in NORMALIZER_DROPPED_KEYS {
            assert!(
                !normalized.contains(dropped),
                "{dropped} survived the normalizer"
            );
        }
        // P: and N: lines are not E: properties and are not projection.
        assert!(!normalized.contains("/devices/virtual"));
    }

    // Requirements: SAFE-005, SAFE-006
    //   The one declared canonicalization sorts DEVLINKS tokens and preserves
    //   every token and every other line, so two captures differing only in
    //   udevadm's symlink-set rendering order compare stable, and a genuinely
    //   different token set still fails.
    // Work-Package: WP-035
    // Evidence: devlinks_canonicalization_sorts_tokens_and_preserves_content
    #[cfg(target_os = "linux")]
    #[test]
    fn devlinks_canonicalization_sorts_tokens_and_preserves_content() {
        let first = b"E: DEVLINKS=/dev/disk/by-partuuid/aa /dev/disk/by-partlabel/Data\n\
E: ID_FS_TYPE=ext4\n";
        let second = b"E: DEVLINKS=/dev/disk/by-partlabel/Data /dev/disk/by-partuuid/aa\n\
E: ID_FS_TYPE=ext4\n";
        assert_eq!(canonicalize_info(first), canonicalize_info(second));
        assert!(canonicalize_info(first).contains("/dev/disk/by-partuuid/aa"));
        assert!(canonicalize_info(first).contains("/dev/disk/by-partlabel/Data"));

        let dropped_token = b"E: DEVLINKS=/dev/disk/by-partlabel/Data\n";
        assert_ne!(canonicalize_info(first), canonicalize_info(dropped_token));

        let untouched = "E: ID_FS_UUID=abc";
        assert_eq!(canonicalize_devlinks_line(untouched), untouched);
    }

    // Requirements: SAFE-005, SAFE-006
    //   Stability compares the projection — the E: property sequence with the
    //   declared DEVLINKS canonicalization — so udevadm's S:-block set-order
    //   nondeterminism cannot fail it, while a genuinely changed property or
    //   reordered property sequence still does.
    // Work-Package: WP-035
    // Evidence: stability_projection_ignores_symlink_block_order_but_not_properties
    #[cfg(target_os = "linux")]
    #[test]
    fn stability_projection_ignores_symlink_block_order_but_not_properties() {
        let first = b"P: /devices/virtual/block/loop0/loop0p2\n\
S: disk/by-partuuid/aa\n\
S: disk/by-partlabel/Data\n\
E: DEVTYPE=partition\n\
E: DEVLINKS=/dev/disk/by-partuuid/aa /dev/disk/by-partlabel/Data\n";
        let second = b"P: /devices/virtual/block/loop0/loop0p2\n\
S: disk/by-partlabel/Data\n\
S: disk/by-partuuid/aa\n\
E: DEVTYPE=partition\n\
E: DEVLINKS=/dev/disk/by-partlabel/Data /dev/disk/by-partuuid/aa\n";
        assert_eq!(projection_lines(first), projection_lines(second));

        let changed = b"P: /devices/virtual/block/loop0/loop0p2\n\
S: disk/by-partuuid/aa\n\
E: DEVTYPE=disk\n\
E: DEVLINKS=/dev/disk/by-partuuid/aa /dev/disk/by-partlabel/Data\n";
        assert_ne!(projection_lines(first), projection_lines(changed));

        let reordered_properties =
            b"E: DEVLINKS=/dev/disk/by-partuuid/aa /dev/disk/by-partlabel/Data\n\
E: DEVTYPE=partition\n";
        assert_ne!(
            projection_lines(first),
            projection_lines(reordered_properties)
        );
    }

    // Requirements: SAFE-005, SAFE-006
    //   A successfully captured entry retaining no property is a no-entry
    //   state, reported so the decisive pair goes inconclusive rather than
    //   letting two empty projections compare equal and read as a negative.
    // Work-Package: WP-035
    // Evidence: coverage_gate_names_entries_that_retained_no_property
    #[cfg(target_os = "linux")]
    #[test]
    fn coverage_gate_names_entries_that_retained_no_property() {
        let populated = b"P: /devices/virtual/block/loop0\nN: loop0\nE: DEVTYPE=disk\n".to_vec();
        // Exit 0 with a header but no properties: the reachable no-entry
        // shape, since a nonzero udev exit is refused at capture time.
        let empty = b"P: /devices/virtual/block/loop0\nN: loop0\n".to_vec();

        let session = |entry: &str, capture: Vec<u8>| Session {
            entry: entry.to_owned(),
            fixture: BASIC.to_owned(),
            partitions_observed: 0,
            facts_line: "disk={} partitions=[]".to_owned(),
            info_captures: vec![("disk".to_owned(), vec![capture.clone(), capture])],
            privileged: Vec::new(),
            monitor: Vec::new(),
        };

        assert!(
            evaluate_coverage_gate(&[session("ok", populated)]).is_empty(),
            "a retained property is coverage"
        );
        assert_eq!(
            evaluate_coverage_gate(&[session("bare", empty.clone())]),
            ["bare/disk"],
            "a header-only entry is observed(absent)"
        );

        // The failure this gate exists to prevent: two empty projections are
        // string-equal, so without the gate the pair reads as a negative.
        let healthy = normalize_session(&session("h", empty.clone()));
        let conflicting = normalize_session(&session("c", empty));
        assert_eq!(
            healthy, conflicting,
            "two no-entry projections do compare equal"
        );
    }

    // Requirements: SAFE-005
    //   Hex round-trips exactly and malformed hex refuses.
    // Work-Package: WP-035
    // Evidence: raw_record_hex_round_trips_or_refuses
    #[cfg(target_os = "linux")]
    #[test]
    fn raw_record_hex_round_trips_or_refuses() {
        assert_eq!(hex(b"\x00\xffAB"), "00ff4142");
        assert_eq!(decode_hex("00ff4142").expect("valid hex"), b"\x00\xffAB");
        assert!(decode_hex("0").is_err());
        assert!(decode_hex("zz").is_err());
    }

    // Requirements: SAFE-005
    //   The compiled schedule is the preregistered one: negative controls
    //   first, alternating order-balanced trials, one closing healthy control.
    // Work-Package: WP-035
    // Evidence: compiled_schedule_matches_the_preregistered_shape
    #[cfg(target_os = "linux")]
    #[test]
    fn compiled_schedule_matches_the_preregistered_shape() {
        assert_eq!(SCHEDULE.len(), 10);
        assert!(SCHEDULE[..3].iter().all(|(_, _, f)| *f == BASIC));
        assert_eq!(SCHEDULE[1].1, Root::B, "the second control uses root b");
        let trials: Vec<&str> = SCHEDULE[3..9].iter().map(|(_, _, f)| *f).collect();
        assert_eq!(
            trials,
            [BASIC, CONFLICTING, BASIC, CONFLICTING, BASIC, CONFLICTING],
            "trials alternate and are order-balanced"
        );
        assert_eq!(SCHEDULE[9].2, BASIC, "the closing control is healthy");
        let basic_trials = trials.iter().filter(|f| **f == BASIC).count();
        let conflicting_trials = trials.iter().filter(|f| **f == CONFLICTING).count();
        assert!(basic_trials >= 3 && conflicting_trials >= 3);
    }

    // Requirements: SAFE-005
    //   Projection differences name retained keys from either side.
    // Work-Package: WP-035
    // Evidence: projection_differences_name_the_differing_keys
    #[cfg(target_os = "linux")]
    #[test]
    fn projection_differences_name_the_differing_keys() {
        let healthy = "ID_FS_TYPE=ext4\nID_PART_TABLE_TYPE=gpt\n";
        let conflicting = "ID_FS_TYPE=ext4\nID_PART_TABLE_TYPE=dos\nID_EXTRA=1\n";
        let differences = projection_differences(healthy, conflicting);
        assert!(differences.contains(&"ID_PART_TABLE_TYPE".to_owned()));
        assert!(differences.contains(&"ID_EXTRA".to_owned()));
        assert!(!differences.contains(&"ID_FS_TYPE".to_owned()));
    }
}
