//! Chassis tests. Every behavior is asserted through [`dispatch_with`] over
//! a scripted launcher, or [`dispatch_os`], as pure data. The only
//! executable classes tests launch are `git` and the compile-time-selected
//! `cargo`: Cargo is the structural-dependency oracle; Git supplies both the
//! successful real-launch subject and an intentional nonzero subject. No
//! Tier-1 test ever launches a roster tool.

use super::doctor::{
    ProbeOutcome, ProbeReport, Resolution, SystemLauncher, TestedVersion, ToolLauncher, ToolSpec,
    doctor_human, doctor_json, examine, parse_version,
};
use super::facts::{FACTS, facts_human, facts_json};
use super::{
    ALL_COMMANDS, Command, ENVELOPE_SCHEMA, EXIT_OK, EXIT_REFUSAL, EXIT_USAGE, Outcome, Refusal,
    VERSION, dispatch_os, dispatch_with, envelope, help_text, json_escaped,
};
use std::path::Path;

/// A launcher that finds nothing and must never be asked to probe. The
/// contract-wide tests run every command through it, which is what keeps
/// the doctor's real I/O out of Tier 1.
struct NothingInstalled;

impl ToolLauncher for NothingInstalled {
    fn exists(&self, _path: &Path) -> bool {
        false
    }
    fn probe_version(&self, path: &Path) -> ProbeOutcome {
        panic!("probe of {} without a prior existence hit", path.display());
    }
    // A launch failure rather than a panic, deliberately: on the macOS CI
    // leg the contract-wide tests reach the enumeration through this
    // launcher, and the honest outcome of "nothing exists here" is a failed
    // launch the answer must carry as `failed` — never a fake empty machine.
    fn launch(&self, _path: &Path, _arguments: &[&str], _output_limit: usize) -> ProbeOutcome {
        ProbeOutcome::LaunchFailed("this launcher launches nothing".to_owned())
    }
}

/// [`dispatch_with`] over [`NothingInstalled`]: the pure dispatch every
/// contract-wide test uses.
fn fdispatch(arguments: &[String]) -> Outcome {
    dispatch_with(arguments, &NothingInstalled)
}

/// One shared enumeration of invocation shapes, so every contract-wide test
/// covers the same set. The command words are derived from [`ALL_COMMANDS`],
/// whose companion `match` in [`Command::name`] is wildcard-free — adding a
/// variant fails that match and brings the author here; extending the alias
/// and refused lists below is the review obligation that visit carries, an
/// obligation this comment states rather than a guarantee it claims.
fn every_invocation() -> Vec<Vec<String>> {
    let mut words: Vec<String> = ALL_COMMANDS
        .iter()
        .map(|command| command.name().to_owned())
        .collect();
    // Alias spellings and refused shapes, beyond what the enum can derive.
    for extra in [
        "--help",
        "-h",
        "--version",
        "-V",
        "frobnicate",
        "--frob",
        "--replay",
        "",
    ] {
        words.push(extra.to_owned());
    }
    let mut invocations = Vec::new();
    for word in words {
        for json in [false, true] {
            let mut arguments: Vec<String> = Vec::new();
            if !word.is_empty() {
                arguments.push(word.clone());
            }
            if json {
                arguments.push("--json".to_owned());
            }
            invocations.push(arguments);
        }
    }
    invocations
}

fn json_outcomes() -> Vec<Outcome> {
    every_invocation()
        .into_iter()
        .filter(|arguments| arguments.iter().any(|token| token == "--json"))
        .map(|arguments| fdispatch(&arguments))
        .collect()
}

// Requirements: MODEL-003, CLI-008
//   Every JSON emission, including usage refusals, is one well-formed envelope carrying the schema version; domain payloads are absent rather than emitted unversioned
// Evidence: every_json_emission_is_one_well_formed_schema_versioned_envelope
#[test]
fn every_json_emission_is_one_well_formed_schema_versioned_envelope() {
    for outcome in json_outcomes() {
        assert!(
            outcome.stderr.is_empty(),
            "--json puts the machine-readable answer on stdout alone"
        );
        let parsed: serde_json::Value = serde_json::from_str(&outcome.stdout)
            .expect("--json output must parse as JSON, or the envelope is a lie");
        assert_eq!(
            parsed.get("schema").and_then(serde_json::Value::as_str),
            Some(ENVELOPE_SCHEMA),
            "an emission without the schema version is unversioned JSON, which MODEL-003 forbids"
        );
        let outcome_object = parsed
            .get("outcome")
            .expect("the envelope's field list is fixed");
        assert!(
            outcome_object.get("kind").is_some(),
            "a consumer dispatches on `kind`, so it cannot be optional"
        );
    }
}

// Requirements: CLI-008
//   No output in any mode contains an ANSI escape byte, so NO_COLOR is honored by construction and --json output is ANSI-free
// Evidence: no_output_in_any_mode_contains_an_ansi_sequence
#[test]
fn no_output_in_any_mode_contains_an_ansi_sequence() {
    for arguments in every_invocation() {
        let outcome = fdispatch(&arguments);
        for (stream, text) in [("stdout", &outcome.stdout), ("stderr", &outcome.stderr)] {
            assert!(
                !text.contains('\u{1b}'),
                "{stream} of {arguments:?} contains an ANSI escape byte"
            );
        }
    }
}

// Requirements: SAFE-005, Section 12
//   Bare inspect answers with a typed no-adapter statement naming the platform package and the standing gated list carrying its register issues — never a plausible empty machine, and never silence about what the inspector will not say
// Evidence: inspect_answers_with_typed_statements_not_a_fake_topology
#[test]
fn inspect_answers_with_typed_statements_not_a_fake_topology() {
    let human = fdispatch(&["inspect".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    // Since increment 8 the bare answer is platform-dependent: Linux and
    // macOS have contracts and enumerate — through this test's launcher,
    // which launches nothing, so the macOS answer here is the enumeration's
    // typed `failed`, never a fake empty machine — while Windows carries
    // the typed no-adapter statement naming its recorded deferral. The
    // invariant this test exists for is unchanged: a typed statement or
    // real rows, never a plausible empty machine, and never silence about
    // what the inspector will not say.
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    assert!(
        human.stdout.contains("adapters: not-implemented"),
        "a platform without a contract must say so: {}",
        human.stdout
    );
    #[cfg(target_os = "macos")]
    assert!(
        human.stdout.contains("adapters: failed"),
        "a failed launch must be carried as failed, never rendered as an empty \
         machine: {}",
        human.stdout
    );
    for fragment in [
        "identity-strength: not-established (SI-28)",
        "partition-table-state: helper-authored (ADR-0014)",
        "same-device-claims: never-inferred (ADR-0011)",
    ] {
        assert!(
            human.stdout.contains(fragment),
            "the human answer must carry `{fragment}`: {}",
            human.stdout
        );
    }

    let json = fdispatch(&["inspect".to_owned(), "--json".to_owned()]);
    assert_eq!(json.code, EXIT_OK);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("the answer rides the ordinary envelope");
    let inspect_object = &parsed["outcome"]["inspect"];
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        assert_eq!(inspect_object["adapters"]["state"], "not-implemented");
        assert_eq!(
            inspect_object["adapters"]["reference"],
            super::inspect::platform_adapter_package(),
            "the statement names the package that changes it"
        );
        assert_eq!(
            inspect_object["observations"].as_array().map(Vec::len),
            Some(0),
            "no adapter ran and the answer says so rather than inventing records"
        );
    }
    #[cfg(target_os = "macos")]
    {
        assert_eq!(
            inspect_object["adapters"]["state"], "failed",
            "this test's launcher launches nothing, and the JSON answer must say \
             failed rather than invent devices or an empty machine"
        );
        assert_eq!(
            inspect_object["devices"].as_array().map(Vec::len),
            Some(0),
            "a failed enumeration reports no devices beside its typed state"
        );
    }
    // On Linux a contract exists. Either it listed devices, or it said in a
    // typed word why it did not — an empty `devices` array with no statement
    // beside it would be the plausible empty machine this test forbids.
    #[cfg(target_os = "linux")]
    {
        let listed = inspect_object["devices"].as_array().map_or(0, Vec::len);
        assert!(
            listed > 0 || !inspect_object["adapters"]["state"].is_null(),
            "an empty device list must carry a typed statement saying why: {}",
            json.stdout
        );
    }
    let gated = inspect_object["gated"]
        .as_array()
        .expect("the gated list is part of every inspect answer");
    assert_eq!(
        serde_json::Value::Array(gated.clone()),
        serde_json::json!([
            {"surface": "identity-strength", "state": "not-established", "gate": "SI-28"},
            {"surface": "partition-table-state", "state": "helper-authored", "gate": "ADR-0014"},
            {"surface": "same-device-claims", "state": "never-inferred", "gate": "ADR-0011"},
        ]),
        "the complete ordered gate contract is pinned: no duplicate, extra, or omitted entry"
    );
}

// Requirements: SAFE-005
//   Replay opens are non-blocking on every supported Unix target using that
//   target's reviewed ABI values; Linux itself has generic, MIPS, and SPARC
//   families, and Darwin differs again, so a shared literal is mechanically rejected;
//   a source-use guard keeps that constant wired into the actual open call
// Evidence: replay_open_flags_match_the_supported_target_abi
#[cfg(unix)]
#[test]
fn replay_open_flags_match_the_supported_target_abi() {
    #[cfg(target_os = "linux")]
    {
        assert_eq!(super::inspect::LINUX_GENERIC_REPLAY_OPEN_FLAGS, 0x0900);
        assert_eq!(super::inspect::LINUX_MIPS_REPLAY_OPEN_FLAGS, 0x0880);
        assert_eq!(super::inspect::LINUX_SPARC_REPLAY_OPEN_FLAGS, 0xc000);
    }
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
    assert_eq!(super::inspect::REPLAY_OPEN_FLAGS, 0x0900);
    #[cfg(all(
        target_os = "linux",
        any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6"
        )
    ))]
    assert_eq!(super::inspect::REPLAY_OPEN_FLAGS, 0x0880);
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "sparc", target_arch = "sparc64")
    ))]
    assert_eq!(super::inspect::REPLAY_OPEN_FLAGS, 0xc000);
    #[cfg(target_os = "macos")]
    assert_eq!(super::inspect::REPLAY_OPEN_FLAGS, 0x0002_0004);

    let source = include_str!("inspect.rs");
    assert_eq!(
        source.matches(".custom_flags(REPLAY_OPEN_FLAGS)").count(),
        1,
        "the reviewed target constant must remain wired into the replay open"
    );
}

// Requirements: CLI-005
//   The documented exit codes are pinned as literals — 0 ok, 2 usage refusal, 3 typed refusal — so renumbering a constant is a visible test edit, and the help text interpolates the same constants so a renumbering cannot desynchronize the two
// Evidence: exit_codes_match_the_contract_the_help_text_documents
#[test]
fn exit_codes_match_the_contract_the_help_text_documents() {
    // Literal pins first: without these, every other assertion in this file
    // compares the code against the same constant that produced it, and a
    // mutation of EXIT_REFUSAL to 0 — refusal indistinguishable from
    // success — left the whole suite green when tried.
    assert_eq!(EXIT_OK, 0, "the ok code is part of the documented contract");
    assert_eq!(
        EXIT_USAGE, 2,
        "the usage code is part of the documented contract"
    );
    assert_eq!(
        EXIT_REFUSAL, 3,
        "the refusal code is part of the documented contract"
    );

    assert_eq!(fdispatch(&["help".to_owned()]).code, EXIT_OK);
    assert_eq!(fdispatch(&["version".to_owned()]).code, EXIT_OK);
    assert_eq!(fdispatch(&["export-diagnostics".to_owned()]).code, EXIT_OK);
    assert_eq!(fdispatch(&["inspect".to_owned()]).code, EXIT_OK);
    assert_eq!(fdispatch(&["inventory".to_owned()]).code, EXIT_REFUSAL);
    assert_eq!(fdispatch(&["topology".to_owned()]).code, EXIT_REFUSAL);
    assert_eq!(fdispatch(&["capabilities".to_owned()]).code, EXIT_REFUSAL);
    assert_eq!(fdispatch(&["frobnicate".to_owned()]).code, EXIT_USAGE);
    assert_eq!(fdispatch(&[]).code, EXIT_USAGE);

    let help = help_text();
    for (code, phrase) in [
        (EXIT_OK, "the command produced its answer"),
        (EXIT_USAGE, "the structured parser refused the arguments"),
        (EXIT_REFUSAL, "a surface refused with a typed value"),
    ] {
        assert!(
            help.contains(&format!(" {code}  {phrase}")),
            "help must document exit code {code} as `{phrase}`"
        );
    }
}

// Requirements: MODEL-003, SAFE-005, Section 12
//   Reserved inventory, topology, and capability requests are recognized and
//   refuse on stdout at exit 3 with exact typed gates in both modes; their JSON
//   envelopes contain no domain payload key, so absence never masquerades as
//   an empty machine, snapshot, or verdict set
// Evidence: domain_requests_refuse_with_exact_typed_gates_and_no_payload
#[test]
fn domain_requests_refuse_with_exact_typed_gates_and_no_payload() {
    let cases = [
        (
            "inventory",
            "not-implemented",
            "SI-28, ADR-0014, ADR-0019",
            "a canonical inventory payload is decided but not consumed here: node naming \
             and collision behavior are ADR-0019's landed types and partition-table state \
             is helper-authored under ADR-0014 (never computed by this client), while \
             identity-strength attribution stays open (SI-28); no increment wires WP-010's \
             types into this chassis; use partman inspect for adapter-attributed \
             observations",
        ),
        (
            "topology",
            "not-implemented",
            "SI-28, ADR-0014, ADR-0016, ADR-0019",
            "the TopologySnapshot body is decided and typed (WP-010 increment 3): node \
             naming by ADR-0019, verdict placement by ADR-0016, and table state \
             helper-authored at validation by ADR-0014 — a valid snapshot is the helper's \
             to produce, never this unprivileged client's — while identity-strength \
             attribution stays open (SI-28); no partial snapshot is emitted",
        ),
        (
            "capabilities",
            "not-implemented",
            "CAP-005",
            "per-target capability payloads are not implemented: CAP-005 requires the CLI to \
             use the shared capability engine delivered by WP-050; doctor and facts report \
             inputs, never verdicts",
        ),
    ];

    for (command, state, reference, detail) in cases {
        let human = fdispatch(&[command.to_owned()]);
        assert_eq!(human.code, EXIT_REFUSAL);
        assert!(human.stderr.is_empty());
        assert_eq!(
            human.stdout,
            format!(
                "{command}: refused\n  state: {state}\n  reference: {reference}\n  detail: {detail}\n"
            )
        );

        let json = fdispatch(&[command.to_owned(), "--json".to_owned()]);
        assert_eq!(json.code, EXIT_REFUSAL);
        assert!(json.stderr.is_empty());
        let parsed: serde_json::Value =
            serde_json::from_str(&json.stdout).expect("typed refusal envelope");
        assert_eq!(
            parsed,
            serde_json::json!({
                "schema": ENVELOPE_SCHEMA,
                "command": command,
                "outcome": {
                    "kind": "refusal",
                    "state": state,
                    "reference": reference,
                    "detail": detail,
                }
            }),
            "the refusal envelope is exact and contains no domain payload"
        );
    }

    let help = help_text();
    for command in ["inventory", "topology", "capabilities"] {
        assert!(help.contains(command), "help names reserved `{command}`");
    }
}

