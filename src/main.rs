mod statement;
mod combinators;
mod lexer;

use statement::Statement;
use std::io::{stdin, BufRead};
use std;

type Error = String;
type Result<T> = std::result::Result<T, String>;

pub use lexer::{Operator, Token};

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

fn main() {
    stdin()
        .lock()
        .lines()
        .filter_map(|line| line.ok()) // Actually ignoring iostream errors
        .for_each(|line| {
            println!("= {:?}", line)
        });
}
