use std::collections::VecDeque;

use crate::lexer::*;
use crate::shared::*;

macro_rules! assert_lexeme {
    ($self:ident, $lex:pat, $msg:expr) => {
        match $self.pop() {
            Some($lex) => {}
            _ => return Err($msg.into()),
        };
    };
}

pub struct AstProgram<'s> {
    statements: Vec<AstStatement<'s>>,
}

pub enum AstStatement<'s> {
    FnDef {
        name: &'s str,
        block: Vec<AstBlockLine<'s>>,
    },
    BlockLine(AstBlockLine<'s>),
}

pub enum AstBlockLine<'s> {
    Expr(AstExpr<'s>),
}

pub enum AstExpr<'s> {
    FnCall(Vec<AstExpr<'s>>),
    Str(&'s str),
    Int(i32),
}

pub struct Parser<'s> {
    lexemes: VecDeque<Lexeme<'s>>,
    ptr: usize,
}

impl<'s> Parser<'s> {
    pub fn new(lexemes: VecDeque<Lexeme<'s>>) -> Parser<'s> {
        Parser { lexemes, ptr: 0 }
    }

    pub fn build_ast(&mut self) -> Result<AstProgram<'s>, Error> {
        let mut statements = vec![];

        loop {
            if self.is_end() {
                break;
            }

            let statement = self.build_statement()?;
            statements.push(statement);
        }

        Ok(AstProgram { statements })
    }

    fn build_statement(&mut self) -> Result<AstStatement<'s>, Error> {
        match self.peek() {
            Some(&Lexeme::Fn) => self.build_fn_def(),
            Some(_) => Ok(AstStatement::BlockLine(self.build_block_line()?)),
            None => Err("Reached end before reading statement".into()),
        }
    }

    fn build_fn_def(&mut self) -> Result<AstStatement<'s>, Error> {
        assert_lexeme!(self, Lexeme::Fn, "Expected Fn lexeme");

        let name = match self.pop() {
            Some(Lexeme::Name(s)) => s,
            _ => return Err("Expected function name".into()),
        };

        assert_lexeme!(self, Lexeme::ParenOpen, "Expected paren open");
        assert_lexeme!(self, Lexeme::ParenClose, "Expected paren close");
        assert_lexeme!(self, Lexeme::BraceOpen, "Expected brace open");

        let mut block = vec![];
        loop {
            if let Some(&Lexeme::BraceClose) = self.peek() {
                break;
            }

            let statement = self.build_block_line()?;
            block.push(statement);
        }

        assert_lexeme!(self, Lexeme::BraceClose, "Expected brace close");

        Ok(AstStatement::FnDef { name, block })
    }

    fn build_block_line(&mut self) -> Result<AstBlockLine<'s>, Error> {
        Ok(AstBlockLine::Expr(self.build_expr()?))
    }

    fn build_expr(&mut self) -> Result<AstExpr<'s>, Error> {
        match self.peek() {
            Some(Lexeme::Int(_)) => self.build_expr_int(),
            Some(Lexeme::Str(_)) => self.build_expr_str(),
            Some(Lexeme::Name(fn_name)) => self.build_expr_fn_call(),
            _ => Err("Cannot build expression".into()),
        }
    }

    fn build_expr_int(&mut self) -> Result<AstExpr<'s>, Error> {
        match self.pop() {
            Some(Lexeme::Int(n)) => Ok(AstExpr::Int(n)),
            _ => Err("Expected integer".into()),
        }
    }

    fn build_expr_str(&mut self) -> Result<AstExpr<'s>, Error> {
        match self.pop() {
            Some(Lexeme::Str(s)) => Ok(AstExpr::Str(s)),
            _ => Err("Expected string".into()),
        }
    }

    fn build_expr_fn_call(&mut self) -> Result<AstExpr<'s>, Error> {
        let name = match self.pop() {
            Some(Lexeme::Name(name)) => name,
            _ => return Err("Expected name".into()),
        };

        assert_lexeme!(self, Lexeme::ParenOpen, "Expected paren open");

        let mut args = vec![];

        if let Some(Lexeme::ParenClose) = self.peek() {
            // Just for pattern matching, skip arg collection.
        } else {
            loop {
                let arg = self.build_expr()?;
                args.push(arg);

                if let Some(&Lexeme::Comma) = self.peek() {
                    continue;
                } else {
                    break;
                }
            }
        }

        assert_lexeme!(self, Lexeme::ParenOpen, "Expected paren close");

        Ok(AstExpr::FnCall(args))
    }

    fn is_end(&self) -> bool {
        self.ptr >= self.lexemes.len()
    }

    fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.front()
    }

    fn pop(&mut self) -> Option<Lexeme<'s>> {
        self.lexemes.pop_front()
    }
}
