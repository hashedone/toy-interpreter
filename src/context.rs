use crate::Result;
use crate::lexer::{Operator, Token};

pub struct Context;

#[derive(Debug, PartialEq)]
enum Value {
    Number(f32), // Literal or value of variable
    Placeholder(usize), // Function arg, `usize` is index of argument
}

#[derive(Debug, PartialEq)]
enum Expression {
    Value(Value),
    Op(Operator, Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Top level perator priority:
    /// no operator (also bracketed) = 0
    /// Add/Sub = 1
    /// Mul/Div/Mod = 2
    fn priority(&self) -> u8 {
        match self {
            Expression::Value(_) => 0,
            Expression::Op(op, _, _) => op.priority(),
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Context
    }

    fn parse_op_expression(
        &self,
        tokens: impl Iterator<Item=Token>,
        priority: u8, // Expression should have at most given priority
        _args: &[String], // Argument names for function
    )
        -> Result<(Option<Expression>, impl Iterator<Item=Token>)>
    {
        let mut tokens = tokens.peekable();
        if priority == 0 {
            // Only literal, variable or assignment
            match tokens.peek() {
                Some(Token::Number(x)) => {
                    let x = *x;
                    tokens.next();
                    Ok((
                        Some(Expression::Value(Value::Number(x))),
                        tokens
                    ))
                }
                _ => Ok((None, tokens))
            }
        } else {
            Ok((None, tokens))
        }
    }
}

#[cfg(test)]
mod test {

use super::*;
use crate::lexer::tokenize;

#[test]
fn number_as_expression() {
    let tokens = tokenize("10").map(|t| t.unwrap());
    let (expr, mut tail) = Context::new()
        .parse_op_expression(tokens, 0, &[])
        .unwrap();
    let expected = Expression::Value(Value::Number(10.0));

    assert_eq!(Some(expected), expr);
    assert_eq!(None, tail.next());
}

}
