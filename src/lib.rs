mod error;
mod lexer;
mod parser;

pub use error::*;
pub use lexer::*;
pub use parser::*;

#[cfg(test)]
mod tests {
    use crate::{
        Expr,
        error::ParserError,
        lexer::{Lexer, Operator, Token},
        parse,
    };

    #[test]
    fn lexer_malformed_number() {
        let err = Lexer::new("1 + 3.4.5 + 10").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber {
                pos: 4,
                str: "3.4.5".into()
            }
        );

        let err = Lexer::new("1 + 100.4124.51244 + 10").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber {
                pos: 4,
                str: "100.4124.51244".into()
            }
        );

        let err = Lexer::new("1 + 100.4124.51244").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber {
                pos: 4,
                str: "100.4124.51244".into()
            }
        );

        let err = Lexer::new("100.4124.51244").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber {
                pos: 0,
                str: "100.4124.51244".into()
            }
        );
    }

    #[test]
    fn lexer_unexpected_char() {
        let err = Lexer::new("$").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar { pos: 0, c: '$' });

        let err = Lexer::new("5 + .12").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar { pos: 4, c: '.' });

        let err = Lexer::new("abs(Z)^2 ß 10 + C").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar { pos: 9, c: 'ß' });

        let err = Lexer::new("5 + 10a/2").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar { pos: 6, c: 'a' });
    }

    #[test]
    fn lexer_next() {
        let mut lexer = Lexer::new("abs(Z)^2 + C").expect("the formula is formatted correctly");

        assert_eq!(Token::Ident("abs".into()), lexer.next().token);
        assert_eq!(Token::LParen, lexer.next().token);
        assert_eq!(Token::Ident("Z".into()), lexer.next().token);
        assert_eq!(Token::RParen, lexer.next().token);
        assert_eq!(Token::Op(Operator::Pow), lexer.next().token);
        assert_eq!(Token::Int(2), lexer.next().token);
        assert_eq!(Token::Op(Operator::Add), lexer.next().token);
        assert_eq!(Token::Ident("C".into()), lexer.next().token);
        assert_eq!(Token::Eof, lexer.next().token);
    }

    #[test]
    fn lexer_peek() {
        let mut lexer = Lexer::new("abs(Z)^2 + C").expect("the formula is formatted correctly");

        assert_eq!(Token::Ident("abs".into()), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::LParen, lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Ident("Z".into()), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::RParen, lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Op(Operator::Pow), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Int(2), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Op(Operator::Add), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Ident("C".into()), lexer.peek().token);
        lexer.next();
        assert_eq!(Token::Eof, lexer.peek().token);
    }

    #[test]
    fn parser_builds() {
        let ast = parse("Z^2 + C").unwrap();
        let expected = Expr::Binary {
            op: Operator::Add,
            lhs: Box::new(Expr::Binary {
                op: Operator::Pow,
                lhs: Box::new(Expr::Var("Z".into())),
                rhs: Box::new(Expr::Int(2)),
            }),
            rhs: Box::new(Expr::Var("C".into())),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn parser_bad_prefix() {
        let err = parse("*10 + 5").err().expect("bad format");
        let expected_err = ParserError::UnexpectedPrefixToken(Token::Op(Operator::Mul));
        assert_eq!(err, expected_err);

        let err = parse("9 * +10 + 5").err().expect("bad format");
        let expected_err = ParserError::UnexpectedPrefixToken(Token::Op(Operator::Add));
        assert_eq!(err, expected_err);

        let err = parse("abs(, 10)").err().expect("bad format");
        let expected_err = ParserError::UnexpectedPrefixToken(Token::Comma);
        assert_eq!(err, expected_err);

        let err = parse("abs(10,)").err().expect("bad format");
        let expected_err = ParserError::UnexpectedToken(Token::Comma);
        assert_eq!(err, expected_err);

        let err = parse("10 + 8)").err().expect("bad format");
        let expected_err = ParserError::UnexpectedToken(Token::RParen);
        assert_eq!(err, expected_err);

        assert!(parse("-10 + 5").is_ok());
        assert!(parse("-10^(-15 * 4.23) + -Z").is_ok());
    }

    #[test]
    fn parser_reached_eof() {
        let err = parse("10 + ").err().expect("bad format");
        let expected_err = ParserError::ReachedEOF;
        assert_eq!(err, expected_err);

        let err = parse("abs(z^2) + c / ").err().expect("bad format");
        let expected_err = ParserError::ReachedEOF;
        assert_eq!(err, expected_err);
    }

    #[test]
    fn parser_reached_eof_args() {
        let err = parse("function(").err().expect("bad format");
        let expected_err = ParserError::ReachedEOFArgs;
        assert_eq!(err, expected_err);

        let err = parse("10 + abs(-Z * a").err().expect("bad format");
        let expected_err = ParserError::ReachedEOFArgs;
        assert_eq!(err, expected_err);

        let err = parse("10 + comp(z, c + 10").err().expect("bad format");
        let expected_err = ParserError::ReachedEOFArgs;
        assert_eq!(err, expected_err);
    }

    #[test]
    fn parser_parens() {
        let err = parse("(a + b").err().expect("bad format");
        let expected_err = ParserError::MissingExprRParen(Token::Eof);
        assert_eq!(err, expected_err);

        let err = parse("(2 * 3 -) (a + b)").err().expect("bad format");
        let expected_err = ParserError::UnexpectedToken(Token::RParen);
        assert_eq!(err, expected_err);

        let err = parse("((((a + b * 10 ))").err().expect("bad format");
        let expected_err = ParserError::MissingExprRParen(Token::Eof);
        assert_eq!(err, expected_err);

        assert!(parse("((((10 + 594 * z^2 + abs(4)))))").is_ok());
        assert!(parse("((2)) - (((5)) * (3 + (4 * 10)))").is_ok());
    }

    #[test]
    fn op_precidence() {
        let expr = parse("1 + 2 * 3").expect("correct format");
        let expected_expr = Expr::Binary {
            op: Operator::Add,
            lhs: Box::new(Expr::Int(1)),
            rhs: Box::new(Expr::Binary {
                op: Operator::Mul,
                lhs: Box::new(Expr::Int(2)),
                rhs: Box::new(Expr::Int(3)),
            }),
        };
        assert_eq!(expr, expected_expr);

        let expr = parse("1 * 2^3").expect("correct format");
        let expected_expr = Expr::Binary {
            op: Operator::Mul,
            lhs: Box::new(Expr::Int(1)),
            rhs: Box::new(Expr::Binary {
                op: Operator::Pow,
                lhs: Box::new(Expr::Int(2)),
                rhs: Box::new(Expr::Int(3)),
            }),
        };
        assert_eq!(expr, expected_expr);

        let expr = parse("1 * -2^3").expect("correct format");
        let expected_expr = Expr::Binary {
            op: Operator::Mul,
            lhs: Box::new(Expr::Int(1)),
            rhs: Box::new(Expr::Unary {
                op: Operator::Sub,
                operand: Box::new(Expr::Binary {
                    op: Operator::Pow,
                    lhs: Box::new(Expr::Int(2)),
                    rhs: Box::new(Expr::Int(3)),
                }),
            }),
        };
        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn expr_print() {
        let infix_not = parse("1 + 2 * 3").unwrap().infix_notation();
        assert_eq!(infix_not, "(1 + (2 * 3))".to_string());

        let infix_not = parse("-10^(-15 * 4.23) + -Z * 4").unwrap().infix_notation();
        assert_eq!(infix_not, "(-(10 ^ (-15 * 4.23)) + (-Z * 4))".to_string());
    }
}

