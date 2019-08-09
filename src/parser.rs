use std::iter::Peekable;
use std::any::Any;
use crate::{Token, Context, Result, Operator};

pub trait AST: std::fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    fn is_same(&self, other: &dyn AST) -> bool;

    /// Used to return value if known without any context
    fn value(&self) -> Option<f32>;
}

#[derive(Debug, PartialEq)]
enum Terminal {
    Value(f32), // Literal or substituted variable value
}

#[derive(Debug)]
struct OpExpr {
    op: Operator,
    left: Box<dyn AST>,
    right: Box<dyn AST>
}

impl AST for Terminal {
    fn as_any(&self) -> &dyn Any { self }

    fn is_same(&self, other: &dyn AST) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self == o)
    }

    fn value(&self) -> Option<f32> {
        match self {
            Terminal::Value(v) => Some(*v),
        }
    }

}

impl AST for OpExpr {
    fn as_any(&self) -> &dyn Any { self }

    fn is_same(&self, other: &dyn AST) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.op == other.op &&
            self.left.is_same(other.left.as_ref()) &&
            self.right.is_same(other.right.as_ref())
        } else {
            false
        }
    }

    fn value(&self) -> Option<f32> {
        let (left, right) = (self.left.value(), self.right.value());
        if let (Some(left), Some(right)) = (left, right) {
            Some(self.op.eval(left, right))
        } else {
            None
        }
    }
}

impl Terminal {
    fn parse(
        tokens: &mut Peekable<impl Iterator<Item=Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>>
    {
        match tokens.peek() {
            Some(Token::Number(x)) => {
                let x = *x;
                tokens.next();
                Ok(Box::new(Terminal::Value(x)))
            },
            Some(Token::LBracket) => {
                tokens.next();
                let expr = OpExpr::parse(tokens, context)?;
                if let Some(Token::RBracket) = tokens.peek() {
                    tokens.next();
                    Ok(expr)
                } else {
                    Err(format!(
                        "Invalid token {:?}, expected `)`",
                        tokens.next()
                    ))
                }
            },
            Some(token) => {
                Err(format!(
                    "Unexpected token while parsing terminal expression: {:?}",
                    token
                ))
            }
            None => {
                Err("Unexpected end of tokens list while parsing terminal expression".to_owned())
            }
        }
    }
}

impl OpExpr {
    fn get_next_multiplicative(
        tokens: &mut Peekable<impl Iterator<Item=Token>>
    )
        -> Option<Operator>
    {
        match tokens.peek() {
            Some(Token::Operator(Operator::Mul)) => {
                tokens.next();
                Some(Operator::Mul)
            },
            Some(Token::Operator(Operator::Div)) => {
                tokens.next();
                Some(Operator::Div)
            }
            Some(Token::Operator(Operator::Mod)) => {
                tokens.next();
                Some(Operator::Mod)
            }
            _ => None,
        }
    }

    fn parse_multiplicative(
        tokens: &mut Peekable<impl Iterator<Item=Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>>
    {
        let mut result = Terminal::parse(tokens, context)?;

        while let Some(op) = Self::get_next_multiplicative(tokens) {
            let right = Terminal::parse(tokens, context)?;
            result = Box::new(OpExpr {
                op,
                left: result,
                right,
            });

            if let Some(val) = result.value() {
                result = Box::new(Terminal::Value(val))
            }
        }

        Ok(result)
    }

    fn get_next_additive(
        tokens: &mut Peekable<impl Iterator<Item=Token>>
    )
        -> Option<Operator>
    {
        match tokens.peek() {
            Some(Token::Operator(Operator::Add)) => {
                tokens.next();
                Some(Operator::Add)
            },
            Some(Token::Operator(Operator::Sub)) => {
                tokens.next();
                Some(Operator::Sub)
            }
            _ => None,
        }
    }

    fn parse_additive(
        tokens: &mut Peekable<impl Iterator<Item=Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>>
    {
        let mut result = Self::parse_multiplicative(tokens, context)?;

        while let Some(op) = Self::get_next_additive(tokens) {
            let right = Self::parse_multiplicative(tokens, context)?;
            result = Box::new(OpExpr {
                op,
                left: result,
                right,
            });

            if let Some(val) = result.value() {
                result = Box::new(Terminal::Value(val))
            }
        }

        Ok(result)
    }

    fn parse(
        tokens: &mut Peekable<impl Iterator<Item=Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>>
    {
        Self::parse_additive(tokens, context)
    }
}

impl Context {
    pub fn parse(&self, tokens: impl Iterator<Item=Token>)
        -> Result<Box<dyn AST>>
    {
        let mut tokens = tokens.peekable();
        let result = OpExpr::parse_additive(&mut tokens, self)?;

        if tokens.peek().is_some() {
            let err = format!(
                "Not whole input consumed, left tokens: {:?}",
                tokens.collect::<Vec<_>>()
            );
            return Err(err);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {

use super::*;

fn tokenize<'a>(src: &'a str) -> Peekable<impl Iterator<Item=Token> + 'a> {
    use crate::lexer::tokenize;

    tokenize(src)
        .map(|t| t.unwrap())
        .peekable()
}

#[test]
fn test_terminal_number() {
    let number = Terminal::parse(&mut tokenize("10"), &Context::new()).unwrap();
    let expected = Terminal::Value(10.0);
    assert!(expected.is_same(number.as_ref()));
}

#[test]
fn text_op_expr_mul() {
    let expr = OpExpr::parse_multiplicative(
        &mut tokenize("10"),
        &Context::new()
    ).unwrap();
    let expected = Terminal::Value(10.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_multiplicative(
        &mut tokenize("10 * 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(20.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_multiplicative(
        &mut tokenize("10 / 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(5.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_multiplicative(
        &mut tokenize("10 % 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(0.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_multiplicative(
        &mut tokenize("11 % 2 * 5 / 3"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(5.0f32 / 3.0f32);
    assert!(expected.is_same(expr.as_ref()));
}

#[test]
fn text_op_expr_add() {
    let expr = OpExpr::parse_additive(
        &mut tokenize("10"),
        &Context::new()
    ).unwrap();
    let expected = Terminal::Value(10.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_additive(
        &mut tokenize("10 + 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(12.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_additive(
        &mut tokenize("10 - 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(8.0);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_additive(
        &mut tokenize("11 + 2 - 5"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(8.0f32);
    assert!(expected.is_same(expr.as_ref()));

    let expr = OpExpr::parse_additive(
        &mut tokenize("10 * 3 - 6 / 2"),
        &Context::new()
    ).unwrap();

    let expected = Terminal::Value(27.0);;
    assert!(expected.is_same(expr.as_ref()));
}
}
