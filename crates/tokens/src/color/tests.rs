//! Tests for the colour maths.
//!
//! The contrast tests are anchored to values WCAG itself publishes rather than
//! to numbers this implementation produced: black on white is exactly `21:1`,
//! and any colour against itself is exactly `1:1`. A ratio checked against a
//! figure the same code emitted would prove only that the code is deterministic.

use super::{Deficiency, Srgb, contrast_ratio, delta_e_76, simulate};

const BLACK: Srgb = Srgb { r: 0, g: 0, b: 0 };
const WHITE: Srgb = Srgb {
    r: 255,
    g: 255,
    b: 255,
};
const RED: Srgb = Srgb { r: 255, g: 0, b: 0 };
const GREEN: Srgb = Srgb { r: 0, g: 255, b: 0 };
const BLUE: Srgb = Srgb { r: 0, g: 0, b: 255 };

fn parse(text: &str) -> Srgb {
    Srgb::parse(text).expect("test colour parses")
}

#[test]
fn parsing_accepts_six_digit_hexadecimal_in_either_case() {
    assert_eq!(parse("#000000"), BLACK);
    assert_eq!(parse("#FFFFFF"), WHITE);
    assert_eq!(parse("#ffffff"), WHITE);
    assert_eq!(
        parse("#7FB3FF"),
        Srgb {
            r: 127,
            g: 179,
            b: 255
        }
    );
}

#[test]
fn parsing_refuses_every_other_spelling() {
    // Each of these is a plausible colour string that a strict reader must
    // reject rather than interpret, because interpreting it silently would
    // produce a contrast figure for a colour nobody wrote.
    for rejected in [
        "",
        "#",
        "#FFF",
        "#FFFFFFF",
        "FFFFFF",
        "#GGGGGG",
        "#FFFF FF",
        "rgb(0,0,0)",
        "white",
        "#-00000",
    ] {
        assert!(
            Srgb::parse(rejected).is_err(),
            "{rejected:?} should not parse"
        );
    }
}

#[test]
fn black_on_white_is_the_published_maximum_of_twenty_one_to_one() {
    // WCAG's own stated bound. If the transfer function or the luminance
    // coefficients were wrong, this is the first thing that would move.
    let ratio = contrast_ratio(BLACK, WHITE);
    assert!(
        (ratio - 21.0).abs() < 1e-9,
        "black on white should be exactly 21:1, computed {ratio}"
    );
}

#[test]
fn contrast_is_symmetric_and_bottoms_out_at_one() {
    assert!((contrast_ratio(WHITE, BLACK) - contrast_ratio(BLACK, WHITE)).abs() < 1e-12);
    for color in [BLACK, WHITE, RED, GREEN, BLUE, parse("#16181C")] {
        let ratio = contrast_ratio(color, color);
        assert!(
            (ratio - 1.0).abs() < 1e-12,
            "a colour against itself is 1:1, computed {ratio}"
        );
    }
}

#[test]
fn luminance_is_ordered_by_lightness() {
    // A weaker property than the exact figures, but one that fails loudly if
    // the transfer function is ever inverted.
    let ordered = [
        BLACK,
        parse("#16181C"),
        parse("#6E7684"),
        parse("#B6BDC8"),
        WHITE,
    ];
    for pair in ordered.windows(2) {
        assert!(
            pair[0].relative_luminance() < pair[1].relative_luminance(),
            "{:?} should be darker than {:?}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn the_green_channel_dominates_luminance() {
    // The 0.2126 / 0.7152 / 0.0722 weighting is the whole reason a mid green
    // reads as lighter than a mid blue at equal channel value. If the
    // coefficients were transposed this ordering would break.
    assert!(GREEN.relative_luminance() > RED.relative_luminance());
    assert!(RED.relative_luminance() > BLUE.relative_luminance());
}

#[test]
fn simulation_leaves_greys_untouched() {
    // Every deficiency matrix must map the achromatic axis onto itself: a
    // colour with no chroma has nothing to lose. A matrix typed with a wrong
    // row sum shifts greys, so this catches a whole class of transcription
    // error without needing to trust any individual digit.
    for deficiency in Deficiency::ALL {
        for grey in ["#000000", "#404040", "#808080", "#C0C0C0", "#FFFFFF"] {
            let source = parse(grey);
            let seen = simulate(source, deficiency);
            let drift = delta_e_76(source, seen);
            assert!(
                drift < 2.0,
                "{} shifted grey {grey} by delta-E {drift:.2}",
                deficiency.name()
            );
        }
    }
}

#[test]
fn red_and_green_converge_for_protanopia_and_deuteranopia() {
    // The defining property of both deficiencies, and the reason the matrices
    // are here at all. Asserting the property rather than the coefficients
    // means a mistyped digit surfaces as a pair that failed to converge, not as
    // a plausible number with nothing to check it against.
    let apart = delta_e_76(RED, GREEN);
    for deficiency in [Deficiency::Protanopia, Deficiency::Deuteranopia] {
        let together = delta_e_76(simulate(RED, deficiency), simulate(GREEN, deficiency));
        assert!(
            together < apart / 2.0,
            "{} should collapse red against green: {apart:.1} became {together:.1}",
            deficiency.name()
        );
    }
}

#[test]
fn tritanopia_collapses_blue_without_collapsing_red_against_green() {
    // The complementary property: tritanopia is a blue-yellow deficiency, so it
    // must *not* behave like the other two. Without this, a matrix accidentally
    // copied from the deuteranopia row would still pass the test above.
    let blue_yellow_apart = delta_e_76(BLUE, parse("#FFFF00"));
    let blue_yellow_together = delta_e_76(
        simulate(BLUE, Deficiency::Tritanopia),
        simulate(parse("#FFFF00"), Deficiency::Tritanopia),
    );
    assert!(
        blue_yellow_together < blue_yellow_apart,
        "tritanopia should bring blue and yellow together"
    );

    let red_green_together = delta_e_76(
        simulate(RED, Deficiency::Tritanopia),
        simulate(GREEN, Deficiency::Tritanopia),
    );
    assert!(
        red_green_together > delta_e_76(RED, GREEN) / 2.0,
        "tritanopia should leave red and green largely apart, unlike protan/deutan"
    );
}

#[test]
fn delta_e_is_zero_for_identity_and_symmetric() {
    for color in [BLACK, WHITE, RED, parse("#7BD5A0")] {
        assert!(delta_e_76(color, color) < 1e-9);
    }
    assert!((delta_e_76(RED, BLUE) - delta_e_76(BLUE, RED)).abs() < 1e-9);
}

#[test]
fn delta_e_separates_black_from_white_by_the_full_lightness_range() {
    // CIELAB lightness runs 0..=100, so black against white is 100 by
    // construction. Another externally fixed anchor rather than a recorded run.
    let difference = delta_e_76(BLACK, WHITE);
    assert!(
        (difference - 100.0).abs() < 0.5,
        "black against white should be delta-E 100, computed {difference:.3}"
    );
}
