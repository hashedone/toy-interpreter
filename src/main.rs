mod combinators;
mod lexer;
mod context;
mod parser;

use std::io::{stdin, BufRead};
use std;

type Result<T> = std::result::Result<T, String>;

use lexer::{Operator, Token};
use context::Context;

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
                Ok(Some(val)) => println!("= {}", val),
                Ok(None) => println!("()"),
                Err(err) => println!("Error: {}", err),
            }
        });
}
