use std::collections::VecDeque;

use crate::ast::*;
use crate::lexer::*;
use crate::shared::*;

macro_rules! assert_lexeme {
    ($self:ident, $lex:pat, $msg:expr) => {
        let got = $self.pop();
        match &got {
            Some($lex) => {}
            _ => {
                return {
                    let full_msg = format!(
                        "{} | Got: {:?} | Rest lexemes: {:?} | Loc {}:{}",
                        $msg,
                        got,
                        $self.lexemes,
                        file!(),
                        line!()
                    );
                    Err(full_msg.into())
                }
            }
        };
    };
}

pub struct Parser<'s> {
    lexemes: VecDeque<Lexeme<'s>>,
}

impl<'s> Parser<'s> {
    pub fn new(lexemes: VecDeque<Lexeme<'s>>) -> Parser<'s> {
        Parser { lexemes }
    }

    pub fn build_ast(&mut self) -> Result<AstProgram<'s>, Error> {
        debug!("Build: program");

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
        debug!("Build: statement");

        match self.peek() {
            Some(&Lexeme::Fn) => self.build_fn_def(),
            Some(_) => Ok(AstStatement::BlockLine(self.build_block_line()?)),
            None => Err("Reached end before reading statement".into()),
        }
    }

    fn build_fn_def(&mut self) -> Result<AstStatement<'s>, Error> {
        debug!("Build: fn def");

        assert_lexeme!(self, Lexeme::Fn, "Expected Fn lexeme");

        let name = match self.pop() {
            Some(Lexeme::Name(s)) => s,
            _ => return Err("Expected function name".into()),
        };

        assert_lexeme!(self, Lexeme::ParenOpen, "Expected paren open");

        let mut args = vec![];

        if let Some(Lexeme::ParenClose) = self.peek() {
            // Pattern match to skip args.
        } else {
            loop {
                match self.pop() {
                    Some(Lexeme::Name(name)) => args.push(name),
                    _ => return Err("Expected argument name".into()),
                };

                if let Some(Lexeme::Comma) = self.peek() {
                    self.pop();
                    continue;
                }

                break;
            }
        }

        assert_lexeme!(self, Lexeme::ParenClose, "Expected paren close");

        let block = self.build_block()?;

