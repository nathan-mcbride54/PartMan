//! CLI for fail-closed ADR-0009 static and replay checks.

use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use partman_slint_feasibility::{
    CheckError, GraphPhase, load_or_collect_metadata, verify_environment_inventory, verify_graph,
    verify_source,
};

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("slint-feasibility: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Source,
    Graph,
    EnvironmentInventory,
    All,
}

struct Options {
    command: Command,
    metadata: Option<PathBuf>,
    manifest: Option<PathBuf>,
    phase: GraphPhase,
}

fn run(arguments: Vec<std::ffi::OsString>) -> Result<(), CheckError> {
    let options = parse_arguments(arguments)?;
    let metadata = load_or_collect_metadata(
        options.metadata.as_deref(),
        options.manifest.as_deref(),
        options.phase,
    )?;
    if matches!(options.command, Command::Source | Command::All) {
        let report = verify_source(&metadata)?;
        println!(
            "source verified: i-slint-compiler {} commit {} tree {} ({} files)",
            report.compiler_version,
            report.tag_commit,
            report.published_tree_sha256,
            report.file_count
        );
    }
    if matches!(options.command, Command::Graph | Command::All) {
        let report = verify_graph(&metadata, options.phase)?;
        println!(
            "graph verified: phase={} host-packages={} target-packages={} final-runtime-proven={} conditional-edges-conservatively-included={} lockfile-only-advisories={:?}",
            report.phase,
            report.host_package_count,
            report.target_package_count,
            report.final_runtime_proven,
            report.conservatively_included_predicates,
            report.lockfile_only_advisories
        );
    }
    if matches!(
        options.command,
        Command::EnvironmentInventory | Command::All
    ) {
        let report = verify_environment_inventory(&metadata)?;
        println!(
            "environment inventory verified: resolved-names={} rejected-rerun-names={} upstream-controlled-names={}",
            report.resolved_names.len(),
            report.rejected_rerun_names.len(),
            report.upstream_controlled_names.len()
        );
    }
    Ok(())
}

fn parse_arguments(arguments: Vec<std::ffi::OsString>) -> Result<Options, CheckError> {
    let mut arguments = arguments.into_iter();
    let command = match arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        Some("verify-source") => Command::Source,
        Some("verify-graph") => Command::Graph,
        Some("verify-environment-inventory") => Command::EnvironmentInventory,
        Some("verify-all") => Command::All,
        _ => return Err(usage()),
    };
    let mut metadata = None;
    let mut manifest = None;
    let mut phase = None;
    while let Some(option) = arguments.next() {
        let option = option.into_string().map_err(|_| usage())?;
        let value = arguments.next().ok_or_else(usage)?;
        match option.as_str() {
            "--metadata" => set_once(&mut metadata, PathBuf::from(value), "--metadata")?,
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), "--manifest")?,
            "--phase" => {
                let value = value
                    .into_string()
                    .map_err(|_| CheckError::new("--phase is not Unicode"))?;
                set_once(&mut phase, GraphPhase::from_str(&value)?, "--phase")?;
            }
            _ => return Err(usage()),
        }
    }
    let phase = phase.ok_or_else(|| CheckError::new("--phase is required"))?;
    match (&metadata, &manifest) {
        (Some(_), None) | (None, Some(_)) => {}
        _ => return Err(usage()),
    }
    Ok(Options {
        command,
        metadata,
        manifest,
        phase,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<(), CheckError> {
    if slot.replace(value).is_some() {
        return Err(CheckError::new(format!("duplicate option {name}")));
    }
    Ok(())
}

fn usage() -> CheckError {
    CheckError::new(
        "usage: partman-slint-feasibility <verify-source|verify-graph|verify-environment-inventory|verify-all> --phase <compiler-only|final-runtime> (--metadata FILE | --manifest ABSOLUTE)",
    )
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_arguments};

    // Requirements: SAFE-004, SEC-010
    //   The CLI accepts only narrow explicit subcommands, one input mode, and an explicit proof phase
    // Evidence: command_line_contract_is_narrow_and_explicit
    #[test]
    fn command_line_contract_is_narrow_and_explicit() {
        let options = parse_arguments(
            [
                "verify-all",
                "--phase",
                "compiler-only",
                "--metadata",
                "metadata.json",
            ]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect(),
        )
        .expect("explicit replay command parses");
        assert_eq!(options.command, Command::All);
        for rejected in [
            vec!["verify-all", "--metadata", "metadata.json"],
            vec![
                "verify-all",
                "--phase",
                "compiler-only",
                "--metadata",
                "a",
                "--manifest",
                "b",
            ],
            vec![
                "verify-all",
                "--phase",
                "compiler-only",
                "--cargo",
                "C:/substituted/cargo.exe",
                "--manifest",
                "C:/workspace/Cargo.toml",
            ],
            vec!["unknown", "--phase", "compiler-only", "--metadata", "a"],
        ] {
            assert!(
                parse_arguments(rejected.into_iter().map(std::ffi::OsString::from).collect())
                    .is_err()
            );
        }
    }
}
