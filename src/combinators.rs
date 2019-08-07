use crate::{Operator, Factor, Result, Assignment};

#[derive(Debug, PartialEq)]
struct ParseProgress<'a, T> {
    tail: &'a str,
    token: Option<T>,
}

type ParseResult<'a, T> = Result<ParseProgress<'a, T>>;

impl<'a, T> ParseProgress<'a, T> {
    fn none(tail: &'a str) -> ParseResult<'a, T> {
        Ok(ParseProgress {
            tail,
            token: None,
        })
    }

    fn some(tail: &'a str, token: T) -> ParseResult<'a, T> {
        Ok(ParseProgress {
            tail,
            token: Some(token),
        })
    }

}

macro_rules! assume {
    ($e:expr, $tail:expr) => {{
        let e = $e?;
        if e.token.is_some() { (e.tail, e.token.unwrap()) }
        else { return ParseProgress::none($tail); }
    }}
}

fn number(src: &str) -> ParseResult<f32> {
    let first_not = src
        .find(|c| !"0123456789.".contains(c))
        .unwrap_or(src.len());

    if first_not == 0 {
        return ParseProgress::none(src);
    }

    let literal = &src[..first_not];
    let tail = &src[first_not..];
    if literal.chars().filter(|&c| c == '.').count() > 1 {
        Err(format!("Invalid number: {}, only one decimal point allowed", literal))
    } else {
        let number = literal
            .parse()
            .map_err(|err| format!("Invalid numer: {}, {}", literal, err))?;
        ParseProgress::some(tail, number)
    }
}

fn identifier(src: &str) -> ParseResult<&str> {
    if src.is_empty() {
        ParseProgress::none(src)
    } else if src.starts_with('_') {
        let first_not = src
            .find(|c: char| -> bool {
                !(c == '_' || c.is_ascii_alphanumeric())
            })
            .unwrap_or(src.len());
        let literal = &src[..first_not];
        let tail = &src[first_not..];
        ParseProgress::some(tail, literal)
    } else if src.chars().next().unwrap().is_ascii_alphabetic() {
        ParseProgress::some(&src[1..], &src[..1])
    } else {
        ParseProgress::none(src)
    }
}

fn operator(src: &str) -> ParseResult<Operator> {
    let op = match src {
        _ if src.starts_with('+') => Operator::Add,
        _ if src.starts_with('-') => Operator::Sub,
        _ if src.starts_with('*') => Operator::Mul,
        _ if src.starts_with('/') => Operator::Div,
        _ if src.starts_with('%') => Operator::Mod,
        _ => return ParseProgress::none(src),
    };

    ParseProgress::some(&src[1..], op)
}

fn assignment(src: &str) -> ParseResult<Assignment> {
    let (left, var) = assume!(identifier(src), src);
    let left = left.trim_start();
    let left =
        if left.starts_with("=") { &left[1..] }
        else { return ParseProgress::none(src) };
    let left = left.trim_start();
    let res = number(left)?;
    let val = res.token.ok_or(
        "Invalid right side of assignment - expected expression".to_owned()
    )?;

    ParseProgress::some(res.tail, Assignment { var: var.to_owned(), val })
}

fn factor(src: &str) -> Result<(&str, Option<Factor>)> {
    unimplemented!()
}

#[cfg(test)]
mod test {

use super::*;

#[test]
fn test_number() {
    assert_eq!(ParseProgress::none(""), number(""));
    assert_eq!(ParseProgress::none("tail"), number("tail"));
    assert_eq!(ParseProgress::some("", 10.0f32), number("10"));
    assert_eq!(ParseProgress::some("", 10.4f32), number("10.4"));
    assert_eq!(ParseProgress::some("tail", 10.4f32), number("10.4tail"));
    number("10.4.5").unwrap_err();
}

#[test]
fn test_identifier() {
    assert_eq!(ParseProgress::none(""), identifier(""));
    assert_eq!(ParseProgress::none("10"), identifier("10"));
    assert_eq!(ParseProgress::some("", "a"), identifier("a"));
    assert_eq!(ParseProgress::some("b", "a"), identifier("ab"));
    assert_eq!(ParseProgress::some("", "_ab"), identifier("_ab"));
    assert_eq!(ParseProgress::some(".", "_ab"), identifier("_ab."));
    assert_eq!(ParseProgress::some("", "__"), identifier("__"));
    assert_eq!(ParseProgress::some("", "_1"), identifier("_1"));
}

#[test]
fn test_operator() {
    assert_eq!(ParseProgress::none(""), operator(""));
    assert_eq!(ParseProgress::none("10"), operator("10"));
    assert_eq!(ParseProgress::some("", Operator::Add), operator("+"));
    assert_eq!(ParseProgress::some("", Operator::Sub), operator("-"));
    assert_eq!(ParseProgress::some("", Operator::Mul), operator("*"));
    assert_eq!(ParseProgress::some("", Operator::Div), operator("/"));
    assert_eq!(ParseProgress::some("", Operator::Mod), operator("%"));
    assert_eq!(ParseProgress::some("x", Operator::Mod), operator("%x"));
}

#[test]
fn test_assignment() {
    assert_eq!(ParseProgress::none(""), assignment(""));
    assert_eq!(ParseProgress::none("10"), assignment("10"));
    assert_eq!(
        ParseProgress::some("", Assignment::new("a", 10.0)),
        assignment("a = 10")
    );
    assert_eq!(
        ParseProgress::some("x", Assignment::new("a", 10.0)),
        assignment("a = 10x")
    );
    assignment("a = =").unwrap_err();
}

}
