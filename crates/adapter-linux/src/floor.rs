//! Increment 5b: the Section 9 floor determination for the Linux tiers,
//! read from two files on the rows that measured them, and reported as
//! WP-050's [`PlatformFact`] — met, below, or **undetermined**, never
//! guessed.
//!
//! **The rows.** DR16 and DR17 (jammy), DR18 (the first Arch guest) and
//! DR19 (the first Debian guest), `docs/quality/observability.md`, the
//! floor-input cells and the Debian 12 `os-release` cell, 2026-08-19:
//! `/etc/os-release` is a client-readable file on all three (a symlink to
//! `/usr/lib/os-release`), `KEY=value` lines, one trailing newline; Ubuntu
//! carries `ID=ubuntu` unquoted, `VERSION_ID="22.04"` double-quoted,
//! `ID_LIKE=debian`; Debian 12 carries `ID=debian` unquoted and
//! **`VERSION_ID="12"`** — double-quoted, one numeric part, no minor — and
//! **no** `ID_LIKE`; Arch carries `ID=arch`, `BUILD_ID=rolling` and **no**
//! `VERSION_ID`, `ID_LIKE` or `VERSION_CODENAME`; `/proc/sys/kernel/osrelease`
//! is `uname -r` plus one newline on all three. Those are the only shapes
//! this module claims to read; everything else it answers undetermined.
//!
//! **The row it determines against** is Section 9's, verbatim
//! (`AGENT_BUILD_SPEC.md`, "Platform support floors"): Debian/Ubuntu —
//! "Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15; `UDisks2` ≥ 2.9"; Arch —
//! "Current rolling … tool-version-gated". Floors change only via ADR; "the
//! capability engine may narrow further at runtime (CAP-004); it may never
//! widen below these floors."
//!
//! **Three conjuncts, three honest answers.** The distribution conjunct
//! reads `ID` and, for Ubuntu, `VERSION_ID`, stripping exactly the double
//! quotes DR16 measured and comparing the release numerically against the
//! row (`22.04` is the floor; a later release is above it — that much
//! arithmetic the word "floor" states, and no more). Arch's row names no
//! version, so `ID=arch` alone meets it. **Debian compares one numeric
//! part**: DR19 measured `VERSION_ID="12"` with no minor, so the Debian arm
//! parses the leading integer and compares it against the row's 12 — a
//! later major is above the floor, `11` is a measured shortfall — and must
//! not demand the `major.minor` shape Ubuntu carries (until that row
//! landed, this arm answered undetermined rather than borrow Ubuntu's
//! shape; the evidence rule will not let a spec sentence stand in for a
//! measured byte). The kernel conjunct parses `major.minor` from
//! `osrelease` and compares against `5.15`; a string that does not parse
//! is undetermined, never assumed. **The `UDisks2` conjunct is undetermined
//! by construction**: no file under this contract carries the daemon's
//! version, LIN-001's route is undecided, and DR18 measured the second tier
//! shipping without the daemon at all. Under Section 9's own sentence an
//! undetermined conjunct is not met and was not measured below, which is
//! exactly what [`PlatformFact::Undetermined`] (WP-050 increment 5) exists
//! to carry: the engine blocks and names the conjunct.
//!
//! **Composition, fail-closed.** Any conjunct measured **unmet** makes the
//! floor [`PlatformFact::BelowFloor`] — a shortfall somebody measured;
//! otherwise any undetermined conjunct makes it
//! [`PlatformFact::Undetermined`], naming the first; only three met
//! conjuncts (Arch: the one its row states) reach [`PlatformFact::MeetsFloor`].
//! On every measured host today the Debian/Ubuntu answer is therefore
//! `Undetermined` on the `UDisks2` conjunct — the honest answer, and the same
//! one the WP-020 acceptance environments would get, since they run without
//! `udisks2` (recorded there in terms).
//!
//! **What this reads and nothing else.** Two files, through the bounded
//! record seam; no process, no D-Bus, no `uname` call — the SAFE-002
//! structural guard over every shipped module covers this one.

