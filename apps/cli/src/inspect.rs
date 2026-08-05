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
//! - **ADR-C4's outcome vocabulary, as written.** `observed` carries bytes
//!   or a positively determined absence — absence is a value, and the
//!   state word says so. `unavailable` is the platform not exposing an
//!   answer; `failed` is the read itself erroring; and the two are kept
//!   distinct because collapsing them is the paraphrase an earlier draft
//!   shipped and review refused. No outcome ever renders a non-answer as
//!   an absence: treating "could not look" as "looked and found nothing"
//!   is the fail-closed violation SAFE-005 exists to prevent.
//! - **No path echo, no stable handle.** The replayed object is reported
//!   under a session-local selector; the caller knows what they named, and
//!   a path is on SEC-006's deny-floor. SI-27 keeps stable handles gated.
//!
//! The replay adapter reads one caller-named **regular file**. Anything
//! else is refused unread: a pre-open look refuses devices and directories
//! in the common case before any open touches them, and the authority is
//! `fstat` through the opened handle — the interlock's discipline — so a
//! device swapped in by a rebinding race is opened read-only at most long
//! enough for the handle to identify itself, then refused with no byte
//! read. The exact boundary, stated rather than rounded: no command reads
//! a block device, and nothing opens one with write intent; a momentary
//! read-only open under a race is the stated residue of choosing handle
//! verification over trusting a name.

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

/// How one observation ended — ADR-C4's outcome vocabulary, taken as the
/// ADR wrote it rather than paraphrased: an earlier draft of this module
/// shipped `value / observed-absent / unavailable` and folded read errors
/// into unavailability, and adversarial review correctly refused it —
/// "the read itself errored" and "the platform did not expose it" are the
/// distinction the ADR deliberately keeps, and a positively observed
/// absence is a **value**, which the state word must say.
pub enum Outcome {
    /// The adapter looked and determined something. Absence is a value in
    /// this family (ADR-C4): knowing the bytes do not exist is knowledge,
    /// and it renders under the same `observed` state as bytes do.
    Observed(ObservedValue),
    /// The adapter looked; the platform did not expose the answer. The
    /// replay adapter has no such case today — a regular file exposes
    /// everything it has — and the variant is kept because the vocabulary
    /// is ADR-C4's, not this adapter's to shrink.
    Unavailable {
        /// Why the platform could not expose the answer.
        reason: String,
    },
    /// The read itself errored. Distinct from unavailability, and never
    /// rendered as absence: treating could-not-look as
    /// looked-and-found-nothing is the fail-closed violation SAFE-005
    /// exists to prevent.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// What an `observed` outcome determined.
pub enum ObservedValue {
    /// Bytes, hex-encoded, uninterpreted.
    Bytes(String),
    /// A decimal quantity, as text.
    Decimal(String),
    /// A positively determined absence, carrying what established it.
    Absent {
        /// What established the absence.
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

/// The standing gated-surface list, rendered in every inspect answer —
/// observation answers and refusals alike — so what the inspector will not
/// say is stated in-band, never inferred from silence. Each entry carries
/// its state and names the authority that gates it: an open register issue
/// for the surfaces still undecided, and an accepted decision for a surface
/// whose question has been resolved. ADR-0011 (spec 4.3.0) resolved SI-12
/// by making no-cross-path-sameness-inference a standing rule, so
/// `same-device-claims` is `never-inferred` by decision rather than
/// `not-established` by open question — the prohibition is identical; its
/// authority changed from a question to an answer, and citing the retired
/// question would be the drift the register's sole-authority rule forbids.
pub const GATED: &[(&str, &str, &str)] = &[
    ("identity-strength", "not-established", "SI-28"),
    ("partition-table-state", "not-established", "SI-35"),
    ("same-device-claims", "never-inferred", "ADR-0011"),
];

/// The session-local selector for the one replayed object. One constant,
/// shared by both renderers, so the two cannot drift; the `0` is the
/// session index the boundary requires in place of any stable handle
/// (SI-27), and the parser refuses a second object per invocation.
pub const REPLAY_SELECTOR: &str = "replay:0";

// Linux does not assign these flags uniformly across its supported ABIs.
// Keep the three UAPI families explicit: using the generic value on MIPS or
// SPARC can omit O_NONBLOCK and turn a rebinding race into a blocking open.
#[cfg(all(
    target_os = "linux",
    any(
        test,
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "csky",
        target_arch = "hexagon",
        target_arch = "loongarch64",
        target_arch = "m68k",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "x86",
        target_arch = "x86_64"
    )
))]
pub(crate) const LINUX_GENERIC_REPLAY_OPEN_FLAGS: i32 = 0x0000_0800 | 0x0000_0100;
#[cfg(all(
    target_os = "linux",
    any(
        test,
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6"
    )
))]
pub(crate) const LINUX_MIPS_REPLAY_OPEN_FLAGS: i32 = 0x0000_0080 | 0x0000_0800;
#[cfg(all(
    target_os = "linux",
    any(test, target_arch = "sparc", target_arch = "sparc64")
))]
pub(crate) const LINUX_SPARC_REPLAY_OPEN_FLAGS: i32 = 0x0000_4000 | 0x0000_8000;

