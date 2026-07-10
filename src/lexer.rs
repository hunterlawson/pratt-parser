use strum::{Display, EnumIs};

use crate::error::{ParserError, ParserResult};

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Token {
    Int(i64),
    Float(f64),
    Ident(String), // Identifier that starts with a letter
    Op(Operator),
    LParen,
    RParen,
    Comma,
    Eof,
}

/// Represents a token with a position in the expression
///
/// pos: `None` represents special tokens like EOF that don't have a pos
#[derive(Clone)]
pub(crate) struct TokenWithPos {
    pub(crate) token: Token,
    pub(crate) pos: Option<usize>,
}

const EOF_TOKEN_POS: TokenWithPos = TokenWithPos {
    token: Token::Eof,
    pos: None,
};

fn token_with_pos(token: Token, pos: usize) -> TokenWithPos {
    match token {
        Token::Eof => EOF_TOKEN_POS,
        _ => TokenWithPos {
            token,
            pos: Some(pos),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIs)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Token {
    fn parse_number(s: &String, pos: usize) -> ParserResult<Token> {
        if s.contains(".") {
            let num = s.parse::<f64>().map_err(|_| ParserError::MalformedNumber {
                pos,
                str: format!("{s}"),
            })?;

            Ok(Token::Float(num))
        } else {
            let num = s.parse::<i64>().map_err(|_| ParserError::MalformedNumber {
                pos,
                str: format!("{s}"),
            })?;

            Ok(Token::Int(num))
        }
    }
}

pub(crate) struct Lexer {
    tokens: Vec<TokenWithPos>,
}

impl Lexer {
    pub(crate) fn new(expr: &str) -> ParserResult<Self> {
        let expr_raw = String::from(expr);
        let mut expr_iter = expr_raw
            .chars()
            .enumerate()
            .filter(|(_, c)| !c.is_ascii_whitespace());
        let mut current_char = expr_iter.next();

        #[derive(PartialEq)]
        enum State {
            Start,
            InIdent,
            InNum,
        }

        let mut tokens = vec![];
        let mut state = State::Start;
        let mut current_str = String::new();
        let mut current_str_pos = 0;

        while let Some((i, char)) = current_char {
            match state {
                State::Start => {
                    // check if it's an operator
                    if let Some(token) = Self::char_to_token(char) {
                        tokens.push(token_with_pos(token, i));
                        current_char = expr_iter.next();
                        continue;
                    }
                    // otherwise match on the character
                    match char {
                        // beginning of an ident
                        c if c.is_ascii_alphabetic() => {
                            state = State::InIdent;
                            current_str.push(c);
                            current_str_pos = i;
                        }
                        // begining of number
                        c if c.is_ascii_digit() => {
                            state = State::InNum;
                            current_str.push(c);
                            current_str_pos = i;
                        }
                        _ => return Err(ParserError::UnexpectedChar { pos: i, c: char }),
                    }

                    current_char = expr_iter.next();
                }
                State::InIdent => match char {
                    c if c.is_alphanumeric() => {
                        current_str.push(c);
                        current_char = expr_iter.next();
                    }
                    _ => {
                        state = State::Start;
                        tokens.push(token_with_pos(Token::Ident(current_str), current_str_pos));
                        current_str = String::new();
                    }
                },
                State::InNum => match char {
                    c if c.is_ascii_digit() || c == '.' => {
                        current_str.push(c);
                        current_char = expr_iter.next();
                    }
                    c if c.is_alphabetic() => {
                        return Err(ParserError::UnexpectedChar { pos: i, c });
                    }
                    _ => {
                        state = State::Start;
                        tokens.push(token_with_pos(
                            Token::parse_number(&current_str, current_str_pos)?,
                            current_str_pos,
                        ));
                        current_str = String::new();
                    }
                },
            }
        }

        // clean up any leftover characters
        if current_str.len() > 0 {
            match state {
                State::Start => (),
                State::InIdent => {
                    tokens.push(token_with_pos(Token::Ident(current_str), current_str_pos))
                }
                State::InNum => tokens.push(token_with_pos(
                    Token::parse_number(&current_str, current_str_pos)?,
                    current_str_pos,
                )),
            }
        }

        // reverse it because next() reads off the top of the stack
        tokens.reverse();

        Ok(Self { tokens })
    }

    fn char_to_token(c: char) -> Option<Token> {
        match c {
            '+' => Some(Token::Op(Operator::Add)),
            '-' => Some(Token::Op(Operator::Sub)),
            '*' => Some(Token::Op(Operator::Mul)),
            '/' => Some(Token::Op(Operator::Div)),
            '^' => Some(Token::Op(Operator::Pow)),
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            ',' => Some(Token::Comma),
            _ => None,
        }
    }

    /// Return the next character and its position
    pub(crate) fn next(&mut self) -> TokenWithPos {
        self.tokens.pop().unwrap_or(EOF_TOKEN_POS)
    }

    /// Peek the next character and its position
    pub(crate) fn peek(&self) -> TokenWithPos {
        self.tokens.last().cloned().unwrap_or(EOF_TOKEN_POS)
    }
}
