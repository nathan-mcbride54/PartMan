//! Safe, unprivileged repository task runner.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use partman_fixtures::{catalogue, interlock, prober};

const PINNED_RUST_VERSION: &str = "1.96.0";
const WORKFLOW_DIRECTORY: &str = ".github/workflows";

/// Toolchain used only for fuzzing.
///
/// `cargo-fuzz` needs nightly for libFuzzer, which the pinned stable toolchain
/// cannot provide. It is pinned by exact date for the same reason
/// `rust-toolchain.toml` pins a version: an unpinned `nightly` changes under CI
/// without a commit. See `docs/quality/fuzzing.md`.
const FUZZ_TOOLCHAIN: &str = "nightly-2026-07-01";

/// Default smoke-run duration, in seconds, per fuzz target.
///
/// Section 11.4 requires short smoke runs to gate every pull request touching a
/// parser; long runs are scheduled separately.
const FUZZ_SMOKE_SECONDS: u32 = 60;

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    match parse(&args).and_then(|task| execute(&task)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Every task the runner can perform.
///
/// Parsing is separated from execution so that the whole command surface is
/// unit-testable without launching a subprocess.
#[derive(Debug, PartialEq, Eq)]
enum Task {
    Ci,
    CrossLanguage,
    Fixtures,
    Fuzz { seconds: u32 },
    Fmt,
    FmtCheck,
    Help,
    Probe,
    SupplyChain,
    Test { tier: u8, profile: Option<String> },
    VerifyActions,
    VerifyToolchain,
}

fn parse(args: &[OsString]) -> Result<Task, TaskError> {
    let Some(command) = args.first() else {
        return Ok(Task::Help);
    };
    let Some(command) = command.to_str() else {
        return Err(TaskError::Usage(
            "task names are ASCII; run `cargo xtask help`".to_owned(),
        ));
    };
    let rest = &args[1..];

    match command {
        "ci" => nullary(Task::Ci, command, rest),
        "cross-language" => nullary(Task::CrossLanguage, command, rest),
        "fixtures" => nullary(Task::Fixtures, command, rest),
        "fuzz" => parse_fuzz(rest),
        "fmt" => nullary(Task::Fmt, command, rest),
        "fmt-check" => nullary(Task::FmtCheck, command, rest),
        "help" | "--help" | "-h" => nullary(Task::Help, command, rest),
        "probe" => nullary(Task::Probe, command, rest),
        "supply-chain" => nullary(Task::SupplyChain, command, rest),
        "verify-actions" => nullary(Task::VerifyActions, command, rest),
        "verify-toolchain" => nullary(Task::VerifyToolchain, command, rest),
        "test" => parse_test(rest),
        other => Err(TaskError::Usage(format!(
            "unknown task {other:?}; run `cargo xtask help`"
        ))),
    }
}

fn execute(task: &Task) -> Result<(), TaskError> {
    match *task {
        Task::Ci => {
            verify_toolchain()?;
            verify_action_pins(&repository_root())?;
            cargo(&["fmt", "--all", "--", "--check"])?;
            cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--",
                "-D",
                "warnings",
            ])?;
            run_tier(1, None)
        }
        Task::CrossLanguage => cross_language(),
        Task::Fuzz { seconds } => fuzz(seconds),
        Task::Fmt => cargo(&["fmt", "--all"]),
        Task::FmtCheck => cargo(&["fmt", "--all", "--", "--check"]),
        Task::Help => {
            print_help();
            Ok(())
        }
        Task::SupplyChain => {
            cargo(&["deny", "check", "advisories", "bans", "licenses", "sources"])?;
            cargo(&["audit", "--deny", "warnings"])
        }
        Task::Test { tier, ref profile } => run_tier(tier, profile.as_deref()),
        Task::Fixtures => generate_fixtures(),
        Task::Probe => probe_fixtures(),
        Task::VerifyActions => verify_action_pins(&repository_root()),
        Task::VerifyToolchain => verify_toolchain(),
    }
}

