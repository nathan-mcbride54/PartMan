use std::collections::HashSet;

use super::{DisplayLimit, RawIdentifier};

fn hex_nibble(byte: u8) -> Option<u16> {
    match byte {
        b'0'..=b'9' => Some(u16::from(byte - b'0')),
        b'A'..=b'F' => Some(u16::from(byte - b'A' + 10)),
        _ => None,
    }
}

fn decode_bytes(display: &str) -> Result<Vec<u8>, &'static str> {
    let encoded = display
        .strip_prefix("b:")
        .ok_or("missing byte-domain prefix")?
        .as_bytes();
    let mut decoded = Vec::new();
    let mut index = 0;
    while index < encoded.len() {
        match encoded[index] {
            b'\\' if encoded.get(index + 1) == Some(&b'\\') => {
                decoded.push(b'\\');
                index += 2;
            }
            b'\\' if encoded.get(index + 1) == Some(&b'x') => {
                let high = encoded
                    .get(index + 2)
                    .copied()
                    .and_then(hex_nibble)
                    .ok_or("invalid byte escape")?;
                let low = encoded
                    .get(index + 3)
                    .copied()
                    .and_then(hex_nibble)
                    .ok_or("invalid byte escape")?;
                decoded.push(u8::try_from((high << 4) | low).expect("two hex digits fit u8"));
                index += 4;
            }
            b'\\' => return Err("unknown byte escape"),
            byte @ 0x20..=0x7e => {
                decoded.push(byte);
                index += 1;
            }
            _ => return Err("non-ASCII byte presentation"),
        }
    }
    Ok(decoded)
}

fn decode_wtf16(display: &str) -> Result<Vec<u16>, &'static str> {
    let encoded = display
        .strip_prefix("w:")
        .ok_or("missing WTF-16-domain prefix")?
        .as_bytes();
    let mut decoded = Vec::new();
    let mut index = 0;
    while index < encoded.len() {
        match encoded[index] {
            b'\\' if encoded.get(index + 1) == Some(&b'\\') => {
                decoded.push(u16::from(b'\\'));
                index += 2;
            }
            b'\\' if encoded.get(index + 1) == Some(&b'u') => {
                let mut unit = 0_u16;
                for offset in 2..6 {
                    let nibble = encoded
                        .get(index + offset)
                        .copied()
                        .and_then(hex_nibble)
                        .ok_or("invalid WTF-16 escape")?;
                    unit = (unit << 4) | nibble;
                }
                decoded.push(unit);
                index += 6;
            }
            b'\\' => return Err("unknown WTF-16 escape"),
            byte @ 0x20..=0x7e => {
                decoded.push(u16::from(byte));
                index += 1;
            }
            _ => return Err("non-ASCII WTF-16 presentation"),
        }
    }
    Ok(decoded)
}

fn assert_fragment_has_complete_tokens(fragment: &str, escape: u8, digits: usize) {
    let encoded = fragment.as_bytes();
    let mut index = 0;
    while index < encoded.len() {
        match encoded[index] {
            b'\\' if encoded.get(index + 1) == Some(&b'\\') => index += 2,
            b'\\' if encoded.get(index + 1) == Some(&escape) => {
                let end = index + 2 + digits;
                assert!(end <= encoded.len(), "partial escape in {fragment:?}");
                assert!(
                    encoded[index + 2..end]
                        .iter()
                        .copied()
                        .all(|byte| hex_nibble(byte).is_some()),
                    "noncanonical escape in {fragment:?}"
                );
                index = end;
            }
            b'\\' => panic!("unknown escape in {fragment:?}"),
            0x20..=0x7e => index += 1,
            _ => panic!("non-ASCII source token in {fragment:?}"),
        }
    }
}

fn assert_bounded_tokens(display: &str, truncated: bool) {
    let (prefix, escape, digits) = if display.starts_with("b:") {
        ("b:", b'x', 2)
    } else {
        assert!(display.starts_with("w:"));
        ("w:", b'u', 4)
    };
    let body = &display[prefix.len()..];
    if truncated {
        let parts = body.split('…').collect::<Vec<_>>();
        assert_eq!(parts.len(), 2, "one unambiguous marker is required");
        assert_fragment_has_complete_tokens(parts[0], escape, digits);
        assert_fragment_has_complete_tokens(parts[1], escape, digits);
    } else {
        assert!(!body.contains('…'));
        assert_fragment_has_complete_tokens(body, escape, digits);
    }
}

// Requirements: UI-008, Section 12
//   Every byte and every WTF-16 code unit has a unique domain-separated full display that an independent decoder returns to the exact authority value
// Evidence: every_single_authority_unit_round_trips_without_collision
#[test]
fn every_single_authority_unit_round_trips_without_collision() {
    let mut byte_displays = HashSet::new();
    for byte in u8::MIN..=u8::MAX {
        let raw = RawIdentifier::Bytes(Box::new([byte]));
        let display = raw.full_display();
        assert_eq!(decode_bytes(display.as_str()), Ok(vec![byte]));
        assert!(byte_displays.insert(display.to_string()));
    }

    let mut wide_displays = HashSet::new();
    for unit in u16::MIN..=u16::MAX {
        let raw = RawIdentifier::Wtf16(Box::new([unit]));
        let display = raw.full_display();
        assert_eq!(decode_wtf16(display.as_str()), Ok(vec![unit]));
        assert!(wide_displays.insert(display.to_string()));
    }

    assert!(byte_displays.is_disjoint(&wide_displays));
}