use std::path::Path;

use partman_capability::engine::PlatformFact;
use partman_domain::canonical::Value;
use partman_domain::model::provenance::{Observation, Outcome};

use crate::contract::{ContractSource, RecordRead, read_record};
use crate::observation::{Interface, observe_unavailable};

/// The OS release record, relative to the OS-release root (DR16, DR18).
pub const OS_RELEASE_FILE: &str = "os-release";
/// The kernel release, relative to the procfs root (DR17, DR18).
pub const KERNEL_RELEASE_FILE: &str = "sys/kernel/osrelease";
/// The `os-release` keys this determination reads.
pub const OS_RELEASE_KEYS: &[&str] = &["ID", "VERSION_ID"];
/// Section 9's kernel floor for the Debian/Ubuntu tier, as `(major, minor)`.
pub const KERNEL_FLOOR: (u64, u64) = (5, 15);
/// Section 9's Ubuntu release floor, as `(major, minor)`.
pub const UBUNTU_FLOOR: (u64, u64) = (22, 4);
/// Section 9's Debian release floor, as the one numeric part DR19 measured
/// (`VERSION_ID="12"`, no minor).
pub const DEBIAN_FLOOR: u64 = 12;

/// The Section 9 Linux tier a host's `ID` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tier {
    /// `ID=ubuntu`: the Debian/Ubuntu row, Ubuntu half.
    Ubuntu,
    /// `ID=debian`: the Debian/Ubuntu row, Debian half — `VERSION_ID`
    /// one numeric part (DR19), compared against 12.
    Debian,
    /// `ID=arch`: the Arch row.
    Arch,
    /// An `ID` no Section 9 row names.
    Unrecognized {
        /// The `ID` value as read.
        id: String,
    },
    /// `ID` could not be read.
    Unknown {
        /// Why.
        reason: String,
    },
}

/// One conjunct's answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conjunct {
    /// Measured at or above the row.
    Met,
    /// Measured below the row.
    Unmet {
        /// What was measured.
        measured: String,
    },
    /// Could not be established — never assumed either way.
    Undetermined {
        /// Why.
        reason: String,
    },
    /// The row states no such conjunct for this tier.
    NotInRow,
}

/// The floor determination, with its inputs' observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorReport {
    /// The tier `ID` named.
    pub tier: Tier,
    /// The distribution-and-version conjunct.
    pub distribution: Conjunct,
    /// The kernel conjunct.
    pub kernel: Conjunct,
    /// The `UDisks2` conjunct — undetermined by construction on the
    /// Debian/Ubuntu row, not in the Arch row.
    pub udisks2: Conjunct,
    /// WP-050's fact, composed fail-closed from the three.
    pub platform: PlatformFact,
    /// MODEL-004 observations: each `os-release` key read (on the
    /// OS-release interface) and the kernel release (on procfs).
    pub observations: Vec<Observation>,
}

/// Determine the Section 9 floor from the two files.
#[must_use]
pub fn platform_floor(
    source: &dyn ContractSource,
    os_release_root: &Path,
    procfs_root: &Path,
) -> FloorReport {
    let mut observations = Vec::new();
    let release = read_os_release(
        source,
        &os_release_root.join(OS_RELEASE_FILE),
        &mut observations,
    );
    let tier = match &release {
        Ok(keys) => match keys.get("ID").map(String::as_str) {
            Some("ubuntu") => Tier::Ubuntu,
            Some("debian") => Tier::Debian,
            Some("arch") => Tier::Arch,
            Some(other) => Tier::Unrecognized {
                id: other.to_owned(),
            },
            None => Tier::Unknown {
                reason: "`ID` is absent from os-release".to_owned(),
            },
        },
        Err(reason) => Tier::Unknown {
            reason: reason.clone(),
        },
    };
    let distribution = distribution_conjunct(&tier, &release);
    let kernel = match tier {
        Tier::Arch => Conjunct::NotInRow,
        _ => match read_kernel(
            source,
            &procfs_root.join(KERNEL_RELEASE_FILE),
            &mut observations,
        ) {
            Ok(text) => match parse_major_minor(&text) {
                Some(pair) if pair >= KERNEL_FLOOR => Conjunct::Met,
                Some(_) => Conjunct::Unmet {
                    measured: format!("kernel {text}, below 5.15"),
                },
                None => Conjunct::Undetermined {
                    reason: format!("kernel release {text:?} does not parse as major.minor"),
                },
            },
            Err(reason) => Conjunct::Undetermined {
                reason: format!("kernel: {reason}"),
            },
        },
    };
    let udisks2 = match tier {
        Tier::Arch => Conjunct::NotInRow,
        _ => Conjunct::Undetermined {
            reason: "UDisks2 >= 2.9: no client-readable source under this contract, and LIN-001's \
                     route is undecided"
                .to_owned(),
        },
    };
    let platform = compose(&distribution, &kernel, &udisks2);
    FloorReport {
        tier,
        distribution,
        kernel,
        udisks2,
        platform,
        observations,
    }
}

