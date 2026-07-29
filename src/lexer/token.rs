use std::fmt::{Debug, Display};

use super::default_types::{Delimited, Operator};

/// Represents valid tokens produced by the Lexer
#[derive(Debug, PartialEq)]
pub enum Token<O, D> {
    Int(i64),
    Float(f64),
    Ident(String),
    Op(O),
    Delimited(D),
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self { line: 1, col: 1 }
    }
}

impl Display for TextPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, col: {}", self.line, self.col)
    }
}

/// Represents a token at the given position (line, col)
#[derive(Debug, PartialEq)]
pub struct TokenPos<O: Operator, D: Delimited> {
    pub token: Token<O, D>,
    pub pos: Option<TextPosition>,
}

impl<O, D> TokenPos<O, D>
where
    O: Operator,
    D: Delimited,
{
    pub fn new(token: Token<O, D>, pos: TextPosition) -> Self {
        Self {
            token,
            pos: Some(pos),
        }
    }

    pub fn eof() -> Self {
        Self {
            token: Token::<O, D>::Eof,
            pos: None,
        }
    }
}
