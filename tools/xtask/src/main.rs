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

/// Every `FROM` base image in a Dockerfile that is not pinned by digest.
///
/// A Docker action with `image: Dockerfile` builds from source in the action
/// directory, so the base images in that Dockerfile are the executable
/// dependency. `FROM x AS builder` and `FROM --platform=… x` are both accepted
/// spellings; a stage name defined earlier in the same file is an internal
/// reference and not a pull.
fn unpinned_dockerfile_bases(text: &str) -> Vec<String> {
    let mut unpinned = Vec::new();
    let mut stages: BTreeSet<String> = BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some(rest) = trimmed
            .strip_prefix("FROM ")
            .or_else(|| trimmed.strip_prefix("from "))
        else {
            continue;
        };
        let mut tokens = rest
            .split_whitespace()
            .filter(|token| !token.starts_with("--"));
        let Some(base) = tokens.next() else { continue };
        // `FROM base AS name` defines a stage that later FROMs may reference.
        let mut remaining = tokens;
        if remaining
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("as"))
            && let Some(name) = remaining.next()
        {
            stages.insert(name.to_owned());
        }
        if stages.contains(base) || base.starts_with('$') {
            continue;
        }
        if image_violation(base).is_some() {
            unpinned.push(base.to_owned());
        }
    }
    unpinned
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
            // it. Counting it as coverage would let a package be declared
            // generated and then be claimed by nobody.
            if claim.kind == ClaimKind::Owned {
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
    let changed: Vec<String> = git(root, &["diff", "--name-only", &range])?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();

    if changed.is_empty() {
        println!("verify-change-ownership: no paths changed against {base}");
        return Ok(());
    }

    let messages = git(root, &["log", "--format=%B%x00", &format!("{base}..HEAD")])?;
    let commits: Vec<&str> = messages
        .split('\0')
        .map(str::trim)
        .filter(|body| !body.is_empty())
        .collect();
    if commits.is_empty() {
        return Err(TaskError::Policy(format!(
            "{} path(s) differ from {base} but no commit does so; refusing to guess which work \
             package owns the change",
            changed.len()
        )));
    }

    let (declared, governance) = read_declarations(&commits);

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

    // Declared by *any* package, because "this file is generated" is a property
    // of the file, not a privilege of one assignment.
    let derived: Vec<&str> = catalogue
        .values()
        .flatten()
        .filter(|claim| claim.derived())
        .map(|claim| claim.pattern.as_str())
        .collect();

    // Every lockfile that exists, so a manifest is matched to the one that
    // actually resolves it rather than to the outermost declaration.
    let at_base = git(root, &["ls-tree", "-r", "--name-only", base])?;
    let lockfiles: Vec<&str> = at_base
        .lines()
        .map(str::trim)
        .chain(changed.iter().map(String::as_str))
        .filter(|path| is_named(path, "Cargo.lock"))
        .collect();

    let (strays, regenerated) = classify(&changed, claims, &derived, &lockfiles);

    if !strays.is_empty() {
        return Err(stray_paths(&package, base, &strays, &derived));
    }
    println!(
        "verify-change-ownership: {} path(s) all belong to {package} as assigned at {base}",
        changed.len()
    );
    for path in &regenerated {
        println!("  regenerated, not authored: {path}");
    }
    Ok(())
}

/// Split the changed paths into the ones this assignment cannot account for and
/// the ones it regenerated rather than authored.
fn classify<'a>(
    changed: &'a [String],
    claims: &[OwnershipClaim],
    derived: &[&str],
    lockfiles: &[&str],
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
        if derived.iter().any(|pattern| {
            claim_matches(pattern, path) && derivation_is_plausible(path, changed, lockfiles)
        }) {
            regenerated.push(path.as_str());
        } else {
            strays.push(path.clone());
        }
    }
    (strays, regenerated)
}

