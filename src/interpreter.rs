use std::collections::HashMap;

use crate::ast::*;
use crate::shared::*;

struct Scope<'s> {
    functions: HashMap<&'s str, (Vec<&'s str>, Vec<AstBlockLine<'s>>)>,
    variables: HashMap<&'s str, ExprResult>,
}

impl<'s> Scope<'s> {
    fn new() -> Scope<'s> {
        Scope {
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprResult {
    Int(i32),
    Str(String),
    Null,
}

pub struct Interpreter<'s> {
    global_frame: Scope<'s>,
    frames: Vec<Scope<'s>>,
}

impl<'s> Interpreter<'s> {
    pub fn new() -> Interpreter<'s> {
        Interpreter {
            global_frame: Scope::new(),
            frames: vec![Scope::new()],
        }
    }

    pub fn interpret(&mut self, program: AstProgram<'s>) -> Result<Option<ExprResult>, Error> {
        let mut last_result = None;
        for statement in program.statements {
            match statement {
                AstStatement::FnDef { name, args, block } => {
                    self.interpret_fn_def(name, args, block)
                }
                AstStatement::BlockLine(line) => {
                    last_result = self.interpret_block_line(line)?;
                }
            };
        }

        Ok(last_result)
    }

    fn interpret_fn_def(
        &mut self,
        name: &'s str,
        args: Vec<&'s str>,
        block: Vec<AstBlockLine<'s>>,
    ) {
        self.global_frame.functions.insert(name, (args, block));
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
            AstExpr::Name(name) => self.variable_get(name),
            AstExpr::Assignment { varname, expr } => {
                let result = self.interpret_expr(*expr)?;
                self.variable_set(varname, result.clone())?;
                Ok(result)
            }
        }
    }

    fn interpret_expr_fn_call(
        &mut self,
        name: &'s str,
        call_args: Vec<AstExpr<'s>>,
    ) -> Result<ExprResult, Error> {
        match name {
            "print" => return self.interpret_expr_fn_call_print(call_args),
            _ => {}
        };

        let mut last_result = ExprResult::Null;

        let (args_names, lines) = self
            .global_frame
            .functions
            .get(name)
            .ok_or::<String>("Missing function".into())?;

        let args_names = args_names.clone();
        let lines = lines.clone();

        let mut new_frame = Scope::new();

        // Setup frame.
        if call_args.len() != args_names.len() {
            return Err("Argument lenght mismatch".into());
        }
        for i in 0..call_args.len() {
            new_frame
                .variables
                .insert(args_names[i], self.interpret_expr(call_args[i].clone())?);
        }

        self.frames.push(new_frame);

        for line in lines {
            last_result = self
                .interpret_block_line(line.clone())?
                .unwrap_or(ExprResult::Null);
        }

        // Remove frame.
        self.frames.pop();

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

    fn variable_get(&self, name: &'s str) -> Result<ExprResult, Error> {
        let top_frame = self
            .frames
            .last()
            .ok_or::<String>("No more frames".into())?;

        if top_frame.variables.contains_key(name) {
            return Ok(top_frame.variables[name].clone());
        }

        Err("Variable not found".into())
    }

    fn variable_set(&mut self, name: &'s str, value: ExprResult) -> Result<(), Error> {
        let top_frame = self
            .frames
            .last_mut()
            .ok_or::<String>("No more frames".into())?;

        top_frame.variables.insert(name, value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::*;
    use crate::lexer::*;
    use crate::parser::*;
    use crate::source_reader::*;

    #[test]
    fn test_empty_program() {
        assert_eq!(None, interpret_this(""));
    }

    #[test]
    fn test_expression_value() {
        assert_eq!(Some(ExprResult::Int(7)), interpret_this("1; 3; 7;"));
    }

    #[test]
    fn test_empty_fn_call() {
        assert_eq!(
            Some(ExprResult::Null),
            interpret_this(
                r#"
                fn main() {
                }
                main();
        "#
            )
        );
    }

    #[test]
    fn test_fn_call() {
        assert_eq!(
            Some(ExprResult::Str("ok".into())),
            interpret_this(
                r#"
                fn main() {
                    "ok";
                }
                main();
        "#
            )
        );
    }

    #[test]
    fn test_multiple_fn_call() {
        assert_eq!(
            Some(ExprResult::Int(9)),
            interpret_this(
                r#"
                fn main(v) {
                    proxy(v);
                }
                fn proxy(x) {
                    x;
                }
                main(9);
        "#
            )
        );
    }

    #[test]
    fn test_fn_call_with_fn_call_arg() {
        assert_eq!(
            Some(ExprResult::Int(6)),
            interpret_this(
                r#"
                fn main(v) {
                    v;
                }
                fn fixed() {
                    6;
                }
                main(fixed());
        "#
            )
        );
    }

    fn interpret_this(input: &'static str) -> Option<ExprResult> {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        let ast_root = Parser::new(lexemes.into()).build_ast().unwrap();
        Interpreter::new().interpret(ast_root).unwrap()
    }
}
