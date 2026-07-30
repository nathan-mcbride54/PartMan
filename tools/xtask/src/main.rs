//! Safe, unprivileged repository task runner.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use partman_fixtures::{catalogue, interlock, prober};

const PINNED_RUST_VERSION: &str = "1.96.0";
const WORKFLOW_DIRECTORY: &str = ".github/workflows";
/// Composite actions committed to this repository. Optional, unlike the
/// workflow directory — but scanned whenever present, because a local action's
/// own `uses:` references are remote supply chain like any other.
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
    Tokens,
    Traceability { write: bool },
    VerifyChangeOwnership { base: String },
    VerifyOwnership,
    VerifyActions,
    VerifyLicenses,
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
        "tokens" => nullary(Task::Tokens, command, rest),
        "traceability" => parse_traceability(command, rest),
        "verify-ownership" => nullary(Task::VerifyOwnership, command, rest),
        "verify-change-ownership" => parse_change_ownership(command, rest),
        "verify-actions" => nullary(Task::VerifyActions, command, rest),
        "verify-licenses" => nullary(Task::VerifyLicenses, command, rest),
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
            verify_manifest_licenses(&repository_root())?;
            verify_workspace_lints(&repository_root())?;
            verify_path_ownership(&repository_root())?;
            audit_tokens()?;
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
            run_tier(1, None)?;
            // Last, because it enumerates the test binaries the tier-1 run has
            // just built. Running it first would build them twice.
            verify_traceability(&repository_root(), false)
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
            // Before anything that resolves. `cargo deny` builds its graph by
            // resolving the manifest, and the follow-up audit showed it
            // silently repairing a stale `fuzz/Cargo.lock` while doing so —
            // the policy tool committing the fail-open shape it exists to
            // prevent. The preflight is shared with `fuzz()` so neither entry
            // point can be the one that repairs the lock it audits.
            verify_fuzz_lock()?;
            cargo(&["deny", "check", "advisories", "bans", "licenses", "sources"])?;
            cargo(&["audit", "--deny", "warnings"])?;
            // The fuzz crate is excluded from the workspace, so the two
            // commands above never see its dependency graph. Until 2026-07-29
            // nothing did: its lockfile was gitignored and its dependencies
            // were advisory-, licence- and source-checked by nobody, on the
            // job that executes hostile-byte parser tests. Same policy file,
            // second graph.
            cargo(&[
                "deny",
                "--manifest-path",
                "fuzz/Cargo.toml",
                "check",
                "advisories",
                "bans",
                "licenses",
                "sources",
            ])?;
            cargo(&["audit", "--deny", "warnings", "--file", "fuzz/Cargo.lock"])
        }
        Task::Test { tier, ref profile } => run_tier(tier, profile.as_deref()),
        Task::Fixtures => generate_fixtures(),
        Task::Probe => probe_fixtures(),
        Task::Tokens => audit_tokens(),
        Task::Traceability { write } => verify_traceability(&repository_root(), write),
        Task::VerifyActions => verify_action_pins(&repository_root()),
        Task::VerifyLicenses => verify_manifest_licenses(&repository_root()),
        Task::VerifyOwnership => verify_path_ownership(&repository_root()),
        Task::VerifyChangeOwnership { ref base } => {
            verify_change_ownership(&repository_root(), base)
        }
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

/// Parse `verify-change-ownership --base <revision>`.
///
/// The base is required rather than defaulted to `origin/main`. A default would
/// silently verify against whatever a stale local ref happened to point at, and
/// a check that quietly measures the wrong thing is worse than one that asks.
/// `traceability`, optionally with `--write`.
///
/// Writing is a separate word rather than the default, so the gate cannot be
/// satisfied by the act of running it. `cargo xtask ci` never passes it.
fn parse_traceability(command: &str, rest: &[OsString]) -> Result<Task, TaskError> {
    match rest {
        [] => Ok(Task::Traceability { write: false }),
        [flag] if flag == "--write" => Ok(Task::Traceability { write: true }),
        _ => Err(TaskError::Usage(format!(
            "usage: cargo xtask {command} [--write]"
        ))),
    }
}

fn parse_change_ownership(command: &str, rest: &[OsString]) -> Result<Task, TaskError> {
    match rest {
        [flag, value] if flag == "--base" => {
            let base = value.to_string_lossy().trim().to_owned();
            if base.is_empty() {
                return Err(TaskError::Usage(format!(
                    "`cargo xtask {command} --base <revision>` needs a revision"
                )));
            }
            Ok(Task::VerifyChangeOwnership { base })
        }
        _ => Err(TaskError::Usage(format!(
            "`cargo xtask {command}` requires `--base <revision>`, for example \
             `--base origin/main`"
        ))),
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

/// Audit `schemas/design-tokens.json` against UI-001, UI-007 and UI-008.
///
/// Unprivileged and Tier 1: it reads one JSON file and computes numbers from
/// it. The caveats are printed on success as well as failure, because a green
/// accessibility check is exactly the output someone would over-read.
fn audit_tokens() -> Result<(), TaskError> {
    let report = partman_tokens::audit_repository_tokens()
        .map_err(|error| TaskError::Policy(format!("could not read the design tokens: {error}")))?;

    println!("{}", report.summary());
    if report.is_clean() {
        println!("\nThis check does not establish:");
        for caveat in partman_tokens::Report::caveats() {
            println!("  - {caveat}");
        }
        return Ok(());
    }
    Err(TaskError::Policy(format!(
        "the design tokens violate {} accessibility rule(s); see the findings above",
        report.findings.len()
    )))
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
        let unreadable = |error: prober::Unreadable| {
            TaskError::Usage(format!(
                "{}: {error}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ))
        };
        let observed = prober::Observation {
            udev: prober::parse_udev(&raw_udev).map_err(unreadable)?,
            signatures: prober::parse_wipefs(&raw_wipefs).map_err(unreadable)?,
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
    // Not `from_utf8_lossy`. Replacing an undecodable byte with U+FFFD would
    // hand the parser a line that is not what the tool emitted, and the parsers
    // below refuse what they cannot read precisely so that a changed output
    // shape cannot pass as an empty observation.
    String::from_utf8(output.stdout).map_err(|error| {
        TaskError::Usage(format!(
            "`{tool}` produced output that is not UTF-8 for {}: {error}",
            path.display()
        ))
    })
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
    npm(&package, &["run", "typecheck"])?;
    npm(&package, &["test"])?;

    // This is the only gate with a Node toolchain, so SEC-010's advisory
    // requirement for npm dependencies is enforced here rather than in
    // `supply-chain`, which runs without Node.
    audit_npm_packages(&repository_root())
}

/// Audit **every** npm package in the repository, not one named directory.
///
/// The advisory check used to run in `packages/canonical` because that was the
/// only npm package there was. WP-030's shell reserves `packages/ui/`,
/// `packages/design-tokens/` and `apps/desktop/`, and a Tauri front end normally
/// brings its own `package.json` — each of which would have been audited by
/// nobody while the gate went on reporting success. Discovery is a tree walk for
/// the same reason the action scanner's is: a gate that checks a hard-coded path
/// silently stops covering the repository the moment the repository grows.
///
/// A package without a committed `package-lock.json` is a violation rather than
/// a skip. `docs/quality/dependency-policy.md` requires the lockfile, `npm audit`
/// needs it to know what is actually installed, and auditing a package whose
/// tree is decided at install time would report a verdict about nothing.
fn audit_npm_packages(root: &Path) -> Result<(), TaskError> {
    let mut manifests = Vec::new();
    manifest_files_under(root, &mut manifests)?;
    let packages: Vec<PathBuf> = manifests
        .into_iter()
        .filter(|path| path.file_name().and_then(OsStr::to_str) == Some("package.json"))
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();

    if packages.is_empty() {
        return Err(TaskError::Policy(
            "no npm package was found; refusing to report an advisory pass for a search that \
             matched nothing"
                .to_owned(),
        ));
    }

    let unlocked: Vec<String> = packages
        .iter()
        .filter(|directory| !directory.join("package-lock.json").is_file())
        .map(|directory| directory.display().to_string())
        .collect();
    if !unlocked.is_empty() {
        return Err(TaskError::Policy(format!(
            "every npm package needs a committed `package-lock.json`, or `npm audit` reports on a \
             tree that install time decides:\n  {}",
            unlocked.join("\n  ")
        )));
    }

    for directory in &packages {
        npm(directory, &["audit", "--audit-level=moderate"])?;
    }
    println!(
        "cross-language: {} npm package(s) audited at moderate and above",
        packages.len()
    );
    Ok(())
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

    verify_fuzz_lock()?;

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

/// Enforce the SEC-010 rule that every executable workflow dependency is
/// pinned by an immutable identifier.
///
/// Mutable tags such as `@v6` are rejected: they let an upstream account move a
/// release tag onto new code that would then run with repository credentials.
///
/// **Discovery is a structural YAML parse, after three failed attempts at doing
/// it by reading source text.** The history is worth keeping, because it is the
/// argument for the dependency:
///
/// 1. A line reader keyed on `uses:` — defeated by `"uses":`, a quoted key.
/// 2. The same reader plus refusals for shapes it could not parse — defeated by
///    `&pin uses:`, an anchored key, which it neither read nor refused.
/// 3. A syntax-independent sweep for `owner/repo@ref` tokens — defeated three
///    ways at once: `"actions/checkout@v7"` hides the `@` behind a YAML
///    escape the sweep never decodes, `docker://alpine:3.20` is a mutable
///    reference containing no `@` at all, and a local action outside
///    `.github/actions/` was never recursed into.
///
/// Each attempt reported *success with one fewer reference* — silence shaped
/// like a pass, which is the worst failure mode a gate has. The lesson is that
/// deciding what a YAML document *says* requires reading it as YAML. A parser
/// is pinned and audited like every other dependency; interpreting
/// security-relevant YAML incorrectly a fourth time is the larger risk.
///
/// Two layers, and they answer different questions:
///
/// - **Discovery and pinning** come from the parsed document. Every `uses`
///   mapping key anywhere in the tree is a reference, whatever surrounds it,
///   with its value decoded by the parser. Local references are resolved and
///   recursed into with a visited set.
/// - **Auditability** stays textual. A remote reference must also appear
///   plainly in the source with its release tag in a trailing comment, so a
///   reviewer can tell which release a digest is. A reference spelled so
///   obscurely that the text layer cannot find it fails this check — which is
///   deliberate, and is why writing one that way is a build failure rather than
///   a way to disappear.
///
/// The check fails closed when no workflow can be read, so a moved, renamed, or
/// unreadable workflow directory can never pass vacuously.
fn verify_action_pins(root: &Path) -> Result<(), TaskError> {
    let directory = root.join(WORKFLOW_DIRECTORY);
    let entries = fs::read_dir(&directory).map_err(|source| TaskError::Io {
        path: directory.clone(),
        source,
    })?;

    let mut queue = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        if is_yaml(&path) {
            queue.push(path);
        }
    }
    queue.sort();

    if queue.is_empty() {
        return Err(TaskError::Policy(format!(
            "no workflow files found in {}; SEC-010 action pinning cannot be verified",
            directory.display()
        )));
    }

    let mut violations = Vec::new();
    let mut pinned = 0_usize;
    let mut visited: BTreeSet<PathBuf> = BTreeSet::new();

    while let Some(file) = queue.pop() {
        let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
        if !visited.insert(canonical) {
            continue;
        }
        // Recursed files arrive canonicalized, which on Windows carries a
        // verbatim `\\?\` prefix that `strip_prefix(root)` cannot match. Strip
        // the canonical root too, so a violation names a path a reader can find.
        let display = root
            .canonicalize()
            .ok()
            .and_then(|canonical| file.strip_prefix(canonical).ok())
            .or_else(|| file.strip_prefix(root).ok())
            .unwrap_or(&file)
            .display()
            .to_string()
            .replace('\\', "/");
        let text = fs::read_to_string(&file).map_err(|source| TaskError::Io {
            path: file.clone(),
            source,
        })?;

        let documents = yaml_rust2::YamlLoader::load_from_str(&text).map_err(|error| {
            // A workflow this tool cannot parse is a violation, not a file to
            // skip: GitHub might still run it.
            TaskError::Policy(format!("{display} is not valid YAML: {error}"))
        })?;

        let mut found = Vec::new();
        for document in &documents {
            collect_dependencies(document, &mut found);
        }

        for dependency in found {
            check_dependency(
                &ScanContext {
                    root,
                    display: &display,
                    file: &file,
                    text: &text,
                },
                dependency,
                &mut queue,
                &mut violations,
                &mut pinned,
            )?;
        }
    }

    if violations.is_empty() {
        println!(
            "verify-actions: {pinned} remote reference(s) pinned and tagged across {} parsed \
             file(s)",
            visited.len()
        );
        Ok(())
    } else {
        Err(TaskError::Policy(format!(
            "SEC-010 requires every executable workflow dependency to be pinned to an immutable \
             identifier, with the release tag kept in a trailing comment. Offending \
             references:\n  {}",
            violations.join("\n  ")
        )))
    }
}

/// Where one dependency was declared, for the checks that need the file's text
/// or its neighbours on disk.
struct ScanContext<'a> {
    root: &'a Path,
    display: &'a str,
    file: &'a Path,
    text: &'a str,
}

/// Apply the SEC-010 policy to one declared dependency.
///
/// Extracted from `verify_action_pins` so each policy reads on its own: images
/// are pinned by content digest, Dockerfiles are followed to their base images,
/// local references are resolved and queued, and remote references need both an
/// immutable identifier and a readable release tag at every site.
fn check_dependency(
    context: &ScanContext<'_>,
    dependency: Dependency,
    queue: &mut Vec<PathBuf>,
    violations: &mut Vec<String>,
    pinned: &mut usize,
) -> Result<(), TaskError> {
    let display = context.display;
    let reference = match dependency {
        Dependency::Image(image) => {
            if names_a_dockerfile(&image) {
                // A Docker action building from source: the executable
                // dependency is that Dockerfile's base images.
                let dockerfile = context
                    .file
                    .parent()
                    .map(|directory| directory.join(image.trim_start_matches("./")))
                    .filter(|path| path.is_file());
                match dockerfile {
                    None => violations.push(format!(
                        "{display}: `image: {image}` names a Dockerfile that does not exist \
                         beside the action metadata"
                    )),
                    Some(path) => {
                        let body = fs::read_to_string(&path).map_err(|source| TaskError::Io {
                            path: path.clone(),
                            source,
                        })?;
                        for base in unpinned_dockerfile_bases(&body) {
                            violations.push(format!(
                                "{display}: Dockerfile base image `{base}` is not pinned by \
                                 digest; write `name@sha256:<64 hex>`"
                            ));
                        }
                    }
                }
                return Ok(());
            }
            if let Some(reason) = image_violation(&image) {
                violations.push(format!("{display}: {reason}"));
            } else {
                *pinned += 1;
            }
            return Ok(());
        }
        Dependency::Uses(reference) => reference,
    };

    if let ReferenceKind::Local(relative) = classify_reference(&reference) {
        // A local action or reusable workflow runs code from this repository, so
        // it needs no digest — but its *own* references are remote supply chain,
        // and assuming local actions live under `.github/actions/` left anywhere
        // else unread.
        match resolve_local_reference(context.root, &relative) {
            Ok(targets) => queue.extend(targets),
            Err(reason) => {
                violations.push(format!(
                    "{display}: local reference `{reference}` — {reason}"
                ));
            }
        }
        return Ok(());
    }

    if let Some(reason) = remote_violation(&reference) {
        violations.push(format!("{display}: {reference} — {reason}"));
        return Ok(());
    }

    let audit = every_occurrence_tagged(context.text, &reference);
    if let Some(comment) = audit.bad_comment {
        violations.push(format!(
            "{display}: {reference} — pinned, but {comment:?} does not name a release tag"
        ));
    } else if audit.occurrences == 0 {
        violations.push(format!(
            "{display}: {reference} — pinned, but no plain occurrence is readable in the source, \
             so no reviewer can see which release the digest is. Write the step plainly \
             (`uses: owner/repo@<sha> # vX.Y.Z`)"
        ));
    } else if audit.untagged > 0 {
        violations.push(format!(
            "{display}: {reference} — {} of {} occurrence(s) carry no `# <tag>` comment. Every \
             site needs its own; borrowing one from elsewhere in the file proves nothing about \
             this step",
            audit.untagged, audit.occurrences
        ));
    } else {
        *pinned += 1;
    }
    Ok(())
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
        })
}

/// Every `uses` mapping value in a parsed document, decoded.
///
/// Deliberately context-free: any `uses` or `image` key anywhere counts, rather
/// than only the positions GitHub documents today. Walking the whole tree cannot
/// miss a context this tool did not anticipate, and a stray key somewhere
/// harmless costs one false violation to fix rather than one silent omission to
/// be exploited.
///
/// `image` is collected because `uses` is not the only way a workflow runs
/// third-party code. A job container (`jobs.<id>.container.image`), a service
/// container (`jobs.<id>.services.<name>.image`), and a Docker action's
/// `runs.image` are all executable dependencies that GitHub pulls and runs, and
/// the previous version of this scanner saw none of them.
fn collect_dependencies(node: &yaml_rust2::Yaml, found: &mut Vec<Dependency>) {
    match node {
        yaml_rust2::Yaml::Hash(map) => {
            for (key, value) in map {
                match key.as_str() {
                    Some("uses") => {
                        if let Some(text) = value.as_str() {
                            found.push(Dependency::Uses(text.to_owned()));
                        }
                    }
                    Some("container") => {
                        // `container: alpine:3.20` is the shorthand for
                        // `container: { image: alpine:3.20 }`, so a scalar here
                        // is itself an image reference.
                        if let Some(text) = value.as_str() {
                            found.push(Dependency::Image(text.to_owned()));
                        }
                    }
                    Some("image") => {
                        if let Some(text) = value.as_str() {
                            found.push(Dependency::Image(text.to_owned()));
                        }
                    }
                    _ => {}
                }
                collect_dependencies(value, found);
            }
        }
        yaml_rust2::Yaml::Array(items) => {
            for item in items {
                collect_dependencies(item, found);
            }
        }
        _ => {}
    }
}

/// A declared dependency, and which policy applies to it.
enum Dependency {
    /// A `uses:` value: an action, a reusable workflow, or a `docker://` image.
    Uses(String),
    /// An `image:` or scalar `container:` value: a container GitHub pulls and
    /// runs, or the literal `Dockerfile` a Docker action builds.
    Image(String),
}

/// What kind of thing a `uses` value names.
enum ReferenceKind {
    /// A path inside this repository.
    Local(String),
    /// Anything fetched from outside: an action, a reusable workflow, or a
    /// container image.
    Remote,
}

fn classify_reference(reference: &str) -> ReferenceKind {
    if reference.starts_with("./") || reference.starts_with(".\\") {
        ReferenceKind::Local(reference[2..].replace('\\', "/"))
    } else {
        ReferenceKind::Remote
    }
}

/// Resolve a local `./...` reference to the file(s) whose own `uses` keys must
/// be inspected.
///
/// A directory must carry action metadata; a file is taken as a reusable
/// workflow. Both are required to stay beneath the repository root, so a
/// `./../..` reference is refused rather than followed out of the tree.
fn resolve_local_reference(root: &Path, relative: &str) -> Result<Vec<PathBuf>, String> {
    let candidate = root.join(relative);
    let resolved = candidate
        .canonicalize()
        .map_err(|error| format!("cannot resolve {relative}: {error}"))?;
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve the repository root: {error}"))?;
    if !resolved.starts_with(&canonical_root) {
        return Err(format!("{relative} resolves outside the repository"));
    }

    if resolved.is_dir() {
        // Containment is re-checked on the metadata file itself, not inferred
        // from the directory. `is_file()` follows links, so a symlinked
        // `action.yml` aimed outside the repository would otherwise be read and
        // trusted — the directory passing the check above says nothing about
        // where its contents point.
        let mut metadata = Vec::new();
        for name in ["action.yml", "action.yaml"] {
            let candidate = resolved.join(name);
            if !candidate.is_file() {
                continue;
            }
            let file = candidate
                .canonicalize()
                .map_err(|error| format!("cannot resolve {relative}/{name}: {error}"))?;
            if !file.starts_with(&canonical_root) {
                return Err(format!(
                    "{relative}/{name} resolves outside the repository, so its contents are not \
                     this repository's code to exempt"
                ));
            }
            metadata.push(file);
        }
        if metadata.is_empty() {
            return Err(format!(
                "{relative} is a directory with no action.yml or action.yaml, so GitHub could \
                 not run it and this tool cannot inspect what it would"
            ));
        }
        Ok(metadata)
    } else if is_yaml(&resolved) {
        Ok(vec![resolved])
    } else {
        Err(format!(
            "{relative} is neither a directory containing action metadata nor a YAML file"
        ))
    }
}

/// Why a remote reference fails SEC-010, or `None` if its identifier is
/// immutable.
fn remote_violation(reference: &str) -> Option<String> {
    if let Some(image) = reference.strip_prefix("docker://") {
        // `docker://alpine:3.20` is a documented step-level reference and is
        // mutable — a tag can be repointed at any image. Mutation B rode in on
        // the previous check requiring an `@` before it looked.
        return match image.rsplit_once("@sha256:") {
            Some((_, digest)) if is_lowercase_hex(digest, 64) => None,
            _ => Some(
                "a container image must be pinned by digest (`docker://image@sha256:<64 hex>`); \
                 a tag can be repointed at different code"
                    .to_owned(),
            ),
        };
    }
    if is_pinned(reference) {
        None
    } else {
        Some("not pinned to a full commit SHA".to_owned())
    }
}

/// Why an image reference fails SEC-010, or `None` if it is immutable.
///
/// A tag is not an identifier: `alpine:3.20` can be repointed at different code
/// by whoever controls the repository, which is the same substitution a mutable
/// action tag allows. `Dockerfile` is the one non-reference value GitHub
/// accepts, and it is handled by the caller, which reads that file's `FROM`
/// lines instead.
fn image_violation(image: &str) -> Option<String> {
    let bare = image.strip_prefix("docker://").unwrap_or(image);
    match bare.rsplit_once("@sha256:") {
        Some((_, digest)) if is_lowercase_hex(digest, 64) => None,
        _ => Some(format!(
            "container image `{image}` is not pinned by digest; write \
             `name@sha256:<64 hex>`, because a tag can be repointed at different code"
        )),
    }
}

/// Whether an `image:` value names a Dockerfile to build rather than an image to
/// pull.
fn names_a_dockerfile(image: &str) -> bool {
    let trimmed = image.trim_start_matches("./");
    trimmed == "Dockerfile" || trimmed.ends_with("/Dockerfile")
}

/// Every image a Dockerfile pulls that is not pinned by digest.
///
/// A Docker action with `image: Dockerfile` builds from source in the action
/// directory, so every image that build pulls is an executable dependency under
/// SEC-010 — and "every image" is wider than "every `FROM`".
///
/// Structural YAML parsing closed the workflow half of this gate. It did not
/// make *this* parser structural, and an audit plus an adversarial pass found
/// nine ways a mutable image passed unseen. Four needed no unusual syntax at
/// all:
///
/// - **A tab after `FROM`.** The old matcher was `strip_prefix("FROM ")`, one
///   literal space. `BuildKit` splits on `[\t\v\f\r ]+`, so every other member of
///   that class was invisible. This was the cheapest bypass in the file.
/// - **A UTF-8 BOM** on the first line, which `BuildKit` strips and `str::trim`
///   does not — the failure mode a Windows contributor produces by accident.
/// - **`COPY --from=<image>`** and **`RUN --mount=…,from=<image>`**, which pull
///   images that never appear in any `FROM`.
/// - **`FROM alpine AS alpine`**, where the stage was registered from the same
///   line before the base was tested against it, so the image shadowed itself.
///
/// The rest were the reviewer's three — a `$`-prefixed base skipped outright,
/// case-sensitive instruction matching, and the `# syntax=` parser directive
/// naming a `BuildKit` frontend image that was skipped as a comment — plus a
/// continuation whose `\` was stored as a stage name.
///
/// Two things it deliberately does *not* do, both checked rather than assumed:
/// `FROM scratch` is not a pull and is not a violation, and `# check=` and
/// `# escape=` name no image, so only `syntax=` is treated as a dependency.
fn unpinned_dockerfile_bases(text: &str) -> Vec<String> {
    let mut unpinned = Vec::new();
    let mut stages: BTreeSet<String> = BTreeSet::new();
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);

    for reference in parser_directive_images(body) {
        if image_violation(&reference).is_some() {
            unpinned.push(reference);
        }
    }

    for logical in logical_lines(body) {
        let trimmed = logical.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((instruction, rest)) = split_instruction(trimmed) else {
            continue;
        };

        // `COPY --from=` and `RUN --mount=…,from=` pull an image that no `FROM`
        // ever names.
        if instruction.eq_ignore_ascii_case("COPY")
            || instruction.eq_ignore_ascii_case("ADD")
            || instruction.eq_ignore_ascii_case("RUN")
        {
            for reference in mounted_images(rest) {
                if !is_stage(&stages, &reference) && image_violation(&reference).is_some() {
                    unpinned.push(reference);
                }
            }
            continue;
        }

        if !instruction.eq_ignore_ascii_case("FROM") {
            continue;
        }

        let mut tokens = rest
            .split(DOCKERFILE_SPACE)
            .filter(|token| !token.is_empty() && !token.starts_with("--"));
        let Some(base) = tokens.next() else { continue };

        // Test the base against the stages defined *before* this line, then
        // register this line's own stage. Registering first let
        // `FROM alpine AS alpine` shadow itself.
        let internal = is_stage(&stages, base);
        let mut remaining = tokens;
        if remaining
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("as"))
            && let Some(name) = remaining.next()
            && name != "\\"
        {
            stages.insert(name.to_ascii_lowercase());
        }
        if internal || base.eq_ignore_ascii_case("scratch") {
            continue;
        }
        // A variable base is refused rather than skipped. Resolving `ARG`
        // would have to prove no `--build-arg` can override it, so the first
        // policy is simply that a base image is written out.
        if base.contains('$') || image_violation(base).is_some() {
            unpinned.push(base.to_owned());
        }
    }
    unpinned
}

