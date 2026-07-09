use thiserror::Error;

use crate::lexer::{Operator, Token};

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Lexer error: Malformed number at pos {0}: `{1}`")]
    MalformedNumber(usize, String),
    #[error("Lexer error: Unexpected character at pos {0}: `{1}`")]
    UnexpectedChar(usize, char),
    #[error("Parsing error: Unexpected expression prefix token: `{0}`")]
    UnexpectedPrefixToken(Token),
    #[error("Parsing error: Missing `)` for function: `{0}`")]
    MissingFunctionRParen(String),
    #[error("Parsing error: Expected: `)`, got: `{0}`")]
    MissingExprRParen(Token),
    #[error("Parsing error: Unexpected token: `{0}`")]
    UnexpectedToken(Token),
}

pub type ParserResult<T> = Result<T, ParserError>;
