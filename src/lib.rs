//! Secure, human-friendly identifiers with a fixed `LLD` rhythm.
//!
//! `tidyid` uses the operating system CSPRNG directly, rejects biased random
//! values, and never falls back to a predictable source.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod random;

use std::error::Error;
use std::fmt;

#[cfg(feature = "metrics")]
use num_bigint::BigUint;
use random::fill_structured_id;

/// Unambiguous lowercase letters used at letter positions.
pub const LETTERS: &str = "abcdefghjkmnpqrtuvwxyz";
/// Unambiguous digits used at every third position.
pub const DIGITS: &str = "23456789";
/// Unambiguous uppercase and lowercase letters used when uppercase is enabled.
pub const LETTERS_WITH_UPPERCASE: &str = "ABCDEFGHJKMNPQRTUVWXYZabcdefghjkmnpqrtuvwxyz";
/// Recommended default ID length.
pub const DEFAULT_LENGTH: usize = 32;
/// Smallest supported ID length.
pub const MIN_LENGTH: usize = 3;
/// Largest supported ID length.
pub const MAX_LENGTH: usize = 256;

const fn alphabet_lookup(alphabet: &[u8]) -> [bool; 256] {
    let mut lookup = [false; 256];
    let mut index = 0;
    while index < alphabet.len() {
        lookup[alphabet[index] as usize] = true;
        index += 1;
    }
    lookup
}

const LETTER_LOOKUP: [bool; 256] = alphabet_lookup(LETTERS.as_bytes());
const UPPERCASE_LETTER_LOOKUP: [bool; 256] = alphabet_lookup(LETTERS_WITH_UPPERCASE.as_bytes());
const DIGIT_LOOKUP: [bool; 256] = alphabet_lookup(DIGITS.as_bytes());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when an ID length is outside `3..=256`.
pub struct InvalidIdLengthError;

impl fmt::Display for InvalidIdLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("length must be between 3 and 256")
    }
}

impl Error for InvalidIdLengthError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when a value does not follow the TidyID format.
pub struct InvalidIdFormatError;

impl fmt::Display for InvalidIdFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("value is not a valid TidyID")
    }
}

impl Error for InvalidIdFormatError {}

#[derive(Debug)]
/// Error returned while generating an ID.
pub enum GenerateError<E> {
    /// The requested length is outside the supported range.
    InvalidLength(InvalidIdLengthError),
    /// The secure random source failed.
    Random(E),
}

impl<E: fmt::Display> fmt::Display for GenerateError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(error) => error.fmt(formatter),
            Self::Random(error) => write!(formatter, "secure random source failed: {error}"),
        }
    }
}

impl<E: Error + 'static> Error for GenerateError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLength(error) => Some(error),
            Self::Random(error) => Some(error),
        }
    }
}

impl<E> From<InvalidIdLengthError> for GenerateError<E> {
    fn from(error: InvalidIdLengthError) -> Self {
        Self::InvalidLength(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned by [`ensure_valid_id`].
pub enum ValidationError {
    /// The requested exact length is outside the supported range.
    InvalidLength(InvalidIdLengthError),
    /// The value does not follow the requested TidyID format.
    InvalidFormat(InvalidIdFormatError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(error) => error.fmt(formatter),
            Self::InvalidFormat(error) => error.fmt(formatter),
        }
    }
}

impl Error for ValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLength(error) => Some(error),
            Self::InvalidFormat(error) => Some(error),
        }
    }
}

#[inline]
const fn is_valid_length(length: usize) -> bool {
    length >= MIN_LENGTH && length <= MAX_LENGTH
}

#[inline]
fn ensure_valid_length(length: usize) -> Result<(), InvalidIdLengthError> {
    is_valid_length(length)
        .then_some(())
        .ok_or(InvalidIdLengthError)
}

/// Generates an ID using fresh bytes from the platform CSPRNG.
///
/// Set `allow_uppercase` to sample letter positions from
/// [`LETTERS_WITH_UPPERCASE`] instead of [`LETTERS`].
#[inline]
pub fn tidyid(
    length: usize,
    allow_uppercase: bool,
) -> Result<String, GenerateError<getrandom::Error>> {
    ensure_valid_length(length)?;
    let mut output = vec![0; length];
    let mut random = [0; MAX_LENGTH * 2];
    let random = &mut random[..length * 2];
    let mut output_offset = 0;
    while output_offset < length {
        getrandom::fill(random).map_err(GenerateError::Random)?;
        output_offset = fill_structured_id(&mut output, output_offset, random, allow_uppercase);
    }
    Ok(String::from_utf8(output).expect("TidyID contains only ASCII characters"))
}

/// Returns whether `value` follows the TidyID format.
///
/// When `length` is `Some`, the value must have exactly that valid length.
pub fn is_valid_id(value: &str, length: Option<usize>, allow_uppercase: bool) -> bool {
    if let Some(expected) = length {
        if !is_valid_length(expected) || value.len() != expected {
            return false;
        }
    }
    if !is_valid_length(value.len()) {
        return false;
    }

    value.bytes().enumerate().all(|(index, byte)| {
        let lookup = if (index + 1) % 3 == 0 {
            &DIGIT_LOOKUP
        } else if allow_uppercase {
            &UPPERCASE_LETTER_LOOKUP
        } else {
            &LETTER_LOOKUP
        };
        lookup[byte as usize]
    })
}

