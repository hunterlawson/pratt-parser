use thiserror::Error;

use crate::TextPos;

/// Represents an error that occurred during parsing
#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserError {
    /// A lexing error that occurred during parsing. See [`LexerError`].
    #[error("Lexer error while parsing")]
    LexerError(#[from] LexerError),
}

/// Represents an error that occurred from the [`Lexer`](crate::Lexer)
#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexerError {
    /// Unexpected string error. Occurs when the given string is not recognized.
    #[error("Unexpected string: '{str}' at {pos}")]
    UnexpectedString {
        /// The unexpected string
        str: String,
        /// Position of the string in the text
        pos: TextPos,
    },

    /// Unexpected character error. Occurs when the given character is not expected in
    /// the context.
    #[error("Unexpected character: '{c}' at {pos}")]
    UnexpectedChar {
        /// The unexpected character
        c: char,
        /// Position of the character in the text
        pos: TextPos,
    },

    /// Integer parsing error. Occurs when there is an error while parsing an integer
    /// string into an `i64` type.
    #[error("Error parsing integer string: '{str}' at {pos}")]
    IntegerParsingError {
        /// The integer string that produced the parsing error
        str: String,
        /// The position of the integer string in the text
        pos: TextPos,
    },

    /// Float parsing error. Occurs when there is an error while parsing a float
    /// string into an `f64` type.
    #[error("Error parsing float string: '{str}' at {pos}")]
    FloatParsingError {
        /// The float string that produced the parsing error
        str: String,
        /// The position of the float string in the text
        pos: TextPos,
    },

    /// Missing closing delimiter error. Occurs when a closing delmiter was not found
    /// for a given opening delimiter.
    #[error("Missing closing delimiter `{expected}` for opening delimiter '{open}' at {pos}")]
    MissingClosingDelimiter {
        /// The opening delimiter that was found
        open: String,
        /// The closing delimiter that was expected but not found
        expected: String,
        /// The position in the text of the opening delimiter
        pos: TextPos,
    },
}

/// Result from parsing a text into expressions.
///
/// `T` depends on the context but is typically a [`TokenPos`](crate::TokenPos)
pub type ParserResult<T> = Result<T, ParserError>;
/// Result from lexing a text into tokens.
///
/// `T` depends on the context but is typically a [`TokenPos`](crate::TokenPos)
pub type LexerResult<T> = Result<T, LexerError>;
