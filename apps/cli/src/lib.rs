//! `partman` — the WP-035 read-only CLI chassis.
//!
//! This library is the evidence instrument's frame, not the instrument:
//! structured argument parsing, a documented exit-code contract, a
//! schema-versioned JSON envelope, and a typed refusal vocabulary. It
//! inspects nothing yet — `partman inspect` refuses honestly rather than
//! printing something plausible — and it is forbidden every surface the
//! spec-issue register gates: no identity strength, no partition-table
//! state, no typed topology node, no hash, no plan. The boundary and the
//! gate behind each prohibition are in `docs/work-packages/WP-035.md`.
//!
//! Two properties are guarded mechanically, and each guard's exact reach is
//! stated, because an overstated guard sentence is how this repository's
//! recorded defects shipped:
//!
//! - **The shipped dependency closure is empty**, asserted through `cargo
//!   metadata`, so no hash or plan implementation can arrive from outside
//!   this crate. What that guard cannot see — `std`'s own hashers used
//!   deliberately in-crate — is held off by [`Outcome`]'s compile-fail
//!   non-`Hash` proof and, past that, is a named review obligation rather
//!   than a claimed impossibility.
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

use std::ffi::OsString;

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
/// inspector output. Implementing it fails this doctest:
///
/// ```compile_fail
/// fn requires_hash<T: std::hash::Hash>(_value: T) {}
/// fn hash_the_output(outcome: partman_cli::Outcome) {
///     requires_hash(outcome);
/// }
/// ```
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
    /// The instrument's surface. Refuses until increment 4 has observation
    /// records for it to print.
    Inspect,
}

/// Every command, for contract-wide tests. Kept beside [`Command::name`],
/// whose wildcard-free match is what brings an author here when a variant is
/// added: the new variant fails that match until it is handled, and
/// extending this array in the same edit is a review obligation — stated as
/// one, not claimed as a compiler guarantee.
pub const ALL_COMMANDS: [Command; 3] = [Command::Help, Command::Version, Command::Inspect];

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
        }
    }
}

/// One parsed invocation: a command plus the output mode.
struct Invocation {
    command: Command,
    json: bool,
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
        }
    }
}

/// A typed refusal: the machine-readable statement of why an answer was not
/// produced.
///
/// This vocabulary is the chassis's first real feature and the model for
/// every later gated surface. `not-implemented` is the only state this
/// increment constructs, because it is the only state it can construct
/// honestly; a `not-established` state carrying a spec-issue gate arrives
/// with the first surface the register actually gates (increment 4's
/// observation records), not before there is something for it to be true of.
struct Refusal {
    /// The state word, from the vocabulary above.
    state: &'static str,
    /// What a reader follows to see the gate: an assignment increment today,
    /// a spec-issue id when a register-gated surface exists.
    reference: &'static str,
    /// One sentence a human can act on.
    detail: &'static str,
}

/// The refusal `partman inspect` gives while there is nothing honest for it
/// to print.
const INSPECT_REFUSAL: Refusal = Refusal {
    state: "not-implemented",
    reference: "WP-035 increment 4",
    detail: "inspect arrives with increment 4's adapter-attributed observation records; \
             this chassis increment ships argument parsing, the exit-code contract, the \
             JSON envelope, and this refusal vocabulary, and printing a plausible empty \
             inspection instead of refusing would be a fake success path",
};

/// Escape one string for placement between JSON quotes.
///
/// Emission only — this binary parses no JSON. Escaping covers the quote,
/// the backslash, and every control byte below 0x20, which is also what
/// keeps a hostile string from smuggling ANSI (0x1b) into `--json` output.
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
            control if (control as u32) < 0x20 => {
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
        "partman {VERSION} — read-only CLI chassis (WP-035 increment 1)\n\
         \n\
         Not a usable partition manager, and must not be represented as one.\n\
         This chassis inspects nothing yet and mutates nothing ever.\n\
         \n\
         Usage: partman [--json] <command>\n\
         \n\
         Commands:\n\
         \x20 help       this text\n\
         \x20 version    the version, as a line or a JSON envelope\n\
         \x20 inspect    refuses with a typed value until observation records exist\n\
         \n\
         Flags:\n\
         \x20 --json     one schema-versioned JSON envelope on stdout ({ENVELOPE_SCHEMA});\n\
         \x20            provisional within major version 0; contains no ANSI sequences\n\
         \n\
         Exit codes (documented contract, provisional within major version 0):\n\
         \x20 {EXIT_OK}  the command produced its answer\n\
         \x20 {EXIT_USAGE}  the structured parser refused the arguments\n\
         \x20 {EXIT_REFUSAL}  a surface refused with a typed value on stdout\n\
         \n\
         Output contains no ANSI sequences in any mode, so NO_COLOR is honored\n\
         by construction. Domain payloads (inventory, topology, capabilities)\n\
         are absent from every surface: their schemas are gated by the\n\
         spec-issue register, and absence-with-refusal is the honest state.\n"
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
    for token in arguments {
        match token.as_str() {
            "--json" => json = true,
            "help" | "--help" | "-h" => set_command(&mut command, Command::Help, token)?,
            "version" | "--version" | "-V" => set_command(&mut command, Command::Version, token)?,
            "inspect" => set_command(&mut command, Command::Inspect, token)?,
            flag if flag.starts_with('-') => {
                return Err(UsageRefusal::UnknownFlag(flag.to_owned()));
            }
            word => return Err(UsageRefusal::UnknownCommand(word.to_owned())),
        }
    }
    command.map_or(Err(UsageRefusal::MissingCommand), |command| {
        Ok(Invocation { command, json })
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

/// Run one parsed invocation. Pure: no I/O, no environment, no process.
fn run(invocation: &Invocation) -> Outcome {
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
        Command::Inspect => refusal_outcome(Command::Inspect, &INSPECT_REFUSAL, invocation.json),
    }
}

/// Render a typed refusal. The refusal is the command's answer, so it goes
/// to stdout in both modes — never only an exit code, never a stderr string,
/// never a silent omission.
fn refusal_outcome(command: Command, refusal: &Refusal, json: bool) -> Outcome {
    let stdout = if json {
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
    };
    Outcome {
        stdout,
        stderr: String::new(),
        code: EXIT_REFUSAL,
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
            stderr: format!("partman: {}\n", refusal.detail()),
            code: EXIT_USAGE,
        }
    }
}

/// Parse and run one argument list, producing the outcome the binary
/// prints. Public so every behavior is assertable without spawning a
/// process.
#[must_use]
pub fn dispatch(arguments: &[String]) -> Outcome {
    match parse(arguments) {
        Ok(invocation) => run(&invocation),
        Err(refusal) => usage_outcome(&refusal, wants_json(arguments)),
    }
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
