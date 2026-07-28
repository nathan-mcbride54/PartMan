//! Canonical encoder for the `pce/1` profile.

use super::{Error, Value, compare_keys};

/// Encode a value into its one canonical byte string.
///
/// # Errors
///
/// Returns [`Error::NegativeOutOfRange`] if a [`Value::Negative`] holds a
/// non-negative number. Every other value in the model is encodable, because
/// the model cannot represent anything this profile excludes.
pub fn encode(value: &Value) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    write_value(&mut out, value)?;
    Ok(out)
}

fn write_value(out: &mut Vec<u8>, value: &Value) -> Result<(), Error> {
    match value {
        Value::Unsigned(number) => write_head(out, 0, *number),
        Value::Negative(number) => {
            if *number >= 0 {
                return Err(Error::NegativeOutOfRange(*number));
            }
            // -1 - n, computed without overflow at i64::MIN.
            let argument = number.unsigned_abs() - 1;
            write_head(out, 1, argument);
        }
        Value::Bytes(bytes) => {
            write_head(out, 2, length_argument(bytes.len()));
            out.extend_from_slice(bytes);
        }
        Value::Text(text) => {
            write_head(out, 3, length_argument(text.len()));
            out.extend_from_slice(text.as_bytes());
        }
        Value::Array(items) => {
            write_head(out, 4, length_argument(items.len()));
            for item in items {
                write_value(out, item)?;
            }
        }
        Value::Map(entries) => {
            write_head(out, 5, length_argument(entries.len()));
            // BTreeMap iterates in bytewise key order, which is not this
            // profile's length-first order, so the keys are sorted here.
            let mut keys = entries.keys().collect::<Vec<_>>();
            keys.sort_by(|left, right| compare_keys(left, right));
            for key in keys {
                write_head(out, 3, length_argument(key.len()));
                out.extend_from_slice(key.as_bytes());
                write_value(out, &entries[key])?;
            }
        }
        Value::Bool(true) => out.push(0xf5),
        Value::Bool(false) => out.push(0xf4),
        Value::Null => out.push(0xf6),
    }
    Ok(())
}

/// Widen a container length to the argument type.
///
/// `usize` is at most 64 bits on every platform this project supports, so this
/// is lossless. It is written as a `From` conversion rather than a cast so that
/// a hypothetical 128-bit target would fail to compile instead of truncating.
fn length_argument(length: usize) -> u64 {
    u64::try_from(length).expect("usize fits in u64 on all supported targets")
}

/// Write `major << 5 | additional`, followed by the shortest argument encoding.
fn write_head(out: &mut Vec<u8>, major: u8, argument: u64) {
    let major_bits = major << 5;
    let bytes = argument.to_be_bytes();
    match argument {
        0..=0x17 => out.push(major_bits | bytes[7]),
        0x18..=0xff => {
            out.push(major_bits | 0x18);
            out.extend_from_slice(&bytes[7..]);
        }
        0x100..=0xffff => {
            out.push(major_bits | 0x19);
            out.extend_from_slice(&bytes[6..]);
        }
        0x1_0000..=0xffff_ffff => {
            out.push(major_bits | 0x1a);
            out.extend_from_slice(&bytes[4..]);
        }
        _ => {
            out.push(major_bits | 0x1b);
            out.extend_from_slice(&bytes);
        }
    }
}