/// The whitespace `BuildKit` accepts between a Dockerfile instruction and its
/// arguments.
const DOCKERFILE_SPACE: [char; 5] = [' ', '\t', '\u{b}', '\u{c}', '\r'];

/// Join continuation lines, so a `FROM` split across two lines is one line.
fn logical_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        if let Some(head) = line.trim_end().strip_suffix('\\') {
            current.push_str(head);
            current.push(' ');
        } else {
            current.push_str(line);
            lines.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// An instruction keyword and the rest of its line.
fn split_instruction(line: &str) -> Option<(&str, &str)> {
    let end = line.find(DOCKERFILE_SPACE)?;
    let (instruction, rest) = line.split_at(end);
    Some((instruction, rest.trim_start_matches(DOCKERFILE_SPACE)))
}

/// Images named by `--from=` on `COPY`/`ADD`, or by a `from=` key inside a
/// `RUN --mount=`.
fn mounted_images(rest: &str) -> Vec<String> {
    let mut images = Vec::new();
    for token in rest.split(DOCKERFILE_SPACE) {
        if let Some(value) = token.strip_prefix("--from=") {
            images.push(value.to_owned());
        } else if let Some(mount) = token.strip_prefix("--mount=") {
            for field in mount.split(',') {
                if let Some(value) = field.strip_prefix("from=") {
                    images.push(value.to_owned());
                }
            }
        }
    }
    images
}

/// A `--from=` value naming an earlier stage is internal, not a pull. Stage
/// names are case-insensitive to `BuildKit`, and a numeric index names a stage
/// by position.
fn is_stage(stages: &BTreeSet<String>, reference: &str) -> bool {
    stages.contains(&reference.to_ascii_lowercase()) || reference.parse::<usize>().is_ok()
}

/// Images named by a parser directive.
///
/// `# syntax=<ref>` tells `BuildKit` to fetch a frontend image and run it as the
/// builder — an executable dependency by any reading of SEC-010, and one the
/// old scanner discarded as a comment. Directives are only legal before the
/// first instruction, and `escape=` and `check=` name no image.
fn parser_directive_images(text: &str) -> Vec<String> {
    let mut images = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(directive) = trimmed.strip_prefix('#') else {
            break;
        };
        let directive = directive.trim();
        let Some((key, value)) = directive.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("syntax") {
            images.push(value.trim().to_owned());
        }
    }
    images
}

/// Whether **every** plain occurrence of `reference` in the source carries a
/// release-tag comment, and how many were seen.
///
/// Binding matters. The previous version returned the first comment found
/// anywhere in the file for that reference, so two steps sharing a SHA — one
/// tagged, one bare — both passed on the tagged one's comment. Requiring every
/// occurrence to be tagged binds the check to each site without needing source
/// positions from the parser, and is stricter than pairing them one-to-one
/// would be.
fn every_occurrence_tagged(text: &str, reference: &str) -> TagAudit {
    let mut occurrences = 0_usize;
    let mut untagged = 0_usize;
    let mut bad_comment = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let (code, comment) = match trimmed.split_once('#') {
            Some((code, comment)) => (code, Some(comment.trim())),
            None => (trimmed, None),
        };
        if !code.contains(reference) {
            continue;
        }
        occurrences += 1;
        match comment.filter(|text| !text.is_empty()) {
            None => untagged += 1,
            Some(comment) if !names_a_release(comment) => {
                bad_comment = Some(comment.to_owned());
            }
            Some(_) => {}
        }
    }
    TagAudit {
        occurrences,
        untagged,
        bad_comment,
    }
}

/// What the textual auditability pass saw for one reference.
struct TagAudit {
    occurrences: usize,
    untagged: usize,
    bad_comment: Option<String>,
}

/// The SPDX expression every manifest in this repository must declare.
const PROJECT_LICENSE: &str = "MIT OR Apache-2.0";

/// Every tracked file is claimed by a work package, and every claim matches
/// something.
///
/// Section 1.10 says CI enforces path ownership via CODEOWNERS. It cannot:
/// CODEOWNERS requires an owner's review and says nothing about which work
/// package a path belongs to. `docs/traceability/WP-000.md` has recorded that
/// gap since WP-000, and both 2026-07-29 audits found real consequences —
/// WP-030 increment 1 edited five files it did not own, and the assignment was
/// widened afterwards to match.
///
/// This closes the half that is mechanically decidable. The `owned-paths`
/// blocks in `docs/work-packages/WP-*.md` are the single source of truth, so
/// the prose a reviewer reads *is* the data the checker parses, and:
///
/// - a file nothing claims is a violation, which is what catches a new file
///   appearing outside every assignment;
/// - a claim matching no file is a violation, which is what catches a stale or
///   mistyped claim — except in an `owned-paths-reserved` block, where matching
///   nothing is the point and is reported instead;
/// - overlaps are reported, not forbidden. `tools/xtask/**` is genuinely shared
///   by three packages, and forbidding that would only push the sharing back
///   into prose where nothing can see it.
///
/// **What this does not do:** decide whether a given change was made by the
/// package that owns the path. That needs a mapping from a pull request to a
/// work package, which is process metadata this repository does not carry, and
/// it is the remaining half of issue #39. Sub-file grants — "its own status rows
/// in `README.md`" — are narrower than any path checker can express and stay a
/// review obligation.
fn verify_path_ownership(root: &Path) -> Result<(), TaskError> {
    let tracked = tracked_files(root)?;
    let packages = ownership_claims(root)?;
    if packages.is_empty() {
        return Err(TaskError::Policy(
            "no work package declares an `owned-paths` block; ownership cannot be verified"
                .to_owned(),
        ));
    }

    let mut violations = Vec::new();
    let mut reservations = Vec::new();
    let mut overlaps: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (package, claims) in &packages {
        for claim in claims {
            let matched: Vec<&String> = tracked
                .iter()
                .filter(|file| claim_matches(&claim.pattern, file))
                .collect();
            if matched.is_empty() {
                if claim.reserved() {
                    reservations.push(format!("{package}: {}", claim.pattern));
                } else {
                    violations.push(format!(
                        "{package} claims `{}`, which matches no tracked file; a stale claim \
                         reads as coverage",
                        claim.pattern
                    ));
                }
            }
            // A derived declaration says how a path comes to be, not who owns
            // it. Counting it as coverage would let a path be declared
            // generated and then be claimed by nobody.
            //
            // A *reservation* that has started matching files is different, and
            // treating it as no coverage was a real defect: the two halves of
            // Section 1.10 disagreed. `verify_change_ownership` already lets a
            // package write inside its own reservation, so the first commit to
            // do so passed the change gate and then failed the inventory —
            // "claimed by no work package" about a path the package had claimed
            // in advance, in the document, precisely so this could not happen.
            // The promotion that would have fixed it has no legal route: a
            // governance change moving the paths early leaves `main` red on a
            // stale claim, and moving them in the same change as the files is
            // an assignment edit under a `Work-Package:` trailer, which
            // `AGENTS.md` forbids. So a reservation counts once it matches.
            if claim.kind != ClaimKind::Derived {
                for file in matched {
                    overlaps
                        .entry(file.clone())
                        .or_default()
                        .push(package.clone());
                }
            }
        }
    }

    for file in &tracked {
        if !overlaps.contains_key(file) {
            violations.push(format!(
                "{file} is claimed by no work package; add it to an `owned-paths` block or \
                 explain why it exists"
            ));
        }
    }

    let shared: Vec<(&String, &Vec<String>)> = overlaps
        .iter()
        .filter(|(_, owners)| owners.len() > 1)
        .collect();

    if violations.is_empty() {
        println!(
            "verify-ownership: {} tracked file(s) claimed across {} package(s); {} shared, \
             {} reserved",
            tracked.len(),
            packages.len(),
            shared.len(),
            reservations.len()
        );
        for reservation in &reservations {
            println!("  reserved (matches nothing yet): {reservation}");
        }
        Ok(())
    } else {
        Err(TaskError::Policy(format!(
            "Section 1.10 requires every path to belong to a work-package assignment. \
             Findings:\n  {}",
            violations.join("\n  ")
        )))
    }
}

/// One declared claim.
struct OwnershipClaim {
    pattern: String,
    kind: ClaimKind,
}

/// Which block a claim came from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClaimKind {
    /// `owned-paths`: this package authors the path.
    Owned,
    /// `owned-paths-reserved`: matching nothing yet is the point.
    Reserved,
    /// `derived-paths`: the path is generated from other files, so a change
    /// that regenerates it is not authoring it. See [`derivation_is_plausible`].
    Derived,
}

impl OwnershipClaim {
    fn reserved(&self) -> bool {
        self.kind == ClaimKind::Reserved
    }

    fn derived(&self) -> bool {
        self.kind == ClaimKind::Derived
    }
}

