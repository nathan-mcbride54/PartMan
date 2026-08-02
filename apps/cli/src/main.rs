//! The `partman` binary: the thinnest possible shell over
//! [`partman_cli::dispatch_os`], so that every behavior — including the
//! non-Unicode-argument refusal — is testable as pure data in the library.

use std::io::Write as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    let outcome = partman_cli::dispatch_os(std::env::args_os().skip(1));
    // Best-effort emission: a closed pipe (`partman help | head -1`) must
    // not convert an answered command into a panic and an undocumented exit
    // code. The outcome's own code is the contract; the write result is not.
    let _ = std::io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = std::io::stderr().write_all(outcome.stderr.as_bytes());
    ExitCode::from(outcome.code)
}
