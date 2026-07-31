//! Explicit writer/checker for the committed generated Slint token contract.

use std::process::ExitCode;

use partman_tokens::TokenSet;
use partman_tokens::slint::{render, repository_generated_slint_path};

fn main() -> ExitCode {
    match run() {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("generate-slint-tokens: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    let mode = arguments.next().ok_or_else(usage)?;
    if arguments.next().is_some() {
        return Err(usage());
    }
    let mode = mode.to_str().ok_or_else(usage)?;
    if !matches!(mode, "--check" | "--write") {
        return Err(usage());
    }

    let set = TokenSet::load_repository_tokens().map_err(|error| error.to_string())?;
    let expected = render(&set).map_err(|error| error.to_string())?;
    let path = repository_generated_slint_path();

    if mode == "--check" {
        let actual = std::fs::read(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if actual != expected.as_bytes() {
            return Err(format!(
                "{} is stale; regenerate it with --write",
                path.display()
            ));
        }
        return Ok(format!(
            "generated Slint tokens are current: {}",
            path.display()
        ));
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("generated path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    std::fs::write(&path, expected.as_bytes())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(format!("wrote generated Slint tokens: {}", path.display()))
}

fn usage() -> String {
    "usage: generate_slint_tokens --check|--write".to_owned()
}
