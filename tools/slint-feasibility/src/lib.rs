//! Fail-closed static and replay checks for ADR-0009's bounded Slint candidate.
//!
//! This crate is a developer evidence tool, not product runtime code. It reads
//! Cargo metadata and registry source trees; it never discovers storage,
//! launches a helper, requests elevation, or mutates a device. The live CLI
//! invokes only the compile-time-selected, identity-checked Cargo executable
//! with fixed, structured metadata arguments and bounded output.

mod environment;
mod error;
mod graph;
mod metadata;
mod process;
mod source;

pub use environment::{EnvironmentInventory, verify_environment_inventory};
pub use error::CheckError;
pub use graph::{GraphConfiguration, GraphPhase, GraphReport, TargetContext, verify_graph};
pub use metadata::CargoMetadata;
pub use process::load_or_collect_metadata;
pub use source::{SourceReport, verify_source};
