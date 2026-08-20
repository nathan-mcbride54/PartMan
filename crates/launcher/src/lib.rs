//! The one SAFE-004 launch mechanism: launch a compiled absolute
//! executable with a structured argument array, a cleared child
//! environment (plus `LC_ALL=C`, written, never read), bounded output
//! drained per stream, and a kill at the caller's deadline.
//!
//! This crate is **mechanism only**. Every policy value belongs to the
//! caller: the fixed executable allow-list (SAFE-004's roster is each
//! caller's own), the per-stream output bound, and the deadline — a
//! version probe and a `mkfs` over a large volume are legitimately
//! different lengths, and one constant serving both would be wrong in one
//! direction or the other, which is why the deadline became caller-stated
//! when the mechanism moved here (the launcher-home round, option A,
//! `docs/reviews/LINUX_LAUNCHER_HOME_ROUND_2026-08-20.md`). What this
//! crate does not do is equally deliberate: no `PATH` lookup, no shell,
//! no environment read, no policy constant, no dependency — `apps/cli`'s
//! dependency-closure guard asserts the last so the shipped CLI closure
//! stays hash-free and plan-free transitively.
//!
//! Moved verbatim from `apps/cli/src/doctor.rs` (WP-035's increment 3
//! mechanism, its guarantees unchanged) apart from the caller-stated
//! deadline. SAFE-004's identity carve-out travels with it: existence and
//! launch happen at the trusted absolute path, and what a candidate
//! resolves to — a symlink's target — is executed as that path; identity
//! verification beyond the path (a package record or a recorded content
//! digest, per ADR-0056's discipline) belongs to the caller that owns the
//! roster.

#![forbid(unsafe_code)]

use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// How a launch attempt ended, before any interpretation.
pub enum ProbeOutcome {
    /// The tool exited successfully within the limits; both streams captured.
    Completed {
        /// Bounded bytes from stdout.
        stdout: Vec<u8>,
        /// Bounded bytes from stderr, kept because some tools banner there.
        stderr: Vec<u8>,
    },
    /// The tool exited unsuccessfully within the limits. Its output remains
    /// bounded provenance, but it is never parsed as an answer.
    NonzeroExit {
        /// The numeric exit code, or `None` when the platform supplied none
        /// (for example, termination by a Unix signal).
        code: Option<i32>,
        /// Bounded bytes from stdout.
        stdout: Vec<u8>,
        /// Bounded bytes from stderr.
        stderr: Vec<u8>,
    },
    /// The tool exceeded the caller's deadline and was killed.
    TimedOut,
    /// One tool-output stream produced more than the caller's per-stream
    /// bound and was refused. At most the bound is retained per stream.
    OverOutputLimit,
    /// The launch itself failed.
    LaunchFailed(String),
}

/// The launch seam callers inject.
///
/// Tests inject a fake so Tier 1 never launches a caller's real tool —
/// the tier's process set stays `git`, the compile-time-selected `cargo`,
/// and nothing else. The real implementation is [`SystemLauncher`];
/// Tier 1 exercises it against Git at a reviewed absolute path, already
/// in that set.
pub trait ToolLauncher {
    /// Whether a regular file exists at this compiled absolute path.
    fn exists(&self, path: &Path) -> bool;
    /// Launch one compiled absolute executable with a fixed structured
    /// argument array under the controls described in the crate doc —
    /// cleared environment plus `LC_ALL=C`, no shell, no `PATH` lookup —
    /// with the per-stream output bound and the deadline both stated by
    /// the caller.
    fn launch(
        &self,
        path: &Path,
        arguments: &[&str],
        output_limit: usize,
        deadline: Duration,
    ) -> ProbeOutcome;
}

/// The real launcher: `fs::metadata` for existence, `std::process::Command`
/// with a cleared environment (plus `LC_ALL=C`, written, never read), piped
/// bounded output drained on a thread, and a kill at the caller's deadline.
pub struct SystemLauncher;

impl ToolLauncher for SystemLauncher {
    fn exists(&self, path: &Path) -> bool {
        std::fs::metadata(path).is_ok_and(|m| m.is_file())
    }

    fn launch(
        &self,
        path: &Path,
        arguments: &[&str],
        output_limit: usize,
        deadline: Duration,
    ) -> ProbeOutcome {
        launch_bounded(path, arguments, output_limit, deadline)
    }
}

