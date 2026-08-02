//! Chassis tests. Every behavior is asserted through [`dispatch`] or
//! [`dispatch_os`] as pure data; the only process any test launches is the
//! compile-time-selected `cargo`, for the structural dependency guard.

use super::{
    ALL_COMMANDS, Command, ENVELOPE_SCHEMA, EXIT_OK, EXIT_REFUSAL, EXIT_USAGE, Outcome, VERSION,
    dispatch, dispatch_os, envelope, help_text, json_escaped,
};

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

    assert_eq!(dispatch(&["help".to_owned()]).code, EXIT_OK);
    assert_eq!(dispatch(&["version".to_owned()]).code, EXIT_OK);
    assert_eq!(dispatch(&["export-diagnostics".to_owned()]).code, EXIT_OK);
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
//   A token the parser does not recognize is refused with its exact spelling rather than guessed at, in both output modes; a second command word is refused carrying the second token's spelling, never a canonical name the user did not type
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

    // The second-command refusal carries the second token's exact spelling.
    // An earlier draft reported the canonical name — `partman version -V`
    // said "unknown command `version`", declaring a known command unknown
    // and showing a word the user never typed — and only the exit code was
    // asserted, which is how the misreport survived review.
    let doubled = dispatch(&["version".to_owned(), "-V".to_owned()]);
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
    let human = dispatch(&["version".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    assert_eq!(human.stdout, format!("partman {VERSION}\n"));

    let json = dispatch(&["version".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    assert_eq!(parsed["outcome"]["kind"], "ok");
    assert_eq!(parsed["outcome"]["version"].as_str(), Some(VERSION));
}

// Requirements: SAFE-006, INV-007, CLI-002
//   The diagnostics bundle admits exactly the allowlisted fields — deny-by-default is the builder's type, not a filter — and the expected key set is pinned as literals so widening the allowlist is a visible reviewed edit
// Evidence: export_diagnostics_admits_exactly_the_allowlisted_fields
#[test]
fn export_diagnostics_admits_exactly_the_allowlisted_fields() {
    let json = dispatch(&["export-diagnostics".to_owned(), "--json".to_owned()]);
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

    // The human rendering carries the same fields, so the two modes cannot
    // quietly diverge in what they disclose.
    let human = dispatch(&["export-diagnostics".to_owned()]);
    assert_eq!(human.code, EXIT_OK);
    for key in expected {
        assert!(
            human.stdout.contains(key),
            "human diagnostics must carry `{key}` too"
        );
    }
}

// Requirements: SAFE-006
//   No output in any mode carries the host's username, home path, or computer name, nor any environment value six bytes or longer that is not byte-equal to a compile-time constant the bundle renders by definition — a tripwire so the first code path that reads the environment cannot leak it silently
// Evidence: no_output_in_any_mode_carries_an_environment_value
#[test]
fn no_output_in_any_mode_carries_an_environment_value() {
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
    // that cries wolf gets deleted. One further exemption, exact and named:
    // a value byte-equal to a compile-time constant the bundle renders by
    // definition. WSL's login shell exports HOSTTYPE=x86_64, which equals
    // `std::env::consts::ARCH` — the bundle printing its own build target is
    // not an environment read, and the byte-determinism test is what proves
    // output is environment-independent. The exemption lists the constants
    // themselves rather than loosening the sweep.
    let rendered_constants = [
        std::env::consts::OS,
        std::env::consts::ARCH,
        VERSION,
        ENVELOPE_SCHEMA,
    ];
    for (name, value) in std::env::vars() {
        if value.len() >= 6 && !rendered_constants.contains(&value.as_str()) {
            sensitive.push((name, value));
        }
    }

    for arguments in every_invocation() {
        let outcome = dispatch(&arguments);
        for (stream, text) in [("stdout", &outcome.stdout), ("stderr", &outcome.stderr)] {
            for (name, value) in &sensitive {
                assert!(
                    !text.contains(value.as_str()),
                    "{stream} of {arguments:?} contains the value of ${name} — an \
                     environment value reached the output"
                );
            }
        }
    }
}

// Requirements: SEC-007
//   The bundle is byte-identical across invocations because every field is compile-time data; nothing is read from the host, so offline is the construction rather than a mode
// Evidence: export_diagnostics_is_byte_identical_across_invocations
#[test]
fn export_diagnostics_is_byte_identical_across_invocations() {
    for arguments in [
        vec!["export-diagnostics".to_owned()],
        vec!["export-diagnostics".to_owned(), "--json".to_owned()],
    ] {
        let first = dispatch(&arguments);
        let second = dispatch(&arguments);
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
    let json = dispatch(&["export-diagnostics".to_owned(), "--json".to_owned()]);
    let parsed: serde_json::Value = serde_json::from_str(&json.stdout).expect("envelope parses");
    let evidence = &parsed["outcome"]["diagnostics"]["discovery-evidence"];
    assert_eq!(evidence["state"], "not-implemented");
    assert_eq!(evidence["reference"], "WP-035 increment 4");
    assert!(
        evidence["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("allowlist")),
        "the refusal must say how future evidence enters the bundle: through the allowlist"
    );

    let human = dispatch(&["export-diagnostics".to_owned()]);
    assert!(
        human.stdout.contains("discovery-evidence: not-implemented"),
        "the human bundle must carry the refusal in-band too: {}",
        human.stdout
    );
}
