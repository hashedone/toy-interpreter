mod statement;
mod combinators;

use statement::Statement;
use std::io::{stdin, BufRead};
use std;

type Error = String;
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

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
    Assignment(String, ()), // TODO: second one is expression
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