fn nullary(task: Task, command: &str, rest: &[OsString]) -> Result<Task, TaskError> {
    if rest.is_empty() {
        Ok(task)
    } else {
        Err(TaskError::Usage(format!(
            "`cargo xtask {command}` takes no arguments; run `cargo xtask help`"
        )))
    }
}

/// Parse `fuzz`, with an optional `--seconds <n>`.
fn parse_fuzz(args: &[OsString]) -> Result<Task, TaskError> {
    match args {
        [] => Ok(Task::Fuzz {
            seconds: FUZZ_SMOKE_SECONDS,
        }),
        [flag, value] if flag == OsStr::new("--seconds") => value
            .to_str()
            .and_then(|text| text.parse::<u32>().ok())
            .filter(|seconds| *seconds > 0)
            .map(|seconds| Task::Fuzz { seconds })
            .ok_or_else(|| TaskError::Usage("--seconds takes a positive integer".to_owned())),
        _ => Err(TaskError::Usage(
            "expected `cargo xtask fuzz [--seconds <n>]`".to_owned(),
        )),
    }
}

/// Parse `test --tier <n> [--profile <word>]`.
///
/// The profile is an argument rather than an environment variable on purpose.
/// SAFE-007 rules out proving destructive intent with a single variable, and an
/// argument cannot be inherited by accident from a parent shell.
fn parse_test(args: &[OsString]) -> Result<Task, TaskError> {
    let (tier_args, profile) = match args {
        [tier_flag, tier_value] => ([tier_flag, tier_value], None),
        [tier_flag, tier_value, profile_flag, profile_value]
            if profile_flag == OsStr::new("--profile") =>
        {
            let profile = profile_value
                .to_str()
                .ok_or_else(|| TaskError::Usage("--profile takes an ASCII word".to_owned()))?;
            ([tier_flag, tier_value], Some(profile.to_owned()))
        }
        _ => {
            return Err(TaskError::Usage(
                "expected `cargo xtask test --tier <1|2|3> [--profile <word>]`".to_owned(),
            ));
        }
    };

    let tier = parse_tier_args(tier_args)?;
    Ok(Task::Test { tier, profile })
}

fn parse_tier_args(args: [&OsString; 2]) -> Result<u8, TaskError> {
    if args[0] != OsStr::new("--tier") {
        return Err(TaskError::Usage(
            "expected `cargo xtask test --tier <1|2|3> [--profile <word>]`".to_owned(),
        ));
    }

    args[1]
        .to_str()
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|tier| (1..=3).contains(tier))
        .ok_or_else(|| TaskError::Usage("test tier must be 1, 2, or 3".to_owned()))
}

/// Where generated fixtures live. Ignored by git; never committed (Section 16).
fn fixture_root() -> PathBuf {
    repository_root().join("tests").join("generated")
}

fn generate_fixtures() -> Result<(), TaskError> {
    let root = fixture_root();
    let manifest = catalogue::generate(&root).map_err(|error| {
        TaskError::Usage(format!(
            "could not generate fixtures in {}: {error}",
            root.display()
        ))
    })?;

    println!(
        "generated {} fixtures in {}",
        manifest.names().count(),
        root.display()
    );
    for name in manifest.names() {
        println!("  {name}");
    }
    println!();
    println!("SAFE-007 disposable-test token:");
    println!("  {}", manifest.token());
    println!();
    println!(
        "A destructive tier additionally needs `--profile {}` and {}=<the token above>.",
        interlock::DESTRUCTIVE_PROFILE,
        interlock::TOKEN_VARIABLE
    );
    Ok(())
}

