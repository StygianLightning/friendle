use crate::constants::WORD_LENGTH;
use anyhow::anyhow;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Evaluation {
    Absent,
    Present,
    Correct,
}

pub fn evaluate(guess: &str, solution: &str) -> anyhow::Result<Vec<Evaluation>> {
    if guess.len() != WORD_LENGTH {
        anyhow!("Expected word of length {WORD_LENGTH}, received {guess}");
    }
    if !(guess.is_ascii() && guess.chars().all(|c| c.is_alphabetic())) {
        anyhow!("Only English words with letters A-Z are supported");
    }
    let chars_guess = guess.chars().collect::<Vec<_>>();
    let chars_solution = guess.chars().collect::<Vec<_>>();
    let mut evaluation = vec![Evaluation::Absent; WORD_LENGTH as usize];

    for i in 0..WORD_LENGTH {
        if chars_guess[i] == chars_solution[i] {
            evaluation[i] = Evaluation::Correct;
        }
    }

    Ok(evaluation)
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
}
