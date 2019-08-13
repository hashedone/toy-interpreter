use crate::{Context, Operator, Result, Token};
use std::any::Any;
use std::iter::Peekable;
use std::rc::Rc;

pub trait AST: std::fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    fn is_same(&self, other: &dyn AST) -> bool;
    fn arity(&self) -> usize {
        0
    }

    /// Used to return value if known without any context
    fn value(&self) -> Option<f32>;

    fn evaluate(&self, context: &mut Context, args: &[f32]) -> Option<f32>;
}

#[derive(Debug)]
enum Terminal {
    Value(f32), // Literal or substituted variable value
    Assign(String, Box<dyn AST>),
    Argument(usize), // Function argument of given index
}

#[derive(Debug)]
struct OpExpr {
    op: Operator,
    left: Box<dyn AST>,
    right: Box<dyn AST>,
}

#[derive(Debug)]
struct CallExpr {
    func: Rc<dyn AST>,
    args: Vec<Box<dyn AST>>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub expr: Rc<dyn AST>,
}

impl AST for Terminal {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_same(&self, other: &dyn AST) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |o| match (self, o) {
                (Terminal::Value(x), Terminal::Value(y)) => (x - y).abs() < 0.001,
                (Terminal::Assign(v1, val1), Terminal::Assign(v2, val2)) => {
                    v1 == v2 && val1.is_same(val2.as_ref())
                }
                _ => false,
            })
    }

    fn value(&self) -> Option<f32> {
        match self {
            Terminal::Value(v) => Some(*v),
            Terminal::Assign(_, _) => None,
            Terminal::Argument(_) => None,
        }
    }

    fn evaluate(&self, context: &mut Context, args: &[f32]) -> Option<f32> {
        match self {
            Terminal::Value(v) => Some(*v),
            Terminal::Assign(var, val) => {
                let val = val.evaluate(context, args)?;
                context.update_var(var, val);
                Some(val)
            }
            Terminal::Argument(arg) => args.get(*arg).cloned(),
        }
    }
}

impl AST for OpExpr {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_same(&self, other: &dyn AST) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.op == other.op
                && self.left.is_same(other.left.as_ref())
                && self.right.is_same(other.right.as_ref())
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

    fn evaluate(&self, context: &mut Context, args: &[f32]) -> Option<f32> {
        let (left, right) = (
            self.left.evaluate(context, args)?,
            self.right.evaluate(context, args)?,
        );

        Some(self.op.eval(left, right))
    }
}

impl AST for CallExpr {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_same(&self, other: &dyn AST) -> bool {
        other.as_any().downcast_ref::<Self>().is_some()
    }

    fn value(&self) -> Option<f32> {
        None
    }

    fn evaluate(&self, context: &mut Context, args: &[f32]) -> Option<f32> {
        let args: Option<Vec<_>> = self
            .args
            .iter()
            .map(|arg| arg.evaluate(context, args))
            .collect();
        let args = args?;

        self.func.evaluate(context, &args)
    }
}

impl AST for Function {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_same(&self, other: &dyn AST) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.name == other.name
                && self.arity == other.arity
                && self.expr.is_same(other.expr.as_ref())
        } else {
            false
        }
    }

    fn value(&self) -> Option<f32> {
        None
    }

    fn evaluate(&self, context: &mut Context, _args: &[f32]) -> Option<f32> {
        context.update_func(self);
        None
    }
}

impl Terminal {
    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
        match tokens.next() {
            Some(Token::Number(x)) => Ok(Box::new(Terminal::Value(x))),
            Some(Token::LBracket) => {
                tokens.next();
                let expr = OpExpr::parse(tokens, context)?;
                if let Some(Token::RBracket) = tokens.peek() {
                    tokens.next();
                    Ok(expr)
                } else {
                    Err(format!("Invalid token {:?}, expected `)`", tokens.next()))
                }
            }
            Some(Token::Assign(var)) => {
                if context.is_var(&var) {
                    let expr = CallExpr::parse(tokens, context)?;
                    Ok(Box::new(Terminal::Assign(var, expr)))
                } else {
                    Err(format!(
                        "Assigning to symbol which is not variable: {}",
                        var
                    ))
                }
            }
            Some(Token::Id(var)) => {
                if let Some(var) = context.get_var(&var) {
                    Ok(Box::new(Terminal::Value(var)))
                } else if let Some(var) = context.get_arg(&var) {
                    Ok(Box::new(Terminal::Argument(var)))
                } else {
                    Err(format!(
                        "Non variable symbol as terminal token occured: {}",
                        var
                    ))
                }
            }
            Some(token) => Err(format!(
                "Unexpected token while parsing terminal expression: {:?}",
                token
            )),
            None => {
                Err("Unexpected end of tokens list while parsing terminal expression".to_owned())
            }
        }
    }
}

