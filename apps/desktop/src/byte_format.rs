//! Lossless byte-count presentation for the native desktop view model.
//!
//! Display values use a deterministic English IEC spelling with one decimal
//! place. Exact values retain every `u64` digit. Neither representation is an
//! identity or an input to storage decisions.

const IEC_UNITS: &[(u64, &str)] = &[
    (1, "B"),
    (1 << 10, "KiB"),
    (1 << 20, "MiB"),
    (1 << 30, "GiB"),
    (1 << 40, "TiB"),
    (1 << 50, "PiB"),
    (1 << 60, "EiB"),
];

/// Human-readable and exact presentations of one unsigned byte count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattedBytes {
    /// Rounded, human-readable IEC value.
    pub display: String,
    /// Fully grouped decimal byte count with a `B` suffix.
    pub exact: String,
}

/// Format an unsigned byte count as a deterministic English IEC value.
///
/// The calculation uses integer arithmetic, rounds half up to one decimal
/// place, and promotes values that would otherwise render as `1024.0` of the
/// smaller unit.
#[must_use]
pub fn format_iec_bytes(bytes: u64) -> String {
    if bytes < IEC_UNITS[1].0 {
        return format!("{bytes} B");
    }

    let mut unit_index = 1;
    while unit_index + 1 < IEC_UNITS.len() && bytes >= IEC_UNITS[unit_index + 1].0 {
        unit_index += 1;
    }

    let mut tenths = rounded_tenths(bytes, IEC_UNITS[unit_index].0);
    if tenths >= 10_240 && unit_index + 1 < IEC_UNITS.len() {
        unit_index += 1;
        tenths = rounded_tenths(bytes, IEC_UNITS[unit_index].0);
    }

    format!(
        "{}.{:01} {}",
        tenths / 10,
        tenths % 10,
        IEC_UNITS[unit_index].1
    )
}

/// Format every decimal digit of an unsigned byte count with English grouping.
#[must_use]
pub fn format_exact_bytes(bytes: u64) -> String {
    let digits = bytes.to_string();
    let separator_count = digits.len().saturating_sub(1) / 3;
    let mut grouped = String::with_capacity(digits.len() + separator_count + 2);
    for (index, digit) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(char::from(digit));
    }
    grouped.push_str(" B");
    grouped
}

/// Produce both supported presentations from the same exact input.
#[must_use]
pub fn format_bytes(bytes: u64) -> FormattedBytes {
    FormattedBytes {
        display: format_iec_bytes(bytes),
        exact: format_exact_bytes(bytes),
    }
}

fn rounded_tenths(bytes: u64, unit_bytes: u64) -> u128 {
    (u128::from(bytes) * 10 + u128::from(unit_bytes) / 2) / u128::from(unit_bytes)
}

#[cfg(test)]
mod tests {
    use super::{FormattedBytes, format_bytes, format_exact_bytes, format_iec_bytes};

    // Requirements: MODEL-001, UI-013
    //   Human-readable sizes are derived with integer arithmetic while exact
    //   `u64` bytes remain present and unabridged beside them.
    // Work-Package: WP-030
    // Evidence: byte_formatting_is_integer_exact_and_boundary_canonical
    #[test]
    fn byte_formatting_is_integer_exact_and_boundary_canonical() {
        assert_eq!(format_iec_bytes(0), "0 B");
        assert_eq!(format_iec_bytes(1_023), "1023 B");
        assert_eq!(format_iec_bytes(1_024), "1.0 KiB");
        assert_eq!(format_iec_bytes(1_073_741_823), "1.0 GiB");
        assert_eq!(format_iec_bytes(1_000_204_886_016), "931.5 GiB");
        assert_eq!(format_iec_bytes(u64::MAX), "16.0 EiB");

        assert_eq!(format_exact_bytes(0), "0 B");
        assert_eq!(format_exact_bytes(1_000_204_886_016), "1,000,204,886,016 B");
        assert_eq!(format_exact_bytes(u64::MAX), "18,446,744,073,709,551,615 B");
        assert_eq!(
            format_bytes(4_096),
            FormattedBytes {
                display: "4.0 KiB".to_owned(),
                exact: "4,096 B".to_owned(),
            }
        );
    }
}
