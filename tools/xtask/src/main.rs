//! Safe, unprivileged repository task runner.

use std::collections::BTreeMap;
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
const LOCAL_ACTIONS_DIRECTORY: &str = ".github/actions";

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
    Tokens,
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
        "verify-ownership" => nullary(Task::VerifyOwnership, command, rest),
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
        Task::VerifyActions => verify_action_pins(&repository_root()),
        Task::VerifyLicenses => verify_manifest_licenses(&repository_root()),
        Task::VerifyOwnership => verify_path_ownership(&repository_root()),
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

    // Composite actions committed to this repository run `uses:` references of
    // their own. Exempting a local action from digest checks is safe only if
    // its remote references are inspected; scanning nothing under
    // `.github/actions/` would make `./.github/actions/foo` a place to keep a
    // mutable tag. The directory is optional — workflows are not — so its
    // absence is fine, but when present every YAML file under it is read.
    let actions_directory = root.join(LOCAL_ACTIONS_DIRECTORY);
    if actions_directory.is_dir() {
        yaml_files_under(&actions_directory, &mut workflows)?;
        workflows.sort();
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
        let mut attributed: Vec<(usize, String)> = Vec::new();
        for entry in action_references(&text) {
            attributed.push((entry.line, entry.reference.clone()));
            match entry.violation() {
                None => pinned += 1,
                Some(reason) => violations.push(format!(
                    "{name}:{}: {} — {reason}",
                    entry.line, entry.reference
                )),
            }
        }
        // The key-shaped reader above can only find references it can parse,
        // and twice now an audit has spelled a `uses` key in valid YAML it
        // could not: a quoted key, then an anchored one (`&pin uses: …`). Each
        // time the scanner reported success having simply counted one fewer
        // reference, which is the worst failure mode a gate has — silence that
        // looks like a pass.
        //
        // So discovery no longer depends on recognising the key. An action
        // reference must literally contain `owner/repo@ref`, whatever syntax
        // surrounds it, and this sweep finds every such token independently.
        // Anything the reader did not attribute to a `uses:` key is a
        // violation, so a spelling this tool cannot parse is a build failure
        // to fix rather than a reference to miss. That inverts the property
        // from "the scanner understands YAML" — unachievable without a real
        // parser — to "an action reference cannot hide from a text search",
        // which holds for anchors, tags, flow mappings, and every future
        // spelling equally.
        for (line, token) in reference_shaped_tokens(&text) {
            let already = attributed
                .iter()
                .any(|(seen_line, reference)| *seen_line == line && reference.contains(&token));
            if !already {
                violations.push(format!(
                    "{name}:{line}: {token} — an action-reference-shaped token that this scanner \
                     could not attribute to a `uses:` key. Rewrite the step in plain block style \
                     (`uses: owner/repo@<sha> # vX.Y.Z`); node properties, anchors, tags and flow \
                     mappings on the key are not read"
                ));
            }
        }
    }

    if violations.is_empty() {
        println!("verify-actions: {pinned} action reference(s) pinned by digest and tagged");
        Ok(())
    } else {
        Err(TaskError::Policy(format!(
            "SEC-010 requires every GitHub Action to be pinned to a full commit SHA, with the \
             release tag kept in a trailing comment. Offending references:\n  {}",
            violations.join("\n  ")
        )))
    }
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
                if claim.reserved {
                    reservations.push(format!("{package}: {}", claim.pattern));
                } else {
                    violations.push(format!(
                        "{package} claims `{}`, which matches no tracked file; a stale claim \
                         reads as coverage",
                        claim.pattern
                    ));
                }
            }
            if !claim.reserved {
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
    /// From an `owned-paths-reserved` block: matching nothing is expected.
    reserved: bool,
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
        let mut claims = Vec::new();
        let mut inside: Option<bool> = None;
        for line in text.lines() {
            let trimmed = line.trim();
            match inside {
                None => {
                    if trimmed == "```owned-paths" {
                        inside = Some(false);
                    } else if trimmed == "```owned-paths-reserved" {
                        inside = Some(true);
                    }
                }
                Some(reserved) => {
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
                    validate_claim_pattern(&name, pattern)?;
                    claims.push(OwnershipClaim {
                        pattern: pattern.to_owned(),
                        reserved,
                    });
                }
            }
        }
        if inside.is_some() {
            return Err(TaskError::Policy(format!(
                "{name}: an `owned-paths` block is not closed"
            )));
        }
        if claims.is_empty() {
            return Err(TaskError::Policy(format!(
                "{name} declares no owned paths; every work package must state its assignment \
                 in an `owned-paths` block"
            )));
        }
        packages.insert(name, claims);
    }
    Ok(packages)
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

