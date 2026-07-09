use thiserror::Error;

use crate::lexer::Token;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Lexer error: Malformed number at pos {0}: `{1}`")]
    MalformedNumber(usize, String),
    #[error("Lexer error: Unexpected character at pos {0}: `{1}`")]
    UnexpectedChar(usize, char),
    #[error("Parsing error: Unexpected expression prefix token: {0}")]
    UnexpectedPrefixToken(Token)
}

pub type ParserResult<T> = Result<T, ParserError>;