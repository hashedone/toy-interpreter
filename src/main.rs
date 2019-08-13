mod statement;
mod combinators;
mod lexer;
mod context;
mod parser;

use statement::Statement;
use std::io::{stdin, BufRead};
use std;

type Error = String;
type Result<T> = std::result::Result<T, String>;

use lexer::{Operator, Token};
use context::Context;

#[derive(Debug, PartialEq)]
pub struct Assignment {
    var: String,
    val: f32, // TODO: This should be actually expression
}

impl Assignment {
    fn new(var: impl ToString, val: f32) -> Self {
        Self {
            var: var.to_string(),
            val,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Factor {
    Expression, // Actually bracket expression
    Number(f32),
    Ident(String),
    Assignment(Assignment),
}

fn run(line: &str, context: &mut Context) -> Result<Option<f32>> {
    let tokens: Result<Vec<_>> = lexer::tokenize(line).collect();
    let tokens = tokens?.into_iter();
    Ok(context.parse(tokens)?.evaluate(context, &[]))
}

fn main() {
    let mut context = Context::new();
    stdin()
        .lock()
        .lines()
        .filter_map(|line| line.ok()) // Actually ignoring iostream errors
        .for_each(|line| {
            match run(&line, &mut context) {
                Ok(Some(val)) => println!("{}", val),
                Ok(None) => println!("()"),
                Err(err) => println!("Error: {}", err),
            }
        });
}
