//! Increment 1's suite: the published table as the only representable
//! shape, the terminal-with-effect structure, and the byte-fresh
//! machine-readable document.

use crate::{Effect, State, TerminalRecord, Transition};

/// Section 8's transition table, transcribed by hand from the
/// specification as `(from, to)` name pairs — deliberately a second,
/// independent spelling so the property test compares the crate's
/// variants against the specification's rows rather than against
/// itself.
const PUBLISHED_ROWS: [(&str, &str); 23] = [
    ("Draft", "Validated"),
    ("Validated", "Draft"),
    ("Validated", "AwaitingAuthorization"),
    ("AwaitingAuthorization", "Revalidating"),
    ("AwaitingAuthorization", "Cancelled"),
    ("Revalidating", "Protecting"),
    ("Revalidating", "Failed"),
    ("Protecting", "Executing"),
    ("Protecting", "Failed"),
    ("Executing", "Verifying"),
    ("Executing", "Paused"),
    ("Executing", "RebootPending"),
    ("Executing", "RecoveryRequired"),
    ("Executing", "Cancelled"),
    ("Paused", "Executing"),
    ("Paused", "Cancelled"),
    ("Paused", "RecoveryRequired"),
    ("RebootPending", "Revalidating"),
    ("RebootPending", "RecoveryRequired"),
    ("Verifying", "Completed"),
    ("Verifying", "RecoveryRequired"),
    ("RecoveryRequired", "Executing"),
    ("RecoveryRequired", "Failed"),
];

// Requirements: Section 8, Section 11.6
//   The undeclared-transition obligation (ADR-0027 obligation 1,
//   imported by the assignment): over all 169 ordered state pairs, a
//   Transition variant exists exactly for the 23 published rows — an
//   undeclared pair has no variant and is therefore unrepresentable at
//   construction, and no two variants share a row, so the variant set
//   *is* the published table, compared here against an independent
//   transcription of the specification rather than against itself.
// Evidence: undeclared_transitions_are_unrepresentable
#[test]
fn undeclared_transitions_are_unrepresentable() {
    let mut published: Vec<(&str, &str)> = PUBLISHED_ROWS.to_vec();
    published.sort_unstable();
    let mut encoded: Vec<(&str, &str)> = Transition::ALL
        .iter()
        .map(|t| (t.from().name(), t.to().name()))
        .collect();
    encoded.sort_unstable();
    assert_eq!(
        encoded, published,
        "the variant set must equal the published table exactly"
    );

    for from in State::ALL {
        for to in State::ALL {
            let declared = PUBLISHED_ROWS
                .iter()
                .any(|&(f, t)| f == from.name() && t == to.name());
            let representable = Transition::ALL
                .iter()
                .any(|t| t.from() == from && t.to() == to);
            assert_eq!(
                representable,
                declared,
                "pair {} -> {} must be representable iff published",
                from.name(),
                to.name()
            );
        }
    }
}

// Requirements: Section 8
//   Terminal states are exactly the published three, no transition
//   leaves a terminal state, and every non-terminal state is exited by
//   at least one declared transition — RecoveryRequired persists until
//   the user acts, and its two exits are ADR-0027's two arms
//   (roll-forward to Executing; accepted failure to Failed), asserted
//   here as the exact exit set.
// Evidence: terminal_states_are_exactly_the_published_three
#[test]
fn terminal_states_are_exactly_the_published_three() {
    for state in State::ALL {
        let expected = matches!(state.name(), "Completed" | "Failed" | "Cancelled");
        assert_eq!(state.is_terminal(), expected, "{}", state.name());
        let has_exit = Transition::ALL.iter().any(|t| t.from() == state);
        assert_eq!(
            has_exit,
            !state.is_terminal(),
            "{} must have an exit iff non-terminal",
            state.name()
        );
    }

    let mut recovery_exits: Vec<&str> = Transition::ALL
        .iter()
        .filter(|t| t.from() == State::RecoveryRequired)
        .map(|t| t.to().name())
        .collect();
    recovery_exits.sort_unstable();
    assert_eq!(
        recovery_exits,
        ["Executing", "Failed"],
        "ADR-0027's two arms are the exact RecoveryRequired exit set"
    );
}

// Requirements: Section 8
//   Every terminal record includes an effect summary, structurally: a
//   TerminalRecord constructs only for the three terminal states and
//   always carries its effect, a non-terminal state is a typed refusal
//   naming itself, and the effect constraints the published rows state
//   are carried per transition — no-writes alone on the three
//   no-writes rows, no-writes-or-partial on the honored cancel, and
//   None where the row constrains nothing (per-journal, and the
//   Completed entry whose record still carries its effect).
// Evidence: every_terminal_record_carries_an_effect
#[test]
fn every_terminal_record_carries_an_effect() {
    for state in State::ALL {
        let record = TerminalRecord::new(state, Effect::NoWrites);
        assert_eq!(record.is_ok(), state.is_terminal(), "{}", state.name());
        if let Ok(record) = record {
            assert_eq!(record.terminal(), state);
            assert_eq!(record.effect(), Effect::NoWrites);
        } else if let Err(refusal) = record {
            assert_eq!(refusal.state, state, "the refusal names the state");
        }
    }

    for transition in Transition::ALL {
        let constraint = transition.effect_constraint();
        match transition {
            Transition::DeclinedOrExpired
            | Transition::IdentityMismatch
            | Transition::BackupFailure => {
                assert_eq!(constraint, Some(&[Effect::NoWrites][..]), "{transition:?}");
            }
            Transition::CancelHonored => {
                assert_eq!(
                    constraint,
                    Some(&[Effect::NoWrites, Effect::Partial][..]),
                    "{transition:?}"
                );
            }
            _ => assert_eq!(constraint, None, "{transition:?}"),
        }
        if constraint.is_some() {
            assert!(
                transition.to().is_terminal(),
                "only terminal-entering rows state effect constraints"
            );
        }
    }
}

// Requirements: Section 8, Section 11.6
//   The machine-readable table Section 8 requires under schemas/ is one
//   source with the types: schemas/state-machine.md is byte-equal to
//   published_markdown()'s render of the same variants the property
//   tests prove equal to the specification's rows — drift between the
//   document, the types, and the spec fails the suite rather than
//   waiting for a reader to notice.
// Evidence: the_published_table_is_byte_fresh
#[test]
fn the_published_table_is_byte_fresh() {
    let committed = include_str!("../../../schemas/state-machine.md");
    let rendered = crate::published_markdown();
    assert_eq!(
        committed, rendered,
        "schemas/state-machine.md must be byte-equal to published_markdown(); \
         regenerate by writing that function's output over the file"
    );
}
