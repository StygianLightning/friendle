use super::coding::Code;
use super::evaluation::{evaluate, get_emoji, Evaluation};
use super::validate_word::validate_word_format;
use crate::constants::MAX_GUESSES;
use crate::util::get_regional_indicator;
use anyhow::{bail, Result};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Guess {
    pub word: String,
    pub evaluation: Vec<Evaluation>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug, Clone)]
pub struct Game {
    code: Code,
    solution: String,
    state: GameState,
    history: Vec<Guess>,
}

impl Game {
    pub fn new(code: Code, solution: String) -> Result<Self> {
        // TODO add word list and check if solution is in word list; if not, add a flag here and a warning to each in-progress state and the initial message!
        validate_word_format(&solution)?;
        Ok(Self {
            code,
            solution,
            history: vec![],
            state: GameState::InProgress,
        })
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn solution(&self) -> &str {
        &self.solution
    }

    pub fn history(&self) -> &[Guess] {
        &self.history
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn guess(&mut self, guess: String, word_list: &HashSet<String>) -> Result<()> {
        if self.state != GameState::InProgress {
            bail!("Game is already finished.");
        }
        let evaluation = evaluate(&guess, &self.solution, word_list)?;
        if evaluation.iter().all(|eval| *eval == Evaluation::Correct) {
            self.state = GameState::Won;
        }
        self.history.push(Guess {
            word: guess,
            evaluation,
        });
        if self.state != GameState::Won && self.history.len() == MAX_GUESSES {
            self.state = GameState::Lost;
        }
        Ok(())
    }

    pub fn display_state(&self, message_builder: &mut serenity::utils::MessageBuilder) {
        for guess in &self.history {
            if self.state == GameState::InProgress {
                // guessed word converted to emojis
                message_builder.push_line(String::from_iter(
                    guess
                        .word
                        .chars()
                        // add a zero-width space unicode character after each emoji to prevent Serenity from merging successive emojis.
                        .map(|c| format!("{}\u{200c}", get_regional_indicator(c))),
                ));
            }
            // evaluation converted to emojis
            message_builder.push_line_safe(String::from_iter(
                guess
                    .evaluation
                    .iter()
                    .map(|eval| format!("{}", get_emoji(*eval))),
            ));
            if self.state == GameState::InProgress {
                message_builder.push_line_safe("");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win() {
        let word = String::from("tales");
        let word_list = HashSet::from_iter(std::iter::once(word.clone()));
        let mut game = Game::new(Code { value: 1234 }, word.clone()).unwrap(); // not the real code, but it doesn't matter here since it's only used for reporting
        game.guess(word.clone(), &word_list).unwrap();
        assert_eq!(game.state, GameState::Won);
        assert!(game.guess(word.clone(), &word_list).is_err());
    }
}
