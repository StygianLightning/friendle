use super::evaluation::{evaluate, Evaluation};
use super::validate_word::validate_word;
use crate::constants::MAX_GUESSES;
use anyhow::{bail, Result};
use std::collections::HashSet;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Game {
    solution: String,
    state: GameState,
    history: Vec<Guess>,
}

impl Game {
    pub fn new(solution: String, word_list: &HashSet<String>) -> Result<Self> {
        validate_word(&solution, word_list)?;
        Ok(Self {
            solution,
            history: vec![],
            state: GameState::InProgress,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win() {
        let word = String::from("tales");
        let word_list = HashSet::from_iter(std::iter::once(word.clone()));
        let mut game = Game::new(word.clone(), &word_list).unwrap();
        game.guess(word.clone(), &word_list).unwrap();
        assert_eq!(game.state, GameState::Won);
        assert!(game.guess(word.clone(), &word_list).is_err());
    }
}