/// The distribution conjunct of Section 9's row, from the tier and the
/// `os-release` keys: Ubuntu on `major.minor` against 22.04, Debian on the
/// one numeric part DR19 measured against 12, Arch on `ID` alone; every
/// other shape undetermined.
fn distribution_conjunct(
    tier: &Tier,
    release: &Result<std::collections::BTreeMap<String, String>, String>,
) -> Conjunct {
    match (tier, release) {
        (Tier::Ubuntu, Ok(keys)) => match keys.get("VERSION_ID") {
            Some(version) => match parse_major_minor(version) {
                Some(pair) if pair >= UBUNTU_FLOOR => Conjunct::Met,
                Some(_) => Conjunct::Unmet {
                    measured: format!("Ubuntu VERSION_ID={version}, below 22.04"),
                },
                None => Conjunct::Undetermined {
                    reason: format!("Ubuntu VERSION_ID={version:?} does not parse as major.minor"),
                },
            },
            None => Conjunct::Undetermined {
                reason: "Ubuntu os-release carries no VERSION_ID".to_owned(),
            },
        },
        (Tier::Debian, Ok(keys)) => match keys.get("VERSION_ID") {
            Some(version) => match parse_major(version) {
                Some(major) if major >= DEBIAN_FLOOR => Conjunct::Met,
                Some(_) => Conjunct::Unmet {
                    measured: format!("Debian VERSION_ID={version}, below 12"),
                },
                None => Conjunct::Undetermined {
                    reason: format!(
                        "Debian VERSION_ID={version:?} does not parse as a leading integer"
                    ),
                },
            },
            None => Conjunct::Undetermined {
                reason: "Debian os-release carries no VERSION_ID".to_owned(),
            },
        },
        (Tier::Arch, _) => Conjunct::Met,
        (Tier::Unrecognized { id }, _) => Conjunct::Undetermined {
            reason: format!("distribution: no Section 9 row for ID={id}"),
        },
        (Tier::Unknown { reason }, _) | (Tier::Ubuntu | Tier::Debian, Err(reason)) => {
            Conjunct::Undetermined {
                reason: format!("distribution: {reason}"),
            }
        }
    }
}

/// Fail-closed composition: a measured shortfall is below; else an
/// undetermined conjunct is undetermined, naming the first; else met.
#[must_use]
pub fn compose(distribution: &Conjunct, kernel: &Conjunct, udisks2: &Conjunct) -> PlatformFact {
    let all = [distribution, kernel, udisks2];
    if all.iter().any(|c| matches!(c, Conjunct::Unmet { .. })) {
        return PlatformFact::BelowFloor;
    }
    for conjunct in all {
        if let Conjunct::Undetermined { reason } = conjunct {
            return PlatformFact::Undetermined {
                conjunct: reason.clone(),
            };
        }
    }
    PlatformFact::MeetsFloor
}