// Requirements: SAFE-005
//   A token the parser does not recognize is refused with its exact logical spelling rather than guessed at: JSON round-trips the exact scalar string and human output uses a lossless injective visible encoding; a second command word carries the second token, never a canonical name the user did not type
// Evidence: unknown_tokens_are_refused_with_their_exact_spelling
#[test]
fn unknown_tokens_are_refused_with_their_exact_spelling() {
    let command = fdispatch(&["frobnicate".to_owned()]);
    assert_eq!(command.code, EXIT_USAGE);
    assert!(command.stderr.contains("unknown command `frobnicate`"));
    assert!(command.stdout.is_empty());

    let flag = fdispatch(&["--frob".to_owned(), "version".to_owned()]);
    assert_eq!(flag.code, EXIT_USAGE);
    assert!(flag.stderr.contains("unknown flag `--frob`"));

    // The second-command refusal carries the second token's exact spelling.
    // An earlier draft reported the canonical name — `partman version -V`
    // said "unknown command `version`", declaring a known command unknown
    // and showing a word the user never typed — and only the exit code was
    // asserted, which is how the misreport survived review.
    let doubled = fdispatch(&["version".to_owned(), "-V".to_owned()]);
    assert_eq!(doubled.code, EXIT_USAGE);
    assert!(
        doubled.stderr.contains("second command word `-V`"),
        "the refusal must name the token the user actually typed: {}",
        doubled.stderr
    );
    assert!(
        !doubled.stderr.contains("unknown command"),
        "a second command word is not an unknown command: {}",
        doubled.stderr
    );

    for (arguments, expected_detail) in [
        (
            vec!["frobnicate".to_owned(), "--json".to_owned()],
            "unknown command `frobnicate`; run `partman help` for the command list",
        ),
        (
            vec![
                "--frob".to_owned(),
                "version".to_owned(),
                "--json".to_owned(),
            ],
            "unknown flag `--frob`; run `partman help` for the flag list",
        ),
        (
            vec!["version".to_owned(), "-V".to_owned(), "--json".to_owned()],
            "second command word `-V`; one command per invocation",
        ),
    ] {
        let json = fdispatch(&arguments);
        assert_eq!(json.code, EXIT_USAGE);
        let parsed: serde_json::Value =
            serde_json::from_str(&json.stdout).expect("usage refusals are envelopes too");
        assert_eq!(parsed["outcome"]["kind"], "usage-refusal");
        assert_eq!(parsed["outcome"]["detail"], expected_detail);
        assert_eq!(
            parsed["command"],
            serde_json::Value::Null,
            "no command was accepted, and the field says so rather than being omitted"
        );
    }
}

// Requirements: SAFE-005, CLI-005
//   A non-Unicode argument is a typed usage refusal at the documented code, not a panic: std::env::args() would abort with an undocumented 101, so the binary's entry seam owns the conversion
// Evidence: a_non_unicode_argument_is_refused_not_a_panic
#[test]
fn a_non_unicode_argument_is_refused_not_a_panic() {
    let invalid = invalid_unicode_argument();

    let human = dispatch_os(vec![std::ffi::OsString::from("version"), invalid.clone()]);
    assert_eq!(human.code, EXIT_USAGE);
    assert!(
        human.stderr.contains("is not valid Unicode"),
        "the refusal must say what was wrong: {}",
        human.stderr
    );
    assert!(
        !human.stderr.contains('\u{1b}'),
        "a control byte beside invalid Unicode must not reach the terminal"
    );

    let json = dispatch_os(vec![
        std::ffi::OsString::from("--json"),
        std::ffi::OsString::from("version"),
        invalid,
    ]);
    assert_eq!(json.code, EXIT_USAGE);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout)
        .expect("a non-Unicode refusal under --json is an envelope like any other");
    assert_eq!(parsed["outcome"]["kind"], "usage-refusal");
}

/// An `OsString` that is deliberately not valid Unicode, built per platform
/// because the two encodings break differently: an unpaired UTF-16
/// surrogate on Windows, a bare 0xFF byte on Unix.
fn invalid_unicode_argument() -> std::ffi::OsString {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt as _;
        std::ffi::OsString::from_wide(&[0xD800, 0x001B])
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        std::ffi::OsString::from_vec(vec![0xFF, 0x1B])
    }
}

// Requirements: CLI-008
//   Quotes, backslashes, and control bytes in any rendered string are escaped, so no value can break the envelope's structure or smuggle a control byte into --json output
// Evidence: hostile_strings_cannot_break_the_envelope
#[test]
fn hostile_strings_cannot_break_the_envelope() {
    let hostile = [
        "plain",
        "with \"quotes\" and \\backslashes\\",
        "newline\nreturn\rtab\t",
        "escape byte \u{1b}[31m, C1 CSI \u{009b}, DEL \u{007f}, and null \u{0}",
        "ünïcode 分区 🗜",
        "{\"kind\":\"ok\"}",
    ];
    for value in hostile {
        let escaped = json_escaped(value);
        let wrapped: serde_json::Value = serde_json::from_str(&escaped)
            .unwrap_or_else(|error| panic!("escaping {value:?} produced invalid JSON: {error}"));
        assert_eq!(
            wrapped.as_str(),
            Some(value),
            "escaping must round-trip {value:?} exactly"
        );

        let body = format!("{{\"kind\":\"ok\",\"probe\":{escaped}}}");
        let enveloped = envelope(Some(Command::Version), &body);
        let parsed: serde_json::Value = serde_json::from_str(&enveloped)
            .expect("a hostile value must not break the surrounding envelope");
        assert_eq!(parsed["outcome"]["probe"].as_str(), Some(value));
    }
}

// Requirements: CLI-008
//   Caller-controlled command, flag, second-command, replay-value, and
//   non-Unicode refusal text cannot inject C0, DEL, or C1 controls into human
//   output; when a refusal emits a caller token, JSON preserves that exact value
//   only as escapes, while replay refusals deliberately do not echo paths
// Evidence: hostile_arguments_cannot_inject_terminal_controls
#[test]
fn hostile_arguments_cannot_inject_terminal_controls() {
    let hostile = "\u{1b}[31mline\nnext\r\t\u{007f}\u{009b}\\n".to_owned();
    let hostile_value = hostile.clone();
    let cases = [
        (vec![hostile.clone()], 1, true),
        (vec![format!("--{hostile}")], 1, true),
        (vec!["version".to_owned(), hostile.clone()], 1, true),
        (
            vec!["inspect".to_owned(), "--replay".to_owned(), hostile],
            8,
            false,
        ),
    ];

    for (arguments, human_line_count, human_echoes_input) in cases {
        for json in [false, true] {
            let mut invocation = arguments.clone();
            if json {
                invocation.push("--json".to_owned());
            }
            let outcome = fdispatch(&invocation);
            let rendered = format!("{}{}", outcome.stdout, outcome.stderr);
            assert!(
                rendered.ends_with('\n'),
                "every outcome has one record terminator"
            );
            assert_eq!(
                rendered
                    .chars()
                    .filter(|character| *character == '\n')
                    .count(),
                if json { 1 } else { human_line_count },
                "hostile input must not add record boundaries: {invocation:?}"
            );
            let body = rendered
                .strip_suffix('\n')
                .expect("the final structural newline was just asserted");
            for character in body.chars().filter(|character| *character != '\n') {
                assert!(
                    !character.is_control(),
                    "{invocation:?} emitted control U+{:04X}",
                    character as u32
                );
            }
            if json {
                let parsed = serde_json::from_str::<serde_json::Value>(&outcome.stdout)
                    .expect("escaped hostile output remains one JSON envelope");
                let detail = parsed["outcome"]["detail"]
                    .as_str()
                    .expect("a usage or replay refusal carries detail");
                if human_echoes_input {
                    assert!(
                        detail.contains(&hostile_value),
                        "an emitted caller token must round-trip exactly through JSON"
                    );
                } else {
                    assert!(
                        !detail.contains(&hostile_value),
                        "replay refusals deliberately do not echo caller paths"
                    );
                }
            } else if human_echoes_input {
                for visible in [
                    "\\u{001b}",
                    "\\n",
                    "\\r",
                    "\\t",
                    "\\u{007f}",
                    "\\u{009b}",
                    "\\\\n",
                ] {
                    assert!(
                        body.contains(visible),
                        "human refusal must retain hostile input as visible {visible:?}: {body}"
                    );
                }
            }
        }
    }
}

// Requirements: CLI-008
//   Human rendering is an injective visible encoding: controls cannot execute,
//   and neither a literal escape spelling nor U+FFFD can alias a control input
// Evidence: human_terminal_encoding_is_injective_for_controls_and_backslashes
#[test]
fn human_terminal_encoding_is_injective_for_controls_and_backslashes() {
    let encoded = super::terminal_safe("\\\n\r\t\u{1b}\u{7f}\u{9b}\u{fffd}");
    assert_eq!(
        encoded,
        concat!(
            "\\\\",
            "\\n",
            "\\r",
            "\\t",
            "\\u{001b}",
            "\\u{007f}",
            "\\u{009b}",
            "\u{fffd}"
        )
    );
    assert_ne!(super::terminal_safe("\n"), super::terminal_safe("\\n"));
    assert_ne!(
        super::terminal_safe("\u{1b}"),
        super::terminal_safe("\u{fffd}")
    );
}

// Requirements: Section 14
//   No normal or build dependency exists, so no hash or plan implementation can arrive from outside the crate; std's own hashers are held off the output type by a Tier-1 compile-time ambiguity proof, and past that the boundary is a named review obligation
// Evidence: the_shipped_dependency_closure_is_empty
#[test]
fn the_shipped_dependency_closure_is_empty() {
    // The compile-time-selected Cargo, launched with a structured argument
    // array, is the dependency-closure oracle. Git is the only other executable
    // class launched by chassis tests, for the bounded-launcher success/failure
    // proofs; neither comes from PATH.
    let output = std::process::Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .output()
        .expect("cargo metadata is how the workspace answers structural questions");
    assert!(
        output.status.success(),
        "cargo metadata refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON");
    let package = metadata["packages"]
        .as_array()
        .expect("metadata carries a package list")
        .iter()
        .find(|package| package["name"] == "partman-cli")
        .expect("the chassis package must be in its own workspace");

    let shipped: Vec<String> = package["dependencies"]
        .as_array()
        .expect("every package carries a dependency list")
        .iter()
        // `kind` is null for a normal dependency, "build" for build scripts,
        // "dev" for test-only. Dev-dependencies cannot reach the shipped
        // binary, which is exactly the boundary this guard draws. With the
        // direct non-dev set empty the transitive closure is empty by
        // entailment; if a dependency is ever allowlisted, this test must
        // resolve the real closure or be renamed, because its name would
        // otherwise overclaim. The assertion is a snapshot of the manifest,
        // not of what tests link.
        .filter(|dependency| dependency["kind"] != "dev")
        .map(|dependency| dependency["name"].to_string())
        .collect();
    assert!(
        shipped.is_empty(),
        "the shipped closure gained {shipped:?}; widening it is a reviewed decision — the \
         guard exists so a hash or plan implementation cannot arrive as a transitive convenience"
    );
}

// Requirements: Section 14
//   Outcome does not implement Hash: this regular unit-test target carries two otherwise-disjoint trait candidates, so adding Hash makes the marker selection ambiguous and fails Tier-1 compilation
// Evidence: the_output_type_does_not_implement_hash
#[test]
fn the_output_type_does_not_implement_hash() {
    trait AmbiguousIfHash<Marker> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfHash<()> for T {}
    impl<T: ?Sized + std::hash::Hash> AmbiguousIfHash<u8> for T {}

    let marker = <Outcome as AmbiguousIfHash<_>>::marker;
    marker();
}