// Requirements: UI-008, Section 12
//   Invalid UTF-8, ill-formed WTF-16, controls, bidi, combining text, non-Western text, literal backslashes, and escape lookalikes all retain their exact original units
// Evidence: the_hostile_identifier_corpus_round_trips_exactly
#[test]
fn the_hostile_identifier_corpus_round_trips_exactly() {
    let byte_corpus = [
        Vec::new(),
        vec![0],
        vec![b'\n', b'\r', b'\t'],
        vec![0x80],
        vec![0xc0, 0xaf],
        vec![0xe2, 0x82],
        br"literal\backslash\x41".to_vec(),
        "a\u{0301} · \u{202e} · 😀 · 中 · ＼".as_bytes().to_vec(),
    ];
    for source in byte_corpus {
        let display = RawIdentifier::Bytes(source.clone().into_boxed_slice()).full_display();
        assert_eq!(decode_bytes(display.as_str()), Ok(source));
        assert!(!display.as_str().chars().any(char::is_control));
        assert!(!display.as_str().contains('\u{202e}'));
    }

    let mut readable_wide = "a\u{0301} · \u{202e} · 😀 · 中 · ＼"
        .encode_utf16()
        .collect::<Vec<_>>();
    readable_wide.extend([0, b'\n'.into(), 0xd800, 0xdc00]);
    let wide_corpus = [
        Vec::new(),
        vec![0xd800],
        vec![0xdc00],
        vec![0xd800, 0x0041, 0xdc00],
        "literal\\backslash\\uD800".encode_utf16().collect(),
        readable_wide,
    ];
    for source in wide_corpus {
        let display = RawIdentifier::Wtf16(source.clone().into_boxed_slice()).full_display();
        assert_eq!(decode_wtf16(display.as_str()), Ok(source));
        assert!(!display.as_str().chars().any(char::is_control));
        assert!(!display.as_str().contains('\u{202e}'));
    }
}

// Requirements: UI-008, Section 12
//   Literal backslashes and text that resembles an escape remain distinct from escaped authority units in both representation domains
// Evidence: escape_lookalikes_cannot_collide_with_encoded_units
#[test]
fn escape_lookalikes_cannot_collide_with_encoded_units() {
    let byte_lookalike = RawIdentifier::Bytes(br"\x41".to_vec().into_boxed_slice()).full_display();
    let byte_value = RawIdentifier::Bytes(Box::new([b'A'])).full_display();
    assert_eq!(byte_lookalike.as_str(), r"b:\\x41");
    assert_eq!(byte_value.as_str(), "b:A");
    assert_ne!(byte_lookalike, byte_value);

    let wide_lookalike = RawIdentifier::Wtf16(
        "\\uD800"
            .encode_utf16()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
    .full_display();
    let wide_value = RawIdentifier::Wtf16(Box::new([0xd800])).full_display();
    assert_eq!(wide_lookalike.as_str(), r"w:\\uD800");
    assert_eq!(wide_value.as_str(), r"w:\uD800");
    assert_ne!(wide_lookalike, wide_value);
}

// Requirements: UI-008, Section 12
//   Every bounded representation honors its character limit, preserves complete escape tokens, and marks omission with a scalar absent from full output
// Evidence: bounded_displays_cut_only_between_complete_tokens
#[test]
fn bounded_displays_cut_only_between_complete_tokens() {
    let identifiers = [
        RawIdentifier::Bytes(
            b"prefix\\\0middle\nwith\x80suffix"
                .to_vec()
                .into_boxed_slice(),
        ),
        RawIdentifier::Wtf16(
            [
                0x0050, 0x0052, 0x0045, 0x005c, 0, 0x202e, 0xd800, 0x004d, 0x0049, 0x0044, 0xdc00,
                0x0053, 0x0055, 0x0046,
            ]
            .into(),
        ),
    ];

    for identifier in identifiers {
        let full = identifier.full_display();
        assert!(!full.as_str().contains('…'));
        for characters in 3..full.as_str().chars().count() {
            let limit = DisplayLimit::new(characters).expect("three characters is valid");
            let bounded = identifier.bounded_display(limit);
            assert!(bounded.is_truncated());
            assert!(bounded.as_str().chars().count() <= characters);
            assert_bounded_tokens(bounded.as_str(), true);
        }

        let exact_limit = DisplayLimit::new(full.as_str().chars().count()).expect("full fits");
        let exact = identifier.bounded_display(exact_limit);
        assert!(!exact.is_truncated());
        assert_eq!(exact.as_str(), full.as_str());
        assert_bounded_tokens(exact.as_str(), false);
    }
}

// Requirements: UI-008, Section 12
//   Bounded presentation handles adversarially long input while returning only the configured small output rather than materializing the full escaped display
// Evidence: very_long_identifiers_have_strictly_bounded_visual_output
#[test]
fn very_long_identifiers_have_strictly_bounded_visual_output() {
    let bytes = RawIdentifier::Bytes(vec![0_u8; 1_000_000].into_boxed_slice());
    let wide = RawIdentifier::Wtf16(vec![0xd800_u16; 1_000_000].into_boxed_slice());
    let limit = DisplayLimit::new(64).expect("test limit is valid");

    for identifier in [&bytes, &wide] {
        let bounded = identifier.bounded_display(limit);
        assert!(bounded.is_truncated());
        assert!(bounded.as_str().chars().count() <= limit.get());
        assert_bounded_tokens(bounded.as_str(), true);
    }
}

// Requirements: UI-008, Section 12
//   A display limit too small for a domain prefix and unambiguous marker is rejected rather than silently exceeded
// Evidence: invalid_display_limits_fail_closed
#[test]
fn invalid_display_limits_fail_closed() {
    for value in 0..3 {
        let error = DisplayLimit::new(value).expect_err("short limits must fail");
        assert!(error.to_string().contains("at least 3"));
    }
    assert_eq!(DisplayLimit::new(3).expect("minimum is valid").get(), 3);
}
