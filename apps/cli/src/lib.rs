//! `partman` — the WP-035 read-only CLI chassis.
//!
//! This library is the evidence instrument's frame and, since increment 4,
//! its first observing surface: structured argument parsing, a documented
//! exit-code contract, a schema-versioned JSON envelope, a typed refusal
//! vocabulary, and adapter-attributed observation records over replayed
//! regular files. It remains forbidden every surface the spec-issue
//! register gates — no identity strength, no partition-table state, no
//! typed topology node, no hash, no plan. Every gated field represented by an
//! inspect answer travels in-band as a typed refusal naming its issue or
//! accepted decision. The reserved inventory, topology, and capability
//! commands likewise name the authority that prevents an honest payload;
//! mutation/planning command words and hash/plan types are absent rather than
//! represented as refusals. The boundary is in `docs/work-packages/WP-035.md`.
//!
//! Two properties are guarded mechanically, and each guard's exact reach is
//! stated, because an overstated guard sentence is how this repository's
//! recorded defects shipped:
//!
//! - **The shipped dependency closure is empty**, asserted through `cargo
//!   metadata`, so no hash or plan implementation can arrive from outside
//!   this crate. What that guard cannot see — `std`'s own hashers used
//!   deliberately in-crate — is held off by [`Outcome`]'s Tier-1 compile-time
//!   ambiguity proof and, past that, is a named review obligation rather than
//!   a claimed impossibility.
//! - **No unversioned JSON.** Every JSON emission is wrapped in one envelope
//!   carrying [`ENVELOPE_SCHEMA`] (MODEL-003). Domain payloads — inventory,
//!   topology, capability data — are absent from the output surface
//!   entirely, not emitted provisionally; their schemas belong to WP-010's
//!   MODEL-003 regime and are gated by the register.
//!
//! Output discipline: no ANSI sequence is emitted anywhere, in any mode, so
//! `NO_COLOR` (CLI-008) is honored by having nothing to disable. If colour
//! is ever added it arrives behind `NO_COLOR` and non-TTY detection, and the
//! no-ANSI test moves from "always" to "when disabled".
//!
//! I/O reach, restated each time it grows (increments 3 and 4 grew it): the
//! shipped binary opens no socket and reads no environment variable. Its
//! file-system reads and process launches are exactly two: the doctor's
//! existence checks and `--version` probes of roster tools at compiled
//! absolute paths, and `inspect --replay`'s read of one caller-named
//! regular file. A device named to `--replay` is refused **unread**: a
//! pre-open look refuses it before any open in the common case, and under
//! a rebinding race it is opened read-only at most long enough for the
//! handle to identify itself, then refused with no byte read — no command
//! reads a block device, and nothing opens one with write intent. The
//! exact statements live in [`doctor`]'s and [`inspect`]'s module docs,
//! beside the code they describe.

use std::ffi::OsString;

pub mod doctor;
pub mod facts;
pub mod inspect;
pub mod reach;

/// The schema identifier every JSON emission carries.
///
/// The `/0` is a real claim, not decoration: major version 0, provisional,
/// documented as free to change until CLI-001's stable schema regime exists
/// (WP-010 delivers MODEL-003's versioned schemas for domain payloads; this
/// envelope is the chassis's own surface and versions independently).
pub const ENVELOPE_SCHEMA: &str = "partman.cli.envelope/0";

/// The version `partman version` reports — the workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The command produced its answer.
pub const EXIT_OK: u8 = 0;

/// The structured parser refused the arguments. Nothing was interpreted.
pub const EXIT_USAGE: u8 = 2;

/// A surface refused with a typed value. The refusal is the answer: it is on
/// stdout, machine-readable under `--json`, and never merely this code.
pub const EXIT_REFUSAL: u8 = 3;

/// What one run of the binary produced. Pure data so tests assert on it
/// without spawning a process; the binary's `main` prints it verbatim.
///
/// This type deliberately does not implement `Hash`, and that is a guard,
/// not an omission — the assignment forbids a hash function reachable from
/// inspector output. A Tier-1 ambiguity assertion fails compilation if a
/// `Hash` implementation is added.
pub struct Outcome {
    /// Bytes for stdout, already rendered. Empty when nothing belongs there.
    pub stdout: String,
    /// Bytes for stderr. Only usage refusals in human mode use it.
    pub stderr: String,
    /// The documented exit code.
    pub code: u8,
}