/// Launch one absolute executable with a structured argument array under
/// the crate's controls and the caller's bounds. Split out privately so
/// Tier 1 can prove both successful and unsuccessful exits without adding
/// another executable class.
fn launch_bounded(
    path: &Path,
    arguments: &[&str],
    output_limit: usize,
    deadline: Duration,
) -> ProbeOutcome {
    let mut command = std::process::Command::new(path);
    command
        .args(arguments)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return ProbeOutcome::LaunchFailed(error.to_string()),
    };

    // Drain both pipes on threads. Each drain keeps reading past the cap
    // (discarding the excess) so a chatty child can flush, exit, and be
    // reported over-output-limit rather than stalling on a full pipe
    // until the deadline mislabels it timed-out. Results come back over
    // channels rather than joins, because a join is unbounded: a
    // descendant process that inherited the pipe keeps it open after the
    // child exits, and the caller must not hang on someone else's
    // daemon — an expired drain window is reported timed-out, and the
    // reader thread dies with the process.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let (stdout_sender, stdout_receiver) = std::sync::mpsc::channel();
    let (stderr_sender, stderr_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = stdout_sender.send(drain_bounded(stdout_pipe, output_limit));
    });
    std::thread::spawn(move || {
        let _ = stderr_sender.send(drain_bounded(stderr_pipe, output_limit));
    });

    let deadline = Instant::now() + deadline;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return ProbeOutcome::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                let _ = child.kill();
                return ProbeOutcome::LaunchFailed(error.to_string());
            }
        }
    };

    let remaining = deadline.saturating_duration_since(Instant::now());
    let Ok((stdout, stdout_overflowed)) = stdout_receiver.recv_timeout(remaining) else {
        return ProbeOutcome::TimedOut;
    };
    let remaining = deadline.saturating_duration_since(Instant::now());
    let Ok((stderr, stderr_overflowed)) = stderr_receiver.recv_timeout(remaining) else {
        return ProbeOutcome::TimedOut;
    };
    if stdout_overflowed || stderr_overflowed {
        return ProbeOutcome::OverOutputLimit;
    }
    if status.success() {
        ProbeOutcome::Completed { stdout, stderr }
    } else {
        ProbeOutcome::NonzeroExit {
            code: status.code(),
            stdout,
            stderr,
        }
    }
}

