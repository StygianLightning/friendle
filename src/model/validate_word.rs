use crate::constants::WORD_LENGTH;
use std::collections::HashSet;

use super::guess_error::{GuessError, InvalidWordError};

pub fn validate_word_format(word: &str) -> Result<(), GuessError> {
    if word.len() != WORD_LENGTH {
        return Err(GuessError::InvalidWord(InvalidWordError::WrongLength {
            expected_length: WORD_LENGTH,
            given_length: word.len(),
        }));
    }
    if !(word.is_ascii() && word.chars().all(|c| c.is_alphabetic())) {
        return Err(GuessError::InvalidWord(InvalidWordError::NonLatinAlpha));
    }
    Ok(())
}

pub fn validate_word(
    word: &str,
    word_list: &HashSet<String>,
    solution: &str,
) -> Result<(), GuessError> {
    validate_word_format(word)?;
    if word != solution && !word_list.contains(word) {
        Err(InvalidWordError::NotInWordList {
            word: String::from(word),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_invalid_len() {
        let word = "abcdef";
        let word_list = HashSet::from_iter(std::iter::once(String::from(word)));
        assert!(validate_word_format(word).is_err());
        assert!(validate_word(word, &word_list, "abbac").is_err());
    }

    #[test]
    fn test_word_invalid_letter() {
        let word = "naïve";
        let word_list = HashSet::from_iter(std::iter::once(String::from(word)));
        assert!(validate_word_format(word).is_err());
        assert!(validate_word(word, &word_list, word).is_err());
    }

    #[test]
    fn test_word_valid_when_solution_and_format_valid() {
        let word = "abcde";
        let word_list = HashSet::from_iter(std::iter::once(String::from("tales")));
        assert!(validate_word_format(word).is_ok());
        assert!(validate_word(word, &word_list, word).is_ok());
    }

    #[test]
    fn test_partial_eval() {
        let word = "abbac";
        let word_list = HashSet::from_iter(std::iter::once(String::from(word)));
        assert!(validate_word_format(word).is_ok());
        assert!(validate_word(word, &word_list, "abcdef").is_ok());
    }
}
