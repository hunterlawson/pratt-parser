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

/// Lexer for use with the Parser struct
#[derive(Debug)]
pub struct Lexer<O: Operator, D: Delimited = NullDelimiter> {
    op_map: HashMap<String, O>,
    dl_l_map: HashMap<String, (D, String)>,
    valid_symbol_chars: HashSet<char>,
    // prefix tables
    symbol_prefixes: HashSet<String>,
    // text information
    text: String,
    text_index: usize,
    text_pos: TextPosition,
    current_err: Option<LexerError>,
    // cached values
    cached_tokens: Option<Vec<TokenPos<O, D>>>,
}

impl<O, D> Lexer<O, D>
where
    O: Operator,
    D: Delimited,
{
    /// Initialize an empty lexer with the given operator and delimited type rules
    pub fn new() -> Self {
        // build the maps of operators and delineators
        let op_map = O::iter().map(|v| (v.to_string(), v)).collect();

        // map left_delimiter -> (D, right_delimiter)
        // where D is an instance of the correct delimiter type
        let dl_l_map = D::iter()
            .filter_map(|v| v.delimiters().map(|d| (d.0, (v, d.1))))
            .collect();

        // add every char from the operators and the delimiters
        let mut valid_symbol_chars: HashSet<char> = O::iter()
            .flat_map(|o| o.to_string().chars().collect::<Vec<char>>())
            .collect();
        let delimiters: Vec<(String, String)> = D::iter().flat_map(|d| d.delimiters()).collect();
        for (l, r) in delimiters {
            valid_symbol_chars.extend(l.chars());
            valid_symbol_chars.extend(r.chars());
        }

        // build a set of valid operator and delim prefixes
        let mut symbol_prefixes: HashSet<String> = O::iter()
            .flat_map(|o| string_prefixes(&o.to_string()))
            .collect();
        // Add the left delimiter prefixes to the symbol prefix set
        let dl_l_prefixes: HashSet<String> = D::iter()
            .flat_map(|d| d.delimiters())
            .flat_map(|(l, _)| string_prefixes(&l))
            .collect();
        symbol_prefixes.extend(dl_l_prefixes.clone());

        Self {
            op_map,
            dl_l_map,
            valid_symbol_chars,
            symbol_prefixes,
            text: String::new(),
            text_index: 0,
            text_pos: TextPosition::default(),
            current_err: None,
            cached_tokens: None,
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

    /// Peek the current character in the text if it exists
    fn current_char(&self) -> Option<char> {
        self.peek_char_offset(0)
    }

    fn consume_chars(&mut self, num: usize) {
        for _ in 0..num {
            self.consume_char();
        }
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
            self.consume_char();
        }
    }

    /// Return if the given character is valid to start any token
    ///
    /// valid: valid_symbol_chars, alphanumeric characters and '_'
    fn valid_token_starting_char(&self, c: char) -> bool {
        self.valid_symbol_chars.contains(&c) || valid_identifier_char(c)
    }

    fn next_token(&mut self) -> LexerResult<TokenPos<O, D>> {
        self.skip_whitespace();

        let Some(c) = self.current_char() else {
            return Ok(TokenPos::eof());
        };

        return match c {
            _ if !self.valid_token_starting_char(c) => Err(LexerError::UnexpectedChar {
                c,
                pos: self.text_pos,
            }),
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

        // iterate through the found prefixes and return the largest one that
        // is a valid operator or a valid left delimiter
        prefixes.reverse();
        for p in prefixes {
            // if it's a valid left delimiter
            if let Some((d, r)) = self.dl_l_map.get(&p) {
                let d = d.clone();
                let r = r.clone();
                // Consume the left delimiter
                self.consume_chars(p.len());
                return self.resolve_dl(d, p.clone(), r, start_pos);
            }

            // check if it's a valid operator otherwise continue to the next iter
            let Some(o) = self.op_map.get(&p) else {
                continue;
            };
            let o = o.clone();
            // consume all the characters in the prefix
            self.consume_chars(p.len());
            return Ok(TokenPos::new(Token::Op(o), start_pos));
        }

        Err(LexerError::UnexpectedString {
            str: symbol,
            pos: start_pos,
        })
    }

    /// Resolve a delimited type given the delimiter and the correct right-delimiter string
    fn resolve_dl(
        &mut self,
        mut d: D,
        l: String,
        r: String,
        l_dl_pos: TextPosition,
    ) -> LexerResult<TokenPos<O, D>> {
        let start_pos = l_dl_pos;
        let mut dl_value = String::new();
        let mut found_closing = false;

        // consume characters until we've hit a valid right delimiter for this type
        while let Some(c) = self.current_char() {
            dl_value.push(c);
            self.consume_char();

            if dl_value.ends_with(&r) {
                found_closing = true;
                break;
            }
        }

        if found_closing {
            // remove the closing right delimiter from the string before storing it in the type
            dl_value = dl_value.replace(&r, "");
            d.set(dl_value);
            return Ok(TokenPos {
                token: Token::Delimited(d),
                pos: Some(start_pos),
            });
        }

        Err(LexerError::MissingClosingDelimiter {
            open: l,
            expected: r,
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

    /// Attempt to lex the entire input string into tokens. Reads until EOF is reached. The last
    /// token of the vector will be a `TokenPos::Eof`
    ///
    /// Resets the internal token position used by `next`
    pub fn lex_all(&mut self) -> LexerResult<Vec<TokenPos<O, D>>> {
        if let Some(e) = &self.current_err {
            return Err(e.clone());
        }

        if let Some(v) = &self.cached_tokens {
            return Ok(v.clone());
        }

        let v = self.lex_all_tokens()?;
        self.cached_tokens = Some(v.clone());

        Ok(v)
    }

    fn lex_all_tokens(&mut self) -> LexerResult<Vec<TokenPos<O, D>>> {
        self.reset_position();

        let mut tokens = vec![];
        let mut reached_eof = false;
        loop {
            let tp = self.next()?;
            if tp.token.is_eof() {
                reached_eof = true;
            }
            tokens.push(tp);

            if reached_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn reset_position(&mut self) {
        self.text_index = 0;
        self.text_pos = TextPosition::default();
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.current_err = None;
        self.cached_tokens = None;
        self.text = text.as_ref().into();
        self.reset_position();
    }

    /// Get a reference to the underlying text
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Default for Lexer<DefaultOperators, NullDelimiter> {
    fn default() -> Self {
        Self::new()
    }
}
