use std::collections::HashMap;

use crate::ast::*;
use crate::shared::*;

type RegVal = usize;
type ImmVal = i32;
// This might be a hack for now, but a simple auto-inc usize will do it.
type Label = usize;
type CondCode = Vec<CondResult>;
type ResultRegAndOps = (RegVal, Vec<Operation>);

#[derive(Debug, PartialEq)]
pub enum CondResult {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Add {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    Sub {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    Mul {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    Div {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },

    AddI {
        lhs: RegVal,
        rhs: ImmVal,
        out: RegVal,
    },
    SubI {
        lhs: RegVal,
        rhs: ImmVal,
        out: RegVal,
    },
    MulI {
        lhs: RegVal,
        rhs: ImmVal,
        out: RegVal,
    },
    DivI {
        lhs: RegVal,
        rhs: ImmVal,
        out: RegVal,
    },

    Load {
        addr: RegVal,
        out: RegVal,
    },
    LoadAI {
        addr: RegVal,
        offs: ImmVal,
        out: RegVal,
    },
    LoadAO {
        addr: RegVal,
        offs: RegVal,
        out: RegVal,
    },
    LoadI {
        val: ImmVal,
        out: RegVal,
    },

    Store {
        reg: RegVal,
        addr: RegVal,
    },
    StoreAI {
        reg: RegVal,
        addr: RegVal,
        offs: ImmVal,
    },
    StoreAO {
        reg: RegVal,
        addr: RegVal,
        offs: RegVal,
    },

    I2i {
        lhs: RegVal,
        rhs: RegVal,
    },

    Ci2i {
        cond: RegVal,
        lhs: RegVal,
        rhs: RegVal,
    },

    JumpI(Label),
    Jump(RegVal),

    Tbl {
        reg: RegVal,
        label: Label,
    },

    CmpLt {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    CmpLte {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    CmpGt {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    CmpGte {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    CmpEq {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },
    CmpNotEq {
        lhs: RegVal,
        rhs: RegVal,
        out: RegVal,
    },

    CondBranch {
        cond: RegVal,
        label_true: Label,
        label_false: Label,
    },

    Comp {
        lhs: RegVal,
        rhs: RegVal,
        out: CondCode,
    },
}

pub struct IRBuilder {
    next_free_reg_addr: usize,
    variables: HashMap<String, RegVal>,
}

impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder {
            next_free_reg_addr: 0,
            variables: HashMap::new(),
        }
    }

    pub fn build(&mut self, ast: AstProgram) -> Result<IR, Error> {
        let instructions = self.build_program(ast)?;

        Ok(IR { instructions })
    }

    fn build_program(&mut self, ast: AstProgram) -> Result<Vec<Operation>, Error> {
        let mut ins = vec![];
        for stmt in ast.statements {
            let mut stmt_ins = self.build_statement(stmt)?;
            ins.append(&mut stmt_ins);
        }

        Ok(ins)
    }

    fn build_statement(&mut self, stmt: AstStatement) -> Result<Vec<Operation>, Error> {
        match stmt {
            AstStatement::FnDef { name, args, block } => self.build_fn_def(name, args, block),
            AstStatement::BlockLine(line) => self.build_block_line(line),
        }
    }

    fn build_fn_def(
        &mut self,
        name: &str,
        args: Vec<&str>,
        block: AstBlock,
    ) -> Result<Vec<Operation>, Error> {
        // Make a label to current (start-of-function) and last+1 (end-of-function).
        unimplemented!()
    }

    fn build_block_line(&mut self, line: AstBlockLine) -> Result<Vec<Operation>, Error> {
        match line {
            AstBlockLine::Expr(expr) => {
                let (_, ops) = self.build_expr(expr)?;
                Ok(ops)
            }
            AstBlockLine::Loop(block) => unimplemented!(),
            AstBlockLine::Break => unimplemented!(),
        }
    }

    fn build_expr(&mut self, expr: AstExpr) -> Result<ResultRegAndOps, Error> {
        match expr {
            AstExpr::FnCall { name, args } => unimplemented!(),
            AstExpr::Str(s) => unimplemented!(),
            AstExpr::Int(i) => self.build_expr_int(i),
            AstExpr::Name(name) => self.build_expr_name(name),
            AstExpr::Boolean(b) => unimplemented!(),
            AstExpr::Assignment { varname, expr } => unimplemented!(),
            AstExpr::BinOp { lhs, op, rhs } => self.build_expr_binop(*lhs, op, *rhs),
            AstExpr::If {
                cond,
                true_block,
                false_block,
            } => unimplemented!(),
            AstExpr::ParenExpr(expr) => unimplemented!(),
        }
    }

    fn build_expr_binop(
        &mut self,
        lhs: AstExpr,
        op: Op,
        rhs: AstExpr,
    ) -> Result<ResultRegAndOps, Error> {
        let (lhs_reg, mut lhs_ops) = self.build_expr(lhs)?;
        let (rhs_reg, mut rhs_ops) = self.build_expr(rhs)?;

        let mut ops = vec![];
        ops.append(&mut lhs_ops);
        ops.append(&mut rhs_ops);

        let out = self.next_free_reg_addr();

        match op {
            Op::Add => {
                ops.push(Operation::Add {
                    lhs: lhs_reg,
                    rhs: rhs_reg,
                    out,
                });
            }
            _ => unimplemented!(),
        };

        Ok((out, ops))
    }

    fn build_expr_name(&mut self, name: &str) -> Result<ResultRegAndOps, Error> {
        let addr = self.register_variable_name(name);
        Ok((addr, vec![]))
    }

    fn build_expr_int(&mut self, val: i32) -> Result<ResultRegAndOps, Error> {
        let out = self.next_free_reg_addr();
        let op = Operation::LoadI { val, out };
        Ok((out, vec![op]))
    }

    fn next_free_reg_addr(&mut self) -> usize {
        let addr = self.next_free_reg_addr;
        self.next_free_reg_addr += 1;
        addr
    }

    fn register_variable_name(&mut self, name: &str) -> RegVal {
        let addr = self.next_free_reg_addr();
        self.variables.insert(name.into(), addr);
        addr
    }

    fn get_variable_addr(&mut self, name: &str) -> Result<RegVal, Error> {
        self.variables
            .get(name.into())
            .map(|v| *v)
            .ok_or("Variable not found".into())
    }
}

#[derive(Debug)]
pub struct IR {
    instructions: Vec<Operation>,
}

#[cfg(test)]
mod test {
    use crate::ir::*;
    use crate::lexer::*;
    use crate::parser::*;
    use crate::source_reader::*;

    #[test]
    fn test_empty() {
        assert!(ir_this("").instructions.is_empty());
    }

    #[test]
    fn test_int() {
        assert_eq!(
            vec![Operation::LoadI { val: 4, out: 0 }],
            ir_this("4;").instructions
        );
    }

    #[test]
    fn test_int_binop_add() {
        assert_eq!(
            vec![
                Operation::LoadI { val: 4, out: 0 },
                Operation::LoadI { val: 1, out: 1 },
                Operation::Add {
                    lhs: 0,
                    rhs: 1,
                    out: 2
                },
            ],
            ir_this("4 + 1;").instructions
        );
    }

    fn ir_this(input: &'static str) -> IR {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        let ast_root = Parser::new(lexemes.into()).build_ast().unwrap();
        IRBuilder::new().build(ast_root).unwrap()
    }
}