/// Files git tracks, as forward-slash relative paths.
///
/// `git ls-files` rather than a directory walk, so `.gitignore` is honoured by
/// the tool that defines it instead of being reimplemented. Fails closed: no
/// git, no output, or an empty list is an error, because a check that verifies
/// ownership of nothing would pass.
fn tracked_files(root: &Path) -> Result<Vec<String>, TaskError> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .map_err(|source| TaskError::Launch {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(TaskError::Policy(format!(
            "`git ls-files` failed, so path ownership cannot be verified: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect();
    if files.is_empty() {
        return Err(TaskError::Policy(
            "`git ls-files` reported no files; refusing to verify ownership of nothing".to_owned(),
        ));
    }
    Ok(files)
}

/// Parse every `owned-paths` block out of `docs/work-packages/WP-*.md`.
fn ownership_claims(root: &Path) -> Result<BTreeMap<String, Vec<OwnershipClaim>>, TaskError> {
    let directory = root.join("docs/work-packages");
    let entries = fs::read_dir(&directory).map_err(|source| TaskError::Io {
        path: directory.clone(),
        source,
    })?;
    let mut packages = BTreeMap::new();
    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned();
        if !name.starts_with("WP-") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|source| TaskError::Io {
            path: path.clone(),
            source,
        })?;
        packages.insert(name.clone(), parse_owned_paths(&name, &text)?);
    }
    Ok(packages)
}

/// The ownership catalogue **as of a git revision**.
///
/// Reading the catalogue from the base rather than the working tree is what
/// stops a pull request widening its own `owned-paths` block and then passing
/// against the widened version — the hole the 2026-07-29 second follow-up audit
/// identified in the inventory check.
fn ownership_claims_at(
    root: &Path,
    revision: &str,
) -> Result<BTreeMap<String, Vec<OwnershipClaim>>, TaskError> {
    let listing = git(
        root,
        &["ls-tree", "--name-only", revision, "docs/work-packages/"],
    )?;
    let mut packages = BTreeMap::new();
    for path in listing.lines() {
        let path = path.trim();
        let Some(name) = Path::new(path)
            .file_stem()
            .and_then(OsStr::to_str)
            .filter(|stem| stem.starts_with("WP-"))
        else {
            continue;
        };
        let text = git(root, &["show", &format!("{revision}:{path}")])?;
        packages.insert(name.to_owned(), parse_owned_paths(name, &text)?);
    }
    if packages.is_empty() {
        return Err(TaskError::Policy(format!(
            "no work-package assignments found at {revision}; change ownership cannot be verified"
        )));
    }
    Ok(packages)
}

/// Parse the `owned-paths`, `owned-paths-reserved` and `derived-paths` blocks
/// out of one work-package document.
fn parse_owned_paths(name: &str, text: &str) -> Result<Vec<OwnershipClaim>, TaskError> {
    let mut claims = Vec::new();
    let mut inside: Option<ClaimKind> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        match inside {
            None => {
                if trimmed == "```owned-paths" {
                    inside = Some(ClaimKind::Owned);
                } else if trimmed == "```owned-paths-reserved" {
                    inside = Some(ClaimKind::Reserved);
                } else if trimmed == "```derived-paths" {
                    inside = Some(ClaimKind::Derived);
                }
            }
            Some(kind) => {
                if trimmed == "```" {
                    inside = None;
                    continue;
                }
                let pattern = trimmed
                    .split_once('#')
                    .map_or(trimmed, |(before, _)| before.trim());
                if pattern.is_empty() {
                    continue;
                }
                validate_claim_pattern(name, pattern)?;
                if kind == ClaimKind::Derived {
                    validate_derived_pattern(name, pattern)?;
                }
                claims.push(OwnershipClaim {
                    pattern: pattern.to_owned(),
                    kind,
                });
            }
        }
    }
    if inside.is_some() {
        return Err(TaskError::Policy(format!(
            "{name}: an `owned-paths` block is not closed"
        )));
    }
    if !claims.iter().any(|claim| claim.kind == ClaimKind::Owned) {
        return Err(TaskError::Policy(format!(
            "{name} declares no owned paths; every work package must state its assignment in an \
             `owned-paths` block"
        )));
    }
    Ok(claims)
}

/// Every path a change touches belongs to the work package that change declares.
///
/// This is the half of Section 1.10 that `verify-ownership` deliberately does
/// not attempt, and the 2026-07-29 second follow-up audit was right that the
/// inventory alone is not enough: *"a feature PR can widen its own `owned-paths`
/// block and then pass against the widened current tree."* It caught a real
/// instance — PR #47 was a nominal WP-000 change that also edited WP-010,
/// WP-020 and WP-030 documents, and the inventory passed because every path was
/// claimed by *someone*.
///
/// Two design choices make it work without new infrastructure:
///
/// - **The declaration is a commit trailer**, `Work-Package: WP-030`. This
///   repository already uses trailers (`Co-Authored-By`), the value stays in the
///   log forever, and it needs no API call, no label, and no branch-name
///   convention — branch names here are inconsistent, so keying on them would
///   have been a guess dressed as a rule.
/// - **The catalogue is read from the base revision**, never the working tree.
///   Widening your own assignment in the same change therefore buys nothing,
///   which is the specific hole the audit named.
///
/// A change to the assignments themselves needs `Governance: <reason>`, and in
/// that mode **only** work-package documents may change — so a governance
/// trailer cannot be used to smuggle code past the check.
fn verify_change_ownership(root: &Path, base: &str) -> Result<(), TaskError> {
    let range = format!("{base}...HEAD");
    let changed = changed_paths(root, &range)?;

    if changed.is_empty() {
        println!("verify-change-ownership: no paths changed against {base}");
        return Ok(());
    }

    let (declared, governance) = read_declarations(root, base, changed.len())?;

    // The mode must be unanimous. Governance used to win by being tested first,
    // so a range mixing the two was judged as paperwork and the work package it
    // also declared was never checked against anything.
    if !declared.is_empty() && !governance.is_empty() {
        return Err(TaskError::Policy(format!(
            "this range mixes an assignment change with ordinary work: {} declared, and \
             `Governance:` given as {}. Land the assignment change as its own pull request first \
             — that ordering is the point of the separation, not a formality",
            declared
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            governance.join("; ")
        )));
    }

    if !governance.is_empty() {
        return governance_change(&changed, &governance);
    }

    let package = match declared.len() {
        0 => {
            return Err(TaskError::Policy(
                "no commit declares a work package. Add a `Work-Package: WP-0NN` trailer to the \
                 commit, or `Governance: <reason>` if the change edits assignments themselves. \
                 Section 1.10 requires a change to belong to an assignment, and no tool can \
                 infer which one from a diff"
                    .to_owned(),
            ));
        }
        1 => declared.iter().next().expect("one element").clone(),
        _ => {
            return Err(TaskError::Policy(format!(
                "commits declare more than one work package ({}). One change belongs to one \
                 assignment; a shared path still has exactly one owning package for a given \
                 change. Split the pull request",
                declared
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    };

    let catalogue = ownership_claims_at(root, base)?;
    let claims = catalogue.get(&package).ok_or_else(|| {
        TaskError::Policy(format!(
            "commits declare `{package}`, which has no assignment at {base}. A new work package \
             needs its `docs/work-packages/{package}.md` and its `owned-paths` block to land in a \
             `Governance:` change first"
        ))
    })?;

    // Declared generated by *any* package — but only by one that also answers
    // for the path. Generatedness is a property of the file rather than a
    // privilege of one assignment, and that argument survives; what needed
    // guarding is a document asserting it about a file it does not own, which
    // would have let any package grant every package the exemption.
    let derived: Vec<&str> = catalogue
        .values()
        .flat_map(|declared| {
            declared.iter().filter(|claim| {
                claim.derived()
                    && declared.iter().any(|owned| {
                        owned.kind == ClaimKind::Owned && owned.pattern == claim.pattern
                    })
            })
        })
        .map(|claim| claim.pattern.as_str())
        .collect();

    // Which manifests actually resolve into each generated lockfile the change
    // touches. Asked of cargo, and only for a lockfile this change carries, so
    // an ordinary change pays nothing for the question.
    let mut resolves = BTreeSet::new();
    for pattern in &derived {
        if changed.iter().any(|path| claim_matches(pattern, path)) {
            resolves.extend(workspace_manifests(root, pattern)?);
        }
    }

    let (strays, regenerated) = classify(&changed, claims, &derived, &resolves);

    if !strays.is_empty() {
        return Err(stray_paths(&package, base, &strays, &derived, &resolves));
    }
    println!(
        "verify-change-ownership: {} path(s) belong to {package} as assigned at {base}; {} \
         regenerated, not authored",
        changed.len() - regenerated.len(),
        regenerated.len()
    );
    for path in &regenerated {
        println!("  regenerated: {path}");
    }
    Ok(())
}

/// Split the changed paths into the ones this assignment cannot account for and
/// the ones it regenerated rather than authored.
fn classify<'a>(
    changed: &'a [String],
    claims: &[OwnershipClaim],
    derived: &[&str],
    resolves: &BTreeSet<String>,
) -> (Vec<String>, Vec<&'a str>) {
    let mut strays = Vec::new();
    let mut regenerated = Vec::new();
    for path in changed {
        let assigned = claims
            .iter()
            .any(|claim| !claim.derived() && claim_matches(&claim.pattern, path));
        if assigned {
            continue;
        }
        if derived.iter().any(|pattern| claim_matches(pattern, path))
            && derivation_is_plausible(changed, resolves)
        {
            regenerated.push(path.as_str());
        } else {
            strays.push(path.clone());
        }
    }
    (strays, regenerated)
}

/// The refusal, naming every stray and why a generated one was still refused.
fn stray_paths(
    package: &str,
    base: &str,
    strays: &[String],
    derived: &[&str],
    resolves: &BTreeSet<String>,
) -> TaskError {
    let alone: Vec<&str> = strays
        .iter()
        .map(String::as_str)
        .filter(|path| derived.iter().any(|pattern| claim_matches(pattern, path)))
        .collect();
    let note = if alone.is_empty() {
        String::new()
    } else {
        // Name the manifests that would have worked. The author who trips this
        // has usually edited *a* manifest and is entitled to know why it did not
        // count — an audit found the first version pointing them at the thing
        // they had already done.
        let mut resolving: Vec<&str> = resolves.iter().map(String::as_str).collect();
        resolving.sort_unstable();
        format!(
            "\n\n{} of these are generated files, and a generated file moving on its own is not \
             regeneration — nothing in this change asks the generator for a different answer:\n  \
             {}\n\nCarrying one needs a change to a manifest it actually resolves. Cargo says \
             those are:\n  {}\n\nOtherwise the pin belongs to the package that owns the lockfile.",
            alone.len(),
            alone.join("\n  "),
            if resolving.is_empty() {
                "(none — cargo was not asked, because no generated path was claimed here)"
                    .to_owned()
            } else {
                resolving.join("\n  ")
            }
        )
    };
    TaskError::Policy(format!(
        "this change declares `{package}`, but {} path(s) are outside that assignment as it stood \
         at {base}. Widening the assignment in this same change does not help — the catalogue is \
         read from the base for exactly that reason. Either land the assignment change as a \
         separate `Governance:` pull request, or move these edits to the package that owns \
         them:\n  {}{note}",
        strays.len(),
        strays.join("\n  ")
    ))
}

/// Every path a change touches, both halves of a rename included.
///
/// Three defects lived in the one line this replaces, and an audit found all
/// three in the same expression:
///
/// - **`--no-renames`.** Rename detection is on by default, and `--name-only`
///   prints only a rename's *destination*. So `git mv` moved a file out of
///   another package's territory and the gate checked only where it landed —
///   and worse, a `Governance:` change could delete any file in the repository
///   by renaming it to a `docs/work-packages/WP-*.md` name, because the source
///   never appeared. Disabling detection makes the source a deletion and the
///   destination an addition, so both are judged.
/// - **`-z`.** `--name-only` C-quotes any path with a non-ASCII byte, so a
///   legitimate `crates/tokens/src/café.rs` arrived as `"crates/..caf\303\251.rs"`
///   and matched no claim — a gate refusing work it should permit, which erodes
///   trust in it exactly as fast as a bypass does. `verify_path_ownership`
///   already used `ls-files -z`; this makes the two agree.
/// - **No `trim`.** Git does not quote a leading space, so `.map(str::trim)`
///   silently normalised ` crates/tokens/src/lib.rs` onto the owned path. A
///   path is a byte string, and trimming it is a normalisation the ownership
///   catalogue never agreed to.
fn changed_paths(root: &Path, range: &str) -> Result<Vec<String>, TaskError> {
    Ok(
        git(root, &["diff", "--no-renames", "-z", "--name-only", range])?
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

/// A `Governance:` change may edit the assignments themselves, and nothing else.
///
/// Restricting the blast radius is what stops the trailer becoming a universal
/// bypass for the check it sits beside.
fn governance_change(changed: &[String], reasons: &[String]) -> Result<(), TaskError> {
    let stray: Vec<&str> = changed
        .iter()
        .map(String::as_str)
        .filter(|path| !is_assignment_document(path))
        .collect();
    if stray.is_empty() {
        println!(
            "verify-change-ownership: governance change touching {} assignment document(s) ({})",
            changed.len(),
            reasons.join("; ")
        );
        return Ok(());
    }
    Err(TaskError::Policy(format!(
        "a `Governance:` change may only edit work-package assignments, so that the trailer \
         cannot be used to carry code past the ownership check. Also changed:\n  {}",
        stray.join("\n  ")
    )))
}

/// The declaration on **every non-merge commit** in the range.
///
/// Two things were wrong with the version this replaces, and both let a commit
/// travel without saying what it belonged to.
///
/// **The trailers were unioned across the range.** One declaring commit
/// laundered every undeclared commit beside it, so a two-commit pull request
/// passed with a trailer on only the second. Each commit is now asked
/// individually, and one that declares nothing is named in the refusal.
///
/// **The parse was a line scan, not a trailer parse.** Any line that began with
/// the key after trimming counted — including a fenced example inside a commit
/// body, and including a leading-space continuation line that git does not treat
/// as a trailer at all — while a genuine lowercase `work-package:` trailer, which
/// git accepts, was refused. Git's own parser now answers, through the
/// `%(trailers:…)` atom in the same `git log` call: same semantics as
/// `git interpret-trailers --parse`, no second process, and no house dialect to
/// keep in step with git's.
///
/// **Merge commits are exempt, deliberately and not by accident.** They author
/// no content of their own, and the exemption is not a convenience: branch
/// protection is `strict: true`, so `gh pr update-branch` merges `main` into
/// every stale branch and GitHub writes those merge commits itself — three are
/// already in this history — and for a `pull_request` event CI checks out
/// GitHub's generated `refs/pull/N/merge`. `main` carries 51 merge commits and
/// exactly none of them has a trailer. A literal "every commit" rule would fail
/// every pull request on the day it landed, which is why the documents that
/// claimed one were corrected rather than the code being tightened to match.
fn read_declarations(
    root: &Path,
    base: &str,
    changed: usize,
) -> Result<(BTreeSet<String>, Vec<String>), TaskError> {
    const RECORD: &str = "%H%x1f%(trailers:key=Work-Package,valueonly,only)\
                          %x1f%(trailers:key=Governance,valueonly,only)%x00";
    let log = git(
        root,
        &[
            "log",
            "--no-merges",
            &format!("--format={RECORD}"),
            &format!("{base}..HEAD"),
        ],
    )?;

    let mut declared = BTreeSet::new();
    let mut governance = Vec::new();
    let mut undeclared = Vec::new();
    let mut both = Vec::new();
    let mut seen = 0_usize;

    for record in log.split('\0').filter(|record| !record.trim().is_empty()) {
        seen += 1;
        let mut fields = record.trim_start_matches(['\r', '\n']).split('\x1f');
        let commit = fields.next().unwrap_or_default().trim();
        let short = commit.get(..8).unwrap_or(commit);
        let packages = trailer_values(fields.next().unwrap_or_default());
        let reasons = trailer_values(fields.next().unwrap_or_default());

        match (packages.is_empty(), reasons.is_empty()) {
            (true, true) => undeclared.push(short.to_owned()),
            (false, false) => both.push(short.to_owned()),
            _ => {
                declared.extend(packages);
                governance.extend(reasons);
            }
        }
    }

    if seen == 0 {
        return Err(TaskError::Policy(format!(
            "{changed} path(s) differ from {base} but no commit does so; refusing to guess which \
             work package owns the change"
        )));
    }
    if !both.is_empty() {
        return Err(TaskError::Policy(format!(
            "a change is either ordinary work or a change to the assignments, never both, but {} \
             commit(s) declare `Work-Package:` and `Governance:` together: {}",
            both.len(),
            both.join(", ")
        )));
    }
    if !undeclared.is_empty() {
        return Err(TaskError::Policy(format!(
            "{} commit(s) carry no declaration: {}. Every non-merge commit needs a \
             `Work-Package: WP-0NN` trailer, or `Governance: <reason>` if it edits assignments. \
             A trailer is git's own last-paragraph `Key: value`, so an example quoted in the body \
             does not count and a rewritten message must keep it",
            undeclared.len(),
            undeclared.join(", ")
        )));
    }
    Ok((declared, governance))
}

/// Non-empty trailer values, one per line, as `%(trailers:valueonly)` emits them.
///
/// An empty value is dropped rather than recorded: `Governance:` with no reason
/// used to be accepted and printed as an empty parenthesis, which is an audit
/// record of nothing. Dropping it here makes the commit undeclared, and the
/// refusal then says so by name.
fn trailer_values(field: &str) -> Vec<String> {
    field
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether a path is a work-package assignment document.
///
/// Case-insensitive on the extension: git can carry `WP-020.MD`, and a
/// governance check that silently declined to recognise it would let an
/// assignment edit slip through as ordinary code — or the reverse.
fn is_assignment_document(path: &str) -> bool {
    path.starts_with("docs/work-packages/WP-")
        && Path::new(path)
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

/// Run a git command in `root` and return its stdout.
fn git(root: &Path, args: &[&str]) -> Result<String, TaskError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| TaskError::Launch {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(TaskError::Policy(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Only exact paths and `prefix/**` are supported, and anything else is an
/// error rather than a pattern quietly matching nothing — the failure mode the
/// action scanner was audited for twice.
fn validate_claim_pattern(package: &str, pattern: &str) -> Result<(), TaskError> {
    if pattern.contains('\\') {
        return Err(TaskError::Policy(format!(
            "{package}: claim `{pattern}` uses a backslash; declare paths with forward slashes"
        )));
    }
    let body = pattern.strip_suffix("/**").unwrap_or(pattern);
    if body.contains('*') {
        return Err(TaskError::Policy(format!(
            "{package}: claim `{pattern}` uses an unsupported wildcard. Only an exact path or \
             `directory/**` is understood"
        )));
    }
    Ok(())
}

/// A derived path is only understood if this tool knows how it is derived.
///
/// `Cargo.lock` is the one derivation defined today. Anything else is refused
/// rather than exempted, because a `derived-paths` block is an *exemption* from
/// the ownership check and an exemption nobody can verify is a hole with a
/// comment beside it. Adding a second kind means writing its rule first.
fn validate_derived_pattern(package: &str, pattern: &str) -> Result<(), TaskError> {
    if is_named(pattern, "Cargo.lock") {
        return Ok(());
    }
    Err(TaskError::Policy(format!(
        "{package}: `{pattern}` is declared derived, but no derivation is defined for it. A \
         derived path is exempt from change ownership, so the exemption may only cover a path \
         whose regeneration this tool can check. Today that is a Cargo lockfile"
    )))
}

/// Whether a change plausibly *regenerated* a derived path rather than edited it.
///
/// A Cargo lockfile is a function of the manifests it resolves. Cargo already
/// proves the two agree — `--locked` sits in the `xtask` alias, so a lock that
/// does not match its manifests refuses before any gate runs. What that cannot
/// see is a lockfile changed **on its own**: re-pinning a transitive dependency
/// to a different version with a valid checksum still satisfies every manifest.
///
/// So the exemption is conditional. A change may carry a lockfile it does not
/// own when it also changes a manifest, because then the lockfile churn is a
/// consequence of work the change is entitled to do. A lockfile moving by
/// itself is not regeneration, and belongs to the package that owns it.
///
/// The manifest must be one this lockfile actually resolves, and **cargo is
/// asked, not the pathname**. Two earlier versions of this rule guessed:
///
/// - The first accepted any `Cargo.toml` anywhere. `fuzz/` is *excluded* from
///   the root workspace and carries its own lockfile, so editing
///   `fuzz/Cargo.toml` cannot change the root `Cargo.lock` — yet it unlocked it.
/// - The second matched a manifest to the nearest lockfile above it. That was a
///   proxy for workspace membership, and the proxy was writable: an adversarial
///   pass showed a file merely *named* `Cargo.toml` — a note, a fixture, a
///   symlink — anywhere a package owned would unlock the root lockfile, and that
///   deleting `fuzz/Cargo.lock` in one pull request let `fuzz/Cargo.toml` vouch
///   for the root lock in the next while `fuzz` stayed excluded.
///
/// A fourth lexical predicate standing in for a semantic fact was not worth
/// writing. [`workspace_manifests`] asks `cargo metadata` which manifests belong
/// to the workspace that lockfile locks, so membership is answered by the tool
/// that defines it. A path that is not a member's manifest is not a manifest,
/// whatever it is called.
///
/// **What this does not establish:** a re-pin travelling *alongside* a genuine
/// manifest change passes. Distinguishing the two needs the resolver's answer at
/// both revisions, which means base's whole tree and a full resolution on every
/// pull request. The residual risk is the same one the repository has always
/// carried — nothing here makes it worse — and `cargo deny`, `cargo audit` and
/// owner review are what stand against it. Recorded in
/// `docs/quality/dependency-policy.md` rather than implied to be covered.
fn derivation_is_plausible(changed: &[String], resolves: &BTreeSet<String>) -> bool {
    changed.iter().any(|path| resolves.contains(path))
}

/// Every workspace member inherits the workspace lint policy.
///
/// `[workspace.lints]` denies `unsafe_code` and warns on `missing_docs`, and CI
/// turns warnings into errors — but **only for a crate that opts in** with
/// `[lints] workspace = true`. A member that omits the stanza inherits nothing,
/// and this was measured rather than reasoned about: a new member with an
/// `unsafe fn` in it produced **zero** diagnostics and `cargo xtask ci` stayed
/// green.
///
/// That is the shape this repository rejects everywhere else — a safety property
/// resting on a declaration somebody has to remember rather than on something
/// computed. The membership list comes from `cargo metadata`, so a crate cannot
/// escape by not being mentioned here.
///
/// The manifest **text** is read rather than `cargo metadata`, because metadata
/// resolves the inheritance away: by the time cargo reports a package, a manifest
/// that opted in and one that did not look the same.
fn verify_workspace_lints(root: &Path) -> Result<(), TaskError> {
    let mut violations = Vec::new();
    let members = workspace_manifests(root, "Cargo.lock")?;
    let mut checked = 0_usize;

    for relative in &members {
        if relative == "Cargo.toml" {
            continue; // The virtual root declares the policy; it inherits nothing.
        }
        let path = root.join(relative);
        let text = fs::read_to_string(&path).map_err(|source| TaskError::Io {
            path: path.clone(),
            source,
        })?;
        checked += 1;
        if !inherits_workspace_lints(&text) {
            violations.push(format!(
                "{relative} does not declare `[lints]\\nworkspace = true`, so it inherits none of \
                 `[workspace.lints]` — `unsafe_code = \"deny\"` included"
            ));
        }
    }

    if violations.is_empty() {
        println!("verify-lints: {checked} workspace member(s) inherit the workspace lint policy");
        return Ok(());
    }
    Err(TaskError::Policy(format!(
        "SAFE-009 rests on `unsafe_code = \"deny\"` reaching every crate, and a member that omits \
         the opt-in silently escapes it:\n  {}",
        violations.join("\n  ")
    )))
}

/// Whether a manifest opts into `[workspace.lints]`.
///
/// Comments are stripped first: `# [lints]` above `# workspace = true` is not an
/// opt-in, and a check that read it as one would report coverage that does not
/// exist.
fn inherits_workspace_lints(manifest: &str) -> bool {
    let mut inside = false;
    for line in manifest.lines() {
        let code = line
            .split_once('#')
            .map_or(line, |(before, _)| before)
            .trim();
        if code.starts_with('[') {
            inside = code == "[lints]";
            continue;
        }
        if inside && code.replace(' ', "") == "workspace=true" {
            return true;
        }
    }
    false
}

/// Every manifest belonging to the workspace that `lockfile` locks.
///
/// The workspace root manifest is included explicitly, because this repository's
/// root is a *virtual* manifest — `[workspace]` with no `[package]` — so it
/// never appears in `packages`, and adding a member to it is the single most
/// legitimate reason for the lockfile to move.
///
/// `--locked` for the reason it sits in the alias: this check must not be the
/// thing that repairs a stale lockfile. A workspace that does not resolve is an
/// error here rather than an empty set, so a broken manifest cannot quietly
/// become "no manifest changed".
fn workspace_manifests(root: &Path, lockfile: &str) -> Result<BTreeSet<String>, TaskError> {
    let directory = lockfile.rsplit_once('/').map(|(parent, _)| parent);
    let manifest = directory.map_or_else(
        || "Cargo.toml".to_owned(),
        |parent| format!("{parent}/Cargo.toml"),
    );

    let (_, paths) = cargo_package_licenses(root, &manifest)?;
    let mut manifests = BTreeSet::new();
    manifests.insert(manifest);
    for path in paths {
        manifests.insert(relative_to_root(root, &path)?);
    }
    Ok(manifests)
}

/// A manifest path from `cargo metadata`, as a repository-relative git path.
///
/// **Both spellings of the root are tried, and failing to relativize is an
/// error.** The first version silently dropped a path it could not strip, and
/// macOS caught it within minutes of the change reaching CI: `std::env::temp_dir`
/// is `/var/folders/…`, `/var` is a symlink to `/private/var`, and `cargo
/// metadata` answers with the resolved `/private/var/…`. Every member manifest
/// failed to strip, the set silently shrank to the workspace root alone, and a
/// legitimate change was refused with a message listing manifests that did not
/// include the one the author had just edited.
///
/// Canonicalizing the root alone would trade one platform for another: on
/// Windows `fs::canonicalize` yields a `\\?\` prefix that `cargo metadata`'s
/// plain `D:\…` never matches. So both are tried, and a path that matches
/// neither is a refusal rather than a quiet omission — a check that forgets part
/// of its own input is the failure mode this repository keeps finding.
fn relative_to_root(root: &Path, path: &Path) -> Result<String, TaskError> {
    let canonical = fs::canonicalize(root).ok();
    let relative = path
        .strip_prefix(root)
        .ok()
        .or_else(|| {
            canonical
                .as_deref()
                .and_then(|base| path.strip_prefix(base).ok())
        })
        .ok_or_else(|| {
            TaskError::Policy(format!(
                "`cargo metadata` reported the manifest {} , which is not inside {}. Refusing to \
                 drop it: a workspace-membership answer with a member missing would refuse a \
                 change that is entitled to carry the lockfile",
                path.display(),
                root.display()
            ))
        })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn is_named(path: &str, file: &str) -> bool {
    Path::new(path).file_name().and_then(OsStr::to_str) == Some(file)
}

fn claim_matches(pattern: &str, file: &str) -> bool {
    match pattern.strip_suffix("/**") {
        Some(prefix) => file.starts_with(&format!("{prefix}/")),
        None => pattern == file,
    }
}

/// Refuse if `fuzz/Cargo.lock` no longer matches `fuzz/Cargo.toml`.
///
/// The fuzz crate is excluded from the workspace, so nothing in the root graph
/// proves its committed lockfile is current. `--locked` refuses rather than
/// repairs; without it a fresh checkout resolved the fuzzer dependencies to
/// whatever the registry served that day and ran their build scripts outside
/// every policy gate — on the job that exists to execute hostile-byte parser
/// tests.
///
/// **Called first by both `supply-chain` and `fuzz`.** It began life inside
/// `fuzz()` alone, and the 2026-07-29 follow-up audit showed why that was not
/// enough: `cargo deny` resolves the manifest to build its graph, so running
/// it first silently *repaired* a stale lock and then audited the repaired
/// version — the policy tool committing the very fail-open shape it exists to
/// catch, and leaving a subsequent `fuzz` preflight nothing left to refuse.
/// Deliberately runs before the nightly toolchain is involved, so a stale lock
/// is reported as a stale lock rather than as a toolchain problem.
fn verify_fuzz_lock() -> Result<(), TaskError> {
    let root = repository_root();
    if !root.join("fuzz/Cargo.toml").is_file() {
        return Ok(());
    }
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            "fuzz/Cargo.toml",
        ])
        .current_dir(&root)
        .output()
        .map_err(|source| TaskError::Launch {
            program: "cargo".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(TaskError::Policy(format!(
            "fuzz/Cargo.lock does not match fuzz/Cargo.toml; commit an updated lock rather \
             than letting a policy or fuzz command resolve fresh dependencies outside the \
             supply-chain gates.\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

/// Every manifest declares the project licence, and the licence texts exist.
///
/// `cargo deny` gates the workspace crates, but two manifests sit outside its
/// graph: `fuzz/Cargo.toml` is excluded from the workspace, and
/// `packages/canonical/package.json` is not Cargo at all. Until this check
/// existed, either could lose its licence declaration with every gate green —
/// recorded as a known gap in `docs/traceability/WP-000.md` from the day the
/// licence was adopted.
///
/// **The checks are semantic, not lexical.** The first version matched trimmed
/// lines, and the 2026-07-29 follow-up audit defeated it by moving the JSON
/// property under a `metadata` object: the line still read
/// `"license": "MIT OR Apache-2.0"`, while the document's root `license` was
/// `undefined`. A line cannot tell you where in a document it sits. Cargo
/// licences now come from `cargo metadata`, which resolves workspace
/// inheritance and is the same view the toolchain has, and `package.json` is
/// parsed as JSON with the property required at the root.
///
/// The tree walk is kept so a manifest neither Cargo graph knows about is still
/// found the day it is added, not the day someone remembers to register it.
fn verify_manifest_licenses(root: &Path) -> Result<(), TaskError> {
    let mut violations = Vec::new();
    let mut checked = 0_usize;

    for text_name in ["LICENSE-MIT", "LICENSE-APACHE"] {
        let path = root.join(text_name);
        checked += 1;
        if !fs::metadata(&path).is_ok_and(|meta| meta.is_file() && meta.len() > 0) {
            violations.push(format!(
                "{text_name} is missing or empty; the declared licence has no text"
            ));
        }
    }

    // Authoritative for Cargo: `cargo metadata` reports the licence each
    // package actually resolves to, including through `license.workspace`.
    let mut cargo_seen: Vec<PathBuf> = Vec::new();
    for workspace in ["Cargo.toml", "fuzz/Cargo.toml"] {
        let manifest = root.join(workspace);
        if !manifest.is_file() {
            continue;
        }
        let (declared, members) = cargo_package_licenses(root, workspace)?;
        cargo_seen.extend(members);
        // A virtual workspace manifest carries no `[package]`, so
        // `cargo metadata --no-deps` never lists it. It is an entry point
        // rather than a package, and its `[workspace.package] license` is what
        // the members above were just resolved through.
        cargo_seen.push(manifest);
        for (package, license) in declared {
            checked += 1;
            if license.as_deref() != Some(PROJECT_LICENSE) {
                violations.push(format!(
                    "{workspace}: package `{package}` resolves to licence {} rather than \
                     `{PROJECT_LICENSE}`",
                    license.as_deref().unwrap_or("<none>")
                ));
            }
        }
    }

    let mut manifests = Vec::new();
    manifest_files_under(root, &mut manifests)?;
    manifests.sort();
    for manifest in &manifests {
        let name = manifest
            .strip_prefix(root)
            .unwrap_or(manifest)
            .display()
            .to_string()
            .replace('\\', "/");
        if manifest
            .file_name()
            .is_some_and(|file| file == OsStr::new("Cargo.toml"))
        {
            // Covered authoritatively above — unless no Cargo graph mentioned
            // it, which means a manifest exists that neither workspace knows
            // about. That is a violation, not a file to skip.
            checked += 1;
            // Compare canonically: `cargo metadata` reports absolute resolved
            // paths, while the walk builds them from `root`, and on Windows the
            // two spellings differ (verbatim prefix, case) for the same file.
            let same_file = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
                (Ok(left), Ok(right)) => left == right,
                _ => a == b,
            };
            let covered = cargo_seen.iter().any(|seen| same_file(seen, manifest));
            if !covered {
                violations.push(format!(
                    "{name} is a Cargo manifest that neither the root workspace nor \
                     `fuzz/` includes, so no licence gate resolves it"
                ));
            }
            continue;
        }

        checked += 1;
        let text = fs::read_to_string(manifest).map_err(|source| TaskError::Io {
            path: manifest.clone(),
            source,
        })?;
        let document: serde_json::Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(error) => {
                violations.push(format!("{name} is not valid JSON: {error}"));
                continue;
            }
        };
        // Root-level, and a string. `document["license"]` on a non-object
        // yields Null, so a JSON array or scalar manifest also fails here.
        if document.get("license").and_then(serde_json::Value::as_str) != Some(PROJECT_LICENSE) {
            violations.push(format!(
                "{name} has no root-level `\"license\": \"{PROJECT_LICENSE}\"` (a nested \
                 property does not count)"
            ));
        }
    }

    if violations.is_empty() {
        println!("verify-licenses: {checked} manifest(s) and licence text(s) verified");
        Ok(())
    } else {
        Err(TaskError::Policy(format!(
            "SEC-005's licence inventory requires every manifest to declare the project \
             licence. Offending files:\n  {}",
            violations.join("\n  ")
        )))
    }
}

/// What one Cargo graph reports: each package's `(name, licence)`, and the
/// manifest paths those packages were read from.
type CargoLicences = (Vec<(String, Option<String>)>, Vec<PathBuf>);

/// Licences and manifest paths of the packages in one Cargo graph, from
/// `cargo metadata`.
///
/// `--no-deps` keeps this to first-party packages; third-party licences are
/// `cargo deny`'s job and are governed by `deny.toml`'s allow-list. `--locked`
/// so this check cannot be the thing that repairs a stale lockfile.
fn cargo_package_licenses(root: &Path, manifest: &str) -> Result<CargoLicences, TaskError> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            manifest,
        ])
        .current_dir(root)
        .output()
        .map_err(|source| TaskError::Launch {
            program: "cargo".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(TaskError::Policy(format!(
            "cannot read package licences from {manifest}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        TaskError::Policy(format!(
            "cargo metadata for {manifest} was not JSON: {error}"
        ))
    })?;
    let packages = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            TaskError::Policy(format!("cargo metadata for {manifest} listed no packages"))
        })?;
    let mut licenses = Vec::new();
    let mut paths = Vec::new();
    for package in packages {
        let name = package
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<unnamed>")
            .to_owned();
        let license = package
            .get("license")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        licenses.push((name, license));
        if let Some(path) = package
            .get("manifest_path")
            .and_then(serde_json::Value::as_str)
        {
            paths.push(PathBuf::from(path));
        }
    }
    Ok((licenses, paths))
}

/// Collect every `Cargo.toml` and `package.json` under `directory`, skipping
/// build output and vendored trees.
fn manifest_files_under(directory: &Path, found: &mut Vec<PathBuf>) -> Result<(), TaskError> {
    let entries = fs::read_dir(directory).map_err(|source| TaskError::Io {
        path: directory.to_owned(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.to_owned(),
                source,
            })?
            .path();
        let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
        if path.is_dir() {
            // Build output, dependency trees, and fuzzer data contain
            // third-party manifests this policy does not govern.
            //
            // `generated` is deliberately absent from this list. The follow-up
            // audit pointed out that skipping it by name would hide a future
            // first-party package that happened to be called that;
            // `tests/generated/` holds disk images and no manifest, so looking
            // costs nothing.
            if matches!(
                name,
                ".git" | "target" | "node_modules" | "corpus" | "artifacts" | "coverage"
            ) {
                continue;
            }
            manifest_files_under(&path, found)?;
        } else if name == "Cargo.toml" || name == "package.json" {
            found.push(path);
        }
    }
    Ok(())
}

/// Does this comment name a release, rather than merely say something?
///
/// A tag is what makes a digest auditable: without it nobody can tell which
/// release a SHA corresponds to, and reviewing a bump means resolving 40 hex
/// characters by hand. The rule is deliberately loose about form — `v7.0.1`,
/// `7.0.1` and `v4` are all real GitHub Action tags — and strict about
/// substance: some token must look like a version.
fn names_a_release(comment: &str) -> bool {
    comment.split_whitespace().any(|token| {
        let token = token.trim_start_matches('v');
        !token.is_empty()
            && token.starts_with(|character: char| character.is_ascii_digit())
            && token
                .chars()
                .all(|character| character.is_ascii_digit() || character == '.')
    })
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

/// The marker that opens a requirement annotation.
///
/// Deliberately not a `#[doc]` attribute or a macro. Test names here are prose
/// by convention, so an ID cannot live in the name, and a registration macro
/// would have to be added to every crate to record something a comment already
/// says. The cost of choosing a comment is that a *text* scan reads it, and
/// this repository has watched three text scanners be defeated — so nothing
/// rests on the scan alone. See [`verify_traceability`].
const ANNOTATION_MARKER: &str = "// Requirements:";

/// One requirement annotation, bound to the test that follows it.
#[derive(Debug, PartialEq, Eq)]
struct Annotation {
    /// Repository-relative source file.
    file: String,
    /// 1-indexed line of the marker, so a refusal can be navigated to.
    line: usize,
    /// The requirement IDs or stable section references claimed, in order.
    requirements: Vec<String>,
    /// What this evidence establishes. Becomes the evidence-table cell, which
    /// is why it lives beside the code rather than in the document: there is
    /// nowhere else for it to drift to.
    claim: String,
    /// The test function this annotation binds to.
    test: String,
    /// Whether a `#[cfg(...)]` sits between the annotation and its function,
    /// which means the test may legitimately be absent from this platform.
    platform_gated: bool,
    /// The work package this evidence belongs to, where the file is shared and
    /// ownership cannot answer on its own.
    declared_package: Option<String>,
    /// The test this annotation says it documents, checked against the one it
    /// positionally binds to so the two can disagree.
    declared_evidence: Option<String>,
}

/// The fence containing structured evidence that cannot live beside one test.
///
/// Requirement relationships are necessarily authored somewhere. This block is
/// source metadata, not a hand-written output table: the parser validates every
/// requirement against the specification, every path against git and package
/// ownership, every test against libtest, and every command against xtask's
/// parser before the generator is allowed to render it.
const TRACEABILITY_EVIDENCE_BLOCK: &str = "```traceability-evidence";

/// One typed reference inside a structured evidence declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DeclaredEvidenceItem {
    /// A tracked repository file owned by the declaring package.
    Path(String),
    /// A real xtask invocation accepted by the same parser users reach.
    Command(String),
    /// A live test leaf name reported by libtest.
    Test(String),
}

impl DeclaredEvidenceItem {
    /// The value rendered in the generated evidence cell.
    fn value(&self) -> &str {
        match self {
            Self::Path(value) | Self::Command(value) | Self::Test(value) => value,
        }
    }
}

/// One structured evidence relationship from a work-package document.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DeclaredEvidence {
    /// The package authoring and owning this relationship.
    package: String,
    /// Repository-relative work-package document.
    file: String,
    /// 1-indexed line of the opening fence.
    line: usize,
    /// Specification IDs or stable section references the evidence supports.
    requirements: Vec<String>,
    /// Typed artifacts, commands, and tests supporting the claim.
    evidence: Vec<DeclaredEvidenceItem>,
    /// What the evidence establishes.
    claim: String,
}

/// Check that every requirement annotation names a requirement the
/// specification defines and a test that actually exists.
///
/// **The generator must not own the vocabulary it validates against.** That is
/// the same rule the canonical vectors follow — neither language may hold its
/// own copy — and the same one the token audit broke when its thresholds lived
/// inside the file it audited. So the ID set is parsed out of
/// `AGENT_BUILD_SPEC.md` at its definition sites, and an ID this repository
/// invents is a refusal rather than a new requirement.
///
/// **A text scan is not trusted on its own.** Annotations are read from source
/// text, but the function each one binds to is checked against the list libtest
/// reports from the compiled binaries, so an annotation sitting above something
/// that is not a live test is refused.
///
/// **The annotation also names the test it documents, and that redundancy is
/// the mechanism rather than an oversight.** Binding is positional — the
/// annotation attaches to the next function below it — so on its own it cannot
/// notice that the test it was written for has been renamed or deleted. The
/// first version of this check had no name to compare against and stayed green
/// through a rename; measured, not reasoned. Writing the name out gives the two
/// sources something to disagree about, so a rename, a deletion that slides an
/// annotation onto its neighbour, and an annotation adrift above a helper are
/// all refused.
///
/// It landed before the rollout rather than after: one extra line is cheap at
/// twelve annotations and expensive at two hundred, and the rollout is exactly
/// when a mis-bound annotation becomes likely.
///
/// **What no version of this can check.** Nothing here establishes that a test
/// *exercises* the requirement it claims. That stays a review obligation, and
/// the honest version of it is printed with the summary rather than left for a
/// reader to infer.
///
/// # Errors
///
/// Returns [`TaskError`] when the specification yields no stable references,
/// when no annotations exist at all, when an annotation names an unknown
/// reference or test, when structured evidence is stale or malformed, or when
/// two tests share a leaf name so binding would be ambiguous.
fn verify_traceability(root: &Path, write: bool) -> Result<(), TaskError> {
    let vocabulary = spec_traceability_references(root)?;
    let annotations = collect_annotations(root)?;
    let tests = listed_test_names(root)?;
    judge_annotations(&vocabulary, &annotations, &tests)?;
    let declared = collect_declared_evidence(root, &vocabulary, &tests)?;
    let covered: BTreeSet<&str> = annotations
        .iter()
        .flat_map(|annotation| annotation.requirements.iter().map(String::as_str))
        .chain(
            declared
                .iter()
                .flat_map(|evidence| evidence.requirements.iter().map(String::as_str)),
        )
        .collect();

    if write {
        return write_generated_traceability(root, &annotations, &declared);
    }
    verify_generated_traceability(root, &annotations, &declared)?;

    println!(
        "traceability: {} annotation(s) and {} structured evidence row(s) over {} requirement(s), \
         checked against {} spec reference(s) and {} live tests",
        annotations.len(),
        declared.len(),
        covered.len(),
        vocabulary.len(),
        tests.len()
    );
    println!(
        "  not checked, and not checkable here: whether the evidence logically establishes the \
         requirement it claims. That is a review obligation"
    );
    Ok(())
}

/// The judgement itself, over data rather than over the filesystem.
///
/// Separated from [`verify_traceability`] for the same reason [`parse`] is
/// separated from [`execute`]: a check nobody can drive with synthetic input is
/// a check nobody proves can fail, and Section 12 requires every check to be
/// shown capable of failing.
///
/// Returns how many distinct requirements carry evidence.
///
/// # Errors
///
/// Returns [`TaskError`] on an empty annotation set, an unknown requirement ID,
/// a missing claim, or a test no binary reports.
fn judge_annotations(
    vocabulary: &BTreeSet<String>,
    annotations: &[Annotation],
    tests: &BTreeSet<String>,
) -> Result<usize, TaskError> {
    // Vacuity, per Section 12. An empty run must fail, never report a pass over
    // nothing — the same rule that makes an empty destructive target list a
    // refusal rather than a vacuous success.
    if annotations.is_empty() {
        return Err(TaskError::Policy(format!(
            "no requirement annotations found. Every behaviour change needs requirement-ID \
             traceability, so finding none is a failure rather than a clean run. Annotate a \
             test with `{ANNOTATION_MARKER} <ID>` above it"
        )));
    }

    let mut problems = Vec::new();
    for annotation in annotations {
        let where_ = format!("{}:{}", annotation.file, annotation.line);
        if annotation.requirements.is_empty() {
            problems.push(format!("{where_}: annotation names no requirement"));
        }
        for requirement in &annotation.requirements {
            if !vocabulary.contains(requirement) {
                problems.push(format!(
                    "{where_}: `{requirement}` is not defined in AGENT_BUILD_SPEC.md. An \
                     annotation cannot invent a requirement"
                ));
            }
        }
        if annotation.claim.is_empty() {
            problems.push(format!(
                "{where_}: annotation has no claim line. Write what the evidence establishes on \
                 the following `//   ` line; it is the traceability table's cell"
            ));
        }
        if annotation.test.is_empty() {
            problems.push(format!(
                "{where_}: annotation is not followed by a function, so it binds to nothing"
            ));
        } else if annotation.declared_evidence.as_deref() != Some(annotation.test.as_str()) {
            // The two sources disagreeing is the whole point of writing the
            // name out. Either the test was renamed and the annotation was not,
            // or an annotated test was deleted and this annotation has slid
            // onto its neighbour.
            problems.push(match &annotation.declared_evidence {
                Some(declared) => format!(
                    "{where_}: says it documents `{declared}` but sits above `{}`. An annotation \
                     that has slid onto another test credits it with someone else's requirement",
                    annotation.test
                ),
                None => format!(
                    "{where_}: does not say which test it documents. Add `// Evidence: {}` — \
                     binding is positional, so without the name a deleted test silently hands \
                     its annotation to the next one",
                    annotation.test
                ),
            });
        } else if !tests.contains(&annotation.test) && !annotation.platform_gated {
            problems.push(format!(
                "{where_}: binds to `{}`, which no test binary reports. Either it is not a test \
                 at all, or the annotation has drifted away from the one it documents",
                annotation.test
            ));
        }
    }

    if !problems.is_empty() {
        return Err(TaskError::Policy(format!(
            "traceability annotations are not usable as evidence:\n  {}",
            problems.join("\n  ")
        )));
    }

    let covered: BTreeSet<&str> = annotations
        .iter()
        .flat_map(|annotation| annotation.requirements.iter().map(String::as_str))
        .collect();
    Ok(covered.len())
}

/// The first line of a generated traceability document.
///
/// Whole-file generation rather than a managed region inside a hand-written
/// file: the gate is then plain byte equality, which is obviously correct in a
/// way that "extract the region, regenerate it, compare, and also check nobody
/// added a row outside it" is not.
const GENERATED_MARKER: &str =
    "<!-- Generated by `cargo xtask traceability`. Edit evidence sources, not this file. -->";

/// The block in a work-package document that declares how its traceability is
/// produced: a fenced ```` ```traceability ```` block containing `mode:
/// generated` or `mode: hand-maintained`.
///
/// **Each package declares its own, and that placement is load-bearing.** The
/// first version of this was a constant in `tools/xtask`, which WP-000, WP-020
/// and WP-030 all own — and WP-010 does not. Under that arrangement WP-010
/// could never convert itself without editing a file outside its assignment,
/// so the rollout was blocked by the very ownership rule it was meant to
/// respect. Found by working out what increment 3 would have to do, not by
/// review.
///
/// The declaration is checked **in both directions**: a package declaring
/// `hand-maintained` whose document carries the generated marker is refused, so
/// a stale exemption cannot outlive the conversion; and a package declaring
/// `generated` is held to byte equality with what its annotations and typed
/// evidence declarations produce. A package declaring nothing is refused
/// outright, so a new work package cannot be born hand-maintained by omission.
const TRACEABILITY_BLOCK: &str = "```traceability";

/// How a package's traceability document is produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceabilityMode {
    /// Rendered from validated evidence sources; hand edits fail.
    Generated,
    /// Still written by hand, and still drifting. An honest interim state.
    HandMaintained,
}

/// The `mode:` line inside a document's fenced `traceability` block.
///
/// **A line-structural read, not a substring search, and the difference was
/// measured.** The first version did `text.split_once("```traceability")`, and
/// WP-000's own document defeated it within the hour: the prose explaining the
/// block mentions its name, that mention appears before the real block, and the
/// parser read the sentence instead of the data. That is the same defect the
/// action scanner was rewritten three times to escape — a document is allowed
/// to talk about its own syntax, and `prose_and_comments_are_not_references`
/// exists next door for exactly this reason.
///
/// So the fence must be a line of its own, and the block ends at the next line
/// that is a bare fence.
fn declared_traceability_mode(text: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside {
            inside = trimmed == TRACEABILITY_BLOCK;
            continue;
        }
        if trimmed == "```" {
            return None;
        }
        if let Some(mode) = trimmed.strip_prefix("mode:") {
            return Some(mode.trim().to_owned());
        }
    }
    None
}

/// Read each package's declared traceability mode from its own document.
fn traceability_modes(root: &Path) -> Result<BTreeMap<String, TraceabilityMode>, TaskError> {
    let directory = root.join("docs/work-packages");
    let entries = fs::read_dir(&directory).map_err(|source| TaskError::Io {
        path: directory.clone(),
        source,
    })?;

    let mut modes = BTreeMap::new();
    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        let Some(package) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !package.starts_with("WP-") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|source| TaskError::Io {
            path: path.clone(),
            source,
        })?;
        let declared = declared_traceability_mode(&text);
        if declared.is_none() && !text.lines().any(|line| line.trim() == TRACEABILITY_BLOCK) {
            return Err(TaskError::Policy(format!(
                "{package} declares no traceability mode. Add a fenced `traceability` block \
                 saying `mode: generated` or `mode: hand-maintained`; a package cannot be born \
                 hand-maintained by omission"
            )));
        }
        let declared = declared.as_deref();
        let mode = match declared {
            Some("generated") => TraceabilityMode::Generated,
            Some("hand-maintained") => TraceabilityMode::HandMaintained,
            other => {
                return Err(TaskError::Policy(format!(
                    "{package}: traceability mode {other:?} is not understood; expected \
                     `generated` or `hand-maintained`"
                )));
            }
        };
        modes.insert(package.to_owned(), mode);
    }
    Ok(modes)
}

/// Extract one exact fenced block without mistaking prose for its opening.
fn exact_fenced_block(text: &str, marker: &str) -> Result<Option<(usize, String)>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = None;
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim() != marker {
            index += 1;
            continue;
        }
        if found.is_some() {
            return Err(format!("declares more than one {marker} block"));
        }

        let opening_line = index + 1;
        index += 1;
        let body_start = index;
        while index < lines.len() && lines[index].trim() != "```" {
            index += 1;
        }
        if index == lines.len() {
            return Err(format!(
                "opens a {marker} block at line {opening_line} but never closes it"
            ));
        }
        found = Some((opening_line, lines[body_start..index].join("\n")));
        index += 1;
    }
    Ok(found)
}

/// Parse and validate every structured evidence row declared by a package.
fn collect_declared_evidence(
    root: &Path,
    vocabulary: &BTreeSet<String>,
    tests: &BTreeSet<String>,
) -> Result<Vec<DeclaredEvidence>, TaskError> {
    let directory = root.join("docs/work-packages");
    let entries = fs::read_dir(&directory).map_err(|source| TaskError::Io {
        path: directory.clone(),
        source,
    })?;
    let tracked: BTreeSet<String> = tracked_files(root)?.into_iter().collect();
    let claims = ownership_claims(root)?;
    let mut declared = Vec::new();
    let mut problems = Vec::new();

    for entry in entries {
        let path = entry
            .map_err(|source| TaskError::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        let Some(package) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !package.starts_with("WP-") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|source| TaskError::Io {
            path: path.clone(),
            source,
        })?;
        let block = exact_fenced_block(&text, TRACEABILITY_EVIDENCE_BLOCK)
            .map_err(|problem| TaskError::Policy(format!("{package}: {problem}")))?;
        let Some((line, body)) = block else {
            continue;
        };
        let file = format!("docs/work-packages/{package}.md");
        let rows = parse_declared_evidence(package, &file, line, &body)
            .map_err(|problem| TaskError::Policy(format!("{file}:{line}: {problem}")))?;

        for row in rows {
            validate_declared_evidence(&row, vocabulary, tests, &tracked, &claims)
                .unwrap_or_else(|row_problems| problems.extend(row_problems));
            declared.push(row);
        }
    }

    if !problems.is_empty() {
        return Err(TaskError::Policy(format!(
            "structured traceability evidence is not usable:\n  {}",
            problems.join("\n  ")
        )));
    }
    Ok(declared)
}

/// Decode one YAML evidence block.
fn parse_declared_evidence(
    package: &str,
    file: &str,
    line: usize,
    body: &str,
) -> Result<Vec<DeclaredEvidence>, String> {
    use yaml_rust2::{Yaml, YamlLoader};

    let documents =
        YamlLoader::load_from_str(body).map_err(|error| format!("invalid YAML: {error}"))?;
    if documents.len() != 1 {
        return Err(format!(
            "expected one YAML document, found {}",
            documents.len()
        ));
    }
    let Some(rows) = documents[0].as_vec() else {
        return Err("the block must be a YAML sequence of evidence rows".to_owned());
    };
    if rows.is_empty() {
        return Err("the block contains no evidence rows".to_owned());
    }

    let mut parsed = Vec::new();
    let mut unique_rows = BTreeSet::new();
    for (offset, row) in rows.iter().enumerate() {
        let number = offset + 1;
        let Some(fields) = row.as_hash() else {
            return Err(format!("row {number} must be a mapping"));
        };
        for key in fields.keys() {
            let Some(key) = key.as_str() else {
                return Err(format!("row {number} has a non-string field name"));
            };
            if !matches!(key, "requirements" | "evidence" | "claim") {
                return Err(format!(
                    "row {number} has unknown field {key:?}; expected requirements, evidence, claim"
                ));
            }
        }
        let field = |name: &str| fields.get(&Yaml::String(name.to_owned()));

        let requirements = string_sequence(field("requirements"), number, "requirements")?;
        if requirements.iter().collect::<BTreeSet<_>>().len() != requirements.len() {
            return Err(format!("row {number} repeats a requirement"));
        }
        let evidence = parse_declared_evidence_items(field("evidence"), number)?;

        let Some(claim) = field("claim")
            .and_then(Yaml::as_str)
            .map(str::trim)
            .filter(|claim| !claim.is_empty())
        else {
            return Err(format!("row {number} field `claim` must be a string"));
        };
        let row = DeclaredEvidence {
            package: package.to_owned(),
            file: file.to_owned(),
            line,
            requirements,
            evidence,
            claim: claim.to_owned(),
        };
        if !unique_rows.insert(row.clone()) {
            return Err(format!("row {number} duplicates an earlier evidence row"));
        }
        parsed.push(row);
    }
    Ok(parsed)
}

/// Decode the typed items in one structured evidence row.
fn parse_declared_evidence_items(
    value: Option<&yaml_rust2::Yaml>,
    row: usize,
) -> Result<Vec<DeclaredEvidenceItem>, String> {
    let Some(items) = value.and_then(yaml_rust2::Yaml::as_vec) else {
        return Err(format!(
            "row {row} field `evidence` must be a non-empty sequence"
        ));
    };
    if items.is_empty() {
        return Err(format!("row {row} field `evidence` is empty"));
    }

    let mut evidence = Vec::new();
    let mut unique = BTreeSet::new();
    for item in items {
        let Some(mapping) = item.as_hash() else {
            return Err(format!(
                "row {row} evidence item must be a one-field mapping"
            ));
        };
        if mapping.len() != 1 {
            return Err(format!(
                "row {row} evidence item must name exactly one of path, command, or test"
            ));
        }
        let (kind, value) = mapping.iter().next().expect("one entry was checked");
        let Some(kind) = kind.as_str() else {
            return Err(format!("row {row} evidence item has a non-string kind"));
        };
        let Some(value) = value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Err(format!(
                "row {row} evidence item {kind:?} has no string value"
            ));
        };
        let item = match kind {
            "path" => DeclaredEvidenceItem::Path(value.to_owned()),
            "command" => DeclaredEvidenceItem::Command(value.to_owned()),
            "test" => DeclaredEvidenceItem::Test(value.to_owned()),
            _ => {
                return Err(format!(
                    "row {row} evidence kind {kind:?} is not understood; expected path, command, \
                     or test"
                ));
            }
        };
        if !unique.insert(format!("{kind}\0{value}")) {
            return Err(format!(
                "row {row} repeats the same {kind} evidence {value:?}"
            ));
        }
        evidence.push(item);
    }
    Ok(evidence)
}

/// Read a required YAML sequence of non-empty strings.
fn string_sequence(
    value: Option<&yaml_rust2::Yaml>,
    row: usize,
    field: &str,
) -> Result<Vec<String>, String> {
    let Some(items) = value.and_then(yaml_rust2::Yaml::as_vec) else {
        return Err(format!(
            "row {row} field `{field}` must be a non-empty sequence"
        ));
    };
    if items.is_empty() {
        return Err(format!("row {row} field `{field}` is empty"));
    }
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("row {row} field `{field}` contains a non-string value"))
        })
        .collect()
}

