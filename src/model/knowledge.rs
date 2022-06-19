use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use thiserror::Error;

use super::{evaluation::Evaluation, game::Guess};

#[derive(Debug, Clone)]
pub struct PositionalKnowledge {
    index: usize,
    knowledge_state: PositionalKnowledgeState,
}

#[derive(Debug, Clone)]
pub enum PositionalKnowledgeState {
    /// The correct character for this position has been discovered.
    FixedLetter(char),
    /// The character for this position hasn't been discovered yet.
    /// The set contains characters that are known to not be present at this position.
    IncorrectLetters(HashSet<char>),
}

impl PositionalKnowledge {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            knowledge_state: PositionalKnowledgeState::IncorrectLetters(Default::default()),
        }
    }

    pub fn validate(&self, c: char) -> Result<(), KnowledgeValidationError> {
        match &self.knowledge_state {
            PositionalKnowledgeState::FixedLetter(fixed_char) => {
                if c != *fixed_char {
                    Err(KnowledgeValidationError::UnusedFixedCharacter {
                        position: self.index,
                        given_character: c,
                        correct_character: *fixed_char,
                    })
                } else {
                    Ok(())
                }
            }
            PositionalKnowledgeState::IncorrectLetters(incorrect_set) => {
                if incorrect_set.contains(&c) {
                    Err(KnowledgeValidationError::IncorrectPlacement {
                        position: self.index,
                        character: c,
                    })
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CharacterBound {
    Minimum(usize),
    Exact(usize),
}

#[derive(Debug, Clone)]
pub struct Knowledge {
    // both for absent characters as well as character limits.
    known_character_bounds: HashMap<char, CharacterBound>,
    positional_knowledge: Vec<PositionalKnowledge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeValidationError {
    #[error(
        "Position {position} is known to be `{correct_character}` but `{given_character}` was given."
    )]
    UnusedFixedCharacter {
        position: usize,
        given_character: char,
        correct_character: char,
    },
    #[error("Position {position} is known to not be given character `{character}`.")]
    IncorrectPlacement { position: usize, character: char },
    #[error("Character {character} is known to occur {bound}")]
    WrongCount {
        character: char,
        given_count: usize,
        bound: CharacterBound,
    },
}

impl CharacterBound {
    fn new(count: usize, exact: bool) -> Self {
        if exact {
            CharacterBound::Exact(count)
        } else {
            CharacterBound::Minimum(count)
        }
    }

    fn best_bound(lhs: CharacterBound, rhs: CharacterBound) -> CharacterBound {
        match (lhs, rhs) {
            (CharacterBound::Exact(_), _) => lhs,
            (_, CharacterBound::Exact(_)) => rhs,
            (CharacterBound::Minimum(left_min), CharacterBound::Minimum(right_min)) => {
                CharacterBound::Minimum(usize::max(left_min, right_min))
            }
        }
    }

    fn count_string(&self) -> String {
        let count = match self {
            CharacterBound::Minimum(count) | CharacterBound::Exact(count) => count,
        };

        match count {
            1 => String::from("once"),
            2 => String::from("twice"),
            n => format!("{n} times"),
        }
    }
}

impl Display for CharacterBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterBound::Minimum(_) => write!(f, "at least {}.", self.count_string()),
            CharacterBound::Exact(_) => write!(f, "exactly {}.", self.count_string()),
        }
    }
}

impl Default for CharacterBound {
    fn default() -> Self {
        CharacterBound::Minimum(0)
    }
}

impl CharacterBound {
    fn validate_count(
        &self,
        character: char,
        given_count: usize,
    ) -> Result<(), KnowledgeValidationError> {
        match self {
            CharacterBound::Minimum(min) => {
                if given_count >= *min {
                    Ok(())
                } else {
                    Err(KnowledgeValidationError::WrongCount {
                        character: character,
                        given_count,
                        bound: *self,
                    })
                }
            }
            CharacterBound::Exact(exact) => {
                if given_count == *exact {
                    Ok(())
                } else {
                    Err(KnowledgeValidationError::WrongCount {
                        character: character,
                        given_count,
                        bound: *self,
                    })
                }
            }
        }
    }
}