/// Collect every `.yml`/`.yaml` file under `directory`, recursively.
fn yaml_files_under(directory: &Path, found: &mut Vec<PathBuf>) -> Result<(), TaskError> {
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
        if path.is_dir() {
            yaml_files_under(&path, found)?;
        } else if path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
            })
        {
            found.push(path);
        }
    }
    Ok(())
}

/// One `uses:` entry, with the trailing comment the policy requires.
///
/// The comment is carried rather than discarded. It used to be stripped before
/// the check, so the tag half of the rule this tool reports was enforced by
/// nothing: a bare 40-character SHA passed while the error message said a
/// release tag was required. A gate that states a rule it does not apply is
/// worse than one that states nothing, because it is read as evidence.
#[derive(Debug, PartialEq, Eq)]
struct ActionReference {
    line: usize,
    reference: String,
    /// Text after `#` on the same line, trimmed. `None` when absent.
    comment: Option<String>,
    /// Why the scanner could not positively read this `uses` construct.
    ///
    /// `Some` is an automatic violation. The 2026-07-29 project audit rewrote
    /// one pinned step as `"uses": actions/checkout@v7` — valid YAML that
    /// GitHub executes — and the scanner reported success with one *fewer*
    /// reference: the mutable action was invisible rather than rejected. The
    /// scanner now enforces a deliberately small YAML subset and refuses
    /// anything outside it, so an exotic spelling is a failure to fix, never
    /// a reference to miss.
    unrecognized: Option<String>,
}

impl ActionReference {
    /// Why this reference fails SEC-010, or `None` if it satisfies it.
    fn violation(&self) -> Option<String> {
        if let Some(reason) = &self.unrecognized {
            return Some(reason.clone());
        }
        // An action committed to this repository carries no independent supply
        // chain, so it needs neither a digest nor a release tag.
        if self.reference.starts_with("./") {
            return None;
        }
        if !is_pinned(&self.reference) {
            return Some("not pinned to a full commit SHA".to_owned());
        }
        match self.comment.as_deref() {
            None => Some("pinned, but the release tag comment is missing".to_owned()),
            Some(comment) if !names_a_release(comment) => Some(format!(
                "pinned, but {comment:?} does not name a release tag"
            )),
            Some(_) => None,
        }
    }
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

/// Every `owner/repo@ref`-shaped token in a workflow, with its line number.
///
/// Syntax-independent discovery, and the reason the scanner no longer fails
/// open on YAML it cannot parse. A GitHub Action reference must contain this
/// shape verbatim — no anchor, tag, quoting style, or flow mapping changes the
/// reference text — so a sweep for the shape cannot be evaded by spelling the
/// *key* differently.
///
/// Comment-only lines are skipped so documentation may name an action; a
/// reference inside a `run:` script would be reported, which is deliberate
/// over-refusal. The current workflows contain exactly the seven real
/// references and nothing else shaped like one, verified before this was
/// written.
fn reference_shaped_tokens(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        // Strip a trailing comment so a release-tag comment is never mistaken
        // for a second reference.
        let code = trimmed
            .split_once('#')
            .map_or(trimmed, |(before, _)| before);
        for token in code.split(|character: char| {
            !(character.is_ascii_alphanumeric()
                || matches!(character, '.' | '_' | '-' | '/' | '@' | ':'))
        }) {
            // `docker://alpine@sha256:…` and `owner/repo@ref` both qualify; a
            // bare `owner/repo` without `@` does not, since it cannot be a
            // pinned-or-unpinned reference on its own.
            let candidate = token.trim_matches(|c: char| c == ':' || c == '.');
            if candidate.contains('/') && candidate.contains('@') && !candidate.starts_with("./") {
                found.push((index + 1, candidate.to_owned()));
            }
        }
    }
    found
}

/// What one line of workflow YAML says about `uses`.
#[derive(Debug, PartialEq, Eq)]
enum LineReading {
    /// A `uses` reference the scanner positively parsed.
    Reference {
        reference: String,
        comment: Option<String>,
    },
    /// A `uses`-shaped construct outside the enforced subset.
    ///
    /// Always a violation. The alternative — interpreting leniently or
    /// skipping — is what let a quoted key carry a mutable tag past this gate.
    Unrecognized(String),
}

