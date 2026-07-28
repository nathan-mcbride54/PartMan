//! Strict validating decoder for the `pce/1` profile.
//!
//! Every rejection here is a security property, not tidiness. A decoder that
//! accepted a non-canonical encoding would let bytes that hash one way decode
//! to a plan that was authorized under a different hash.

use std::collections::BTreeMap;

use super::{Error, MAX_DEPTH, Value, compare_keys};

/// Decode canonical bytes into a value, rejecting anything non-canonical.
///
/// # Errors
///
/// Returns the [`Error`] naming the specific rule violated. The obligations are
/// enumerated in `schemas/canonical-encoding.md` §6: excluded constructs,
/// non-shortest arguments, out-of-range negatives, non-text or misordered map
/// keys, ill-formed UTF-8, lengths beyond the input, nesting past
/// [`MAX_DEPTH`], and trailing bytes.
pub fn decode(input: &[u8]) -> Result<Value, Error> {
    let mut reader = Reader { input, position: 0 };
    let value = reader.read_value(0)?;
    let remaining = reader.remaining();
    if remaining == 0 {
        Ok(value)
    } else {
        Err(Error::TrailingBytes(remaining))
    }
}

struct Reader<'a> {
    input: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn take(&mut self, count: usize) -> Result<&[u8], Error> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(Error::UnexpectedEnd)?;
        let slice = self
            .input
            .get(self.position..end)
            .ok_or(Error::UnexpectedEnd)?;
        self.position = end;
        Ok(slice)
    }

    fn take_byte(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn remaining(&self) -> usize {
        self.input.len() - self.position
    }

    /// Read a head for major types 0 to 6 and return `(major type, argument)`.
    ///
    /// Major type 7 never reaches here: its additional information selects a
    /// simple value or a float payload, neither of which is an integer
    /// argument, so applying the shortest-form rule of §2 to it would be
    /// meaningless.
    fn read_head(&mut self) -> Result<(u8, u64), Error> {
        let initial = self.take_byte()?;
        let major = initial >> 5;
        debug_assert!(major != 7, "major 7 is handled before read_head");
        let additional = initial & 0x1f;

        let argument = match additional {
            0..=23 => u64::from(additional),
            24 => {
                let value = u64::from(self.take_byte()?);
                if value < 24 {
                    return Err(Error::NonShortestArgument);
                }
                value
            }
            25 => {
                let bytes: [u8; 2] = self.take(2)?.try_into().expect("took exactly 2 bytes");
                let value = u64::from(u16::from_be_bytes(bytes));
                if value <= 0xff {
                    return Err(Error::NonShortestArgument);
                }
                value
            }
            26 => {
                let bytes: [u8; 4] = self.take(4)?.try_into().expect("took exactly 4 bytes");
                let value = u64::from(u32::from_be_bytes(bytes));
                if value <= 0xffff {
                    return Err(Error::NonShortestArgument);
                }
                value
            }
            27 => {
                let bytes: [u8; 8] = self.take(8)?.try_into().expect("took exactly 8 bytes");
                let value = u64::from_be_bytes(bytes);
                if value <= 0xffff_ffff {
                    return Err(Error::NonShortestArgument);
                }
                value
            }
            28..=30 => return Err(Error::ReservedAdditionalInformation(additional)),
            _ => return Err(Error::IndefiniteLength),
        };

        Ok((major, argument))
    }

    /// Convert a declared length to `usize` only after proving the bytes exist.
    ///
    /// Checking before allocating is what stops a hostile length header from
    /// requesting a large allocation for data that is not present. Every item
    /// this is used for occupies at least one byte, so the remaining byte count
    /// is a sound upper bound on an element or pair count as well as on a
    /// string length.
    fn checked_length(&self, declared: u64) -> Result<usize, Error> {
        let remaining = self.remaining();
        let available = u64::try_from(remaining).expect("usize fits in u64");
        if declared > available {
            return Err(Error::LengthExceedsInput {
                declared,
                remaining,
            });
        }
        usize::try_from(declared).map_err(|_| Error::LengthExceedsInput {
            declared,
            remaining,
        })
    }

    fn read_value(&mut self, depth: usize) -> Result<Value, Error> {
        if depth > MAX_DEPTH {
            return Err(Error::DepthLimitExceeded);
        }

        // Major type 7 is dispatched before read_head, see its documentation.
        let initial = *self.input.get(self.position).ok_or(Error::UnexpectedEnd)?;
        if initial >> 5 == 7 {
            self.position += 1;
            return self.read_simple_or_float(initial & 0x1f);
        }

        let (major, argument) = self.read_head()?;
        match major {
            0 => Ok(Value::Unsigned(argument)),
            1 => {
                let magnitude =
                    i64::try_from(argument).map_err(|_| Error::NegativeTooLarge(argument))?;
                // -1 - magnitude, exact even at magnitude == i64::MAX.
                Ok(Value::Negative(-1 - magnitude))
            }
            2 => {
                let length = self.checked_length(argument)?;
                Ok(Value::Bytes(self.take(length)?.to_vec()))
            }
            3 => Ok(Value::Text(self.read_text(argument)?)),
            4 => {
                let length = self.checked_length(argument)?;
                let mut items = Vec::with_capacity(length);
                for _ in 0..length {
                    items.push(self.read_value(depth + 1)?);
                }
                Ok(Value::Array(items))
            }
            5 => self.read_map(argument, depth),
            _ => Err(Error::TagNotAllowed(argument)),
        }
    }

    /// Handle major type 7: `false`, `true`, `null`, or a rejection.
    fn read_simple_or_float(&mut self, additional: u8) -> Result<Value, Error> {
        match additional {
            20 => Ok(Value::Bool(false)),
            21 => Ok(Value::Bool(true)),
            22 => Ok(Value::Null),
            // Half, single, and double precision. The payload is consumed so
            // that the reported error is the float, not a later desync.
            25 => {
                self.take(2)?;
                Err(Error::FloatNotAllowed)
            }
            26 => {
                self.take(4)?;
                Err(Error::FloatNotAllowed)
            }
            27 => {
                self.take(8)?;
                Err(Error::FloatNotAllowed)
            }
            24 => Err(Error::SimpleValueNotAllowed(self.take_byte()?)),
            28..=30 => Err(Error::ReservedAdditionalInformation(additional)),
            // 31 is the break stop code, which only appears with
            // indefinite-length items.
            31 => Err(Error::IndefiniteLength),
            other => Err(Error::SimpleValueNotAllowed(other)),
        }
    }

    fn read_text(&mut self, argument: u64) -> Result<String, Error> {
        let length = self.checked_length(argument)?;
        let bytes = self.take(length)?;
        core::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::InvalidUtf8)
    }

    fn read_map(&mut self, argument: u64, depth: usize) -> Result<Value, Error> {
        let declared = self.checked_length(argument)?;
        let mut entries = BTreeMap::new();
        let mut previous: Option<String> = None;

        for _ in 0..declared {
            let key_initial = *self.input.get(self.position).ok_or(Error::UnexpectedEnd)?;
            if key_initial >> 5 != 3 {
                return Err(Error::MapKeyNotText);
            }
            let (_, key_argument) = self.read_head()?;
            let key = self.read_text(key_argument)?;

            if let Some(previous) = &previous
                && compare_keys(previous, &key) != core::cmp::Ordering::Less
            {
                return Err(Error::MapKeyNotStrictlyIncreasing { key });
            }

            let value = self.read_value(depth + 1)?;
            previous = Some(key.clone());
            entries.insert(key, value);
        }

        Ok(Value::Map(entries))
    }
}