impl Knowledge {
    pub fn new(num_letters: usize) -> Self {
        Self {
            known_character_bounds: Default::default(),
            positional_knowledge: (0..num_letters)
                .map(|i| PositionalKnowledge::new(i))
                .collect(),
        }
    }

    pub fn add(&mut self, guess: &Guess) -> Result<(), KnowledgeValidationError> {
        let guess_character_frequency = itertools::Itertools::counts(guess.word.chars());
        let characters_in_guess = guess.word.chars().collect::<Vec<_>>();

        // Validate all known per-character limits
        for (character, bound) in self.known_character_bounds.iter() {
            let count_given = *guess_character_frequency.get(character).unwrap_or(&0);
            bound.validate_count(*character, count_given)?;
        }

        // Check each position for known fixed or incorrect characters
        for (i, c) in characters_in_guess.iter().enumerate() {
            let positional_knowledge = &self.positional_knowledge[i];
            positional_knowledge.validate(*c)?;
        }

        // No validation errors past this point.

        // Update character limits
        // We can identify exact character bounds only through "Absent" evaluations.
        // If there is an absent evaluation, the exact bound is the number of non-Absent evaluations for the same character.
        // Otherwise, we might have a new (higher) minimum bound.
        for evaluation_index in 0..guess.word.len() {
            let c = characters_in_guess[evaluation_index];
            // Count -Absent occurrences of `c`.
            // This is technically quadratic in the word size, but the word size is constant.

            let mut correct_count = 0;
            let mut has_absent_eval = false;
            for i in 0..guess.word.len() {
                if characters_in_guess[i] == c {
                    let eval = &guess.evaluation[i];
                    if *eval == Evaluation::Absent {
                        has_absent_eval = true;
                    } else {
                        correct_count += 1;
                    }
                }
            }
            let new_bound = CharacterBound::new(correct_count, has_absent_eval);
            let existing_bound = self
                .known_character_bounds
                .get(&c)
                .cloned()
                .unwrap_or_default();
            let new_bound = CharacterBound::best_bound(new_bound, existing_bound);
            self.known_character_bounds.insert(c, new_bound);
        }

        // Update positional knowledge
        for (i, c) in guess.word.chars().enumerate() {
            match guess.evaluation[i] {
                Evaluation::Correct => {
                    // A Correct guess cannot fail any validations, and we lose no information by updating the state.
                    self.positional_knowledge[i].knowledge_state =
                        PositionalKnowledgeState::FixedLetter(c);
                }
                Evaluation::Present | Evaluation::Absent => {
                    // Absent evaluations (black Wordle characters) still go here; the character might appear multiple times and be present in another position.
                    // Character limits are handled outside of positional knowledge.
                    if let PositionalKnowledgeState::IncorrectLetters(incorrect_letters) =
                        &mut self.positional_knowledge[i].knowledge_state
                    {
                        incorrect_letters.insert(c);
                    } else {
                        eprintln!(
                            "Incorrect letter {} added to position knowledge ,
                            but the knowledge state doesn't have a set of incorrect letters: {:?}.
                            This means Knowledge::add() shouldn't be called, and validation should catch this.",
                            c, self.positional_knowledge[i]
                        )
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        game::{Game, GameState, StrictMode},
        guess_error::GuessError,
    };

    use super::*;

    #[test]
    fn test_bounds_merge() -> anyhow::Result<()> {
        let bound_a = CharacterBound::new(1, false);
        let bound_b = CharacterBound::new(2, false);
        assert_eq!(CharacterBound::best_bound(bound_a, bound_b), bound_b);
        assert_eq!(CharacterBound::best_bound(bound_b, bound_a), bound_b);

        let bound_c = CharacterBound::new(3, false);
        assert_eq!(CharacterBound::best_bound(bound_b, bound_c), bound_c);
        assert_eq!(CharacterBound::best_bound(bound_c, bound_b), bound_c);

        let bound_d = CharacterBound::new(3, true);
        assert_eq!(CharacterBound::best_bound(bound_b, bound_d), bound_d);
        assert_eq!(CharacterBound::best_bound(bound_d, bound_b), bound_d);
        assert_eq!(CharacterBound::best_bound(bound_c, bound_d), bound_d);
        assert_eq!(CharacterBound::best_bound(bound_d, bound_c), bound_d);

        Ok(())
    }

    #[test]
    fn test_knowledge_validation() -> anyhow::Result<()> {
        let word = "tales";
        let code = crate::model::coding::encode(word);

        let mut word_list = HashSet::new();
        word_list.insert(String::from("value"));
        word_list.insert(String::from("pales"));
        word_list.insert(String::from("earth"));
        word_list.insert(String::from("slime"));

        // game in non-strict mode.
        let mut game = Game::new(code, String::from(word), &word_list)?;
        // Enabling strict mode will make game::guess() report errors against its internal knowledge.
        game.set_strict_mode(StrictMode::Enabled).unwrap();

        // start by guessing a word.
        game.guess(String::from("value"), &word_list)?;

        // guessing the same word again fails
        let err = game.guess(String::from("value"), &word_list).unwrap_err();
        assert!(matches!(
            err,
            GuessError::RejectedByStrictMode(KnowledgeValidationError::WrongCount {
                // We can't assert_eq here because we'll get a wrong count for one of the letters that we know is incorrect: either 'v' or 'u'
                // Which letter we get depends on HashMap iteration order, which is nondeterministic.
                // All other letters are in the solution, and counts are checked before positional knowledge, so we can have one of two errors here:
                character: 'v' | 'u',
                given_count: 1,
                bound: CharacterBound::Exact(0)
            })
        ));

        // guesses rejected by strict mode validation aren't counted
        assert_eq!(game.history().len(), 1);

        {
            // clone the game to not affect the original
            let mut game = game.clone();
            // guessing slime also fails
            let err = game.guess(String::from("slime"), &word_list).unwrap_err();
            assert_eq!(
                err,
                GuessError::RejectedByStrictMode(KnowledgeValidationError::WrongCount {
                    character: 'a',
                    given_count: 0,
                    bound: CharacterBound::Minimum(1)
                })
            );
        }

        // guessing "pales" works
        game.guess(String::from("pales"), &word_list)?;

        // guessing the solution, "tales" works
        game.guess(String::from("tales"), &word_list)?;
        assert_eq!(game.state(), GameState::Won);

        Ok(())
    }

    #[test]
    fn test_yellow_character_is_rejected() -> anyhow::Result<()> {
        let word = "schwa";
        let code = crate::model::coding::encode(word);

        let mut word_list = HashSet::new();
        let first_guess = String::from("scant");
        let second_guess = String::from("scald");
        word_list.insert(String::from(word));
        word_list.insert(first_guess.clone());
        word_list.insert(second_guess.clone());

        let mut game = Game::new(code, String::from(word), &word_list)?;
        game.set_strict_mode(StrictMode::Enabled).unwrap();

        game.guess(first_guess, &word_list)?;
        let err = game.guess(second_guess, &word_list).unwrap_err();
        assert_eq!(
            err,
            GuessError::RejectedByStrictMode(KnowledgeValidationError::IncorrectPlacement {
                position: 2,
                character: 'a'
            })
        );
        Ok(())
    }

    #[test]
    fn test_character_limits_duplicates() -> anyhow::Result<()> {
        let word = "fates";
        let code = crate::model::coding::encode(word);

        let mut word_list = HashSet::new();
        let first_guess = String::from("fluff");
        let second_guess = String::from("fifty");
        word_list.insert(String::from("word"));
        word_list.insert(first_guess.clone());
        word_list.insert(second_guess.clone());

        let mut game = Game::new(code, String::from(word), &word_list)?;
        game.set_strict_mode(StrictMode::Enabled).unwrap();

        game.guess(first_guess.clone(), &word_list)?;

        // Trying the same guess again fails:
        let err = game.guess(first_guess, &word_list).unwrap_err();
        assert!(matches!(
            err,
            GuessError::RejectedByStrictMode(KnowledgeValidationError::WrongCount { .. })
        ));
        // failed guesses aren't counted.
        assert_eq!(game.history().len(), 1);

        let err = game.guess(second_guess, &word_list).unwrap_err();
        assert_eq!(
            err,
            GuessError::RejectedByStrictMode(KnowledgeValidationError::WrongCount {
                character: 'f',
                given_count: 2,
                bound: CharacterBound::Exact(1)
            })
        );

        Ok(())
    }
}