/// Validate a structured relationship against sources outside its own block.
fn validate_declared_evidence(
    row: &DeclaredEvidence,
    vocabulary: &BTreeSet<String>,
    tests: &BTreeSet<String>,
    tracked: &BTreeSet<String>,
    claims: &BTreeMap<String, Vec<OwnershipClaim>>,
) -> Result<(), Vec<String>> {
    let at = format!("{}:{}", row.file, row.line);
    let mut problems = Vec::new();

    for requirement in &row.requirements {
        if !vocabulary.contains(requirement) {
            problems.push(format!(
                "{at}: `{requirement}` is not a requirement ID or stable section reference \
                 defined by AGENT_BUILD_SPEC.md"
            ));
        }
    }
    for item in &row.evidence {
        match item {
            DeclaredEvidenceItem::Path(path) => {
                if path.contains('\\')
                    || Path::new(path).is_absolute()
                    || Path::new(path)
                        .components()
                        .any(|component| !matches!(component, std::path::Component::Normal(_)))
                {
                    problems.push(format!(
                        "{at}: evidence path {path:?} must be a normalized repository-relative file"
                    ));
                    continue;
                }
                if !tracked.contains(path) {
                    problems.push(format!(
                        "{at}: evidence path {path:?} is not a tracked file"
                    ));
                    continue;
                }
                let owners = owning_packages(claims, path);
                if !owners.iter().any(|owner| owner == &row.package) {
                    problems.push(format!(
                        "{at}: {} declares evidence path {path:?} but does not own it; owners: {}",
                        row.package,
                        if owners.is_empty() {
                            "none".to_owned()
                        } else {
                            owners.join(", ")
                        }
                    ));
                }
            }
            DeclaredEvidenceItem::Command(command) => {
                let Some(arguments) = command.strip_prefix("cargo xtask ") else {
                    problems.push(format!(
                        "{at}: command evidence {command:?} is not an xtask invocation; only \
                         commands whose parser this gate can validate are accepted"
                    ));
                    continue;
                };
                let arguments: Vec<OsString> = arguments
                    .split_ascii_whitespace()
                    .map(OsString::from)
                    .collect();
                if arguments.is_empty() || parse(&arguments).is_err() {
                    problems.push(format!(
                        "{at}: command evidence {command:?} is not accepted by xtask's parser"
                    ));
                }
            }
            DeclaredEvidenceItem::Test(test) => {
                if !tests.contains(test) {
                    problems.push(format!(
                        "{at}: test evidence `{test}` is not reported by any compiled test binary"
                    ));
                }
            }
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Which work package an annotation's evidence belongs to.
///
/// Answered from the `owned-paths` blocks rather than from anything the
/// annotation says, so evidence lands in the document of the package that
/// actually owns the code. Where a path is shared — `tools/xtask/**` is claimed
/// by three packages — no inference is possible and the annotation must say,
/// which is then checked against ownership rather than believed.
fn owning_packages(claims: &BTreeMap<String, Vec<OwnershipClaim>>, file: &str) -> Vec<String> {
    claims
        .iter()
        .filter(|(_, patterns)| {
            patterns
                .iter()
                .any(|claim| claim.kind == ClaimKind::Owned && claim_matches(&claim.pattern, file))
        })
        .map(|(package, _)| package.clone())
        .collect()
}

/// Decide which package's document an annotation's evidence belongs in.
///
/// A declaration is checked against ownership rather than trusted: a package
/// cannot claim evidence sitting in a file it does not own, which is the same
/// rule `verify-change-ownership` applies to a diff.
fn route_annotation(
    claims: &BTreeMap<String, Vec<OwnershipClaim>>,
    annotation: &Annotation,
) -> Result<String, String> {
    let owners = owning_packages(claims, &annotation.file);
    let at = format!("{}:{}", annotation.file, annotation.line);

    if let Some(declared) = &annotation.declared_package {
        return if owners.iter().any(|owner| owner == declared) {
            Ok(declared.clone())
        } else if owners.is_empty() {
            Err(format!(
                "{at}: declares {declared}, but no package owns this file"
            ))
        } else {
            Err(format!(
                "{at}: declares {declared}, which does not own this file. Owners: {}",
                owners.join(", ")
            ))
        };
    }

    match owners.len() {
        0 => Err(format!(
            "{at}: no work package owns this file, so its evidence has nowhere to go"
        )),
        1 => Ok(owners[0].clone()),
        _ => Err(format!(
            "{at}: {} all own this path, so which package's evidence this is cannot be \
             inferred. Add `// Work-Package: WP-0NN` to the annotation",
            owners.join(", ")
        )),
    }
}

/// Route every annotation, refusing the whole set if any one is ambiguous.
fn annotations_by_package<'a>(
    claims: &BTreeMap<String, Vec<OwnershipClaim>>,
    annotations: &'a [Annotation],
) -> Result<BTreeMap<String, Vec<&'a Annotation>>, TaskError> {
    let mut by_package = BTreeMap::new();
    let mut problems = Vec::new();
    for annotation in annotations {
        match route_annotation(claims, annotation) {
            Ok(package) => by_package
                .entry(package)
                .or_insert_with(Vec::new)
                .push(annotation),
            Err(problem) => problems.push(problem),
        }
    }
    if !problems.is_empty() {
        return Err(TaskError::Policy(format!(
            "annotations cannot be routed to a work package:\n  {}",
            problems.join("\n  ")
        )));
    }
    Ok(by_package)
}

/// Escape content before placing it inside a Markdown table cell.
fn markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

/// Render one package's traceability document.
fn render_traceability(
    package: &str,
    annotations: &[&Annotation],
    declared: &[&DeclaredEvidence],
) -> String {
    use std::fmt::Write as _;

    let mut requirements: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for annotation in annotations {
        for requirement in &annotation.requirements {
            requirements
                .entry(requirement.clone())
                .or_default()
                .insert((format!("`{}`", annotation.test), annotation.claim.clone()));
        }
    }
    for row in declared {
        let evidence = row
            .evidence
            .iter()
            .map(|item| format!("`{}`", item.value()))
            .collect::<Vec<_>>()
            .join(", ");
        for requirement in &row.requirements {
            requirements
                .entry(requirement.clone())
                .or_default()
                .insert((evidence.clone(), row.claim.clone()));
        }
    }

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str("\n\n# ");
    out.push_str(package);
    out.push_str(" traceability\n\n");
    out.push_str(
        "Every row below is generated from either a requirement annotation beside a test or a\n\
         typed evidence declaration in the package's work-package document. The generator checks\n\
         each specification reference and checks that every test, tracked path, and xtask command\n\
         exists and belongs here. Narrative — what a package has *not* established, and why —\n\
         remains in its work-package document.\n\n",
    );
    out.push_str("| Requirement | Evidence | What it establishes |\n");
    out.push_str("| --- | --- | --- |\n");
    for (requirement, evidence) in &requirements {
        for (items, claim) in evidence {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                markdown_table_cell(requirement),
                markdown_table_cell(items),
                markdown_table_cell(claim)
            );
        }
    }

    let _ = writeln!(
        out,
        "\n{} requirement(s), {} annotation(s), {} structured evidence row(s).",
        requirements.len(),
        annotations.len(),
        declared.len()
    );
    out.push_str(
        "\n**Not established here:** that the evidence logically proves the requirement it claims.\n\
         The generator checks stable references, existence, command syntax, and ownership; the\n\
         semantic relationship remains a review obligation.\n",
    );
    out
}

/// Regenerate every generated traceability document and refuse any difference.
///
/// # Errors
///
/// Returns [`TaskError`] when a generated document differs from what the
/// validated evidence sources produce, when a package's hand-maintained status
/// disagrees with its declared mode, or when evidence cannot be routed.
fn verify_generated_traceability(
    root: &Path,
    annotations: &[Annotation],
    declared: &[DeclaredEvidence],
) -> Result<(), TaskError> {
    let claims = ownership_claims(root)?;
    let by_package = annotations_by_package(&claims, annotations)?;
    let mut declared_by_package: BTreeMap<String, Vec<&DeclaredEvidence>> = BTreeMap::new();
    for row in declared {
        declared_by_package
            .entry(row.package.clone())
            .or_default()
            .push(row);
    }

    let modes = traceability_modes(root)?;
    let directory = root.join("docs/traceability");
    let mut drifted = Vec::new();

    for package in claims.keys() {
        let path = directory.join(format!("{package}.md"));
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let is_generated = existing.starts_with(GENERATED_MARKER);
        let declared_hand = modes.get(package.as_str()) == Some(&TraceabilityMode::HandMaintained);

        if declared_hand && is_generated {
            drifted.push(format!(
                "{package} declares `mode: hand-maintained` but its document is generated. \
                 Update the declaration in docs/work-packages/{package}.md; a stale exemption is \
                 one nobody revisits"
            ));
            continue;
        }
        if !declared_hand && !is_generated {
            drifted.push(format!(
                "{package}'s traceability is neither generated nor declared hand-maintained. A \
                 new work package cannot be born hand-maintained by omission"
            ));
            continue;
        }
        if declared_hand {
            continue;
        }

        let expected = render_traceability(
            package,
            by_package.get(package).unwrap_or(&Vec::new()),
            declared_by_package.get(package).unwrap_or(&Vec::new()),
        );
        if existing != expected {
            drifted.push(format!(
                "{} is out of date with its evidence sources. Run `cargo xtask traceability \
                 --write`; do not hand-edit a generated file",
                path.display()
            ));
        }
    }

    if !drifted.is_empty() {
        return Err(TaskError::Policy(format!(
            "generated traceability has drifted:\n  {}",
            drifted.join("\n  ")
        )));
    }

    let still_by_hand: Vec<&str> = modes
        .iter()
        .filter(|(_, mode)| **mode == TraceabilityMode::HandMaintained)
        .map(|(package, _)| package.as_str())
        .collect();
    println!(
        "traceability: {} generated document(s), {} still hand-maintained{}",
        modes.len() - still_by_hand.len(),
        still_by_hand.len(),
        if still_by_hand.is_empty() {
            String::new()
        } else {
            format!(" ({})", still_by_hand.join(", "))
        }
    );
    Ok(())
}

