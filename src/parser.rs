use crate::{
    error::{ParserError, ParserResult},
    lexer::{Lexer, Operator, Token},
};

pub struct Parser {
    pub(crate) lexer: Lexer,
}

/// AST Output
#[derive(Debug, PartialEq)]
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

    pub fn parse(&mut self) -> ParserResult<Expr> {
        let expr = self.parse_expr(0)?;

        match self.peek() {
            Token::Eof => Ok(expr),
            t => Err(ParserError::UnexpectedToken(t))
        }
    }

    /// Parse the expression with the given minimum binding power
    ///
    /// The minimum binding power comes from the previous expression (recursive)
    fn parse_expr(&mut self, bp_min: u8) -> ParserResult<Expr> {
        // Parse the left hand side (lhs) of the expression
        let mut lhs = self.parse_prefix()?;

        // Get the right hand side (rhs)
        // Loop through and absorb infix operators that are greater than bp_min
        loop {
            // Get the infix operator. If it doesn't exist, break
            let op = match self.peek() {
                Token::Op(operator) => operator,
                _ => break,
            };

            // Check that the binding power is greater than bp_min
            let (bp_left, bp_right) = Self::infix_binding_power(op);
            if bp_left < bp_min {
                // If the left binding power of the current operator
                // is less than the current minimum bp, this operator
                // can't consume the expression so break
                break;
            }

            self.next(); // consume the operator
            let rhs = self.parse_expr(bp_right)?;

            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    /// Parses the prefix of an infix expression
    fn parse_prefix(&mut self) -> ParserResult<Expr> {
        match self.next() {
            Token::Int(n) => Ok(Expr::Int(n)),
            Token::Float(n) => Ok(Expr::Float(n)),
            Token::Ident(name) => self.var_or_call(name),
            Token::Op(op) if op.is_sub() => self.parse_unary_minus(),
            Token::LParen => self.parse_grouping(),
            t => Err(ParserError::UnexpectedPrefixToken(t)),
        }
    }

    fn parse_unary_minus(&mut self) -> ParserResult<Expr> {
        // The '-' was already consumed
        let rhs = self.parse_expr(5)?;

        Ok(Expr::Unary {
            op: Operator::Sub,
            operand: Box::new(rhs),
        })
    }

    fn parse_grouping(&mut self) -> ParserResult<Expr> {
        // Consume the interior of the parenthesis
        let interior = self.parse_expr(0)?;
        // Check for closing paren
        match self.next() {
            Token::RParen => Ok(interior),
            t => return Err(ParserError::MissingExprRParen(t)),
        }
    }

    fn var_or_call(&mut self, name: String) -> ParserResult<Expr> {
        // Check if the next character is a LParen
        match self.peek() {
            Token::LParen => {
                // this is a function call, consume the insides
                self.next(); // consume the LParen
                let mut args = vec![];
                loop {
                    args.push(self.parse_expr(0)?);
                    match self.next() {
                        Token::Comma => continue,
                        Token::RParen => break,
                        t => return Err(ParserError::MissingExprRParen(t)),
                    }
                }

                Ok(Expr::Call { func: name, args })
            }
            _ => Ok(Expr::Var(name)),
        }
    }

    fn infix_binding_power(op: Operator) -> (u8, u8) {
        match op {
            Operator::Add | Operator::Sub => (1, 2),
            Operator::Mul | Operator::Div => (3, 4),
            Operator::Pow => (8, 7), // right < left for this operator
        }
    }
}