        Ok(AstStatement::FnDef { name, args, block })
    }

    fn build_block_line(&mut self) -> Result<AstBlockLine<'s>, Error> {
        debug!("Build: block line");

        let expr = self.build_expr()?;

        // Does it need a semicolon?
        match expr {
            AstExpr::If { .. } => {}
            _ => {
                assert_lexeme!(self, Lexeme::Semicolon, "Expected semicolon");
            }
        };

        let line = AstBlockLine::Expr(expr);

        Ok(line)
    }

    fn build_expr(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr");

        let expr = match self.peek() {
            Some(Lexeme::Int(_)) => self.build_expr_int(),
            Some(Lexeme::Str(_)) => self.build_expr_str(),
            Some(Lexeme::True) => {
                self.pop();
                Ok(AstExpr::Boolean(true))
            }
            Some(Lexeme::False) => {
                self.pop();
                Ok(AstExpr::Boolean(false))
            }
            Some(Lexeme::Name(_)) => match self.peekn(1) {
                Some(Lexeme::ParenOpen) => self.build_expr_fn_call(),
                Some(Lexeme::Assign) => self.build_expr_assignment(),
                _ => self.build_expr_name(),
            },
            Some(Lexeme::If) => self.build_expr_if(),
            _ => Err("Cannot build expression".into()),
        }?;

        match self.peek() {
            Some(Lexeme::OpAdd) | Some(Lexeme::OpSub) | Some(Lexeme::OpMul)
            | Some(Lexeme::OpDiv) | Some(Lexeme::OpMod) | Some(Lexeme::OpEq)
            | Some(Lexeme::OpLt) | Some(Lexeme::OpLte) | Some(Lexeme::OpGt)
            | Some(Lexeme::OpGte) => {
                let op = Op::from_lexeme(self.pop().unwrap())?;
                let rhs = self.build_expr()?;

                Ok(self.reorder_binop_precedence(AstExpr::BinOp {
                    lhs: Box::new(expr),
                    op,
                    rhs: Box::new(rhs),
                }))
            }
            _ => Ok(expr),
        }
    }

    fn build_expr_if(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/if");

        assert_lexeme!(self, Lexeme::If, "Expected keyword if");
        assert_lexeme!(self, Lexeme::ParenOpen, "Expected paren open");

        let cond = self.build_expr()?;

        assert_lexeme!(self, Lexeme::ParenClose, "Expected paren close");

        let true_block = self.build_block()?;

        assert_lexeme!(self, Lexeme::Else, "Expected keyword else");

        let false_block = self.build_block()?;

        Ok(AstExpr::If {
            cond: Box::new(cond),
            true_block,
            false_block,
        })
    }

    fn build_block(&mut self) -> Result<AstBlock<'s>, Error> {
        assert_lexeme!(self, Lexeme::BraceOpen, "Expected brace open");

        let mut block_lines = vec![];
        loop {
            if let Some(&Lexeme::BraceClose) = self.peek() {
                break;
            }

            let statement = self.build_block_line()?;
            block_lines.push(statement);
        }

        assert_lexeme!(self, Lexeme::BraceClose, "Expected brace close");

        Ok(AstBlock(block_lines))
    }

    fn build_expr_assignment(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/assign");

        let varname = match self.pop() {
            Some(Lexeme::Name(name)) => name,
            _ => return Err("Expected name for assignment".into()),
        };

        assert_lexeme!(self, Lexeme::Assign, "Expected assign");

        let expr = self.build_expr()?;

        Ok(AstExpr::Assignment {
            varname,
            expr: Box::new(expr),
        })
    }

    fn build_expr_int(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/int");

        match self.pop() {
            Some(Lexeme::Int(n)) => Ok(AstExpr::Int(n)),
            _ => Err("Expected integer".into()),
        }
    }

    fn build_expr_str(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/str");

        match self.pop() {
            Some(Lexeme::Str(s)) => Ok(AstExpr::Str(s)),
            _ => Err("Expected string".into()),
        }
    }

    fn build_expr_name(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/name");

        match self.pop() {
            Some(Lexeme::Name(s)) => Ok(AstExpr::Name(s)),
            _ => Err("Expected name".into()),
        }
    }

    fn build_expr_fn_call(&mut self) -> Result<AstExpr<'s>, Error> {
        debug!("Build: expr/fn-call");

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
                    self.pop();
                    continue;
                }

                break;
            }
        }

        assert_lexeme!(self, Lexeme::ParenClose, "Expected paren close");

        Ok(AstExpr::FnCall { name, args })
    }

    fn reorder_binop_precedence(&self, expr: AstExpr<'s>) -> AstExpr<'s> {
        match expr {
            AstExpr::BinOp { lhs, op, rhs } => match *rhs {
                AstExpr::BinOp {
                    lhs: rhs_lhs,
                    op: rhs_op,
                    rhs: rhs_rhs,
                } => {
                    if op.precedence() > rhs_op.precedence() {
                        // Wrong precendence. Needs to rotate the branches (recursively to the left subtree):
                        //   1            2
                        //  / \          / \
                        // A   2   =>   1   C
                        //    / \      / \
                        //   B   C    A   B
                        AstExpr::BinOp {
                            lhs: Box::new(self.reorder_binop_precedence(AstExpr::BinOp {
                                lhs,
                                op,
                                rhs: rhs_lhs,
                            })),
                            op: rhs_op,
                            rhs: rhs_rhs,
                        }
                    } else {
                        AstExpr::BinOp {
                            lhs,
                            op,
                            rhs: Box::new(AstExpr::BinOp {
                                lhs: rhs_lhs,
                                op: rhs_op,
                                rhs: rhs_rhs,
                            }),
                        }
                    }
                }
                _ => AstExpr::BinOp { lhs, op, rhs },
            },
            other => other,
        }
    }

    fn is_end(&self) -> bool {
        self.lexemes.is_empty()
    }

    fn peek(&self) -> Option<&Lexeme> {
        self.peekn(0)
    }

    fn peekn(&self, n: usize) -> Option<&Lexeme> {
        self.lexemes.get(n)
    }

    fn pop(&mut self) -> Option<Lexeme<'s>> {
        self.lexemes.pop_front()
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::*;
    use crate::parser::*;
    use crate::source_reader::*;

    #[test]
    fn build_empty_program() {
        let root = parse_this(r#""#);
        assert_eq!(0, root.statements.len());
    }

    #[test]
    fn build_minimal_program() {
        let root = parse_this(
            r#"
            fn main(word, second) {
                print(word);
                print(second);
                print(fixed());
                fixed();
            }

            fn fixed() {
                123;
            }

            main(123, "hello");
        "#,
        );
        assert_eq!(3, root.statements.len());
    }

    #[test]
    fn test_expr_assignment() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / assign
                expr / fncall
                "#
            .trim()
            .to_owned(),
            parse_this("a = calc();").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_name() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / name
    stmt / fndef
        blocklinelist
            blockline
                expr / name
                "#
            .trim()
            .to_owned(),
            parse_this("abc; fn main(x){ x; }").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_int() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / int
                "#
            .trim()
            .to_owned(),
            parse_this("2;").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_str() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / str
                "#
            .trim()
            .to_owned(),
            parse_this("\"hi\";").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_fn_call() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / fncall
                "#
            .trim()
            .to_owned(),
            parse_this("main(123);").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_if() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / if
                blocklinelist
                    blockline
                        expr / fncall
                blocklinelist
                    blockline
                        expr / str
                "#
            .trim()
            .to_owned(),
            parse_this("if (2) { main(); } else { \"abc\"; }").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_bool() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / bool
                "#
            .trim()
            .to_owned(),
            parse_this("true;").ast_dump(0)
        );

        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / bool
                "#
            .trim()
            .to_owned(),
            parse_this("false;").ast_dump(0)
        );
    }

    #[test]
    fn test_fndef() {
        assert_eq!(
            r#"
prg
    stmt / fndef
        blocklinelist
            blockline
                expr / int
                "#
            .trim()
            .to_owned(),
            parse_this("fn main() { 0; }").ast_dump(0)
        );
    }

    #[test]
    fn test_single_op() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / binop
                expr / int
                expr / int
                "#
            .trim()
            .to_owned(),
            parse_this("1 + 2;").ast_dump(0)
        );
    }

    #[test]
    fn test_multiple_op() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / binop
                expr / int
                expr / binop
                    expr / int
                    expr / binop
                        expr / int
                        expr / int
                "#
            .trim()
            .to_owned(),
            parse_this("1 + 2 * 3 / 4;").ast_dump(0)
        );
    }

    #[test]
    fn test_string_op() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / binop
                expr / str
                expr / str
                "#
            .trim()
            .to_owned(),
            parse_this("\"a\" + \"b\";").ast_dump(0)
        );
    }

    #[test]
    fn test_expr_binop_eq() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / binop
                expr / str
                expr / str
                "#
            .trim()
            .to_owned(),
            parse_this("\"a\" == \"b\";").ast_dump(0)
        );
    }

    #[test]
    fn test_op_in_argument() {
        assert_eq!(
            r#"
prg
    stmt
        blockline
            expr / fncall
                "#
            .trim()
            .to_owned(),
            parse_this("main(a + 3 * other());").ast_dump(0)
        );
    }

    fn parse_this(input: &'static str) -> AstProgram<'static> {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        Parser::new(lexemes.into()).build_ast().unwrap()
    }
}