impl OpExpr {
    fn get_next_multiplicative(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Option<Operator> {
        match tokens.peek() {
            Some(Token::Operator(Operator::Mul)) => {
                tokens.next();
                Some(Operator::Mul)
            }
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
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
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

    fn get_next_additive(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Option<Operator> {
        match tokens.peek() {
            Some(Token::Operator(Operator::Add)) => {
                tokens.next();
                Some(Operator::Add)
            }
            Some(Token::Operator(Operator::Sub)) => {
                tokens.next();
                Some(Operator::Sub)
            }
            _ => None,
        }
    }

    fn parse_additive(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
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
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
        Self::parse_additive(tokens, context)
    }
}

impl CallExpr {
    fn get_func(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Option<String> {
        if let Some(Token::Id(f)) = tokens.peek() {
            if context.is_func(f) {
                let name = f.clone();
                tokens.next();
                Some(name)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
        if let Some(name) = Self::get_func(tokens, context) {
            let arity = context.get_arity(&name).unwrap_or(0);
            let func = context
                .get_func(&name)
                .ok_or_else(|| format!("No function named {}", name))?;

            let mut args = vec![];
            for _ in 0..arity {
                let arg = CallExpr::parse(tokens, context)?;
                args.push(arg);
            }

            Ok(Box::new(CallExpr { func, args }))
        } else {
            OpExpr::parse(tokens, context)
        }
    }
}

impl Function {
    fn get_id(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Option<String> {
        match tokens.peek() {
            Some(Token::Id(id)) => {
                let id = id.clone();
                tokens.next();
                Some(id)
            }
            _ => None,
        }
    }

    fn parse(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        context: &Context,
    ) -> Result<Box<dyn AST>> {
        let name = Self::get_id(tokens).ok_or_else(|| format!(
            "Expected function name, but got: {:?}",
            tokens.peek()
        ))?;

        if !context.is_func(&name) {
            return Err(format!(
                "Expected function name, but got not function id: {}",
                name
            ));
        }

        let mut args = vec![];
        while let Some(arg) = Self::get_id(tokens) {
            args.push(arg.clone());
        }

        if tokens.next() != Some(Token::Func) {
            return Err("Expected => token".to_string());
        }

        let arity = args.len();
        let ctx = Context::function_ctx(args, context);
        let expr = CallExpr::parse(tokens, &ctx)?.into();

        Ok(Box::new(Function { name, arity, expr }))
    }
}

impl Context {
    pub fn parse(&self, tokens: impl Iterator<Item = Token>) -> Result<Box<dyn AST>> {
        let tokens: Vec<_> = tokens.collect();

        if tokens.contains(&Token::Func) {
            Function::parse(&mut tokens.into_iter().peekable(), self)
        } else {
            CallExpr::parse(&mut tokens.into_iter().peekable(), self)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn tokenize<'a>(src: &'a str) -> Peekable<impl Iterator<Item = Token> + 'a> {
        use crate::lexer::tokenize;

        tokenize(src).map(|t| t.unwrap()).peekable()
    }

    #[test]
    fn test_terminal_number() {
        let number = Terminal::parse(&mut tokenize("10"), &Context::new()).unwrap();
        let expected = Terminal::Value(10.0);
        assert!(expected.is_same(number.as_ref()));
    }

    #[test]
    fn test_terminal_assignment() {
        let assign = Terminal::parse(&mut tokenize("a = 10 + 2"), &Context::new()).unwrap();
        let expected = Terminal::Assign("a".to_string(), Box::new(Terminal::Value(12.0)));
        assert!(expected.is_same(assign.as_ref()));

        let assign = OpExpr::parse(&mut tokenize("2 + a = 10"), &Context::new()).unwrap();
        let expected = OpExpr {
            op: Operator::Add,
            left: Box::new(Terminal::Value(2.0)),
            right: Box::new(Terminal::Assign(
                "a".to_string(),
                Box::new(Terminal::Value(10.0)),
            )),
        };
        assert!(expected.is_same(assign.as_ref()));
    }

    #[test]
    fn text_op_expr_mul() {
        let expr = OpExpr::parse_multiplicative(&mut tokenize("10"), &Context::new()).unwrap();
        let expected = Terminal::Value(10.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_multiplicative(&mut tokenize("10 * 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(20.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_multiplicative(&mut tokenize("10 / 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(5.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_multiplicative(&mut tokenize("10 % 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(0.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr =
            OpExpr::parse_multiplicative(&mut tokenize("11 % 2 * 5 / 3"), &Context::new()).unwrap();

        let expected = Terminal::Value(5.0f32 / 3.0f32);
        assert!(expected.is_same(expr.as_ref()));
    }

    #[test]
    fn text_op_expr_add() {
        let expr = OpExpr::parse_additive(&mut tokenize("10"), &Context::new()).unwrap();
        let expected = Terminal::Value(10.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_additive(&mut tokenize("10 + 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(12.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_additive(&mut tokenize("10 - 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(8.0);
        assert!(expected.is_same(expr.as_ref()));

        let expr = OpExpr::parse_additive(&mut tokenize("11 + 2 - 5"), &Context::new()).unwrap();

        let expected = Terminal::Value(8.0f32);
        assert!(expected.is_same(expr.as_ref()));

        let expr =
            OpExpr::parse_additive(&mut tokenize("10 * 3 - 6 / 2"), &Context::new()).unwrap();

        let expected = Terminal::Value(27.0);;
        assert!(expected.is_same(expr.as_ref()));
    }
}
