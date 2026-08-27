use crate::{DIGITS, LETTERS, LETTERS_WITH_UPPERCASE, MAX_LENGTH};

const fn letter_byte_codes(alphabet: &[u8]) -> [u8; 256] {
    let mut lookup = [0; 256];
    let accept_limit = 256 - 256 % alphabet.len();
    let mut byte = 0;
    while byte < accept_limit {
        lookup[byte] = alphabet[byte % alphabet.len()];
        byte += 1;
    }
    lookup
}

const LOWERCASE_CODES: [u8; 256] = letter_byte_codes(LETTERS.as_bytes());
const UPPERCASE_CODES: [u8; 256] = letter_byte_codes(LETTERS_WITH_UPPERCASE.as_bytes());

#[inline]
pub(crate) fn fill_structured_id(
    output: &mut [u8],
    mut output_offset: usize,
    random: &[u8],
    allow_uppercase: bool,
) -> usize {
    debug_assert!(output.len() <= MAX_LENGTH);
    debug_assert!(output_offset <= output.len());
    let letter_codes = if allow_uppercase {
        &UPPERCASE_CODES
    } else {
        &LOWERCASE_CODES
    };

    for &byte in random {
        if output_offset == output.len() {
            break;
        }
        if (output_offset + 1) % 3 == 0 {
            output[output_offset] = DIGITS.as_bytes()[(byte & 7) as usize];
        } else {
            let code = letter_codes[byte as usize];
            if code == 0 {
                continue;
            }
            output[output_offset] = code;
        }
        output_offset += 1;
    }
    output_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_ranges_are_exact_and_uniform() {
        assert_eq!(256 % DIGITS.len(), 0);

        for (alphabet, allow_uppercase) in [
            (LETTERS.as_bytes(), false),
            (LETTERS_WITH_UPPERCASE.as_bytes(), true),
        ] {
            let limit = 256 - 256 % alphabet.len();
            let mut counts = vec![0; alphabet.len()];
            for input in 0..=u8::MAX {
                let mut output = [0];
                let written = fill_structured_id(&mut output, 0, &[input], allow_uppercase);
                if usize::from(input) >= limit {
                    assert_eq!(written, 0);
                    continue;
                }
                assert_eq!(written, 1);
                let index = alphabet
                    .iter()
                    .position(|candidate| *candidate == output[0])
                    .unwrap();
                counts[index] += 1;
            }
            assert!(counts.iter().all(|count| *count == limit / alphabet.len()));
        }

        let mut counts = [0; 8];
        for input in 0..=u8::MAX {
            let mut output = [0; 3];
            assert_eq!(fill_structured_id(&mut output, 2, &[input], false), 3);
            counts[(input & 7) as usize] += 1;
        }
        assert!(counts.iter().all(|count| *count == 32));
    }

    #[test]
    fn rejection_sampling_continues_with_the_next_block() {
        let mut output = [0; 3];
        let written = fill_structured_id(&mut output, 0, &[255; 6], false);
        assert_eq!(written, 0);
        let written = fill_structured_id(&mut output, written, &[0; 6], false);
        assert_eq!(written, output.len());
        assert_eq!(&output, b"aa2");
    }

    #[test]
    fn uppercase_sampling_preserves_the_lld_structure() {
        let mut output = [0; 3];
        assert_eq!(fill_structured_id(&mut output, 0, &[0, 22, 0], true), 3);
        assert_eq!(&output, b"Aa2");
    }
}
