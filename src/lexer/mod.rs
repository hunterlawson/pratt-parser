pub mod default_types;
pub mod token;

use std::collections::{HashMap, HashSet};

use crate::{LexerError, LexerResult};
use default_types::{DefaultOperators, Delimited, NullDelimiter, Operator};
use token::{TextPosition, Token, TokenPos};

fn string_prefixes(s: &String) -> Vec<String> {
    s.char_indices()
        .flat_map(|(i, _)| s.get(..=i))
        .map(|s| String::from(s))
        .collect()
}

/// Return whether the given character is allowed inside an identifier
///
/// Identifiers can contain alphanumeric characters and underscores `_`
fn valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

enum SymbolType {
    Op,
    Dl,
}

/// Lexer for use with the Parser struct
#[derive(Debug)]
pub struct Lexer<O, D = NullDelimiter> {
    op_map: HashMap<String, O>,
    dl_l_map: HashMap<String, (D, String)>,
    valid_symbols: HashSet<char>,
    // prefix tables
    symbol_prefixes: HashSet<String>,
    dl_l_prefixes: HashSet<String>,
    dl_r_prefixes: HashSet<String>,
    // text information
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

        // build a set of valid operator and delim prefixes
        let symbol_prefixes: HashSet<String> = O::iter()
            .flat_map(|o| string_prefixes(&o.to_string()))
            .collect();
        let dl_l_prefixes: HashSet<String> = D::iter()
            .flat_map(|d| d.delimiters())
            .flat_map(|(l, _)| string_prefixes(&l))
            .collect();
        let dl_r_prefixes: HashSet<String> = D::iter()
            .flat_map(|d| d.delimiters())
            .flat_map(|(_, r)| string_prefixes(&r))
            .collect();