/// Parse the leading integer of a release string (`12`, `12.1`, `6.1.0-52`);
/// anything else is `None`. The Debian arm's comparison: DR19 measured
/// `VERSION_ID="12"` with no minor part.
#[must_use]
pub fn parse_major(text: &str) -> Option<u64> {
    text.split(['.', '-', '_']).next()?.parse::<u64>().ok()
}

/// Parse `major.minor` from the start of a release string (`22.04`,
/// `5.15.0-186-generic`, `7.1.8-arch1-3`); anything else is `None`.
#[must_use]
pub fn parse_major_minor(text: &str) -> Option<(u64, u64)> {
    let mut parts = text.split(['.', '-', '_']);
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    Some((major, minor))
}

/// Read `os-release`, keying only [`OS_RELEASE_KEYS`], stripping exactly
/// one pair of double quotes where DR16 measured them.
fn read_os_release(
    source: &dyn ContractSource,
    path: &Path,
    observations: &mut Vec<Observation>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let text = match read_record(source, path) {
        RecordRead::Present { text, .. } => text,
        RecordRead::NoRecord => {
            let reason = "os-release is not present".to_owned();
            observations.push(observe_unavailable(Interface::OsRelease, &reason));
            return Err(reason);
        }
        RecordRead::OverLimit { seen } => {
            let reason = format!("os-release is {seen} bytes, over the limit");
            observations.push(failed(Interface::OsRelease, &reason));
            return Err(reason);
        }
        RecordRead::NotText => {
            let reason = "os-release is not UTF-8".to_owned();
            observations.push(failed(Interface::OsRelease, &reason));
            return Err(reason);
        }
        RecordRead::Failed { error } => {
            let reason = format!("os-release could not be read: {error}");
            observations.push(failed(Interface::OsRelease, &reason));
            return Err(reason);
        }
    };
    let mut keys = std::collections::BTreeMap::new();
    for wanted in OS_RELEASE_KEYS {
        let found = text.lines().find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name == *wanted).then(|| unquote(value).to_owned())
        });
        match found {
            Some(value) => {
                observations.push(observed(Interface::OsRelease, &format!("{wanted}={value}")));
                keys.insert((*wanted).to_owned(), value);
            }
            None => observations.push(Observation {
                adapter: Interface::OsRelease.adapter(),
                adapter_version: crate::VERSION.to_owned(),
                method: Interface::OsRelease.method(),
                outcome: Outcome::ObservedAbsent,
            }),
        }
    }
    Ok(keys)
}

/// Strip exactly one leading and one trailing double quote, both or neither.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

fn read_kernel(
    source: &dyn ContractSource,
    path: &Path,
    observations: &mut Vec<Observation>,
) -> Result<String, String> {
    match read_record(source, path) {
        RecordRead::Present { text, .. } => {
            observations.push(observed(Interface::Procfs, &text));
            Ok(text)
        }
        RecordRead::NoRecord => {
            let reason = "osrelease is not present".to_owned();
            observations.push(observe_unavailable(Interface::Procfs, &reason));
            Err(reason)
        }
        RecordRead::OverLimit { seen } => {
            let reason = format!("osrelease is {seen} bytes, over the limit");
            observations.push(failed(Interface::Procfs, &reason));
            Err(reason)
        }
        RecordRead::NotText => {
            let reason = "osrelease is not UTF-8".to_owned();
            observations.push(failed(Interface::Procfs, &reason));
            Err(reason)
        }
        RecordRead::Failed { error } => {
            let reason = format!("osrelease could not be read: {error}");
            observations.push(failed(Interface::Procfs, &reason));
            Err(reason)
        }
    }
}

fn observed(interface: Interface, value: &str) -> Observation {
    Observation {
        adapter: interface.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method: interface.method(),
        outcome: Outcome::Observed {
            value: Value::Text(value.to_owned()),
        },
    }
}

fn failed(interface: Interface, error: &str) -> Observation {
    Observation {
        adapter: interface.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method: interface.method(),
        outcome: Outcome::Failed {
            error: error.to_owned(),
        },
    }
}