/// `O_NONBLOCK | O_NOCTTY` in the generic Linux userspace ABI family.
#[cfg(all(
    target_os = "linux",
    any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "csky",
        target_arch = "hexagon",
        target_arch = "loongarch64",
        target_arch = "m68k",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "x86",
        target_arch = "x86_64"
    )
))]
pub(crate) const REPLAY_OPEN_FLAGS: i32 = LINUX_GENERIC_REPLAY_OPEN_FLAGS;

/// `O_NONBLOCK | O_NOCTTY` in the Linux MIPS userspace ABI family.
#[cfg(all(
    target_os = "linux",
    any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6"
    )
))]
pub(crate) const REPLAY_OPEN_FLAGS: i32 = LINUX_MIPS_REPLAY_OPEN_FLAGS;

/// `O_NONBLOCK | O_NOCTTY` in the Linux SPARC userspace ABI family.
#[cfg(all(
    target_os = "linux",
    any(target_arch = "sparc", target_arch = "sparc64")
))]
pub(crate) const REPLAY_OPEN_FLAGS: i32 = LINUX_SPARC_REPLAY_OPEN_FLAGS;

#[cfg(all(
    target_os = "linux",
    not(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "csky",
        target_arch = "hexagon",
        target_arch = "loongarch64",
        target_arch = "m68k",
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "sparc",
        target_arch = "sparc64",
        target_arch = "x86",
        target_arch = "x86_64"
    ))
))]
compile_error!("fixture replay open flags have not been reviewed for this Linux target ABI");

/// `O_NONBLOCK | O_NOCTTY` in Darwin's userspace ABI. Darwin assigns the
/// Linux values to `O_EXCL | O_NOFOLLOW`; keeping this target-specific is a
/// safety property, not a portability cosmetic.
#[cfg(target_os = "macos")]
pub(crate) const REPLAY_OPEN_FLAGS: i32 = 0x0000_0004 | 0x0002_0000;

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
compile_error!("fixture replay open flags have not been reviewed for this Unix target");

/// Replay one regular file through the fixture-replay adapter.
///
/// # Errors
///
/// Refuses, with a typed value: anything the opened handle reports as not
/// a regular file; anything a pre-open look already shows is not one (a
/// hygiene check, raceable and therefore not the authority — the handle
/// is); and any object that cannot be opened at all. On Unix the open
/// itself is non-blocking, so a FIFO with no writer is refused instead of
/// hanging the inspector on an open that never returns.
pub fn replay(path: &Path) -> Result<Vec<Observation>, ReplayRefusal> {
    // Hygiene, not authority: in the common case a device or directory is
    // refused here, before any open touches it. A rebinding race can still
    // swap one in after this look, which is why the post-open fstat below
    // remains the check that decides.
    if let Ok(before) = std::fs::symlink_metadata(path)
        && !before.is_file()
    {
        return Err(not_a_regular_file());
    }

    let mut file = open_for_replay(path).map_err(|error| ReplayRefusal {
        state: "refused",
        reference: "SAFE-005",
        detail: format!("the object could not be opened: {error}"),
    })?;
    // fstat through the handle, not stat on the path: this answers "what
    // did I actually open", which no rebinding of the name can change.
    replay_handle(&mut file)
}