/// Write every generated traceability document.
///
/// # Errors
///
/// Returns [`TaskError`] when a document cannot be written or evidence cannot
/// be routed to a package.
fn write_generated_traceability(
    root: &Path,
    annotations: &[Annotation],
    declared: &[DeclaredEvidence],
) -> Result<(), TaskError> {
    let claims = ownership_claims(root)?;
    let modes = traceability_modes(root)?;
    let by_package = annotations_by_package(&claims, annotations)?;
    let mut declared_by_package: BTreeMap<String, Vec<&DeclaredEvidence>> = BTreeMap::new();
    for row in declared {
        declared_by_package
            .entry(row.package.clone())
            .or_default()
            .push(row);
    }

    for package in claims.keys() {
        if modes.get(package.as_str()) != Some(&TraceabilityMode::Generated) {
            continue;
        }
        let path = root.join("docs/traceability").join(format!("{package}.md"));
        let rendered = render_traceability(
            package,
            by_package.get(package).unwrap_or(&Vec::new()),
            declared_by_package.get(package).unwrap_or(&Vec::new()),
        );
        fs::write(&path, rendered).map_err(|source| TaskError::Io {
            path: path.clone(),
            source,
        })?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// Every stable traceability reference the specification defines.
///
/// This includes requirement IDs at their definition sites, numeric Markdown
/// headings, and the numbered operating-contract items under Section 1. The
/// latter is deliberately narrow: the specification itself refers to
/// `Section 1.10`, although item 10 is a list item rather than a heading. Merely
/// accepting `Section 1.N` because Section 1 exists would let an invented item
/// pass, so each accepted item must be present in the source document.
fn spec_traceability_references(root: &Path) -> Result<BTreeSet<String>, TaskError> {
    let path = root.join("AGENT_BUILD_SPEC.md");
    let text = fs::read_to_string(&path).map_err(|error| {
        TaskError::Policy(format!(
            "cannot read the specification at {}: {error}",
            path.display()
        ))
    })?;
    let references = spec_traceability_references_from(&text);

    if references.is_empty() {
        return Err(TaskError::Policy(format!(
            "no traceability references found in {}. The vocabulary must come from the \
             specification, so an empty parse is a failure rather than an empty allow-list",
            path.display()
        )));
    }
    Ok(references)
}

/// Parse stable references from specification text.
fn spec_traceability_references_from(text: &str) -> BTreeSet<String> {
    let mut references = BTreeSet::new();
    let mut top_level_section = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        let section = numeric_section_heading(trimmed);
        if trimmed.starts_with("## ") {
            top_level_section.clone_from(&section);
        }
        if let Some(section) = section {
            references.insert(format!("Section {section}"));
        }

        // Section 1 is an explicitly numbered operating contract. The spec and
        // repository documents cite items such as Section 1.10 and 1.11, so
        // recognize only list items that actually exist there.
        if top_level_section.as_deref() == Some("1")
            && line == trimmed
            && let Some((number, _)) = trimmed.split_once(". ")
            && !number.is_empty()
            && number.chars().all(|character| character.is_ascii_digit())
        {
            references.insert(format!("Section 1.{number}"));
        }

        if let Some(rest) = trimmed.strip_prefix("### ")
            && let Some(id) = leading_requirement_id(rest)
        {
            references.insert(id);
        }
        // `- **ID:** …`, the form most requirements use.
        if let Some(rest) = trimmed
            .strip_prefix("- **")
            .or_else(|| trimmed.strip_prefix("**"))
            && let Some(id) = leading_requirement_id(rest)
        {
            references.insert(id);
        }
    }
    references
}

/// `### 11.7 Traceability` -> `11.7`; only numeric heading tokens qualify.
fn numeric_section_heading(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("### ")
        .or_else(|| line.strip_prefix("## "))?;
    let candidate = rest.split_ascii_whitespace().next()?.trim_end_matches('.');
    if candidate.is_empty()
        || candidate.starts_with('.')
        || candidate.ends_with('.')
        || candidate
            .split('.')
            .any(|part| part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    Some(candidate.to_owned())
}

/// `SAFE-007: …` -> `SAFE-007`, and `Some` only when the text opens with an ID
/// followed immediately by a colon.
fn leading_requirement_id(text: &str) -> Option<String> {
    let (candidate, _) = text.split_once(':')?;
    let candidate = candidate.trim_end_matches('*').trim();
    let (prefix, number) = candidate.split_once('-')?;
    let prefix_ok = (2..=5).contains(&prefix.len())
        && prefix
            .chars()
            .all(|character| character.is_ascii_uppercase());
    let number_ok = number.len() == 3 && number.chars().all(|c| c.is_ascii_digit());
    (prefix_ok && number_ok).then(|| candidate.to_owned())
}

/// Read every annotation out of the repository's tracked Rust sources.
fn collect_annotations(root: &Path) -> Result<Vec<Annotation>, TaskError> {
    let mut found = Vec::new();
    for file in tracked_files(root)? {
        if Path::new(&file).extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let text = fs::read_to_string(root.join(&file))
            .map_err(|error| TaskError::Policy(format!("cannot read {file}: {error}")))?;
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix(ANNOTATION_MARKER) else {
                continue;
            };
            found.push(parse_annotation(&file, index, rest, &lines));
        }
    }
    Ok(found)
}

/// Build one annotation from its marker line and what follows it.
fn parse_annotation(file: &str, index: usize, rest: &str, lines: &[&str]) -> Annotation {
    let requirements = rest
        .split(',')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    // Continuation lines are indented under the marker (`//   text`), which
    // distinguishes the claim from an ordinary comment that happens to sit
    // beneath an annotation.
    let mut claim = String::new();
    let mut cursor = index + 1;
    while let Some(line) = lines.get(cursor) {
        let Some(comment) = line.trim_start().strip_prefix("//") else {
            break;
        };
        let Some(text) = comment.strip_prefix("   ") else {
            break;
        };
        if !claim.is_empty() {
            claim.push(' ');
        }
        claim.push_str(text.trim());
        cursor += 1;
    }

    // An optional `// Work-Package: WP-0NN`, needed only where a path is
    // claimed by more than one package and inference is therefore impossible.
    // Deliberately not required everywhere: a declaration that is usually
    // redundant gets copied without thought, and then it is not evidence of
    // anything. Where it does appear it is checked against the `owned-paths`
    // blocks rather than believed.
    let mut declared_package = None;
    if let Some(line) = lines.get(cursor)
        && let Some(rest) = line.trim_start().strip_prefix("// Work-Package:")
    {
        declared_package = Some(rest.trim().to_owned());
        cursor += 1;
    }

    // The evidence the annotation is *for*, written out.
    //
    // Redundant with the function below it, deliberately and load-bearing:
    // redundancy is the only thing that lets two sources disagree. Binding is
    // positional, so without this, deleting an annotated test lets its
    // annotation slide onto the next function and credit that test with
    // another's requirement and claim — silently. Measured on the first version
    // of this check: renaming an annotated test left the gate green, because
    // there was no stored name to contradict.
    //
    // Written before the rollout rather than after it. Adding a line to every
    // annotation is cheap at twelve and expensive at two hundred, and the
    // rollout is exactly when a mis-bound annotation becomes likely.
    let mut declared_evidence = None;
    if let Some(line) = lines.get(cursor)
        && let Some(rest) = line.trim_start().strip_prefix("// Evidence:")
    {
        declared_evidence = Some(rest.trim().to_owned());
        cursor += 1;
    }

    // Then the binding: the next function declared. Attributes, ordinary
    // comments and blank lines may sit between, which is what `#[test]` and
    // `#[cfg(windows)]` need.
    //
    // A `#[cfg(` on the way down is remembered, because a platform-gated test
    // is absent from *this* platform's listing and must not be reported as
    // missing evidence. It is counted as unverified-here instead.
    let mut test = String::new();
    let mut platform_gated = false;
    for line in lines.iter().skip(cursor).take(12) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(") {
            platform_gated = true;
        }
        if let Some(after) = trimmed.split_once("fn ") {
            let name: String = after
                .1
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                test = name;
            }
            break;
        }
    }

    Annotation {
        file: file.to_owned(),
        line: index + 1,
        requirements,
        claim,
        test,
        platform_gated,
        declared_package,
        declared_evidence,
    }
}

