//! Observation records: what an adapter reported, tagged with who reported
//! it — the precursor toward MODEL-004, whose envelope residence and hashed
//! artifacts are WP-010's to deliver, not this module's to anticipate.
//!
//! Three rules carry the register's boundary here, and each is enforced by
//! a test rather than by this comment:
//!
//! - **Bytes, never classifications.** An observation reports what was read,
//!   in hex, labelled by the adapter that read it. Whether those bytes are a
//!   partition table, and what state it is in, is exactly what SI-35 holds
//!   open — the inspector prints the raw material and the standing gated
//!   list, and the reader does the interpreting.
//! - **ADR-C4's trichotomy is real.** A successful read is a value. A probe
//!   beyond the object's end is a **positively observed absence** — the
//!   handle's own length says those bytes do not exist, which is knowledge,
//!   not failure. A read error is **unavailable** — the adapter could not
//!   answer, and unavailability must never masquerade as absence, because
//!   treating "could not look" as "looked and found nothing" is the
//!   fail-closed violation SAFE-005 exists to prevent.
//! - **No path echo, no stable handle.** The replayed object is reported
//!   under a session-local selector; the caller knows what they named, and
//!   a path is on SEC-006's deny-floor. SI-27 keeps stable handles gated.
//!
//! The replay adapter reads one caller-named **regular file**, verified
//! through the opened handle — `fstat` on the handle, not `stat` on the
//! path, the same discipline the SAFE-007 interlock records — so a block
//! device named here is refused before its first byte is read, and the
//! repository's boundary sentence (no command opens a block device at all
//! today) survives this increment too.

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::{VERSION, json_escaped};

/// Who produced an observation, per MODEL-004's shape: source adapter,
/// version, method. The envelope these eventually live in is WP-010's.
pub struct Attribution {
    /// The adapter that made the observation.
    pub adapter: &'static str,
    /// The adapter's version — this crate's, for the replay adapter.
    pub version: &'static str,
    /// How the observation was made.
    pub method: &'static str,
}

/// The replay adapter's attribution, on every observation it makes.
const REPLAY: Attribution = Attribution {
    adapter: "fixture-replay",
    version: VERSION,
    method: "seek-and-read through the verified handle",
};

/// How one observation ended — ADR-C4's trichotomy, in the order of how
/// much was learned.
pub enum Outcome {
    /// The adapter read these bytes. Raw, hex-encoded, uninterpreted.
    Value(String),
    /// The adapter positively determined the asked-for thing does not
    /// exist. This is a value — knowledge of absence — not a failure.
    ObservedAbsent {
        /// What established the absence.
        reason: String,
    },
    /// The adapter could not answer. Not an absence, and never rendered as
    /// one.
    Unavailable {
        /// Why the answer could not be produced.
        reason: String,
    },
}

/// One observation: subject, who observed it, what came back.
pub struct Observation {
    /// What was asked, in neutral terms — an offset range, a length —
    /// never a format name.
    pub subject: String,
    /// Who answered.
    pub attribution: Attribution,
    /// What came back.
    pub outcome: Outcome,
}

/// Why a replay was refused outright.
pub struct ReplayRefusal {
    /// The state word: always `refused` here.
    pub state: &'static str,
    /// The requirement whose discipline refused it.
    pub reference: &'static str,
    /// One sentence a human can act on. Never contains the caller's path.
    pub detail: String,
}

/// The byte ranges the replay adapter probes: offsets where storage formats
/// customarily place structure, so the raw material is useful to a reader —
/// who does the interpreting themselves, because classification is gated
/// (SI-35, SI-28; the standing list travels in every inspect answer).
pub const PROBES: &[(u64, u64)] = &[(0, 16), (510, 2), (512, 16), (1024, 16)];

/// The standing gated-surface list, rendered in every inspect answer so
/// what the inspector will not say is stated in-band, never inferred from
/// silence. Each entry names the register issue that gates it.
pub const GATED: &[(&str, &str)] = &[
    ("identity-strength", "SI-28"),
    ("partition-table-state", "SI-35"),
    ("same-device-claims", "SI-12"),
];

/// Replay one regular file through the fixture-replay adapter.
///
/// # Errors
///
/// Refuses, with a typed value, anything the opened handle reports as not
/// a regular file, and any object that cannot be opened at all.
pub fn replay(path: &Path) -> Result<Vec<Observation>, ReplayRefusal> {
    let mut file = std::fs::File::open(path).map_err(|error| ReplayRefusal {
        state: "refused",
        reference: "SAFE-005",
        detail: format!("the object could not be opened: {error}"),
    })?;
    // fstat through the handle, not stat on the path: this answers "what
    // did I actually open", which no rebinding of the name can change.
    let metadata = file.metadata().map_err(|error| ReplayRefusal {
        state: "refused",
        reference: "SAFE-005",
        detail: format!("the opened handle would not describe itself: {error}"),
    })?;
    if !metadata.is_file() {
        return Err(ReplayRefusal {
            state: "refused",
            reference: "SAFE-005",
            detail: "replay reads regular files only, and the opened handle reports \
                     something else; a device is not a fixture, and refusing here is \
                     what keeps that sentence true"
                .to_owned(),
        });
    }
    let length = metadata.len();

    let mut observations = vec![Observation {
        subject: "object-length".to_owned(),
        attribution: REPLAY,
        outcome: Outcome::Value(length.to_string()),
    }];

    for &(offset, count) in PROBES {
        let subject = format!("bytes[{offset}..{end})", end = offset + count);
        let outcome = if offset + count <= length {
            match read_exact_at(&mut file, offset, count) {
                Ok(bytes) => Outcome::Value(hex(&bytes)),
                Err(error) => Outcome::Unavailable {
                    reason: format!("the read failed: {error}"),
                },
            }
        } else {
            Outcome::ObservedAbsent {
                reason: format!(
                    "the object ends at byte {length}; the probed range does not exist \
                     on it — known from the handle's own length, not from a failed read"
                ),
            }
        };
        observations.push(Observation {
            subject,
            attribution: REPLAY,
            outcome,
        });
    }
    Ok(observations)
}

