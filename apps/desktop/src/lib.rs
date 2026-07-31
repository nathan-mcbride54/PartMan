//! Native desktop presentation boundaries for PartMan's Slint feasibility work.
//!
//! This crate currently contains no windowing toolkit. It owns the closed
//! English string catalogue, lossless presentation of hostile operating-system
//! identifiers, and opaque selection IDs that the later Slint adapter will
//! consume. It performs no storage discovery, launches no process, contacts no
//! helper, requests no elevation, and executes no storage operation.
//!
//! Raw identifiers remain Rust values. Escaped and bounded strings are
//! presentation products only and must never be parsed as device identity or
//! authorization evidence.

pub mod catalogue;
pub mod identifier_display;
pub mod selection;
