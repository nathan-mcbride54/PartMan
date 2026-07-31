//! Native desktop presentation boundaries for PartMan's Slint feasibility work.
//!
//! It owns the closed English string catalogue, lossless presentation of
//! hostile operating-system identifiers, opaque selection IDs, and the narrow
//! startup boundary for ADR-0009's native Slint feasibility candidate. It
//! performs no storage discovery, launches no process, contacts no helper,
//! requests no elevation, and executes no storage operation.
//!
//! Raw identifiers remain Rust values. Escaped and bounded strings are
//! presentation products only and must never be parsed as device identity or
//! authorization evidence.

pub mod byte_format;
pub mod catalogue;
pub mod identifier_display;
pub mod runtime;
pub mod selection;
pub mod view_model;

#[doc(hidden)]
pub mod generated_ui {
    include!(concat!(env!("OUT_DIR"), "/partman_ui.rs"));
}

#[path = "../build_support/environment.rs"]
mod slint_environment;

#[cfg(not(any(feature = "renderer-femtovg", feature = "renderer-software")))]
compile_error!("PartMan desktop requires exactly one candidate renderer feature");

#[cfg(all(
    feature = "renderer-femtovg",
    feature = "renderer-software",
    not(feature = "comparison-combined")
))]
compile_error!(
    "multiple candidate renderers are permitted only in the non-shipping comparison-combined control"
);
