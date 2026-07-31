//! Process entry point for ADR-0009's synthetic native feasibility shell.

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let renderer = match partman_desktop::runtime::renderer_from_arguments(&arguments) {
        Ok(renderer) => renderer,
        Err(error) => {
            eprintln!("partman-desktop: {error}");
            return ExitCode::FAILURE;
        }
    };
    match partman_desktop::runtime::run(renderer) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("partman-desktop: {error}");
            ExitCode::FAILURE
        }
    }
}