/// The commands the chassis knows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    /// Print the command list and the exit-code contract.
    Help,
    /// Print the version.
    Version,
    /// The instrument's surface: observation records, labelled by the
    /// adapter that produced them. `--replay <file>` runs the
    /// fixture-replay adapter over one regular file; without it, the
    /// answer states that no device adapter exists on this platform yet.
    /// Every register-gated surface travels in-band as a typed refusal.
    Inspect,
    /// The redacted diagnostics bundle: this build's identity and surface
    /// states, admitted field-by-field through the deny-by-default
    /// allowlist, on stdout.
    ExportDiagnostics,
    /// The dependency doctor: roster tools at compiled absolute paths —
    /// present, version, in or out of the tested range — as facts with
    /// provenance, never capability verdicts.
    Doctor,
    /// Immutable technology limits with the basis for each: FS-007's
    /// inputs, with the blocked-reason surface left to WP-050.
    Facts,
    /// Reserved canonical inventory request. It exists only to return its
    /// typed register-gate refusal; no inventory payload is representable.
    Inventory,
    /// Reserved canonical topology request. It exists only to return its
    /// typed register-gate refusal; no snapshot payload is representable.
    Topology,
    /// Reserved per-target capability request. It exists only to return the
    /// shared-engine requirement refusal; no verdict payload is implemented or
    /// emitted by this chassis.
    Capabilities,
}

/// Every command, for contract-wide tests. Kept beside [`Command::name`],
/// whose wildcard-free match is what brings an author here when a variant is
/// added: the new variant fails that match until it is handled, and
/// extending this array in the same edit is a review obligation — stated as
/// one, not claimed as a compiler guarantee.
pub const ALL_COMMANDS: [Command; 9] = [
    Command::Help,
    Command::Version,
    Command::Inspect,
    Command::ExportDiagnostics,
    Command::Doctor,
    Command::Facts,
    Command::Inventory,
    Command::Topology,
    Command::Capabilities,
];

impl Command {
    /// The name rendered into envelopes and matched from argv.
    ///
    /// Wildcard-free on purpose; see [`ALL_COMMANDS`].
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Version => "version",
            Self::Inspect => "inspect",
            Self::ExportDiagnostics => "export-diagnostics",
            Self::Doctor => "doctor",
            Self::Facts => "facts",
            Self::Inventory => "inventory",
            Self::Topology => "topology",
            Self::Capabilities => "capabilities",
        }
    }
}

/// One parsed invocation: a command, the output mode, and inspect's
/// optional replay object.
struct Invocation {
    command: Command,
    json: bool,
    replay: Option<String>,
}

/// Why the parser refused. The exact spelling of the refused token travels
/// with the refusal so the user is told what was rejected, not what the
/// parser guessed.
enum UsageRefusal {
    /// No command word was given.
    MissingCommand,
    /// A command word the chassis does not know.
    UnknownCommand(String),
    /// A flag the chassis does not know.
    UnknownFlag(String),
    /// A second command word after one was already accepted. Carries the
    /// second token's exact spelling: an earlier draft reported the
    /// canonical name of the command it aliases, which told the user a
    /// known command was unknown and showed a word they never typed.
    SecondCommand(String),
    /// `--replay` with no value after it.
    ReplayNeedsValue,
    /// `--replay` given twice. Two objects is an ambiguity, not a list;
    /// unlike `--json`, repetition here would change what the invocation
    /// means, so it is refused like a second command word.
    ReplayTwice,
    /// `--replay` alongside a command that is not `inspect`.
    ReplayNeedsInspect(String),
    /// An argument that is not valid Unicode, rendered lossily. Owned as a
    /// refusal because the alternative — `std::env::args()` — panics, and a
    /// panic is an undocumented exit code wearing a stack trace.
    NotUnicode(String),
}

