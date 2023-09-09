use crate::shared::*;
use crate::source_reader::*;

#[derive(Debug, PartialEq)]
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
    Comma,
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
                    ',' => {
                        self.reader.next();
                        Lexeme::Comma
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
            .read_until(|c| c.is_ascii_digit())
            .ok_or("Empty number".into())
            .and_then(|slice| {
                i32::from_str_radix(slice, 10)
                    .map(|num| Lexeme::Int(num))
                    .map_err(|_| "Failed converting string to number".into())
            })
    }

    fn read_name(&mut self) -> Result<Lexeme<'a>, Error> {
        self.reader
            .read_until(|c| c.is_ascii_alphanumeric())
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

#[cfg(test)]
mod test {
    use crate::lexer::*;

    #[test]
    fn test_empty_input() {
        assert!(lex_these("").unwrap().is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(vec![Lexeme::Name("hi")], lex_these("\thi \n").unwrap());
    }

    #[test]
    fn test_int() {
        assert_eq!(vec![Lexeme::Int(1024)], lex_these("\t1024 \n").unwrap());
    }

    #[test]
    fn test_str() {
        assert_eq!(
            vec![Lexeme::Str("bla blu")],
            lex_these("\t\"bla blu\" \n").unwrap()
        );
    }

    #[test]
    fn test_fn() {
        assert_eq!(vec![Lexeme::Fn], lex_these("\tfn \n").unwrap());
    }

    #[test]
    fn test_paren_open() {
        assert_eq!(vec![Lexeme::ParenOpen], lex_these("\t( \n").unwrap());
    }

    #[test]
    fn test_paren_close() {
        assert_eq!(vec![Lexeme::ParenClose], lex_these("\t) \n").unwrap());
    }

    #[test]
    fn test_brace_open() {
        assert_eq!(vec![Lexeme::BraceOpen], lex_these("\t{ \n").unwrap());
    }

    #[test]
    fn test_brace_close() {
        assert_eq!(vec![Lexeme::BraceClose], lex_these("\t} \n").unwrap());
    }

    #[test]
    fn test_semicolon() {
        assert_eq!(vec![Lexeme::Semicolon], lex_these("\t; \n").unwrap());
    }

    #[test]
    fn test_messy_mix() {
        assert_eq!(
            vec![
                Lexeme::Name("hello"),
                Lexeme::Int(123),
                Lexeme::Fn,
                Lexeme::ParenOpen,
                Lexeme::ParenClose,
                Lexeme::BraceOpen,
                Lexeme::BraceClose,
                Lexeme::Str("no")
            ],
            lex_these("\thello 123     fn(){}\"no\"\n").unwrap()
        );
    }

    fn lex_these(input: &'static str) -> Result<Vec<Lexeme>, Error> {
        let reader = Box::new(StrReader::new(input));
        Lexer::new(reader).read_any()
    }
}
