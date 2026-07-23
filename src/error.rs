use thiserror::Error;

use crate::token::TextPosition;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserError {
    #[error("Lexer error while parsing")]
    LexerError(#[from] LexerError),
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexerError {
    #[error("Unexpected string: '{str}' at {pos}")]
    UnexpectedString { str: String, pos: TextPosition },
    #[error("Unexpected character: '{c}' at {pos}")]
    UnexpectedChar { c: char, pos: TextPosition },
    #[error("Error parsing integer string: '{str}' at {pos}")]
    IntegerParsingError { str: String, pos: TextPosition },
    #[error("Error parsing float string: '{str}' at {pos}")]
    FloatParsingError { str: String, pos: TextPosition },
    #[error("Missing closing delimiter `{expected}` for opening delimiter '{open}' at {pos}")]
    MissingClosingDelimiter {
        open: String,
        expected: String,
        pos: TextPosition,
    },
}

pub type ParserResult<T> = Result<T, ParserError>;
pub type LexerResult<T> = Result<T, LexerError>;
