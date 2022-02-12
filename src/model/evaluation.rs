use crate::constants::WORD_LENGTH;
use anyhow::bail;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Evaluation {
    Absent,
    Present,
    Correct,
}

pub fn evaluate(guess: &str, solution: &str) -> anyhow::Result<Vec<Evaluation>> {
    if guess.len() != WORD_LENGTH {
        bail!("Expected word of length {WORD_LENGTH}, received {guess}");
    }
    if !(guess.is_ascii() && guess.chars().all(|c| c.is_alphabetic())) {
        bail!("Only English words with letters A-Z are supported");
    }
    let chars_guess = guess.chars().collect::<Vec<_>>();
    let chars_solution = solution.chars().collect::<Vec<_>>();
    let mut evaluation = vec![Evaluation::Absent; WORD_LENGTH as usize];
    let mut solution_frequencies = letter_frequencies(&chars_solution);

    for i in 0..WORD_LENGTH {
        if chars_guess[i] == chars_solution[i] {
            evaluation[i] = Evaluation::Correct;
            *solution_frequencies.get_mut(&chars_guess[i]).unwrap() -= 1;
        }
    }
    for i in 0..WORD_LENGTH {
        if chars_guess[i] != chars_solution[i] {
            if let Some(val) = solution_frequencies.get_mut(&chars_guess[i]) {
                if *val > 0 {
                    evaluation[i] = Evaluation::Present;
                    *val -= 1;
                }
            }
        }
    }

    Ok(evaluation)
}

fn letter_frequencies(chars: &[char]) -> HashMap<char, usize> {
    let mut freq = HashMap::new();
    for c in chars {
        *freq.entry(*c).or_default() += 1;
    }
    freq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_eval() {
        let word = "tales";
        let eval = evaluate(word, word).unwrap();
        assert_eq!(eval, vec![Evaluation::Correct; WORD_LENGTH]);
    }

    #[test]
    fn test_word_invalid_len() {
        let word = "abcdef";
        let solution = "abcde";
        assert!(evaluate(word, solution).is_err());
    }

    #[test]
    fn test_word_invalid_letter() {
        let word = "naïve";
        let solution = "abcde";
        assert!(evaluate(word, solution).is_err());
    }

    #[test]
    fn test_partial_eval() {
        let word = "abbac";
        let solution = "bdaab";
        let eval = evaluate(word, solution).unwrap();
        assert_eq!(
            &eval,
            &[
                Evaluation::Present,
                Evaluation::Present,
                Evaluation::Present,
                Evaluation::Correct,
                Evaluation::Absent,
            ]
        );
    }

    #[test]
    fn test_letter_frequencies() {
        let word = "abbac";
        let freq = letter_frequencies(&word.chars().collect::<Vec<_>>());
        let mut frequencies_vec = freq.into_iter().collect::<Vec<_>>();
        frequencies_vec.sort_by(|(a, _), (b, _)| a.cmp(b));
        assert_eq!(frequencies_vec, vec![('a', 2usize), ('b', 2), ('c', 1)]);
    }
}
