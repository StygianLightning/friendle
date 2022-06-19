use thiserror::Error;

use super::knowledge::KnowledgeValidationError;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum InvalidWordError {
    #[error("Expected word of length {expected_length}, received word of length {given_length}")]
    WrongLength {
        expected_length: usize,
        given_length: usize,
    },
    #[error("Only English words with letters A-Z are supported")]
    NonLatinAlpha,
    #[error("Given word {word} is not in the list of supported words")]
    NotInWordList { word: String },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GuessError {
    #[error("No game is currently in progress.")]
    GameNotInProgress,
    #[error("{0}")]
    InvalidWord(#[from] InvalidWordError),
    #[error("{0}")]
    RejectedByStrictMode(#[from] KnowledgeValidationError),
}
