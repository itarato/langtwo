use std::collections::HashMap;

use crate::ast::*;
use crate::shared::*;

/**
 * --------------
 * return address <- ARP
 * arg1           \
 * ...             > args
 * argn           /
 * local var 1    \
 * local var 2     > local variables
 * ...              
 */

type ImmVal = i32;
// This might be a hack for now, but a simple auto-inc usize will do it.
type Label = usize;
type CondCode = Vec<CondResult>;
type OutRegAndOps = (Reg, Vec<Operation>);
type RegAddr = usize;

#[derive(Debug, PartialEq)]
enum Reg {
    Global(RegAddr),
    Arp(RegAddr), // ARP + offset.
}

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
    // This is a hack during generation.
    Label(Label),
    // This is not part of ILOC but without these it's not trivial how to make proc calls.
    Call(Label),
    Return,

    Add {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    Sub {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    Mul {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    Div {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },

    AddI {
        lhs: Reg,
        rhs: ImmVal,
        out: Reg,
    },
    SubI {
        lhs: Reg,
        rhs: ImmVal,
        out: Reg,
    },
    MulI {
        lhs: Reg,
        rhs: ImmVal,
        out: Reg,
    },
    DivI {
        lhs: Reg,
        rhs: ImmVal,
        out: Reg,
    },

    Load {
        addr: Reg,
        out: Reg,
    },
    LoadAI {
        addr: Reg,
        offs: ImmVal,
        out: Reg,
    },
    LoadAO {
        addr: Reg,
        offs: Reg,
        out: Reg,
    },
    LoadI {
        val: ImmVal,
        out: Reg,
    },

    Store {
        reg: Reg,
        addr: Reg,
    },
    StoreAI {
        reg: Reg,
        addr: Reg,
        offs: ImmVal,
    },
    StoreAO {
        reg: Reg,
        addr: Reg,
        offs: Reg,
    },

    I2i {
        lhs: Reg,
        rhs: Reg,
    },

    Ci2i {
        cond: Reg,
        lhs: Reg,
        rhs: Reg,
    },

    JumpI(Label),
    Jump(Reg),

    Tbl {
        reg: Reg,
        label: Label,
    },

    CmpLt {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    CmpLte {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    CmpGt {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    CmpGte {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    CmpEq {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },
    CmpNotEq {
        lhs: Reg,
        rhs: Reg,
        out: Reg,
    },

    CondBranch {
        cond: Reg,
        label_true: Label,
        label_false: Label,
    },

    Comp {
        lhs: Reg,
        rhs: Reg,
        out: CondCode,
    },
}

struct Scope {
    next_free_reg_addr: RegAddr,
    variables: HashMap<String, Reg>,
    fn_out_regs: HashMap<String, Reg>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            next_free_reg_addr: 0,
            variables: HashMap::new(),
            fn_out_regs: HashMap::new(),
        }
    }
}

pub struct IRBuilder {
    next_free_label: Label,
    fn_labels: HashMap<String, Label>,
    frames: Vec<Scope>,
}

impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder {
            next_free_label: 0,
            fn_labels: HashMap::new(),
            frames: vec![Scope::new()],
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
        let mut ops = vec![];

        let fn_start_label = self.get_fn_label(name);
        let fn_end_label = self.next_free_label();

        // Need to declare: `Label(end-of(name))` (so we can jump from pre-function line to after the function)
        ops.push(Operation::JumpI(fn_end_label));

        // Need to declare: `Label(name)`
        ops.push(Operation::Label(fn_start_label));

        // We need to allocate Size(args) registers to work with `args` for names of `args`
        // We need to render the ops for `block`
        let args_segment_size = args.len();
        let ar_local_variable_start = args_segment_size + 1; // +1 is for the return address
        let (block_out_reg, mut block_ops) =
            self.build_block(block, Some(ar_local_variable_start))?;
        self.fn_out_regs.insert(name.into(), block_out_reg);
        ops.append(&mut block_ops);

        ops.push(Operation::Label(fn_end_label));

        Ok(ops)
    }

    fn build_block(
        &mut self,
        block: AstBlock,
        arp_offs: Option<RegAddr>,
    ) -> Result<OutRegAndOps, Error> {
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

    fn build_expr(&mut self, expr: AstExpr) -> Result<OutRegAndOps, Error> {
        match expr {
            AstExpr::FnCall { name, args } => unimplemented!(),
            AstExpr::Str(s) => unimplemented!(),
            AstExpr::Int(i) => self.build_expr_int(i),
            AstExpr::Name(name) => self.build_expr_name(name),
            AstExpr::Boolean(b) => unimplemented!(),
            AstExpr::Assignment { varname, expr } => self.build_expr_assignment(varname, *expr),
            AstExpr::BinOp { lhs, op, rhs } => self.build_expr_binop(*lhs, op, *rhs),
            AstExpr::If {
                cond,
                true_block,
                false_block,
            } => unimplemented!(),
            AstExpr::ParenExpr(expr) => unimplemented!(),
        }
    }

    fn build_expr_assignment(
        &mut self,
        varname: &str,
        expr: AstExpr,
    ) -> Result<OutRegAndOps, Error> {
        let (expr_reg, mut expr_ops) = self.build_expr(expr)?;
        let out = self.get_variable_reg_addr(varname);
        let mut ops = vec![];
        ops.append(&mut expr_ops);

        ops.push(Operation::I2i {
            lhs: expr_reg,
            rhs: out,
        });

        Ok((out, ops))
    }

    fn build_expr_binop(
        &mut self,
        lhs: AstExpr,
        op: Op,
        rhs: AstExpr,
    ) -> Result<OutRegAndOps, Error> {
        let (lhs_reg, mut lhs_ops) = self.build_expr(lhs)?;
        let (rhs_reg, mut rhs_ops) = self.build_expr(rhs)?;

        let mut ops = vec![];
        ops.append(&mut lhs_ops);
        ops.append(&mut rhs_ops);

        let out = Reg::Global(self.next_free_reg_addr());

        match op {
            Op::Add => {
                ops.push(Operation::Add {
                    lhs: lhs_reg,
                    rhs: rhs_reg,
                    out,
                });
            }
            Op::Sub => {
                ops.push(Operation::Sub {
                    lhs: lhs_reg,
                    rhs: rhs_reg,
                    out,
                });
            }
            Op::Div => {
                ops.push(Operation::Div {
                    lhs: lhs_reg,
                    rhs: rhs_reg,
                    out,
                });
            }
            Op::Mul => {
                ops.push(Operation::Mul {
                    lhs: lhs_reg,
                    rhs: rhs_reg,
                    out,
                });
            }
            _ => unimplemented!(),
        };

        Ok((out, ops))
    }

    fn build_expr_name(&mut self, name: &str) -> Result<OutRegAndOps, Error> {
        let addr = self.get_variable_reg_addr(name);
        Ok((addr, vec![]))
    }

    fn build_expr_int(&mut self, val: i32) -> Result<OutRegAndOps, Error> {
        let out = Reg::Global(self.next_free_reg_addr());
        let op = Operation::LoadI { val, out };
        Ok((out, vec![op]))
    }

    fn next_free_reg_addr(&mut self) -> Reg {
        if self.frames.len() < 1 {
            panic!("Missing frames");
        }

        let addr = self.frames.last().unwrap().next_free_reg_addr;
        self.frames.last_mut().unwrap().next_free_reg_addr += 1;

        if self.frames.len() == 1 {
            Reg::Global(addr)
        } else {
            Reg::Arp(addr)
        }
    }

    fn get_variable_reg_addr(&mut self, name: &str) -> Reg {
        if self
            .frames
            .last()
            .unwrap()
            .variables
            .contains_key(name.into())
        {
            self.frames.last().unwrap().variables[name.into()]
        } else {
            let addr = Reg::Global(self.next_free_reg_addr());
            self.frames
                .last_mut()
                .unwrap()
                .variables
                .insert(name.into(), addr);
            addr
        }
    }

    fn next_free_label(&mut self) -> Label {
        let label = self.next_free_label;
        self.next_free_label += 1;
        label
    }

    fn get_fn_label(&mut self, name: &str) -> Label {
        if self.fn_labels.contains_key(name.into()) {
            self.fn_labels[name.into()]
        } else {
            let label = self.next_free_label();
            self.fn_labels.insert(name.into(), label);
            label
        }
    }
}

#[derive(Debug)]
pub struct IR {
    pub instructions: Vec<Operation>,
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
            vec![Operation::LoadI {
                val: 4,
                out: Reg::Global(0)
            }],
            ir_this("4;").instructions
        );
    }

    #[test]
    fn test_int_binop_add() {
        assert_eq!(
            vec![
                Operation::LoadI {
                    val: 4,
                    out: Reg::Global(0)
                },
                Operation::LoadI {
                    val: 1,
                    out: Reg::Global(1)
                },
                Operation::Add {
                    lhs: Reg::Global(0),
                    rhs: Reg::Global(1),
                    out: Reg::Global(2)
                },
            ],
            ir_this("4 + 1;").instructions
        );
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            vec![
                Operation::LoadI {
                    val: 4,
                    out: Reg::Global(0)
                }, // 4 -> r0
                Operation::I2i {
                    lhs: Reg::Global(0),
                    rhs: Reg::Global(1)
                }, // r0 -> r1(a)
                Operation::LoadI {
                    val: 2,
                    out: Reg::Global(2)
                }, // 2 -> r2
                Operation::Add {
                    // r1(a) + r2 -> r3
                    lhs: Reg::Global(1),
                    rhs: Reg::Global(2),
                    out: Reg::Global(3)
                },
                Operation::I2i {
                    lhs: Reg::Global(3),
                    rhs: Reg::Global(1)
                }, // r3 -> r1(a)
            ],
            ir_this("a = 4; a = a + 2;").instructions
        );
    }

    fn ir_this(input: &'static str) -> IR {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        let ast_root = Parser::new(lexemes.into()).build_ast().unwrap();
        IRBuilder::new().build(ast_root).unwrap()
    }
}