/// Every test name the compiled binaries report, as leaf names.
///
/// This is the structural half of the check, and the reason a comment scan is
/// acceptable for the other half: libtest is asked what tests exist rather than
/// the source being asked what it looks like.
///
/// Leaf names because an annotation sits beside a function and knows nothing
/// about its module path. That is only unambiguous while leaf names are unique,
/// so this refuses a duplicate rather than assuming — measured at 216 tests and
/// 216 distinct names when written, which is a fact that could stop being true.
fn listed_test_names(root: &Path) -> Result<BTreeSet<String>, TaskError> {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "--list",
        ])
        .output()
        .map_err(|error| TaskError::Policy(format!("cannot list tests: {error}")))?;
    if !output.status.success() {
        return Err(TaskError::Policy(format!(
            "listing tests failed, so annotations cannot be checked against reality: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let listing = String::from_utf8_lossy(&output.stdout);
    let mut names = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for line in listing.lines() {
        let Some(path) = line.strip_suffix(": test") else {
            continue;
        };
        let leaf = path.rsplit("::").next().unwrap_or(path).trim();
        if !names.insert(leaf.to_owned()) {
            duplicates.insert(leaf.to_owned());
        }
    }

    if !duplicates.is_empty() {
        return Err(TaskError::Policy(format!(
            "two or more tests share a leaf name, so an annotation could not say which one it \
             means: {}. Rename one, or annotations stop being evidence",
            duplicates.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    if names.is_empty() {
        return Err(TaskError::Policy(
            "no tests were listed, so every annotation would be unverifiable. Failing rather \
             than passing over nothing"
                .to_owned(),
        ));
    }
    Ok(names)
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
  cargo xtask tokens             Audit the design tokens for UI-001/007/008
  cargo xtask traceability [--write]
                                 Check annotations and typed evidence against the spec,
                                 live tests, tracked ownership, and xtask's parser, and
                                 check generated documents. `--write` regenerates (11.7)
  cargo xtask verify-actions     Verify every GitHub Action is pinned by digest
  cargo xtask verify-licenses    Verify every manifest declares MIT OR Apache-2.0
  cargo xtask verify-ownership   Verify every tracked path belongs to a work package
  cargo xtask verify-change-ownership --base <rev>
                                 Verify this change belongs to the work package its
                                 commits declare, judged against <rev>
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
        Task, TaskError, claim_matches, derivation_is_plausible, inherits_workspace_lints,
        is_pinned, parse, parse_test, relative_to_root, repository_root, run_tier,
        validate_claim_pattern, validate_derived_pattern, verify_action_pins,
        verify_change_ownership, verify_manifest_licenses, verify_path_ownership,
        verify_workspace_lints, workspace_manifests,
    };
    use std::collections::BTreeSet;
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use yaml_rust2::{Yaml, YamlLoader};

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

    /// A vocabulary, annotations and a test list, for driving
    /// `judge_annotations` without touching the filesystem.
    fn traceability_inputs() -> (
        std::collections::BTreeSet<String>,
        Vec<super::Annotation>,
        std::collections::BTreeSet<String>,
    ) {
        let vocabulary = ["SAFE-007".to_owned(), "SEC-010".to_owned()]
            .into_iter()
            .collect();
        let tests = ["a_real_test".to_owned()].into_iter().collect();
        let annotations = vec![super::Annotation {
            file: "tools/xtask/src/main.rs".to_owned(),
            line: 1,
            requirements: vec!["SAFE-007".to_owned()],
            claim: "what this establishes".to_owned(),
            test: "a_real_test".to_owned(),
            platform_gated: false,
            declared_package: None,
            declared_evidence: Some("a_real_test".to_owned()),
        }];
        (vocabulary, annotations, tests)
    }

    /// Ownership as the routing table: one exclusive path, one shared three
    /// ways, mirroring `tools/xtask/**`.
    fn routing_claims() -> std::collections::BTreeMap<String, Vec<super::OwnershipClaim>> {
        let owned = |pattern: &str| super::OwnershipClaim {
            pattern: pattern.to_owned(),
            kind: super::ClaimKind::Owned,
        };
        let mut claims = std::collections::BTreeMap::new();
        claims.insert(
            "WP-000".to_owned(),
            vec![owned("tools/xtask/**"), owned("deny.toml")],
        );
        claims.insert("WP-020".to_owned(), vec![owned("tools/xtask/**")]);
        claims.insert("WP-030".to_owned(), vec![owned("tools/xtask/**")]);
        claims
    }

    fn annotation_in(file: &str, declared: Option<&str>) -> super::Annotation {
        super::Annotation {
            file: file.to_owned(),
            line: 1,
            requirements: vec!["SEC-010".to_owned()],
            claim: "a claim".to_owned(),
            test: "a_real_test".to_owned(),
            platform_gated: false,
            declared_package: declared.map(ToOwned::to_owned),
            declared_evidence: Some("a_real_test".to_owned()),
        }
    }

    fn declared_evidence_in(package: &str) -> super::DeclaredEvidence {
        super::DeclaredEvidence {
            package: package.to_owned(),
            file: format!("docs/work-packages/{package}.md"),
            line: 3,
            requirements: vec!["Section 1.10".to_owned()],
            evidence: vec![
                super::DeclaredEvidenceItem::Path("deny.toml".to_owned()),
                super::DeclaredEvidenceItem::Command("cargo xtask ci".to_owned()),
                super::DeclaredEvidenceItem::Test("a_real_test".to_owned()),
            ],
            claim: "typed evidence is checked rather than copied".to_owned(),
        }
    }

    #[test]
    fn a_document_may_discuss_the_traceability_block_without_being_misread() {
        // Measured regression. The first version searched for the fence as a
        // substring, and WP-000's own document defeated it: the paragraph
        // explaining the block names it, that mention precedes the real block,
        // and the parser read the sentence. Same shape as the action scanner's
        // three defeats, and `prose_and_comments_are_not_references` is the
        // neighbouring test for the same principle.
        let document = "\
# WP-000

Each package declares its mode in a ```traceability block, and `mode: generated`
means the file is output rather than prose.

## Traceability

```traceability
mode: hand-maintained
```
";
        assert_eq!(
            super::declared_traceability_mode(document).as_deref(),
            Some("hand-maintained"),
            "the real block must win over prose that merely names it"
        );

        // And an empty block reads as undeclared rather than as a default.
        let empty = "## Traceability\n\n```traceability\n```\n";
        assert_eq!(super::declared_traceability_mode(empty), None);
    }

    #[test]
    fn evidence_in_an_exclusively_owned_file_needs_no_declaration() {
        let claims = routing_claims();
        assert_eq!(
            super::route_annotation(&claims, &annotation_in("deny.toml", None)),
            Ok("WP-000".to_owned())
        );
    }

    #[test]
    fn evidence_in_a_shared_file_must_say_which_package_it_belongs_to() {
        // `tools/xtask/**` is genuinely claimed by three packages. Guessing
        // would put one package's evidence in another's document, which is the
        // drift the whole exercise exists to remove — so it refuses instead.
        let claims = routing_claims();
        let refusal =
            super::route_annotation(&claims, &annotation_in("tools/xtask/src/main.rs", None))
                .expect_err("a shared path cannot be routed by inference");
        assert!(refusal.contains("WP-000, WP-020, WP-030"), "{refusal}");

        assert_eq!(
            super::route_annotation(
                &claims,
                &annotation_in("tools/xtask/src/main.rs", Some("WP-020"))
            ),
            Ok("WP-020".to_owned())
        );
    }

    #[test]
    fn a_declaration_cannot_claim_evidence_in_a_file_the_package_does_not_own() {
        // The same rule `verify-change-ownership` applies to a diff: the
        // declaration is checked against the `owned-paths` blocks rather than
        // believed. Otherwise a package could annotate its way into crediting
        // itself with another's work.
        let claims = routing_claims();
        let refusal = super::route_annotation(&claims, &annotation_in("deny.toml", Some("WP-020")))
            .expect_err("WP-020 does not own deny.toml");
        assert!(refusal.contains("does not own"), "{refusal}");
    }

    #[test]
    fn unroutable_annotations_refuse_generation_before_rendering() {
        let claims = routing_claims();
        let annotations = vec![annotation_in("tools/xtask/src/main.rs", None)];
        let refusal = super::annotations_by_package(&claims, &annotations)
            .expect_err("generation must not silently drop ambiguous evidence");
        assert!(
            refusal.to_string().contains("cannot be routed"),
            "{refusal}"
        );
    }

    #[test]
    fn a_generated_document_announces_itself_and_is_a_pure_function_of_its_input() {
        // The marker is what the drift gate keys on, and determinism is what
        // makes byte equality a usable comparison at all.
        let annotation = annotation_in("deny.toml", None);
        let declared = declared_evidence_in("WP-000");
        let first = super::render_traceability("WP-000", &[&annotation], &[&declared]);
        let second = super::render_traceability("WP-000", &[&annotation], &[&declared]);
        assert_eq!(first, second, "generation must be deterministic");
        assert!(first.starts_with(super::GENERATED_MARKER));
        assert!(first.contains("a_real_test"));
        assert!(first.contains("deny.toml"));
        assert!(first.contains("1 structured evidence row(s)"));
        assert!(
            first.contains("Not established here"),
            "the document must carry its own limit, not rely on a reader knowing it"
        );
    }

    // Requirements: Section 11.7, Section 12
    //   structured evidence is parsed from an exact fenced block, and malformed, duplicate, or unclosed sources are refused rather than skipped
    // Work-Package: WP-000
    // Evidence: structured_evidence_blocks_are_structural_and_typed
    #[test]
    fn structured_evidence_blocks_are_structural_and_typed() {
        let document = "\
Prose may mention ```traceability-evidence without opening the block.

```traceability-evidence
- requirements:
  - Section 1.10
  evidence:
  - path: deny.toml
  - command: cargo xtask ci
  - test: a_real_test
  claim: typed evidence is checked rather than copied
```
";
        let (line, body) = super::exact_fenced_block(document, super::TRACEABILITY_EVIDENCE_BLOCK)
            .expect("the block is structurally valid")
            .expect("the exact fence is present");
        let rows =
            super::parse_declared_evidence("WP-000", "docs/work-packages/WP-000.md", line, &body)
                .expect("the typed YAML must parse");
        assert_eq!(rows, vec![declared_evidence_in("WP-000")]);

        let duplicate = format!("{document}\n{document}");
        assert!(
            super::exact_fenced_block(&duplicate, super::TRACEABILITY_EVIDENCE_BLOCK)
                .expect_err("two blocks are ambiguous")
                .contains("more than one")
        );
        assert!(
            super::exact_fenced_block(
                "```traceability-evidence\n- requirements: []\n",
                super::TRACEABILITY_EVIDENCE_BLOCK
            )
            .expect_err("an unclosed block must not disappear")
            .contains("never closes")
        );

        for invalid in [
            "- requirements: []\n  evidence: []\n  claim: empty must refuse\n",
            "- requirement:\n  - SAFE-007\n  evidence:\n  - test: a_real_test\n  claim: misspelled field\n",
            "- requirements:\n  - SAFE-007\n  - SAFE-007\n  evidence:\n  - test: a_real_test\n  claim: duplicate requirement\n",
            "- requirements:\n  - SAFE-007\n  evidence:\n  - path: deny.toml\n    test: a_real_test\n  claim: ambiguous evidence kind\n",
            "- requirements:\n  - SAFE-007\n  evidence:\n  - test: a_real_test\n  claim: first claim\n  claim: silently replaced claim\n",
            "- &same\n  requirements:\n  - SAFE-007\n  evidence:\n  - test: a_real_test\n  claim: duplicate row\n- *same\n",
        ] {
            assert!(
                super::parse_declared_evidence(
                    "WP-000",
                    "docs/work-packages/WP-000.md",
                    1,
                    invalid
                )
                .is_err(),
                "malformed structured evidence must fail closed: {invalid}"
            );
        }
    }

    // Requirements: Section 11.7, Section 12
    //   a structured row must name known requirements and live, package-owned evidence; invalid commands and stale references each fail closed
    // Work-Package: WP-000
    // Evidence: structured_evidence_is_validated_against_independent_sources
    #[test]
    fn structured_evidence_is_validated_against_independent_sources() {
        let vocabulary = ["Section 1.10".to_owned()].into_iter().collect();
        let tests = ["a_real_test".to_owned()].into_iter().collect();
        let tracked = ["deny.toml".to_owned()].into_iter().collect();
        let claims = routing_claims();
        let row = declared_evidence_in("WP-000");
        super::validate_declared_evidence(&row, &vocabulary, &tests, &tracked, &claims)
            .expect("every independent source supports this row");

        let mut mutations = Vec::new();
        let mut unknown_requirement = row.clone();
        unknown_requirement.requirements = vec!["Section 1.99".to_owned()];
        mutations.push(unknown_requirement);

        let mut untracked_path = row.clone();
        untracked_path.evidence = vec![super::DeclaredEvidenceItem::Path("missing.md".to_owned())];
        mutations.push(untracked_path);

        let mut wrong_owner = row.clone();
        wrong_owner.package = "WP-020".to_owned();
        mutations.push(wrong_owner);

        let mut missing_test = row.clone();
        missing_test.evidence = vec![super::DeclaredEvidenceItem::Test("renamed_test".to_owned())];
        mutations.push(missing_test);

        let mut invalid_command = row;
        invalid_command.evidence = vec![super::DeclaredEvidenceItem::Command(
            "cargo xtask imaginary".to_owned(),
        )];
        mutations.push(invalid_command);

        for mutation in mutations {
            assert!(
                super::validate_declared_evidence(
                    &mutation,
                    &vocabulary,
                    &tests,
                    &tracked,
                    &claims
                )
                .is_err(),
                "every mutation must be refused: {mutation:?}"
            );
        }
    }

    #[test]
    fn a_platform_gated_test_is_not_reported_as_missing_evidence() {
        // A `#[cfg(unix)]` test is absent from a Windows listing and vice
        // versa. Without this the gate would pass on one operating system and
        // fail on another for the same source, which is worse than no gate:
        // the failure would be read as noise and the check disabled.
        let (vocabulary, mut annotations, tests) = traceability_inputs();
        annotations[0].test = "a_test_only_built_on_the_other_platform".to_owned();
        annotations[0].declared_evidence = Some(annotations[0].test.clone());

        annotations[0].platform_gated = false;
        super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect_err("an ungated test that is not listed must still be refused");

        annotations[0].platform_gated = true;
        super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect("a cfg-gated test absent from this platform's listing must be tolerated");
    }

    #[test]
    fn an_annotation_with_no_function_under_it_binds_to_nothing_and_is_refused() {
        // The orphan case: an annotation left behind in a file, or pushed out
        // of range of the function it was written for. Covered here rather than
        // by a source mutation because aiming a mutation at it reliably is
        // harder than stating it directly.
        let lines = vec![
            "    // Requirements: SAFE-007",
            "    //   a claim with nothing beneath it",
            "",
            "    struct NotAFunction;",
        ];
        let annotation = super::parse_annotation("f.rs", 0, " SAFE-007", &lines);
        assert_eq!(annotation.test, "");

        let (vocabulary, _, tests) = traceability_inputs();
        let refusal = super::judge_annotations(&vocabulary, &[annotation], &tests)
            .expect_err("an annotation binding to nothing must be refused");
        assert!(
            refusal.to_string().contains("binds to nothing"),
            "{refusal}"
        );
    }

    #[test]
    fn an_annotation_that_has_slid_onto_another_test_is_refused() {
        // Binding is positional, so on its own it cannot notice that the test
        // it was written for has been renamed or deleted — measured: the first
        // version of this check stayed green through a rename. Writing the name
        // out is what gives the two sources something to disagree about.
        //
        // The redundancy is the mechanism, not an oversight.
        let lines = vec![
            "    // Requirements: SAFE-007",
            "    //   a claim written for one test",
            "    // Evidence: the_test_this_was_written_for",
            "    #[test]",
            "    fn whatever_happens_to_be_here_now() {",
        ];
        let annotation = super::parse_annotation("f.rs", 0, " SAFE-007", &lines);
        assert_eq!(annotation.test, "whatever_happens_to_be_here_now");
        assert_eq!(
            annotation.declared_evidence.as_deref(),
            Some("the_test_this_was_written_for")
        );

        let (vocabulary, _, mut tests) = traceability_inputs();
        tests.insert("whatever_happens_to_be_here_now".to_owned());
        let refusal = super::judge_annotations(&vocabulary, &[annotation], &tests)
            .expect_err("an annotation naming a different test than it sits above must refuse");
        assert!(refusal.to_string().contains("slid onto"), "{refusal}");
    }

    #[test]
    fn an_annotation_that_does_not_name_its_evidence_is_refused() {
        let (vocabulary, mut annotations, tests) = traceability_inputs();
        annotations[0].declared_evidence = None;
        let refusal = super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect_err("an annotation must say which test it documents");
        assert!(
            refusal.to_string().contains("does not say which test"),
            "{refusal}"
        );
    }

    // Requirements: Section 11.7, Section 1.10
    //   the traceability vocabulary comes from real ID definitions, numbered headings, and actual numbered operating-contract items rather than from strings this tool invents
    // Work-Package: WP-000
    // Evidence: the_requirement_vocabulary_comes_from_the_specification_not_from_this_tool
    #[test]
    fn the_requirement_vocabulary_comes_from_the_specification_not_from_this_tool() {
        // The rule the token audit broke when its thresholds lived inside the
        // file it audited, and the rule the canonical vectors follow. If this
        // tool owned the list, an annotation could name anything the tool had
        // been told about.
        let ids = super::spec_traceability_references(&super::repository_root())
            .expect("the specification must yield its own traceability references");
        for defined in ["SAFE-007", "SEC-010", "SEC-005", "MODEL-005", "UI-008"] {
            assert!(ids.contains(defined), "{defined} is defined in the spec");
        }
        for section in ["Section 1", "Section 1.10", "Section 11.7", "Section 12"] {
            assert!(ids.contains(section), "{section} is anchored in the spec");
        }
        // Mentioned in the specification, never *defined* there. Reading only
        // definition sites excludes them without a hand-maintained deny list,
        // which is the property that keeps the set clean as the spec grows.
        for mentioned_only in ["SHA-256", "WP-000", "WP-095", "Section 1.99"] {
            assert!(
                !ids.contains(mentioned_only),
                "{mentioned_only} is mentioned but not defined, so it is not a requirement"
            );
        }

        let synthetic = "\
## 1. Contract
1. First
10. Tenth
Mention Section 1.99 in prose.
## Appendix
11. This is not item 11 of Section 1.
### 11.7 Real numeric heading
";
        let parsed = super::spec_traceability_references_from(synthetic);
        for present in ["Section 1", "Section 1.1", "Section 1.10", "Section 11.7"] {
            assert!(parsed.contains(present), "{present} has a definition site");
        }
        for absent in ["Section 1.11", "Section 1.99"] {
            assert!(
                !parsed.contains(absent),
                "{absent} is mentioned outside a definition site"
            );
        }
    }

    #[test]
    fn an_annotation_cannot_invent_a_requirement() {
        let (vocabulary, mut annotations, tests) = traceability_inputs();
        annotations[0].requirements = vec!["SAFE-999".to_owned()];
        let refusal = super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect_err("an undefined requirement must be refused");
        assert!(refusal.to_string().contains("SAFE-999"), "{refusal}");
    }

    #[test]
    fn an_annotation_naming_a_test_that_no_longer_exists_is_refused() {
        // The failure mode that matters. `docs/traceability/WP-000.md` has
        // already had to delete three rows naming tests that had been removed,
        // and a document citing absent evidence is worse than one citing none.
        let (vocabulary, mut annotations, tests) = traceability_inputs();
        annotations[0].test = "a_test_that_was_renamed".to_owned();
        let refusal = super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect_err("an annotation naming a missing test must be refused");
        assert!(
            refusal.to_string().contains("a_test_that_was_renamed"),
            "{refusal}"
        );
    }

    #[test]
    fn an_annotation_without_a_claim_is_refused() {
        // The claim is the traceability table's cell. Without it the generated
        // document would have to invent one, which is where hand-maintenance
        // creeps back in.
        let (vocabulary, mut annotations, tests) = traceability_inputs();
        annotations[0].claim = String::new();
        let refusal = super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect_err("an annotation with no claim must be refused");
        assert!(refusal.to_string().contains("claim"), "{refusal}");
    }

    #[test]
    fn finding_no_annotations_is_a_failure_not_a_clean_run() {
        // Section 12: the generator must not be able to pass vacuously. Zero
        // annotations is the shape an accidentally-disabled scanner takes.
        let (vocabulary, _, tests) = traceability_inputs();
        let refusal = super::judge_annotations(&vocabulary, &[], &tests)
            .expect_err("an empty annotation set must fail rather than report a clean run");
        assert!(refusal.to_string().contains("no requirement annotations"));
    }

    #[test]
    fn the_happy_path_reports_the_requirements_it_actually_covered() {
        // The control. Without it every assertion above could be passing
        // because the judgement refuses everything.
        let (vocabulary, annotations, tests) = traceability_inputs();
        let covered = super::judge_annotations(&vocabulary, &annotations, &tests)
            .expect("a well-formed annotation must be accepted");
        assert_eq!(covered, 1);
    }

    #[test]
    fn an_annotation_binds_to_the_function_below_it_across_its_attributes() {
        // `#[test]` and `#[cfg(windows)]` sit between the annotation and the
        // function, so the binding has to step over them.
        let lines = vec![
            "    // Requirements: SAFE-007",
            "    //   the claim, which continues",
            "    //   onto a second line",
            "    #[cfg(windows)]",
            "    #[test]",
            "    fn the_bound_test() {",
        ];
        let annotation = super::parse_annotation("f.rs", 0, " SAFE-007", &lines);
        assert_eq!(annotation.requirements, vec!["SAFE-007".to_owned()]);
        assert_eq!(annotation.test, "the_bound_test");
        assert_eq!(
            annotation.claim,
            "the claim, which continues onto a second line"
        );
    }

    #[test]
    fn an_ordinary_comment_under_an_annotation_is_not_swallowed_as_a_claim() {
        // Continuation lines are indented under the marker. Without that rule
        // the next ordinary comment would be absorbed into the claim and end up
        // printed in the traceability table.
        let lines = vec![
            "    // Requirements: SAFE-007",
            "    //   the real claim",
            "    // an ordinary comment about the implementation",
            "    fn the_bound_test() {",
        ];
        let annotation = super::parse_annotation("f.rs", 0, " SAFE-007", &lines);
        assert_eq!(annotation.claim, "the real claim");
    }

    #[test]
    fn requirement_ids_are_recognised_only_in_definition_position() {
        assert_eq!(
            super::leading_requirement_id("SAFE-007: Host protection"),
            Some("SAFE-007".to_owned())
        );
        assert_eq!(
            super::leading_requirement_id("SEC-010:** Supply chain"),
            Some("SEC-010".to_owned())
        );
        assert_eq!(
            super::leading_requirement_id("SAFE-007 without a colon"),
            None
        );

        // **Shape is not what makes the vocabulary clean, and the first
        // version of this test claimed it was.** `SHA-256` and `WP-000` are
        // both ID-shaped and this function accepts them — asserted rather than
        // hidden, because the wrong version passed review in my own head and
        // was caught only by running it.
        //
        // What excludes them is that neither appears in *definition* position
        // in the specification. That is a property of the document, not of this
        // predicate, and it is checked against the real document by
        // `the_requirement_vocabulary_comes_from_the_specification_not_from_this_tool`.
        // If the spec ever grew a `### SHA-256:` heading the vocabulary would
        // absorb it and that test is what would notice.
        assert_eq!(
            super::leading_requirement_id("SHA-256: a digest"),
            Some("SHA-256".to_owned())
        );
        assert_eq!(
            super::leading_requirement_id("WP-000: a package"),
            Some("WP-000".to_owned())
        );
        assert_eq!(
            super::leading_requirement_id("SAFE-7: too few digits"),
            None
        );
    }

    // Requirements: SAFE-007, SAFE-005
    //   Tier 2 and Tier 3 refuse by default; an unavailable tier fails closed rather than running
    // Work-Package: WP-000
    // Evidence: unavailable_destructive_tiers_fail_closed
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
            parse(&args(&["traceability"])).expect("traceability"),
            Task::Traceability { write: false }
        );
        assert_eq!(
            parse(&args(&["traceability", "--write"])).expect("traceability --write"),
            Task::Traceability { write: true }
        );
        assert_eq!(
            parse(&args(&["supply-chain"])).expect("supply-chain"),
            Task::SupplyChain
        );
        assert_eq!(parse(&args(&["tokens"])).expect("tokens"), Task::Tokens);
        assert_eq!(
            parse(&args(&["verify-ownership"])).expect("verify-ownership"),
            Task::VerifyOwnership
        );
        assert_eq!(
            parse(&args(&["verify-actions"])).expect("verify-actions"),
            Task::VerifyActions
        );
        assert_eq!(
            parse(&args(&["verify-licenses"])).expect("verify-licenses"),
            Task::VerifyLicenses
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

    // Requirements: SAFE-007
    //   the profile is a command-line argument, so it cannot be inherited by accident from a parent shell
    // Work-Package: WP-000
    // Evidence: a_destructive_profile_is_an_argument_not_an_environment_variable
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

    // Requirements: SAFE-007
    //   no destructive suite exists, so the runner refuses rather than reporting a pass over an empty run
    // Work-Package: WP-000
    // Evidence: a_destructive_tier_refuses_even_with_the_profile_word
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

    // Requirements: SEC-010
    //   a full commit SHA is accepted and a tag or branch reference is refused, which is what "pinned by digest" has to mean
    // Work-Package: WP-000
    // Evidence: digest_pins_are_accepted_and_mutable_references_are_not
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

    /// Build a temporary repository root holding one workflow, and optionally
    /// one local action, then run the real gate over it.
    fn scan_workflow(
        tag: &str,
        workflow: &str,
        local: Option<(&str, &str)>,
    ) -> Result<(), TaskError> {
        // A per-call counter as well as the process id, so no two invocations
        // ever address the same directory. Reusing a path was flaky on Windows,
        // where `remove_dir_all` can return before the deletion is visible and
        // the next `create_dir_all` then fails with NotFound. Never reusing a
        // name removes the race instead of retrying around it.
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "partman-xtask-scan-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let workflows = root.join(super::WORKFLOW_DIRECTORY);
        fs::create_dir_all(&workflows).expect("create workflow directory");
        fs::write(workflows.join("ci.yml"), workflow).expect("write workflow");
        if let Some((relative, contents)) = local {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create action directory");
            fs::write(path, contents).expect("write action metadata");
        }
        let result = verify_action_pins(&root);
        let _ = fs::remove_dir_all(&root);
        result
    }

    fn pinned_workflow() -> String {
        format!(
            "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@{} # v7.0.1\n",
            "a".repeat(40)
        )
    }

    #[test]
    fn a_plainly_pinned_and_tagged_workflow_passes() {
        scan_workflow("baseline", &pinned_workflow(), None)
            .expect("a digest-pinned, tag-commented reference is the accepted form");
    }

    // Requirements: SEC-010
    //   action discovery is a structural YAML parse: the quoted-key, anchored-key and YAML-escape bypasses that defeated three text scanners are permanent regressions
    // Work-Package: WP-000
    // Evidence: the_three_bypasses_the_text_scanners_missed_are_refused
    #[test]
    fn the_three_bypasses_the_text_scanners_missed_are_refused() {
        // Every row is a spelling that passed a previous version of this gate
        // while *reducing* the reported reference count — silence shaped like a
        // pass. They are kept permanently because each one cost an audit round
        // to find.
        //
        // A: YAML decodes `@` to `@`, so a scanner searching source text
        //    for a literal `@` never sees the reference at all.
        // B: `docker://image:tag` is a documented step-level reference that is
        //    mutable and contains no `@` whatsoever.
        // C: a local action outside `.github/actions/` was never recursed into,
        //    so its own remote references went unread.
        let escaped =
            "jobs:\n  build:\n    steps:\n      - &pin uses: \"actions/checkout\\u0040v7\"\n";
        let error = scan_workflow("escape", escaped, None)
            .expect_err("a YAML-escaped `@` must not hide a mutable reference");
        assert!(
            error.to_string().contains("actions/checkout@v7"),
            "the decoded reference must be named: {error}"
        );

        let docker = "jobs:\n  build:\n    steps:\n      - &pin uses: docker://alpine:3.20\n";
        let error = scan_workflow("docker", docker, None)
            .expect_err("a mutable container tag must be refused");
        let message = error.to_string();
        assert!(
            message.contains("docker://alpine:3.20"),
            "the image reference must be named: {error}"
        );
        // Asserting the container-specific guidance, not merely that *some*
        // refusal happened. A deletion sweep found that removing the container
        // branch still refused this input — `is_pinned` catches it as "not
        // pinned to a full commit SHA", which is true but tells a reader to
        // look for a git SHA on a Docker image. The branch exists for the
        // message, so the test has to hold it to the message.
        assert!(
            message.contains("pinned by digest") && message.contains("sha256"),
            "a container must be told to use an image digest, not a commit SHA: {error}"
        );

        let local_workflow = "jobs:\n  build:\n    steps:\n      - uses: ./.github/local-action\n";
        let metadata = "runs:\n  using: composite\n  steps:\n    - uses: actions/cache@v4\n";
        let error = scan_workflow(
            "local",
            local_workflow,
            Some((".github/local-action/action.yml", metadata)),
        )
        .expect_err("a local action's own remote references must be inspected wherever it lives");
        assert!(
            error.to_string().contains("actions/cache@v4"),
            "the nested reference must be named: {error}"
        );
    }

    #[test]
    fn discovery_does_not_depend_on_how_the_key_is_spelled() {
        // The parser decides what the document says, so these are all the same
        // key and all carry the same mutable tag. Each must be refused; none may
        // simply vanish from the count.
        for spelling in [
            "      - uses: actions/checkout@v7",
            "      - \"uses\": actions/checkout@v7",
            "      - 'uses': actions/checkout@v7",
            "      - &pin uses: actions/checkout@v7",
            "      - !!str uses: actions/checkout@v7",
            "      - { uses: actions/checkout@v7 }",
        ] {
            let workflow = format!("jobs:\n  build:\n    steps:\n{spelling}\n");
            let result = scan_workflow("spelling", &workflow, None);
            assert!(
                result.is_err(),
                "{spelling:?} carries a mutable tag and must be refused, however the key is \
                 spelled"
            );
        }
    }

    #[test]
    fn a_pinned_digest_still_needs_a_readable_release_tag() {
        // The parser owns discovery; the trailing comment remains the
        // auditability layer, because a bare 40-character SHA tells a reviewer
        // nothing about which release it is. A reference spelled so obscurely
        // that the comment cannot be associated with it fails here, which is
        // why writing one that way is a build failure rather than an escape.
        let sha = "a".repeat(40);
        let no_comment =
            format!("jobs:\n  build:\n    steps:\n      - uses: actions/checkout@{sha}\n");
        let error = scan_workflow("no-tag", &no_comment, None)
            .expect_err("a digest with no release-tag comment must be refused");
        assert!(error.to_string().contains("no `# <tag>` comment"));

        let vague = format!(
            "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@{sha} # do not touch\n"
        );
        let error = scan_workflow("vague-tag", &vague, None)
            .expect_err("a comment that names no version must be refused");
        assert!(error.to_string().contains("does not name a release tag"));
    }

    #[test]
    fn a_local_reference_must_actually_be_runnable_and_inside_the_repository() {
        // A `./` reference is exempt from pinning because it runs this
        // repository's own code — so the exemption is only sound if the thing
        // it names exists, is inspectable, and is really local.
        let missing = "jobs:\n  build:\n    steps:\n      - uses: ./.github/nope\n";
        assert!(
            scan_workflow("missing-local", missing, None).is_err(),
            "a local reference resolving to nothing must be refused, not exempted"
        );

        let bare_directory = "jobs:\n  build:\n    steps:\n      - uses: ./.github/bare\n";
        let error = scan_workflow(
            "bare-local",
            bare_directory,
            Some((".github/bare/readme.txt", "not action metadata\n")),
        )
        .expect_err("a local directory without action metadata must be refused");
        assert!(error.to_string().contains("action.yml"));

        let escaping = "jobs:\n  build:\n    steps:\n      - uses: ./../outside\n";
        assert!(
            scan_workflow("escaping-local", escaping, None).is_err(),
            "a local reference pointing outside the repository must be refused"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_action_metadata_file_cannot_escape_the_repository() {
        // A `./` reference is exempt from pinning because it runs this
        // repository's own code. Checking the *directory* is inside the tree
        // says nothing about where its contents point: a symlinked `action.yml`
        // aimed outside would have been read and trusted, and whatever it
        // declared would have been treated as first-party.
        //
        // A deletion sweep found this fix had no test — the containment check
        // could be removed with every test still green, which is exactly the
        // criticism the audit made of the traversal coverage.
        let root =
            std::env::temp_dir().join(format!("partman-xtask-symlink-meta-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let workflows = root.join(super::WORKFLOW_DIRECTORY);
        fs::create_dir_all(&workflows).expect("create workflow directory");
        fs::write(
            workflows.join("ci.yml"),
            "jobs:\n  build:\n    steps:\n      - uses: ./.github/act\n",
        )
        .expect("write workflow");
        let action = root.join(".github/act");
        fs::create_dir_all(&action).expect("create action directory");

        // The real metadata lives outside the repository and declares a mutable
        // reference, so following the link would both escape containment and
        // read an unpinned dependency as though it were ours.
        let outside = std::env::temp_dir().join(format!(
            "partman-xtask-outside-meta-{}.yml",
            std::process::id()
        ));
        fs::write(
            &outside,
            "runs:\n  using: composite\n  steps:\n    - uses: actions/cache@v4\n",
        )
        .expect("write outside metadata");
        std::os::unix::fs::symlink(&outside, action.join("action.yml"))
            .expect("symlink action.yml outside the repository");

        let result = verify_action_pins(&root);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&outside);

        let error = result.expect_err("a symlinked action.yml pointing outside must be refused");
        assert!(
            error.to_string().contains("outside the repository"),
            "the refusal must say containment failed, not merely that something was unpinned: \
             {error}"
        );
    }

    #[test]
    fn a_workflow_that_cannot_be_parsed_is_a_violation_not_a_skip() {
        // GitHub might still run what this tool cannot read, so unparseable
        // input fails closed.
        let broken = "jobs:\n  build:\n  steps:\n   - uses: [unclosed\n";
        assert!(
            scan_workflow("unparseable", broken, None).is_err(),
            "invalid YAML must be refused rather than skipped"
        );
    }

    #[test]
    fn prose_and_comments_are_not_references() {
        // A parser reads `run:` as a string and comments not at all, so neither
        // registers — without the over-refusal the text sweep needed.
        // The `run:` value is single-quoted because `echo "uses: x"` as a bare
        // plain scalar is not valid YAML — a colon-space inside an unquoted
        // scalar. Worth noting in passing: the parser refuses that, where the
        // old text scanner accepted it silently.
        let sha = "a".repeat(40);
        let workflow = format!(
            "jobs:\n  build:\n    steps:\n      # uses: actions/stale@v9\n      - uses: actions/checkout@{sha} # v7.0.1\n      - run: 'echo \"uses: actions/checkout@v6\"'\n"
        );
        scan_workflow("prose", &workflow, None)
            .expect("a mention inside a script or a comment is not a dependency");
    }

    #[test]
    fn recursion_through_local_references_terminates() {
        // A composite action that references itself must not spin. The visited
        // set is keyed on the canonical path.
        let workflow = "jobs:\n  build:\n    steps:\n      - uses: ./.github/loop\n";
        let metadata = "runs:\n  using: composite\n  steps:\n    - uses: ./.github/loop\n";
        scan_workflow(
            "cycle",
            workflow,
            Some((".github/loop/action.yml", metadata)),
        )
        .expect("a self-referential local action is not a violation, only a cycle to survive");
    }

    fn yaml_field<'a>(node: &'a Yaml, key: &str) -> &'a Yaml {
        node.as_hash()
            .and_then(|map| map.get(&Yaml::String(key.to_owned())))
            .unwrap_or_else(|| panic!("workflow field {key:?} must exist"))
    }

    fn workflow_document(name: &str) -> Yaml {
        let workflow_path = repository_root().join(".github/workflows").join(name);
        let text = fs::read_to_string(&workflow_path).expect("read workflow");
        let mut documents = YamlLoader::load_from_str(&text).expect("workflow is YAML");
        assert_eq!(
            documents.len(),
            1,
            "{name} must contain exactly one YAML document"
        );
        documents.remove(0)
    }

    fn workflow_os_matrix<'a>(job: &'a Yaml, description: &str) -> BTreeSet<&'a str> {
        yaml_field(yaml_field(yaml_field(job, "strategy"), "matrix"), "os")
            .as_vec()
            .unwrap_or_else(|| panic!("{description} OS matrix must be a sequence"))
            .iter()
            .map(|os| os.as_str().expect("runner labels must be strings"))
            .collect()
    }

    fn assert_pull_request_gates_remain_complete() {
        let workflow = workflow_document("ci.yml");
        let triggers = yaml_field(&workflow, "on")
            .as_hash()
            .expect("CI `on` must be a mapping");
        assert!(
            triggers.contains_key(&Yaml::String("pull_request".to_owned())),
            "CI must continue to gate pull requests"
        );

        let jobs = yaml_field(&workflow, "jobs");
        let job_map = jobs.as_hash().expect("CI jobs must be a mapping");
        let job_ids: BTreeSet<&str> = job_map
            .keys()
            .map(|key| key.as_str().expect("CI job IDs must be strings"))
            .collect();
        assert_eq!(
            job_ids,
            [
                "cross-language",
                "fuzz-smoke",
                "prober-acceptance",
                "supply-chain",
                "tier-1",
            ]
            .into_iter()
            .collect(),
            "scheduled maintenance must not replace or remove a pull-request gate"
        );

        let every_os: BTreeSet<&str> = ["macos-15", "ubuntu-24.04", "windows-2025"]
            .into_iter()
            .collect();
        for (job_id, display_name) in [
            ("tier-1", "Tier 1 / ${{ matrix.os }}"),
            (
                "cross-language",
                "Cross-language hash parity / ${{ matrix.os }}",
            ),
            ("supply-chain", "Supply-chain policy / ${{ matrix.os }}"),
        ] {
            let job = yaml_field(jobs, job_id);
            assert_eq!(
                yaml_field(job, "name").as_str(),
                Some(display_name),
                "branch protection depends on the {job_id} display name"
            );
            assert_eq!(
                workflow_os_matrix(job, job_id),
                every_os,
                "{job_id} must continue to cover all three operating systems"
            );
        }
        for (job_id, display_name) in [
            ("prober-acceptance", "Real-prober acceptance (FS-004)"),
            ("fuzz-smoke", "Fuzz smoke (Section 11.4)"),
        ] {
            let job = yaml_field(jobs, job_id);
            assert_eq!(yaml_field(job, "name").as_str(), Some(display_name));
            assert_eq!(yaml_field(job, "runs-on").as_str(), Some("ubuntu-24.04"));
        }
    }

    fn assert_maintenance_triggers(workflow: &Yaml) {
        let triggers = yaml_field(workflow, "on");
        let trigger_map = triggers.as_hash().expect("`on` must be a mapping");
        assert!(
            trigger_map.contains_key(&Yaml::String("workflow_dispatch".to_owned())),
            "maintenance must remain manually runnable"
        );
        let crons: Vec<&str> = yaml_field(triggers, "schedule")
            .as_vec()
            .expect("`on.schedule` must be a sequence")
            .iter()
            .map(|entry| {
                yaml_field(entry, "cron")
                    .as_str()
                    .expect("each schedule needs a cron string")
            })
            .collect();
        assert_eq!(
            crons,
            ["0 6 * * 1"],
            "maintenance must run every Monday at 06:00 UTC"
        );
    }

    fn assert_long_fuzz_job(job: &Yaml) {
        assert_eq!(
            yaml_field(job, "runs-on").as_str(),
            Some("ubuntu-24.04"),
            "cargo-fuzz's supported CI platform is Linux"
        );
        assert!(
            yaml_field(job, "timeout-minutes")
                .as_i64()
                .is_some_and(|minutes| minutes >= 45),
            "the timeout must cover two 15-minute targets plus setup"
        );

        let steps = yaml_field(job, "steps")
            .as_vec()
            .expect("long-fuzz steps must be a sequence");
        let cache_step = steps
            .iter()
            .find(|step| {
                step.as_hash()
                    .and_then(|map| map.get(&Yaml::String("uses".to_owned())))
                    .and_then(Yaml::as_str)
                    .is_some_and(|uses| uses.starts_with("actions/cache@"))
            })
            .expect("the long run must restore and save its corpus");
        let cache_inputs = yaml_field(cache_step, "with");
        assert_eq!(
            yaml_field(cache_inputs, "path").as_str(),
            Some("fuzz/corpus/"),
            "the cache must contain cargo-fuzz's per-target corpora"
        );
        assert!(
            yaml_field(cache_inputs, "key")
                .as_str()
                .is_some_and(|key| key.contains("${{ github.run_id }}")),
            "each successful run needs a new immutable cache key"
        );
        assert_eq!(
            yaml_field(cache_inputs, "restore-keys").as_str(),
            Some("fuzz-corpus-${{ runner.os }}-\n"),
            "a run must restore the latest earlier corpus"
        );
        assert!(
            steps.iter().any(|step| {
                step.as_hash()
                    .and_then(|map| map.get(&Yaml::String("run".to_owned())))
                    .and_then(Yaml::as_str)
                    == Some("cargo xtask fuzz --seconds 900")
            }),
            "the scheduled run must give each target 15 minutes"
        );
    }

    fn assert_scheduled_supply_chain_job(job: &Yaml) {
        assert_eq!(
            workflow_os_matrix(job, "scheduled supply-chain"),
            ["macos-15", "ubuntu-24.04", "windows-2025"]
                .into_iter()
                .collect(),
            "scheduled advisory checks must cover every supported operating system"
        );
        assert!(
            yaml_field(job, "steps")
                .as_vec()
                .expect("supply-chain steps must be a sequence")
                .iter()
                .any(|step| {
                    step.as_hash()
                        .and_then(|map| map.get(&Yaml::String("run".to_owned())))
                        .and_then(Yaml::as_str)
                        == Some("cargo xtask supply-chain")
                }),
            "the scheduled matrix must run the real repository policy command"
        );
    }

    // Requirements: Section 11.4, SEC-010
    //   weekly and manually triggered maintenance retains a growing fuzz corpus, gives every parser target substantially longer than the pull-request smoke pass, and rechecks both dependency graphs on every supported operating system without changing pull-request jobs
    // Work-Package: WP-000
    // Evidence: scheduled_maintenance_retains_long_fuzz_corpora_and_audits_every_os
    #[test]
    fn scheduled_maintenance_retains_long_fuzz_corpora_and_audits_every_os() {
        assert_pull_request_gates_remain_complete();
        let workflow = workflow_document("maintenance.yml");
        assert_maintenance_triggers(&workflow);
        let jobs = yaml_field(&workflow, "jobs");
        assert_long_fuzz_job(yaml_field(jobs, "long-fuzz"));
        assert_scheduled_supply_chain_job(yaml_field(jobs, "supply-chain"));
    }

    // Requirements: SEC-010
    //   job containers, the container shorthand, service containers and a Docker action image are all pinned, not only `uses:` references
    // Work-Package: WP-000
    // Evidence: container_images_are_executable_dependencies_too
    #[test]
    fn container_images_are_executable_dependencies_too() {
        // `uses:` is not the only way a workflow runs third-party code, and the
        // previous scanner saw none of these. GitHub pulls and runs a job
        // container, a service container, and a Docker action's `runs.image`.
        // Each must be pinned by content digest, because a tag can be repointed
        // exactly like a mutable action tag.
        let job_container =
            "jobs:\n  build:\n    container:\n      image: alpine:3.20\n    steps: []\n";
        let error = scan_workflow("job-container", job_container, None)
            .expect_err("a job container must be pinned by digest");
        assert!(
            error.to_string().contains("alpine:3.20"),
            "the image must be named: {error}"
        );

        // The scalar shorthand for the same thing.
        let shorthand = "jobs:\n  build:\n    container: alpine:3.20\n    steps: []\n";
        assert!(
            scan_workflow("container-shorthand", shorthand, None).is_err(),
            "`container: <image>` is the documented shorthand and must be checked too"
        );

        let service = "jobs:\n  build:\n    services:\n      db:\n        image: postgres:16\n    steps: []\n";
        let error = scan_workflow("service", service, None)
            .expect_err("a service container must be pinned by digest");
        assert!(error.to_string().contains("postgres:16"));

        // A digest-pinned container is the accepted form.
        let pinned = format!(
            "jobs:\n  build:\n    container:\n      image: alpine@sha256:{}\n    steps: []\n",
            "b".repeat(64)
        );
        scan_workflow("container-pinned", &pinned, None)
            .expect("an image pinned by sha256 digest is immutable and acceptable");

        // A local Docker action's own base image.
        let workflow = "jobs:\n  build:\n    steps:\n      - uses: ./.github/docker-action\n";
        let metadata = "runs:\n  using: docker\n  image: docker://alpine:3.20\n";
        let error = scan_workflow(
            "docker-action",
            workflow,
            Some((".github/docker-action/action.yml", metadata)),
        )
        .expect_err("a Docker action's image must be pinned by digest");
        assert!(error.to_string().contains("alpine:3.20"));
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one regression per confirmed bypass; splitting them hides the roster"
    )]
    // Requirements: SEC-010
    //   a Docker action is followed into its Dockerfile, where every image the build pulls must be digest-pinned
    // Work-Package: WP-000
    // Evidence: a_dockerfile_action_is_followed_to_its_base_images
    fn a_dockerfile_action_is_followed_to_its_base_images() {
        // `image: Dockerfile` builds from source, so the executable dependency
        // is that file's `FROM` lines rather than a pullable reference.
        let root =
            std::env::temp_dir().join(format!("partman-xtask-dockerfile-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let workflows = root.join(super::WORKFLOW_DIRECTORY);
        fs::create_dir_all(&workflows).expect("create workflow directory");
        fs::write(
            workflows.join("ci.yml"),
            "jobs:\n  build:\n    steps:\n      - uses: ./.github/built\n",
        )
        .expect("write workflow");
        let action = root.join(".github/built");
        fs::create_dir_all(&action).expect("create action directory");
        fs::write(
            action.join("action.yml"),
            "runs:\n  using: docker\n  image: Dockerfile\n",
        )
        .expect("write action metadata");

        fs::write(action.join("Dockerfile"), "FROM alpine:3.20\nRUN true\n")
            .expect("write Dockerfile");
        let error = verify_action_pins(&root).expect_err("an unpinned FROM must be refused");
        assert!(
            error.to_string().contains("alpine:3.20"),
            "the base image must be named: {error}"
        );

        // Digest-pinned, plus a multi-stage build whose second FROM references
        // an internal stage rather than pulling anything.
        let pinned = format!(
            "FROM alpine@sha256:{} AS builder\nFROM builder\nRUN true\n",
            "c".repeat(64)
        );
        fs::write(action.join("Dockerfile"), pinned).expect("write pinned Dockerfile");
        verify_action_pins(&root)
            .expect("a digest-pinned base and an internal stage reference are both fine");

        // Nine ways a mutable image used to pass this gate unseen. Each is a
        // permanent regression, and each was confirmed against the scanner
        // before the rewrite -- the gate exited successfully on all of them.
        let digest = format!("alpine@sha256:{}", "c".repeat(64));
        for (name, dockerfile, expected) in [
            // The reviewer's three.
            (
                "an ARG-supplied base",
                "ARG BASE=alpine:3.20\nFROM ${BASE}\n".to_owned(),
                "${BASE}",
            ),
            (
                "a mixed-case instruction",
                "From alpine:3.20\n".to_owned(),
                "alpine:3.20",
            ),
            (
                "a BuildKit frontend, which is pulled and executed as the builder",
                format!("# syntax=docker/dockerfile:1\nFROM {digest}\n"),
                "docker/dockerfile:1",
            ),
            // Four that need no unusual syntax at all.
            (
                "a tab instead of a space",
                "FROM\talpine:3.20\n".to_owned(),
                "alpine:3.20",
            ),
            (
                "a UTF-8 BOM on the first line",
                "\u{feff}FROM alpine:3.20\n".to_owned(),
                "alpine:3.20",
            ),
            (
                "COPY --from, which pulls an image no FROM names",
                format!("FROM {digest}\nCOPY --from=busybox:1.36 /bin/busybox /bin/\n"),
                "busybox:1.36",
            ),
            (
                "RUN --mount=from, same",
                format!("FROM {digest}\nRUN --mount=type=bind,from=golang:1.24,target=/go true\n"),
                "golang:1.24",
            ),
            // And two spellings of a stage shadowing an image.
            (
                "a stage that names itself after the image it pulls",
                "FROM alpine AS alpine\nFROM alpine\n".to_owned(),
                "alpine",
            ),
            (
                "a continuation whose backslash was stored as a stage name",
                "FROM alpine:3.20 AS \\\n  builder\n".to_owned(),
                "alpine:3.20",
            ),
        ] {
            fs::write(action.join("Dockerfile"), &dockerfile).expect("write Dockerfile");
            let error = verify_action_pins(&root).expect_err("must be refused");
            assert!(
                error.to_string().contains(expected),
                "{name}: the refusal must name {expected}: {error}"
            );
        }

        // Two things that must NOT be violations, checked rather than assumed:
        // `scratch` is not a pull, and `check=`/`escape=` name no image.
        for (name, dockerfile) in [
            ("FROM scratch", "FROM scratch\n".to_owned()),
            (
                "a check directive",
                format!("# check=error=true\nFROM {digest}\n"),
            ),
            (
                "an escape directive",
                format!("# escape=`\nFROM {digest}\n"),
            ),
            (
                "COPY --from naming an earlier stage",
                format!("FROM {digest} AS build\nFROM {digest}\nCOPY --from=build /x /x\n"),
            ),
            (
                "COPY --from naming a stage by index",
                format!("FROM {digest}\nFROM {digest}\nCOPY --from=0 /x /x\n"),
            ),
        ] {
            fs::write(action.join("Dockerfile"), &dockerfile).expect("write Dockerfile");
            verify_action_pins(&root)
                .unwrap_or_else(|error| panic!("{name} is not a pull and must pass: {error}"));
        }

        let _ = fs::remove_dir_all(&root);
    }

    // Requirements: SEC-010
    //   npm advisory coverage follows discovery rather than a hard-coded path, so a new package cannot be born unaudited
    // Work-Package: WP-000
    // Evidence: the_npm_advisory_check_finds_every_package_not_one_named_directory
    #[test]
    fn the_npm_advisory_check_finds_every_package_not_one_named_directory() {
        // The advisory check ran in `packages/canonical` by name, because that
        // was the only npm package there was. WP-030 reserves `packages/ui/`,
        // `packages/design-tokens/` and `apps/desktop/`, and a Tauri front end
        // brings its own `package.json` -- each of which would have been audited
        // by nobody while the gate reported success.
        let mut manifests = Vec::new();
        super::manifest_files_under(&repository_root(), &mut manifests)
            .expect("walk the repository");
        let packages: Vec<&PathBuf> = manifests
            .iter()
            .filter(|path| path.file_name().and_then(OsStr::to_str) == Some("package.json"))
            .collect();
        assert!(
            packages
                .iter()
                .any(|path| path.ends_with("packages/canonical/package.json")),
            "the package that used to be named must still be found: {packages:?}"
        );
        // Every one that exists carries the lockfile the audit needs, so the
        // gate's own refusal path is not tripped by the current tree.
        for path in &packages {
            let directory = path.parent().expect("package directory");
            assert!(
                directory.join("package-lock.json").is_file(),
                "{} has no committed lockfile, so `npm audit` would report on a tree that \
                 install time decides",
                directory.display()
            );
        }
    }

    #[test]
    fn a_manifest_outside_the_root_is_refused_rather_than_dropped() {
        // macOS caught this within minutes of the change reaching CI. The first
        // version wrote `if let Ok(relative) = path.strip_prefix(root)`, which
        // silently discarded anything that did not strip -- and on macOS nothing
        // did, because `std::env::temp_dir()` is `/var/folders/…` while `/var` is
        // a symlink to `/private/var` and `cargo metadata` answers with the
        // resolved path. The membership set shrank to the workspace root alone
        // and a legitimate change was refused, listing manifests that did not
        // include the one the author had just edited.
        let root = repository_root();
        assert_eq!(
            relative_to_root(
                &root,
                &root.join("crates").join("tokens").join("Cargo.toml")
            )
            .expect("a path under the root relativizes"),
            "crates/tokens/Cargo.toml"
        );

        // The property that matters: a path that cannot be placed inside the
        // root is an error, never a quiet omission. A membership answer with a
        // member missing refuses a change that was entitled to carry the
        // lockfile, and says nothing about why.
        let error = relative_to_root(&root, Path::new("/elsewhere/Cargo.toml"))
            .expect_err("an outside manifest must be refused, not dropped");
        assert!(
            error.to_string().contains("Refusing to drop it"),
            "the refusal must say why silence would be wrong: {error}"
        );
    }

    // Requirements: SAFE-009
    //   `unsafe_code = "deny"` reaches a crate only if it opts into the workspace lints, so a member omitting the stanza is refused rather than silently exempt
    // Work-Package: WP-000
    // Evidence: every_workspace_member_inherits_the_lint_policy
    #[test]
    fn every_workspace_member_inherits_the_lint_policy() {
        verify_workspace_lints(&repository_root())
            .expect("every member of this workspace opts into `[workspace.lints]`");

        // The hole this closes, measured: a member that omits the stanza gets
        // none of `[workspace.lints]`, so an `unsafe fn` compiles clean and
        // `cargo xtask ci` stays green. `unsafe_code = "deny"` was opt-in.
        assert!(inherits_workspace_lints(
            "[package]\nname = \"x\"\n\n[lints]\nworkspace = true\n"
        ));
        assert!(inherits_workspace_lints("[lints]\nworkspace=true\n"));
        assert!(!inherits_workspace_lints("[package]\nname = \"x\"\n"));
        assert!(
            !inherits_workspace_lints("[package]\nname = \"x\"\n\n[lints]\nworkspace = false\n"),
            "opting out is not opting in"
        );
        // A commented-out stanza inherits nothing. (This one holds because a
        // `#` line is not a table header, not because comments are stripped --
        // a deletion sweep showed the assertion passing either way, so it is
        // labelled honestly rather than left looking like evidence.)
        assert!(
            !inherits_workspace_lints("[package]\nname = \"x\"\n# [lints]\n# workspace = true\n"),
            "a commented stanza inherits nothing"
        );
        // Comment stripping earns its place in the other direction: a trailing
        // comment must not make a real opt-in invisible. Refusing a manifest
        // that did opt in would send an author looking for a fault that is not
        // there.
        assert!(
            inherits_workspace_lints("[lints] # policy\nworkspace = true # everything\n"),
            "a trailing comment must not hide a real opt-in"
        );
        // And the stanza must be in `[lints]`, not merely somewhere in the file.
        assert!(
            !inherits_workspace_lints("[lints]\nrust = {}\n\n[dependencies]\nworkspace = true\n"),
            "`workspace = true` under another table is a different setting"
        );
    }

    #[test]
    fn a_release_comment_cannot_be_borrowed_from_another_step() {
        // Two steps sharing one SHA, only the second tagged. The previous check
        // searched the whole file for the reference and returned the first
        // comment it found, so the bare step passed on the tagged step's
        // comment — a reviewer looking at step one would see no version at all.
        let sha = "a".repeat(40);
        let borrowed = format!(
            "jobs:\n  one:\n    steps:\n      - uses: actions/checkout@{sha}\n  two:\n    steps:\n      - uses: actions/checkout@{sha} # v7.0.1\n"
        );
        let error = scan_workflow("borrowed-tag", &borrowed, None)
            .expect_err("every occurrence needs its own release-tag comment");
        assert!(
            error.to_string().contains("carry no `# <tag>` comment"),
            "the refusal must say which sites are untagged: {error}"
        );

        // Both tagged is fine, and proves the check is not simply counting
        // occurrences and refusing repeats.
        let both = format!(
            "jobs:\n  one:\n    steps:\n      - uses: actions/checkout@{sha} # v7.0.1\n  two:\n    steps:\n      - uses: actions/checkout@{sha} # v7.0.1\n"
        );
        scan_workflow("both-tagged", &both, None)
            .expect("the same action pinned twice, tagged at both sites, is acceptable");
    }

    // Requirements: SEC-010
    //   this repository's own workflows pass the digest-pinning gate, so the check is exercised against real input rather than only against fixtures
    // Work-Package: WP-000
    // Evidence: the_repository_workflows_pass_the_real_gate
    #[test]
    fn the_repository_workflows_pass_the_real_gate() {
        verify_action_pins(&repository_root())
            .expect("this repository's own workflows must satisfy the gate");
    }

    // Requirements: SEC-005
    //   every Cargo and npm manifest declares the project licence, checked semantically so the release inventory has a subject
    // Work-Package: WP-000
    // Evidence: every_repository_manifest_declares_the_project_licence
    #[test]
    fn every_repository_manifest_declares_the_project_licence() {
        verify_manifest_licenses(&repository_root())
            .expect("every manifest in this repository declares MIT OR Apache-2.0");
    }

    // Requirements: SEC-005
    //   a `license` key nested anywhere but the document root does not satisfy the inventory
    // Work-Package: WP-000
    // Evidence: a_nested_json_licence_property_does_not_satisfy_the_gate
    #[test]
    fn a_nested_json_licence_property_does_not_satisfy_the_gate() {
        // The 2026-07-29 follow-up audit's reproduction: the old check matched
        // trimmed lines, so moving the property under `metadata` left the text
        // `"license": "MIT OR Apache-2.0"` on a line while the document's root
        // `license` was undefined. A line cannot tell you where in a document
        // it sits, which is why the check parses JSON now.
        let root = std::env::temp_dir().join(format!(
            "partman-xtask-nested-licence-{}",
            std::process::id()
        ));
        let write = |relative: &str, contents: &str| {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            fs::write(path, contents).expect("write file");
        };
        write("LICENSE-MIT", "MIT License\n");
        write("LICENSE-APACHE", "Apache License\n");

        // Root-level: accepted.
        write(
            "packages/web/package.json",
            "{\n  \"license\": \"MIT OR Apache-2.0\"\n}\n",
        );
        verify_manifest_licenses(&root).expect("a root-level licence passes");

        // Nested under another object: refused, even though the *line* matches.
        write(
            "packages/web/package.json",
            "{\n  \"metadata\": {\n    \"license\": \"MIT OR Apache-2.0\"\n  }\n}\n",
        );
        let error = verify_manifest_licenses(&root)
            .expect_err("a nested licence property must not satisfy the gate");
        assert!(
            error.to_string().contains("root-level"),
            "the refusal should say why: {error}"
        );

        // Not a string, and not an object at all: both refused rather than
        // coerced.
        write("packages/web/package.json", "{\n  \"license\": 42\n}\n");
        assert!(
            verify_manifest_licenses(&root).is_err(),
            "non-string licence"
        );
        write("packages/web/package.json", "[]\n");
        assert!(verify_manifest_licenses(&root).is_err(), "array manifest");
        write("packages/web/package.json", "{ not json\n");
        assert!(verify_manifest_licenses(&root).is_err(), "invalid JSON");

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn every_tracked_path_belongs_to_a_work_package() {
        verify_path_ownership(&repository_root())
            .expect("every tracked file is claimed by a work-package assignment");
    }

    #[test]
    fn ownership_claim_patterns_are_understood_or_refused() {
        // A pattern the checker cannot interpret must be an error, not a
        // pattern that quietly matches nothing. That is the failure mode the
        // action scanner was audited for twice, and it applies here for the
        // same reason: a claim matching nothing reads as coverage.
        for good in [
            "crates/domain/**",
            "Cargo.toml",
            "docs/work-packages/WP-000.md",
        ] {
            validate_claim_pattern("WP-test", good).unwrap_or_else(|error| {
                panic!("{good:?} should be understood, got {error}");
            });
        }
        for bad in [
            "crates/*/src",
            "*.md",
            "crates/**/tests",
            "crates\\domain\\**",
        ] {
            assert!(
                validate_claim_pattern("WP-test", bad).is_err(),
                "{bad:?} must be refused rather than silently matching nothing"
            );
        }
    }

    #[test]
    fn a_change_must_declare_which_work_package_it_belongs_to() {
        // The 2026-07-29 second follow-up audit's F-06: the inventory check
        // proves every path is claimed by *someone*, which is why PR #47 passed
        // while editing three other packages' documents. These tests exercise
        // the four behaviours that close it, in a throwaway repository so they
        // cannot depend on this one's history.
        let repo = GitFixture::new("declare");

        // No trailer: refused, because no tool can infer the package from a diff.
        repo.write("tools/xtask/src/main.rs", "// edited\n");
        repo.commit("touch xtask with no declaration");
        let error = repo
            .check()
            .expect_err("an undeclared change must be refused");
        assert!(
            error.to_string().contains("carry no declaration"),
            "the refusal must name the undeclared commits: {error}"
        );

        // The right trailer: accepted.
        repo.amend("touch xtask\n\nWork-Package: WP-000");
        repo.check().expect("WP-000 owns tools/xtask/**");

        // Two packages in one change: refused. A shared path still has exactly
        // one owning package for a given change.
        repo.amend("touch xtask\n\nWork-Package: WP-000\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("a change belonging to two packages must be split");
        assert!(error.to_string().contains("more than one work package"));
    }

    #[test]
    fn every_commit_declares_for_itself_and_git_decides_what_a_trailer_is() {
        // Four ways a commit used to travel without saying what it belonged to.
        // All four were found by audit against the version that unioned the
        // trailers of a whole range and scanned for them by hand.
        let repo = GitFixture::new("per-commit");

        // One trailered commit used to launder every untrailered commit beside
        // it: the set contained exactly one package, so the range passed.
        repo.write("tools/xtask/src/main.rs", "// first\n");
        repo.commit("first commit, no trailer at all");
        repo.write("tools/xtask/src/second.rs", "// second\n");
        repo.commit("second commit\n\nWork-Package: WP-000");
        let error = repo
            .check()
            .expect_err("a trailer on one commit must not cover the commit before it");
        assert!(
            error.to_string().contains("carry no declaration"),
            "the undeclared commit must be named: {error}"
        );

        // A quoted example is not a trailer. Git's parser says so, and the hand
        // scan did not: it accepted any line that began with the key after
        // trimming, which a fenced block in a commit body satisfies.
        let repo = GitFixture::new("fenced");
        repo.write("tools/xtask/src/main.rs", "// edited\n");
        repo.commit(
            "explain the ownership rule\n\nRun it like this:\n\n    Work-Package: WP-000\n\nand \
             it passes.\n",
        );
        let error = repo
            .check()
            .expect_err("an example in the body is not a declaration");
        assert!(error.to_string().contains("carry no declaration"));

        // Git's key matching is case-insensitive, and the hand scan's was not,
        // so a valid trailer was refused. Adopting git's parser fixes both
        // directions at once.
        repo.amend("edited\n\nwork-package: WP-000");
        repo.check()
            .expect("git accepts a lowercase trailer key, so this gate must too");

        // A governance change with no reason is an audit record of nothing. It
        // used to be accepted and printed as an empty parenthesis.
        let repo = GitFixture::new("empty-reason");
        repo.write(
            "docs/work-packages/WP-020.md",
            "# WP-020\n\n```owned-paths\ndocs/work-packages/WP-020.md\n```\n\nedited\n",
        );
        repo.commit("move a path\n\nGovernance:");
        let error = repo
            .check()
            .expect_err("a governance declaration needs its reason");
        assert!(error.to_string().contains("carry no declaration"));

        // Declaring both modes in one commit is a contradiction, not a
        // precedence question. Governance used to win silently, so the work
        // package named beside it was never checked against anything.
        repo.amend("move a path\n\nWork-Package: WP-000\nGovernance: paperwork");
        let error = repo
            .check()
            .expect_err("a commit is ordinary work or an assignment change, never both");
        assert!(
            error.to_string().contains("never both"),
            "the refusal must say why: {error}"
        );
    }

    #[test]
    fn a_rename_is_judged_at_both_ends() {
        // Rename detection is on by default and `--name-only` prints only a
        // rename's destination, so `git mv` carried a file out of another
        // package's territory and the gate saw only where it landed.
        let repo = GitFixture::new("rename");
        super::git(
            &repo.root,
            &[
                "mv",
                "docs/work-packages/WP-020.md",
                "tools/xtask/stolen.md",
            ],
        )
        .expect("rename across packages");
        repo.commit("reorganise\n\nWork-Package: WP-000");
        let error = repo
            .check()
            .expect_err("the source of a rename is a deletion and must be judged");
        assert!(
            error.to_string().contains("docs/work-packages/WP-020.md"),
            "the vacated path must be named, not just the destination: {error}"
        );

        // Worse, the same blind spot let a `Governance:` change delete any file
        // in the repository by renaming it to an assignment document's name:
        // every path the check could see was an assignment document.
        let repo = GitFixture::new("rename-governance");
        super::git(
            &repo.root,
            &[
                "mv",
                "tools/xtask/src/main.rs",
                "docs/work-packages/WP-777.md",
            ],
        )
        .expect("rename code into a document name");
        repo.commit("paperwork\n\nGovernance: adding WP-777");
        let error = repo
            .check()
            .expect_err("a governance change must not be able to delete code");
        assert!(error.to_string().contains("tools/xtask/src/main.rs"));

        // A rename wholly inside one package is honest work and still passes.
        let repo = GitFixture::new("rename-inside");
        super::git(
            &repo.root,
            &["mv", "tools/xtask/src/main.rs", "tools/xtask/src/cli.rs"],
        )
        .expect("rename within a package");
        repo.commit("rename a module\n\nWork-Package: WP-000");
        repo.check()
            .expect("both halves are owned, so both halves pass");
    }

    #[test]
    fn a_path_is_a_byte_string_not_a_trimmed_quoted_one() {
        // Two more defects lived in the same expression as the rename one.
        //
        // `--name-only` C-quotes a non-ASCII path, so a legitimate file inside
        // owned territory arrived as `"tools/xtask/caf\303\251.rs"` and matched
        // no claim -- the gate refusing work it should permit, which costs trust
        // as fast as a bypass does.
        let repo = GitFixture::new("unicode");
        repo.write("tools/xtask/café.rs", "// accented\n");
        repo.commit("add an accented filename\n\nWork-Package: WP-000");
        repo.check()
            .expect("a non-ASCII path inside an owned directory is owned");

        // And git does not quote a leading space, so `.map(str::trim)` silently
        // normalised ` tools/xtask/x.rs` onto the owned path. Trimming a path is
        // a normalisation the ownership catalogue never agreed to.
        let repo = GitFixture::new("leading-space");
        repo.write(" not-owned.rs", "// leading space\n");
        repo.commit("add a path with a leading space\n\nWork-Package: WP-000");
        let error = repo
            .check()
            .expect_err("a leading space must not be normalised away");
        assert!(error.to_string().contains("not-owned.rs"));
    }

    #[test]
    fn a_merge_commit_is_exempt_because_this_repository_cannot_trailer_one() {
        // The exemption is deliberate and load-bearing, not an oversight
        // inherited from the union bug. Branch protection is `strict: true`, so
        // `gh pr update-branch` merges main into every stale branch and GitHub
        // writes that commit itself; for a `pull_request` event CI checks out
        // GitHub's generated `refs/pull/N/merge`. Neither can carry a trailer.
        // `main` holds 51 merge commits and none of them has one, so a literal
        // "every commit" rule would have failed every pull request on the day
        // it landed.
        let repo = GitFixture::new("merge");
        repo.write("tools/xtask/src/main.rs", "// on the branch\n");
        repo.commit("do the work\n\nWork-Package: WP-000");
        repo.merge_untrailered_into_head();
        repo.check()
            .expect("an untrailered merge commit must not fail the gate");
    }

    #[test]
    fn a_change_cannot_reach_outside_its_assignment_or_widen_it_to_fit() {
        let repo = GitFixture::new("stray");

        // The PR #47 shape: declare WP-000, edit a document WP-020 owns.
        repo.write(
            "docs/work-packages/WP-020.md",
            "# WP-020\n\n```owned-paths\ndocs/work-packages/WP-020.md\n```\n\nedited\n",
        );
        repo.commit("edit WP-020's document\n\nWork-Package: WP-000");
        let error = repo
            .check()
            .expect_err("reaching into another package's paths must be refused");
        assert!(
            error.to_string().contains("docs/work-packages/WP-020.md"),
            "the stray path must be named: {error}"
        );

        // And widening your own assignment in the same change must not rescue
        // it — the catalogue is read from the base for exactly this reason.
        // This is the hole the audit identified in the inventory check.
        repo.write(
            "docs/work-packages/WP-000.md",
            "# WP-000\n\n```owned-paths\ntools/xtask/**\ndocs/work-packages/WP-000.md\ndocs/work-packages/WP-020.md\n```\n",
        );
        repo.commit("widen WP-000 to cover it\n\nWork-Package: WP-000");
        let error = repo
            .check()
            .expect_err("self-widening must not defeat the check");
        assert!(
            error.to_string().contains("does not help"),
            "the refusal should explain why widening failed: {error}"
        );
    }

    #[test]
    fn a_governance_change_may_move_assignments_but_carry_nothing_else() {
        let repo = GitFixture::new("governance");

        // Assignments alone: accepted.
        repo.write(
            "docs/work-packages/WP-020.md",
            "# WP-020\n\n```owned-paths\ndocs/work-packages/WP-020.md\ncrates/fixtures/**\n\
             crates/tokens/**\n```\n",
        );
        repo.commit("reassign a path\n\nGovernance: crates/tokens moves to WP-020");
        repo.check()
            .expect("a governance change may edit assignments");

        // Assignments plus code: refused, or the trailer becomes a universal
        // bypass for exactly the check it sits beside.
        repo.write("tools/xtask/src/main.rs", "// smuggled\n");
        repo.commit("and quietly change code too\n\nGovernance: still just paperwork");
        let error = repo
            .check()
            .expect_err("a governance change must not carry code");
        assert!(error.to_string().contains("tools/xtask/src/main.rs"));
    }

    #[test]
    fn a_generated_lockfile_is_regenerated_by_whoever_changes_a_manifest() {
        // The deadlock this closes, measured against `02ec952`: a WP-030 change
        // creating `apps/desktop/src-tauri` was refused for `Cargo.lock` and
        // `Cargo.toml`; the same tree declaring WP-000 was refused for the crate
        // it had to create. Neither package could take the first step, and the
        // wall is not WP-030's -- every package that adds a dependency rewrites
        // the lockfile, which only WP-000 claims.
        let repo = GitFixture::new("derived");

        // A manifest this package owns, and the lockfile that follows from it:
        // accepted, because the lockfile is generated rather than authored.
        repo.write(
            "crates/fixtures/Cargo.toml",
            "[package]\nname = \"fixture-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
             publish = false\n# a dependency was added\n",
        );
        repo.write("Cargo.lock", "# regenerated\n");
        repo.commit("add a dependency\n\nWork-Package: WP-020");
        repo.check()
            .expect("a lockfile that follows a manifest this package owns is regeneration");

        // A file merely *named* `Cargo.toml`, inside territory this package
        // genuinely owns, does not unlock the lockfile. This is the hole an
        // adversarial pass found in the previous version, where the predicate
        // was the basename: a note, a fixture or a symlink called `Cargo.toml`
        // anywhere a package owned was accepted as a manifest.
        let repo = GitFixture::new("derived-decoy");
        repo.write("crates/fixtures/notes/Cargo.toml", "# still not a member\n");
        repo.write("Cargo.lock", "# carried in on a decoy\n");
        repo.commit("add a note that looks like a manifest\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("a path is not a manifest merely for being called one");
        assert!(
            error.to_string().contains("Cargo.lock"),
            "the carried lockfile must be the refusal: {error}"
        );

        // The lockfile alone: refused. Nothing in the change asks the resolver
        // for a different answer, so this is a hand edit wearing regeneration's
        // clothes -- a transitive dependency re-pinned to a different version
        // still satisfies every manifest, so `--locked` would accept it.
        let repo = GitFixture::new("derived-alone");
        repo.write("Cargo.lock", "# re-pinned by hand\n");
        repo.commit("quietly move a pin\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("a lockfile moving on its own is not regeneration");
        assert!(
            error.to_string().contains("not regeneration"),
            "the refusal must explain why a generated file was still refused: {error}"
        );

        // A manifest in an excluded workspace does not unlock the root lockfile,
        // because it cannot change it. The first version of this rule accepted
        // any `Cargo.toml` anywhere and would have passed this; the second
        // matched the nearest lockfile above it, which held only while that
        // lockfile existed. Membership comes from `exclude` now, so deleting
        // `nested/Cargo.lock` cannot re-open it.
        let repo = GitFixture::new("derived-nested");
        repo.write(
            "nested/Cargo.toml",
            "[package]\nname = \"nested-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
             publish = false\n# a dependency was added\n",
        );
        repo.write("Cargo.lock", "# and the root lock moved too\n");
        repo.commit("edit the excluded workspace\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("an excluded manifest cannot vouch for the root lockfile");
        assert!(error.to_string().contains("Cargo.lock"));

        // And the exemption is load-bearing: without the `derived-paths`
        // declaration the accepted case above goes back to being refused. This
        // is the deletion sweep -- a check that cannot fail is not a check.
        let repo = GitFixture::new_without_derived_declaration("underived");
        repo.write(
            "crates/fixtures/Cargo.toml",
            "[package]\nname = \"fixture-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
             publish = false\n# a dependency was added\n",
        );
        repo.write("Cargo.lock", "# regenerated\n");
        repo.commit("add a dependency\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("undeclared, the lockfile is WP-000's alone");
        assert!(error.to_string().contains("Cargo.lock"));

        // And a package cannot declare a file generated that it does not answer
        // for. Generatedness is a property of the file rather than a privilege
        // of one assignment, which is why any document may state it -- but an
        // adversarial pass pointed out that a document stating it about someone
        // else's file is a unilateral grant to everybody, made in a change that
        // only edits assignment documents.
        let repo = GitFixture::new_with_a_stranger_declaring_it("stranger");
        repo.write(
            "crates/fixtures/Cargo.toml",
            "[package]\nname = \"fixture-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
             publish = false\n# a dependency was added\n",
        );
        repo.write("Cargo.lock", "# regenerated\n");
        repo.commit("add a dependency\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("declaring another package's file generated grants nothing");
        assert!(error.to_string().contains("Cargo.lock"));
    }

    #[test]
    fn declaring_a_path_generated_is_not_claiming_it() {
        // `derived-paths` says how a file comes to be, not who answers for it.
        // If it counted as coverage, "this is generated" would be a way to make
        // a file belong to nobody while the inventory still read as complete.
        let repo = GitFixture::new("inventory");
        verify_path_ownership(&repo.root).expect("the base catalogue covers every tracked file");

        repo.write(
            "docs/work-packages/WP-000.md",
            "# WP-000\n\n```owned-paths\ntools/xtask/**\nCargo.toml\n\
             docs/work-packages/WP-000.md\n```\n\n```derived-paths\nCargo.lock\n```\n",
        );
        let error = verify_path_ownership(&repo.root)
            .expect_err("a path only declared generated is claimed by nobody");
        assert!(
            error.to_string().contains("Cargo.lock"),
            "the unclaimed path must be named: {error}"
        );
    }

    #[test]
    fn a_reservation_that_has_started_matching_files_is_coverage() {
        // The two halves of Section 1.10 disagreed, and it deadlocked the shell.
        // `verify_change_ownership` lets a package write inside its own
        // reservation, so the first commit to do so passed the change gate --
        // and then `cargo xtask ci` failed the inventory with "claimed by no
        // work package" about a path the package had claimed in advance, in the
        // document, precisely so this could not happen.
        //
        // The promotion that would have resolved it has no legal route: a
        // governance change moving the paths early leaves `main` red on a stale
        // claim, and moving them in the same change as the files is an
        // assignment edit under a `Work-Package:` trailer, which AGENTS.md
        // forbids. So a reservation counts once it matches something.
        let repo = GitFixture::new("reserved");
        repo.write("apps/desktop/src-tauri/Cargo.toml", "# the shell arrives\n");
        repo.commit("start the shell\n\nWork-Package: WP-020");

        verify_path_ownership(&repo.root)
            .expect("a reserved claim that now matches files covers them");
        repo.check()
            .expect("and the change gate agrees, as it always did");

        // A reservation matching nothing is still reported rather than counted,
        // and a file no claim reaches at all is still a violation.
        let repo = GitFixture::new("reserved-empty");
        verify_path_ownership(&repo.root).expect("an unmatched reservation is not a violation");
        repo.write("unclaimed.rs", "// nobody claims this\n");
        repo.commit("add an unclaimed file\n\nWork-Package: WP-000");
        let error = verify_path_ownership(&repo.root)
            .expect_err("a path outside every claim is still unclaimed");
        assert!(error.to_string().contains("unclaimed.rs"));
    }

    #[test]
    fn a_derived_path_needs_a_derivation_this_tool_can_check() {
        // `derived-paths` is an exemption from the ownership check. An exemption
        // covering a path whose regeneration nothing can verify is a hole with a
        // comment beside it, so an unknown derivation is refused rather than
        // trusted.
        validate_derived_pattern("WP-test", "Cargo.lock").expect("the one defined derivation");
        validate_derived_pattern("WP-test", "fuzz/Cargo.lock").expect("nested lockfiles too");
        for unknown in [
            "Cargo.toml",
            "docs/traceability/WP-000.md",
            "package-lock.json",
        ] {
            assert!(
                validate_derived_pattern("WP-test", unknown).is_err(),
                "{unknown:?} has no defined derivation and must be refused, not exempted"
            );
        }

        // The plausibility rule is membership now, not spelling: `cargo
        // metadata` answers, so the set is the real workspace.
        let resolves =
            workspace_manifests(&repository_root(), "Cargo.lock").expect("the root workspace");
        assert!(
            resolves.contains("Cargo.toml"),
            "the virtual root manifest must count — adding a member to it is the most \
             legitimate reason of all for the lockfile to move: {resolves:?}"
        );
        assert!(resolves.contains("crates/fixtures/Cargo.toml"));

        assert!(derivation_is_plausible(
            &["crates/fixtures/Cargo.toml".to_owned()],
            &resolves
        ));
        assert!(derivation_is_plausible(
            &["Cargo.toml".to_owned()],
            &resolves
        ));
        assert!(!derivation_is_plausible(&[], &resolves));

        // The hole an adversarial pass found in the second version of this rule:
        // a file merely *named* `Cargo.toml`, anywhere a package already owns,
        // unlocked the root lockfile. Membership is not a spelling.
        for decoy in [
            "docs/quality/Cargo.toml",
            "crates/fixtures/tests/Cargo.toml",
            "docs/quality/Cargo.toml.md",
        ] {
            assert!(
                !derivation_is_plausible(&[decoy.to_owned()], &resolves),
                "{decoy} is not a member's manifest and must not unlock the lockfile"
            );
        }

        // And the hole found in the first version: `fuzz/` is excluded from the
        // root workspace, so its manifest cannot change the root lock. That now
        // follows from `exclude` rather than from a lockfile sitting beside it,
        // so deleting `fuzz/Cargo.lock` can no longer re-open it.
        assert!(
            !resolves.contains("fuzz/Cargo.toml"),
            "cargo agrees `fuzz` is not a member: {resolves:?}"
        );
        assert!(!derivation_is_plausible(
            &["fuzz/Cargo.toml".to_owned()],
            &resolves
        ));
    }

    /// Who, if anyone, declares `Cargo.lock` generated in a fixture catalogue.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Declared {
        /// WP-000, which also owns it. The real arrangement.
        ByItsOwner,
        /// WP-020, which does not own it.
        ByAStranger,
        /// Nobody.
        NotAtAll,
    }

    /// A throwaway git repository with a minimal ownership catalogue.
    ///
    /// Built rather than reusing this repository, so the tests neither depend on
    /// nor disturb real history, and `--base` means something definite.
    struct GitFixture {
        root: PathBuf,
    }

    impl GitFixture {
        fn new(tag: &str) -> Self {
            Self::build(tag, Declared::ByItsOwner)
        }

        /// The same catalogue with WP-000's `derived-paths` block removed, so a
        /// test can watch the exemption stop working.
        fn new_without_derived_declaration(tag: &str) -> Self {
            Self::build(tag, Declared::NotAtAll)
        }

        /// `Cargo.lock` declared generated by WP-020, which does not own it.
        fn new_with_a_stranger_declaring_it(tag: &str) -> Self {
            Self::build(tag, Declared::ByAStranger)
        }

        fn build(tag: &str, derived: Declared) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "partman-xtask-git-{tag}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let fixture = Self { root };
            fs::create_dir_all(&fixture.root).expect("create fixture repository");
            for args in [
                vec!["init", "--initial-branch=main"],
                vec!["config", "user.email", "test@example.invalid"],
                vec!["config", "user.name", "Test"],
                vec!["config", "commit.gpgsign", "false"],
            ] {
                super::git(&fixture.root, &args).expect("initialise the fixture repository");
            }
            // The base revision's catalogue: WP-000 owns xtask, the workspace
            // files and its own document; WP-020 owns its own and the fixture
            // crate. WP-000 also declares the lockfile generated, which is what
            // lets another package's manifest change carry it.
            let by_owner = if derived == Declared::ByItsOwner {
                "\n```derived-paths\nCargo.lock\n```\n"
            } else {
                ""
            };
            let by_stranger = if derived == Declared::ByAStranger {
                "\n```derived-paths\nCargo.lock\n```\n"
            } else {
                ""
            };
            fixture.write(
                "docs/work-packages/WP-000.md",
                &format!(
                    "# WP-000\n\n```owned-paths\ntools/xtask/**\nCargo.toml\nCargo.lock\n\
                     docs/work-packages/WP-000.md\n```\n{by_owner}"
                ),
            );
            fixture.write(
                "docs/work-packages/WP-020.md",
                &format!(
                    "# WP-020\n\n```owned-paths\ndocs/work-packages/WP-020.md\n\
                     crates/fixtures/**\nnested/**\n```\n\n```owned-paths-reserved\napps/**\n\
                     ```\n{by_stranger}"
                ),
            );
            // A *real* Cargo workspace, because the lockfile rule asks cargo
            // which manifests are members. A fixture of text files named
            // `Cargo.toml` would prove the plumbing and skip the question the
            // rule now turns on -- which is exactly the mistake that made the
            // previous version of this rule wrong.
            fixture.write(
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/fixtures\"]\nexclude = [\"nested\"]\n\
                 resolver = \"3\"\n",
            );
            fixture.write(
                "crates/fixtures/Cargo.toml",
                "[package]\nname = \"fixture-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
                 publish = false\n",
            );
            fixture.write("crates/fixtures/src/lib.rs", "");
            // Named like a manifest, owned by WP-020, and not a member: the
            // decoy the adversarial pass used to unlock the root lockfile.
            fixture.write("crates/fixtures/notes/Cargo.toml", "# not a member\n");
            // A workspace excluded from the root one -- `fuzz/` in the real
            // repository.
            fixture.write(
                "nested/Cargo.toml",
                "[package]\nname = \"nested-crate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
                 publish = false\n",
            );
            fixture.write("nested/src/lib.rs", "");
            fixture.write("tools/xtask/src/main.rs", "// base\n");
            for manifest in ["Cargo.toml", "nested/Cargo.toml"] {
                let status = Command::new("cargo")
                    .args([
                        "generate-lockfile",
                        "--offline",
                        "--manifest-path",
                        manifest,
                    ])
                    .current_dir(&fixture.root)
                    .output()
                    .expect("run cargo generate-lockfile");
                assert!(
                    status.status.success(),
                    "fixture workspace {manifest} must resolve: {}",
                    String::from_utf8_lossy(&status.stderr)
                );
            }
            fixture.commit("base");
            super::git(&fixture.root, &["tag", "base"]).expect("tag the base revision");
            fixture
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            fs::write(path, contents).expect("write fixture file");
        }

        fn commit(&self, message: &str) {
            super::git(&self.root, &["add", "-A"]).expect("stage");
            super::git(&self.root, &["commit", "-m", message]).expect("commit");
        }

        fn amend(&self, message: &str) {
            super::git(&self.root, &["commit", "--amend", "-m", message]).expect("amend");
        }

        /// Add a genuine untrailered merge commit, the shape
        /// `gh pr update-branch` produces when `main` moves under a branch.
        ///
        /// The side branch must really diverge: merging an ancestor is a no-op
        /// that creates no commit at all, and a deletion sweep caught exactly
        /// that — the first version of this helper made a test that could not
        /// fail.
        fn merge_untrailered_into_head(&self) {
            let branch = super::git(&self.root, &["rev-parse", "HEAD"]).expect("head");
            super::git(&self.root, &["checkout", "-q", "base"]).expect("detach onto base");
            self.write("tools/xtask/src/meanwhile.rs", "// landed on main\n");
            self.commit("unrelated xtask work landed on main\n\nWork-Package: WP-000");
            let sideline = super::git(&self.root, &["rev-parse", "HEAD"]).expect("sideline");
            super::git(&self.root, &["checkout", "-q", branch.trim()]).expect("back to the branch");
            super::git(
                &self.root,
                &[
                    "merge",
                    "--no-ff",
                    "--no-edit",
                    "-m",
                    "Merge branch 'main' into work/something",
                    sideline.trim(),
                ],
            )
            .expect("merge without a trailer");
            assert_eq!(
                super::git(
                    &self.root,
                    &["rev-list", "--count", "--merges", "base..HEAD"]
                )
                .expect("count merges")
                .trim(),
                "1",
                "the fixture must actually contain a merge commit, or the test cannot fail"
            );
        }

        fn check(&self) -> Result<(), TaskError> {
            verify_change_ownership(&self.root, "base")
        }
    }

    impl Drop for GitFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn claim_matching_is_prefix_exact_not_substring() {
        // `crates/tokens/**` must not claim `crates/tokens-extra/lib.rs`: a
        // sibling directory sharing a name prefix is a different package's
        // territory, and a substring match would silently annex it.
        assert!(claim_matches(
            "crates/tokens/**",
            "crates/tokens/src/lib.rs"
        ));
        assert!(!claim_matches(
            "crates/tokens/**",
            "crates/tokens-extra/src/lib.rs"
        ));
        assert!(!claim_matches("crates/tokens/**", "crates/tokens"));
        assert!(claim_matches("Cargo.toml", "Cargo.toml"));
        assert!(!claim_matches("Cargo.toml", "fuzz/Cargo.toml"));
    }

    #[test]
    fn the_fuzz_lock_preflight_is_shared_by_supply_chain_and_fuzz() {
        // The follow-up audit removed a package entry from `fuzz/Cargo.lock`,
        // ran `supply-chain`, and watched it pass *and repair the lock* —
        // `cargo deny` resolves the manifest to build its graph, so whichever
        // command runs first is the one that can silently fix what it audits.
        // Both entry points now call this preflight before any resolving
        // command. Asserting on the source keeps the ordering from silently
        // regressing, which no runtime assertion in this process could.
        let source = fs::read_to_string(repository_root().join("tools/xtask/src/main.rs"))
            .expect("read xtask source");
        let supply_chain = source
            .split_once("Task::SupplyChain =>")
            .expect("the supply-chain arm exists")
            .1;
        let arm = &supply_chain[..supply_chain.find("Task::").unwrap_or(supply_chain.len())];
        let preflight = arm
            .find("verify_fuzz_lock()")
            .expect("supply-chain must run the fuzz-lock preflight");
        let first_deny = arm.find("\"deny\"").unwrap_or(usize::MAX);
        assert!(
            preflight < first_deny,
            "the preflight must run before cargo-deny, which resolves and can repair the lock"
        );
        assert!(
            source.contains("fn verify_fuzz_lock()"),
            "the preflight is a shared function, not duplicated logic"
        );
    }

    #[test]
    fn a_manifest_that_loses_its_licence_key_is_a_violation() {
        // The WP-000 gap this check closes: fuzz/Cargo.toml sits outside
        // cargo-deny's graph and package.json outside any licence gate, so
        // either could lose its declaration with CI green. Each mutation below
        // must fail, and the passing tree proves the check is not vacuous.
        let root =
            std::env::temp_dir().join(format!("partman-xtask-licenses-{}", std::process::id()));
        let write = |relative: &str, contents: &str| {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            fs::write(path, contents).expect("write manifest");
        };

        // No Cargo manifests in this tree: Cargo licences are resolved by
        // `cargo metadata` against a real workspace, which the repository-wide
        // test above covers. Here the subject is the npm manifest and the
        // licence texts.
        write("LICENSE-MIT", "MIT License\n");
        write("LICENSE-APACHE", "Apache License\n");
        write(
            "packages/web/package.json",
            "{\n  \"license\": \"MIT OR Apache-2.0\"\n}\n",
        );
        verify_manifest_licenses(&root).expect("a fully declared tree passes");

        // A different licence is a violation, not a variation.
        write(
            "packages/web/package.json",
            "{\n  \"license\": \"GPL-3.0-only\"\n}\n",
        );
        assert!(verify_manifest_licenses(&root).is_err(), "wrong licence");

        // Absent entirely.
        write("packages/web/package.json", "{\n  \"name\": \"web\"\n}\n");
        assert!(verify_manifest_licenses(&root).is_err(), "missing licence");
        write(
            "packages/web/package.json",
            "{\n  \"license\": \"MIT OR Apache-2.0\"\n}\n",
        );

        // A declaration with no licence text behind it is a violation too.
        fs::remove_file(root.join("LICENSE-APACHE")).expect("remove licence text");
        assert!(verify_manifest_licenses(&root).is_err(), "missing text");

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_cargo_manifest_outside_every_workspace_is_a_violation() {
        // The other half of making Cargo licences semantic: `cargo metadata`
        // is authoritative for packages it knows about, so a manifest neither
        // graph includes is not "fine by default" — it is a package no licence
        // gate resolves, which is exactly the gap `verify-licenses` exists to
        // close.
        //
        // A synthetic workspace, not the real repository: planting an orphan in
        // the shared tree would race every other test that reads it, which the
        // first version of this test did.
        let root =
            std::env::temp_dir().join(format!("partman-xtask-orphan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let write = |relative: &str, contents: &str| {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            fs::write(path, contents).expect("write file");
        };
        write("LICENSE-MIT", "MIT License\n");
        write("LICENSE-APACHE", "Apache License\n");
        write(
            "Cargo.toml",
            "[workspace]\nmembers = [\"member\"]\nresolver = \"3\"\n\n\
             [workspace.package]\nversion = \"0.0.0\"\nedition = \"2024\"\n\
             license = \"MIT OR Apache-2.0\"\n",
        );
        write(
            "member/Cargo.toml",
            "[package]\nname = \"member\"\nversion.workspace = true\n\
             edition.workspace = true\nlicense.workspace = true\n",
        );
        write("member/src/lib.rs", "");
        verify_manifest_licenses(&root).expect("a real workspace with inherited licence passes");

        // Now an orphan: a Cargo manifest no workspace includes, so no licence
        // gate resolves it — a violation even though its own key is correct.
        write(
            "orphan/Cargo.toml",
            "[package]\nname = \"orphan\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
             license = \"MIT OR Apache-2.0\"\n",
        );
        let error = verify_manifest_licenses(&root)
            .expect_err("a Cargo manifest outside every workspace must be refused");
        assert!(
            error.to_string().contains("orphan"),
            "the refusal must name the orphan manifest: {error}"
        );

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn the_xtask_alias_enforces_the_lockfile_at_the_gate_boundary() {
        // The `--locked` flags inside this binary bind only once the binary is
        // built. The build that loads the gate is governed by the alias in
        // `.cargo/config.toml`, and the 2026-07-29 audit showed what its
        // absence permits: a deleted Cargo.lock entry was silently regenerated
        // while building xtask, and all 160 tests then passed against a
        // lockfile the repository never committed. This test cannot re-run
        // that experiment cheaply, but it can make the flag's removal fail CI
        // by name.
        let config = repository_root().join(".cargo/config.toml");
        let text = fs::read_to_string(&config).expect("read .cargo/config.toml");
        let alias = text
            .lines()
            .find(|line| line.trim_start().starts_with("xtask ="))
            .expect("the xtask alias exists");
        assert!(
            alias.contains("--locked"),
            "the xtask alias must carry --locked; without it the gate can repair the \
             lockfile it claims to enforce: {alias}"
        );
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
