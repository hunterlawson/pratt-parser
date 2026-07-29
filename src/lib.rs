mod error;
mod lexer;

pub use error::*;

#[cfg(test)]
mod tests {
    use strum::{Display, EnumIter};

    use crate::lexer::{
        Lexer,
        default_types::{DefaultOperators, Delimited, Operator},
        token::{TextPosition, Token, TokenPos},
    };

    #[derive(Clone, PartialEq, EnumIter, Debug)]
    enum TestDelimiter {
        Str(String),
        Arr(String),
    }

    impl Delimited for TestDelimiter {
        fn delimiters(&self) -> Option<(String, String)> {
            Some(match self {
                TestDelimiter::Str(_) => ("\"".into(), "\"".into()),
                TestDelimiter::Arr(_) => ("[".into(), "]".into()),
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

    fn print_next_token<O: Operator, D: Delimited>(lexer: &mut Lexer<O, D>) {
        let res = lexer.next();
        match res {
            Ok(t) => match t.pos {
                Some(p) => println!("{:?} at ({})", t.token, p),
                None => println!("{:?}", t.token),
            },
            Err(e) => println!("{e}"),
        }
    }

    fn op_pos<O: Operator, D: Delimited>(op: O, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Op(op), TextPosition { line, col })
    }

    fn ident_pos<O: Operator, D: Delimited>(s: &str, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Ident(s.into()), TextPosition { line, col })
    }

    fn int_pos<O: Operator, D: Delimited>(i: i64, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Int(i), TextPosition { line, col })
    }

    fn float_pos<O: Operator, D: Delimited>(f: f64, line: usize, col: usize) -> TokenPos<O, D> {
        TokenPos::new(Token::Float(f), TextPosition { line, col })
    }

    #[test]
    fn lexer_lex_testoperators_simple() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        // println!("{lexer:#?}");

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
    fn lexer_lex_testoperators_complex() {
        let mut lexer = Lexer::<TestOperator, TestDelimiter>::new();

        // println!("{lexer:#?}");

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
}
