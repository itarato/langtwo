use crate::shared::*;
use crate::source_reader::*;

#[derive(Debug, PartialEq)]
pub enum Lexeme<'a> {
    Name(&'a str),
    Int(i32),
    Str(&'a str),
    True,
    False,
    Fn,
    If,
    Else,
    Loop,
    Break,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Semicolon,
    Comma,
    Assign,
    Op(Op),
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
                    'a'..='z' => self.read_word()?,
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
                    '=' => {
                        self.reader.next();
                        match self.reader.peek() {
                            Some('=') => {
                                self.reader.next();
                                Lexeme::Op(Op::Eq)
                            }
                            _ => Lexeme::Assign,
                        }
                    }
                    '+' => {
                        self.reader.next();
                        Lexeme::Op(Op::Add)
                    }
                    '-' => {
                        self.reader.next();
                        Lexeme::Op(Op::Sub)
                    }
                    '*' => {
                        self.reader.next();
                        Lexeme::Op(Op::Mul)
                    }
                    '/' => {
                        self.reader.next();
                        Lexeme::Op(Op::Div)
                    }
                    '%' => {
                        self.reader.next();
                        Lexeme::Op(Op::Mod)
                    }
                    '<' => {
                        self.reader.next();
                        match self.reader.peek() {
                            Some('=') => {
                                self.reader.next();
                                Lexeme::Op(Op::Lte)
                            }
                            _ => Lexeme::Op(Op::Lt),
                        }
                    }
                    '>' => {
                        self.reader.next();
                        match self.reader.peek() {
                            Some('=') => {
                                self.reader.next();
                                Lexeme::Op(Op::Gte)
                            }
                            _ => Lexeme::Op(Op::Gt),
                        }
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

    fn read_word(&mut self) -> Result<Lexeme<'a>, Error> {
        self.reader
            .read_until(|c| c.is_ascii_alphanumeric())
            .ok_or("Empty name".into())
            .map(|slice| match slice {
                "fn" => Lexeme::Fn,
                "if" => Lexeme::If,
                "else" => Lexeme::Else,
                "true" => Lexeme::True,
                "false" => Lexeme::False,
                "loop" => Lexeme::Loop,
                "break" => Lexeme::Break,
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
        assert!(lex_this("").unwrap().is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(vec![Lexeme::Name("hi")], lex_this("\thi \n").unwrap());
    }

    #[test]
    fn test_int() {
        assert_eq!(vec![Lexeme::Int(1024)], lex_this("\t1024 \n").unwrap());
    }

    #[test]
    fn test_str() {
        assert_eq!(
            vec![Lexeme::Str("bla blu")],
            lex_this("\t\"bla blu\" \n").unwrap()
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            vec![
                Lexeme::Fn,
                Lexeme::If,
                Lexeme::Else,
                Lexeme::Loop,
                Lexeme::Break
            ],
            lex_this("\tfn if else loop break\n").unwrap()
        );
    }

    #[test]
    fn test_paren_open() {
        assert_eq!(vec![Lexeme::ParenOpen], lex_this("\t( \n").unwrap());
    }

    #[test]
    fn test_paren_close() {
        assert_eq!(vec![Lexeme::ParenClose], lex_this("\t) \n").unwrap());
    }

    #[test]
    fn test_brace_open() {
        assert_eq!(vec![Lexeme::BraceOpen], lex_this("\t{ \n").unwrap());
    }

    #[test]
    fn test_brace_close() {
        assert_eq!(vec![Lexeme::BraceClose], lex_this("\t} \n").unwrap());
    }

    #[test]
    fn test_semicolon() {
        assert_eq!(vec![Lexeme::Semicolon], lex_this("\t; \n").unwrap());
    }

    #[test]
    fn test_comma() {
        assert_eq!(vec![Lexeme::Comma], lex_this("\t, \n").unwrap());
    }

    #[test]
    fn test_assign() {
        assert_eq!(vec![Lexeme::Assign], lex_this("\t= \n").unwrap());
    }

    #[test]
    fn test_ops() {
        assert_eq!(
            vec![
                Lexeme::Op(Op::Add),
                Lexeme::Op(Op::Sub),
                Lexeme::Op(Op::Mul),
                Lexeme::Op(Op::Div),
                Lexeme::Op(Op::Mod)
            ],
            lex_this("\t+    -    */  % \n").unwrap()
        );
    }

    #[test]
    fn test_op_eq_and_assign() {
        assert_eq!(
            vec![
                Lexeme::Assign,
                Lexeme::Op(Op::Eq),
                Lexeme::Op(Op::Eq),
                Lexeme::Assign
            ],
            lex_this("\t= == == = \n").unwrap()
        );
    }

    #[test]
    fn test_compare() {
        assert_eq!(
            vec![
                Lexeme::Op(Op::Eq),
                Lexeme::Op(Op::Gt),
                Lexeme::Op(Op::Gte),
                Lexeme::Op(Op::Lt),
                Lexeme::Op(Op::Lte)
            ],
            lex_this("\t== > >= < <=\n").unwrap()
        );
    }

    #[test]
    fn test_boolean() {
        assert_eq!(
            vec![Lexeme::True, Lexeme::False],
            lex_this("\t true \n\r false \n").unwrap()
        );
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
            lex_this("\thello 123     fn(){}\"no\"\n").unwrap()
        );
    }

    fn lex_this(input: &'static str) -> Result<Vec<Lexeme>, Error> {
        let reader = Box::new(StrReader::new(input));
        Lexer::new(reader).read_any()
    }
}
