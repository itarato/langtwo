use std::collections::HashMap;

use crate::ast::*;
use crate::shared::*;

struct Scope<'s> {
    functions: HashMap<&'s str, (Vec<&'s str>, AstBlock<'s>)>,
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
    Bool(bool),
    Null,
}

enum CtrlResult<T> {
    Break,
    Other(T),
}

type CtrlOrExprResult = CtrlResult<ExprResult>;
type CtrlOrMaybeExprResult = CtrlResult<Option<ExprResult>>;

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
                    last_result = match self.interpret_block_line(line)? {
                        CtrlResult::Break => {
                            return Err("Break from the program out of loop context".into())
                        }
                        CtrlResult::Other(other) => other,
                    }
                }
            };
        }

        Ok(last_result)
    }

    fn interpret_fn_def(&mut self, name: &'s str, args: Vec<&'s str>, block: AstBlock<'s>) {
        self.global_frame.functions.insert(name, (args, block));
    }

    fn interpret_block_line(
        &mut self,
        line: AstBlockLine<'s>,
    ) -> Result<CtrlOrMaybeExprResult, Error> {
        match line {
            AstBlockLine::Expr(expr) => match self.interpret_expr(expr)? {
                CtrlResult::Break => Ok(CtrlResult::Break),
                CtrlResult::Other(other) => Ok(CtrlResult::Other(Some(other))),
            },
            AstBlockLine::Loop(block) => {
                self.interpret_loop(block)?;
                Ok(CtrlResult::Other(None))
            }
        }
    }

    fn interpret_loop(&mut self, block: AstBlock<'s>) -> Result<(), Error> {
        unimplemented!()
    }

    fn interpret_expr(&mut self, expr: AstExpr<'s>) -> Result<CtrlOrExprResult, Error> {
        match expr {
            AstExpr::Int(v) => Ok(CtrlResult::Other(ExprResult::Int(v))),
            AstExpr::Str(s) => Ok(CtrlResult::Other(ExprResult::Str(s.to_string()))),
            AstExpr::Boolean(b) => Ok(CtrlResult::Other(ExprResult::Bool(b))),
            AstExpr::FnCall { name, args } => self.interpret_expr_fn_call(name, args),
            AstExpr::Name(name) => self.variable_get(name),
            AstExpr::Assignment { varname, expr } => {
                let result = match self.interpret_expr(*expr)? {
                    CtrlResult::Break => return Ok(CtrlResult::Break),
                    CtrlResult::Other(other) => other,
                };
                self.variable_set(varname, result.clone())?;
                Ok(CtrlResult::Other(result))
            }
            AstExpr::BinOp { lhs, op, rhs } => self.interpret_expr_binop(lhs, op, rhs),
            AstExpr::If {
                cond,
                true_block,
                false_block,
            } => self.interpret_expr_if(*cond, true_block, false_block),
            AstExpr::ParenExpr(inner_expr) => self.interpret_expr(*inner_expr),
        }
    }

    fn interpret_expr_if(
        &mut self,
        cond: AstExpr<'s>,
        true_block: AstBlock<'s>,
        false_block: Option<AstBlock<'s>>,
    ) -> Result<CtrlOrExprResult, Error> {
        let cond_result = match self.interpret_expr(cond)? {
            CtrlResult::Break => return Ok(CtrlResult::Break),
            CtrlResult::Other(other) => other,
        };

        let bool_result = match cond_result {
            ExprResult::Bool(b) => b,
            ExprResult::Null => false,
            ExprResult::Int(v) => v != 0,
            ExprResult::Str(s) => !s.is_empty(),
        };

        if bool_result {
            self.interpret_block(true_block)
        } else {
            match false_block {
                Some(block) => self.interpret_block(block),
                _ => Ok(CtrlResult::Other(ExprResult::Null)),
            }
        }
    }

    fn interpret_expr_binop(
        &mut self,
        lhs: Box<AstExpr<'s>>,
        op: Op,
        rhs: Box<AstExpr<'s>>,
    ) -> Result<CtrlOrExprResult, Error> {
        let lhs_result = match self.interpret_expr(*lhs)? {
            CtrlResult::Break => return Ok(CtrlResult::Break),
            CtrlResult::Other(other) => other,
        };
        let rhs_result = match self.interpret_expr(*rhs)? {
            CtrlResult::Break => return Ok(CtrlResult::Break),
            CtrlResult::Other(other) => other,
        };

        let result = match (op, lhs_result, rhs_result) {
            (Op::Add, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Int(a + b),
            (Op::Sub, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Int(a - b),
            (Op::Mul, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Int(a * b),
            (Op::Div, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Int(a / b),
            (Op::Mod, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Int(a % b),

            (Op::Eq, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Bool(a == b),
            (Op::Eq, ExprResult::Str(a), ExprResult::Str(b)) => ExprResult::Bool(a == b),
            (Op::Eq, ExprResult::Bool(a), ExprResult::Bool(b)) => ExprResult::Bool(a == b),
            (Op::Eq, ExprResult::Null, ExprResult::Null) => ExprResult::Bool(true),
            (Op::Eq, _, _) => ExprResult::Bool(false),

            (Op::Lt, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Bool(a < b),
            (Op::Lte, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Bool(a <= b),
            (Op::Gt, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Bool(a > b),
            (Op::Gte, ExprResult::Int(a), ExprResult::Int(b)) => ExprResult::Bool(a >= b),

            (op, lhs, rhs) => {
                return Err(
                    format!("Incompatible binop types: {:?} {:?} {:?}", lhs, op, rhs).into(),
                )
            }
        };

        Ok(CtrlResult::Other(result))
    }

    fn interpret_expr_fn_call(
        &mut self,
        name: &'s str,
        call_args: Vec<AstExpr<'s>>,
    ) -> Result<CtrlOrExprResult, Error> {
        match name {
            "print" => return self.interpret_expr_fn_call_print(call_args),
            _ => {}
        };

        let (args_names, block) = self
            .global_frame
            .functions
            .get(name)
            .ok_or::<String>("Missing function".into())?;

        let block = block.clone();
        let args_names = args_names.clone();

        let mut new_frame = Scope::new();

        // Setup frame.
        if call_args.len() != args_names.len() {
            return Err("Argument lenght mismatch".into());
        }
        for i in 0..call_args.len() {
            let var_value = match self.interpret_expr(call_args[i].clone())? {
                CtrlResult::Break => return Ok(CtrlResult::Break),
                CtrlResult::Other(v) => v,
            };
            new_frame.variables.insert(args_names[i], var_value);
        }

        self.frames.push(new_frame);

        let block_result = self.interpret_block(block)?;

        // Remove frame.
        self.frames.pop();

        Ok(block_result)
    }

    fn interpret_block(&mut self, block: AstBlock<'s>) -> Result<CtrlOrExprResult, Error> {
        let mut last_result = ExprResult::Null;
        let lines = block.0;

        for line in lines {
            last_result = match self.interpret_block_line(line.clone())? {
                CtrlResult::Break => return Ok(CtrlResult::Break),
                CtrlResult::Other(None) => ExprResult::Null,
                CtrlResult::Other(Some(result)) => result,
            };
        }

        Ok(CtrlResult::Other(last_result))
    }

    fn interpret_expr_fn_call_print(
        &mut self,
        args: Vec<AstExpr<'s>>,
    ) -> Result<CtrlOrExprResult, Error> {
        if args.len() != 1 {
            return Err("Function 'print' expects 1 argument".into());
        }
        let result = self.interpret_expr(args[0].clone())?;

        match result {
            CtrlResult::Break => return Ok(CtrlResult::Break),
            CtrlResult::Other(ExprResult::Null) => print!("null"),
            CtrlResult::Other(ExprResult::Int(v)) => print!("{}", v),
            CtrlResult::Other(ExprResult::Str(s)) => print!("{}", s),
            CtrlResult::Other(ExprResult::Bool(b)) => print!("{}", b),
        };

        Ok(CtrlResult::Other(ExprResult::Null))
    }

    fn variable_get(&self, name: &'s str) -> Result<CtrlOrExprResult, Error> {
        let top_frame = self
            .frames
            .last()
            .ok_or::<String>("No more frames".into())?;

        if top_frame.variables.contains_key(name) {
            return Ok(CtrlResult::Other(top_frame.variables[name].clone()));
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

    #[test]
    fn test_binop_simple() {
        assert_eq!(
            Some(ExprResult::Int(17)),
            interpret_this(
                r#"
                a = 12;
                b = 5;
                a + b;
        "#
            )
        );

        assert_eq!(Some(ExprResult::Int(3)), interpret_this("13 % 10;"));
    }

    #[test]
    fn test_binop_complex() {
        assert_eq!(
            Some(ExprResult::Int(62)),
            interpret_this(
                r#"
                fn id(v) { v; }
                a = 12;
                b = 5;
                a + b * 100 / id(10);
        "#
            )
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            Some(ExprResult::Int(2)),
            interpret_this(
                r#"
                if (1) {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(3)),
            interpret_this(
                r#"
                if (0) {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(2)),
            interpret_this(
                r#"
                if ("ok") {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(3)),
            interpret_this(
                r#"
                if ("") {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(3)),
            interpret_this(
                r#"
                fn empty() {}

                if (empty()) {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(2)),
            interpret_this(
                r#"
                if (true) {
                    2;
                } else {
                    3;
                }
        "#
            )
        );

        assert_eq!(
            Some(ExprResult::Int(3)),
            interpret_this(
                r#"
                if (false) {
                    2;
                } else {
                    3;
                }
        "#
            )
        );
    }

    #[test]
    fn test_recursion() {
        assert_eq!(
            Some(ExprResult::Int(3628800)),
            interpret_this(
                r#"
                fn factor(n) {
                    if (n) {
                        factor(n - 1) * n;
                    } else {
                        1;
                    }
                }

                factor(10);
        "#
            )
        );
    }

    #[test]
    fn test_recursion_with_boolean() {
        assert_eq!(
            Some(ExprResult::Int(3628800)),
            interpret_this(
                r#"
                fn factor(n) {
                    if (n == 1) {
                        n;
                    } else {
                        factor(n - 1) * n;
                    }
                }

                factor(10);
        "#
            )
        );
    }

    #[test]
    fn test_compare() {
        assert_eq!(
            Some(ExprResult::Bool(true)),
            interpret_this("2 + 2 * 2 - 6 == 6 / 6 - 3 / 3;")
        );
        assert_eq!(Some(ExprResult::Bool(false)), interpret_this("3 == 2;"));

        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("1 < 2;"));
        assert_eq!(Some(ExprResult::Bool(false)), interpret_this("3 < 2;"));

        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("1 <= 2;"));
        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("2 <= 2;"));
        assert_eq!(Some(ExprResult::Bool(false)), interpret_this("3 <= 2;"));

        assert_eq!(Some(ExprResult::Bool(false)), interpret_this("1 >= 2;"));
        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("2 >= 2;"));
        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("3 >= 2;"));
    }

    #[test]
    fn test_arithmetic_precedence() {
        assert_eq!(Some(ExprResult::Int(10)), interpret_this("2 * 3 + 4;"));
        assert_eq!(Some(ExprResult::Int(14)), interpret_this("2 + 3 * 4;"));
        assert_eq!(
            Some(ExprResult::Int(3)),
            interpret_this("2 + 2 * 2 / 2 - 2 / 2;")
        );
        assert_eq!(
            Some(ExprResult::Int(14)),
            interpret_this("2 * 2 * 2 + 2 + 2 + 2;")
        );
    }

    #[test]
    fn test_parens() {
        assert_eq!(Some(ExprResult::Int(10)), interpret_this("(2 * 3) + 4;"));
        assert_eq!(Some(ExprResult::Int(10)), interpret_this("4 + (2 * 3);"));
        assert_eq!(Some(ExprResult::Int(14)), interpret_this("2 * (3 + 4);"));
        assert_eq!(Some(ExprResult::Int(14)), interpret_this("(3 + 4) * 2;"));
    }

    #[test]
    fn test_op_precedence() {
        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("1 + 1 == 2;"));
        assert_eq!(Some(ExprResult::Bool(true)), interpret_this("2 == 1 + 1;"));
        assert_eq!(
            Some(ExprResult::Bool(true)),
            interpret_this("2 * 2 + 2 == 3 / 3 + 5;")
        );
    }

    #[test]
    fn test_fizzbuzz() {
        assert_eq!(
            Some(ExprResult::Int(100)),
            interpret_this(
                r#"
                fn fizzbuzz(i, limit) {
                    print(i);
                    print(" ");
                    if (i % 3 == 0) {
                        print("fizz");
                    }
                    if (i % 5 == 0) {
                        print("buzz");
                    }
                    print(" ");

                    if (i < limit) {
                        fizzbuzz(i + 1, limit);
                    } else {
                        i;
                    }
                }

                fizzbuzz(1, 100);
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
