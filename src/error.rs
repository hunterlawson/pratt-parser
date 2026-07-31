use thiserror::Error;

use crate::lexer::token::TextPos;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserError {
    #[error("Lexer error while parsing")]
    LexerError(#[from] LexerError),
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexerError {
    #[error("Unexpected string: '{str}' at {pos}")]
    UnexpectedString { str: String, pos: TextPos },
    #[error("Unexpected character: '{c}' at {pos}")]
    UnexpectedChar { c: char, pos: TextPos },
    #[error("Error parsing integer string: '{str}' at {pos}")]
    IntegerParsingError { str: String, pos: TextPos },
    #[error("Error parsing float string: '{str}' at {pos}")]
    FloatParsingError { str: String, pos: TextPos },
    #[error("Missing closing delimiter `{expected}` for opening delimiter '{open}' at {pos}")]
    MissingClosingDelimiter {
        open: String,
        expected: String,
        pos: TextPos,
    },
}

pub type ParserResult<T> = Result<T, ParserError>;
pub type LexerResult<T> = Result<T, LexerError>;