/// Validates a TidyID or returns an explicit length or format error.
pub fn ensure_valid_id(
    value: &str,
    length: Option<usize>,
    allow_uppercase: bool,
) -> Result<(), ValidationError> {
    if let Some(expected) = length {
        ensure_valid_length(expected).map_err(ValidationError::InvalidLength)?;
    }
    is_valid_id(value, length, allow_uppercase)
        .then_some(())
        .ok_or(ValidationError::InvalidFormat(InvalidIdFormatError))
}

#[cfg(feature = "metrics")]
/// Returns the exact number of possible IDs for a length and character mode.
pub fn get_id_capacity(
    length: usize,
    allow_uppercase: bool,
) -> Result<BigUint, InvalidIdLengthError> {
    ensure_valid_length(length)?;
    let digit_count = length / 3;
    let letter_count = length - digit_count;
    let letters = if allow_uppercase {
        LETTERS_WITH_UPPERCASE.len()
    } else {
        LETTERS.len()
    };
    Ok(BigUint::from(letters).pow(letter_count as u32)
        * BigUint::from(DIGITS.len()).pow(digit_count as u32))
}

#[cfg(feature = "metrics")]
/// Returns the entropy, in bits, for a length and character mode.
pub fn get_id_entropy(length: usize, allow_uppercase: bool) -> Result<f64, InvalidIdLengthError> {
    ensure_valid_length(length)?;
    let digit_count = length / 3;
    let letter_count = length - digit_count;
    let letters = if allow_uppercase {
        LETTERS_WITH_UPPERCASE.len()
    } else {
        LETTERS.len()
    };
    Ok(letter_count as f64 * (letters as f64).log2()
        + digit_count as f64 * (DIGITS.len() as f64).log2())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_the_reference_format() {
        assert_eq!(LETTERS, "abcdefghjkmnpqrtuvwxyz");
        assert_eq!(DIGITS, "23456789");
        assert_eq!(
            LETTERS_WITH_UPPERCASE,
            "ABCDEFGHJKMNPQRTUVWXYZabcdefghjkmnpqrtuvwxyz"
        );
    }

    #[test]
    fn generates_all_boundary_lengths_and_modes() {
        for length in [3, 8, 10, 16, 256] {
            for allow_uppercase in [false, true] {
                for _ in 0..25 {
                    let id = tidyid(length, allow_uppercase).unwrap();
                    assert_eq!(id.len(), length);
                    assert!(is_valid_id(&id, Some(length), allow_uppercase));
                }
            }
        }
    }

    #[test]
    fn validates_structure_and_exact_length() {
        assert!(is_valid_id("mk7qw2xy", None, false));
        assert!(is_valid_id("mk7qw2xy", Some(8), false));
        assert!(!is_valid_id("mk7qw2xy", Some(16), false));
        assert!(is_valid_id("mk7qw2x", None, false));
        assert!(!is_valid_id("mk7qw2x9", None, false));
        assert!(!is_valid_id("m27qw2xy", None, false));
        assert!(!is_valid_id("MK7QW2XY", None, false));
        assert!(is_valid_id("MK7QW2XY", Some(8), true));
        assert!(!is_valid_id("aI2", Some(3), true));
        assert!(!is_valid_id("aZ0", Some(3), true));
    }

    #[cfg(feature = "metrics")]
    #[test]
    fn reports_exact_capacity_and_entropy() {
        assert_eq!(
            get_id_capacity(8, false).unwrap(),
            BigUint::from(7_256_313_856_u64)
        );
        assert_eq!(
            get_id_capacity(16, false).unwrap(),
            BigUint::from(19_146_942_100_646_395_904_u128)
        );
        assert!((get_id_entropy(8, false).unwrap() - 32.756589711823786).abs() < 1e-12);
        assert!((get_id_entropy(16, false).unwrap() - 64.05374780501026).abs() < 1e-12);
        assert_eq!(get_id_capacity(3, true).unwrap(), BigUint::from(15_488_u32));
    }

    #[test]
    fn rejects_invalid_lengths() {
        for length in [0, 1, 2, 257, usize::MAX] {
            assert!(matches!(
                tidyid(length, false),
                Err(GenerateError::InvalidLength(_))
            ));
        }
    }

    #[test]
    fn generates_concurrently_without_shared_state() {
        let workers: Vec<_> = (0..16)
            .map(|worker| {
                std::thread::spawn(move || {
                    let length = MIN_LENGTH + worker % (MAX_LENGTH - MIN_LENGTH + 1);
                    let allow_uppercase = worker % 2 == 1;
                    for _ in 0..250 {
                        let id = tidyid(length, allow_uppercase).unwrap();
                        assert!(is_valid_id(&id, Some(length), allow_uppercase));
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    }
}
