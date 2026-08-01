//! CLI for ADR-0009's fixed-path evidence report.

use std::path::PathBuf;
use std::process::ExitCode;

use partman_slint_feasibility::{CheckError, verify_or_write_report};

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("slint-feasibility: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    root: PathBuf,
    write: bool,
}

fn run(arguments: Vec<std::ffi::OsString>) -> Result<(), CheckError> {
    let options = parse_arguments(arguments)?;
    let summary = verify_or_write_report(&options.root, options.write)?;
    println!(
        "report verified: decision={} gates={} raw-evidence-manifest=pce/1:{}",
        summary.decision, summary.gate_count, summary.raw_evidence_manifest_hash
    );
    Ok(())
}

fn parse_arguments(arguments: Vec<std::ffi::OsString>) -> Result<Options, CheckError> {
    let mut arguments = arguments.into_iter();
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("render-report")) {
        return Err(usage());
    }
    let mut root = None;
    let mut write = false;
    while let Some(option) = arguments.next() {
        match option.to_str() {
            Some("--root") => {
                let value = arguments.next().ok_or_else(usage)?;
                if root.replace(PathBuf::from(value)).is_some() {
                    return Err(CheckError::new("duplicate option --root"));
                }
            }
            Some("--write") if !write => write = true,
            _ => return Err(usage()),
        }
    }
    Ok(Options {
        root: root.ok_or_else(|| CheckError::new("--root is required"))?,
        write,
    })
}

fn usage() -> CheckError {
    CheckError::new("usage: partman-slint-feasibility render-report --root ABSOLUTE [--write]")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Options, parse_arguments};

    // Requirements: SEC-010, Section 12
    //   The landed evidence tool accepts one fixed report action, one root, and
    //   one explicit write switch; no candidate runtime or verdict is selectable.
    // Work-Package: WP-030
    // Evidence: evidence_only_report_command_is_fixed_and_narrow
    #[test]
    fn evidence_only_report_command_is_fixed_and_narrow() {
        assert_eq!(
            parse_arguments(
                ["render-report", "--root", "C:/PartMan", "--write"]
                    .into_iter()
                    .map(std::ffi::OsString::from)
                    .collect()
            )
            .expect("fixed report command parses"),
            Options {
                root: PathBuf::from("C:/PartMan"),
                write: true,
            }
        );
        for rejected in [
            vec![],
            vec!["verify-graph"],
            vec!["render-report"],
            vec!["render-report", "--root", "a", "--root", "b"],
            vec!["render-report", "--root", "a", "--result", "pass"],
        ] {
            assert!(
                parse_arguments(rejected.into_iter().map(std::ffi::OsString::from).collect())
                    .is_err()
            );
        }
    }
}
