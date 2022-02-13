use super::evaluation::{evaluate, Evaluation};
use super::validate_word::validate_word;
use anyhow::{bail, Result};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Guess {
    pub word: String,
    pub evaluation: Evaluation,
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

    pub fn guess(&mut self, guess: &str, word_list: &HashSet<String>) -> Result<()> {
        evaluate(guess, &self.solution, word_list)?;
        Ok(())
    }
}
