use crate::shared::*;
use crate::source_reader::*;

#[derive(Debug)]
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

pub struct Lexer<'a> {
    reader: Box<dyn SourceReader<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn parse(reader: Box<dyn SourceReader<'a>>) -> Result<Vec<Lexeme<'a>>, Error> {
        let lexer = Lexer::new(reader);
        lexer.read_any()
    }

    fn new(reader: Box<dyn SourceReader<'a>>) -> Lexer<'a> {
        Lexer { reader }
    }

    fn read_any(&self) -> Result<Vec<Lexeme<'a>>, Error> {
        Ok(vec![Lexeme::Fn])
    }
}