/// Read up to `limit` bytes from a pipe, then keep draining and discarding
/// so the writer can finish. Returns the bounded bytes and whether the
/// limit was exceeded.
fn drain_bounded(pipe: Option<impl Read>, limit: usize) -> (Vec<u8>, bool) {
    let mut buffer = Vec::new();
    let Some(mut pipe) = pipe else {
        return (buffer, false);
    };
    let cap = u64::try_from(limit).expect("the limit fits");
    let _ = pipe.by_ref().take(cap + 1).read_to_end(&mut buffer);
    let overflowed = buffer.len() > limit;
    if overflowed {
        buffer.truncate(limit);
        let _ = std::io::copy(&mut pipe, &mut std::io::sink());
    }
    (buffer, overflowed)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::Path;
    use std::time::Duration;

    use super::{ProbeOutcome, SystemLauncher, ToolLauncher, drain_bounded};

    /// A bound generous enough for any version banner; the tests' own
    /// policy value, exactly as a caller would state one.
    const TEST_OUTPUT_LIMIT: usize = 4096;
    /// A deadline generous enough for `git --version`; the tests' own
    /// policy value.
    const TEST_DEADLINE: Duration = Duration::from_secs(5);

    #[cfg(windows)]
    const TEST_GIT: &[&str] = &[
        r"C:\Program Files\Git\cmd\git.exe",
        r"C:\Program Files\Git\bin\git.exe",
    ];
    #[cfg(target_os = "linux")]
    const TEST_GIT: &[&str] = &["/usr/bin/git", "/bin/git", "/usr/local/bin/git"];
    #[cfg(target_os = "macos")]
    const TEST_GIT: &[&str] = &[
        "/Library/Developer/CommandLineTools/usr/bin/git",
        "/Applications/Xcode.app/Contents/Developer/usr/bin/git",
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "/usr/bin/git",
    ];

    fn test_git() -> &'static Path {
        TEST_GIT
            .iter()
            .map(Path::new)
            .find(|path| path.is_file())
            .expect("Tier 1 requires Git at one reviewed absolute path")
    }

    // Requirements: SAFE-004
    //   The real launcher completes a successful bounded launch at a
    //   compiled absolute path and captures both streams within the
    //   caller's bound
    // Evidence: a_successful_launch_completes_with_bounded_streams
    #[test]
    fn a_successful_launch_completes_with_bounded_streams() {
        match SystemLauncher.launch(test_git(), &["--version"], TEST_OUTPUT_LIMIT, TEST_DEADLINE) {
            ProbeOutcome::Completed { stdout, stderr } => {
                assert!(!stdout.is_empty(), "git --version banners on stdout");
                assert!(stdout.len() <= TEST_OUTPUT_LIMIT);
                assert!(stderr.len() <= TEST_OUTPUT_LIMIT);
            }
            _ => panic!("git --version at a reviewed absolute path must complete"),
        }
    }

    // Requirements: SAFE-004
    //   An unsuccessful process exit is distinguished from a successful
    //   answer, with bounded output retained as provenance only
    // Evidence: a_nonzero_exit_is_reported_as_failure_with_bounded_provenance
    #[test]
    fn a_nonzero_exit_is_reported_as_failure_with_bounded_provenance() {
        match SystemLauncher.launch(
            test_git(),
            &["--partman-intentional-invalid-option"],
            TEST_OUTPUT_LIMIT,
            TEST_DEADLINE,
        ) {
            ProbeOutcome::NonzeroExit {
                code,
                stdout,
                stderr,
            } => {
                let code = code.expect("Git's ordinary invalid-option exit has a numeric code");
                assert_ne!(code, 0);
                assert!(stdout.len() <= TEST_OUTPUT_LIMIT);
                assert!(stderr.len() <= TEST_OUTPUT_LIMIT);
                assert!(
                    !(stdout.is_empty() && stderr.is_empty()),
                    "Git's refusal carries provenance"
                );
            }
            _ => panic!("an invalid Git option must be a completed nonzero exit"),
        }
    }

    // Requirements: SAFE-004
    //   The per-stream output bound is the caller's: a launch whose output
    //   exceeds a one-byte bound is refused over-output-limit, never
    //   silently truncated into a success
    // Evidence: the_callers_output_bound_is_enforced
    #[test]
    fn the_callers_output_bound_is_enforced() {
        match SystemLauncher.launch(test_git(), &["--version"], 1, TEST_DEADLINE) {
            ProbeOutcome::OverOutputLimit => {}
            _ => panic!("a one-byte bound must refuse git --version as over-output-limit"),
        }
    }

    // Requirements: SAFE-004
    //   The deadline is the caller's: a launch whose deadline has already
    //   expired at entry is killed and reported timed-out, never allowed
    //   to run on some constant of the mechanism's own
    // Evidence: the_callers_deadline_is_enforced
    #[test]
    fn the_callers_deadline_is_enforced() {
        // A zero deadline is expired before the child can plausibly exit:
        // the first wait poll happens within microseconds of the spawn,
        // long before an exec completes, so this is deterministic in
        // practice while spawning no deliberately hanging process (the
        // cost the original increment declined and recorded).
        match SystemLauncher.launch(
            test_git(),
            &["--version"],
            TEST_OUTPUT_LIMIT,
            Duration::ZERO,
        ) {
            ProbeOutcome::TimedOut => {}
            _ => panic!("an already-expired deadline must be reported timed-out"),
        }
    }

    // Requirements: SAFE-004
    //   A launch that cannot start is its own outcome, never confused with
    //   a tool's answer; existence is a plain regular-file check at the
    //   absolute path
    // Evidence: a_missing_executable_fails_the_launch_and_the_existence_check
    #[test]
    fn a_missing_executable_fails_the_launch_and_the_existence_check() {
        let missing = Path::new("/partman-test/does-not-exist/tool");
        assert!(!SystemLauncher.exists(missing));
        assert!(SystemLauncher.exists(test_git()));
        match SystemLauncher.launch(missing, &["--version"], TEST_OUTPUT_LIMIT, TEST_DEADLINE) {
            ProbeOutcome::LaunchFailed(_) => {}
            _ => panic!("a missing executable must be a launch failure"),
        }
    }

    // Requirements: SAFE-004
    //   The drain keeps exactly the bound, reports overflow, and lets the
    //   writer finish
    // Evidence: the_drain_is_bounded_and_reports_overflow
    #[test]
    fn the_drain_is_bounded_and_reports_overflow() {
        let (at_limit, overflowed) = drain_bounded(Some(Cursor::new(vec![b'x'; 8])), 8);
        assert_eq!(at_limit.len(), 8);
        assert!(!overflowed);

        let (truncated, overflowed) = drain_bounded(Some(Cursor::new(vec![b'x'; 9])), 8);
        assert_eq!(truncated.len(), 8);
        assert!(overflowed);

        let (empty, overflowed) = drain_bounded(None::<Cursor<Vec<u8>>>, 8);
        assert!(empty.is_empty());
        assert!(!overflowed);
    }

    // Requirements: SAFE-004
    //   The mechanism owns exactly one process constructor, so every
    //   launch in this crate flows through the bounded, sanitized path —
    //   the structural pin the CLI's source guard holds from the other
    //   side (zero constructors there)
    // Evidence: the_mechanism_owns_exactly_one_process_constructor
    #[test]
    fn the_mechanism_owns_exactly_one_process_constructor() {
        let source = include_str!("lib.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("the production source precedes this test module");
        assert_eq!(
            production.matches("std::process::Command::new").count(),
            1,
            "the mechanism owns exactly one direct process constructor"
        );
    }
}
