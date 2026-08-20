//! Unprivileged whole-device enumeration for macOS, through the launcher
//! seam and the bounded plist reader — and nothing it does not need.
//!
//! WP-035's charter governs every line: *prints raw identifier strings
//! labelled by reporting interface; computes no strength, table state,
//! hash, verdict, or plan.* The macOS contract this package reads is
//! `diskutil`'s structured output — the interface the increment 6 matrix
//! measured — through exactly two launches: `diskutil list -plist` for the
//! `WholeDisks` names, and `diskutil info -plist <name>` per whole device
//! for identity attributes. Both go through [`crate::doctor::ToolLauncher`]
//! under the SAFE-004-derived controls (compiled absolute path, structured
//! argv, cleared environment, bounded output, a time limit), with the
//! per-stream output bounds stated here because an enumeration and a
//! version banner are legitimately different sizes.
//!
//! **What it deliberately does not read**, each because the record or the
//! register says so:
//!
//! - **No `Content`, no partition-scheme field of any spelling.** The
//!   scheme name is partition-table material — helper-authored under
//!   ADR-0014, never this client's — and the increment 7 adversarial
//!   round refused reach cells resting on it; the hybrid trap it recorded
//!   still stands. The reach declaration therefore stays all-negative:
//!   this contract reads identity attributes and no table-state surface.
//! - **No UUID keys and no APFS fields.** A `DiskUUID` is derived from the
//!   partition scheme, and APFS membership is table-state and verdict
//!   material — ADR-0014's and ADR-0016's helper-authored layers, not an
//!   inventory's to report.
//! - **No partition rows.** The `WholeDisks` array is the only source of
//!   devices, so there is no filter that could fail open and promote a
//!   partition — the shape the Linux adapter had to defend with an
//!   error-kind match is unrepresentable here.
//!
//! **Clamping** is the same decision the Linux adapter recorded: there is
//! no privilege-conditional branch, and running this as root asks the same
//! tool the same two questions. The values are what `diskutil` reported at
//! call time; no device is opened by this crate, and the tool's own reads
//! are its contract, not this package's claim.
//!
//! This module is deliberately compilable on every platform — it is pure
//! over the injected seam — so its Tier-1 tests run on all three CI legs
//! rather than only where a defect would be least convenient to find. Only
//! `inspect`'s dispatch consults the target OS.

use std::path::Path;

use crate::devices::{DEVICE_LIMIT, Device, Enumeration, RawField, VALUE_LIMIT, selector};
use crate::doctor::{DOCTOR_TIME_LIMIT, ProbeOutcome, ToolLauncher};
use crate::inspect::{ObservedValue, Outcome};
use crate::plist;

/// The compiled absolute path the launches use. No `PATH` lookup exists in
/// the launcher, so this is the only spelling that can run.
pub const DISKUTIL: &str = "/usr/sbin/diskutil";

/// Per-stream output bound for `diskutil list -plist`, aligned with the
/// plist reader's own input cap: the two bounds refuse at the same size,
/// so the parser's promise never depends on the launcher's.
pub const LIST_OUTPUT_LIMIT: usize = plist::INPUT_LIMIT;

/// Per-stream output bound for one `diskutil info -plist` launch. The
/// measured capture is under four kilobytes; sixty-four is headroom.
pub const INFO_OUTPUT_LIMIT: usize = 64 * 1024;

/// The interface label carried on every attribute row.
const DISKUTIL_INFO: &str = "macos-diskutil-info";

/// The in-band method statement on every value: what the bytes are and are
/// not, travelling with the data rather than living in a comment.
pub const DISKUTIL_METHOD: &str = "structured output of a bounded diskutil launch; values are what the tool reported \
     at call time, and no device was opened by this package";

/// The `diskutil info -plist` keys this contract reads, in reporting
/// order — identity attributes only, grounded in the sitting-2 captures.
/// `Content`, UUID keys, and APFS fields are deliberately absent; see the
/// module doc.
pub const INFO_KEYS: [&str; 12] = [
    "BusProtocol",
    "DeviceBlockSize",
    "DeviceNode",
    "Ejectable",
    "Internal",
    "IORegistryEntryName",
    "MediaName",
    "Removable",
    "RemovableMedia",
    "Size",
    "TotalSize",
    "VirtualOrPhysical",
];

