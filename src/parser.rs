use crate::{
    error::{ParserError, ParserResult}, lexer::{Lexer, Operator, Token},
};

pub struct Parser {
    pub(crate) lexer: Lexer,
}

/// AST Output
#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Var(String),
    Unary {
        op: Operator,
        operand: Box<Expr>,
    },
    Binary {
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
}

impl Parser {
    pub fn new(expr: &str) -> ParserResult<Self> {
        Ok(Self {
            lexer: Lexer::new(expr)?,
        })
    }

    fn next(&mut self) -> Token {
        self.lexer.next()
    }

    fn peek(&self) -> Token {
        self.lexer.peek()
    }

    fn parse_expr(&mut self, min_bp: u8) -> ParserResult<Expr> {
        todo!();
    }

    fn parse_prefix(&mut self) -> ParserResult<Expr> {
        match self.next() {
            Token::Int(n) => Ok(Expr::Int(n)),
            Token::Float(n) => Ok(Expr::Float(n)),
            Token::Ident(name) => self.ident_or_call(name),
            Token::Op(op) if op.is_sub() => self.parse_unary_minus(),
            Token::LParen => self.parse_grouping(),
            t => Err(ParserError::UnexpectedPrefixToken(t))
        }
    }

    fn parse_unary_minus(&mut self) -> ParserResult<Expr> {
        todo!();
    }

    fn parse_grouping(&mut self) -> ParserResult<Expr> {
        todo!();
    }

    fn ident_or_call(&mut self, name: String) -> ParserResult<Expr> {
        todo!();
    }

    fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
        match op {
            Operator::Add | Operator::Sub => Some((1, 2)),
            Operator::Mul | Operator::Div => Some((3, 4)),
            Operator::Pow => Some((8, 7)), // right < left for this operator
            _ => None,
        }
    }
}

