//! Pratt parser

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod error;
mod lexer;

pub use error::*;
pub use lexer::*;

#[cfg(test)]
mod tests {
    use strum::{Display, EnumIter};

    use crate::{
        DefaultOperators, Delimiter, Lexer, LexerError, Operator, TextPos, Token, TokenPos,
    };

    #[derive(Clone, PartialEq, EnumIter, Debug)]
    enum TestDelimiter {
        Str(String),
        Arr(String),
    }

    impl Delimiter for TestDelimiter {
        fn delimiters(&self) -> Option<(String, String)> {
            Some(match self {
                TestDelimiter::Str(_) => ("\"".into(), "\"".into()),
                TestDelimiter::Arr(_) => ("[[".into(), "]]".into()),
            })
        }

        fn set(&mut self, input: String) {
            match self {
                TestDelimiter::Str(x) => *x = input,
                TestDelimiter::Arr(x) => *x = input,
            }
        }
    }

    #[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq)]
    enum TestOperator {
        #[strum(to_string = "+")]
        Add,
        #[strum(to_string = "-")]
        Sub,
        #[strum(to_string = ">")]
        Gt,
        #[strum(to_string = "=")]
        Eq,
        #[strum(to_string = "*")]
        Mul,
        #[strum(to_string = ">=>$")]
        Crazy,
    }

    impl Operator for TestOperator {
        fn binding_power(&self) -> (Option<u16>, Option<u16>) {
            todo!()
        }
    }