/// Extract every `uses:` entry in a workflow, with its trailing comment.
fn action_references(text: &str) -> Vec<ActionReference> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            action_reference(line).map(|reading| match reading {
                LineReading::Reference { reference, comment } => ActionReference {
                    line: index + 1,
                    reference,
                    comment,
                    unrecognized: None,
                },
                LineReading::Unrecognized(reason) => ActionReference {
                    line: index + 1,
                    reference: line.trim().to_owned(),
                    comment: None,
                    unrecognized: Some(reason),
                },
            })
        })
        .collect()
}

/// Read one line for a `uses` construct.
///
/// The scanner enforces a deliberately small YAML subset: a block-mapping
/// `uses` key — bare, double-quoted, or single-quoted, with optional space
/// before the colon — whose value is a plain or quoted scalar on the same
/// line. Everything else `uses`-shaped is [`LineReading::Unrecognized`], which
/// is an automatic violation. Failing closed on the exotic spellings is the
/// lesson of the 2026-07-29 audit; a scanner that skips what it cannot parse
/// reports one fewer reference instead of one violation, and success with a
/// smaller number is still success.
fn action_reference(line: &str) -> Option<LineReading> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    // YAML explicit-key syntax (`? key`) can spell any key, including `uses`,
    // across two lines. Nothing in a workflow needs it.
    if trimmed.starts_with("? ") || trimmed == "?" {
        return Some(LineReading::Unrecognized(
            "YAML explicit-key syntax is outside the subset this scanner enforces; \
             write plain `key: value` mappings"
                .to_owned(),
        ));
    }
    let mut rest = trimmed;
    if let Some(item) = rest.strip_prefix("- ") {
        rest = item.trim_start();
    }

    // A double-quoted key may contain escapes, so `"u\x73es"` decodes to a key
    // this scanner cannot see without implementing YAML string decoding.
    // Escaped keys have no legitimate use in a workflow; refuse them all.
    if let Some(reason) = escaped_quoted_key(rest) {
        return Some(LineReading::Unrecognized(reason));
    }

    if let Some(value) = uses_key_value(rest) {
        return Some(read_uses_value(value));
    }

    // Flow mappings (`- { name: x, uses: y }`) put the key mid-line, where the
    // block-style reader above cannot see it. Detect and refuse rather than
    // parse: flow YAML has its own quoting and nesting rules, and a partial
    // implementation of them would be this same defect rebuilt.
    flow_style_uses(rest).map(LineReading::Unrecognized)
}

/// The value following a block-style `uses` key, if this line has one.
///
/// Accepts `uses`, `"uses"`, and `'uses'`, each with optional whitespace
/// before the colon — all spellings YAML reads as the same key.
fn uses_key_value(rest: &str) -> Option<&str> {
    for spelling in ["uses", "\"uses\"", "'uses'"] {
        if let Some(after_key) = rest.strip_prefix(spelling) {
            let after_key = after_key.trim_start();
            if let Some(value) = after_key.strip_prefix(':') {
                return Some(value);
            }
        }
    }
    None
}

/// Parse the scalar value of a `uses` key, or refuse the shapes that would
/// need a real YAML parser to read faithfully.
fn read_uses_value(value: &str) -> LineReading {
    let value = value.trim();
    let refuse = |what: &str| {
        LineReading::Unrecognized(format!(
            "`uses` value is {what}, which is outside the subset this scanner enforces; \
             write the reference as a plain scalar on the same line"
        ))
    };
    if value.is_empty() {
        // The reference may continue on the next line as a plain scalar or a
        // nested node. The old scanner skipped this shape, making it a way to
        // hide a reference; a value the scanner cannot see is a violation.
        return refuse("not on the same line as its key");
    }
    match value.as_bytes()[0] {
        b'|' | b'>' => return refuse("a block scalar"),
        b'*' => return refuse("a YAML alias"),
        b'&' => return refuse("anchored"),
        b'{' | b'[' => return refuse("a flow collection"),
        _ => {}
    }
    // The comment is returned rather than dropped. Discarding it here is what
    // made the tag half of SEC-010 unenforceable further up.
    let (value, comment) = match value.split_once('#') {
        Some((before, after)) => (before, Some(after.trim())),
        None => (value, None),
    };
    let value = value.trim().trim_matches(['"', '\'']);
    let comment = comment.map(str::to_owned).filter(|text| !text.is_empty());
    if value.is_empty() {
        return refuse("empty");
    }
    LineReading::Reference {
        reference: value.to_owned(),
        comment,
    }
}

