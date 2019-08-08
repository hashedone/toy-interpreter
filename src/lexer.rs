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
    Assign(String), // Assignment is actually bitoken including variable which is assigned to
    Func, // =>
}

pub fn tokenize<'a>(mut src: &'a str)
    -> impl Iterator<Item=Result<Token, String>> + 'a
{
    iter::from_fn(move || {
        match next_token(src) {
            Ok(progress) => {
                src = progress.tail.trim_start();
                progress.token.map(|token| Ok(token))
            },
            Err(err) => {
                src = "";
                Some(Err(err))
            }
        }

    })
}

#[cfg(test)]
mod test {

use super::*;

#[test]
fn empty() {
    assert_eq!(None, tokenize("").next());
}

#[test]
fn all_tokens() {
    let src = "x 10.3 + - * / % () x = =>";
    let expected = vec![
        Token::Id("x".to_owned()),
        Token::Number(10.3),
        Token::Operator(Operator::Add),
        Token::Operator(Operator::Sub),
        Token::Operator(Operator::Mul),
        Token::Operator(Operator::Div),
        Token::Operator(Operator::Mod),
        Token::LBracket,
        Token::RBracket,
        Token::Assign("x".to_owned()),
        Token::Func,
    ];

    assert_eq!(Ok(expected), tokenize(src).collect());
}

#[test]
fn invalid() {
    tokenize("^").collect::<Result<Vec<_>, _>>().unwrap_err();
}

}