impl UsageRefusal {
    /// One human-actionable sentence.
    fn detail(&self) -> String {
        match self {
            Self::MissingCommand => {
                "no command given; run `partman help` for the command list".to_owned()
            }
            Self::UnknownCommand(word) => {
                format!("unknown command `{word}`; run `partman help` for the command list")
            }
            Self::UnknownFlag(word) => {
                format!("unknown flag `{word}`; run `partman help` for the flag list")
            }
            Self::SecondCommand(word) => {
                format!("second command word `{word}`; one command per invocation")
            }
            Self::NotUnicode(lossy) => {
                format!("argument `{lossy}` is not valid Unicode; arguments are matched exactly")
            }
            Self::ReplayNeedsValue => {
                "flag `--replay` needs a value: the file to replay".to_owned()
            }
            Self::ReplayTwice => {
                "flag `--replay` given twice; one object per invocation".to_owned()
            }
            Self::ReplayNeedsInspect(command) => {
                format!("flag `--replay` belongs to inspect, not to `{command}`")
            }
        }
    }
}

/// A typed refusal: the machine-readable statement of why an answer was not
/// produced.
///
/// This vocabulary is the chassis's first real feature and the model for
/// every later gated surface. It constructs `not-implemented` for work with
/// an assigned future owner and `not-established` where open register
/// questions prevent an honest payload.
pub struct Refusal {
    /// The state word, from the vocabulary above.
    pub state: &'static str,
    /// What a reader follows to see the gate: a governing assignment,
    /// requirement, spec issue, or accepted decision; possibly a
    /// comma-separated set.
    pub reference: &'static str,
    /// One sentence a human can act on.
    pub detail: &'static str,
}

/// The refusal the diagnostics bundle carries in place of discovery
/// evidence. In-band rather than omitted: an absent field would be
/// indistinguishable from "there was nothing to report", and INV-007's
/// redacted evidence view is a real obligation this bundle will carry — the
/// refusal names when.
const DISCOVERY_EVIDENCE_REFUSAL: Refusal = Refusal {
    state: "not-implemented",
    reference: "WP-W100, WP-L100, WP-M100",
    detail: "the diagnostics bundle admits compile-time data only, so it carries no \
             discovery evidence; observation records exist as per-run inspect output, \
             and evidence from real devices reaches this bundle only when a platform \
             adapter package lands it here through the same field allowlist",
};

const INVENTORY_REFUSAL: Refusal = Refusal {
    state: "not-established",
    reference: "SI-27, SI-28, SI-35",
    detail: "a canonical inventory payload is not established: node naming and collision \
             behavior (SI-27), identity strength (SI-28), and partition-table state (SI-35) \
             remain open; use partman inspect for adapter-attributed observations",
};

const TOPOLOGY_REFUSAL: Refusal = Refusal {
    state: "not-established",
    reference: "SI-27, SI-28, SI-34, SI-35",
    detail: "a versioned TopologySnapshot payload is not established: node naming (SI-27), \
             identity strength (SI-28), protection placement (SI-34), and partition-table \
             state (SI-35) remain open; no partial snapshot is emitted",
};

const CAPABILITIES_REFUSAL: Refusal = Refusal {
    state: "not-implemented",
    reference: "CAP-005",
    detail: "per-target capability payloads are not implemented: CAP-005 requires the CLI to \
             use the shared capability engine delivered by WP-050; doctor and facts report \
             inputs, never verdicts",
};

/// One field the diagnostics allowlist admits.
///
/// **This enum is the redaction mechanism.** The bundle is rendered by
/// iterating [`DIAGNOSTIC_ALLOWLIST`], and each variant renders itself from
/// data this crate owns at compile time — there is no API that accepts a
/// caller-supplied key or value, so deny-by-default is the builder's type
/// rather than a filter applied afterwards. An allowlist needs no knowledge
/// of what the denied fields are, which is what keeps it model-independent:
/// when observation records exist (increment 4), they enter the bundle only
/// by gaining a variant here, a visible reviewed edit that the exact-field
/// test pins.
///
/// The deny-floor this must never fall below is SEC-006's field list —
/// device serials, paths, labels, usernames, keys, file names — adopted as
/// the categories the redaction tests probe for. No variant may ever render
/// a value in those categories un-redacted.
#[derive(Clone, Copy)]
enum DiagnosticField {
    /// The workspace package version this binary was built as.
    ToolVersion,
    /// The envelope schema every JSON emission carries.
    EnvelopeSchema,
    /// The compile-time target: OS family and architecture. Constants from
    /// `std::env::consts`, not probes — nothing is read from the host.
    BuildTarget,
    /// Every command and the state it answers in, so a bug report says what
    /// the build could and could not do without the reporter guessing.
    CommandSurface,
    /// The documented exit-code contract, from the same constants the
    /// binary returns.
    ExitContract,
    /// The typed refusal standing where discovery evidence will be.
    DiscoveryEvidence,
}