    fn op_pos<O: Operator, D: Delimiter>(op: O, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Op(op), TextPos { line, col })
    }

    fn ident_pos<O: Operator, D: Delimiter>(s: &str, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Ident(s.into()), TextPos { line, col })
    }

    fn int_pos<O: Operator, D: Delimiter>(i: i64, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Int(i), TextPos { line, col })
    }

    fn float_pos<O: Operator, D: Delimiter>(f: f64, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Float(f), TextPos { line, col })
    }

    fn dl_pos<O: Operator, D: Delimiter>(d: D, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Delimited(d), TextPos { line, col })
    }

    #[test]
    fn lexer_testoperators_simple() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("my_ident + 3 * 10-1");
        assert_eq!(lexer.next().unwrap(), ident_pos("my_ident", 1, 1));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Add, 1, 10));
        assert_eq!(lexer.next().unwrap(), int_pos(3, 1, 12));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Mul, 1, 14));
        assert_eq!(lexer.next().unwrap(), int_pos(10, 1, 16));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Sub, 1, 18));
        assert_eq!(lexer.next().unwrap(), int_pos(1, 1, 19));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof())
    }

    #[test]
    fn lexer_testoperators_lex_all() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("my_ident + 3 * 10-1");
        let tokens = lexer.lex_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                ident_pos("my_ident", 1, 1),
                op_pos(TestOperator::Add, 1, 10),
                int_pos(3, 1, 12),
                op_pos(TestOperator::Mul, 1, 14),
                int_pos(10, 1, 16),
                op_pos(TestOperator::Sub, 1, 18),
                int_pos(1, 1, 19),
                TokenPos::eof(),
            ]
        );

        // try again
        let tokens = lexer.lex_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                ident_pos("my_ident", 1, 1),
                op_pos(TestOperator::Add, 1, 10),
                int_pos(3, 1, 12),
                op_pos(TestOperator::Mul, 1, 14),
                int_pos(10, 1, 16),
                op_pos(TestOperator::Sub, 1, 18),
                int_pos(1, 1, 19),
                TokenPos::eof(),
            ]
        );
    }

    #[test]
    fn lexer_testoperators_complex() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("left >=>* right >=>$ final");
        assert_eq!(lexer.next().unwrap(), ident_pos("left", 1, 1));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Gt, 1, 6));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Eq, 1, 7));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Gt, 1, 8));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Mul, 1, 9));
        assert_eq!(lexer.next().unwrap(), ident_pos("right", 1, 11));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Crazy, 1, 17));
        assert_eq!(lexer.next().unwrap(), ident_pos("final", 1, 22));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof())
    }

    #[test]
    fn lexer_defaultops() {
        let mut lexer = Lexer::default();

        lexer.set_text("my_ident + 3 * 10-1>=10.2");
        assert_eq!(lexer.next().unwrap(), ident_pos("my_ident", 1, 1));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Add, 1, 10));
        assert_eq!(lexer.next().unwrap(), int_pos(3, 1, 12));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Mul, 1, 14));
        assert_eq!(lexer.next().unwrap(), int_pos(10, 1, 16));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Sub, 1, 18));
        assert_eq!(lexer.next().unwrap(), int_pos(1, 1, 19));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Gte, 1, 20));
        assert_eq!(lexer.next().unwrap(), float_pos(10.2, 1, 22));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof())
    }

    #[test]
    fn lexer_testdelimiter_simple() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("[[this is my array type]]");
        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(TestDelimiter::Arr("this is my array type".into()), 1, 1)
        );

        lexer.set_text("\"this is my string type\"");
        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(TestDelimiter::Str("this is my string type".into()), 1, 1)
        );

        lexer.set_text("\"t\"");
        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(TestDelimiter::Str("t".into()), 1, 1)
        );
    }

    #[test]
    fn lexer_testdelimiter_complex() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("[[this is my array type]]\"and a string\"[[\"this is still an array\"]]");
        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(TestDelimiter::Arr("this is my array type".into()), 1, 1)
        );

        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(TestDelimiter::Str("and a string".into()), 1, 26)
        );
        assert_eq!(
            lexer.next().unwrap(),
            dl_pos(
                TestDelimiter::Arr("\"this is still an array\"".into()),
                1,
                40
            )
        );
    }

    #[test]
    fn lexer_empty_text() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("");
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
        // calling again should keep returning Eof, not panic or error
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }

    #[test]
    fn lexer_whitespace_only() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("   \t\n\n  \n");
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }

    #[test]
    fn lexer_tracks_position_across_newlines() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("abc\n  def+\nghi");
        assert_eq!(lexer.next().unwrap(), ident_pos("abc", 1, 1));
        assert_eq!(lexer.next().unwrap(), ident_pos("def", 2, 3));
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Add, 2, 6));
        assert_eq!(lexer.next().unwrap(), ident_pos("ghi", 3, 1));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }

    #[test]
    fn lexer_identifier_digits_and_underscores() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("_leading var_2 trailing_ __");
        assert_eq!(lexer.next().unwrap(), ident_pos("_leading", 1, 1));
        assert_eq!(lexer.next().unwrap(), ident_pos("var_2", 1, 10));
        assert_eq!(lexer.next().unwrap(), ident_pos("trailing_", 1, 16));
        assert_eq!(lexer.next().unwrap(), ident_pos("__", 1, 26));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }

    #[test]
    fn lexer_unexpected_char_error() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        // '@' isn't part of any operator, delimiter, or identifier
        lexer.set_text("ok @ nope");
        assert_eq!(lexer.next().unwrap(), ident_pos("ok", 1, 1));
        assert_eq!(
            lexer.next(),
            Err(LexerError::UnexpectedChar {
                c: '@',
                pos: TextPos { line: 1, col: 4 },
            })
        );
    }

    #[test]
    fn lexer_error_is_sticky() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("@");
        let first = lexer.next();
        assert!(first.is_err());
        // subsequent calls should keep returning the same cached error
        assert_eq!(first, lexer.next());
        assert_eq!(first, lexer.next());
    }

    #[test]
    fn lexer_set_text_clears_previous_error() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("@");
        assert!(lexer.next().is_err());

        lexer.set_text("+");
        assert_eq!(lexer.next().unwrap(), op_pos(TestOperator::Add, 1, 1));
    }

    #[test]
    fn lexer_unexpected_string_error() {
        // an operator type where a valid prefix never resolves to a complete
        // token if not followed by the right characters
        #[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq)]
        enum PrefixOnlyOperator {
            #[strum(to_string = "ab")]
            Weird,
        }
        impl Operator for PrefixOnlyOperator {}

        let mut lexer = Lexer::<PrefixOnlyOperator>::new();
        lexer.set_text("ac");
        assert_eq!(
            lexer.next(),
            Err(LexerError::UnexpectedString {
                str: "ac".into(),
                pos: TextPos { line: 1, col: 1 },
            })
        );
    }

    #[test]
    fn lexer_missing_closing_delimiter_error() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("\"unterminated");
        assert_eq!(
            lexer.next(),
            Err(LexerError::MissingClosingDelimiter {
                open: "\"".into(),
                expected: "\"".into(),
                pos: TextPos { line: 1, col: 1 },
            })
        );
    }

    #[test]
    fn lexer_integer_parsing_error_on_overflow() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        // larger than i64::MAX
        lexer.set_text("99999999999999999999");
        assert_eq!(
            lexer.next(),
            Err(LexerError::IntegerParsingError {
                str: "99999999999999999999".into(),
                pos: TextPos { line: 1, col: 1 },
            })
        );
    }

    #[test]
    fn lexer_float_parsing_error_on_multiple_dots() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("1.2.3");
        assert_eq!(
            lexer.next(),
            Err(LexerError::FloatParsingError {
                str: "1.2.3".into(),
                pos: TextPos { line: 1, col: 1 },
            })
        );
    }

    #[test]
    fn lexer_lex_all_stops_at_first_error() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        lexer.set_text("ok @ nope");
        assert_eq!(
            lexer.lex_all(),
            Err(LexerError::UnexpectedChar {
                c: '@',
                pos: TextPos { line: 1, col: 4 },
            })
        );
    }

    #[test]
    fn lexer_defaultops_bitwise_and_logical() {
        let mut lexer = Lexer::default();

        lexer.set_text("a & b | c ^ d << 1 >> 2 && e || !f");
        assert_eq!(lexer.next().unwrap(), ident_pos("a", 1, 1));
        assert_eq!(
            lexer.next().unwrap(),
            op_pos(DefaultOperators::BwAnd, 1, 3)
        );
        assert_eq!(lexer.next().unwrap(), ident_pos("b", 1, 5));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::BwOr, 1, 7));
        assert_eq!(lexer.next().unwrap(), ident_pos("c", 1, 9));
        assert_eq!(
            lexer.next().unwrap(),
            op_pos(DefaultOperators::BwXor, 1, 11)
        );
        assert_eq!(lexer.next().unwrap(), ident_pos("d", 1, 13));
        assert_eq!(
            lexer.next().unwrap(),
            op_pos(DefaultOperators::LShift, 1, 15)
        );
        assert_eq!(lexer.next().unwrap(), int_pos(1, 1, 18));
        assert_eq!(
            lexer.next().unwrap(),
            op_pos(DefaultOperators::RShift, 1, 20)
        );
        assert_eq!(lexer.next().unwrap(), int_pos(2, 1, 23));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::And, 1, 25));
        assert_eq!(lexer.next().unwrap(), ident_pos("e", 1, 28));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Or, 1, 30));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Not, 1, 33));
        assert_eq!(lexer.next().unwrap(), ident_pos("f", 1, 34));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }

    #[test]
    fn lexer_defaultops_pow_mod_and_neq() {
        let mut lexer = Lexer::default();

        lexer.set_text("z**2 % 5 != 4");
        assert_eq!(lexer.next().unwrap(), ident_pos("z", 1, 1));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Pow, 1, 2));
        assert_eq!(lexer.next().unwrap(), int_pos(2, 1, 4));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Mod, 1, 6));
        assert_eq!(lexer.next().unwrap(), int_pos(5, 1, 8));
        assert_eq!(lexer.next().unwrap(), op_pos(DefaultOperators::Neq, 1, 10));
        assert_eq!(lexer.next().unwrap(), int_pos(4, 1, 13));
        assert_eq!(lexer.next().unwrap(), TokenPos::eof());
    }
}
