use crate::lexer::*;
use crate::shared::*;

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
    lexemes: Vec<Lexeme<'s>>,
    ptr: usize,
}

impl<'s> Parser<'s> {
    pub fn new(lexemes: Vec<Lexeme<'s>>) -> Parser<'s> {
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
        unimplemented!()
    }

    fn is_end(&self) -> bool {
        self.ptr >= self.lexemes.len()
    }

    fn peek(&self) -> &Lexeme {
        if self.is_end() {
            panic!("Peek lexeme when its ended");
        }

        &self.lexemes[self.ptr]
    }
}