/// The complete allowlist, in rendering order. Extending it is a reviewed
/// decision: the exact-field test pins these keys as literals.
const DIAGNOSTIC_ALLOWLIST: [DiagnosticField; 6] = [
    DiagnosticField::ToolVersion,
    DiagnosticField::EnvelopeSchema,
    DiagnosticField::BuildTarget,
    DiagnosticField::CommandSurface,
    DiagnosticField::ExitContract,
    DiagnosticField::DiscoveryEvidence,
];

impl DiagnosticField {
    /// The bundle key. Stable within the envelope's major version 0.
    fn key(self) -> &'static str {
        match self {
            Self::ToolVersion => "tool-version",
            Self::EnvelopeSchema => "envelope-schema",
            Self::BuildTarget => "build-target",
            Self::CommandSurface => "commands",
            Self::ExitContract => "exit-codes",
            Self::DiscoveryEvidence => "discovery-evidence",
        }
    }

    /// The JSON value for this field. Every arm renders compile-time data;
    /// no runtime value exists in this increment for any arm to leak.
    fn value_json(self) -> String {
        match self {
            Self::ToolVersion => json_escaped(VERSION),
            Self::EnvelopeSchema => json_escaped(ENVELOPE_SCHEMA),
            Self::BuildTarget => format!(
                "{{\"os\":{os},\"arch\":{arch}}}",
                os = json_escaped(std::env::consts::OS),
                arch = json_escaped(std::env::consts::ARCH),
            ),
            Self::CommandSurface => {
                let entries: Vec<String> = ALL_COMMANDS
                    .iter()
                    .map(|command| {
                        format!(
                            "{{\"name\":{name},\"state\":{state}}}",
                            name = json_escaped(command.name()),
                            state = json_escaped(command_state(*command)),
                        )
                    })
                    .collect();
                format!("[{}]", entries.join(","))
            }
            Self::ExitContract => format!(
                "{{\"answered\":{EXIT_OK},\"usage-refusal\":{EXIT_USAGE},\
                 \"typed-refusal\":{EXIT_REFUSAL}}}"
            ),
            Self::DiscoveryEvidence => format!(
                "{{\"state\":{state},\"reference\":{reference},\"detail\":{detail}}}",
                state = json_escaped(DISCOVERY_EVIDENCE_REFUSAL.state),
                reference = json_escaped(DISCOVERY_EVIDENCE_REFUSAL.reference),
                detail = json_escaped(DISCOVERY_EVIDENCE_REFUSAL.detail),
            ),
        }
    }

    /// The human rendering, one aligned block per field.
    fn value_human(self) -> String {
        match self {
            Self::ToolVersion => format!("  tool-version: {VERSION}\n"),
            Self::EnvelopeSchema => format!("  envelope-schema: {ENVELOPE_SCHEMA}\n"),
            Self::BuildTarget => format!(
                "  build-target: {} {}\n",
                std::env::consts::OS,
                std::env::consts::ARCH
            ),
            Self::CommandSurface => {
                let mut block = String::from("  commands:\n");
                for command in ALL_COMMANDS {
                    use std::fmt::Write as _;
                    writeln!(block, "    {}: {}", command.name(), command_state(command))
                        .expect("writing into a String cannot fail");
                }
                block
            }
            Self::ExitContract => format!(
                "  exit-codes: {EXIT_OK} answered, {EXIT_USAGE} usage refusal, \
                 {EXIT_REFUSAL} typed refusal\n"
            ),
            Self::DiscoveryEvidence => format!(
                "  discovery-evidence: {} ({})\n    {}\n",
                DISCOVERY_EVIDENCE_REFUSAL.state,
                DISCOVERY_EVIDENCE_REFUSAL.reference,
                DISCOVERY_EVIDENCE_REFUSAL.detail
            ),
        }
    }
}