/// Re-run the real probers and compare against the recorded expectations.
///
/// This is the check that was manual until now: someone ran `blkid`, read the
/// output, and wrote a table into a document. A fixture a real prober does not
/// recognize proves nothing, and two of the signature writers here were
/// undetectable until their checksums were reproduced — neither the format
/// documentation nor this crate's own tests could have said so.
///
/// It needs Linux, so it is not part of `cargo xtask ci`; CI runs it as its own
/// job. Both tools are read-only and are given regular files, never a device.
fn probe_fixtures() -> Result<(), TaskError> {
    if !cfg!(target_os = "linux") {
        return Err(TaskError::Usage(
            "`cargo xtask probe` needs `blkid` and `wipefs`, which are Linux tools. The recorded \
             expectations are in `crates/fixtures/src/prober.rs`, and CI runs this on \
             ubuntu-24.04."
                .to_owned(),
        ));
    }

    let root = fixture_root();
    catalogue::generate(&root).map_err(|error| {
        TaskError::Usage(format!(
            "could not generate fixtures in {}: {error}",
            root.display()
        ))
    })?;

    // Record what produced these answers. A disagreement is far cheaper to
    // diagnose when the version that disagreed is in the same output, and one
    // expectation is genuinely version-dependent.
    let banner = tool_version("blkid")?;
    println!("{banner}");
    println!("{}", tool_version("wipefs")?);
    let version = prober::parse_util_linux_version(&banner).ok_or_else(|| {
        TaskError::Usage(format!(
            "could not read a util-linux version from {banner:?}. One expectation depends on it, \
             so guessing would silently relax the check."
        ))
    })?;
    println!(
        "reading expectations for util-linux {}.{}",
        version.0, version.1
    );
    println!();

    let mut failures = Vec::new();
    for expectation in prober::expectations() {
        let path = root.join(expectation.fixture);
        // `blkid` exits 2 when it detects nothing, which is the correct and
        // expected answer for the blank fixture.
        let raw_udev = probe_output("blkid", &["-p", "-o", "udev"], &path, &[0, 2])?;
        let raw_wipefs = probe_output("wipefs", &["-n", "--output", "OFFSET,TYPE"], &path, &[0])?;
        let observed = prober::Observation {
            udev: prober::parse_udev(&raw_udev),
            signatures: prober::parse_wipefs(&raw_wipefs),
        };

        let disagreements = prober::compare(&expectation, &observed, version);
        if disagreements.is_empty() {
            println!("  ok    {}", expectation.fixture);
        } else {
            println!("  FAIL  {}", expectation.fixture);
            for reason in &disagreements {
                println!("          {reason}");
            }
            println!("        recorded because: {}", expectation.note);
            // What the tools actually said, verbatim. A disagreement is a
            // measurement, and the next decision — fixture regressed, or prober
            // changed — cannot be made from a diff of parsed values alone.
            println!("        what the probers actually said:");
            for (tool, text) in [
                ("blkid -p -o udev", &raw_udev),
                ("wipefs -n", &raw_wipefs),
                ("blkid -p", &probe_output("blkid", &["-p"], &path, &[0, 2])?),
            ] {
                println!("          $ {tool}");
                if text.trim().is_empty() {
                    println!("            (no output)");
                }
                for line in text.lines() {
                    println!("            {line}");
                }
            }
            failures.push(expectation.fixture);
        }
    }

    println!();
    if failures.is_empty() {
        println!(
            "{} fixtures still report what `crates/fixtures/src/prober.rs` records.",
            prober::expectations().len()
        );
        return Ok(());
    }
    Err(TaskError::Usage(format!(
        "{} fixture(s) no longer match the recorded prober output: {}. Either a fixture \
         regressed, or the prober changed and the record needs updating with the new \
         measurement — decide which, and say so in the commit rather than editing the table to \
         match.",
        failures.len(),
        failures.join(", ")
    )))
}

