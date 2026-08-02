//! The dependency doctor (CAP-004): which external tools this host offers,
//! at which versions, resolved from compiled absolute paths only.
//!
//! SAFE-004's launch discipline, adopted in every clause but two, and the
//! two are named rather than implied: structured argument arrays, a fixed
//! executable allow-list (the roster *is* the allow-list), trusted absolute
//! locations and never a user-controlled `PATH`, bounded output, a time
//! limit, and a sanitized child environment — but **executable identity is
//! not verified beyond the trusted absolute path**: presence and the version
//! banner are checked, no digest or signature is, and a symlinked candidate
//! executes whatever it resolves to. Identity verification arrives with the
//! packages that execute tools against storage, where SAFE-004 demands it;
//! a `--version` probe records that gap instead of claiming it closed. The
//! second carve-out: mapping an out-of-range version to a `blocked`
//! capability is SAFE-004's own last clause, and that mapping belongs to
//! WP-050's capability engine, not here. The doctor reports presence,
//! version, and tested-range membership as **facts with provenance** — which
//! path answered, what the tool printed.
//!
//! **This module is why the shipped binary's I/O statement changed in
//! increment 3.** Its exact reach: existence checks (`fs::metadata`) and
//! `--version` launches of roster tools at compiled absolute paths, nothing
//! else. No device is opened, no path outside the roster is touched, no
//! environment variable is read (`env_clear` writes the child's environment;
//! it reads nothing).
//!
//! The version banner is parsed by hand because util-linux offers no
//! structured `--version` output — Section 16 forbids parsing localized
//! output only *where structured output exists* — and the tiny parser is
//! duplicated from `crates/fixtures`' prober rather than imported, because
//! importing it would breach the empty shipped dependency closure that
//! keeps hash and plan implementations out of this binary's reach.

use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::{Refusal, json_escaped};

/// One tool the roster knows how to look for.
pub struct ToolSpec {
    /// The executable's base name, for reporting.
    pub name: &'static str,
    /// Why PartMan will want it, in one clause.
    pub role: &'static str,
    /// Compiled absolute candidate paths, probed in order. This list is the
    /// SAFE-004 allow-list: no path outside it is ever probed, and no
    /// `PATH` entry can add to it. What a candidate resolves to — a
    /// symlink's target — is executed as that path; see the module doc's
    /// identity carve-out.
    pub candidates: &'static [&'static str],
    /// The version this repository has recorded expectations against.
    pub tested: TestedVersion,
}

/// The tested range, stated as exactly what it is: one measured build
/// extended to its patch family. The prober measured libblkid 2.41.0 on
/// 2026-07-28, and `crates/fixtures`' prober records why the family
/// extension is safe there — a patch release has not changed one of the
/// recorded answers — so `within-tested-range` for an unmeasured 2.41.x
/// means "in the measured build's patch family", never "measured".
pub struct TestedVersion {
    /// Human name of the recorded version, e.g. `util-linux 2.41`.
    pub label: &'static str,
    /// The recorded major.minor pair a parsed version is compared against.
    pub family: (u32, u32),
}

/// The per-platform roster.
///
/// Linux carries the two tools whose behavior this repository has recorded
/// expectations for (`crates/fixtures`' prober measured util-linux 2.41).
/// Windows and macOS are deliberately empty: their inventory routes are
/// native APIs whose adapters arrive with WP-W100 and WP-M100, and an empty
/// roster is reported as a typed state — never as "all dependencies
/// satisfied", which would be a pass for a run of nothing.
#[cfg(target_os = "linux")]
pub const ROSTER: &[ToolSpec] = &[
    ToolSpec {
        name: "blkid",
        role: "partition-table and signature probing, read-only",
        candidates: &["/usr/sbin/blkid", "/sbin/blkid", "/usr/bin/blkid"],
        tested: TestedVersion {
            label: "util-linux 2.41",
            family: (2, 41),
        },
    },
    ToolSpec {
        name: "wipefs",
        role: "signature enumeration, read-only under -n",
        candidates: &["/usr/sbin/wipefs", "/sbin/wipefs", "/usr/bin/wipefs"],
        tested: TestedVersion {
            label: "util-linux 2.41",
            family: (2, 41),
        },
    },
];

/// See the Linux roster's doc comment: empty is a typed state, not a pass.
#[cfg(not(target_os = "linux"))]
pub const ROSTER: &[ToolSpec] = &[];

