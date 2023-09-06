use crate::shared::*;

pub enum Lexeme<'a> {
    Name(&'a str),
    Int(i32),
    Str(&'a str),
    Fn,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Semicolon,
}

pub struct Lexer;

impl Lexer {
    pub fn parse(source: &str) -> Result<Vec<Lexeme>, Error> {
        unimplemented!();
    }
}