/// What state a command answers in, for the diagnostics surface list.
///
/// The match is kept wildcard-free: a future surface must decide its state
/// here explicitly rather than inheriting an "answers" it never earned.
#[expect(
    clippy::match_same_arms,
    reason = "wildcard-free on purpose; a new command must decide its state here"
)]
fn command_state(command: Command) -> &'static str {
    match command {
        Command::Help
        | Command::Version
        | Command::ExportDiagnostics
        | Command::Doctor
        | Command::Facts => "answers",
        Command::Inspect => "answers",
        Command::Inventory | Command::Topology => "refuses:not-established",
        Command::Capabilities => "refuses:not-implemented",
    }
}

/// Render the diagnostics bundle as a JSON object, by iterating the
/// allowlist and nothing else.
fn diagnostics_json() -> String {
    let fields: Vec<String> = DIAGNOSTIC_ALLOWLIST
        .iter()
        .map(|field| format!("{}:{}", json_escaped(field.key()), field.value_json()))
        .collect();
    format!("{{{}}}", fields.join(","))
}

/// Render the diagnostics bundle for humans, same fields, same order.
fn diagnostics_human() -> String {
    let mut out = format!(
        "diagnostics (redacted by allowlist; {count} fields, all compile-time data)\n",
        count = DIAGNOSTIC_ALLOWLIST.len()
    );
    for field in DIAGNOSTIC_ALLOWLIST {
        out.push_str(&field.value_human());
    }
    out
}

/// Escape one string for placement between JSON quotes.
///
/// Emission only — this binary parses no JSON. Escaping covers the quote,
/// the backslash, and every Unicode control character (C0, DEL, and C1),
/// which is also what keeps a hostile string from smuggling ANSI into
/// `--json` output.
fn json_escaped(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => {
                use std::fmt::Write as _;
                write!(out, "\\u{:04x}", control as u32)
                    .expect("writing into a String cannot fail");
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Encode human-facing, caller-influenced text without terminal controls.
///
/// Backslash is escaped too, making the visible representation injective: a
/// literal `\n`, a newline, and U+FFFD remain three different inputs on the
/// terminal. JSON preserves the exact scalar value through its own escaping.
fn terminal_safe(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => {
                use std::fmt::Write as _;
                write!(out, "\\u{{{:04x}}}", control as u32)
                    .expect("writing into a String cannot fail");
            }
            other => out.push(other),
        }
    }
    out
}

/// Render the envelope around one outcome body.
///
/// `command` is `None` for a usage refusal, where no command was accepted —
/// rendered as JSON `null` rather than omitted, so the field list is fixed
/// and a consumer never branches on presence.
fn envelope(command: Option<Command>, body: &str) -> String {
    let command_json = command.map_or_else(|| "null".to_owned(), |c| json_escaped(c.name()));
    format!(
        "{{\"schema\":{schema},\"command\":{command_json},\"outcome\":{body}}}\n",
        schema = json_escaped(ENVELOPE_SCHEMA),
    )
}

/// The help text. The numeric exit codes are interpolated from the same
/// constants the binary returns, so a renumbering cannot desynchronize the
/// two; completeness of the documented set and the prose meanings remain
/// review obligations, and the contract test pins the literal values.
fn help_text() -> String {
    format!(
        "partman {VERSION} — read-only CLI chassis (WP-035)\n\
         \n\
         Not a usable partition manager, and must not be represented as one.\n\
         This chassis inspects caller-named regular files through inspect --replay,\n\
         has no native device adapter yet, and mutates nothing ever.\n\
         \n\
         Usage: partman [--json] <command>\n\
         \n\
         Commands:\n\
         \x20 help                 this text\n\
         \x20 version              the version, as a line or a JSON envelope\n\
         \x20 inspect              observation records, labelled by the adapter that\n\
         \x20                      produced them; register-gated surfaces travel in-band\n\
         \x20                      as typed refusals, and bytes are never classified\n\
         \x20 export-diagnostics   this build's identity and surface states, admitted\n\
         \x20                      field-by-field through a deny-by-default allowlist\n\
         \x20 doctor               roster tools at compiled absolute paths — present,\n\
         \x20                      version, in or out of the tested range; never a verdict\n\
         \x20 facts                immutable technology limits, each with its basis\n\
         \x20 inventory            reserved: typed refusal; no canonical payload exists\n\
         \x20 topology             reserved: typed refusal; no partial snapshot exists\n\
         \x20 capabilities         reserved: typed refusal; shared engine not implemented\n\
         \n\
         Flags:\n\
         \x20 --json               one schema-versioned JSON envelope on stdout\n\
         \x20                      ({ENVELOPE_SCHEMA}); provisional within major\n\
         \x20                      version 0; contains no ANSI sequences\n\
         \x20 --replay <file>      inspect only: replay one regular file through the\n\
         \x20                      fixture-replay adapter, verified through the opened\n\
         \x20                      handle — a device named here is refused unread\n\
         \n\
         Exit codes (documented contract, provisional within major version 0):\n\
         \x20 {EXIT_OK}  the command produced its answer\n\
         \x20 {EXIT_USAGE}  the structured parser refused the arguments\n\
         \x20 {EXIT_REFUSAL}  a surface refused with a typed value on stdout\n\
         \n\
         Output contains no ANSI sequences in any mode, so NO_COLOR is honored\n\
         by construction. Domain payloads (inventory, topology, capabilities)\n\
         are absent from every surface: each typed refusal names its governing\n\
         spec issue or requirement, and absence-with-refusal is the honest state.\n"
    )
}