/// The typed statement an empty roster carries in place of tool reports.
#[cfg(not(target_os = "linux"))]
const EMPTY_ROSTER: Refusal = Refusal {
    state: "not-implemented",
    reference: if cfg!(target_os = "windows") {
        "WP-W100"
    } else {
        "WP-M100"
    },
    detail: "this platform's inventory route is a native API, not an external tool; its \
             adapter work package registers what the doctor must check here, and an empty \
             roster must not be read as all dependencies satisfied",
};

/// How long one `--version` launch may run before it is killed.
const LAUNCH_TIME_LIMIT: Duration = Duration::from_secs(5);

/// How many output bytes one launch may produce before the rest is refused.
/// Section 16 forbids logging raw tool output without size limits; a
/// version banner is one line, so the bound is generous, not permissive.
const OUTPUT_LIMIT: usize = 4096;

/// How a launch attempt ended, before any interpretation.
pub enum ProbeOutcome {
    /// The tool ran to completion within the limits; both streams captured.
    Completed {
        /// Bounded bytes from stdout.
        stdout: Vec<u8>,
        /// Bounded bytes from stderr, kept because some tools banner there.
        stderr: Vec<u8>,
    },
    /// The tool exceeded [`LAUNCH_TIME_LIMIT`] and was killed.
    TimedOut,
    /// The tool produced more than [`OUTPUT_LIMIT`] bytes and was refused.
    OverOutputLimit,
    /// The launch itself failed.
    LaunchFailed(String),
}

/// What the doctor needs from the operating system, as a seam.
///
/// Tests inject a fake so Tier 1 never launches a roster tool — the tier's
/// process set stays `git`, the compile-time-selected `cargo`, and nothing
/// else. The real implementation is [`SystemLauncher`], and the one Tier-1
/// test that exercises it probes the compile-time-selected `cargo` itself,
/// which is already in that set.
pub trait ToolLauncher {
    /// Whether a regular file exists at this compiled absolute path.
    fn exists(&self, path: &Path) -> bool;
    /// Launch `path --version` under the SAFE-004 discipline and report
    /// how it ended.
    fn probe_version(&self, path: &Path) -> ProbeOutcome;
}

/// The real launcher: `fs::metadata` for existence, `std::process::Command`
/// with a cleared environment (plus `LC_ALL=C`, written, never read), piped
/// bounded output drained on a thread, and a kill at the deadline.
///
/// The time-limit path is exercised by review and by manual probe, not by a
/// Tier-1 test — proving it would mean spawning a deliberately hanging
/// process, a cost this increment declines and records rather than hides.
pub struct SystemLauncher;

impl ToolLauncher for SystemLauncher {
    fn exists(&self, path: &Path) -> bool {
        std::fs::metadata(path).is_ok_and(|m| m.is_file())
    }

    fn probe_version(&self, path: &Path) -> ProbeOutcome {
        let mut command = std::process::Command::new(path);
        command
            .arg("--version")
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
        // child exits, and the doctor must not hang on someone else's
        // daemon — an expired drain window is reported timed-out, and the
        // reader thread dies with the process.
        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();
        let (stdout_sender, stdout_receiver) = std::sync::mpsc::channel();
        let (stderr_sender, stderr_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = stdout_sender.send(drain_bounded(stdout_pipe));
        });
        std::thread::spawn(move || {
            let _ = stderr_sender.send(drain_bounded(stderr_pipe));
        });

        let deadline = Instant::now() + LAUNCH_TIME_LIMIT;
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => break,
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
        }

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
        ProbeOutcome::Completed { stdout, stderr }
    }
}

/// Read up to [`OUTPUT_LIMIT`] bytes from a pipe, then keep draining and
/// discarding so the writer can finish. Returns the bounded bytes and
/// whether the limit was exceeded.
fn drain_bounded(pipe: Option<impl Read>) -> (Vec<u8>, bool) {
    let mut buffer = Vec::new();
    let Some(mut pipe) = pipe else {
        return (buffer, false);
    };
    let cap = u64::try_from(OUTPUT_LIMIT).expect("the limit fits");
    let _ = pipe.by_ref().take(cap + 1).read_to_end(&mut buffer);
    let overflowed = buffer.len() > OUTPUT_LIMIT;
    if overflowed {
        buffer.truncate(OUTPUT_LIMIT);
        let _ = std::io::copy(&mut pipe, &mut std::io::sink());
    }
    (buffer, overflowed)
}

/// A version the banner parser recognized.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParsedVersion {
    /// Major component.
    pub major: u32,
    /// Minor component.
    pub minor: u32,
}

