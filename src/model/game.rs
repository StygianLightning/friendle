use super::coding::Code;
use super::evaluation::{evaluate, get_emoji, Evaluation};
use super::validate_word::validate_word_format;
use crate::constants::MAX_GUESSES;
use crate::util::get_regional_indicator_emoji_with_zero_width_space;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Guess {
    pub word: String,
    pub evaluation: Vec<Evaluation>,
}

impl Guess {
    pub fn get_letter_state(&self, letter: char) -> LetterState {
        self.word
            .chars()
            .enumerate()
            .filter_map(|(i, c)| {
                if c == letter {
                    Some(LetterState::from(self.evaluation[i]))
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(LetterState::Unknown)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameFlag {
    SolutionNotInWordList,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GameFlags(HashSet<GameFlag>);

impl Deref for GameFlags {
    type Target = HashSet<GameFlag>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    code: Code,
    flags: GameFlags,
    solution: String,
    state: GameState,
    history: Vec<Guess>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum LetterState {
    Unknown,
    Absent,
    Present,
    Correct,
}

impl From<Evaluation> for LetterState {
    fn from(eval: Evaluation) -> Self {
        match eval {
            Evaluation::Absent => LetterState::Absent,
            Evaluation::Present => LetterState::Present,
            Evaluation::Correct => LetterState::Correct,
        }
    }
}

impl LetterState {
    pub fn to_evaluation(self) -> Option<Evaluation> {
        match self {
            LetterState::Unknown => None,
            LetterState::Absent => Some(Evaluation::Absent),
            LetterState::Present => Some(Evaluation::Present),
            LetterState::Correct => Some(Evaluation::Correct),
        }
    }
}

impl Game {
    pub fn new(code: Code, solution: String, word_list: &HashSet<String>) -> Result<Self> {
        // TODO add word list and check if solution is in word list; if not, add a flag here and a warning to each in-progress state and the initial message!
        validate_word_format(&solution)?;
        let mut flags = GameFlags::default();

        if !word_list.contains(&solution) {
            flags.0.insert(GameFlag::SolutionNotInWordList);
        }

        Ok(Self {
            code,
            solution,
            history: vec![],
            state: GameState::InProgress,
            flags,
        })
    }

    pub fn flags(&self) -> &GameFlags {
        &self.flags
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
                        .map(get_regional_indicator_emoji_with_zero_width_space),
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

    pub fn get_letter_state(&self, letter: char) -> LetterState {
        let letter = letter.to_ascii_lowercase();
        self.history
            .iter()
            .map(|guess| guess.get_letter_state(letter))
            .max()
            .unwrap_or(LetterState::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win() {
        let word = String::from("tales");
        let word_list = HashSet::from_iter(std::iter::once(word.clone()));
        let mut game = Game::new(Code { value: 1234 }, word.clone(), &word_list).unwrap(); // not the real code, but it doesn't matter here since it's only used for reporting
        game.guess(word.clone(), &word_list).unwrap();
        assert_eq!(game.state, GameState::Won);
        assert!(game.guess(word.clone(), &word_list).is_err());
    }

    #[test]
    fn test_guess_get_letter_state() {
        let guess = Guess {
            word: String::from("abcbc"),
            evaluation: vec![
                Evaluation::Absent,
                Evaluation::Absent,
                Evaluation::Absent,
                Evaluation::Present,
                Evaluation::Correct,
            ],
        };

        assert_eq!(LetterState::Absent, guess.get_letter_state('a'));
        assert_eq!(LetterState::Present, guess.get_letter_state('b'));
        assert_eq!(LetterState::Correct, guess.get_letter_state('c'));
        assert_eq!(LetterState::Unknown, guess.get_letter_state('d'));
    }

    #[test]
    fn test_game_get_letter_state() {
        let solution = String::from("tales");

        let mut word_list = HashSet::new();
        word_list.insert(String::from("earth"));
        word_list.insert(String::from("value"));
        word_list.insert(String::from("slime"));

        let mut game = Game::new(Code { value: 1234 }, solution.clone(), &word_list).unwrap();
        for word in &word_list {
            game.guess(word.clone(), &word_list).unwrap();
        }

        assert!(game.flags().contains(&GameFlag::SolutionNotInWordList));

        assert_eq!(LetterState::Present, game.get_letter_state('t'));
        assert_eq!(LetterState::Correct, game.get_letter_state('a'));
        assert_eq!(LetterState::Correct, game.get_letter_state('l'));
        assert_eq!(LetterState::Present, game.get_letter_state('e'));
        assert_eq!(LetterState::Present, game.get_letter_state('s'));

        assert_eq!(LetterState::Absent, game.get_letter_state('r'));
        assert_eq!(LetterState::Absent, game.get_letter_state('h'));
        assert_eq!(LetterState::Absent, game.get_letter_state('v'));
        assert_eq!(LetterState::Absent, game.get_letter_state('u'));

        assert_eq!(LetterState::Absent, game.get_letter_state('m'));
        assert_eq!(LetterState::Absent, game.get_letter_state('i'));

        assert_eq!(LetterState::Unknown, game.get_letter_state('w'));
        assert_eq!(LetterState::Unknown, game.get_letter_state('x'));
        assert_eq!(LetterState::Unknown, game.get_letter_state('y'));
        assert_eq!(LetterState::Unknown, game.get_letter_state('z'));
    }
}