/// Parse a structured token list. Every token is matched exactly; nothing is
/// interpreted, abbreviated, or passed onward.
///
/// A repeated `--json` is accepted deliberately — the flag is idempotent, so
/// repetition changes nothing — while a second command word is refused,
/// because it would change what the invocation means. The asymmetry is a
/// decision, recorded here so it does not read as an accident.
fn parse(arguments: &[String]) -> Result<Invocation, UsageRefusal> {
    let mut json = false;
    let mut command: Option<Command> = None;
    let mut replay: Option<String> = None;
    let mut tokens = arguments.iter();
    while let Some(token) = tokens.next() {
        match token.as_str() {
            "--json" => json = true,
            "--replay" => {
                if replay.is_some() {
                    return Err(UsageRefusal::ReplayTwice);
                }
                // A following flag is not a value: without this check the
                // flag swallowed `--json`, and the caller got human-mode
                // output plus a file-open refusal for a file named
                // `--json` — one token read two contradictory ways. A
                // genuinely dash-named file stays reachable as `./--name`.
                match tokens.next() {
                    Some(value) if !value.starts_with("--") => replay = Some(value.clone()),
                    _ => return Err(UsageRefusal::ReplayNeedsValue),
                }
            }
            "help" | "--help" | "-h" => set_command(&mut command, Command::Help, token)?,
            "version" | "--version" | "-V" => set_command(&mut command, Command::Version, token)?,
            "inspect" => set_command(&mut command, Command::Inspect, token)?,
            "export-diagnostics" => {
                set_command(&mut command, Command::ExportDiagnostics, token)?;
            }
            "doctor" => set_command(&mut command, Command::Doctor, token)?,
            "facts" => set_command(&mut command, Command::Facts, token)?,
            "inventory" => set_command(&mut command, Command::Inventory, token)?,
            "topology" => set_command(&mut command, Command::Topology, token)?,
            "capabilities" => set_command(&mut command, Command::Capabilities, token)?,
            flag if flag.starts_with('-') => {
                return Err(UsageRefusal::UnknownFlag(flag.to_owned()));
            }
            word => return Err(UsageRefusal::UnknownCommand(word.to_owned())),
        }
    }
    let Some(command) = command else {
        return Err(UsageRefusal::MissingCommand);
    };
    if replay.is_some() && command != Command::Inspect {
        return Err(UsageRefusal::ReplayNeedsInspect(command.name().to_owned()));
    }
    Ok(Invocation {
        command,
        json,
        replay,
    })
}

/// Record the command word, refusing a second one — with the second token's
/// exact spelling — rather than silently keeping either.
fn set_command(
    slot: &mut Option<Command>,
    command: Command,
    token: &str,
) -> Result<(), UsageRefusal> {
    if slot.is_some() {
        return Err(UsageRefusal::SecondCommand(token.to_owned()));
    }
    *slot = Some(command);
    Ok(())
}

