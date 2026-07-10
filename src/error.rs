use thiserror::Error;

use crate::lexer::Token;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Lexer error: Malformed number at pos {pos}: `{str}`")]
    MalformedNumber { pos: usize, str: String },
    #[error("Lexer error: Unexpected character at pos {pos}: `{c}`")]
    UnexpectedChar { pos: usize, c: char },
    #[error("Parsing error: Unexpected expression prefix token: `{0}`")]
    UnexpectedPrefixToken(Token),
    #[error("Parsing error: Expected: `)`, got: `{0}`")]
    MissingExprRParen(Token),
    #[error("Parsing error: Unexpected token: `{0}`")]
    UnexpectedToken(Token),
    #[error("Parsing error: Reached EOF while parsing a subexpression")]
    ReachedEOF,
    #[error("Parsing error: Reached EOF while parsing function args")]
    ReachedEOFArgs,
}

pub type ParserResult<T> = Result<T, ParserError>;
