mod combinators;
mod context;
mod lexer;
mod parser;

use std;
use std::io::{stdin, BufRead};

type Result<T> = std::result::Result<T, String>;

use context::Context;
use lexer::{Operator, Token};

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
        .for_each(|line| match run(&line, &mut context) {
            Ok(Some(val)) => println!("= {}", val),
            Ok(None) => println!("()"),
            Err(err) => println!("Error: {}", err),
        });
}
