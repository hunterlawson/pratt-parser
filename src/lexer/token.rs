use std::fmt::{Debug, Display};

use strum::EnumIs;

use super::default_types::{Delimiter, Operator};

/// Represents valid tokens produced by the Lexer
/// 
/// `O` and `D` are the [`Operator`] and [`Delimiter`] types.
/// See [`Lexer`][super::Lexer] for examples.
/// 
/// - `Int(i64)` - Integer types: "256", "100", etc.
/// - `Float(f64)` - Float types: "3.14", "0.0001", etc.
/// - `Ident(String)` - Identifiers: "hello", "my_identifier23", etc.
/// - `Op(O)` - User-defined operators: "+", "<--", "-", etc.
/// - `Delimited(D)` - User-defined delimited types: "\[bracketed_type\]", "\\"strings\\"", etc.
/// - `Eof` - Represents the end of the input text
#[derive(Debug, PartialEq, EnumIs, Clone)]
pub enum Token<O, D> {
    /// Integer types: "256", "100", etc.
    Int(i64),
    /// Float types: "3.14", "0.0001", etc.
    Float(f64),
    /// Identifiers: "hello", "my_identifier23", etc.
    Ident(String),
    /// User-defined operators: "+", "<--", "-", etc.
    Op(O),
    /// User-defined delimited types: "\[bracketed_type\]", "\\"strings\\"", etc.
    Delimited(D),
    /// Represents the end of the input text
    Eof,
}

/// Represents a position in a text. Stored as (line, column).
/// 
/// For example, the first character in a given text is (1, 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextPos {
    /// The line in the text. Starts at 1.
    pub line: usize,
    /// The column in the text. Starts at 1.
    pub col: usize,
}

impl Default for TextPos {
    fn default() -> Self {
        Self { line: 1, col: 1 }
    }
}

impl Display for TextPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, col: {}", self.line, self.col)
    }
}

/// Represents a token at the given position (line, col)
/// 
/// See [`TextPos`] and [`Token`]
/// 
/// `pos` can be `None` in circumstances like the EOF was reached.
#[derive(Debug, PartialEq, Clone)]
pub struct TokenPos<O: Operator, D: Delimiter> {
    /// The [`Token`] at this position
    pub token: Token<O, D>,
    /// The [`TextPos`] position of this token.
    /// 
    /// Can be `None` in circumstances like the EOF was reached.
    pub pos: Option<TextPos>,
}

impl<O, D> TokenPos<O, D>
where
    O: Operator,
    D: Delimiter,
{
    /// Create a new [`TokenPos`] with the provided [`Token`] and 
    /// [`TextPos`] text position.
    pub fn new(token: Token<O, D>, pos: TextPos) -> Self {
        Self {
            token,
            pos: Some(pos),
        }
    }

    /// Return a `Token::Eof` with `None` as the position.
    pub fn eof() -> Self {
        Self {
            token: Token::<O, D>::Eof,
            pos: None,
        }
    }
}
