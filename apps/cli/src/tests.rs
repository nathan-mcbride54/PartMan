//! Chassis tests. Every behavior is asserted through [`dispatch`] as pure
//! data; the only process any test launches is the compile-time-selected
//! `cargo`, for the structural dependency guard.

use super::{
    Command, ENVELOPE_SCHEMA, EXIT_OK, EXIT_REFUSAL, EXIT_USAGE, Outcome, VERSION, dispatch,
    envelope, help_text, json_escaped,
};

/// Every distinct invocation shape the chassis has, so contract-wide tests
/// cannot quietly skip a surface that was added later without extending this
/// list — a test over a hand-picked subset proves the subset.
fn every_invocation() -> Vec<Vec<String>> {
    let mut invocations = Vec::new();
    for command in ["help", "version", "inspect", "frobnicate", "--frob", ""] {
        for json in [false, true] {
            let mut arguments: Vec<String> = Vec::new();
            if !command.is_empty() {
                arguments.push(command.to_owned());
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
        .map(|arguments| dispatch(&arguments))
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
        let outcome = dispatch(&arguments);
        for (stream, text) in [("stdout", &outcome.stdout), ("stderr", &outcome.stderr)] {
            assert!(
                !text.contains('\u{1b}'),
                "{stream} of {arguments:?} contains an ANSI escape byte"
            );
        }
    }
}

// Requirements: SAFE-005, Section 12
//   An unimplemented surface refuses with a typed value carrying state, reference, and detail on stdout — never a silent omission, a bare exit code, or a plausible fake success
// Evidence: inspect_refuses_with_a_typed_value_not_only_an_exit_code
#[test]
fn inspect_refuses_with_a_typed_value_not_only_an_exit_code() {
    let human = dispatch(&["inspect".to_owned()]);
    assert_eq!(human.code, EXIT_REFUSAL);
    for field in ["state: not-implemented", "reference: WP-035 increment 4"] {
        assert!(
            human.stdout.contains(field),
            "the human refusal must carry `{field}` on stdout"
        );
    }

    let json = dispatch(&["inspect".to_owned(), "--json".to_owned()]);
    assert_eq!(json.code, EXIT_REFUSAL);
    let parsed: serde_json::Value =
        serde_json::from_str(&json.stdout).expect("refusals parse like any envelope");
    let outcome_object = &parsed["outcome"];
    assert_eq!(outcome_object["kind"], "refusal");
    assert_eq!(outcome_object["state"], "not-implemented");
    assert_eq!(outcome_object["reference"], "WP-035 increment 4");
    assert!(
        outcome_object["detail"]
            .as_str()
            .is_some_and(|detail| !detail.is_empty()),
        "a refusal without a human-actionable detail is an error code wearing a hat"
    );
}

// Requirements: CLI-005
//   The exit-code contract is documented in the help text from the same constants the binary returns, so the documentation cannot drift from the code
// Evidence: exit_codes_match_the_contract_the_help_text_documents
#[test]
fn exit_codes_match_the_contract_the_help_text_documents() {
    assert_eq!(dispatch(&["help".to_owned()]).code, EXIT_OK);
    assert_eq!(dispatch(&["version".to_owned()]).code, EXIT_OK);
    assert_eq!(dispatch(&["inspect".to_owned()]).code, EXIT_REFUSAL);
    assert_eq!(dispatch(&["frobnicate".to_owned()]).code, EXIT_USAGE);
    assert_eq!(dispatch(&[]).code, EXIT_USAGE);

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
//   A token the parser does not recognize is refused with its exact spelling rather than guessed at, in both output modes, and a second command word is refused rather than either being kept
// Evidence: unknown_tokens_are_refused_with_their_exact_spelling
#[test]
fn unknown_tokens_are_refused_with_their_exact_spelling() {
    let command = dispatch(&["frobnicate".to_owned()]);
    assert_eq!(command.code, EXIT_USAGE);
    assert!(command.stderr.contains("unknown command `frobnicate`"));
    assert!(command.stdout.is_empty());

    let flag = dispatch(&["--frob".to_owned(), "version".to_owned()]);
    assert_eq!(flag.code, EXIT_USAGE);
    assert!(flag.stderr.contains("unknown flag `--frob`"));

    let doubled = dispatch(&["version".to_owned(), "help".to_owned()]);
    assert_eq!(
        doubled.code, EXIT_USAGE,
        "two command words are a refusal, not a silent choice of either"
    );

    let json = dispatch(&["frobnicate".to_owned(), "--json".to_owned()]);
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
//   The shipped dependency closure is empty — no normal or build dependency exists — which is what keeps a hash function unreachable from inspector output and keeps every plan type out of this binary's reach
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
        // binary, which is exactly the boundary this guard draws; this
        // assertion is a snapshot of the manifest, not of what tests link.
        .filter(|dependency| dependency["kind"] != "dev")
        .map(|dependency| dependency["name"].to_string())
        .collect();
    assert!(
        shipped.is_empty(),
        "the shipped closure gained {shipped:?}; widening it is a reviewed decision — the \
         guard exists so a hash function or plan type cannot arrive as a transitive convenience"
    );
}

// Requirements: MODEL-003
//   The version surface reports the workspace package version through the same envelope discipline as every other surface
// Evidence: version_reports_through_the_envelope
#[test]
fn version_reports_through_the_envelope() {
    let human = dispatch(&["version".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    assert_eq!(human.stdout, format!("partman {VERSION}\n"));

    let json = dispatch(&["version".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    assert_eq!(parsed["outcome"]["kind"], "ok");
    assert_eq!(parsed["outcome"]["version"].as_str(), Some(VERSION));
}
