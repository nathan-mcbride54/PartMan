//! Chassis tests. Every behavior is asserted through [`dispatch_with`] over
//! a scripted launcher, or [`dispatch_os`], as pure data. The only
//! executable any test launches is the compile-time-selected `cargo` —
//! twice: as the structural dependency guard's oracle, and as the real
//! launcher's probe subject — so the tier's process set stays `git` and
//! `cargo`, and no Tier-1 test ever launches a roster tool.

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
    for fragment in [
        "adapters: not-implemented",
        "identity-strength: not-established (SI-28)",
        "partition-table-state: not-established (SI-35)",
        "same-device-claims: not-established (SI-12)",
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
    let gated = inspect_object["gated"]
        .as_array()
        .expect("the gated list is part of every inspect answer");
    let gate_ids: Vec<&str> = gated
        .iter()
        .filter_map(|entry| entry["gate"].as_str())
        .collect();
    for gate in ["SI-28", "SI-35", "SI-12"] {
        assert!(
            gate_ids.contains(&gate),
            "the gated list must carry {gate}: {gate_ids:?}"
        );
    }
    for entry in gated {
        assert_eq!(
            entry["state"], "not-established",
            "a gated surface is not-established, never silently omitted"
        );
    }
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

// Requirements: SAFE-005
//   A token the parser does not recognize is refused with its exact spelling rather than guessed at, in both output modes; a second command word is refused carrying the second token's spelling, never a canonical name the user did not type
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

    let json = fdispatch(&["frobnicate".to_owned(), "--json".to_owned()]);
    assert_eq!(json.code, EXIT_USAGE);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("usage refusals are envelopes too");
    assert_eq!(parsed["outcome"]["kind"], "usage-refusal");
    assert_eq!(
        parsed["command"],
        serde_json::Value::Null,
        "no command was accepted, and the field says so rather than being omitted"
    );
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
        std::ffi::OsString::from_wide(&[0xD800])
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        std::ffi::OsString::from_vec(vec![0xFF])
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
        "escape byte \u{1b}[31m and null \u{0}",
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

// Requirements: Section 14
//   No normal or build dependency exists, so no hash or plan implementation can arrive from outside the crate; std's own hashers are held off the output type by its compile-fail non-Hash proof, and past that the boundary is a named review obligation
// Evidence: the_shipped_dependency_closure_is_empty
#[test]
fn the_shipped_dependency_closure_is_empty() {
    // The compile-time-selected cargo, launched with a structured argument
    // array — the same discipline every other Tier-1 gate uses. This is the
    // only process any chassis test launches.
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
//   The shipped sources contain no environment read — env::var, env::vars, and var_os are absent from every shipped source file the test enumerates: lib.rs, main.rs, doctor.rs, and facts.rs — so an environment value cannot reach output regardless of which variables the host sets; compile-time env::consts and env! are the allowed forms
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
    let cases: [(ProbeScript, &str); 5] = [
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
//   The launcher's child environment is cleared and gains exactly one written variable, LC_ALL=C — pinned as a source-text assertion because no behavioral proof fits the tier's process set, with the pin's reach stated: direct spellings, with smuggling routes closed by the empty closure and the wildcard-import lint, and the residue held by review
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
//   The real launcher answers with bounded output, completing before the deadline fires, for a tool that exists, launched by absolute path with a structured argument array; the deadline's kill path is exercised by review and manual probe only, as doctor.rs states, and the probe subject is the compile-time-selected cargo, already in the tier's process set
// Evidence: the_real_launcher_answers_bounded_with_provenance
#[test]
fn the_real_launcher_answers_bounded_with_provenance() {
    let cargo = Path::new(env!("CARGO"));
    assert!(cargo.is_absolute(), "cargo's compile-time path is absolute");
    let launcher = SystemLauncher;
    assert!(
        launcher.exists(cargo),
        "the toolchain that built this test exists"
    );
    match launcher.probe_version(cargo) {
        ProbeOutcome::Completed { stdout, .. } => {
            assert!(!stdout.is_empty(), "cargo --version banners on stdout");
            assert!(stdout.len() <= 4096, "output stayed within the bound");
            let banner = super::doctor::sanitized_first_line(&stdout);
            assert!(
                banner.contains("cargo"),
                "provenance keeps the raw line: {banner}"
            );
            assert!(
                parse_version(&banner).is_some(),
                "a real toolchain banner parses: {banner}"
            );
        }
        other => panic!(
            "cargo --version must complete within the limits; got {}",
            match other {
                ProbeOutcome::TimedOut => "timed-out",
                ProbeOutcome::OverOutputLimit => "over-output-limit",
                ProbeOutcome::LaunchFailed(_) => "launch-failed",
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
//   Every replay observation carries its attribution — adapter name, version, method — and replay over one of WP-020's deterministic images is byte-reproducible, so the record is raw discovery evidence a reader can re-derive, not narrative
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
        "one length observation plus one per probe"
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
            observation["outcome"]["state"], "value",
            "every probe lands inside this image, so every outcome is a value"
        );
    }
    // Two deterministic anchors a reader can check against the fixture's
    // own definition: the object's length, and the bytes at 510..512 —
    // reported as hex, interpreted by nobody.
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

    // The gated list travels with every answer.
    let gated = inspect_object["gated"].as_array().expect("gated list");
    assert!(!gated.is_empty(), "the gated list is never omitted");

    // No path echo: the answer must not contain the temporary file's path.
    assert!(
        !first.stdout.contains(&object.path.display().to_string()),
        "a path is on SEC-006's deny-floor; the caller knows what they named"
    );
}

// Requirements: SAFE-005, INV-007
//   A probe beyond the object's end is a positively observed absence — knowledge derived from the handle's own length — while a read failure is unavailability, and the two are distinct outcome states because treating could-not-look as looked-and-found-nothing is the fail-closed violation
// Evidence: absence_and_unavailability_are_distinct_outcomes
#[test]
fn absence_and_unavailability_are_distinct_outcomes() {
    // A 100-byte object: every probe past byte 100 is observed-absent.
    let object = TempObject::holding("tiny", &[0xAB; 100]);
    let outcome = replay_invocation(&object.path, true);
    assert_eq!(outcome.code, EXIT_OK);
    let parsed: serde_json::Value = serde_json::from_str(&outcome.stdout).expect("envelope");
    let observations = parsed["outcome"]["inspect"]["observations"]
        .as_array()
        .expect("observations");

    let absent: Vec<&serde_json::Value> = observations
        .iter()
        .filter(|observation| observation["outcome"]["state"] == "observed-absent")
        .collect();
    assert!(
        !absent.is_empty(),
        "probes beyond a 100-byte object must be observed-absent"
    );
    for observation in &absent {
        let reason = observation["outcome"]["reason"]
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
    // The in-range probe is still a value.
    assert_eq!(
        observations
            .iter()
            .find(|observation| observation["subject"] == "bytes[0..16)")
            .expect("the head probe exists")["outcome"]["state"],
        "value"
    );

    // Unavailability is a distinct rendered state with a distinct meaning.
    // No portable Tier-1 setup forces a mid-file read error, so the
    // unavailable arm is exercised at the rendering seam with a constructed
    // observation — stated here so this test's claim is not read as
    // behavioral coverage of I/O failure.
    let constructed = super::inspect::Observation {
        subject: "bytes[0..16)".to_owned(),
        attribution: super::inspect::Attribution {
            adapter: "fixture-replay",
            version: VERSION,
            method: "seek-and-read through the verified handle",
        },
        outcome: super::inspect::Outcome::Unavailable {
            reason: "the read failed: simulated I/O error".to_owned(),
        },
    };
    let rendered = super::inspect::replay_json(&[constructed]);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("render parses");
    assert_eq!(
        parsed["observations"][0]["outcome"]["state"], "unavailable",
        "unavailability renders as its own state"
    );
    assert!(
        parsed["observations"][0]["outcome"].get("value").is_none(),
        "an unavailable observation must never masquerade as a value"
    );
}

// Requirements: SAFE-005
//   A replayed object that the opened handle reports as not a regular file is refused with a typed value before its first byte is read, and an unopenable path is a typed refusal, not a panic — so a device named by path is refused unread and the no-device-opens sentence survives
// Evidence: replay_refuses_non_regular_objects_with_a_typed_value
#[test]
fn replay_refuses_non_regular_objects_with_a_typed_value() {
    // A directory: openable on some platforms, never a regular file.
    let directory = std::env::temp_dir();
    let refused = replay_invocation(&directory, false);
    assert_eq!(refused.code, EXIT_REFUSAL);
    assert!(
        refused.stdout.contains("state: refused"),
        "the refusal is a typed value on stdout: {}",
        refused.stdout
    );

    let json = replay_invocation(&directory, true);
    assert_eq!(json.code, EXIT_REFUSAL);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("refusals ride the envelope");
    assert_eq!(parsed["outcome"]["kind"], "refusal");
    assert_eq!(parsed["outcome"]["state"], "refused");
    assert_eq!(parsed["outcome"]["reference"], "SAFE-005");

    // A path that does not exist: refused, typed, at the documented code.
    let missing = std::env::temp_dir().join("partman-inc4-does-not-exist");
    let refused = replay_invocation(&missing, true);
    assert_eq!(refused.code, EXIT_REFUSAL);
    let parsed: serde_json::Value =
        serde_json::from_str(&refused.stdout).expect("refusals ride the envelope");
    assert_eq!(parsed["outcome"]["kind"], "refusal");
}

// Requirements: Section 12, SAFE-005
//   Inspect output reports bytes and access facts and never classifications: the vocabulary of interpretation — table formats, signatures, identity strength — is mechanically refused in both renderings, because classification is exactly what the register holds open
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
    // its purpose — the observations are where a classification would be a
    // violation.
    let json = replay_invocation(&object.path, true);
    assert_eq!(json.code, EXIT_OK);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope");
    let observations_text = parsed["outcome"]["inspect"]["observations"]
        .to_string()
        .to_lowercase();

    let human = replay_invocation(&object.path, false);
    assert_eq!(human.code, EXIT_OK);
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
//   The --replay flag's malformed shapes are structured usage refusals with exact spellings: a missing value, a second occurrence, and attachment to a command that is not inspect are each refused rather than guessed at
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
