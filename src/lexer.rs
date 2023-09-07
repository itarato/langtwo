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
    pub fn new(reader: Box<dyn SourceReader<'a>>) -> Lexer<'a> {
        Lexer { reader }
    }

    pub fn read_any(&'a mut self) -> Result<Vec<Lexeme<'a>>, Error> {
        let mut lexemes = vec![];

        loop {
            {
                self.consume_whitespace();
            }

            let c;
            {
                let peek = self.reader.peek();
                if peek.is_none() {
                    break;
                }
                c = peek.unwrap();
            }
            let lexeme = match c {
                // '0'..='9' => self.read_number()?,
                _ => return Err(format!("Invalid char during lexing: {}", c).into()),
            };

            lexemes.push(lexeme);
        }

        Ok(lexemes)
    }

    fn consume_whitespace(&'a mut self) {
        let _ = self.reader.read_until(|c| c == ' ');
    }

    fn read_number(&self) -> Result<Lexeme<'a>, Error> {
        unimplemented!()
    }
}