/// Run one prober and return its stdout.
///
/// The argument list is structured rather than a shell string, so nothing in a
/// path is ever interpreted.
fn probe_output(
    tool: &str,
    flags: &[&str],
    path: &Path,
    accepted: &[i32],
) -> Result<String, TaskError> {
    let output = Command::new(tool)
        .args(flags)
        .arg(path)
        .output()
        .or_else(|_| {
            // `wipefs` and `blkid` ship in `/usr/sbin`, which is on `PATH` for
            // root and often not for an ordinary user. Falling back by absolute
            // path is what keeps this a Tier-1, unprivileged check rather than
            // one that quietly requires a root shell.
            Command::new(format!("/usr/sbin/{tool}"))
                .args(flags)
                .arg(path)
                .output()
        })
        .map_err(|error| {
            TaskError::Usage(format!(
                "could not run `{tool}`, and not `/usr/sbin/{tool}` either: {error}. Both ship \
                 with util-linux."
            ))
        })?;

    let code = output.status.code().unwrap_or(-1);
    if !accepted.contains(&code) {
        return Err(TaskError::Usage(format!(
            "`{tool}` exited {code} for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The version string a tool reports, for the run's record.
fn tool_version(tool: &str) -> Result<String, TaskError> {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .or_else(|_| {
            Command::new(format!("/usr/sbin/{tool}"))
                .arg("--version")
                .output()
        })
        .map_err(|error| TaskError::Usage(format!("could not run `{tool} --version`: {error}")))?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn run_tier(tier: u8, profile: Option<&str>) -> Result<(), TaskError> {
    match tier {
        1 => cargo(&["test", "--workspace", "--all-targets", "--locked"]),
        2 | 3 => destructive_tier(tier, profile),
        _ => Err(TaskError::Usage("test tier must be 1, 2, or 3".to_owned())),
    }
}

/// Evaluate SAFE-007 for a destructive tier, then report honestly.
///
/// The interlock runs first so that a misconfigured request gets the specific
/// refusal it earned. It passing does **not** mean a suite runs: none exists
/// yet, and reporting success for an empty run would be the fake success path
/// Section 12 and Section 16 forbid.
fn destructive_tier(tier: u8, profile: Option<&str>) -> Result<(), TaskError> {
    let root = fixture_root();

    // The target list comes from the compiled catalogue, not from the manifest
    // on disk. Reading it from disk would let whoever wrote that file choose
    // what a destructive suite addresses.
    let targets = catalogue::expected()
        .names()
        .map(|name| root.join(name))
        .collect();
    let request = interlock::Request {
        profile: profile.map(ToOwned::to_owned),
        token: env::var(interlock::TOKEN_VARIABLE).ok(),
        targets,
    };

    let authorization = interlock::authorize(&root, &request).map_err(|refusal| {
        TaskError::Safety(format!("Tier {tier} refused: {refusal}. Nothing was run"))
    })?;

    Err(TaskError::Safety(format!(
        "the SAFE-007 interlock authorized {} disposable target(s), but no Tier-{tier} suite is \
         registered yet. WP-020 increment 2 supplies the loopback and virtual-machine harness; \
         until then there is nothing to run and reporting success would be a lie",
        authorization.targets().len()
    )))
}

fn verify_toolchain() -> Result<(), TaskError> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|source| TaskError::Launch {
            program: "rustc".to_owned(),
            source,
        })?;

    if !output.status.success() {
        return Err(TaskError::CommandFailed {
            program: "rustc".to_owned(),
            code: output.status.code(),
        });
    }

    let version = String::from_utf8(output.stdout)
        .map_err(|_| TaskError::Usage("rustc returned non-UTF-8 version output".to_owned()))?;
    let expected = format!("rustc {PINNED_RUST_VERSION} ");
    if !version.starts_with(&expected) {
        return Err(TaskError::Usage(format!(
            "expected Rust {PINNED_RUST_VERSION}, got {}; rust-toolchain.toml must control CI",
            version.trim()
        )));
    }

    Ok(())
}

/// Prove the MODEL-005 cross-language parity requirement.
///
/// The Rust half runs inside `cargo xtask ci`; this command adds the TypeScript
/// half, which needs a Node toolchain. It is deliberately not folded into `ci`,
/// so that a contributor working only on Rust is not required to install Node.
/// CI runs it as its own required job, so the proof is never merely skipped.
fn cross_language() -> Result<(), TaskError> {
    let package = repository_root().join("packages/canonical");
    if !package.join("package.json").is_file() {
        return Err(TaskError::Policy(format!(
            "{} is missing; the MODEL-005 parity proof cannot run",
            package.join("package.json").display()
        )));
    }
    npm(&package, &["ci"])?;
    // This is the only gate with a Node toolchain, so SEC-010's advisory
    // requirement for npm dependencies is enforced here rather than in
    // `supply-chain`, which runs without Node.
    npm(&package, &["audit", "--audit-level=moderate"])?;
    npm(&package, &["run", "typecheck"])?;
    npm(&package, &["test"])
}

fn npm(directory: &Path, args: &[&str]) -> Result<(), TaskError> {
    // npm ships as a shell script plus a .cmd shim on Windows, so the command
    // name differs by platform. Nothing here is user-controlled.
    let program = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(program)
        .args(args)
        .current_dir(directory)
        .status()
        .map_err(|source| TaskError::Launch {
            program: program.to_owned(),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(TaskError::CommandFailed {
            program: format!("{program} {}", args.join(" ")),
            code: status.code(),
        })
    }
}

/// Every fuzz target in `fuzz/fuzz_targets`, in the order they are run.
const FUZZ_TARGETS: [&str; 2] = ["decode_is_canonical", "roundtrip_value"];

/// Run a bounded smoke fuzz over every target (Section 11.4).
///
/// This needs the pinned nightly toolchain, so it is not part of
/// `cargo xtask ci` and CI runs it as its own job. A crash leaves a reproducer
/// in `fuzz/artifacts/`, which is git-ignored: Section 11.3 keeps binary
/// fixtures out of the repository, so a reproducer is attached to the report
/// rather than committed.
fn fuzz(seconds: u32) -> Result<(), TaskError> {
    let directory = repository_root().join("fuzz");
    if !directory.join("Cargo.toml").is_file() {
        return Err(TaskError::Policy(format!(
            "{} is missing; the Section 11.4 fuzz targets cannot run",
            directory.join("Cargo.toml").display()
        )));
    }

    for target in FUZZ_TARGETS {
        println!("fuzz: {target} for {seconds}s");
        let toolchain = format!("+{FUZZ_TOOLCHAIN}");
        let max_time = format!("-max_total_time={seconds}");
        let status = Command::new("cargo")
            .args([
                &toolchain,
                "fuzz",
                "run",
                target,
                "--",
                &max_time,
                // Bound a single input so a pathological case is a reported
                // timeout rather than a hung job.
                "-timeout=25",
            ])
            .current_dir(repository_root())
            .status()
            .map_err(|source| TaskError::Launch {
                program: "cargo".to_owned(),
                source,
            })?;

        if !status.success() {
            return Err(TaskError::CommandFailed {
                program: format!("cargo fuzz run {target}"),
                code: status.code(),
            });
        }
    }

    Ok(())
}

/// The repository root, derived from this crate's compile-time location.
///
/// The runner never depends on the caller's working directory.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask is located at <repository>/tools/xtask")
        .to_path_buf()
}

/// Enforce the SEC-010 rule that every GitHub Action is pinned by digest.
///
/// Mutable tags such as `@v6` are rejected: they let an upstream account move a
/// release tag onto new code that would then run with repository credentials.
/// The check fails closed when no workflow can be read, so a moved, renamed, or
/// unreadable workflow directory can never pass vacuously.
fn verify_action_pins(root: &Path) -> Result<(), TaskError> {
    let directory = root.join(WORKFLOW_DIRECTORY);
    let entries = fs::read_dir(&directory).map_err(|source| TaskError::Io {
        path: directory.clone(),
        source,
    })?;

    let mut workflows = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        let is_workflow = path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
            });
        if is_workflow {
            workflows.push(path);
        }
    }
    workflows.sort();

    if workflows.is_empty() {
        return Err(TaskError::Policy(format!(
            "no workflow files found in {}; SEC-010 action pinning cannot be verified",
            directory.display()
        )));
    }

    let mut violations = Vec::new();
    let mut pinned = 0_usize;
    for workflow in &workflows {
        let text = fs::read_to_string(workflow).map_err(|source| TaskError::Io {
            path: workflow.clone(),
            source,
        })?;
        let name = workflow
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("<workflow>");
        for (line, reference) in action_references(&text) {
            if is_pinned(&reference) {
                pinned += 1;
            } else {
                violations.push(format!("{name}:{line}: {reference}"));
            }
        }
    }

    if violations.is_empty() {
        println!("verify-actions: {pinned} action reference(s) pinned by digest");
        Ok(())
    } else {
        Err(TaskError::Policy(format!(
            "SEC-010 requires every GitHub Action to be pinned to a full commit SHA, with the \
             release tag kept in a trailing comment. Unpinned references:\n  {}",
            violations.join("\n  ")
        )))
    }
}

/// Extract `(line number, reference)` for every `uses:` entry in a workflow.
fn action_references(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            action_reference(line).map(|reference| (index + 1, reference.to_owned()))
        })
        .collect()
}

