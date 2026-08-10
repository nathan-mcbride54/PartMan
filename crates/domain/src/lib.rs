//! Canonical domain model for PartMan.
//!
//! This crate owns the types that every other crate agrees on, and the one
//! byte encoding used to hash them. It performs no I/O, launches no process,
//! and never touches storage.

pub mod canonical;
pub mod model;