/// The refusal, naming every stray and why a generated one was still refused.
fn stray_paths(package: &str, base: &str, strays: &[String], derived: &[&str]) -> TaskError {
    let alone: Vec<&str> = strays
        .iter()
        .map(String::as_str)
        .filter(|path| derived.iter().any(|pattern| claim_matches(pattern, path)))
        .collect();
    let note = if alone.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n{} of these are generated files, and a generated file moving on its own is not \
             regeneration — nothing in this change asks the generator for a different answer. \
             Change the manifest the lockfile resolves, or let the package that owns the lockfile \
             make the pin:\n  {}",
            alone.len(),
            alone.join("\n  ")
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

/// The `Work-Package:` and `Governance:` trailers across a range of commits.
fn read_declarations(commits: &[&str]) -> (BTreeSet<String>, Vec<String>) {
    let mut declared = BTreeSet::new();
    let mut governance = Vec::new();
    for body in commits {
        for line in body.lines().map(str::trim) {
            if let Some(value) = line.strip_prefix("Work-Package:") {
                declared.insert(value.trim().to_owned());
            } else if let Some(reason) = line.strip_prefix("Governance:") {
                governance.push(reason.trim().to_owned());
            }
        }
    }
    (declared, governance)
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
    if Path::new(pattern).file_name().and_then(OsStr::to_str) == Some("Cargo.lock") {
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
/// The manifest must also be one this lockfile actually resolves. The first
/// version of this rule accepted any `Cargo.toml` anywhere, and attacking it
/// found the hole immediately: `fuzz/` is *excluded* from the root workspace and
/// carries its own lockfile, so editing `fuzz/Cargo.toml` cannot change the root
/// `Cargo.lock` — yet it would have unlocked it. A manifest is therefore matched
/// to the nearest declared lockfile above it, longest directory prefix winning.
///
/// **What this does not establish:** a re-pin travelling *alongside* a genuine
/// manifest change passes. Distinguishing the two needs the resolver's answer at
/// both revisions, which means base's whole tree and a full resolution on every
/// pull request. The residual risk is the same one the repository has always
/// carried — nothing here makes it worse — and `cargo deny`, `cargo audit` and
/// owner review are what stand against it. Recorded in
/// `docs/quality/dependency-policy.md` rather than implied to be covered.
fn derivation_is_plausible(derived: &str, changed: &[String], lockfiles: &[&str]) -> bool {
    if !is_named(derived, "Cargo.lock") {
        return false;
    }
    changed
        .iter()
        .filter(|path| is_named(path, "Cargo.toml"))
        .any(|manifest| governing_lockfile(manifest, lockfiles) == Some(derived))
}

/// The lockfile nearest above `manifest`, longest prefix winning.
///
/// The candidates are the lockfiles that **exist**, not the ones declared
/// derived. Reading them from the tree is what makes the nesting real: if
/// `fuzz/Cargo.lock` were merely undeclared, matching `fuzz/Cargo.toml` against
/// the root lock would reopen the hole this rule was tightened to close. An
/// undeclared nested lockfile means a manifest under it can carry nothing.
fn governing_lockfile<'a>(manifest: &str, lockfiles: &[&'a str]) -> Option<&'a str> {
    lockfiles
        .iter()
        .filter(|lockfile| is_named(lockfile, "Cargo.lock"))
        .filter(|lockfile| {
            let directory = lockfile
                .rsplit_once('/')
                .map_or(String::new(), |(parent, _)| format!("{parent}/"));
            manifest.starts_with(&directory)
        })
        .max_by_key(|lockfile| lockfile.len())
        .copied()
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
        Task, TaskError, claim_matches, derivation_is_plausible, governing_lockfile, is_pinned,
        parse, parse_test, repository_root, run_tier, validate_claim_pattern,
        validate_derived_pattern, verify_action_pins, verify_change_ownership,
        verify_manifest_licenses, verify_path_ownership,
    };
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

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

        let _ = fs::remove_dir_all(&root);
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

    #[test]
    fn the_repository_workflows_pass_the_real_gate() {
        verify_action_pins(&repository_root())
            .expect("this repository's own workflows must satisfy the gate");
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
            error
                .to_string()
                .contains("no commit declares a work package")
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
        repo.write("crates/fixtures/Cargo.toml", "# a dependency was added\n");
        repo.write("Cargo.lock", "# regenerated\n");
        repo.commit("add a dependency\n\nWork-Package: WP-020");
        repo.check()
            .expect("a lockfile that follows a manifest this package owns is regeneration");

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

        // A manifest in a nested workspace does not unlock the root lockfile,
        // because it cannot change it. The first version of this rule accepted
        // any `Cargo.toml` anywhere and would have passed this.
        let repo = GitFixture::new("derived-nested");
        repo.write("nested/Cargo.toml", "# a fuzz dependency was added\n");
        repo.write("Cargo.lock", "# and the root lock moved too\n");
        repo.commit("edit the nested workspace\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("a nested manifest cannot vouch for the root lockfile");
        assert!(error.to_string().contains("Cargo.lock"));

        // And the exemption is load-bearing: without the `derived-paths`
        // declaration the accepted case above goes back to being refused. This
        // is the deletion sweep -- a check that cannot fail is not a check.
        let repo = GitFixture::new_without_derived_declaration("underived");
        repo.write("crates/fixtures/Cargo.toml", "# a dependency was added\n");
        repo.write("Cargo.lock", "# regenerated\n");
        repo.commit("add a dependency\n\nWork-Package: WP-020");
        let error = repo
            .check()
            .expect_err("undeclared, the lockfile is WP-000's alone");
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

        // The plausibility rule itself: a manifest in the change, or nothing.
        let lockfiles = ["Cargo.lock", "fuzz/Cargo.lock"];
        let manifest = vec!["crates/fixtures/Cargo.toml".to_owned()];
        assert!(derivation_is_plausible("Cargo.lock", &manifest, &lockfiles));
        assert!(!derivation_is_plausible("Cargo.lock", &[], &lockfiles));
        // A path merely *ending* in the right word is not a manifest.
        let decoy = vec!["docs/quality/Cargo.toml.md".to_owned()];
        assert!(!derivation_is_plausible("Cargo.lock", &decoy, &lockfiles));

        // The hole found by attacking the first version of this rule: `fuzz/` is
        // excluded from the root workspace and has its own lockfile, so editing
        // its manifest cannot change the root lock and must not unlock it.
        let fuzz = vec!["fuzz/Cargo.toml".to_owned()];
        assert!(
            !derivation_is_plausible("Cargo.lock", &fuzz, &lockfiles),
            "a manifest in a nested workspace must not vouch for the root lockfile"
        );
        assert!(derivation_is_plausible(
            "fuzz/Cargo.lock",
            &fuzz,
            &lockfiles
        ));
        assert_eq!(
            governing_lockfile("fuzz/Cargo.toml", &lockfiles),
            Some("fuzz/Cargo.lock"),
            "the nearest lockfile above a manifest governs it, not the outermost"
        );
        // The candidates are the lockfiles that exist. A manifest with no
        // lockfile above it at all vouches for nothing.
        assert!(!derivation_is_plausible("Cargo.lock", &manifest, &[]));
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
            Self::build(tag, true)
        }

        /// The same catalogue with WP-000's `derived-paths` block removed, so a
        /// test can watch the exemption stop working.
        fn new_without_derived_declaration(tag: &str) -> Self {
            Self::build(tag, false)
        }

        fn build(tag: &str, derived: bool) -> Self {
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
            let derived_block = if derived {
                "\n```derived-paths\nCargo.lock\n```\n"
            } else {
                ""
            };
            fixture.write(
                "docs/work-packages/WP-000.md",
                &format!(
                    "# WP-000\n\n```owned-paths\ntools/xtask/**\nCargo.toml\nCargo.lock\n\
                     docs/work-packages/WP-000.md\n```\n{derived_block}"
                ),
            );
            fixture.write(
                "docs/work-packages/WP-020.md",
                "# WP-020\n\n```owned-paths\ndocs/work-packages/WP-020.md\ncrates/fixtures/**\n\
                 nested/**\n```\n",
            );
            fixture.write("Cargo.toml", "# base\n");
            fixture.write("Cargo.lock", "# base\n");
            fixture.write("crates/fixtures/Cargo.toml", "# base\n");
            // A workspace excluded from the root one, with its own lockfile --
            // `fuzz/` in the real repository.
            fixture.write("nested/Cargo.toml", "# base\n");
            fixture.write("nested/Cargo.lock", "# base\n");
            fixture.write("tools/xtask/src/main.rs", "// base\n");
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
