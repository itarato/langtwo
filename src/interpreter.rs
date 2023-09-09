use std::collections::HashMap;

use crate::ast::*;
use crate::shared::*;

struct Scope<'s> {
    functions: HashMap<&'s str, Vec<AstBlockLine<'s>>>,
}

impl<'s> Scope<'s> {
    fn new() -> Scope<'s> {
        Scope {
            functions: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum ExprResult {
    Int(i32),
    Str(String),
    Null,
}

pub struct Interpreter<'s> {
    global_frame: Scope<'s>,
}

impl<'s> Interpreter<'s> {
    pub fn new() -> Interpreter<'s> {
        Interpreter {
            global_frame: Scope::new(),
        }
    }

    pub fn interpret(&mut self, program: AstProgram<'s>) {
        for statement in program.statements {
            match statement {
                AstStatement::FnDef { name, block } => self.interpret_fn_def(name, block),
                AstStatement::BlockLine(line) => {
                    self.interpret_block_line(line);
                }
            };
        }
    }

    fn interpret_fn_def(&mut self, name: &'s str, block: Vec<AstBlockLine<'s>>) {
        self.global_frame.functions.insert(name, block);
    }

    fn interpret_block_line(
        &mut self,
        line: AstBlockLine<'s>,
    ) -> Result<Option<ExprResult>, Error> {
        match line {
            AstBlockLine::Expr(expr) => Ok(Some(self.interpret_expr(expr)?)),
        }
    }

    fn interpret_expr(&mut self, expr: AstExpr<'s>) -> Result<ExprResult, Error> {
        match expr {
            AstExpr::Int(v) => Ok(ExprResult::Int(v)),
            AstExpr::Str(s) => Ok(ExprResult::Str(s.to_string())),
            AstExpr::FnCall { name, args } => self.interpret_expr_fn_call(name, args),
        }
    }

    fn interpret_expr_fn_call(
        &mut self,
        name: &'s str,
        args: Vec<AstExpr<'s>>,
    ) -> Result<ExprResult, Error> {
        match name {
            "print" => return self.interpret_expr_fn_call_print(args),
            _ => {}
        };

        let mut last_result = ExprResult::Null;

        let lines = self
            .global_frame
            .functions
            .get(name)
            .ok_or::<String>("Missing function".into())?;

        for line in lines.clone() {
            last_result = self
                .interpret_block_line(line.clone())?
                .unwrap_or(ExprResult::Null);
        }

        Ok(last_result)
    }

    fn interpret_expr_fn_call_print(
        &mut self,
        args: Vec<AstExpr<'s>>,
    ) -> Result<ExprResult, Error> {
        if args.len() != 1 {
            return Err("Function 'print' expects 1 argument".into());
        }
        let result = self.interpret_expr(args[0].clone())?;

        match result {
            ExprResult::Null => print!("null"),
            ExprResult::Int(v) => print!("{}", v),
            ExprResult::Str(s) => print!("{}", s),
        };

        Ok(ExprResult::Null)
    }
}