/// A double-quoted mapping key containing a backslash, at the start of this
/// line's content.
fn escaped_quoted_key(rest: &str) -> Option<String> {
    let inner = rest.strip_prefix('"')?;
    let closing = inner.find('"')?;
    let key = &inner[..closing];
    let after = inner[closing + 1..].trim_start();
    (key.contains('\\') && after.starts_with(':')).then(|| {
        format!(
            "quoted mapping key \"{key}\" contains an escape; escaped keys are outside \
             the subset this scanner enforces"
        )
    })
}

/// A `uses` key in flow-mapping position — immediately after `{` or `,`,
/// allowing whitespace and one layer of quotes.
///
/// Prose is deliberately not matched: `run: echo "uses: x"` has no `{` or `,`
/// introducing the token, so scripts that merely mention `uses` stay ignored.
/// A quoted flow key containing an escape is refused for the same reason as at
/// line start.
fn flow_style_uses(rest: &str) -> Option<String> {
    let bytes = rest.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'{' && *byte != b',' {
            continue;
        }
        let mut cursor = index + 1;
        while bytes.get(cursor).is_some_and(|b| *b == b' ' || *b == b'\t') {
            cursor += 1;
        }
        let quote = match bytes.get(cursor) {
            Some(&q @ (b'"' | b'\'')) => {
                cursor += 1;
                Some(q)
            }
            _ => None,
        };
        let key_start = cursor;
        if let Some(quote) = quote {
            while bytes.get(cursor).is_some_and(|b| *b != quote) {
                cursor += 1;
            }
            if cursor >= bytes.len() {
                continue;
            }
            let key = &rest[key_start..cursor];
            cursor += 1;
            while bytes.get(cursor).is_some_and(|b| *b == b' ' || *b == b'\t') {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b':') {
                if key == "uses" {
                    return Some(flow_refusal());
                }
                if key.contains('\\') {
                    return Some(format!(
                        "quoted flow-mapping key {key:?} contains an escape; escaped keys \
                         are outside the subset this scanner enforces"
                    ));
                }
            }
        } else {
            while bytes
                .get(cursor)
                .is_some_and(|b| !matches!(*b, b':' | b' ' | b'\t' | b',' | b'}'))
            {
                cursor += 1;
            }
            let key = &rest[key_start..cursor];
            while bytes.get(cursor).is_some_and(|b| *b == b' ' || *b == b'\t') {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b':') && key == "uses" {
                return Some(flow_refusal());
            }
        }
    }
    None
}