/// Open with flags that make hostile objects refusable rather than
/// hanging: on Unix, `O_NONBLOCK` (a no-op for regular-file reads once
/// the handle is verified) plus `O_NOCTTY`, so a FIFO or a
/// carrier-waiting device returns immediately and the handle check
/// refuses it.
fn open_for_replay(path: &Path) -> std::io::Result<std::fs::File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(REPLAY_OPEN_FLAGS)
            .open(path)
    }
    #[cfg(not(unix))]
    {
        std::fs::File::open(path)
    }
}

/// The handle-level half of [`replay`], split out so a test can hand it a
/// handle no path-based refusal would produce — a Windows directory handle
/// opened with backup semantics — and prove the fstat gate itself refuses,
/// on the platform where the path-based tests cannot reach it.
///
/// # Errors
///
/// Refuses anything the handle reports as not a regular file.
pub fn replay_handle(file: &mut std::fs::File) -> Result<Vec<Observation>, ReplayRefusal> {
    let metadata = file.metadata().map_err(|error| ReplayRefusal {
        state: "refused",
        reference: "SAFE-005",
        detail: format!("the opened handle would not describe itself: {error}"),
    })?;
    if !metadata.is_file() {
        return Err(not_a_regular_file());
    }
    let length = metadata.len();

    let mut observations = vec![Observation {
        subject: "object-length".to_owned(),
        attribution: REPLAY,
        outcome: Outcome::Observed(ObservedValue::Decimal(length.to_string())),
    }];

    for &(offset, count) in PROBES {
        probe(file, length, offset, count, &mut observations);
    }
    Ok(observations)
}

/// The regular-files-only refusal, shared by the hygiene look and the
/// handle authority so the two cannot drift apart in wording.
fn not_a_regular_file() -> ReplayRefusal {
    ReplayRefusal {
        state: "refused",
        reference: "SAFE-005",
        detail: "replay reads regular files only, and the object is something else; a \
                 device is not a fixture, and it is refused unread"
            .to_owned(),
    }
}