fn action_reference(line: &str) -> Option<&str> {
    let mut trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    if let Some(item) = trimmed.strip_prefix("- ") {
        trimmed = item.trim_start();
    }
    let value = trimmed.strip_prefix("uses:")?.trim();
    let value = value.split_once('#').map_or(value, |(before, _)| before);
    let value = value.trim().trim_matches(['"', '\'']);
    (!value.is_empty()).then_some(value)
}

fn is_pinned(reference: &str) -> bool {
    // Actions committed to this repository carry no independent supply chain.
    if reference.starts_with("./") {
        return true;
    }
    if let Some(image) = reference.strip_prefix("docker://") {
        return image
            .rsplit_once("@sha256:")
            .is_some_and(|(_, digest)| is_lowercase_hex(digest, 64));
    }
    reference
        .rsplit_once('@')
        .is_some_and(|(_, git_ref)| is_lowercase_hex(git_ref, 40))
}

fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn cargo(args: &[&str]) -> Result<(), TaskError> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .map_err(|source| TaskError::Launch {
            program: "cargo".to_owned(),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(TaskError::CommandFailed {
            program: "cargo".to_owned(),
            code: status.code(),
        })
    }
}

fn print_help() {
    println!(
        "\
PartMan repository tasks

  cargo xtask ci                 Run the complete unprivileged Tier-1 gate
  cargo xtask cross-language     Prove Rust and TypeScript hash identically
  cargo xtask fuzz [--seconds n] Smoke-fuzz the parsers (needs pinned nightly)
  cargo xtask fmt                Format the Rust workspace
  cargo xtask fmt-check          Verify Rust formatting
  cargo xtask fixtures           Generate the synthetic disk fixtures (SAFE-001)
  cargo xtask probe              Re-check every fixture against libblkid (Linux)
  cargo xtask test --tier 1      Run safe, unprivileged tests
  cargo xtask test --tier 2|3 --profile destructive
                                 Evaluate the SAFE-007 interlock. Also needs
                                 PARTMAN_DISPOSABLE_TOKEN from `xtask fixtures`.
                                 No destructive suite exists yet, so this still
                                 refuses rather than reporting an empty pass.
  cargo xtask supply-chain       Run cargo-deny and cargo-audit
  cargo xtask verify-actions     Verify every GitHub Action is pinned by digest
  cargo xtask verify-toolchain   Verify the pinned Rust compiler
"
    );
}

