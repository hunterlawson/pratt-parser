mod error;
mod lexer;
mod parser;

pub use parser::*;

#[cfg(test)]
mod tests {
    use crate::{
        Expr,
        error::ParserError,
        lexer::{Operator, Token},
        parser::Parser,
    };

    #[test]
    fn lexer_malformed_number() {
        let err = Parser::new("1 + 3.4.5 + 10").err().unwrap();
        assert_eq!(err, ParserError::MalformedNumber(4, "3.4.5".into()));

        let err = Parser::new("1 + 100.4124.51244 + 10").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber(4, "100.4124.51244".into())
        );

        let err = Parser::new("1 + 100.4124.51244").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber(4, "100.4124.51244".into())
        );

        let err = Parser::new("100.4124.51244").err().unwrap();
        assert_eq!(
            err,
            ParserError::MalformedNumber(0, "100.4124.51244".into())
        );
    }

    #[test]
    fn lexer_unexpected_char() {
        let err = Parser::new("$").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar(0, '$'));

        let err = Parser::new("5 + .12").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar(4, '.'));

        let err = Parser::new("abs(Z)^2 ß 10 + C").err().unwrap();
        assert_eq!(err, ParserError::UnexpectedChar(9, 'ß'));
    }

    #[test]
    fn lexer_next() {
        let mut lexer = Parser::new("abs(Z)^2 + C")
            .expect("the formula is formatted correctly")
            .lexer;

        assert_eq!(Token::Ident("abs".into()), lexer.next());
        assert_eq!(Token::LParen, lexer.next());
        assert_eq!(Token::Ident("Z".into()), lexer.next());
        assert_eq!(Token::RParen, lexer.next());
        assert_eq!(Token::Op(Operator::Pow), lexer.next());
        assert_eq!(Token::Int(2), lexer.next());
        assert_eq!(Token::Op(Operator::Add), lexer.next());
        assert_eq!(Token::Ident("C".into()), lexer.next());
        assert_eq!(Token::Eof, lexer.next());
    }

    #[test]
    fn lexer_peek() {
        let mut lexer = Parser::new("abs(Z)^2 + C")
            .expect("the formula is formatted correctly")
            .lexer;

        assert_eq!(Token::Ident("abs".into()), lexer.peek());
        lexer.next();
        assert_eq!(Token::LParen, lexer.peek());
        lexer.next();
        assert_eq!(Token::Ident("Z".into()), lexer.peek());
        lexer.next();
        assert_eq!(Token::RParen, lexer.peek());
        lexer.next();
        assert_eq!(Token::Op(Operator::Pow), lexer.peek());
        lexer.next();
        assert_eq!(Token::Int(2), lexer.peek());
        lexer.next();
        assert_eq!(Token::Op(Operator::Add), lexer.peek());
        lexer.next();
        assert_eq!(Token::Ident("C".into()), lexer.peek());
        lexer.next();
        assert_eq!(Token::Eof, lexer.peek());
    }

    #[test]
    fn parser_builds() {
        let ast = Parser::new("Z^(-2) + C * 2").unwrap().parse().unwrap();

        println!("{ast:#?}");

        assert_eq!(format!("{ast:#?}"), format!("{:?}", Expr::Var("Z".into())));
    }
}