/// Probe one compiled range, splitting on the object's end so an absence
/// claim is never made about bytes that exist: a range that straddles the
/// end yields the existing prefix as observed bytes under an accurate
/// subject, and the remainder as a positively observed absence. An earlier
/// draft reported the whole straddling range absent, which was a false
/// absence claim for every partial overlap — the exact false-positive
/// class ADR-C4 forbids.
fn probe(
    file: &mut std::fs::File,
    length: u64,
    offset: u64,
    count: u64,
    observations: &mut Vec<Observation>,
) {
    let Some(end) = offset.checked_add(count) else {
        // Unreachable with the compiled probe list; kept so the property
        // survives if the list grows. A range whose end overflows u64
        // cannot exist in any object.
        observations.push(Observation {
            subject: format!("bytes[{offset}..{offset}+{count})"),
            attribution: REPLAY,
            outcome: Outcome::Observed(ObservedValue::Absent {
                reason: "the probed range's end does not fit in 64 bits, so no object can \
                         contain it"
                    .to_owned(),
            }),
        });
        return;
    };

    let readable_end = end.min(length);
    if offset < readable_end {
        let readable = readable_end - offset;
        let outcome = match read_exact_at(file, offset, readable) {
            Ok(bytes) => Outcome::Observed(ObservedValue::Bytes(hex(&bytes))),
            Err(error) => Outcome::Failed {
                error: format!("the read failed: {error}"),
            },
        };
        observations.push(Observation {
            subject: format!("bytes[{offset}..{readable_end})"),
            attribution: REPLAY,
            outcome,
        });
    }
    // The absent record covers exactly the asked-about bytes that do not
    // exist — from the later of the probe's own start and the object's
    // end. An earlier draft started at the object's end unconditionally,
    // which for a wholly-beyond probe made the subject claim an answer
    // about bytes nobody asked after.
    let absent_start = offset.max(readable_end);
    if absent_start < end {
        observations.push(Observation {
            subject: format!("bytes[{absent_start}..{end})"),
            attribution: REPLAY,
            outcome: Outcome::Observed(ObservedValue::Absent {
                reason: format!(
                    "the object ends at byte {length}; bytes from {absent_start} do not \
                     exist on it — known from the handle's own length, not from a failed \
                     read"
                ),
            }),
        });
    }
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

/// Render one outcome as JSON. Absence renders under the `observed` state
/// — the value family, per ADR-C4 — discriminated by an `absence` key
/// where present values carry `value`; `unavailable` and `failed` are the
/// two distinct non-answers.
fn outcome_json(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Observed(ObservedValue::Bytes(value) | ObservedValue::Decimal(value)) => {
            format!(
                "{{\"state\":\"observed\",\"value\":{}}}",
                json_escaped(value)
            )
        }
        Outcome::Observed(ObservedValue::Absent { reason }) => format!(
            "{{\"state\":\"observed\",\"absence\":{}}}",
            json_escaped(reason)
        ),
        Outcome::Unavailable { reason } => format!(
            "{{\"state\":\"unavailable\",\"reason\":{}}}",
            json_escaped(reason)
        ),
        Outcome::Failed { error } => {
            format!("{{\"state\":\"failed\",\"error\":{}}}", json_escaped(error))
        }
    }
}

/// Render the gated-surface list as JSON — part of every inspect answer.
#[must_use]
pub fn gated_json() -> String {
    let entries: Vec<String> = GATED
        .iter()
        .map(|(surface, state, gate)| {
            format!(
                "{{\"surface\":{surface},\"state\":{state},\"gate\":{gate}}}",
                surface = json_escaped(surface),
                state = json_escaped(state),
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
        "{{\"selector\":{selector},\"observations\":[{observations}],\"gated\":{gated}}}",
        selector = json_escaped(REPLAY_SELECTOR),
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
         \"detail\":{detail}}},\"observations\":[],\"gated\":{gated},\"reach\":{reach}}}",
        reference = json_escaped(platform_adapter_package()),
        detail = json_escaped(NO_ADAPTER_DETAIL),
        gated = gated_json(),
        reach = crate::reach::reach_json(),
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

/// Render one outcome for humans, with the same state words as JSON so
/// the two modes cannot teach different vocabularies.
fn outcome_human(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Observed(ObservedValue::Bytes(value) | ObservedValue::Decimal(value)) => {
            format!("observed {value}")
        }
        Outcome::Observed(ObservedValue::Absent { reason }) => {
            format!("observed absence — {reason}")
        }
        Outcome::Unavailable { reason } => format!("unavailable — {reason}"),
        Outcome::Failed { error } => format!("failed — {error}"),
    }
}

/// The gated list as a standalone human block, for answers assembled
/// outside this module — the replay refusal carries it too.
#[must_use]
pub fn gated_block() -> String {
    let mut out = String::new();
    gated_human(&mut out);
    out
}

/// Render the gated list for humans.
fn gated_human(out: &mut String) {
    use std::fmt::Write as _;
    let _ = writeln!(out, "  gated (the inspector will not say, and names why):");
    for (surface, state, gate) in GATED {
        let _ = writeln!(out, "    {surface}: {state} ({gate})");
    }
}

/// Render a replayed observation set for humans.
#[must_use]
pub fn replay_human(observations: &[Observation]) -> String {
    use std::fmt::Write as _;
    let mut out = format!(
        "inspect (fixture-replay adapter; bytes labelled by who read them, never \
         classified)\n  selector: {REPLAY_SELECTOR}\n"
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
    crate::reach::reach_human(&mut out);
    out
}