/// Enumerate whole devices through the injected launcher.
///
/// Pure over the seam: every failure mode of the two launches and the two
/// parses lands in ADR-C4's vocabulary, and no partial device list is ever
/// returned in place of a refusal.
pub fn enumerate(launcher: &dyn ToolLauncher) -> Enumeration {
    let listed = launcher.launch(
        Path::new(DISKUTIL),
        &["list", "-plist"],
        LIST_OUTPUT_LIMIT,
        DOCTOR_TIME_LIMIT,
    );
    let stdout = match listed {
        ProbeOutcome::Completed { stdout, .. } => stdout,
        // A nonzero exit is a failure, never evidence — the doctor's rule,
        // held here too: whatever such a run printed is not parsed.
        ProbeOutcome::NonzeroExit { code, .. } => {
            return Enumeration::Failed {
                error: match code {
                    Some(code) => {
                        format!("diskutil list exited nonzero (code {code}); output not parsed")
                    }
                    None => "diskutil list exited nonzero (no code); output not parsed".to_owned(),
                },
            };
        }
        ProbeOutcome::TimedOut => {
            return Enumeration::Failed {
                error: "diskutil list exceeded the launch time limit".to_owned(),
            };
        }
        ProbeOutcome::OverOutputLimit => {
            return Enumeration::Failed {
                error: format!(
                    "diskutil list produced more than {LIST_OUTPUT_LIMIT} bytes on one stream; \
                     refused rather than truncated"
                ),
            };
        }
        ProbeOutcome::LaunchFailed(error) => {
            return Enumeration::Failed {
                error: format!("diskutil list did not launch: {error}"),
            };
        }
    };

    let names = match plist::whole_disks(&stdout) {
        Ok(names) => names,
        Err(refusal) => {
            return Enumeration::Failed {
                error: refusal.detail(),
            };
        }
    };

    // A name from the list output becomes an argument to the next launch,
    // so only the expected spelling is allowed through: `disk` and digits,
    // nothing else. Anything different is refused before it reaches argv —
    // not because argv can inject through the no-shell launcher, but
    // because a name that does not look like a device should not select
    // what a tool answers about.
    for name in &names {
        let digits = name.strip_prefix("disk").unwrap_or("");
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Enumeration::Failed {
                error: "a WholeDisks entry is not of the form disk<digits>; refused rather \
                        than passed to a launcher"
                    .to_owned(),
            };
        }
    }

    if names.len() > DEVICE_LIMIT {
        return Enumeration::OverLimit { seen: names.len() };
    }

    let devices = names
        .into_iter()
        .enumerate()
        .map(|(index, name)| Device {
            selector: selector(index),
            fields: info_fields_for(launcher, &name),
            kernel_name: name,
        })
        .collect();
    Enumeration::Listed(devices)
}

/// One device's roster rows, from one `diskutil info -plist` launch.
///
/// A launch or parse failure fails every row for this device and no other:
/// the device stays listed — its name is the list output's fact — with each
/// value's outcome saying what happened, per ADR-C4.
fn info_fields_for(launcher: &dyn ToolLauncher, name: &str) -> Vec<RawField> {
    let outcome = launcher.launch(
        Path::new(DISKUTIL),
        &["info", "-plist", name],
        INFO_OUTPUT_LIMIT,
        DOCTOR_TIME_LIMIT,
    );
    let stdout = match outcome {
        ProbeOutcome::Completed { stdout, .. } => stdout,
        ProbeOutcome::NonzeroExit { code, .. } => {
            return all_failed(&match code {
                Some(code) => {
                    format!("diskutil info exited nonzero (code {code}); output not parsed")
                }
                None => "diskutil info exited nonzero (no code); output not parsed".to_owned(),
            });
        }
        ProbeOutcome::TimedOut => {
            return all_failed("diskutil info exceeded the launch time limit");
        }
        ProbeOutcome::OverOutputLimit => {
            return all_failed(&format!(
                "diskutil info produced more than {INFO_OUTPUT_LIMIT} bytes on one stream; \
                 refused rather than truncated"
            ));
        }
        ProbeOutcome::LaunchFailed(error) => {
            return all_failed(&format!("diskutil info did not launch: {error}"));
        }
    };

    let entries = match plist::info_fields(&stdout) {
        Ok(entries) => entries,
        Err(refusal) => return all_failed(&refusal.detail()),
    };

    INFO_KEYS
        .iter()
        .map(|key| {
            let found = entries.iter().find(|(name, _)| name == key);
            let outcome = match found {
                Some((_, plist::InfoValue::Scalar(text))) => {
                    // The parser bounds each text run, so a scalar here is
                    // already within limits; the assertion is structural,
                    // not a second truncation point.
                    debug_assert!(text.len() <= VALUE_LIMIT);
                    Outcome::Observed(ObservedValue::Decimal(text.clone()))
                }
                Some((_, plist::InfoValue::EmptyString)) => {
                    Outcome::Observed(ObservedValue::Absent {
                        reason: "the key is present and its value is empty".to_owned(),
                    })
                }
                // Present but not a scalar: reporting a flattened rendering
                // would put words in the interface's mouth, and reporting
                // absence would be a false positively-determined negative.
                Some((_, plist::InfoValue::Container)) => Outcome::Failed {
                    error: "the key is present but its value is not a scalar; refused rather \
                            than flattened"
                        .to_owned(),
                },
                None => Outcome::Observed(ObservedValue::Absent {
                    reason: "the key is not present in this device's diskutil record".to_owned(),
                }),
            };
            RawField {
                interface: DISKUTIL_INFO,
                method: DISKUTIL_METHOD,
                property: (*key).to_owned(),
                outcome,
            }
        })
        .collect()
}

/// Every roster row failed for the same stated reason.
fn all_failed(error: &str) -> Vec<RawField> {
    INFO_KEYS
        .iter()
        .map(|key| RawField {
            interface: DISKUTIL_INFO,
            method: DISKUTIL_METHOD,
            property: (*key).to_owned(),
            outcome: Outcome::Failed {
                error: error.to_owned(),
            },
        })
        .collect()
}
