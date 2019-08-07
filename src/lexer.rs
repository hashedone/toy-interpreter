use crate::combinators::next_token;
use std::iter;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Id(String),
    Number(f32),
    Operator(Operator),
    LBracket,
    RBracket,
    Assign,
    Func, // =>
}

fn tokenize<'a>(mut src: &'a str)
    -> impl Iterator<Item=Result<Token, String>> + 'a
{
    iter::from_fn(move || {
        match next_token(src) {
            Ok(progress) => {
                src = progress.tail;
                progress.token.map(|token| Ok(token))
            },
            Err(err) => {
                src = "";
                Some(Err(err))
            }
        }

    })
}