        Self {
            op_map,
            dl_l_map,
            valid_symbols,
            symbol_prefixes,
            dl_l_prefixes,
            dl_r_prefixes,
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

    /// Peek a specific char with the given offset from the current char
    fn peek_char_offset(&self, offset: usize) -> Option<char> {
        self.text.chars().nth(self.text_index + offset)
    }

    /// Peek the next character in the text if it exists
    fn next_char(&self) -> Option<char> {
        self.peek_char_offset(1)
    }

    /// Peek the current character in the text if it exists
    fn current_char(&self) -> Option<char> {
        self.peek_char_offset(0)
    }

    /// Consume the next char in the text by incrementing the index by 1
    fn consume_char(&mut self) {
        // update the position
        match self.current_char() {
            Some('\n') => {
                self.text_index += 1;
                self.text_pos.line += 1;
                self.text_pos.col = 1;
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
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char()
            && c.is_whitespace()
        {
            println!(
                "Skipping whitespace `{}` at i={}",
                c.escape_debug().collect::<String>(),
                self.text_index
            );
            self.consume_char();
        }
    }

    fn next_token(&mut self) -> LexerResult<TokenPos<O, D>> {
        self.skip_whitespace();

        let Some(c) = self.current_char() else {
            return Ok(TokenPos::eof());
        };

        return match c {
            _ if !self.valid_symbols.contains(&c) && !valid_identifier_char(c) => {
                Err(LexerError::UnexpectedChar {
                    c,
                    pos: self.text_pos,
                })
            }
            _ if c.is_numeric() => self.resolve_num(),
            _ if self.symbol_prefixes.contains(&String::from(c)) => self.resolve_symbol(),
            _ => self.resolve_ident(),
        };
    }

    /// Resolve an identifier
    fn resolve_ident(&mut self) -> LexerResult<TokenPos<O, D>> {
        let start_pos = self.text_pos;
        let mut ident = String::new();

        while let Some(c) = self.current_char() {
            if !valid_identifier_char(c) {
                break;
            }

            ident.push(c);
            self.consume_char();
        }

        Ok(TokenPos::new(Token::Ident(ident), start_pos))
    }

    /// Resolve a symbol - this might be an operator or a delimiter
    fn resolve_symbol(&mut self) -> LexerResult<TokenPos<O, D>> {
        let start_pos = self.text_pos;
        let mut symbol = String::new();
        symbol.push(self.current_char().expect("guaranteed"));

        // check if this is a prefix of a valid symbol
        let mut prefixes = vec![symbol.clone()];
        let mut offset = 1;
        while let Some(c) = self.peek_char_offset(offset) {
            symbol.push(c);
            if self.symbol_prefixes.contains(&symbol) {
                prefixes.push(symbol.clone());
                offset += 1;
            } else {
                break;
            }
        }

        // println!("{:#?}", prefixes);

        // iterate through the found prefixes and return the largest one that
        // is a valid operator
        prefixes.reverse();
        for p in prefixes {
            let Some(o) = self.op_map.get(&p) else {
                continue;
            };
            let o = o.clone();
            // consume all the characters in the prefix
            for _ in 0..p.len() {
                self.consume_char();
            }
            return Ok(TokenPos::new(Token::Op(o), start_pos));
        }

        Err(LexerError::UnexpectedString {
            str: symbol,
            pos: start_pos,
        })
    }

    /// Resolve a number - this might be an integer i64 or a float f64
    fn resolve_num(&mut self) -> LexerResult<TokenPos<O, D>> {
        let start_pos = self.text_pos;

        let mut num = String::new();

        while let Some(c) = self.current_char() {
            match c {
                c if c.is_numeric() || c == '.' => {
                    num.push(c);
                    self.consume_char();
                }
                _ => break,
            }
        }

        if num.contains(".") {
            let float_res = num
                .parse::<f64>()
                .map_err(|_| LexerError::FloatParsingError {
                    str: num,
                    pos: start_pos,
                })?;
            Ok(TokenPos::new(Token::Float(float_res), start_pos))
        } else {
            let int_res = num
                .parse::<i64>()
                .map_err(|_| LexerError::IntegerParsingError {
                    str: num,
                    pos: start_pos,
                })?;
            Ok(TokenPos::new(Token::Int(int_res), start_pos))
        }
    }

    /// Get the next token from the current text
    // fn next_token(&mut self) -> LexerResult<TokenPos<O, D>> {
    //     #[derive(PartialEq)]
    //     enum State {
    //         Start,
    //         InIdent,
    //         InNum,
    //         InDl,
    //         InOp,
    //     }
    //     // skip any starting whitespace
    //     self.skip_non_whitespace();

    //     let start_pos = self.text_pos;
    //     let mut current_str = String::new();
    //     let mut current_state = State::Start;
    //     let mut current_open_dl = String::new();
    //     let mut expected_closing_dl = String::new();

    //     while let Some(c) = self.current_char() {
    //         // check if the current_str + next is a valid prefix
    //         if self.op_prefixes.contains(&current_str) {
    //             current_state = State::InOp;
    //         }

    //         // check if the current_str matches any starting delimiters
    //         // ignore if we're already inside a delimited type
    //         // this lexer does not handle nexted delimited types - you would need to build another lexer
    //         // to parse the string inside the first type
    //         if let Some((_, s)) = self.dl_l_map.get(&current_str)
    //             && current_state == State::Start
    //         {
    //             current_open_dl = current_str.clone();
    //             expected_closing_dl = s.clone();
    //             current_str = String::new();
    //             current_state = State::InDl;
    //         }

    //         // break if whitespace
    //         if c.is_whitespace() && current_state != State::InDl {
    //             break;
    //         }

    //         match current_state {
    //             State::Start => {
    //                 current_str.push(c);
    //                 match c {
    //                     _ if c.is_alphabetic() => current_state = State::InIdent,
    //                     _ if c.is_numeric() => current_state = State::InNum,
    //                     c if !self.valid_symbols.contains(&c) => {
    //                         return Err(LexerError::UnexpectedChar {
    //                             c,
    //                             pos: self.text_pos,
    //                         });
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //             State::InIdent => match c {
    //                 c if c.is_alphanumeric() => current_str.push(c),
    //                 _ => break,
    //             },
    //             State::InNum => match c {
    //                 c if c.is_numeric() || c == '.' => current_str.push(c),
    //                 _ => break,
    //             },
    //             State::InOp => {}
    //             State::InDl => {
    //                 current_str.push(c);
    //                 if current_str.ends_with(&expected_closing_dl) {
    //                     // return the delimited type
    //                     let dl_str = current_str
    //                         .trim_end_matches(&expected_closing_dl)
    //                         .to_string();
    //                     let mut dl = self
    //                         .dl_l_map
    //                         .get(&current_open_dl)
    //                         .expect("guaranteed")
    //                         .0
    //                         .clone();

    //                     dl.set(dl_str);
    //                     let dl_token = Token::Delimited(dl);
    //                     let dl_token_pos = TokenPos::new(dl_token, start_pos);
    //                     self.consume_char(); // consume the delimiter
    //                     return Ok(dl_token_pos);
    //                 }
    //             }
    //         }

    //         self.consume_char();
    //     }

    //     // Output a token from the string depending on the current state
    //     return match current_state {
    //         State::Start => {
    //             if current_str.len() == 0 {
    //                 Ok(TokenPos::eof())
    //             } else {
    //                 Err(LexerError::UnexpectedString {
    //                     str: current_str,
    //                     pos: start_pos,
    //                 })
    //             }
    //         }
    //         State::InIdent => {
    //             let ident_token = Token::Ident(current_str);
    //             let ident_token_pos = TokenPos::new(ident_token, start_pos);
    //             Ok(ident_token_pos)
    //         }
    //         State::InNum => {
    //             if current_str.contains('.') {
    //                 // parse as float
    //                 let f_res =
    //                     current_str
    //                         .parse::<f64>()
    //                         .map_err(|_| LexerError::FloatParsingError {
    //                             str: current_str,
    //                             pos: start_pos,
    //                         })?;
    //                 let f_token = Token::Float(f_res);
    //                 let f_token_pos = TokenPos::new(f_token, start_pos);
    //                 Ok(f_token_pos)
    //             } else {
    //                 // parse as int
    //                 let i_res = current_str.parse::<i64>().map_err(|_| {
    //                     LexerError::IntegerParsingError {
    //                         str: current_str,
    //                         pos: start_pos,
    //                     }
    //                 })?;
    //                 let i_token = Token::Int(i_res);
    //                 let i_token_pos = TokenPos::new(i_token, start_pos);
    //                 Ok(i_token_pos)
    //             }
    //         }
    //         State::InOp => {
    //             todo!()
    //         }
    //         State::InDl => Err(LexerError::MissingClosingDelimiter {
    //             open: current_open_dl,
    //             expected: expected_closing_dl,
    //             pos: start_pos,
    //         }),
    //     };
    // }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.text = text.as_ref().into();
        self.text_index = 0;
        self.text_pos = TextPosition::default();
    }
}

impl Default for Lexer<DefaultOperators, NullDelimiter> {
    fn default() -> Self {
        Self::new()
    }
}