/// Extract `major.minor` from a version banner, or decline.
///
/// The exact shape, stated rather than rounded: the winning token's first
/// dot-separated part is pure ASCII digits, and its second part begins with
/// digits — the minor's non-digit tail (`2.41-rc1`, `2.41beta`) is dropped,
/// because a build suffix is presentation and its leading number is the
/// version, while the raw banner always travels alongside as provenance.
/// Anything else is **unrecognized, never guessed** — an unparseable banner
/// is reported raw with an unknown range, not rounded to a number.
/// Duplicated in spirit from `crates/fixtures`' prober; see the module doc
/// for why it is not imported.
#[must_use]
pub fn parse_version(banner: &str) -> Option<ParsedVersion> {
    for token in banner.split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == ',') {
        let mut parts = token.split('.');
        if let (Some(major), Some(minor)) = (parts.next(), parts.next())
            && !major.is_empty()
            && major.bytes().all(|b| b.is_ascii_digit())
            && let Ok(major) = major.parse::<u32>()
            && let Ok(minor) = minor
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or_default()
                .parse::<u32>()
        {
            return Some(ParsedVersion { major, minor });
        }
    }
    None
}

/// The banner's first non-empty line, made safe for human output: control
/// characters — C0, DEL, and C1, so neither an escape byte nor a single-byte
/// CSI reaches a terminal — replaced with U+FFFD, and length capped at 200
/// characters. Raw evidence with stated limits, per Section 16.
#[must_use]
pub fn sanitized_first_line(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    line.chars()
        .take(200)
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

/// One tool's report: what was looked for, what answered, what it said.
pub struct ToolReport {
    /// The roster entry this reports on.
    pub name: &'static str,
    /// The roster entry's role, restated for the reader.
    pub role: &'static str,
    /// The tested version label, e.g. `util-linux 2.41`.
    pub tested: &'static str,
    /// How resolution ended.
    pub resolution: Resolution,
}

/// How looking for one tool ended.
pub enum Resolution {
    /// No candidate path held a regular file. Carries every path checked,
    /// so the report says where PartMan looked rather than only that it
    /// failed.
    Absent {
        /// The compiled candidates, in probe order.
        checked: Vec<String>,
    },
    /// A candidate existed and was probed.
    Found {
        /// The absolute path that answered.
        path: String,
        /// What the probe returned.
        probe: ProbeReport,
    },
}

/// The interpreted result of one `--version` probe.
pub enum ProbeReport {
    /// The tool answered; the banner and its interpretation follow.
    Answered {
        /// Sanitized first banner line — the provenance a reader checks.
        raw: String,
        /// The parsed version, if the banner yielded one.
        version: Option<ParsedVersion>,
        /// The range fact: `within-tested-range`, `outside-tested-range`,
        /// or `unknown` when the banner did not parse.
        range: &'static str,
    },
    /// The probe failed; the state word says how.
    Failed {
        /// `timed-out`, `over-output-limit`, or `launch-failed`.
        state: &'static str,
        /// One sentence of detail.
        detail: String,
    },
}

/// Run the doctor over a roster through a launcher.
#[must_use]
pub fn examine(roster: &[ToolSpec], launcher: &dyn ToolLauncher) -> Vec<ToolReport> {
    roster
        .iter()
        .map(|tool| {
            let found = tool
                .candidates
                .iter()
                .find(|candidate| launcher.exists(Path::new(candidate)));
            let resolution = match found {
                None => Resolution::Absent {
                    checked: tool.candidates.iter().map(|c| (*c).to_owned()).collect(),
                },
                Some(path) => Resolution::Found {
                    path: (*path).to_owned(),
                    probe: interpret(tool, launcher.probe_version(Path::new(path))),
                },
            };
            ToolReport {
                name: tool.name,
                role: tool.role,
                tested: tool.tested.label,
                resolution,
            }
        })
        .collect()
}

/// Interpret one probe outcome against the tool's tested version.
fn interpret(tool: &ToolSpec, outcome: ProbeOutcome) -> ProbeReport {
    match outcome {
        ProbeOutcome::Completed { stdout, stderr } => {
            // Some tools banner on stderr; prefer stdout, fall back.
            let banner = if stdout.iter().any(|b| !b.is_ascii_whitespace()) {
                stdout
            } else {
                stderr
            };
            let raw = sanitized_first_line(&banner);
            let version = parse_version(&raw);
            let range = match version {
                Some(version) if (version.major, version.minor) == tool.tested.family => {
                    "within-tested-range"
                }
                Some(_) => "outside-tested-range",
                None => "unknown",
            };
            ProbeReport::Answered {
                raw,
                version,
                range,
            }
        }
        ProbeOutcome::TimedOut => ProbeReport::Failed {
            state: "timed-out",
            detail: format!(
                "no answer within {} seconds; the launch was killed",
                LAUNCH_TIME_LIMIT.as_secs()
            ),
        },
        ProbeOutcome::OverOutputLimit => ProbeReport::Failed {
            state: "over-output-limit",
            detail: format!(
                "more than {OUTPUT_LIMIT} bytes of version output; refused rather than logged"
            ),
        },
        ProbeOutcome::LaunchFailed(detail) => ProbeReport::Failed {
            state: "launch-failed",
            detail,
        },
    }
}

/// The roster-state statement rendered when the roster is empty. `None` on
/// platforms whose roster has entries.
#[must_use]
pub fn empty_roster_statement() -> Option<&'static Refusal> {
    #[cfg(not(target_os = "linux"))]
    {
        Some(&EMPTY_ROSTER)
    }
    #[cfg(target_os = "linux")]
    {
        None
    }
}

