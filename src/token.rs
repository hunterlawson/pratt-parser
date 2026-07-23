use std::{fmt::{Debug, Display}, hash::Hash};

use strum::IntoEnumIterator;

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
#[derive(Debug)]
pub struct TokenPos<O, D> {
    pub token: Token<O, D>,
    pub pos: Option<TextPosition>,
}

/// Types that implement Operator are valid operators for use by the parser
pub trait Operator: Display + IntoEnumIterator + Clone + Copy + Debug {
    /// Get the binding power for this operator type
    fn binding_power(&self) -> (Option<u16>, Option<u16>);
}

/// Types that implement delimited are valid delimited types for use by the parser
pub trait Delimited: IntoEnumIterator + PartialEq + Clone + Debug {
    /// Get the left and right delineators for this type
    fn delimiters(&self) -> Option<(String, String)> {
        None
    }
    /// Set the value of this delimited type
    fn set(&mut self, input: String) {}
}

impl<O, D> TokenPos<O, D>
where
    O: Operator,
    D: Delimited,
{
    pub fn new(token: Token<O, D>, pos: TextPosition) -> Self {
        Self { token, pos: Some(pos) }
    }

    pub fn eof() -> Self {
        Self {
            token: Token::<O, D>::Eof,
            pos: None,
        }
    }
}