/// The inspect command's outcome: the no-adapter statement, a replayed
/// observation set, or a typed refusal — each on stdout in both modes,
/// each carrying the standing gated list.
fn inspect_outcome(replay: Option<&str>, json: bool) -> Outcome {
    let Some(path) = replay else {
        return Outcome {
            stdout: if json {
                envelope(
                    Some(Command::Inspect),
                    &format!(
                        "{{\"kind\":\"ok\",\"inspect\":{}}}",
                        inspect::no_adapter_json()
                    ),
                )
            } else {
                inspect::no_adapter_human()
            },
            stderr: String::new(),
            code: EXIT_OK,
        };
    };
    match inspect::replay(std::path::Path::new(path)) {
        Ok(observations) => Outcome {
            stdout: if json {
                envelope(
                    Some(Command::Inspect),
                    &format!(
                        "{{\"kind\":\"ok\",\"inspect\":{}}}",
                        inspect::replay_json(&observations)
                    ),
                )
            } else {
                inspect::replay_human(&observations)
            },
            stderr: String::new(),
            code: EXIT_OK,
        },
        // The refusal is the answer, and the gated list travels with every
        // inspect answer — refusals included — so "what the inspector will
        // not say" is never inferred from silence.
        Err(refusal) => Outcome {
            stdout: if json {
                envelope(
                    Some(Command::Inspect),
                    &format!(
                        "{{\"kind\":\"refusal\",\"state\":{state},\"reference\":{reference},\
                         \"detail\":{detail},\"gated\":{gated}}}",
                        state = json_escaped(refusal.state),
                        reference = json_escaped(refusal.reference),
                        detail = json_escaped(&refusal.detail),
                        gated = inspect::gated_json(),
                    ),
                )
            } else {
                let mut text = format!(
                    "inspect: refused\n  state: {state}\n  reference: {reference}\n  \
                     detail: {detail}\n",
                    state = refusal.state,
                    reference = refusal.reference,
                    detail = terminal_safe(&refusal.detail),
                );
                text.push_str(&inspect::gated_block());
                text
            },
            stderr: String::new(),
            code: EXIT_REFUSAL,
        },
    }
}

/// Render a recognized command whose only honest answer is a typed refusal.
fn typed_refusal_outcome(command: Command, refusal: &Refusal, json: bool) -> Outcome {
    Outcome {
        stdout: if json {
            envelope(
                Some(command),
                &format!(
                    "{{\"kind\":\"refusal\",\"state\":{state},\"reference\":{reference},\
                     \"detail\":{detail}}}",
                    state = json_escaped(refusal.state),
                    reference = json_escaped(refusal.reference),
                    detail = json_escaped(refusal.detail),
                ),
            )
        } else {
            format!(
                "{command}: refused\n  state: {state}\n  reference: {reference}\n  detail: {detail}\n",
                command = command.name(),
                state = refusal.state,
                reference = refusal.reference,
                detail = refusal.detail,
            )
        },
        stderr: String::new(),
        code: EXIT_REFUSAL,
    }
}

/// Run one parsed invocation. Two arms are impure, each through its own
/// stated seam: the doctor's I/O goes through the injected launcher — how
/// Tier-1 tests keep their process set at `git` and the compile-time
/// `cargo` — and inspect's replay reads one caller-named regular file
/// through the verified handle. Every other arm is pure: no I/O, no
/// environment, no process.
fn run(invocation: &Invocation, launcher: &dyn doctor::ToolLauncher) -> Outcome {
    match invocation.command {
        Command::Help => Outcome {
            stdout: if invocation.json {
                envelope(
                    Some(Command::Help),
                    &format!(
                        "{{\"kind\":\"ok\",\"help\":{}}}",
                        json_escaped(&help_text())
                    ),
                )
            } else {
                help_text()
            },
            stderr: String::new(),
            code: EXIT_OK,
        },
        Command::Version => Outcome {
            stdout: if invocation.json {
                envelope(
                    Some(Command::Version),
                    &format!("{{\"kind\":\"ok\",\"version\":{}}}", json_escaped(VERSION)),
                )
            } else {
                format!("partman {VERSION}\n")
            },
            stderr: String::new(),
            code: EXIT_OK,
        },
        Command::Inspect => inspect_outcome(invocation.replay.as_deref(), invocation.json),
        Command::ExportDiagnostics => Outcome {
            stdout: if invocation.json {
                envelope(
                    Some(Command::ExportDiagnostics),
                    &format!("{{\"kind\":\"ok\",\"diagnostics\":{}}}", diagnostics_json()),
                )
            } else {
                diagnostics_human()
            },
            stderr: String::new(),
            code: EXIT_OK,
        },
        Command::Doctor => {
            let reports = doctor::examine(doctor::ROSTER, launcher);
            let empty = doctor::empty_roster_statement();
            Outcome {
                stdout: if invocation.json {
                    envelope(
                        Some(Command::Doctor),
                        &format!(
                            "{{\"kind\":\"ok\",\"doctor\":{}}}",
                            doctor::doctor_json(&reports, empty)
                        ),
                    )
                } else {
                    doctor::doctor_human(&reports, empty)
                },
                stderr: String::new(),
                code: EXIT_OK,
            }
        }
        Command::Facts => Outcome {
            stdout: if invocation.json {
                envelope(
                    Some(Command::Facts),
                    &format!("{{\"kind\":\"ok\",\"facts\":{}}}", facts::facts_json()),
                )
            } else {
                facts::facts_human()
            },
            stderr: String::new(),
            code: EXIT_OK,
        },
        Command::Inventory => {
            typed_refusal_outcome(Command::Inventory, &INVENTORY_REFUSAL, invocation.json)
        }
        Command::Topology => {
            typed_refusal_outcome(Command::Topology, &TOPOLOGY_REFUSAL, invocation.json)
        }
        Command::Capabilities => typed_refusal_outcome(
            Command::Capabilities,
            &CAPABILITIES_REFUSAL,
            invocation.json,
        ),
    }
}