/// Render the doctor's JSON object.
#[must_use]
pub fn doctor_json(reports: &[ToolReport], empty: Option<&Refusal>) -> String {
    let roster_json = empty.map_or_else(
        || "{\"state\":\"populated\"}".to_owned(),
        |refusal| {
            format!(
                "{{\"state\":{state},\"reference\":{reference},\"detail\":{detail}}}",
                state = json_escaped(refusal.state),
                reference = json_escaped(refusal.reference),
                detail = json_escaped(refusal.detail),
            )
        },
    );
    let tools: Vec<String> = reports.iter().map(tool_json).collect();
    format!(
        "{{\"roster\":{roster_json},\"tools\":[{}]}}",
        tools.join(",")
    )
}

fn tool_json(report: &ToolReport) -> String {
    let resolution = match &report.resolution {
        Resolution::Absent { checked } => {
            let paths: Vec<String> = checked.iter().map(|p| json_escaped(p)).collect();
            format!(
                "{{\"state\":\"absent\",\"candidates-checked\":[{}]}}",
                paths.join(",")
            )
        }
        Resolution::Found { path, probe } => {
            let probe_json = match probe {
                ProbeReport::Answered {
                    raw,
                    version,
                    range,
                } => {
                    let version_json = version.map_or_else(
                        || "{\"state\":\"unrecognized\"}".to_owned(),
                        |v| {
                            format!(
                                "{{\"state\":\"parsed\",\"major\":{},\"minor\":{}}}",
                                v.major, v.minor
                            )
                        },
                    );
                    format!(
                        "{{\"state\":\"answered\",\"raw\":{raw_json},\"version\":{version_json},\
                         \"range\":{range_json}}}",
                        raw_json = json_escaped(raw),
                        range_json = json_escaped(range),
                    )
                }
                ProbeReport::Failed { state, detail } => format!(
                    "{{\"state\":{state},\"detail\":{detail}}}",
                    state = json_escaped(state),
                    detail = json_escaped(detail),
                ),
            };
            format!(
                "{{\"state\":\"found\",\"path\":{path_json},\"probe\":{probe_json}}}",
                path_json = json_escaped(path),
            )
        }
    };
    format!(
        "{{\"name\":{name},\"role\":{role},\"tested-range\":{tested},\"resolution\":{resolution}}}",
        name = json_escaped(report.name),
        role = json_escaped(report.role),
        tested = json_escaped(report.tested),
    )
}

/// Render the doctor's human block.
#[must_use]
pub fn doctor_human(reports: &[ToolReport], empty: Option<&Refusal>) -> String {
    use std::fmt::Write as _;
    let mut out =
        String::from("doctor (roster tools at compiled absolute paths; no PATH search)\n");
    if let Some(refusal) = empty {
        let _ = writeln!(
            out,
            "  roster: {} ({})\n    {}",
            refusal.state, refusal.reference, refusal.detail
        );
        return out;
    }
    for report in reports {
        let _ = writeln!(
            out,
            "  {} — {} (tested: {})",
            report.name, report.role, report.tested
        );
        match &report.resolution {
            Resolution::Absent { checked } => {
                let _ = writeln!(out, "    absent; checked {}", checked.join(", "));
            }
            Resolution::Found { path, probe } => {
                let _ = writeln!(out, "    found at {path}");
                match probe {
                    ProbeReport::Answered {
                        raw,
                        version,
                        range,
                    } => {
                        let version_text = version.map_or_else(
                            || "unrecognized".to_owned(),
                            |v| format!("{}.{}", v.major, v.minor),
                        );
                        let _ = writeln!(out, "    version: {version_text} ({range})");
                        let _ = writeln!(out, "    banner: {raw}");
                    }
                    ProbeReport::Failed { state, detail } => {
                        let _ = writeln!(out, "    probe {state}: {detail}");
                    }
                }
            }
        }
    }
    out
}
