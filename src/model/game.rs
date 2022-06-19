use super::coding::Code;
use super::evaluation::{evaluate, get_emoji, EmojiMode, Evaluation};
use super::guess_error::GuessError;
use super::knowledge::Knowledge;
use super::validate_word::validate_word_format;
use crate::constants::{self, MAX_GUESSES};
use crate::util::get_regional_indicator_emoji_with_zero_width_space;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum StrictMode {
    Enabled,
    Disabled,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ModeChangeError {
    AlreadySet,
    TooManyGuessesAlready,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameFlag {
    SolutionNotInWordList,
    StrictModeEnabled,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GameFlags(HashSet<GameFlag>);

impl Deref for GameFlags {
    type Target = HashSet<GameFlag>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GameFlags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    code: Code,
    flags: GameFlags,
    solution: String,
    state: GameState,
    history: Vec<Guess>,
    knowledge: Knowledge,
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
    pub fn new(
        code: Code,
        solution: String,
        word_list: &HashSet<String>,
    ) -> Result<Self, GuessError> {
        let word_length = solution.len();
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
            knowledge: Knowledge::new(word_length),
        })
    }

    pub fn flags(&self) -> &GameFlags {
        &self.flags
    }

    pub fn flags_mut(&mut self) -> &mut GameFlags {
        &mut self.flags
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

    pub fn can_switch_to_mode(&self, mode: StrictMode) -> Result<(), ModeChangeError> {
        if mode == self.get_strict_mode() {
            return Err(ModeChangeError::AlreadySet);
        }

        match self.get_strict_mode() {
            // we're in strict mode and switching to non-strict mode, which is always possible
            StrictMode::Enabled => Ok(()),
            StrictMode::Disabled => {
                // we want to switch to strict mode, which is only possible until we're one guess in.
                if self.history.len() <= 1 {
                    Ok(())
                } else {
                    Err(ModeChangeError::TooManyGuessesAlready)
                }
            }
        }
    }

    pub fn get_strict_mode(&self) -> StrictMode {
        if self.flags.contains(&GameFlag::StrictModeEnabled) {
            StrictMode::Enabled
        } else {
            StrictMode::Disabled
        }
    }

    pub fn set_strict_mode(&mut self, mode: StrictMode) -> Result<(), ModeChangeError> {
        self.can_switch_to_mode(mode)?;
        match mode {
            StrictMode::Disabled => {
                self.flags.remove(&GameFlag::StrictModeEnabled);
            }
            StrictMode::Enabled => {
                // can only switch to strict mode if there has been at most one guess.
                self.flags.insert(GameFlag::StrictModeEnabled);
            }
        }
        Ok(())
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn guess(
        &mut self,
        guessed_word: String,
        word_list: &HashSet<String>,
    ) -> Result<(), GuessError> {
        if self.state != GameState::InProgress {
            return Err(GuessError::GameNotInProgress);
        }
        let evaluation = evaluate(&guessed_word, &self.solution, word_list)?;
        let guess_eval = Guess {
            word: guessed_word,
            evaluation,
        };
        let res = self.knowledge.add(&guess_eval);
        if res.is_err() {
            if self.flags.contains(&GameFlag::StrictModeEnabled) {
                res?;
            }
        }

        if guess_eval
            .evaluation
            .iter()
            .all(|eval| *eval == Evaluation::Correct)
        {
            self.state = GameState::Won;
        }
        self.history.push(guess_eval);
        if self.state != GameState::Won && self.history.len() == MAX_GUESSES {
            self.state = GameState::Lost;
        }
        Ok(())
    }

    fn strict_mode_star(&self) -> &str {
        if self.get_strict_mode() == StrictMode::Enabled {
            "*"
        } else {
            " "
        }
    }

    pub fn display_game_state_header(&self, message_builder: &mut serenity::utils::MessageBuilder) {
        let code = self.code.value;
        match self.state {
            GameState::InProgress => {
                message_builder.push_line(format!("Friendle `{code}`"));
                message_builder.push(format!(
                    "{}/{}{} [in progress]",
                    self.history().len(),
                    constants::MAX_GUESSES,
                    self.strict_mode_star(),
                ));

                if self.flags().contains(&GameFlag::SolutionNotInWordList) {
                    message_builder.push(" [not in word list]");
                }
                message_builder.push_line("");
            }
            GameState::Won => {
                let line = format!(
                    "{}/{}{}",
                    self.history().len(),
                    constants::MAX_GUESSES,
                    self.strict_mode_star()
                );
                message_builder.push_line(format!("Friendle `{code}`: {line}"));
            }
            GameState::Lost => {
                let line = format!("X/{}{}", constants::MAX_GUESSES, self.strict_mode_star());
                message_builder.push_line(format!("Friendle `{code}`: {line}"));
            }
        }
    }

    pub fn display_state(
        &self,
        message_builder: &mut serenity::utils::MessageBuilder,
        emoji_mode: EmojiMode,
    ) {
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
            message_builder.push_line(String::from_iter(
                guess
                    .evaluation
                    .iter()
                    .map(|eval| get_emoji(*eval, emoji_mode).to_string()),
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
    fn set_strict_mode() {
        let solution = String::from("tales");

        let mut word_list = HashSet::new();
        word_list.insert(String::from("earth"));
        word_list.insert(String::from("value"));
        word_list.insert(String::from("slime"));

        let mut game = Game::new(Code { value: 1234 }, solution, &word_list).unwrap();

        for _ in 0..2 {
            // try changing modes. We do two iterations with one guess in between.
            // We start in non-strict mode
            assert_eq!(game.get_strict_mode(), StrictMode::Disabled);
            assert_eq!(
                game.set_strict_mode(StrictMode::Disabled),
                Err(ModeChangeError::AlreadySet)
            );

            // setting strict mode succeeds
            assert_eq!(game.set_strict_mode(StrictMode::Enabled), Ok(()));
            // setting the same mode again fails
            assert_eq!(
                game.set_strict_mode(StrictMode::Enabled),
                Err(ModeChangeError::AlreadySet)
            );
            // going back to non-strict mode always works.
            assert_eq!(game.set_strict_mode(StrictMode::Disabled), Ok(()));

            game.guess(String::from("earth"), &word_list).unwrap();
        }

        assert_eq!(
            game.set_strict_mode(StrictMode::Enabled),
            Err(ModeChangeError::TooManyGuessesAlready)
        );
    }

    #[test]
    fn test_win() {
        let word = String::from("tales");
        let word_list = HashSet::from_iter(std::iter::once(word.clone()));
        let mut game = Game::new(Code { value: 1234 }, word.clone(), &word_list).unwrap(); // not the real code, but it doesn't matter here since it's only used for reporting
        game.guess(word.clone(), &word_list).unwrap();
        assert_eq!(game.state, GameState::Won);
        assert!(game.guess(word, &word_list).is_err());
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

        let mut game = Game::new(Code { value: 1234 }, solution, &word_list).unwrap();
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