/// Render a usage refusal. With `--json` the machine-readable form goes to
/// stdout; otherwise the sentence goes to stderr, where diagnostics belong.
fn usage_outcome(refusal: &UsageRefusal, json: bool) -> Outcome {
    if json {
        Outcome {
            stdout: envelope(
                None,
                &format!(
                    "{{\"kind\":\"usage-refusal\",\"detail\":{}}}",
                    json_escaped(&refusal.detail())
                ),
            ),
            stderr: String::new(),
            code: EXIT_USAGE,
        }
    } else {
        Outcome {
            stdout: String::new(),
            stderr: format!("partman: {}\n", terminal_safe(&refusal.detail())),
            code: EXIT_USAGE,
        }
    }
}

/// Parse and run one argument list through an injected launcher, producing
/// the outcome the binary prints. Public so every behavior — including the
/// doctor's, against a scripted launcher — is assertable without spawning a
/// process.
#[must_use]
pub fn dispatch_with(arguments: &[String], launcher: &dyn doctor::ToolLauncher) -> Outcome {
    match parse(arguments) {
        Ok(invocation) => run(&invocation, launcher),
        Err(refusal) => usage_outcome(&refusal, wants_json(arguments)),
    }
}

/// [`dispatch_with`] over the real system launcher — the binary's own path.
#[must_use]
pub fn dispatch(arguments: &[String]) -> Outcome {
    dispatch_with(arguments, &doctor::SystemLauncher)
}

/// The binary's real entry seam: raw `OsString` arguments, exactly as the
/// operating system delivers them.
///
/// `std::env::args()` panics on an argument that is not valid Unicode —
/// exit code 101 and a stack trace, neither of which is in the documented
/// contract — so the conversion is owned here and a non-Unicode argument
/// becomes an ordinary typed usage refusal instead of a crash. The `--json`
/// scan honors the flag when it appears among the tokens converted before
/// the refused one; the lossy rendering cannot become `--json` by accident,
/// because the replacement character is not `-`.
#[must_use]
pub fn dispatch_os(arguments: impl IntoIterator<Item = OsString>) -> Outcome {
    let mut converted: Vec<String> = Vec::new();
    for argument in arguments {
        match argument.into_string() {
            Ok(token) => converted.push(token),
            Err(raw) => {
                let refusal = UsageRefusal::NotUnicode(raw.to_string_lossy().into_owned());
                let json = wants_json(&converted);
                return usage_outcome(&refusal, json);
            }
        }
    }
    dispatch(&converted)
}

/// Whether an argument list asks for JSON output. Scanned independently of
/// the parser so a usage refusal still honors the flag: the parser stops at
/// the first refused token, and the output mode is a property of the whole
/// argument list rather than of how far the parser got.
fn wants_json(arguments: &[String]) -> bool {
    arguments.iter().any(|token| token == "--json")
}

#[cfg(test)]
mod tests;
