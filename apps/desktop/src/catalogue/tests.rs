use std::collections::BTreeSet;

use partman_tokens::TokenSet;

use super::{EnglishCatalogue, ExactByteFactId, Message, TOKEN_LABELS, TextId};

fn repository_token_label_ids() -> BTreeSet<String> {
    let tokens = TokenSet::load_repository_tokens().expect("repository tokens load");
    let mut ids = BTreeSet::new();
    ids.insert(tokens.theme_signals.system_selection_label_id);
    ids.extend(tokens.themes.into_values().map(|theme| theme.label_id));
    ids.extend(
        tokens
            .non_color_channels
            .roles
            .into_values()
            .map(|channels| channels.label_id),
    );
    ids
}

// Requirements: UI-001, UI-003, UI-011, UI-013, Section 12
//   The independent English catalogue resolves exactly every canonical v2 theme and semantic label ID without a missing or invented entry
// Evidence: the_catalogue_and_repository_token_label_rosters_are_equal
#[test]
fn the_catalogue_and_repository_token_label_rosters_are_equal() {
    let repository = repository_token_label_ids();
    let catalogue = EnglishCatalogue::token_label_ids()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    assert_eq!(repository, catalogue);
    assert_eq!(catalogue.len(), 25, "the current v2 roster has 25 IDs");

    for id in catalogue {
        let value = EnglishCatalogue::resolve_token_label(&id)
            .expect("every enumerated token label ID resolves");
        assert!(!value.trim().is_empty(), "{id} must not resolve blank");
    }
}

// Requirements: UI-013, Section 12
//   Every closed static shell, accessibility, empty-state, and error-copy ID has one stable key and a nonblank English value
// Evidence: every_static_catalogue_entry_is_unique_and_nonblank
#[test]
fn every_static_catalogue_entry_is_unique_and_nonblank() {
    let mut keys = BTreeSet::new();
    for id in TextId::ALL {
        assert!(keys.insert(id.key()), "duplicate key {}", id.key());
        let value = EnglishCatalogue::resolve(*id);
        assert!(!value.trim().is_empty(), "{} resolves blank", id.key());
        assert!(
            !value.chars().any(char::is_control),
            "{} contains a control character",
            id.key()
        );
    }

    assert_eq!(keys.len(), TextId::ALL.len());
}

// Requirements: UI-013, Section 12
//   Token IDs are unique, their English values are nonblank, and an unknown generated ID fails closed instead of rendering a placeholder
// Evidence: token_label_resolution_is_total_only_for_the_declared_roster
#[test]
fn token_label_resolution_is_total_only_for_the_declared_roster() {
    let ids = TOKEN_LABELS
        .iter()
        .map(|entry| entry.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), TOKEN_LABELS.len(), "token IDs must be unique");
    assert!(TOKEN_LABELS.iter().all(|entry| !entry.english.is_empty()));

    let error = EnglishCatalogue::resolve_token_label("meaning.entity.disk")
        .expect_err("an invented ID must fail");
    assert_eq!(error.id(), "meaning.entity.disk");
    assert!(error.to_string().contains("meaning.entity.disk"));
}

// Requirements: UI-008, UI-013, Section 12
//   Count grammar and composed accessible exact-byte labels are centralized in typed catalogue messages rather than component-owned English
// Evidence: parameterized_messages_apply_the_english_catalogue_grammar
#[test]
fn parameterized_messages_apply_the_english_catalogue_grammar() {
    assert_eq!(
        EnglishCatalogue::format(Message::DeviceCount(0)),
        "0 synthetic devices"
    );
    assert_eq!(
        EnglishCatalogue::format(Message::DeviceCount(1)),
        "1 synthetic device"
    );
    assert_eq!(
        EnglishCatalogue::format(Message::DeviceCount(2)),
        "2 synthetic devices"
    );
    assert_eq!(
        EnglishCatalogue::format(Message::TopologyItemCount(1)),
        "1 topology item"
    );
    assert_eq!(
        EnglishCatalogue::format(Message::TopologyItemCount(2)),
        "2 topology items"
    );
    assert_eq!(
        EnglishCatalogue::format(Message::ExactFactLabel(ExactByteFactId::StartOffset)),
        "Start offset, exact bytes"
    );
    for fact in [
        ExactByteFactId::Size,
        ExactByteFactId::Alignment,
        ExactByteFactId::ClusterSize,
        ExactByteFactId::DeviceSize,
    ] {
        assert!(EnglishCatalogue::format(Message::ExactFactLabel(fact)).ends_with(", exact bytes"));
    }
    assert_eq!(
        EnglishCatalogue::format(Message::Text(TextId::SelectedLabel)),
        "Selected"
    );
}
