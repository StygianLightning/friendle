use crate::constants::WORD_LENGTH;
use anyhow::{bail, Result};
use std::collections::HashSet;

pub fn validate_word_format(word: &str) -> Result<()> {
    if word.len() != WORD_LENGTH {
        bail!("Expected word of length {WORD_LENGTH}, received {word}");
    }
    if !(word.is_ascii() && word.chars().all(|c| c.is_alphabetic())) {
        bail!("Only English words with letters A-Z are supported");
    }
    Ok(())
}

pub fn validate_word(word: &str, word_list: &HashSet<String>) -> Result<()> {
    validate_word_format(word)?;
    if !word_list.contains(word) {
        bail!("Invalid word {word}");
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
        assert!(validate_word(word, &word_list).is_err());
    }

    #[test]
    fn test_word_invalid_letter() {
        let word = "naïve";
        let word_list = HashSet::from_iter(std::iter::once(String::from(word)));
        assert!(validate_word_format(word).is_err());
        assert!(validate_word(word, &word_list).is_err());
    }

    #[test]
    fn test_partial_eval() {
        let word = "abbac";
        let word_list = HashSet::from_iter(std::iter::once(String::from(word)));
        assert!(validate_word_format(word).is_ok());
        assert!(validate_word(word, &word_list).is_ok());
    }
}
