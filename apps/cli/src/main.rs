//! `partman` — the WP-035 read-only CLI chassis.
//!
//! This binary is the evidence instrument's frame, not the instrument:
//! structured argument parsing, a documented exit-code contract, a
//! schema-versioned JSON envelope, and a typed refusal vocabulary. It
//! inspects nothing yet — `partman inspect` refuses honestly rather than
//! printing something plausible — and it is forbidden every surface the
//! spec-issue register gates: no identity strength, no partition-table
//! state, no typed topology node, no hash, no plan. The boundary and the
//! gate behind each prohibition are in `docs/work-packages/WP-035.md`.
//!
//! Two properties are structural rather than reviewed:
//!
//! - **The shipped dependency closure is empty.** No hash function is
//!   reachable from anything this binary renders, and no plan type exists in
//!   its reach, because nothing beyond `std` is linked at all. A test reads
//!   `cargo metadata` and fails the tier if a normal or build dependency
//!   appears.
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

use std::process::ExitCode;

/// The schema identifier every JSON emission carries.
///
/// The `/0` is a real claim, not decoration: major version 0, provisional,
/// documented as free to change until CLI-001's stable schema regime exists
/// (WP-010 delivers MODEL-003's versioned schemas for domain payloads; this
/// envelope is the chassis's own surface and versions independently).
const ENVELOPE_SCHEMA: &str = "partman.cli.envelope/0";

/// The version `partman version` reports — the workspace package version.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The command produced its answer.
const EXIT_OK: u8 = 0;

/// The structured parser refused the arguments. Nothing was interpreted.
const EXIT_USAGE: u8 = 2;

/// A surface refused with a typed value. The refusal is the answer: it is on
/// stdout, machine-readable under `--json`, and never merely this code.
const EXIT_REFUSAL: u8 = 3;

/// What one run of the binary produced. Pure data so tests assert on it
/// without spawning a process.
struct Outcome {
    /// Bytes for stdout, already rendered. Empty when nothing belongs there.
    stdout: String,
    /// Bytes for stderr. Only usage refusals in human mode use it.
    stderr: String,
    /// The documented exit code.
    code: u8,
}

/// The commands the chassis knows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Command {
    /// Print the command list and the exit-code contract.
    Help,
    /// Print the version.
    Version,
    /// The instrument's surface. Refuses until increment 4 has observation
    /// records for it to print.
    Inspect,
}

impl Command {
    /// The name rendered into envelopes and matched from argv.
    fn name(self) -> &'static str {
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

/// Why the parser refused. The exact spelling travels with the refusal so
/// the user is told what was rejected, not what the parser guessed.
enum UsageRefusal {
    /// No command word was given.
    MissingCommand,
    /// A command word the chassis does not know.
    UnknownCommand(String),
    /// A flag the chassis does not know.
    UnknownFlag(String),
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

/// The help text. The exit-code contract is rendered from the constants so
/// this text cannot document codes the binary does not return.
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
fn parse(arguments: &[String]) -> Result<Invocation, UsageRefusal> {
    let mut json = false;
    let mut command: Option<Command> = None;
    for token in arguments {
        match token.as_str() {
            "--json" => json = true,
            "help" | "--help" | "-h" => set_command(&mut command, Command::Help)?,
            "version" | "--version" | "-V" => set_command(&mut command, Command::Version)?,
            "inspect" => set_command(&mut command, Command::Inspect)?,
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

/// Record the command word, refusing a second one rather than silently
/// keeping either.
fn set_command(slot: &mut Option<Command>, command: Command) -> Result<(), UsageRefusal> {
    if slot.is_some() {
        return Err(UsageRefusal::UnknownCommand(command.name().to_owned()));
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

/// Parse and run, producing the outcome `main` prints. Split from `main` so
/// every behavior is assertable without spawning a process.
fn dispatch(arguments: &[String]) -> Outcome {
    match parse(arguments) {
        Ok(invocation) => run(&invocation),
        Err(refusal) => {
            // The parser stops at the first refused token, so `--json` after
            // the offending token is unknown; scanning for the flag keeps the
            // output mode a property of the whole argument list rather than
            // of how far the parser got.
            let json = arguments.iter().any(|token| token == "--json");
            usage_outcome(&refusal, json)
        }
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = dispatch(&arguments);
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    ExitCode::from(outcome.code)
}

#[cfg(test)]
mod tests;
