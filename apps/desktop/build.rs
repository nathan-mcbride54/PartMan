//! Deterministic, fail-closed Slint AOT build boundary for the desktop crate.

#[path = "build_support/aot.rs"]
mod aot;
#[path = "build_support/environment.rs"]
mod slint_environment;

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use aot::{CompileRequest, compile_and_write};
use partman_tokens::TokenSet;
use partman_tokens::slint::render;

fn main() -> Result<(), Box<dyn Error>> {
    slint_environment::guard_current_environment()?;
    emit_environment_invalidation();

    let manifest_directory = required_path_variable("CARGO_MANIFEST_DIR")?;
    let output_directory = required_path_variable("OUT_DIR")?;
    let repository_root = fs::canonicalize(manifest_directory.join("../.."))?;
    let ui_root = manifest_directory.join("ui");
    let root = ui_root.join("main.slint");
    let token_source = repository_root.join("schemas/design-tokens.json");
    let token_contract =
        repository_root.join("packages/design-tokens/generated/partman-tokens.slint");

    for build_input in [
        "Cargo.toml",
        "build.rs",
        "build_support/aot.rs",
        "build_support/environment.rs",
    ] {
        emit_path_invalidation(&manifest_directory.join(build_input));
    }
    emit_path_invalidation(&root);
    emit_path_invalidation(&token_source);
    emit_path_invalidation(&token_contract);
    verify_generated_token_contract(&token_source, &token_contract)?;

    let compiled = compile_and_write(CompileRequest {
        root: &root,
        ui_root: &ui_root,
        token_contract: &token_contract,
        output_directory: &output_directory,
    })?;
    for dependency in compiled.tracked_files() {
        emit_path_invalidation(dependency);
    }
    for resource in compiled.resource_files() {
        emit_path_invalidation(resource);
    }
    if fs::read(compiled.output_path())? != compiled.generated_rust() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the generated Slint Rust read-back differs from validated in-memory bytes",
        )
        .into());
    }
    Ok(())
}

fn required_path_variable(name: &'static str) -> Result<PathBuf, io::Error> {
    std::env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("required Cargo environment name {name} is absent"),
        )
    })
}

fn verify_generated_token_contract(source: &Path, generated: &Path) -> Result<(), Box<dyn Error>> {
    let expected = render(&TokenSet::load(source)?)?;
    let actual = fs::read_to_string(generated)?;
    if actual != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{} is stale relative to {}; run the checked token generator",
                generated.display(),
                source.display()
            ),
        )
        .into());
    }
    Ok(())
}

fn emit_environment_invalidation() {
    for name in slint_environment::KNOWN_SLINT_ENVIRONMENT_NAMES {
        println!("cargo:rerun-if-env-changed={name}");
    }
    println!(
        "cargo:rerun-if-env-changed={}",
        slint_environment::PARTMAN_SLINT_GUARD_NONCE
    );
    println!(
        "cargo:rerun-if-env-changed={}",
        slint_environment::DEP_MCU_EMBED_TEXTURES
    );
}

fn emit_path_invalidation(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
}
