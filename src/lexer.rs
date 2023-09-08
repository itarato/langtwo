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

    pub fn read_any(&mut self) -> Result<Vec<Lexeme<'a>>, Error> {
        let mut lexemes = vec![];

        loop {
            self.consume_whitespace();

            let lexeme = match self.reader.peek() {
                None => break,
                Some(c) => match c {
                    '0'..='9' => self.read_number()?,
                    'a'..='z' => self.read_name()?,
                    '"' => self.read_string()?,
                    '(' => {
                        self.reader.next();
                        Lexeme::ParenOpen
                    }
                    ')' => {
                        self.reader.next();
                        Lexeme::ParenClose
                    }
                    ';' => {
                        self.reader.next();
                        Lexeme::Semicolon
                    }
                    '{' => {
                        self.reader.next();
                        Lexeme::BraceOpen
                    }
                    '}' => {
                        self.reader.next();
                        Lexeme::BraceClose
                    }
                    _ => return Err(format!("Invalid char during lexing: {}", c).into()),
                },
            };

            lexemes.push(lexeme);
        }

        Ok(lexemes)
    }

    fn consume_whitespace(&mut self) {
        let _ = self.reader.read_until(|c| c.is_whitespace());
    }

    fn read_number(&mut self) -> Result<Lexeme<'a>, Error> {
        self.reader
            .read_until(|c| c >= '0' && c <= '9')
            .ok_or("Empty number".into())
            .and_then(|slice| {
                i32::from_str_radix(slice, 10)
                    .map(|num| Lexeme::Int(num))
                    .map_err(|_| "Failed converting string to number".into())
            })
    }

    fn read_name(&mut self) -> Result<Lexeme<'a>, Error> {
        self.reader
            .read_until(|c| c >= 'a' && c <= 'z')
            .ok_or("Empty name".into())
            .map(|slice| match slice {
                "fn" => Lexeme::Fn,
                _ => Lexeme::Name(slice),
            })
    }

    fn read_string(&mut self) -> Result<Lexeme<'a>, Error> {
        if self.reader.next() != Some('"') {
            return Err("String must start with \"".into());
        }

        let str = self.reader.read_until(|c| c != '"').unwrap_or("");

        if self.reader.next() != Some('"') {
            return Err("String must end with \"".into());
        }

        Ok(Lexeme::Str(str))
    }
}
