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

    use crate::{DefaultOperators, Delimiter, Lexer, Operator, TextPos, Token, TokenPos};

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
}
