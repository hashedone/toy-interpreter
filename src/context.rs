use crate::lexer::Operator;
use crate::parser::{AST, Function};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
enum Symbol {
    Variable(f32),
    Function(usize, Rc<dyn AST>),
    Argument(usize),
}

impl Symbol {
    fn is_var(&self) -> bool {
        match self {
            Symbol::Variable(_) => true,
            _ => false,
        }
    }

    fn is_func(&self) -> bool {
        match self {
            Symbol::Function(_, _) => true,
            _ => false,
        }
    }
}

pub struct Context {
    symbols: HashMap<String, Symbol>,
}

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
        Context {
            symbols: HashMap::new(),
        }
    }

    pub fn function_ctx(args: Vec<String>, parent: &Context) -> Self {
        let functions = parent.symbols.iter()
            .filter(|(_, item)| item.is_func())
            .map(|(name, item)| (name.clone(), item.clone()));

        let args = args.into_iter()
            .enumerate()
            .map(|(idx, var)| (var, Symbol::Argument(idx)));

        let symbols = functions.chain(args).collect();

        Self { symbols }
    }

    pub fn update_var(&mut self, var: impl ToString, val: f32) {
        self.symbols.entry(var.to_string())
            .and_modify(|v| match v {
                Symbol::Variable(ref mut v) => {
                     *v = val;
                }
                _ => (),
            })
            .or_insert(Symbol::Variable(val));
    }

    pub fn update_func(&mut self, func: &Function) {
        self.symbols.entry(func.name.clone())
            .and_modify(|v| match v {
                Symbol::Function(ref mut arity, ref mut expr) => {
                    *arity = func.arity;
                    *expr = func.expr.clone();
                },
                _ => (),
            })
            .or_insert_with(|| Symbol::Function(
                func.arity,
                func.expr.clone()
            ));
    }

    pub fn is_var(&self, var: &str) -> bool {
        self.symbols.get(var).map_or(true, Symbol::is_var)
    }

    pub fn is_func(&self, var: &str) -> bool {
        self.symbols.get(var).map_or(true, Symbol::is_func)
    }

    pub fn get_var(&self, var: &str) -> Option<f32> {
        match self.symbols.get(var)? {
            Symbol::Variable(v) => Some(*v),
            _ => None,
        }
    }

    pub fn get_arg(&self, var: &str) -> Option<usize> {
        match self.symbols.get(var)? {
            Symbol::Argument(idx) => Some(*idx),
            _ => None,
        }
    }

    pub fn get_arity(&self, var: &str) -> Option<usize> {
        match self.symbols.get(var)? {
            Symbol::Function(arity, _) => Some(*arity),
            _ => None,
        }
    }

    pub fn get_func(&self, var: &str) -> Option<Rc<dyn AST>> {
        match self.symbols.get(var)? {
            Symbol::Function(_, expr) => Some(expr.clone()),
            _ => None,
        }
    }
}


