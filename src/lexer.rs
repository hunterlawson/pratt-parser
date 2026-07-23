use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use strum::{Display, EnumIter};

use crate::{
    LexerError, LexerResult, ParserError, ParserResult,
    token::{Delimited, Operator, TextPosition, Token, TokenPos},
};

/// Default operator type
#[derive(Display, EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum DefaultOps {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
}

impl Operator for DefaultOps {
    fn binding_power(&self) -> (Option<u16>, Option<u16>) {
        (None, None)
    }
}
/// Null delimited type
#[derive(EnumIter, Debug, PartialEq, Clone)]
pub enum NullDelimiter {}
impl Delimited for NullDelimiter {}

/// Lexer for use with the Parser struct
#[derive(Debug)]
pub struct Lexer<O, D = NullDelimiter> {
    op_map: HashMap<String, O>,
    dl_l_map: HashMap<String, (D, String)>,
    valid_symbols: HashSet<char>,
    text: String,
    text_index: usize,
    text_pos: TextPosition,
    current_err: Option<LexerError>,
}

impl<O, D> Lexer<O, D>
where
    O: Operator,
    D: Delimited,
{
    /// Initialize an empty lexer with the given operator and delimited type rules
    pub fn new() -> Self {
        // Build the maps of operators and delineators
        let op_map = O::iter().map(|v| (v.to_string(), v)).collect();

        let dl_l_map = D::iter()
            .filter_map(|v| v.delimiters().map(|d| (d.0, (v, d.1))))
            .collect();

        // add every char from the operators and the delimiters
        let mut valid_symbols: HashSet<char> = O::iter()
            .flat_map(|o| o.to_string().chars().collect::<Vec<char>>())
            .collect();
        let delimiters: Vec<(String, String)> = D::iter().flat_map(|d| d.delimiters()).collect();
        for (left, right) in delimiters {
            valid_symbols.extend(left.chars());
            valid_symbols.extend(right.chars());
        }

        Self {
            op_map,
            dl_l_map,
            valid_symbols,
            text: String::new(),
            text_index: 0,
            text_pos: TextPosition::default(),
            current_err: None,
        }
    }

    pub fn next(&mut self) -> LexerResult<TokenPos<O, D>> {
        if let Some(e) = &self.current_err {
            return Err(e.clone());
        }

        let res = self.next_token();
        if let Err(e) = &res {
            self.current_err = Some(e.clone());
        }

        res
    }

    /// Peek the current character in the text if it exists
    fn current_char(&mut self) -> Option<char> {
        self.text.chars().nth(self.text_index)
    }

    /// Consume the next char in the text by incrementing the index by 1
    fn consume_char(&mut self) {
        // update the position
        match self.current_char() {
            Some('\n') => {
                self.text_index += 1;
                self.text_pos.line += 1;
                self.text_pos.col = 0;
            }
            Some(_) => {
                self.text_index += 1;
                self.text_pos.col += 1;
            }
            None => (),
        }
    }

    /// Consume chars until the next non-whitespace character in the text
    ///
    /// Increment the index and update the position until we find it
    fn skip_non_whitespace(&mut self) {
        while let Some(c) = self.current_char()
            && c.is_whitespace()
        {
            self.consume_char();
        }
    }

    /// Get the next token from the current text
    fn next_token(&mut self) -> LexerResult<TokenPos<O, D>> {
        #[derive(PartialEq)]
        enum State {
            Start,
            InIdent,
            InNum,
            InDl,
        }
        // skip any starting whitespace
        self.skip_non_whitespace();

        let start_pos = self.text_pos;
        let mut current_str = String::new();
        let mut current_state = State::Start;
        let mut current_open_dl = String::new();
        let mut expected_closing_dl = String::new();

        while let Some(c) = self.current_char() {
            // check if the current_str matches any operators
            if let Some(o) = self.op_map.get(&current_str) {
                let op_token = Token::Op(o.clone());
                let op_token_pos = TokenPos::new(op_token, start_pos);
                self.consume_char(); // consume the operator right char
                return Ok(op_token_pos);
            }
            // check if the current_str matches any starting delimiters
            // ignore if we're already inside a delimited type
            // this lexer does not handle nexted delimited types - you would need to build another lexer
            // that then parses the string inside the first type
            if let Some((_, s)) = self.dl_l_map.get(&current_str)
                && current_state == State::Start
            {
                current_open_dl = current_str.clone();
                expected_closing_dl = s.clone();
                current_str = String::new();
                current_state = State::InDl;
            }

            // break if whitespace
            if c.is_whitespace() && current_state != State::InDl {
                break;
            }

            match current_state {
                State::Start => {
                    current_str.push(c);
                    match c {
                        _ if c.is_alphabetic() => current_state = State::InIdent,
                        _ if c.is_numeric() => current_state = State::InNum,
                        c if !self.valid_symbols.contains(&c) => {
                            return Err(LexerError::UnexpectedChar {
                                c,
                                pos: self.text_pos,
                            });
                        }
                        _ => (),
                    }
                }
                State::InIdent => match c {
                    c if c.is_alphanumeric() => current_str.push(c),
                    _ => break,
                },
                State::InNum => match c {
                    c if c.is_numeric() || c == '.' => current_str.push(c),
                    _ => break,
                },
                State::InDl => {
                    current_str.push(c);
                    if current_str.ends_with(&expected_closing_dl) {
                        // return the delimited type
                        let dl_str = current_str
                            .trim_end_matches(&expected_closing_dl)
                            .to_string();
                        let mut dl = self
                            .dl_l_map
                            .get(&current_open_dl)
                            .expect("guaranteed")
                            .0
                            .clone();

                        dl.set(dl_str);
                        let dl_token = Token::Delimited(dl);
                        let dl_token_pos = TokenPos::new(dl_token, start_pos);
                        self.consume_char(); // consume the delimiter
                        return Ok(dl_token_pos);
                    }
                }
            }

            self.consume_char();
        }

        // Output a token from the string depending on the current state
        match current_state {
            State::Start => {
                if current_str.len() == 0 {
                    Ok(TokenPos::eof())
                } else {
                    Err(LexerError::UnexpectedString {
                        str: current_str,
                        pos: start_pos,
                    })
                }
            }
            State::InIdent => {
                let ident_token = Token::Ident(current_str);
                let ident_token_pos = TokenPos::new(ident_token, start_pos);
                Ok(ident_token_pos)
            }
            State::InNum => {
                if current_str.contains('.') {
                    // parse as float
                    let f_res =
                        current_str
                            .parse::<f64>()
                            .map_err(|_| LexerError::FloatParsingError {
                                str: current_str,
                                pos: start_pos,
                            })?;
                    let f_token = Token::Float(f_res);
                    let f_token_pos = TokenPos::new(f_token, start_pos);
                    Ok(f_token_pos)
                } else {
                    // parse as int
                    let i_res = current_str.parse::<i64>().map_err(|_| {
                        LexerError::IntegerParsingError {
                            str: current_str,
                            pos: start_pos,
                        }
                    })?;
                    let i_token = Token::Int(i_res);
                    let i_token_pos = TokenPos::new(i_token, start_pos);
                    Ok(i_token_pos)
                }
            }
            State::InDl => {
                return Err(LexerError::MissingClosingDelimiter {
                    open: current_open_dl,
                    expected: expected_closing_dl,
                    pos: start_pos,
                });
            }
        }
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.text = text.as_ref().into();
        self.text_index = 0;
        self.text_pos = TextPosition::default();
    }
}

impl Default for Lexer<DefaultOps, NullDelimiter> {
    fn default() -> Self {
        Self::new()
    }
}
