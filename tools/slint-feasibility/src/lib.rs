//! Byte-reproducible reporting for ADR-0009's rejected Slint candidate.
//!
//! This is a non-product evidence tool. It reads one fixed ADR, one normalized
//! JSON manifest, and one generated Markdown path. It contains no Slint runtime,
//! storage discovery, helper contact, elevation, network, or device access.

mod error;
mod report;

pub use error::CheckError;
pub use report::{ReportSummary, verify_or_write_report};