/// Seek and fully read `count` bytes at `offset` through the handle.
fn read_exact_at(file: &mut std::fs::File, offset: u64, count: u64) -> std::io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0_u8; usize::try_from(count).expect("probe sizes are small")];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Lowercase hex. An encoding of what was read — not a digest, not a
/// checksum; no hash function exists in this binary's reach for it to be.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing into a String cannot fail");
    }
    out
}

/// Render one outcome as JSON.
fn outcome_json(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Value(value) => {
            format!("{{\"state\":\"value\",\"value\":{}}}", json_escaped(value))
        }
        Outcome::ObservedAbsent { reason } => format!(
            "{{\"state\":\"observed-absent\",\"reason\":{}}}",
            json_escaped(reason)
        ),
        Outcome::Unavailable { reason } => format!(
            "{{\"state\":\"unavailable\",\"reason\":{}}}",
            json_escaped(reason)
        ),
    }
}

/// Render the gated-surface list as JSON — part of every inspect answer.
#[must_use]
pub fn gated_json() -> String {
    let entries: Vec<String> = GATED
        .iter()
        .map(|(surface, gate)| {
            format!(
                "{{\"surface\":{surface},\"state\":\"not-established\",\"gate\":{gate}}}",
                surface = json_escaped(surface),
                gate = json_escaped(gate),
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// Render a replayed observation set as JSON, under a session-local
/// selector and with the standing gated list alongside.
#[must_use]
pub fn replay_json(observations: &[Observation]) -> String {
    let rendered: Vec<String> = observations
        .iter()
        .map(|observation| {
            format!(
                "{{\"subject\":{subject},\"adapter\":{{\"name\":{name},\"version\":{version},\
                 \"method\":{method}}},\"outcome\":{outcome}}}",
                subject = json_escaped(&observation.subject),
                name = json_escaped(observation.attribution.adapter),
                version = json_escaped(observation.attribution.version),
                method = json_escaped(observation.attribution.method),
                outcome = outcome_json(&observation.outcome),
            )
        })
        .collect();
    format!(
        "{{\"selector\":\"replay:0\",\"observations\":[{observations}],\"gated\":{gated}}}",
        observations = rendered.join(","),
        gated = gated_json(),
    )
}

/// Render the no-adapter inspect answer as JSON: a typed statement, the
/// platform package that changes it, and the standing gated list.
#[must_use]
pub fn no_adapter_json() -> String {
    format!(
        "{{\"adapters\":{{\"state\":\"not-implemented\",\"reference\":{reference},\
         \"detail\":{detail}}},\"observations\":[],\"gated\":{gated}}}",
        reference = json_escaped(platform_adapter_package()),
        detail = json_escaped(NO_ADAPTER_DETAIL),
        gated = gated_json(),
    )
}

/// The platform package that will register this host's device adapter.
#[must_use]
pub fn platform_adapter_package() -> &'static str {
    if cfg!(target_os = "windows") {
        "WP-W100"
    } else if cfg!(target_os = "linux") {
        "WP-L100"
    } else {
        "WP-M100"
    }
}

/// The sentence beside the no-adapter statement.
const NO_ADAPTER_DETAIL: &str = "no device adapter is registered on this platform; observation of real devices \
     arrives with the platform adapter package, and an empty observation list must \
     not be read as an empty machine";

/// Render one outcome for humans.
fn outcome_human(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Value(value) => format!("value {value}"),
        Outcome::ObservedAbsent { reason } => format!("observed-absent — {reason}"),
        Outcome::Unavailable { reason } => format!("unavailable — {reason}"),
    }
}

/// Render the gated list for humans.
fn gated_human(out: &mut String) {
    use std::fmt::Write as _;
    let _ = writeln!(out, "  gated (the inspector will not say, and names why):");
    for (surface, gate) in GATED {
        let _ = writeln!(out, "    {surface}: not-established ({gate})");
    }
}

/// Render a replayed observation set for humans.
#[must_use]
pub fn replay_human(observations: &[Observation]) -> String {
    use std::fmt::Write as _;
    let mut out = String::from(
        "inspect (fixture-replay adapter; bytes labelled by who read them, never \
         classified)\n  selector: replay:0\n",
    );
    for observation in observations {
        let _ = writeln!(
            out,
            "  {subject}: {outcome}\n    adapter: {name} {version} ({method})",
            subject = observation.subject,
            outcome = outcome_human(&observation.outcome),
            name = observation.attribution.adapter,
            version = observation.attribution.version,
            method = observation.attribution.method,
        );
    }
    gated_human(&mut out);
    out
}

/// Render the no-adapter inspect answer for humans.
#[must_use]
pub fn no_adapter_human() -> String {
    let mut out = format!(
        "inspect\n  adapters: not-implemented ({reference})\n    {NO_ADAPTER_DETAIL}\n",
        reference = platform_adapter_package(),
    );
    gated_human(&mut out);
    out
}