// Requirements: MODEL-003
//   The version surface reports the workspace package version through the same envelope discipline as every other surface
// Evidence: version_reports_through_the_envelope
#[test]
fn version_reports_through_the_envelope() {
    let human = fdispatch(&["version".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    assert_eq!(human.stdout, format!("partman {VERSION}\n"));

    let json = fdispatch(&["version".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    assert_eq!(parsed["outcome"]["kind"], "ok");
    assert_eq!(parsed["outcome"]["version"].as_str(), Some(VERSION));
}

// Requirements: SAFE-006, INV-007, CLI-002
//   The bundle's JSON key set equals the pinned allowlist exactly, with the expected keys as literals so widening the allowlist is a visible reviewed edit; that deny-by-default is the builder's type rather than a filter is a property of lib.rs recorded as structured evidence there, not something a key-set assertion can distinguish
// Evidence: export_diagnostics_admits_exactly_the_allowlisted_fields
#[test]
fn export_diagnostics_admits_exactly_the_allowlisted_fields() {
    let json = fdispatch(&["export-diagnostics".to_owned(), "--json".to_owned()]);
    assert_eq!(json.code, EXIT_OK);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("the bundle rides the ordinary envelope");
    assert_eq!(parsed["outcome"]["kind"], "ok");

    let bundle = parsed["outcome"]["diagnostics"]
        .as_object()
        .expect("the bundle is one JSON object");
    // Pinned as literals, exactly like the exit codes: the allowlist living
    // only in the code under test would let the code widen itself and the
    // test agree. Extending this list is the visible reviewed edit.
    let expected = [
        "tool-version",
        "envelope-schema",
        "build-target",
        "commands",
        "exit-codes",
        "discovery-evidence",
    ];
    let mut actual: Vec<&str> = bundle.keys().map(String::as_str).collect();
    actual.sort_unstable();
    let mut pinned = expected;
    pinned.sort_unstable();
    assert_eq!(
        actual, pinned,
        "the bundle's key set must equal the pinned allowlist exactly — nothing smuggled, \
         nothing dropped"
    );

    // The human rendering cannot drop an allowlisted field; additions to
    // the human rendering are caught by the byte-for-byte pin in
    // `the_human_bundle_is_pinned_byte_for_byte`, not by this containment
    // check.
    let human = fdispatch(&["export-diagnostics".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    for key in expected {
        assert!(
            human.stdout.contains(key),
            "human diagnostics must carry `{key}` too"
        );
    }
}

// Requirements: SAFE-006
//   No output in any mode carries the host's username, home path, or computer name, nor any other environment value the host actually sets that is six bytes or longer and not byte-equal to a rendered compile-time constant — a tripwire whose reach stops at variables the test host sets; the source guard is what refuses an environment read regardless of host state
// Evidence: no_output_in_any_mode_carries_an_environment_value
#[test]
fn no_output_in_any_mode_carries_an_environment_value() {
    // Values byte-equal to a compile-time constant the bundle renders by
    // definition are exempt from both lists. WSL's login shell exports
    // HOSTTYPE=x86_64, which equals `std::env::consts::ARCH` — the bundle
    // printing its own build target is not an environment read. What
    // establishes that output is environment-independent is the source
    // guard (`the_shipped_sources_read_no_environment_variable`) plus the
    // empty shipped closure — not this sweep, and not the byte-determinism
    // test, which only proves stability within one environment. The
    // exemption lists the constants themselves rather than loosening the
    // sweep.
    let rendered_constants = [
        std::env::consts::OS,
        std::env::consts::ARCH,
        VERSION,
        ENVELOPE_SCHEMA,
        // GitHub's Windows runners export RUNNER_OS=Windows, and since
        // increment 9 the help text names the deferred platform in prose —
        // static compile-time text, verified by inspection, with the
        // env-read source guard proving no environment read exists. The
        // exact value is exempted per this test's own remedy rather than
        // contorting user-facing text to dodge the tripwire. ("Linux" and
        // "macOS" never reach the sweep: both are under the six-character
        // floor.)
        "Windows",
    ];

    // SEC-006's deny-floor categories, probed with this host's real values.
    // Read in the test, never in the binary: `std::env::var` here is what
    // makes the absence assertion about a genuine secret-shaped value.
    let mut sensitive: Vec<(String, String)> = Vec::new();
    for name in [
        "USERNAME",
        "USER",
        "USERPROFILE",
        "HOME",
        "COMPUTERNAME",
        "HOSTNAME",
        "LOGNAME",
    ] {
        if let Ok(value) = std::env::var(name)
            && value.len() >= 3
            && !rendered_constants.contains(&value.as_str())
        {
            sensitive.push((name.to_owned(), value));
        }
    }
    assert!(
        !sensitive.is_empty(),
        "no identity-bearing environment variable exists on this host, so this tripwire \
         would prove nothing here; extend the name list rather than letting it pass vacuously"
    );
    // Every other environment value long enough to be identifying. Short
    // values ("true", "1", locale fragments) are skipped because they can
    // collide with legitimate static output by coincidence, and a tripwire
    // that cries wolf gets deleted. Non-Unicode values are skipped because
    // every output is a valid-UTF-8 String, so such a value can never
    // appear in one byte-for-byte — and unwrapping it would be the
    // undocumented-panic seam `dispatch_os` exists to own.
    for (name, value) in std::env::vars_os() {
        let Ok(name) = name.into_string() else {
            continue;
        };
        let Ok(value) = value.into_string() else {
            continue;
        };
        if value.len() >= 6 && !rendered_constants.contains(&value.as_str()) {
            sensitive.push((name, value));
        }
    }

    for arguments in every_invocation() {
        let outcome = fdispatch(&arguments);
        for (stream, text) in [("stdout", &outcome.stdout), ("stderr", &outcome.stderr)] {
            for (name, value) in &sensitive {
                assert!(
                    !text.contains(value.as_str()),
                    "{stream} of {arguments:?} contains the value of ${name} — either an \
                     environment value reached the output, or the static output happens to \
                     contain this value by coincidence; verify which by inspection, and if \
                     it is coincidence, add the exact constant to the named exemption \
                     rather than loosening the sweep"
                );
            }
        }
    }
}

// Requirements: SEC-007, SAFE-006
//   The shipped sources contain no environment read — env::var, env::vars, and var_os are absent from every shipped source file the test enumerates: lib.rs, main.rs, doctor.rs, facts.rs, inspect.rs, reach.rs, devices.rs, macos.rs, and plist.rs — so an environment value cannot reach output regardless of which variables the host sets; compile-time env::consts and env! are the allowed forms
// Evidence: the_shipped_sources_read_no_environment_variable
#[test]
fn the_shipped_sources_read_no_environment_variable() {
    // A text scan, and this repository has watched text scanners be
    // defeated — so its exact reach is stated rather than implied: it
    // catches the direct spellings below; a glob import (`use std::env::*`)
    // would evade it and is refused by clippy's `wildcard_imports` lint
    // (pedantic is a warning workspace-wide and CI runs clippy with
    // `-D warnings`); and no other crate can smuggle a read in, because the
    // shipped dependency closure is empty. Residue past those three is
    // review. The environment sweep above complements this from the other
    // side: it sees actual values, but only for variables the test host
    // sets.
    for (file, source) in [
        ("lib.rs", include_str!("lib.rs")),
        ("main.rs", include_str!("main.rs")),
        ("doctor.rs", include_str!("doctor.rs")),
        ("facts.rs", include_str!("facts.rs")),
        ("inspect.rs", include_str!("inspect.rs")),
        ("reach.rs", include_str!("reach.rs")),
        ("devices.rs", include_str!("devices.rs")),
        ("macos.rs", include_str!("macos.rs")),
        ("plist.rs", include_str!("plist.rs")),
    ] {
        for needle in ["env::var", "env::vars", "var_os"] {
            assert!(
                !source.contains(needle),
                "{file} contains `{needle}`: the shipped binary gained an environment \
                 read. The redaction tripwire only sees variables the test host sets, so \
                 do not rely on it — route the value through the diagnostics allowlist \
                 and its review instead"
            );
        }
    }
}

// Requirements: Section 12, CLI-002
//   The bundle's command-surface states agree with dispatch behavior — a command is reported answering iff it exits 0 and refusing iff it exits with the refusal code — so the diagnostics cannot claim an unimplemented surface answers
// Evidence: the_bundle_command_states_agree_with_dispatch_behavior
#[test]
fn the_bundle_command_states_agree_with_dispatch_behavior() {
    let json = fdispatch(&["export-diagnostics".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    let commands = parsed["outcome"]["diagnostics"]["commands"]
        .as_array()
        .expect("the command surface is an array");
    assert_eq!(
        commands.len(),
        ALL_COMMANDS.len(),
        "the bundle must list every command exactly once"
    );
    for command in ALL_COMMANDS {
        let entry = commands
            .iter()
            .find(|entry| entry["name"] == command.name())
            .unwrap_or_else(|| panic!("the bundle must list `{}`", command.name()));
        let state = entry["state"].as_str().expect("state is a string");
        let behavior = fdispatch(&[command.name().to_owned()]);
        match behavior.code {
            code if code == EXIT_OK => assert_eq!(
                state,
                "answers",
                "`{}` answers but the bundle reports {state:?}",
                command.name()
            ),
            code if code == EXIT_REFUSAL => assert!(
                state.starts_with("refuses:"),
                "`{}` refuses but the bundle reports {state:?} — a diagnostics bundle \
                 claiming an unimplemented surface answers is a plausible fake success",
                command.name()
            ),
            other => panic!(
                "`{}` exited {other}, which no state word covers",
                command.name()
            ),
        }
    }
}

// Requirements: SAFE-006
//   The human diagnostics rendering is pinned byte-for-byte against a literal template, so an extra human-only disclosure fails the tier exactly like a smuggled JSON key
// Evidence: the_human_bundle_is_pinned_byte_for_byte
#[test]
fn the_human_bundle_is_pinned_byte_for_byte() {
    // The template is duplicated here as a literal deliberately — pinning
    // against the same constants that render the output would be the
    // self-referential mistake the exit-code pins exist to correct. The
    // three interpolations are compile-time facts that differ per platform
    // and release, nothing else.
    let expected = format!(
        "diagnostics (redacted by allowlist; 6 fields, all compile-time data)\n\
         \x20 tool-version: {version}\n\
         \x20 envelope-schema: partman.cli.envelope/0\n\
         \x20 build-target: {os} {arch}\n\
         \x20 commands:\n\
         \x20   help: answers\n\
         \x20   version: answers\n\
         \x20   inspect: answers\n\
         \x20   export-diagnostics: answers\n\
         \x20   doctor: answers\n\
         \x20   facts: answers\n\
         \x20   inventory: refuses:not-established\n\
         \x20   topology: refuses:not-established\n\
         \x20   capabilities: refuses:not-implemented\n\
         \x20 exit-codes: 0 answered, 2 usage refusal, 3 typed refusal\n\
         \x20 discovery-evidence: not-implemented (WP-W100, WP-L100, WP-M100)\n\
         \x20   the diagnostics bundle admits compile-time data only, so it carries no \
         discovery evidence; observation records exist as per-run inspect output, and \
         evidence from real devices reaches this bundle only when a platform adapter \
         package lands it here through the same field allowlist\n",
        version = VERSION,
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
    );
    let human = fdispatch(&["export-diagnostics".to_owned()]);
    assert_eq!(
        human.stdout, expected,
        "the human bundle must match the pinned template byte-for-byte; a difference is \
         either a new disclosure or a dropped one, and both are reviewed edits"
    );
}

// Requirements: SEC-007
//   The bundle is byte-identical across invocations within one environment — stability, not environment-independence, which the source guard and the empty shipped closure establish
// Evidence: export_diagnostics_is_byte_identical_across_invocations
#[test]
fn export_diagnostics_is_byte_identical_across_invocations() {
    for arguments in [
        vec!["export-diagnostics".to_owned()],
        vec!["export-diagnostics".to_owned(), "--json".to_owned()],
    ] {
        let first = fdispatch(&arguments);
        let second = fdispatch(&arguments);
        assert_eq!(
            first.stdout, second.stdout,
            "two runs of {arguments:?} must produce identical bytes; a difference means \
             something non-constant entered the bundle"
        );
        assert_eq!(first.code, second.code);
    }
}

// Requirements: INV-007, Section 12
//   The missing discovery evidence is an in-band typed refusal naming its increment — never an omission a reader could mistake for a clean bill of health
// Evidence: missing_discovery_evidence_is_a_typed_refusal_not_an_omission
#[test]
fn missing_discovery_evidence_is_a_typed_refusal_not_an_omission() {
    let json = fdispatch(&["export-diagnostics".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    let evidence = &parsed["outcome"]["diagnostics"]["discovery-evidence"];
    assert_eq!(evidence["state"], "not-implemented");
    assert_eq!(evidence["reference"], "WP-W100, WP-L100, WP-M100");
    assert!(
        evidence["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("allowlist")),
        "the refusal must say how future evidence enters the bundle: through the allowlist"
    );

    let human = fdispatch(&["export-diagnostics".to_owned()]);
    assert!(
        human.stdout.contains("discovery-evidence: not-implemented"),
        "the human bundle must carry the refusal in-band too: {}",
        human.stdout
    );
}

// Requirements: SAFE-004, Section 16
//   The doctor probes only the roster's compiled absolute candidate paths, in order, and nothing else — there is no PATH search to influence, and an empty roster is a typed statement rather than a silent pass
// Evidence: the_doctor_probes_only_compiled_absolute_paths
#[test]
fn the_doctor_probes_only_compiled_absolute_paths() {
    use std::cell::RefCell;

    struct Recording {
        seen: RefCell<Vec<String>>,
    }
    impl ToolLauncher for Recording {
        fn exists(&self, path: &Path) -> bool {
            self.seen.borrow_mut().push(path.display().to_string());
            false
        }
        fn probe_version(&self, path: &Path) -> ProbeOutcome {
            panic!("probe of {} without an existence hit", path.display());
        }
        fn launch(&self, path: &Path, _arguments: &[&str], _output_limit: usize) -> ProbeOutcome {
            panic!(
                "the doctor must not use the argument channel: {}",
                path.display()
            );
        }
    }

    let recording = Recording {
        seen: RefCell::new(Vec::new()),
    };
    let reports = examine(super::doctor::ROSTER, &recording);
    let seen = recording.seen.into_inner();

    if super::doctor::ROSTER.is_empty() {
        assert!(
            seen.is_empty() && reports.is_empty(),
            "an empty roster probes nothing"
        );
        let statement = super::doctor::empty_roster_statement()
            .expect("an empty roster must carry its typed statement");
        assert_eq!(statement.state, "not-implemented");
        assert!(
            statement.reference.starts_with("WP-"),
            "the statement names the package that registers this platform's checks"
        );
    } else {
        assert!(
            super::doctor::empty_roster_statement().is_none(),
            "a populated roster carries no empty-roster statement"
        );
        let compiled: Vec<&str> = super::doctor::ROSTER
            .iter()
            .flat_map(|tool| tool.candidates.iter().copied())
            .collect();
        assert_eq!(
            seen.len(),
            compiled.len(),
            "every candidate is probed exactly once when nothing exists"
        );
        for path in &seen {
            assert!(
                Path::new(path).is_absolute(),
                "candidate `{path}` is not absolute; SAFE-004 forbids anything PATH could bend"
            );
            assert!(
                compiled.contains(&path.as_str()),
                "`{path}` was probed but is not a compiled roster candidate"
            );
        }
        for report in &reports {
            match &report.resolution {
                Resolution::Absent { checked } => assert!(
                    !checked.is_empty(),
                    "an absent tool must say where PartMan looked"
                ),
                Resolution::Found { .. } => panic!("nothing exists in this launcher"),
            }
        }
    }
}

// Requirements: INV-006
//   The current shipped source graph has no auto-mount or repair execution route: every production source outside doctor is pinned free of direct process-command construction, the module declaration set is closed, the production doctor launcher invokes every compiled roster path with fixed informational --version, and the launcher's argument-bearing channel has exactly one shipped caller — the macOS adapter's two fixed diskutil invocations; alternate spellings remain a named review obligation, bounded by denied unsafe code and the empty shipped dependency closure
// Evidence: discovery_cannot_auto_mount_or_run_repair_tools
#[test]
fn discovery_cannot_auto_mount_or_run_repair_tools() {
    let library = include_str!("lib.rs");
    let doctor = include_str!("doctor.rs");

    for (file, source) in [
        ("main.rs", include_str!("main.rs")),
        ("lib.rs", library),
        ("facts.rs", include_str!("facts.rs")),
        ("inspect.rs", include_str!("inspect.rs")),
        ("reach.rs", include_str!("reach.rs")),
        ("devices.rs", include_str!("devices.rs")),
        ("macos.rs", include_str!("macos.rs")),
        ("plist.rs", include_str!("plist.rs")),
    ] {
        assert!(
            !source.contains("std::process::Command") && !source.contains("process::{Command"),
            "{file} gained direct process access; only the fixed doctor launcher may own it"
        );
    }

    let modules: Vec<&str> = library
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("mod ") || line.starts_with("pub mod "))
        .collect();
    assert_eq!(
        modules,
        [
            "pub mod devices;",
            "pub mod doctor;",
            "pub mod facts;",
            "pub mod inspect;",
            "pub mod macos;",
            "pub mod plist;",
            "pub mod reach;",
            "mod tests;",
        ],
        "a new shipped module must enter this source guard before it can compile cleanly"
    );

    let production_doctor = doctor
        .split("#[cfg(test)]")
        .next()
        .expect("the production doctor source precedes its test-only module");
    assert_eq!(
        production_doctor
            .matches("std::process::Command::new")
            .count(),
        1,
        "the production doctor owns exactly one direct process constructor"
    );

    let start = doctor
        .find("impl ToolLauncher for SystemLauncher")
        .expect("the production launcher implementation is present");
    let end = doctor[start..]
        .find("/// Launch one absolute executable")
        .map(|offset| start + offset)
        .expect("the launcher implementation has its reviewed boundary");
    let implementation = &doctor[start..end];
    assert_eq!(
        implementation.matches("launch_bounded(").count(),
        2,
        "the production launcher has exactly two invocation routes: the fixed version \
         probe, and the argument-bearing channel the enumeration seam declares"
    );
    assert!(
        implementation.contains("launch_bounded(path, &[\"--version\"], OUTPUT_LIMIT_PER_STREAM)"),
        "the version probe stays fixed: literal --version under the doctor's own bound"
    );
    // The argument-bearing channel exists for the enumeration adapters, and
    // exactly one shipped module calls it. A second caller is a reviewed
    // event, not a drive-by; this pin is what makes it one. The pin's reach
    // is the direct spelling, as with the environment guard: smuggling
    // routes are closed by the empty shipped closure and the
    // wildcard-import lint, and the residue is review's.
    for (file, source) in [
        ("lib.rs", library),
        ("main.rs", include_str!("main.rs")),
        ("facts.rs", include_str!("facts.rs")),
        ("inspect.rs", include_str!("inspect.rs")),
        ("reach.rs", include_str!("reach.rs")),
        ("devices.rs", include_str!("devices.rs")),
        ("plist.rs", include_str!("plist.rs")),
    ] {
        assert!(
            !source.contains(".launch("),
            "{file} calls the launcher's argument channel; macos.rs is its only \
             shipped caller"
        );
    }
    assert_eq!(
        include_str!("macos.rs").matches(".launch(").count(),
        2,
        "the macOS adapter launches exactly its two fixed diskutil invocations"
    );
}

/// A launcher whose probe follows one script; only `/probe/tool` exists.
struct Scripted {
    outcome: ProbeScript,
}

/// One scripted probe outcome, named so the case table stays readable.
type ProbeScript = fn() -> ProbeOutcome;

impl ToolLauncher for Scripted {
    fn exists(&self, path: &Path) -> bool {
        path == Path::new("/probe/tool")
    }
    fn probe_version(&self, _path: &Path) -> ProbeOutcome {
        (self.outcome)()
    }
    fn launch(&self, path: &Path, _arguments: &[&str], _output_limit: usize) -> ProbeOutcome {
        panic!(
            "this launcher scripts version probes only: {}",
            path.display()
        );
    }
}

/// The tool spec the scripted-launcher tests share.
fn scripted_spec() -> ToolSpec {
    ToolSpec {
        name: "tool",
        role: "test subject",
        candidates: &["/probe/tool", "/probe/fallback"],
        tested: TestedVersion {
            label: "util-linux 2.41",
            family: (2, 41),
        },
    }
}

// Requirements: CAP-004
//   The doctor reports presence, version, and tested-range membership as facts with provenance — the answering path and the sanitized banner — and a probe failure is a typed state, never a guessed version
// Evidence: the_doctor_reports_presence_version_and_range_as_facts
#[test]
fn the_doctor_reports_presence_version_and_range_as_facts() {
    let cases: [(ProbeScript, &str); 6] = [
        (
            || ProbeOutcome::Completed {
                stdout: b"tool from util-linux 2.41 (libblkid 2.41.0)".to_vec(),
                stderr: Vec::new(),
            },
            "within-tested-range",
        ),
        (
            || ProbeOutcome::Completed {
                stdout: b"tool from util-linux 2.39.3".to_vec(),
                stderr: Vec::new(),
            },
            "outside-tested-range",
        ),
        (
            || ProbeOutcome::Completed {
                stdout: b"no digits here at all".to_vec(),
                stderr: Vec::new(),
            },
            "unknown",
        ),
        (
            || ProbeOutcome::Completed {
                stdout: Vec::new(),
                stderr: b"banner on stderr: tool 2.41".to_vec(),
            },
            "within-tested-range",
        ),
        (
            || ProbeOutcome::NonzeroExit {
                code: Some(7),
                stdout: Vec::new(),
                stderr: b"\x1b[31mtool refused the probe".to_vec(),
            },
            "nonzero-exit",
        ),
        (|| ProbeOutcome::TimedOut, "timed-out"),
    ];

    for (outcome, expectation) in cases {
        let roster = [scripted_spec()];
        let reports = examine(&roster, &Scripted { outcome });
        let report = reports.first().expect("one tool, one report");
        let Resolution::Found { path, probe } = &report.resolution else {
            panic!("the scripted launcher finds /probe/tool");
        };
        assert_eq!(path, "/probe/tool", "provenance names the answering path");
        match probe {
            ProbeReport::Answered { raw, range, .. } => {
                assert_eq!(*range, expectation, "banner {raw:?} classified wrongly");
                assert!(
                    !raw.is_empty(),
                    "the sanitized banner travels as provenance"
                );
            }
            ProbeReport::Failed { state, detail } => {
                assert_eq!(*state, expectation);
                assert!(!detail.is_empty(), "a typed failure carries its sentence");
                if expectation == "nonzero-exit" {
                    assert!(detail.contains("exit code 7"), "{detail}");
                    assert!(detail.contains("tool refused the probe"), "{detail}");
                    assert!(
                        !detail.contains('\u{1b}'),
                        "nonzero provenance must be terminal-safe: {detail}"
                    );
                }
            }
        }
        // Both renderings stay free of CAP-003 status vocabulary as bare
        // words — the human form has no quotes for a quoted check to catch —
        // because the range is a fact about a version, never a capability
        // verdict.
        let rendered = format!(
            "{}\n{}",
            doctor_json(&reports, None),
            doctor_human(&reports, None)
        );
        for verdict in ["supported", "preview", "blocked"] {
            assert!(
                !rendered.to_lowercase().contains(verdict),
                "doctor output contains the CAP-003 status word `{verdict}`; that \
                 vocabulary is WP-050's, and SAFE-004's out-of-range-means-blocked \
                 mapping happens there"
            );
        }
        let parsed: serde_json::Value = serde_json::from_str(&doctor_json(&reports, None))
            .expect("the doctor's JSON object must parse");
        assert!(parsed["tools"].as_array().is_some_and(|t| t.len() == 1));
    }
}

// Requirements: CAP-004, Section 12
//   An absent tool's report carries every candidate path checked in both renderings and can never render as found — a missing dependency drawn as a pass would be a plausible fake success, and a mutation doing exactly that survived the suite until these pins existed
// Evidence: an_absent_tool_reports_where_partman_looked
#[test]
fn an_absent_tool_reports_where_partman_looked() {
    let roster = [ToolSpec {
        candidates: &["/absent/one", "/absent/two"],
        ..scripted_spec()
    }];
    let reports = examine(&roster, &NothingInstalled);
    let Resolution::Absent { checked } = &reports[0].resolution else {
        panic!("nothing exists in this launcher");
    };
    assert_eq!(
        checked,
        &["/absent/one".to_owned(), "/absent/two".to_owned()]
    );

    let human = doctor_human(&reports, None);
    assert!(
        human.contains("absent; checked /absent/one, /absent/two"),
        "the human report must say where PartMan looked: {human}"
    );
    for fake in ["found at", "version:"] {
        assert!(
            !human.contains(fake),
            "an absent tool must never render `{fake}`: {human}"
        );
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&doctor_json(&reports, None)).expect("doctor JSON parses");
    let resolution = &parsed["tools"][0]["resolution"];
    assert_eq!(resolution["state"], "absent");
    assert_eq!(
        resolution["candidates-checked"],
        serde_json::json!(["/absent/one", "/absent/two"])
    );
    assert!(
        resolution.get("probe").is_none(),
        "an absent tool has no probe to report"
    );
}

// Requirements: SAFE-004
//   The launcher's child environment is cleared and gains exactly one written variable, LC_ALL=C — pinned as a source-text assertion because no behavioral proof fits WP-035's test process set, with the pin's reach stated: direct spellings, with smuggling routes closed by the empty closure and the wildcard-import lint, and the residue held by review
// Evidence: the_launcher_clears_the_child_environment
#[test]
fn the_launcher_clears_the_child_environment() {
    // A text pin, like the env-read source guard beside it, and for the
    // same reason: a behavioral proof would need a canary process outside
    // the tier's git-and-cargo set. A mutation replacing env_clear with an
    // inherited environment survived every behavioral test when tried; this
    // pin is what kills it, and its reach is exactly the direct spellings.
    let source = include_str!("doctor.rs");
    assert!(
        source.contains(".env_clear()"),
        "the launcher must clear the child environment before writing into it"
    );
    let writes = source.matches(".env(").count();
    assert_eq!(
        writes, 1,
        "exactly one environment write is pinned; a second is a reviewed edit"
    );
    assert!(
        source.contains(".env(\"LC_ALL\", \"C\")"),
        "the one write is LC_ALL=C, so tool output is not localized"
    );
}

// Requirements: Section 12
//   An empty roster renders as a typed not-implemented statement naming the platform's adapter package — never as a clean bill that would read as all dependencies satisfied
// Evidence: an_empty_roster_renders_as_a_typed_statement
#[test]
fn an_empty_roster_renders_as_a_typed_statement() {
    let statement = Refusal {
        state: "not-implemented",
        reference: "WP-W100",
        detail: "this platform's inventory route is a native API",
    };
    let json = doctor_json(&[], Some(&statement));
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("doctor JSON parses");
    assert_eq!(parsed["roster"]["state"], "not-implemented");
    assert_eq!(parsed["roster"]["reference"], "WP-W100");
    assert_eq!(
        parsed["tools"].as_array().map(Vec::len),
        Some(0),
        "no tools were checked and the report says so"
    );
    let human = doctor_human(&[], Some(&statement));
    assert!(
        human.contains("not-implemented (WP-W100)"),
        "the human form carries the same typed statement: {human}"
    );
}

// Requirements: Section 16, CAP-004
//   A version banner parses to major.minor or stays unrecognized with its raw line preserved — never guessed — and the recorded util-linux banner shape parses to the recorded family
// Evidence: version_banners_parse_or_stay_unrecognized_never_guessed
#[test]
fn version_banners_parse_or_stay_unrecognized_never_guessed() {
    let parsed = |banner: &str| parse_version(banner).map(|v| (v.major, v.minor));
    // The banner shape crates/fixtures' prober records for util-linux 2.41.
    assert_eq!(
        parsed("blkid from util-linux 2.41  (libblkid 2.41.0, 18-Mar-2025)"),
        Some((2, 41))
    );
    assert_eq!(parsed("wipefs from util-linux 2.39.3"), Some((2, 39)));
    assert_eq!(parsed("cargo 1.96.0 (abc123 2026-01-01)"), Some((1, 96)));
    assert_eq!(
        parsed("version v2.41"),
        None,
        "a v-prefixed token is not shaped digits.digits; the parser strips no prefix it \
         was never shown a banner for, and unrecognized-with-raw-preserved beats a guess"
    );
    assert_eq!(
        parsed("release 7 build 2.41"),
        Some((2, 41)),
        "the first digits.digits token wins even when other numbers precede it"
    );
    assert_eq!(parsed("no version anywhere"), None);
    assert_eq!(parsed(""), None);
    assert_eq!(
        parsed("Fassung zwei Punkt einundvierzig"),
        None,
        "a localized banner without digits stays unrecognized rather than guessed"
    );
}

// Requirements: SAFE-004
//   The real launcher captures a successful exit with bounded output before the deadline by probing Git at one reviewed absolute path; the adjacent doctor-module test supplies Git's intentional nonzero arm, and neither test launches a roster tool or shell
// Evidence: the_real_launcher_answers_bounded_with_provenance
#[test]
fn the_real_launcher_answers_bounded_with_provenance() {
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
    let git = TEST_GIT
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .expect("Tier 1 requires Git at one reviewed absolute path");

    let launcher = SystemLauncher;
    assert!(launcher.exists(git), "the resolved Git executable exists");
    match launcher.probe_version(git) {
        ProbeOutcome::Completed { stdout, .. } => {
            assert!(!stdout.is_empty(), "git --version banners on stdout");
            assert!(stdout.len() <= 4096, "output stayed within the bound");
            let banner = super::doctor::sanitized_first_line(&stdout);
            assert!(
                banner.to_lowercase().contains("git"),
                "provenance keeps the raw line: {banner}"
            );
            assert!(
                parse_version(&banner).is_some(),
                "a real Git banner parses: {banner}"
            );
        }
        other => panic!(
            "git --version must complete within the limits; got {}",
            match other {
                ProbeOutcome::TimedOut => "timed-out",
                ProbeOutcome::OverOutputLimit => "over-output-limit",
                ProbeOutcome::LaunchFailed(_) => "launch-failed",
                ProbeOutcome::NonzeroExit { .. } => "nonzero-exit",
                ProbeOutcome::Completed { .. } => unreachable!(),
            }
        ),
    }
}

// Requirements: FS-007
//   Every shipped fact names a technology, an operation, a limit, and a checkable basis, and neither rendering contains CAP-003 status vocabulary — the facts are inputs to blocked reasons, and the blocked-reason surface is WP-050's
// Evidence: facts_are_technology_properties_without_status_vocabulary
#[test]
fn facts_are_technology_properties_without_status_vocabulary() {
    // Pinned as literals like every other contract: adding a fact is a
    // visible reviewed edit, and the first entry is FS-007's own example.
    let pinned: Vec<(&str, &str)> = vec![
        ("xfs", "shrink"),
        ("ext4", "shrink while mounted"),
        ("linux-swap", "resize in place"),
        ("fat32", "hold a file of 4 GiB or larger"),
        (
            "fat32",
            "address a volume beyond 2 TiB with 512-byte sectors",
        ),
    ];
    let actual: Vec<(&str, &str)> = FACTS
        .iter()
        .map(|fact| (fact.technology, fact.operation))
        .collect();
    assert_eq!(actual, pinned, "the fact roster is a pinned contract");

    for fact in FACTS {
        for (field, value) in [
            ("technology", fact.technology),
            ("operation", fact.operation),
            ("limit", fact.limit),
            ("basis", fact.basis),
        ] {
            assert!(
                !value.is_empty(),
                "a fact with an empty {field} is not a fact"
            );
        }
    }

    let json = facts_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("facts JSON parses");
    assert_eq!(parsed.as_array().map(Vec::len), Some(FACTS.len()));

    let rendered = format!("{json}\n{}", facts_human());
    for verdict in ["supported", "preview", "unsupported", "blocked"] {
        assert!(
            !rendered.to_lowercase().contains(verdict),
            "facts output contains the CAP-003 status word `{verdict}`; a fact is a \
             property of a technology, never a verdict about a target"
        );
    }
}

/// A test-owned temporary file holding exactly these bytes, cleaned up on
/// drop. Test-only I/O, inside the tier's stated boundary: temporary
/// files the tests create and remove themselves.
struct TempObject {
    path: std::path::PathBuf,
}

impl TempObject {
    fn holding(name: &str, bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "partman-inc4-{name}-{pid}-{stamp:?}",
            pid = std::process::id(),
            stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the clock is past 1970")
        ));
        std::fs::write(&path, bytes).expect("the test owns its temporary directory");
        Self { path }
    }
}

impl Drop for TempObject {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn replay_invocation(path: &std::path::Path, json: bool) -> Outcome {
    let mut arguments = vec![
        "inspect".to_owned(),
        "--replay".to_owned(),
        path.display().to_string(),
    ];
    if json {
        arguments.push("--json".to_owned());
    }
    fdispatch(&arguments)
}
// Requirements: MODEL-004, INV-007
//   Every replay observation carries its attribution — adapter name, version, method — and replay over one of WP-020's deterministic images is byte-reproducible, so the record is raw discovery evidence a reader can re-derive, not narrative; neither rendering echoes the replayed object's name
// Evidence: replay_over_a_deterministic_fixture_image_is_attributed_and_reproducible
#[test]
fn replay_over_a_deterministic_fixture_image_is_attributed_and_reproducible() {
    // The assignment's words are "fixture-backed replay over WP-020's
    // deterministic images": one catalogue image, synthesized in memory by
    // the fixtures crate itself (a dev-dependency the closure guard
    // excludes from the shipped binary), written to a test-owned file.
    let fixture = partman_fixtures::catalogue::catalogue()
        .into_iter()
        .find(|fixture| fixture.name == "gpt-basic-512.img")
        .expect("the catalogue carries the basic GPT image");
    let image = (fixture.build)();
    let object = TempObject::holding("gpt-basic", image.bytes());

    let first = replay_invocation(&object.path, true);
    let second = replay_invocation(&object.path, true);
    assert_eq!(first.code, EXIT_OK);
    assert_eq!(
        first.stdout, second.stdout,
        "replay over a deterministic image must be byte-reproducible"
    );

    let parsed: serde_json::Value =
        serde_json::from_str(&first.stdout).expect("the answer rides the envelope");
    let inspect_object = &parsed["outcome"]["inspect"];
    assert_eq!(inspect_object["selector"], "replay:0");
    let observations = inspect_object["observations"]
        .as_array()
        .expect("observations are a list");
    assert_eq!(
        observations.len(),
        1 + super::inspect::PROBES.len(),
        "one length observation plus one per probe; every probe lands inside this image"
    );
    for observation in observations {
        assert_eq!(observation["adapter"]["name"], "fixture-replay");
        assert_eq!(observation["adapter"]["version"], VERSION);
        assert!(
            observation["adapter"]["method"]
                .as_str()
                .is_some_and(|method| !method.is_empty()),
            "an observation without a method is not attributable"
        );
        assert_eq!(
            observation["outcome"]["state"], "observed",
            "every probe lands inside this image, so every outcome is observed"
        );
        assert!(
            observation["outcome"].get("value").is_some(),
            "in-range probes carry a value, never an absence"
        );
    }
    // Two deterministic anchors a reader can check against the fixture's
    // own definition: the object's length, and the bytes at 510..512 —
    // reported as hex, interpreted by nobody, and pinned lowercase so a
    // case change in the encoding is a visible edit.
    assert_eq!(
        observations[0]["outcome"]["value"],
        image.bytes().len().to_string(),
        "the length observation reports the object's real length"
    );
    let boot_probe = observations
        .iter()
        .find(|observation| observation["subject"] == "bytes[510..512)")
        .expect("the 510..512 probe is in the compiled list");
    assert_eq!(
        boot_probe["outcome"]["value"], "55aa",
        "the fixture's bytes at 510..512 are deterministic and reported raw"
    );
    for observation in observations.iter().skip(1) {
        let value = observation["outcome"]["value"]
            .as_str()
            .expect("in-range probe values are strings");
        assert!(
            value
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "probe hex is lowercase [0-9a-f] only: {value}"
        );
    }

    // The gated list travels with every answer.
    let gated = inspect_object["gated"].as_array().expect("gated list");
    assert!(!gated.is_empty(), "the gated list is never omitted");

    // No name echo, in either rendering. The check uses the temporary
    // file's unique NAME rather than its full path: on Windows the JSON
    // escaping doubles backslashes, so a full-path substring check is dead
    // there — a mutation that echoed the path passed it — and file names
    // are themselves on SEC-006's deny-floor.
    let human = replay_invocation(&object.path, false);
    let name = object
        .path
        .file_name()
        .expect("the temp object has a name")
        .to_string_lossy()
        .into_owned();
    for (mode, text) in [("json", &first.stdout), ("human", &human.stdout)] {
        assert!(
            !text.contains(&name),
            "the {mode} rendering echoes the replayed object's name; names and paths \
             are on SEC-006's deny-floor, and the caller knows what they named"
        );
    }
}

// Requirements: SAFE-005, INV-007
//   Absence is a value and renders in the observed family with what established it; a probe straddling the object's end never claims absence of bytes that exist; and a read failure renders as failed — never as unavailability and never as absence — with the three states pinned in both output modes
// Evidence: absence_and_unavailability_are_distinct_outcomes
#[test]
fn absence_and_unavailability_are_distinct_outcomes() {
    // A 100-byte object: probes at 510, 512, and 1024 lie wholly beyond
    // the end and must be observed absences; the head probe is bytes.
    let object = TempObject::holding("tiny", &[0xAB; 100]);
    let outcome = replay_invocation(&object.path, true);
    assert_eq!(outcome.code, EXIT_OK);
    let parsed: serde_json::Value = serde_json::from_str(&outcome.stdout).expect("envelope");
    let observations = parsed["outcome"]["inspect"]["observations"]
        .as_array()
        .expect("observations");
    // Diagnosability pin first: a wrong record count fails here, by name,
    // rather than downstream in an absence-count assert.
    assert_eq!(
        observations.len(),
        1 + super::inspect::PROBES.len(),
        "100 bytes: head probe observed, three probes wholly absent, no splits"
    );
    assert_eq!(observations[0]["outcome"]["value"], "100");

    let absent: Vec<&serde_json::Value> = observations
        .iter()
        .filter(|observation| observation["outcome"].get("absence").is_some())
        .collect();
    assert_eq!(absent.len(), 3, "three probes lie wholly beyond 100 bytes");
    for observation in &absent {
        assert_eq!(
            observation["outcome"]["state"], "observed",
            "absence is a value: it renders in the observed family (ADR-C4)"
        );
        let reason = observation["outcome"]["absence"]
            .as_str()
            .expect("absence carries its reason");
        assert!(
            reason.contains("the object ends at byte 100"),
            "the absence must say what established it: {reason}"
        );
        assert!(
            observation["outcome"].get("value").is_none(),
            "absence carries a reason, not a fabricated value"
        );
    }

    // The human rendering pins the same states — a mutation that swapped
    // the human arms passed the suite when only JSON was pinned.
    let human = replay_invocation(&object.path, false);
    assert!(
        human
            .stdout
            .contains("bytes[510..512): observed absence — the object ends at byte 100"),
        "human absence renders in the observed family with its reason: {}",
        human.stdout
    );
    assert!(
        !human.stdout.contains("unavailable"),
        "nothing about a short regular file is unavailable: {}",
        human.stdout
    );

    // A probe straddling the end must never claim absence of bytes that
    // exist: a 520-byte object splits the 512..528 probe into observed
    // bytes [512..520) and observed absence [520..528).
    let straddle = TempObject::holding("straddle", &[0xCD; 520]);
    let outcome = replay_invocation(&straddle.path, true);
    let parsed: serde_json::Value = serde_json::from_str(&outcome.stdout).expect("envelope");
    let observations = parsed["outcome"]["inspect"]["observations"]
        .as_array()
        .expect("observations");
    assert_eq!(
        observations.len(),
        1 + super::inspect::PROBES.len() + 1,
        "the straddling probe yields two records: its existing prefix and its absent tail"
    );
    let prefix = observations
        .iter()
        .find(|observation| observation["subject"] == "bytes[512..520)")
        .expect("the existing prefix is reported under an accurate subject");
    assert_eq!(prefix["outcome"]["state"], "observed");
    assert_eq!(
        prefix["outcome"]["value"].as_str().map(str::len),
        Some(16),
        "eight existing bytes render as sixteen hex characters"
    );
    let tail = observations
        .iter()
        .find(|observation| observation["subject"] == "bytes[520..528)")
        .expect("the absent tail is reported under an accurate subject");
    assert!(
        tail["outcome"]["absence"]
            .as_str()
            .is_some_and(|reason| reason.contains("bytes from 520 do not exist")),
        "the absence claim covers only the bytes that do not exist"
    );
}

// Requirements: SAFE-005
//   Unavailability and failure are distinct rendered states in both modes, per ADR-C4, and neither masquerades as a value or an absence; exercised at the rendering seam with constructed observations, because no portable Tier-1 setup forces a mid-file read error — stated so this claim is not read as behavioral I/O-failure coverage
// Evidence: unavailability_and_failure_are_distinct_rendered_states
#[test]
fn unavailability_and_failure_are_distinct_rendered_states() {
    // Unavailability and failure are distinct rendered states with
    // distinct meanings, per ADR-C4. No portable Tier-1 setup forces a
    // mid-file read error or a platform-refusing-to-expose case, so both
    // arms are exercised at the rendering seam with constructed
    // observations — stated here so this test's claim is not read as
    // behavioral coverage of I/O failure.
    let attribution = || super::inspect::Attribution {
        adapter: "fixture-replay",
        version: VERSION,
        method: "seek-and-read through the verified handle",
    };
    let constructed = vec![
        super::inspect::Observation {
            subject: "bytes[0..16)".to_owned(),
            attribution: attribution(),
            outcome: super::inspect::Outcome::Unavailable {
                reason: "the platform did not expose the answer".to_owned(),
            },
        },
        super::inspect::Observation {
            subject: "bytes[16..32)".to_owned(),
            attribution: attribution(),
            outcome: super::inspect::Outcome::Failed {
                error: "the read failed: simulated I/O error".to_owned(),
            },
        },
    ];
    let rendered = super::inspect::replay_json(&constructed);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("render parses");
    assert_eq!(parsed["observations"][0]["outcome"]["state"], "unavailable");
    assert_eq!(parsed["observations"][1]["outcome"]["state"], "failed");
    for index in [0, 1] {
        assert!(
            parsed["observations"][index]["outcome"]
                .get("value")
                .is_none()
                && parsed["observations"][index]["outcome"]
                    .get("absence")
                    .is_none(),
            "a non-answer must never masquerade as a value or an absence"
        );
    }
    let rendered_human = super::inspect::replay_human(&constructed);
    assert!(
        rendered_human.contains("unavailable — the platform did not expose the answer"),
        "{rendered_human}"
    );
    assert!(
        rendered_human.contains("failed — the read failed"),
        "{rendered_human}"
    );
}

// Requirements: SAFE-005
//   A replayed object that is not a regular file is refused with a typed value before any byte is read — the pre-open look refuses the common case before any open — an unopenable path is a typed refusal rather than a panic, neither refusal echoes the object's name, and the gated list travels with refusal answers too
// Evidence: replay_refuses_non_regular_objects_with_a_typed_value
#[test]
fn replay_refuses_non_regular_objects_with_a_typed_value() {
    // A directory: never a regular file, refused by the pre-open look.
    let directory = std::env::temp_dir();
    let refused = replay_invocation(&directory, false);
    assert_eq!(refused.code, EXIT_REFUSAL);
    assert!(
        refused.stdout.contains("state: refused"),
        "the refusal is a typed value on stdout: {}",
        refused.stdout
    );
    assert!(
        refused
            .stdout
            .contains("identity-strength: not-established (SI-28)"),
        "the gated list travels with refusal answers too: {}",
        refused.stdout
    );

    let json = replay_invocation(&directory, true);
    assert_eq!(json.code, EXIT_REFUSAL);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("refusals ride the envelope");
    assert_eq!(parsed["outcome"]["kind"], "refusal");
    assert_eq!(parsed["outcome"]["state"], "refused");
    assert_eq!(parsed["outcome"]["reference"], "SAFE-005");
    assert!(
        parsed["outcome"]["gated"]
            .as_array()
            .is_some_and(|gated| !gated.is_empty()),
        "the gated list travels with JSON refusals too"
    );

    // A path that does not exist: refused, typed, at the documented code.
    let missing = std::env::temp_dir().join("partman-inc4-does-not-exist");
    let refused_missing = replay_invocation(&missing, true);
    assert_eq!(refused_missing.code, EXIT_REFUSAL);
    let parsed_missing: serde_json::Value =
        serde_json::from_str(&refused_missing.stdout).expect("refusals ride the envelope");
    assert_eq!(parsed_missing["outcome"]["kind"], "refusal");

    // The refusal does not echo the object's name: SEC-006's deny-floor
    // covers refusal text too, and std's io::Error carrying no path is a
    // fact to pin here, not to trust from documentation. The unique name
    // makes the check escape-proof in both renderings.
    assert!(
        !refused_missing
            .stdout
            .contains("partman-inc4-does-not-exist"),
        "the missing-file refusal echoes the object's name: {}",
        refused_missing.stdout
    );
    let refused_missing_human = replay_invocation(&missing, false);
    assert!(
        !refused_missing_human
            .stdout
            .contains("partman-inc4-does-not-exist"),
        "the human missing-file refusal echoes the object's name: {}",
        refused_missing_human.stdout
    );
}

// Requirements: SAFE-005
//   The handle-level gate refuses a non-regular object on Windows too: a directory handle opened with backup semantics — a handle no path-based Tier-1 test can otherwise produce there — is refused by fstat through the handle, so the platform whose device names most concern the boundary has direct evidence of the gate
// Evidence: the_handle_gate_refuses_a_directory_handle_on_windows
#[cfg(windows)]
#[test]
fn the_handle_gate_refuses_a_directory_handle_on_windows() {
    // On Windows, File::open of a directory fails upstream, so the
    // path-based refusal test never reaches the fstat gate — a mutation
    // deleting the is_file() check passed the Windows suite when tried.
    // Backup semantics yields a real directory handle; the gate must
    // refuse it. A directory, not a device: the tier's boundary holds.
    use std::os::windows::fs::OpenOptionsExt as _;
    let mut handle = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x0200_0000) // FILE_FLAG_BACKUP_SEMANTICS
        .open(std::env::temp_dir())
        .expect("a directory handle opens under backup semantics");
    let Err(refusal) = super::inspect::replay_handle(&mut handle) else {
        panic!("a directory handle must be refused by the handle gate");
    };
    assert_eq!(refusal.state, "refused");
    assert!(
        refusal.detail.contains("regular files only"),
        "{}",
        refusal.detail
    );
}

// Requirements: Section 12, SAFE-005
//   Inspect observations report bytes and access facts, never classifications: a named list of interpretation words is mechanically refused in both renderings' observation sections — the list's reach is exactly those words, with the residue held by review — while the gated section legitimately names what it gates
// Evidence: inspect_reports_bytes_never_classifications
#[test]
fn inspect_reports_bytes_never_classifications() {
    // Replay an object whose bytes spell a well-known table magic — the
    // inspector must report the hex and say nothing about what it means.
    let mut bytes = vec![0_u8; 2048];
    bytes[510] = 0x55;
    bytes[511] = 0xAA;
    bytes[512..520].copy_from_slice(b"EFI PART");
    let object = TempObject::holding("magic", &bytes);

    // The ban scopes to the observations: the gated list legitimately
    // names `partition-table-state`, because saying what is NOT said is
    // its purpose. The banned list is finite and named — these words,
    // no more — and hex output can never collide with most of them, so
    // the guard's real work is on subjects, reasons, and header prose;
    // words beyond the list are review's to catch.
    let json = replay_invocation(&object.path, true);
    assert_eq!(json.code, EXIT_OK);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope");
    let observations_text = parsed["outcome"]["inspect"]["observations"]
        .to_string()
        .to_lowercase();

    let human = replay_invocation(&object.path, false);
    assert_eq!(human.code, EXIT_OK);
    assert!(
        human.stdout.contains("  gated"),
        "the human replay answer must carry the gated section — a mutation dropping it \
         passed while this only split on the marker: {}",
        human.stdout
    );
    for fragment in [
        "identity-strength: not-established (SI-28)",
        "partition-table-state: helper-authored (ADR-0014)",
        "same-device-claims: never-inferred (ADR-0011)",
    ] {
        assert!(
            human.stdout.contains(fragment),
            "the human replay answer must carry `{fragment}`"
        );
    }
    let human_observations = human
        .stdout
        .split("  gated")
        .next()
        .expect("split always yields a first part")
        .to_lowercase();

    for (mode, text) in [("json", &observations_text), ("human", &human_observations)] {
        for classification in [
            "gpt",
            "mbr",
            "guid",
            "boot",
            "table",
            "signature",
            "strength",
            "strong",
            "weak",
            "partition",
        ] {
            assert!(
                !text.contains(classification),
                "inspect {mode} observations contain the classification word \
                 `{classification}`; bytes are reported, readers interpret, and the \
                 register's gates stay gated"
            );
        }
        // The raw material is present for the reader who wants it.
        assert!(
            text.contains("4546492050415254"),
            "the bytes at 512..528 travel as hex in {mode} mode"
        );
    }
}

// Requirements: SAFE-005
//   The --replay flag's malformed shapes are structured usage refusals with exact spellings — a missing value, a following flag mistaken for a value, a second occurrence, and attachment to a command that is not inspect — because a token read two contradictory ways in one invocation is a guess, and this parser refuses rather than guesses
// Evidence: replay_flag_misuse_is_refused_structurally
#[test]
fn replay_flag_misuse_is_refused_structurally() {
    let missing_value = fdispatch(&["inspect".to_owned(), "--replay".to_owned()]);
    assert_eq!(missing_value.code, EXIT_USAGE);
    assert!(
        missing_value
            .stderr
            .contains("flag `--replay` needs a value"),
        "{}",
        missing_value.stderr
    );

    // A following flag is not a value. Before this refusal existed,
    // `inspect --replay --json` swallowed the mode flag as a filename and
    // produced a human-mode file refusal — one token, two readings. Now
    // the token is refused as a value, and the whole-list mode scan may
    // still honor it as the mode for rendering the refusal — consistent,
    // because it was never accepted as anything else.
    let swallowed_json = fdispatch(&[
        "inspect".to_owned(),
        "--replay".to_owned(),
        "--json".to_owned(),
    ]);
    assert_eq!(swallowed_json.code, EXIT_USAGE);
    let parsed: serde_json::Value = serde_json::from_str(&swallowed_json.stdout)
        .expect("with --json in the argument list the refusal rides the envelope");
    assert_eq!(parsed["outcome"]["kind"], "usage-refusal");
    assert!(
        parsed["outcome"]["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("flag `--replay` needs a value")),
        "{}",
        swallowed_json.stdout
    );

    let swallowed_replay = fdispatch(&[
        "inspect".to_owned(),
        "--replay".to_owned(),
        "--replay".to_owned(),
        "x".to_owned(),
    ]);
    assert_eq!(
        swallowed_replay.code, EXIT_USAGE,
        "a second --replay is not a value for the first"
    );

    let twice = fdispatch(&[
        "inspect".to_owned(),
        "--replay".to_owned(),
        "a".to_owned(),
        "--replay".to_owned(),
        "b".to_owned(),
    ]);
    assert_eq!(twice.code, EXIT_USAGE);
    assert!(
        twice.stderr.contains("flag `--replay` given twice"),
        "{}",
        twice.stderr
    );

    let wrong_command = fdispatch(&["version".to_owned(), "--replay".to_owned(), "a".to_owned()]);
    assert_eq!(wrong_command.code, EXIT_USAGE);
    assert!(
        wrong_command
            .stderr
            .contains("flag `--replay` belongs to inspect, not to `version`"),
        "{}",
        wrong_command.stderr
    );

    // The value token is consumed structurally: a path spelled like a
    // command word is a value, never re-interpreted as a command.
    let value_like_command = fdispatch(&[
        "inspect".to_owned(),
        "--replay".to_owned(),
        "version".to_owned(),
    ]);
    assert_eq!(
        value_like_command.code, EXIT_REFUSAL,
        "`version` after --replay is a (nonexistent) file to replay, not a second command"
    );
}

// Evidence: the_reach_declaration_is_complete_and_ordered
#[test]
fn the_reach_declaration_is_complete_and_ordered() {
    // INV-003's third bullet says the declaration "MUST NOT be omitted when
    // the answer is `no`", which is the one property here most able to rot
    // silently: a hand-edited platform table drops a row and nothing
    // notices. A doc comment saying "never partial" is not that guard, so
    // this is.
    let declared: Vec<&str> = crate::reach::REACH
        .cells
        .iter()
        .map(|cell| cell.state)
        .collect();
    assert_eq!(
        declared,
        crate::reach::STATES.to_vec(),
        "the reach declaration must carry one cell per INV-003 state, in INV-003's order, \
         on every platform — a missing cell is an omitted `no`, which INV-003 forbids"
    );
}

// Evidence: the_reach_declaration_claims_no_reach_this_increment
#[test]
fn the_reach_declaration_claims_no_reach_this_increment() {
    // WP-035's Section 14 row grants the declaration "for the contract this
    // package itself reads" and closes: it "is not a claim about interfaces
    // that contract does not read". Increment 7 reads nothing. So every
    // answer is negative, and a `yes` cannot appear until the increment that
    // reads the interface establishing it lands with it.
    //
    // This test is the reason the table is built by one `const fn` rather
    // than hand-copied per platform: there is exactly one place a `true`
    // could be typed, and this fails if it is.
    for cell in &crate::reach::REACH.cells {
        assert!(
            !cell.distinguished,
            "{state} is declared distinguished while this package reads no device \
             interface. A positive answer belongs to the increment that reads the \
             interface, cited to the observability row that establishes it",
            state = cell.state,
        );
        assert_eq!(
            cell.basis,
            crate::reach::basis::NOT_MEASURED,
            "{state} claims a measured basis with nothing read",
            state = cell.state,
        );
        assert!(
            cell.citation.is_none(),
            "{state} carries a citation for a contract that reads nothing",
            state = cell.state,
        );
    }
    // The contract statement must describe the contract that exists, not a
    // fixed string. An earlier version pinned "not-implemented" literally,
    // which would have prevented the statement ever catching up to the code
    // once increment 8 gave Linux a real contract.
    let state = crate::reach::REACH.contract.state;
    assert!(
        state == "not-implemented" || state == "implemented-reaches-no-table-state",
        "the contract statement must say either that nothing is read, or that what \n         is read reaches no table state — never claim a reach the cells deny"
    );
    if cfg!(target_os = "linux") {
        assert_eq!(
            state, "implemented-reaches-no-table-state",
            "Linux has a contract since increment 8; describing it as unimplemented \n             would make the declaration underived from the contract, which INV-003 forbids"
        );
    }
    if cfg!(target_os = "macos") {
        assert_eq!(
            state, "implemented-reaches-no-table-state",
            "macOS has a contract since increment 9, and the same derivation rule holds"
        );
    }
}

// Evidence: the_reach_declaration_reads_nothing
#[test]
fn the_reach_declaration_reads_nothing() {
    // "Derived from the contract rather than from any device" (INV-003) is a
    // property of how the declaration is built, not of what it says. The
    // module is `const` data and two renderers; it must not acquire a device
    // read, a filesystem read, or a process launch — and it must not name a
    // device seam even indirectly, because that is how "declared" becomes
    // "derived" one refactor later.
    let source = include_str!("reach.rs");
    for needle in [
        "std::fs",
        "File::",
        "read_to_string",
        "std::process",
        "DeviceSource",
        "enumerate(",
    ] {
        assert!(
            !source.contains(needle),
            "reach.rs contains `{needle}`: the declaration must stay a property of the \
             contract, declared independently of any device"
        );
    }
}

// Requirements: SAFE-005, Section 14
//   A platform whose adapter is deferred by a recorded decision names that decision in its typed not-implemented answer and its reach reference — a decision, not a promise — while a platform whose increment is merely pending still names the increment, and the two shapes are never conflated
// Evidence: a_deferred_platform_names_the_recorded_decision_not_a_promise
#[test]
fn a_deferred_platform_names_the_recorded_decision_not_a_promise() {
    // The M0.5 gate admits exactly two honest shapes for a platform without
    // an adapter: an increment that will read its interfaces, or a recorded
    // decision that defers them. Windows carries the deferral (the WP-035
    // increment 10 route decision); macOS carries a pending increment; Linux
    // carries a contract and no statement at all. Each platform asserts its
    // own shape, so this test means something different — and true — on all
    // three CI legs.
    if cfg!(target_os = "windows") {
        let decision = super::inspect::platform_deferral()
            .expect("Windows carries a recorded deferral decision");
        assert!(
            decision.contains("WP-W100") && decision.contains("2026-08-08"),
            "the deferral must name the owning package and the decision date: {decision}"
        );
        assert_eq!(
            crate::reach::REACH.contract.reference,
            decision,
            "the reach reference and the inspect answer must name the same decision, \
             or a reader is told two different stories about why Windows waits"
        );
        let json = fdispatch(&["inspect".to_owned(), "--json".to_owned()]);
        let parsed: serde_json::Value =
            serde_json::from_str(&json.stdout).expect("the answer rides the ordinary envelope");
        assert_eq!(
            parsed["outcome"]["inspect"]["adapters"]["deferral"], decision,
            "the typed not-implemented answer must carry the recorded decision in-band"
        );
        let human = fdispatch(&["inspect".to_owned()]);
        assert!(
            human.stdout.contains(decision),
            "the human answer must name the recorded decision too: {}",
            human.stdout
        );
    } else {
        assert!(
            super::inspect::platform_deferral().is_none(),
            "only Windows carries a recorded deferral; a pending increment must not \
             be dressed as a decision"
        );
    }
    if cfg!(target_os = "macos") {
        assert_eq!(
            crate::reach::REACH.contract.reference,
            "WP-035 increment 9",
            "macOS is pending its increment, not deferred, and the reference says which"
        );
    }
}

// Evidence: the_enumeration_answer_publishes_reach_beside_the_gated_list
#[test]
fn the_enumeration_answer_publishes_reach_beside_the_gated_list() {
    // The declaration belongs to the enumeration surface, not to replay:
    // `--replay` answers about a caller-named regular file, and a platform
    // contract is not a property of that file. Pinning both halves, so a
    // later edit cannot quietly move it.
    let json = crate::inspect::no_adapter_json();
    assert!(
        json.contains("\"reach\""),
        "the enumeration answer must publish the reach declaration"
    );
    assert!(
        json.contains(crate::reach::REACH_SCHEMA),
        "the reach payload must carry its own schema version"
    );
    assert!(
        json.contains("\"gated\""),
        "the gated list must still travel in every answer"
    );
    for state in crate::reach::STATES {
        assert!(
            json.contains(state),
            "the enumeration answer omits the INV-003 state `{state}`"
        );
    }

    let human = crate::inspect::no_adapter_human();
    assert!(
        human.contains("  reach (INV-003"),
        "the human answer must publish the reach declaration"
    );
    assert!(
        human.contains("  gated"),
        "the human answer must still carry the gated list — asserting only the order \
         of the two would pass if the gated block were dropped entirely"
    );

    let replayed = crate::inspect::replay_json(&[]);
    assert!(
        !replayed.contains("\"reach\""),
        "a replay answer is about a caller-named file; a platform contract is not a \
         property of that file and must not be rendered as though it were"
    );
}

#[cfg(target_os = "linux")]
/// A synthesized sysfs/udev tree, so no Tier-1 test reads the host's real
/// `/sys` or `/run/udev`. A path not in the map is `NotFound`, which is how
/// the positively-absent case is exercised.
struct FakeDeviceSource {
    dirs: std::collections::BTreeMap<String, Vec<String>>,
    files: std::collections::BTreeMap<String, Result<String, std::io::ErrorKind>>,
}

#[cfg(target_os = "linux")]
impl FakeDeviceSource {
    fn key(path: &std::path::Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }
}

#[cfg(target_os = "linux")]
impl crate::devices::DeviceSource for FakeDeviceSource {
    fn list_dir(&self, path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        self.dirs
            .get(&Self::key(path))
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such directory"))
    }

    fn read_value(&self, path: &std::path::Path) -> Result<String, std::io::Error> {
        match self.files.get(&Self::key(path)) {
            Some(Ok(value)) => Ok(value.clone()),
            Some(Err(kind)) => Err(std::io::Error::new(*kind, "synthesized failure")),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no such file",
            )),
        }
    }
}

#[cfg(target_os = "linux")]
/// One whole disk plus a partition child, with a present serial, an absent
/// wwid, an unreadable attribute and a udev record — every outcome class in
/// one tree.
fn one_disk_tree() -> FakeDeviceSource {
    let mut dirs = std::collections::BTreeMap::new();
    dirs.insert(
        "/sys/class/block".to_owned(),
        vec!["sda".to_owned(), "sda1".to_owned()],
    );
    let mut files = std::collections::BTreeMap::new();
    files.insert(
        "/sys/class/block/sda1/partition".to_owned(),
        Ok("1".to_owned()),
    );
    files.insert(
        "/sys/class/block/sda/size".to_owned(),
        Ok("1000215216".to_owned()),
    );
    files.insert("/sys/class/block/sda/ro".to_owned(), Ok("0".to_owned()));
    files.insert(
        "/sys/class/block/sda/removable".to_owned(),
        Ok("0".to_owned()),
    );
    files.insert(
        "/sys/class/block/sda/queue/logical_block_size".to_owned(),
        Ok("512".to_owned()),
    );
    files.insert(
        "/sys/class/block/sda/device/serial".to_owned(),
        Ok("S3Z9NB0K".to_owned()),
    );
    files.insert(
        "/sys/class/block/sda/queue/physical_block_size".to_owned(),
        Err(std::io::ErrorKind::PermissionDenied),
    );
    files.insert("/sys/class/block/sda/dev".to_owned(), Ok("8:0".to_owned()));
    files.insert(
        "/run/udev/data/b8:0".to_owned(),
        Ok("S:disk/by-id/ata-X\nE:ID_SERIAL=ata-Samsung_S3Z9NB0K\nE:ID_BUS=ata\n".to_owned()),
    );
    FakeDeviceSource { dirs, files }
}

#[cfg(target_os = "linux")]
fn enumerate_fake(source: &FakeDeviceSource) -> crate::devices::Enumeration {
    crate::devices::enumerate(
        source,
        std::path::Path::new("/sys"),
        std::path::Path::new("/run/udev/data"),
    )
}

#[cfg(target_os = "linux")]
/// The state word for an outcome, so a test can compare shapes without
/// reaching into `inspect`'s private renderer.
fn outcome_shape(outcome: &crate::inspect::Outcome) -> &'static str {
    match outcome {
        crate::inspect::Outcome::Observed(crate::inspect::ObservedValue::Absent { .. }) => "absent",
        crate::inspect::Outcome::Observed(_) => "observed",
        crate::inspect::Outcome::Unavailable { .. } => "unavailable",
        crate::inspect::Outcome::Failed { .. } => "failed",
    }
}

// Evidence: enumeration_reports_whole_devices_only
#[test]
#[cfg(target_os = "linux")]
fn enumeration_reports_whole_devices_only() {
    let source = one_disk_tree();
    let crate::devices::Enumeration::Listed(devices) = enumerate_fake(&source) else {
        panic!("the synthesized tree should enumerate");
    };
    assert_eq!(
        devices.len(),
        1,
        "sda1 carries a `partition` attribute and is not a whole device"
    );
    assert_eq!(devices[0].kernel_name, "sda");
    assert_eq!(
        devices[0].selector, "device:0",
        "selectors are session-local positions, never stable handles (ADR-0019's addresses are unconsumed here)"
    );
}

// Evidence: enumeration_keeps_the_three_outcome_classes_apart
#[test]
#[cfg(target_os = "linux")]
fn enumeration_keeps_the_three_outcome_classes_apart() {
    // ADR-C4's distinction, which an earlier draft of `inspect` got wrong: a
    // positively determined absence is a VALUE, a read error is `failed`, and
    // an interface that did not answer is `unavailable`. Collapsing any pair
    // is the fail-closed violation SAFE-005 exists to prevent.
    let source = one_disk_tree();
    let crate::devices::Enumeration::Listed(devices) = enumerate_fake(&source) else {
        panic!("the synthesized tree should enumerate");
    };
    let shape = |property: &str| {
        devices[0]
            .fields
            .iter()
            .find(|f| f.property == property)
            .map_or_else(
                || panic!("{property} missing from the row"),
                |f| outcome_shape(&f.outcome),
            )
    };

    assert_eq!(shape("device/serial"), "observed");
    assert_eq!(
        shape("device/wwid"),
        "absent",
        "an attribute that is not present is a positively determined absence"
    );
    assert_eq!(
        shape("physical_block_size"),
        "failed",
        "an unreadable attribute is `failed`, never rendered as absence"
    );
    assert_eq!(
        shape("ID_WWN"),
        "absent",
        "a key missing from a udev record that exists is absent, not unavailable"
    );
    assert_eq!(shape("ID_SERIAL"), "observed");
}

// Evidence: enumeration_is_identical_under_differing_privilege
#[test]
#[cfg(target_os = "linux")]
fn enumeration_is_identical_under_differing_privilege() {
    // The clamping obligation, made falsifiable. The record measures that
    // `disk`-group membership alone flips raw access, so a contract that
    // widened with privilege would make the published INV-003 reach
    // declaration a per-user statement — which INV-003 forbids, it being a
    // property of the contract and the platform. The adapter has no
    // privilege-conditional branch, and this fails if one is added.
    // An earlier version of this test enumerated the same fake source twice
    // and compared the results. That could never fail: nothing varied between
    // the runs, so a privilege-conditional branch would have sailed through
    // it. Verified by mutation, which is how the weakness was found. The
    // guard is therefore structural — the adapter must contain no way to ask
    // what privilege it holds.
    let source = include_str!("devices.rs");
    for needle in [
        "geteuid",
        "getuid",
        "getgid",
        "is_root",
        "effective_uid",
        "PermissionDenied =>",
    ] {
        assert!(
            !source.contains(needle),
            "devices.rs contains `{needle}`: the contract must not vary with the \
             privilege it runs under. The record measures that `disk`-group membership \
             alone flips raw access, so a widening contract would make the published \
             INV-003 reach declaration a per-user statement — which INV-003 forbids, \
             it being a property of the contract and the platform"
        );
    }

    // And the behavioural half, kept because it pins the shape even though it
    // is the weaker of the two.
    let tree = one_disk_tree();
    let render = |enumeration: crate::devices::Enumeration| match enumeration {
        crate::devices::Enumeration::Listed(devices) => devices
            .iter()
            .flat_map(|device| {
                crate::devices::observations(device)
                    .into_iter()
                    .map(|o| format!("{}={}", o.subject, outcome_shape(&o.outcome)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("|"),
        _ => panic!("the synthesized tree should enumerate"),
    };
    assert_eq!(
        render(enumerate_fake(&tree)),
        render(enumerate_fake(&tree)),
        "enumeration must be reproducible"
    );
}

// Evidence: enumeration_distinguishes_no_interface_from_no_devices
#[test]
#[cfg(target_os = "linux")]
fn enumeration_distinguishes_no_interface_from_no_devices() {
    // An empty list would say "this machine has no disks". A missing sysfs
    // block class says "this interface is not here" — a different statement,
    // and the only honest one.
    let empty = FakeDeviceSource {
        dirs: std::collections::BTreeMap::new(),
        files: std::collections::BTreeMap::new(),
    };
    assert!(
        matches!(
            enumerate_fake(&empty),
            crate::devices::Enumeration::Unavailable { .. }
        ),
        "an absent interface is `unavailable`, never an empty device list"
    );
}

// Evidence: udev_values_carry_the_cached_caveat_in_band
#[test]
#[cfg(target_os = "linux")]
fn udev_values_carry_the_cached_caveat_in_band() {
    // The record establishes that a udev database value is what root's udevd
    // computed at device-add time, not something this client observed. That
    // caveat rides on every value rather than living in a comment.
    let source = one_disk_tree();
    let crate::devices::Enumeration::Listed(devices) = enumerate_fake(&source) else {
        panic!("the synthesized tree should enumerate");
    };
    let observations = crate::devices::observations(&devices[0]);
    let udev = observations
        .iter()
        .find(|o| o.subject.contains("ID_SERIAL"))
        .expect("a udev observation");
    assert!(
        udev.attribution.method.contains("udevd"),
        "a udev value must say in-band that root's udevd computed it, not this client"
    );
    let sysfs = observations
        .iter()
        .find(|o| o.subject.contains("device/serial"))
        .expect("a sysfs observation");
    assert!(
        !sysfs.attribution.method.contains("udevd"),
        "a sysfs attribute is read by this client and must not carry the udev caveat"
    );
}

// Evidence: the_enumeration_adapter_opens_no_device_node
#[test]
fn the_enumeration_adapter_opens_no_device_node() {
    // `docs/quality/test-tiers.md`'s standing claim is that no Tier-1 test
    // opens a block device at all, read or write. This adapter is the thing
    // that could break it, so the guard is source-text rather than a promise:
    // reading `/sys/class/block/sda/size` is an attribute file, and a `/dev`
    // node never appears.
    let source = include_str!("devices.rs");
    for needle in ["/dev/", "std::process", "Command::new"] {
        assert!(
            !source.contains(needle),
            "devices.rs contains `{needle}`: this contract is sysfs and the udev \
             database, read as files. A device node or a subprocess here breaks the \
             tier boundary this package's own precondition exists to protect"
        );
    }
}

/// The XML plist skeleton the captures use, wrapped around one dict body.
fn plist_document(dict_body: &str) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n{dict_body}</dict>\n</plist>\n"
    )
    .into_bytes()
}

/// A `diskutil list -plist`-shaped document: the nested
/// `AllDisksAndPartitions` container the reader must traverse without
/// flattening, then the `WholeDisks` array the adapter consumes.
fn list_shaped(names: &[&str]) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut entries = String::new();
    for name in names {
        writeln!(entries, "\t\t<string>{name}</string>").expect("writing into a String");
    }
    plist_document(&format!(
        "\t<key>AllDisksAndPartitions</key>\n\t<array>\n\t\t<dict>\n\
         \t\t\t<key>Content</key>\n\t\t\t<string>GUID_partition_scheme</string>\n\
         \t\t\t<key>Partitions</key>\n\t\t\t<array/>\n\t\t</dict>\n\t</array>\n\
         \t<key>WholeDisks</key>\n\t<array>\n{entries}\t</array>\n"
    ))
}

/// A `diskutil info -plist`-shaped document mirroring the sitting-2
/// capture: scalars of all three kinds, a positively empty string, and an
/// empty-element nested dict.
fn info_shaped() -> Vec<u8> {
    plist_document(
        "\t<key>BusProtocol</key>\n\t<string>USB</string>\n\
         \t<key>DeviceBlockSize</key>\n\t<integer>512</integer>\n\
         \t<key>DeviceNode</key>\n\t<string>/dev/disk4</string>\n\
         \t<key>Ejectable</key>\n\t<true/>\n\
         \t<key>Internal</key>\n\t<false/>\n\
         \t<key>IORegistryEntryName</key>\n\t<string>USB SanDisk 3.2Gen1 Media</string>\n\
         \t<key>MediaName</key>\n\t<string>SanDisk 3.2Gen1</string>\n\
         \t<key>Removable</key>\n\t<true/>\n\
         \t<key>RemovableMedia</key>\n\t<true/>\n\
         \t<key>SMARTDeviceSpecificKeysMayVaryNotGuaranteed</key>\n\t<dict/>\n\
         \t<key>Size</key>\n\t<integer>250148290560</integer>\n\
         \t<key>TotalSize</key>\n\t<integer>250148290560</integer>\n\
         \t<key>VirtualOrPhysical</key>\n\t<string>Physical</string>\n\
         \t<key>VolumeName</key>\n\t<string></string>\n",
    )
}

// Requirements: INV-006
//   The bounded plist reader parses the measured diskutil shapes — the XML declaration, the DOCTYPE, nested containers, all three scalar kinds, empty-element forms, and the five predefined entities — extracting WholeDisks names and top-level info scalars verbatim, with a present-and-empty string and a present-but-container value each kept distinct from absence
// Evidence: the_plist_reader_reads_the_measured_diskutil_shapes
#[test]
fn the_plist_reader_reads_the_measured_diskutil_shapes() {
    let names = crate::plist::whole_disks(&list_shaped(&["disk0", "disk4"]))
        .expect("the measured list shape parses");
    assert_eq!(names, ["disk0", "disk4"]);

    let fields = crate::plist::info_fields(&info_shaped()).expect("the measured info shape parses");
    let get = |key: &str| {
        fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    };
    match get("MediaName") {
        Some(crate::plist::InfoValue::Scalar(text)) => assert_eq!(text, "SanDisk 3.2Gen1"),
        _ => panic!("MediaName must be a raw scalar"),
    }
    match get("Size") {
        Some(crate::plist::InfoValue::Scalar(text)) => assert_eq!(text, "250148290560"),
        _ => panic!("Size must stay raw digit text, never a number"),
    }
    match get("Removable") {
        Some(crate::plist::InfoValue::Scalar(text)) => assert_eq!(text, "true"),
        _ => panic!("a boolean renders as its textual form"),
    }
    assert!(
        matches!(
            get("VolumeName"),
            Some(crate::plist::InfoValue::EmptyString)
        ),
        "a present-and-empty string is not absence and not a scalar"
    );
    assert!(
        matches!(
            get("SMARTDeviceSpecificKeysMayVaryNotGuaranteed"),
            Some(crate::plist::InfoValue::Container)
        ),
        "a nested container is present-but-not-scalar, never flattened"
    );
    assert!(
        get("Content").is_none(),
        "the fixture carries no Content key at top level"
    );

    // The five predefined entities decode; nothing else does (below).
    let entity_doc = plist_document(
        "\t<key>MediaName</key>\n\t<string>a &amp; b &lt;c&gt; &quot;d&quot; \
         &apos;e&apos;</string>\n",
    );
    let fields = crate::plist::info_fields(&entity_doc).expect("predefined entities decode");
    match &fields[0].1 {
        crate::plist::InfoValue::Scalar(text) => assert_eq!(text, "a & b <c> \"d\" 'e'"),
        _ => panic!("the entity string is a scalar"),
    }
}

// Requirements: SAFE-005
//   The bounded plist reader fails closed on every construct outside its stated grammar — data, date, and real elements, comments, CDATA, numeric character references, undefined entities, a DOCTYPE internal subset, duplicate dictionary keys, non-UTF-8 bytes, over-depth nesting, oversize values, markup inside text, trailing bytes, and non-digit integers — refusing the whole input rather than substituting, truncating, or skipping
// Evidence: the_plist_reader_refuses_what_it_does_not_implement
#[test]
fn the_plist_reader_refuses_what_it_does_not_implement() {
    use crate::plist::{PlistRefusal, parse};

    let cases: [(&str, Vec<u8>, PlistRefusal); 12] = [
        (
            "a data element",
            plist_document("\t<key>K</key>\n\t<data>AAEC</data>\n"),
            PlistRefusal::Unsupported("a data element"),
        ),
        (
            "a date element",
            plist_document("\t<key>K</key>\n\t<date>2026-08-08T00:00:00Z</date>\n"),
            PlistRefusal::Unsupported("a date element"),
        ),
        (
            "a real element",
            plist_document("\t<key>K</key>\n\t<real>1.5</real>\n"),
            PlistRefusal::Unsupported("a real element"),
        ),
        (
            "a comment",
            plist_document("\t<key>K</key>\n\t<!-- hidden --><true/>\n"),
            PlistRefusal::Unsupported("a comment"),
        ),
        (
            "a CDATA section",
            plist_document("\t<key>K</key>\n\t<![CDATA[x]]><true/>\n"),
            PlistRefusal::Unsupported("a CDATA section"),
        ),
        (
            "a numeric character reference",
            plist_document("\t<key>K</key>\n\t<string>&#65;</string>\n"),
            PlistRefusal::Unsupported(
                "a character reference beyond the five predefined entities",
            ),
        ),
        (
            "an undefined entity",
            plist_document("\t<key>K</key>\n\t<string>&nbsp;</string>\n"),
            PlistRefusal::Unsupported(
                "a character reference beyond the five predefined entities",
            ),
        ),
        (
            "a duplicate dict key",
            plist_document("\t<key>K</key>\n\t<true/>\n\t<key>K</key>\n\t<false/>\n"),
            PlistRefusal::Malformed("a duplicate dict key"),
        ),
        (
            "a DOCTYPE internal subset",
            b"<?xml version=\"1.0\"?>\n<!DOCTYPE plist [<!ENTITY x \"y\">]>\n<plist version=\"1.0\"><dict/></plist>"
                .to_vec(),
            PlistRefusal::Unsupported("a DOCTYPE internal subset"),
        ),
        (
            "markup inside a text element",
            plist_document("\t<key>K</key>\n\t<string>a<b</string>\n"),
            PlistRefusal::Malformed("markup inside a text element"),
        ),
        (
            "a non-digit integer body",
            plist_document("\t<key>K</key>\n\t<integer>0x10</integer>\n"),
            PlistRefusal::Malformed("a non-digit integer body"),
        ),
        (
            "bytes after the document",
            b"<?xml version=\"1.0\"?>\n<plist version=\"1.0\"><dict/></plist>trailing".to_vec(),
            PlistRefusal::Malformed("bytes after </plist>"),
        ),
    ];
    for (name, bytes, expected) in cases {
        match parse(&bytes) {
            Err(refusal) => assert_eq!(refusal, expected, "{name}: wrong refusal"),
            Ok(_) => panic!("{name}: accepted by a reader that must refuse it"),
        }
    }

    // Non-UTF-8 refuses before any parsing.
    let mut broken = plist_document("\t<key>K</key>\n\t<string>ok</string>\n");
    broken[100] = 0xFF;
    assert!(matches!(parse(&broken), Err(PlistRefusal::NotUtf8)));

    // Over-depth nesting refuses at the stated limit.
    let mut nested = String::new();
    for _ in 0..=crate::plist::DEPTH_LIMIT {
        nested.push_str("<array>");
    }
    for _ in 0..=crate::plist::DEPTH_LIMIT {
        nested.push_str("</array>");
    }
    let deep = format!("<plist version=\"1.0\">{nested}</plist>").into_bytes();
    assert!(matches!(parse(&deep), Err(PlistRefusal::OverDepth)));

    // An oversize text run refuses rather than truncating.
    let long = "x".repeat(crate::plist::VALUE_LIMIT + 1);
    let oversize = plist_document(&format!("\t<key>K</key>\n\t<string>{long}</string>\n"));
    assert!(matches!(
        parse(&oversize),
        Err(PlistRefusal::OverValueLength)
    ));
}

// Requirements: SAFE-005
//   The plist reader's node-count cap refuses a document holding more values than NODE_LIMIT rather than building an unbounded tree — the one declared bound neither the engine-capped fuzz target nor the grammar refusal table can reach, so this stable boundary test is its only exercise
// Evidence: the_plist_reader_refuses_a_document_over_its_node_budget
#[test]
fn the_plist_reader_refuses_a_document_over_its_node_budget() {
    use crate::plist::{NODE_LIMIT, PlistRefusal, parse};

    // One flat array of NODE_LIMIT scalars: the array itself is the extra
    // node that crosses the budget. Built with a loop (~450 KB), well under
    // the reader's own INPUT_LIMIT, so the only cap that can fire is the
    // one under test.
    let mut over = String::with_capacity(NODE_LIMIT * 8 + 64);
    over.push_str("<plist version=\"1.0\"><array>");
    for _ in 0..NODE_LIMIT {
        over.push_str("<true/>");
    }
    over.push_str("</array></plist>");
    assert!(
        matches!(parse(over.as_bytes()), Err(PlistRefusal::OverNodeCount)),
        "NODE_LIMIT scalars plus their array must cross the node budget"
    );

    // One fewer scalar sits exactly at the budget and parses, so the cap
    // is a measured boundary rather than a region this test never located.
    let mut at_limit = String::with_capacity(NODE_LIMIT * 8 + 64);
    at_limit.push_str("<plist version=\"1.0\"><array>");
    for _ in 0..(NODE_LIMIT - 1) {
        at_limit.push_str("<true/>");
    }
    at_limit.push_str("</array></plist>");
    assert!(
        parse(at_limit.as_bytes()).is_ok(),
        "a document exactly at the node budget parses"
    );
}

/// A launcher scripting the two diskutil invocations and recording every
/// launch: path, arguments, and the stated output bound.
struct DiskutilScript {
    list: fn() -> ProbeOutcome,
    info: fn(&str) -> ProbeOutcome,
    calls: std::cell::RefCell<Vec<(String, Vec<String>, usize)>>,
}

impl DiskutilScript {
    fn new(list: fn() -> ProbeOutcome, info: fn(&str) -> ProbeOutcome) -> Self {
        Self {
            list,
            info,
            calls: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl ToolLauncher for DiskutilScript {
    fn exists(&self, _path: &Path) -> bool {
        false
    }
    fn probe_version(&self, path: &Path) -> ProbeOutcome {
        panic!("the enumeration must not version-probe: {}", path.display());
    }
    fn launch(&self, path: &Path, arguments: &[&str], output_limit: usize) -> ProbeOutcome {
        self.calls.borrow_mut().push((
            path.display().to_string(),
            arguments.iter().map(|a| (*a).to_owned()).collect(),
            output_limit,
        ));
        match arguments {
            ["list", "-plist"] => (self.list)(),
            ["info", "-plist", name] => (self.info)(name),
            other => panic!("an unexpected launch shape: {other:?}"),
        }
    }
}

fn completed(stdout: Vec<u8>) -> ProbeOutcome {
    ProbeOutcome::Completed {
        stdout,
        stderr: Vec::new(),
    }
}

// Requirements: INV-006, SAFE-004
//   The macOS adapter launches exactly diskutil list -plist and one info -plist per whole device, at the compiled absolute path with the stated output bounds, and reports the roster keys as raw interface-labelled strings — a missing key as a positively determined absence, a present-but-container value as a typed failure, never flattened — under session-local selectors in list order
// Evidence: the_macos_adapter_reports_raw_fields_from_fixed_launches
#[test]
fn the_macos_adapter_reports_raw_fields_from_fixed_launches() {
    let script = DiskutilScript::new(
        || completed(list_shaped(&["disk0", "disk4"])),
        |name| match name {
            "disk0" => completed(info_shaped()),
            // disk4's record carries no MediaName, so that key must come
            // back as a positively determined absence, not a failure.
            "disk4" => completed(plist_document(
                "\t<key>Size</key>\n\t<integer>1024</integer>\n\
                 \t<key>Ejectable</key>\n\t<dict/>\n",
            )),
            other => panic!("info for an unlisted device: {other}"),
        },
    );

    let listed = crate::macos::enumerate(&script);
    let crate::devices::Enumeration::Listed(devices) = listed else {
        panic!("two scripted disks must list");
    };
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].selector, "device:0");
    assert_eq!(devices[0].kernel_name, "disk0");
    assert_eq!(devices[1].selector, "device:1");
    assert_eq!(devices[1].kernel_name, "disk4");

    for device in &devices {
        assert_eq!(
            device.fields.len(),
            crate::macos::INFO_KEYS.len(),
            "every roster key answers, in ADR-C4's vocabulary, on every device"
        );
    }

    let field = |device: usize, key: &str| {
        devices[device]
            .fields
            .iter()
            .find(|field| field.property == key)
            .expect("every roster key is present as a row")
    };
    match &field(0, "MediaName").outcome {
        super::inspect::Outcome::Observed(super::inspect::ObservedValue::Decimal(text)) => {
            assert_eq!(text, "SanDisk 3.2Gen1");
        }
        _ => panic!("a present scalar reports its raw string"),
    }
    match &field(1, "MediaName").outcome {
        super::inspect::Outcome::Observed(super::inspect::ObservedValue::Absent { reason }) => {
            assert!(reason.contains("not present"), "absence carries its reason");
        }
        _ => panic!("a missing key is a positively determined absence"),
    }
    match &field(1, "Ejectable").outcome {
        super::inspect::Outcome::Failed { error } => {
            assert!(
                error.contains("not a scalar"),
                "a container-valued roster key is a typed failure, never flattened"
            );
        }
        _ => panic!("a present-but-container value must not read as absence or a value"),
    }

    let calls = script.calls.borrow();
    assert_eq!(
        calls.len(),
        3,
        "one list launch, one info launch per device"
    );
    for (path, _, _) in calls.iter() {
        assert_eq!(
            path,
            crate::macos::DISKUTIL,
            "only the compiled absolute path runs"
        );
    }
    assert_eq!(calls[0].1, ["list", "-plist"]);
    assert_eq!(calls[0].2, crate::macos::LIST_OUTPUT_LIMIT);
    assert_eq!(calls[1].1, ["info", "-plist", "disk0"]);
    assert_eq!(calls[1].2, crate::macos::INFO_OUTPUT_LIMIT);
    assert_eq!(calls[2].1, ["info", "-plist", "disk4"]);
}

// Requirements: SAFE-005
//   The macOS adapter fails closed on every launch and shape defect: a nonzero diskutil exit is a failure whose output is never parsed even when parseable, over-limit output refuses rather than truncates, a whole-disk name outside disk-then-digits is refused before it reaches argv, a device count over the limit refuses rather than truncates, and one device's info failure fails that device's rows without touching its neighbours
// Evidence: the_macos_adapter_fails_closed_on_launch_and_shape_defects
#[test]
fn the_macos_adapter_fails_closed_on_launch_and_shape_defects() {
    use crate::devices::Enumeration;

    // A nonzero exit with a perfectly parseable device list on stdout: the
    // list must NOT appear. If the adapter parsed it anyway, this returns
    // Listed and the assertion names the defect.
    let script = DiskutilScript::new(
        || ProbeOutcome::NonzeroExit {
            code: Some(1),
            stdout: list_shaped(&["disk0"]),
            stderr: Vec::new(),
        },
        |_| panic!("no info launch may follow a failed list"),
    );
    match crate::macos::enumerate(&script) {
        Enumeration::Failed { error } => {
            assert!(
                error.contains("not parsed"),
                "the failure says the output was not parsed: {error}"
            );
        }
        _ => panic!("a nonzero list exit is a failure, never evidence"),
    }

    let script = DiskutilScript::new(
        || ProbeOutcome::OverOutputLimit,
        |_| panic!("no info launch may follow a refused list"),
    );
    match crate::macos::enumerate(&script) {
        Enumeration::Failed { error } => {
            assert!(error.contains("refused rather than truncated"), "{error}");
        }
        _ => panic!("over-limit list output must refuse"),
    }

    // A name that is not disk-then-digits is refused before any info launch.
    let script = DiskutilScript::new(
        || completed(list_shaped(&["disk0", "-verbose"])),
        |_| panic!("a refused name must not reach a launcher"),
    );
    match crate::macos::enumerate(&script) {
        Enumeration::Failed { error } => {
            assert!(error.contains("disk<digits>"), "{error}");
        }
        _ => panic!("an unexpected whole-disk name must refuse the enumeration"),
    }

    // One device's info failure fails that device's rows and no other's.
    let script = DiskutilScript::new(
        || completed(list_shaped(&["disk0", "disk1"])),
        |name| match name {
            "disk0" => ProbeOutcome::TimedOut,
            "disk1" => completed(info_shaped()),
            other => panic!("info for an unlisted device: {other}"),
        },
    );
    match crate::macos::enumerate(&script) {
        Enumeration::Listed(devices) => {
            assert_eq!(devices.len(), 2);
            assert!(
                devices[0]
                    .fields
                    .iter()
                    .all(|field| matches!(field.outcome, super::inspect::Outcome::Failed { .. })),
                "every row of the failed device carries the failure"
            );
            assert!(
                devices[1]
                    .fields
                    .iter()
                    .any(|field| matches!(field.outcome, super::inspect::Outcome::Observed(_))),
                "the neighbour's rows are untouched by the failure"
            );
        }
        _ => panic!("a per-device info failure does not unlist the device"),
    }

    // More names than the device limit refuses rather than truncates, and
    // no info launch follows the refusal.
    let many: Vec<String> = (0..=crate::devices::DEVICE_LIMIT)
        .map(|index| format!("disk{index}"))
        .collect();
    let refs: Vec<&str> = many.iter().map(String::as_str).collect();
    match crate::macos::enumerate(&OverLimitScript {
        stdout: list_shaped(&refs),
    }) {
        Enumeration::OverLimit { seen } => assert_eq!(seen, crate::devices::DEVICE_LIMIT + 1),
        _ => panic!("a device count over the limit refuses rather than truncates"),
    }
}

/// A launcher for the over-limit case: answers the list launch with a
/// prepared document and refuses to be asked anything further.
struct OverLimitScript {
    stdout: Vec<u8>,
}

impl ToolLauncher for OverLimitScript {
    fn exists(&self, _path: &Path) -> bool {
        false
    }
    fn probe_version(&self, path: &Path) -> ProbeOutcome {
        panic!("no version probe here: {}", path.display());
    }
    fn launch(&self, _path: &Path, arguments: &[&str], _limit: usize) -> ProbeOutcome {
        assert_eq!(arguments, ["list", "-plist"], "no launch past the refusal");
        ProbeOutcome::Completed {
            stdout: self.stdout.clone(),
            stderr: Vec::new(),
        }
    }
}
