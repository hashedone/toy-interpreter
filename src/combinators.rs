use crate::{Operator, Factor, Result, Assignment, Token};

#[derive(Debug, PartialEq)]
pub struct ParseProgress<'a, T> {
    pub tail: &'a str,
    pub token: Option<T>,
}

pub type ParseResult<'a, T> = Result<ParseProgress<'a, T>>;

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
    } else if
        src.chars().next().unwrap().is_ascii_alphabetic() ||
        src.starts_with('_')
    {
        let first_not = src
            .find(|c: char| -> bool {
                !(c == '_' || c.is_ascii_alphanumeric())
            })
            .unwrap_or(src.len());
        let literal = &src[..first_not];
        let tail = &src[first_not..];
        ParseProgress::some(tail, literal)
    } else {
        ParseProgress::none(src)
    }
}

fn assignment(src: &str) -> ParseResult<&str> {
    let (tail, ident) = assume!(identifier(src), src);
    let tail = tail.trim_start();
    if tail.starts_with('=') && !tail.starts_with("=>") {
        ParseProgress::some(&tail[1..], ident)
    } else {
        ParseProgress::none(src)
    }
}

pub fn next_token(src: &str) -> ParseResult<Token> {
    if src.is_empty() {
        return ParseProgress::none("");
    }

    let assign = assignment(src)?;
    if let Some(tok) = assign.token {
        return ParseProgress::some(
            assign.tail, Token::Assign(tok.to_owned())
        );
    }

    let id = identifier(src)?;
    if let Some(tok) = id.token {
        return ParseProgress::some(id.tail, Token::Id(tok.to_owned()));
    }

    let num = number(src)?;
    if let Some(tok) = num.token {
        return ParseProgress::some(num.tail, Token::Number(tok));
    }

    if src.starts_with("=>") {
        return ParseProgress::some(&src[2..], Token::Func);
    }

    let tok = match src {
        _ if src.starts_with('+') => Token::Operator(Operator::Add),
        _ if src.starts_with('-') => Token::Operator(Operator::Sub),
        _ if src.starts_with('*') => Token::Operator(Operator::Mul),
        _ if src.starts_with('/') => Token::Operator(Operator::Div),
        _ if src.starts_with('%') => Token::Operator(Operator::Mod),
        _ if src.starts_with('(') => Token::LBracket,
        _ if src.starts_with(')') => Token::RBracket,
        _ => return Err(format!("Invalid token: {}", src)),
    };

    return ParseProgress::some(&src[1..], tok);
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
    assert_eq!(ParseProgress::some("", "ab"), identifier("ab"));
    assert_eq!(ParseProgress::some("", "_ab"), identifier("_ab"));
    assert_eq!(ParseProgress::some(".", "_ab"), identifier("_ab."));
    assert_eq!(ParseProgress::some("", "__"), identifier("__"));
    assert_eq!(ParseProgress::some("", "_1"), identifier("_1"));
}

#[test]
fn test_assignment() {
    assert_eq!(ParseProgress::none(""), assignment(""));
    assert_eq!(ParseProgress::none("x"), assignment("x"));
    assert_eq!(ParseProgress::some("", "x"), assignment("x ="));
    assert_eq!(ParseProgress::none("x =>"), assignment("x =>"));
}

#[test]
fn test_next_token() {
    assert_eq!(ParseProgress::none(""), next_token(""));
    assert_eq!(
        ParseProgress::some("", Token::Operator(Operator::Add)),
        next_token("+")
    );
    assert_eq!(
        ParseProgress::some("", Token::Operator(Operator::Sub)),
        next_token("-")
    );
    assert_eq!(
        ParseProgress::some("", Token::Operator(Operator::Mul)),
        next_token("*")
    );
    assert_eq!(
        ParseProgress::some("", Token::Operator(Operator::Div)),
        next_token("/")
    );
    assert_eq!(
        ParseProgress::some("", Token::Operator(Operator::Mod)),
        next_token("%")
    );
    assert_eq!(
        ParseProgress::some("", Token::LBracket),
        next_token("(")
    );
    assert_eq!(
        ParseProgress::some("", Token::RBracket),
        next_token(")")
    );
    assert_eq!(
        ParseProgress::some("", Token::Assign("x".to_owned())),
        next_token("x =")
    );
    assert_eq!(
        ParseProgress::some("", Token::Func),
        next_token("=>")
    );
    assert_eq!(
        ParseProgress::some("x", Token::Operator(Operator::Mod)),
        next_token("%x")
    );
    assert_eq!(
        ParseProgress::some(" =>", Token::Id("x".to_owned())),
        next_token("x =>")
    );

    next_token("10.0.4").unwrap_err();
    next_token("=").unwrap_err();
}
}