fn flow_refusal() -> String {
    "`uses` appears inside a flow mapping, which is outside the subset this scanner \
     enforces; write the step in block style"
        .to_owned()
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
  cargo xtask tokens             Audit the design tokens for UI-001/007/008
  cargo xtask verify-actions     Verify every GitHub Action is pinned by digest
  cargo xtask verify-licenses    Verify every manifest declares MIT OR Apache-2.0
  cargo xtask verify-ownership   Verify every tracked path belongs to a work package
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
        ActionReference, LineReading, Task, TaskError, action_reference, action_references,
        claim_matches, is_pinned, parse, parse_test, reference_shaped_tokens, repository_root,
        run_tier, validate_claim_pattern, verify_action_pins, verify_manifest_licenses,
        verify_path_ownership,
    };
    use std::ffi::OsString;
    use std::fs;

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
        assert_eq!(found[0].line, 4);
        assert_eq!(
            found[0].reference,
            "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
        );
        assert_eq!(
            found[0].comment.as_deref(),
            Some("v6.0.2"),
            "the trailing comment must be carried, not discarded"
        );
        assert_eq!(found[1].line, 6);
        assert_eq!(
            found[1].reference,
            "actions/setup-node@0000000000000000000000000000000000000000"
        );
        assert_eq!(found[1].comment, None);
        // A `uses` key with no inline value used to be skipped. Its value may
        // continue on the next line, where this scanner cannot see it, so it is
        // now refused rather than ignored.
        assert!(matches!(
            action_reference("        uses:"),
            Some(LineReading::Unrecognized(_))
        ));
    }

    #[test]
    fn a_quoted_uses_key_is_scanned_not_skipped() {
        // The 2026-07-29 audit's live bypass: `"uses"` is the same YAML key as
        // `uses`, GitHub executes it, and the scanner reported success with one
        // fewer reference — the mutable action was invisible, not rejected.
        for spelling in [
            "\"uses\": actions/checkout@v7",
            "'uses': actions/checkout@v7",
            "- \"uses\": actions/checkout@v7",
            "uses : actions/checkout@v7",
        ] {
            let found = action_references(spelling);
            assert_eq!(found.len(), 1, "{spelling:?} must register");
            assert!(
                found[0].violation().is_some(),
                "{spelling:?} carries a mutable tag and must be a violation"
            );
        }

        // And the same spellings with a real pin are accepted, so the fix is
        // recognition, not a blanket ban on quoting.
        let pinned = format!("\"uses\": actions/checkout@{} # v7.0.1", "a".repeat(40));
        let found = action_references(&pinned);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].violation(),
            None,
            "a properly pinned quoted key is fine: {:?}",
            found[0]
        );
    }

    #[test]
    fn an_action_reference_cannot_hide_behind_key_syntax() {
        // The 2026-07-29 follow-up audit's bypass, plus the tag variant it
        // named as the same structural class. Both are valid YAML that GitHub
        // executes, and the key-shaped reader cannot see either — so discovery
        // no longer depends on it. Every `owner/repo@ref` token must be
        // attributable to a `uses:` key the reader parsed; anything else is a
        // violation, which makes an unparseable spelling a build failure
        // rather than a reference that vanishes from the count.
        let sha = "a".repeat(40);
        for spelling in [
            "        &pin uses: actions/checkout@v7",
            "        !!str uses: actions/checkout@v7",
            "        &a2 uses : actions/checkout@v7",
            "        &pin uses: actions/checkout@v7 # v7.0.1",
            // A correctly pinned reference behind an unreadable key is still a
            // violation: the gate cannot confirm what it cannot attribute.
            &format!("        &pin uses: actions/checkout@{sha} # v7.0.1"),
        ] {
            let workflow = format!("jobs:\n  build:\n    steps:\n{spelling}\n");
            let tokens = reference_shaped_tokens(&workflow);
            assert!(
                !tokens.is_empty(),
                "{spelling:?}: the reference must be discovered by shape"
            );
            let attributed: Vec<String> = action_references(&workflow)
                .into_iter()
                .filter(|entry| entry.unrecognized.is_none())
                .map(|entry| entry.reference)
                .collect();
            assert!(
                !tokens
                    .iter()
                    .all(|(_, token)| attributed.iter().any(|r| r.contains(token))),
                "{spelling:?}: an unattributable reference must not be treated as read"
            );
        }
    }

    #[test]
    fn the_reference_sweep_does_not_fire_on_prose_or_comments() {
        // Over-refusal is the safe direction, but not at the cost of being
        // unusable. Comment-only lines and the release-tag comment must not
        // register as references.
        let sha = "a".repeat(40);
        let workflow = format!(
            "jobs:\n  build:\n    steps:\n      # uses: actions/stale@v9\n      - uses: actions/checkout@{sha} # v7.0.1\n"
        );
        let tokens = reference_shaped_tokens(&workflow);
        assert_eq!(
            tokens.len(),
            1,
            "only the real reference should register, found {tokens:?}"
        );
        assert!(tokens[0].1.ends_with(&sha));
    }

    #[test]
    fn the_repository_workflows_have_no_unattributable_references() {
        // The sweep over-refuses by design, so this asserts the design is
        // actually livable on the real workflows: every reference-shaped token
        // in them is attributable to a `uses:` key.
        verify_action_pins(&repository_root())
            .expect("the repository's own workflows must satisfy both discovery paths");
    }

    #[test]
    fn constructs_outside_the_yaml_subset_are_refused_not_skipped() {
        // Each of these is valid YAML that carries (or could carry) a `uses`
        // reference somewhere a line scanner cannot faithfully read. Every one
        // must surface as a violation; silence is how the quoted-key bypass
        // survived. The flow-mapping case is refused even when its pin is
        // valid, because reading flow YAML correctly needs a real parser.
        let sha = "a".repeat(40);
        for construct in [
            format!("- {{ name: x, uses: actions/checkout@{sha} }} # v7.0.1"),
            format!("- {{ \"uses\": actions/checkout@{sha} }} # v7.0.1"),
            "uses: |".to_owned(),
            "uses: >".to_owned(),
            "uses: *pinned_elsewhere".to_owned(),
            format!("uses: &anchor actions/checkout@{sha} # v7.0.1"),
            format!("\"u\\x73es\": actions/checkout@{sha} # v7.0.1"),
            "? uses".to_owned(),
        ] {
            let found = action_references(&construct);
            assert_eq!(found.len(), 1, "{construct:?} must register");
            assert!(
                found[0].unrecognized.is_some(),
                "{construct:?} must be refused as outside the subset"
            );
            assert!(
                found[0].violation().is_some(),
                "an unrecognized construct is always a violation"
            );
        }
    }

    #[test]
    fn prose_that_mentions_uses_still_does_not_register() {
        // Scripts may legitimately talk about actions. The flow detector keys
        // on `{` and `,` in key position, so quoted prose stays prose.
        for prose in [
            "        run: echo \"uses: actions/checkout@v6\"",
            "      # uses: actions/stale@v9",
            "        run: grep uses ci.yml",
        ] {
            assert_eq!(
                action_reference(prose),
                None,
                "{prose:?} is not a reference"
            );
        }
    }

    #[test]
    fn every_repository_manifest_declares_the_project_licence() {
        verify_manifest_licenses(&repository_root())
            .expect("every manifest in this repository declares MIT OR Apache-2.0");
    }

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
    fn composite_action_metadata_is_scanned_when_present() {
        let root =
            std::env::temp_dir().join(format!("partman-xtask-composite-{}", std::process::id()));
        let workflows = root.join(super::WORKFLOW_DIRECTORY);
        fs::create_dir_all(&workflows).expect("create workflow directory");
        fs::write(
            workflows.join("ci.yml"),
            format!(
                "jobs:\n  build:\n    steps:\n      - uses: ./.github/actions/local\n      - uses: actions/checkout@{} # v7.0.1\n",
                "a".repeat(40)
            ),
        )
        .expect("write workflow");

        // The local action passes as `./...` in the workflow — but its own
        // metadata carries a remote, mutable reference. Exempting the local
        // action is safe only because this file is scanned too.
        let action = root.join(super::LOCAL_ACTIONS_DIRECTORY).join("local");
        fs::create_dir_all(&action).expect("create action directory");
        fs::write(
            action.join("action.yml"),
            "runs:\n  using: composite\n  steps:\n    - uses: actions/cache@v4\n",
        )
        .expect("write action metadata");

        let error = verify_action_pins(&root).expect_err("the mutable tag must be found");
        assert!(
            error.to_string().contains("actions/cache@v4"),
            "the violation must name the reference inside the composite action: {error}"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_digest_without_its_release_tag_is_a_violation() {
        // The rule this tool *reports* — "with the release tag kept in a
        // trailing comment" — was enforced by nothing. The comment was stripped
        // before the check, so a bare 40-character SHA passed while the error
        // message claimed a tag was required. A gate that states a rule it does
        // not apply is worse than one that states nothing, because it is read
        // as evidence that the rule holds.
        let sha = "de0fac2e4500dabe0009e67214ff5f5447ce83dd";
        let reference = |comment: Option<&str>| ActionReference {
            line: 1,
            reference: format!("actions/checkout@{sha}"),
            comment: comment.map(str::to_owned),
            unrecognized: None,
        };

        assert_eq!(reference(Some("v6.0.2")).violation(), None);
        assert_eq!(reference(Some("v4")).violation(), None);
        assert_eq!(reference(Some("7.0.1")).violation(), None);
        assert_eq!(
            reference(Some("pinned to v6.0.2 by policy")).violation(),
            None
        );

        for absent in [None, Some(""), Some("pinned"), Some("do not touch")] {
            let comment = absent.filter(|text| !text.is_empty());
            assert!(
                reference(comment).violation().is_some(),
                "a digest with {absent:?} for a tag must be refused"
            );
        }

        // An unpinned reference still fails first, and says so specifically.
        let mutable = ActionReference {
            line: 1,
            reference: "actions/checkout@v6".to_owned(),
            comment: Some("v6".to_owned()),
            unrecognized: None,
        };
        assert!(
            mutable
                .violation()
                .is_some_and(|reason| reason.contains("full commit SHA")),
            "an unpinned reference must fail for being unpinned"
        );

        // A local action has no release to name, so it is exempt from both
        // halves rather than from one.
        let local = ActionReference {
            line: 1,
            reference: "./.github/actions/local".to_owned(),
            comment: None,
            unrecognized: None,
        };
        assert_eq!(local.violation(), None);
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