#[derive(Debug)]
enum TaskError {
    CommandFailed {
        program: String,
        code: Option<i32>,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Launch {
        program: String,
        source: std::io::Error,
    },
    Policy(String),
    Safety(String),
    Usage(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandFailed { program, code } => {
                write!(formatter, "{program} failed with exit code {code:?}")
            }
            Self::Io { path, source } => {
                write!(formatter, "could not read {}: {source}", path.display())
            }
            Self::Launch { program, source } => {
                write!(formatter, "could not launch {program}: {source}")
            }
            Self::Policy(message) | Self::Safety(message) | Self::Usage(message) => {
                formatter.write_str(message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Task, TaskError, action_reference, action_references, is_pinned, parse, parse_test,
        repository_root, run_tier, verify_action_pins,
    };
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn tier_parser_accepts_explicit_tier_one() {
        assert_eq!(
            parse_test(&args(&["--tier", "1"])).expect("Tier 1 must parse"),
            Task::Test {
                tier: 1,
                profile: None
            }
        );
    }

    #[test]
    fn tier_parser_rejects_missing_proof_by_omission() {
        let error = parse_test(&[]).expect_err("A tier must always be explicit");
        assert!(matches!(error, TaskError::Usage(_)));
    }

    #[test]
    fn tier_parser_rejects_out_of_range_and_malformed_tiers() {
        for value in ["0", "4", "255", "one", "1.0", "-1", ""] {
            let error = parse_test(&args(&["--tier", value]))
                .expect_err("only tiers 1, 2, and 3 are addressable");
            assert!(matches!(error, TaskError::Usage(_)), "tier {value:?}");
        }
    }

    #[test]
    fn unavailable_destructive_tiers_fail_closed() {
        for tier in [2, 3] {
            let error = run_tier(tier, None).expect_err("a destructive tier must never run here");
            assert!(matches!(error, TaskError::Safety(_)));
        }
    }

    #[test]
    fn parser_maps_every_documented_task() {
        assert_eq!(parse(&args(&["ci"])).expect("ci"), Task::Ci);
        assert_eq!(
            parse(&args(&["cross-language"])).expect("cross-language"),
            Task::CrossLanguage
        );
        assert_eq!(
            parse(&args(&["fuzz"])).expect("fuzz"),
            Task::Fuzz {
                seconds: super::FUZZ_SMOKE_SECONDS
            }
        );
        assert_eq!(
            parse(&args(&["fuzz", "--seconds", "5"])).expect("fuzz --seconds"),
            Task::Fuzz { seconds: 5 }
        );
        assert_eq!(parse(&args(&["fmt"])).expect("fmt"), Task::Fmt);
        assert_eq!(
            parse(&args(&["fmt-check"])).expect("fmt-check"),
            Task::FmtCheck
        );
        assert_eq!(parse(&args(&["probe"])).expect("probe"), Task::Probe);
        assert_eq!(
            parse(&args(&["supply-chain"])).expect("supply-chain"),
            Task::SupplyChain
        );
        assert_eq!(
            parse(&args(&["verify-actions"])).expect("verify-actions"),
            Task::VerifyActions
        );
        assert_eq!(
            parse(&args(&["verify-toolchain"])).expect("verify-toolchain"),
            Task::VerifyToolchain
        );
        assert_eq!(
            parse(&args(&["test", "--tier", "3"])).expect("tier 3 parses but must not execute"),
            Task::Test {
                tier: 3,
                profile: None
            }
        );
        assert_eq!(parse(&[]).expect("bare invocation"), Task::Help);
    }

    #[test]
    fn the_fixture_generator_is_a_documented_task() {
        assert_eq!(
            parse(&args(&["fixtures"])).expect("fixtures"),
            Task::Fixtures
        );
    }

    #[test]
    fn the_prober_check_refuses_where_its_tools_do_not_exist() {
        // `blkid` and `wipefs` are Linux tools, and the refusal has to say so
        // and say where the expectations live. A task that failed with "could
        // not run blkid" on Windows would read as a broken repository rather
        // than as a check that belongs elsewhere.
        if cfg!(target_os = "linux") {
            return;
        }
        let error = super::probe_fixtures().expect_err("must refuse off Linux");
        let message = error.to_string();
        assert!(message.contains("Linux"), "{message}");
        assert!(message.contains("prober.rs"), "{message}");
    }

    #[test]
    fn a_destructive_profile_is_an_argument_not_an_environment_variable() {
        // SAFE-007 says one environment variable is not proof. Parsing the
        // profile only from the command line is half of why: it cannot be
        // inherited from a parent shell by accident.
        assert_eq!(
            parse(&args(&["test", "--tier", "2", "--profile", "destructive"]))
                .expect("tier 2 with a profile must parse"),
            Task::Test {
                tier: 2,
                profile: Some("destructive".to_owned())
            }
        );
    }

    #[test]
    fn a_profile_without_a_tier_is_rejected() {
        for invocation in [
            vec!["test", "--profile", "destructive"],
            vec!["test", "--tier", "2", "--profile"],
            vec!["test", "--tier", "2", "destructive"],
            vec!["test", "--tier", "2", "--profile", "destructive", "extra"],
        ] {
            let error = parse(&args(&invocation)).expect_err("must not parse");
            assert!(matches!(error, TaskError::Usage(_)), "{invocation:?}");
        }
    }

    #[test]
    fn a_destructive_tier_refuses_even_with_the_profile_word() {
        // The profile alone is one factor of three, and no suite exists to run
        // in any case. Either way this must be a refusal, never a pass.
        for tier in [2, 3] {
            let error = run_tier(tier, Some("destructive"))
                .expect_err("a destructive tier must never report success today");
            assert!(matches!(error, TaskError::Safety(_)), "tier {tier}");
        }
    }

    #[test]
    fn parser_rejects_unknown_tasks_and_stray_arguments() {
        for invocation in [
            vec!["destroy"],
            vec!["ci", "--tier", "2"],
            vec!["verify-actions", "--fix"],
            vec!["test"],
            vec!["test", "1"],
            vec!["test", "--tier"],
            vec!["test", "--tier", "1", "--tier", "2"],
            // A zero-second smoke run would pass without fuzzing anything.
            vec!["fuzz", "--seconds", "0"],
            vec!["fuzz", "--seconds", "abc"],
            vec!["fuzz", "5"],
            vec!["fuzz", "--seconds"],
        ] {
            let error = parse(&args(&invocation))
                .expect_err(&format!("{invocation:?} must not be accepted"));
            assert!(matches!(error, TaskError::Usage(_)), "{invocation:?}");
        }
    }

    #[test]
    fn digest_pins_are_accepted_and_mutable_references_are_not() {
        assert!(is_pinned(
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
        ));
        assert!(is_pinned("./.github/actions/local"));
        assert!(is_pinned(&format!(
            "docker://alpine@sha256:{}",
            "a".repeat(64)
        )));

        for mutable in [
            "actions/checkout@v6",
            "actions/checkout@v6.0.2",
            "actions/checkout@main",
            "actions/checkout",
            "actions/checkout@de0fac2e",
            // Uppercase hex is never produced by Git and would evade a naive comparison.
            "actions/checkout@DE0FAC2E4500DABE0009E67214FF5F5447CE83DD",
            // 39 and 41 characters bracket the accepted length.
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83d",
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83ddd",
            "docker://alpine@sha256:short",
            "docker://alpine:3.20",
        ] {
            assert!(!is_pinned(mutable), "{mutable} must be rejected");
        }
    }

    #[test]
    fn workflow_scanner_reads_references_and_ignores_prose() {
        let workflow = "\
jobs:
  build:
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      - name: Quoted form
        uses: \"actions/setup-node@0000000000000000000000000000000000000000\"
      # uses: actions/stale@v9
      - name: Prose must not register
        run: echo \"uses: actions/checkout@v6\"
";
        let found = action_references(workflow);
        assert_eq!(found.len(), 2, "found {found:?}");
        assert_eq!(
            found[0],
            (
                4,
                "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd".to_owned()
            )
        );
        assert_eq!(
            found[1],
            (
                6,
                "actions/setup-node@0000000000000000000000000000000000000000".to_owned()
            )
        );
        assert_eq!(action_reference("        uses:"), None);
    }

    #[test]
    fn repository_workflows_are_pinned_by_digest() {
        verify_action_pins(&repository_root())
            .expect("every committed workflow must satisfy SEC-010 action pinning");
    }

    #[test]
    fn action_pin_check_fails_closed_on_a_missing_workflow_directory() {
        let error = verify_action_pins(&repository_root().join("tools"))
            .expect_err("a missing workflow directory must never pass silently");
        assert!(matches!(error, TaskError::Io { .. }));
    }

    #[test]
    fn action_pin_check_fails_closed_on_an_empty_workflow_directory() {
        // A renamed or emptied workflow directory must not report success.
        let root = std::env::temp_dir().join(format!("partman-xtask-{}", std::process::id()));
        let workflows = root.join(super::WORKFLOW_DIRECTORY);
        std::fs::create_dir_all(&workflows).expect("temporary workflow directory");

        let result = verify_action_pins(&root);
        std::fs::remove_dir_all(&root).expect("temporary directory cleanup");

        let error = result.expect_err("an empty workflow directory must never pass silently");
        assert!(matches!(error, TaskError::Policy(_)));
    }
}
