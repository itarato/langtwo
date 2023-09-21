use std::collections::HashMap;

use crate::ast::*;
use crate::ir::*;
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

struct Scope {
    next_free_reg_addr: RegAddr,
    variables: HashMap<String, Reg>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            next_free_reg_addr: 0,
            variables: HashMap::new(),
        }
    }
}

pub struct IRBuilder {
    next_free_label: usize,
    frames: Vec<Scope>,
}

impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder {
            next_free_label: 0,
            frames: vec![Scope::new()],
        }
    }

    pub fn build(&mut self, ast: AstProgram) -> Result<IR, Error> {
        let (out, ops) = self.build_program(ast)?;

        Ok(IR::new(ops, out))
    }

    fn build_program(&mut self, ast: AstProgram) -> Result<MaybeOutRegAndOps, Error> {
        let mut ins = vec![];
        let mut out: Option<Reg> = None;
        for stmt in ast.statements {
            let (stmt_out, mut stmt_ins) = self.build_statement(stmt)?;
            ins.append(&mut stmt_ins);
            out = stmt_out;
        }

        Ok((out, ins))
    }

    fn build_statement(&mut self, stmt: AstStatement) -> Result<MaybeOutRegAndOps, Error> {
        match stmt {
            AstStatement::FnDef { name, args, block } => {
                let ops = self.build_fn_def(name, args, block)?;
                Ok((None, ops))
            }
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

        let fn_start_label = Label::Named(name.into());
        let fn_end_label = self.next_free_label();

        // Need to declare: `Label(end-of(name))` (so we can jump from pre-function line to after the function)
        ops.push(Operation::JumpI(fn_end_label.clone()));

        // Need to declare: `Label(name)`
        ops.push(Operation::Label(fn_start_label));

        // We need to allocate Size(args) registers to work with `args` for names of `args`
        // We need to render the ops for `block`

        // Establish new frame.
        self.frames.push(Scope::new());

        // Pop arguments.
        // !!! DANGER !!! Currently there is no check that each push-ed value will be popped. RISK!
        for arg in args {
            let arg_reg = self.get_variable_reg_addr(arg);
            ops.push(Operation::Pop(arg_reg));
        }

        let (block_out_reg, mut block_ops) = self.build_block(block)?;
        if block_out_reg.is_none() {
            return Err("No expression ending for a block.".into());
        }

        self.frames.pop();

        ops.append(&mut block_ops);

        // Save return value.
        ops.push(Operation::Push(block_out_reg.unwrap()));
        ops.push(Operation::Return);

        ops.push(Operation::Label(fn_end_label));

        Ok(ops)
    }

    fn build_block(&mut self, block: AstBlock) -> Result<MaybeOutRegAndOps, Error> {
        let mut ops = vec![];
        let mut out: Option<Reg> = None;
        for line in block.0 {
            let (line_out, mut line_ops) = self.build_block_line(line)?;
            ops.append(&mut line_ops);
            out = line_out;
        }

        Ok((out, ops))
    }

    fn build_block_line(&mut self, line: AstBlockLine) -> Result<MaybeOutRegAndOps, Error> {
        match line {
            AstBlockLine::Expr(expr) => {
                let (expr_reg, ops) = self.build_expr(expr)?;
                Ok((Some(expr_reg), ops))
            }
            AstBlockLine::Loop(block) => unimplemented!(),
            AstBlockLine::Break => unimplemented!(),
        }
    }

    fn build_expr(&mut self, expr: AstExpr) -> Result<OutRegAndOps, Error> {
        match expr {
            AstExpr::FnCall { name, args } => self.build_expr_fn_call(name, args),
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
            } => self.build_expr_if(*cond, true_block, false_block),
            AstExpr::ParenExpr(expr) => unimplemented!(),
        }
    }

    fn build_expr_if(
        &mut self,
        cond: AstExpr,
        true_block: AstBlock,
        false_block: Option<AstBlock>,
    ) -> Result<OutRegAndOps, Error> {
        let out = self.next_free_reg_addr();
        let mut ops = vec![];

        let (cond_out, mut cond_ops) = self.build_expr(cond)?;
        ops.append(&mut cond_ops);

        let label_true = self.next_free_label();
        let label_end = self.next_free_label();
        let label_false = self.next_free_label();

        ops.push(Operation::CondBranch {
            cond: cond_out,
            label_true: label_true.clone(),
            label_false: label_false.clone(),
        });

        let (true_out, mut true_ops) = self.build_block(true_block)?;
        ops.push(Operation::Label(label_true));
        ops.append(&mut true_ops);
        // Saving return value to reg - or set it to 0 for now.
        if let Some(true_out) = true_out {
            ops.push(Operation::I2i {
                lhs: true_out,
                rhs: out,
            });
        } else {
            ops.push(Operation::LoadI { val: 0, out });
        }
        ops.push(Operation::JumpI(label_end.clone()));

        ops.push(Operation::Label(label_false));

        let false_out = if let Some(false_block) = false_block {
            let (false_out, mut false_ops) = self.build_block(false_block)?;
            ops.append(&mut false_ops);
            false_out
        } else {
            None
        };

        if let Some(false_out) = false_out {
            ops.push(Operation::I2i {
                lhs: false_out,
                rhs: out,
            });
        } else {
            ops.push(Operation::LoadI { val: 0, out });
        }
        ops.push(Operation::JumpI(label_end.clone()));

        ops.push(Operation::Label(label_end));

        Ok((out, ops))
    }

    fn build_expr_fn_call(
        &mut self,
        name: &str,
        args: Vec<AstExpr>,
    ) -> Result<OutRegAndOps, Error> {
        let mut ops = vec![];

        // let mut op_lists = vec![];
        let mut op_returns = vec![];
        for arg_expr in args {
            let (arg_expr_reg, mut arg_expr_ops) = self.build_expr(arg_expr)?;
            op_returns.push(arg_expr_reg);
            // op_lists.push(arg_expr_ops);

            ops.append(&mut arg_expr_ops);
        }

        // Reverse order - so `POP` inside the proceduce can read them in order.
        // TODO: Add verification that we push exactly as much as the arg count. This we could do if
        //       save some info about the function.
        while let Some(op_return) = op_returns.pop() {
            ops.push(Operation::Push(op_return));
        }

        // When executing `call` the return adds could automatically saved by the VM.

        ops.push(Operation::Call(Label::Named(name.into())));

        let out = self.next_free_reg_addr();
        ops.push(Operation::Pop(out));

        Ok((out, ops))
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

        let out = self.next_free_reg_addr();

        match op {
            Op::Add => ops.push(Operation::Add {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out,
            }),
            Op::Sub => ops.push(Operation::Sub {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out,
            }),
            Op::Div => ops.push(Operation::Div {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out,
            }),
            Op::Mul => ops.push(Operation::Mul {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out,
            }),
            Op::Eq => ops.push(Operation::CmpEq {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out: out,
            }),
            Op::Lt => ops.push(Operation::CmpLt {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out: out,
            }),
            Op::Lte => ops.push(Operation::CmpLte {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out: out,
            }),
            Op::Gt => ops.push(Operation::CmpGt {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out: out,
            }),
            Op::Gte => ops.push(Operation::CmpGte {
                lhs: lhs_reg,
                rhs: rhs_reg,
                out: out,
            }),
            _ => unimplemented!(),
        };

        Ok((out, ops))
    }

    fn build_expr_name(&mut self, name: &str) -> Result<OutRegAndOps, Error> {
        let addr = self.get_variable_reg_addr(name);
        Ok((addr, vec![]))
    }

    fn build_expr_int(&mut self, val: i32) -> Result<OutRegAndOps, Error> {
        let out = self.next_free_reg_addr();
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
            let addr = self.next_free_reg_addr();
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
        Label::Numbered(label)
    }
}

#[cfg(test)]
mod test {
    use crate::ir_builder::*;
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

    #[test]
    fn test_fn_call() {
        assert_eq!(
            vec![
                Operation::JumpI(Label::Numbered(0)),
                Operation::Label(Label::Named("add".into())), // fn: add
                Operation::Pop(Reg::Arp(0)),                  // pop -> rarp+0 (r1)
                Operation::Pop(Reg::Arp(1)),                  // pop -> rarp+1 (r2)
                Operation::Add {
                    // add rarp+0 rarp+1 -> rarp+2
                    lhs: Reg::Arp(0),
                    rhs: Reg::Arp(1),
                    out: Reg::Arp(2)
                },
                Operation::Push(Reg::Arp(2)), // push rarp+2
                Operation::Return,            // return
                Operation::Label(Label::Numbered(0)),
                Operation::LoadI {
                    // 5 -> r0(x)
                    val: 5,
                    out: Reg::Global(0)
                },
                Operation::I2i {
                    // r0(x) -> r1
                    lhs: Reg::Global(0),
                    rhs: Reg::Global(1)
                },
                Operation::LoadI {
                    // 7 -> r2
                    val: 7,
                    out: Reg::Global(2)
                },
                Operation::Push(Reg::Global(2)), // push r2
                Operation::Push(Reg::Global(1)), // push r1
                Operation::Call(Label::Named("add".into())), // call add
                Operation::Pop(Reg::Global(3))   // after return / pop -> r2 (final result)
            ],
            ir_this("fn add(a, b) { a + b; } x = 5; add(x, 7);").instructions
        )
    }

    #[test]
    fn test_if_then() {
        assert_eq!(
            vec![
                Operation::LoadI {
                    // 1 -> r1
                    val: 1,
                    out: Reg::Global(1)
                },
                Operation::LoadI {
                    // 2 -> r2
                    val: 2,
                    out: Reg::Global(2)
                },
                Operation::CmpLt {
                    // 1 < 2 ? -> r3
                    lhs: Reg::Global(1),
                    rhs: Reg::Global(2),
                    out: Reg::Global(3)
                },
                Operation::CondBranch {
                    // r3 ? l0 : l2
                    cond: Reg::Global(3),
                    label_true: Label::Numbered(0),
                    label_false: Label::Numbered(2)
                },
                Operation::Label(Label::Numbered(0)), // if true
                Operation::LoadI {
                    // 3 -> r4
                    val: 3,
                    out: Reg::Global(4)
                },
                Operation::I2i {
                    // r4 -> r0
                    lhs: Reg::Global(4),
                    rhs: Reg::Global(0)
                },
                Operation::JumpI(Label::Numbered(1)), // jump to end
                Operation::Label(Label::Numbered(2)), // if false
                Operation::LoadI {
                    // 0 -> r0
                    val: 0,
                    out: Reg::Global(0)
                },
                Operation::JumpI(Label::Numbered(1)), // jump to end
                Operation::Label(Label::Numbered(1))  // end
            ],
            ir_this(
                r#"
                if (1 < 2) {
                    3;
                }
            "#
            )
            .instructions
        );
    }

    #[test]
    fn test_if_then_else() {
        assert_eq!(
            vec![
                Operation::LoadI {
                    // 1 -> r1
                    val: 1,
                    out: Reg::Global(1)
                },
                Operation::LoadI {
                    // 2 -> r2
                    val: 2,
                    out: Reg::Global(2)
                },
                Operation::CmpLt {
                    // 1 < 2 ? -> r3
                    lhs: Reg::Global(1),
                    rhs: Reg::Global(2),
                    out: Reg::Global(3)
                },
                Operation::CondBranch {
                    // r3 ? l0 : l2
                    cond: Reg::Global(3),
                    label_true: Label::Numbered(0),
                    label_false: Label::Numbered(2)
                },
                Operation::Label(Label::Numbered(0)), // if true
                Operation::LoadI {
                    // 3 -> r4
                    val: 3,
                    out: Reg::Global(4)
                },
                Operation::I2i {
                    // r4 -> r0
                    lhs: Reg::Global(4),
                    rhs: Reg::Global(0)
                },
                Operation::JumpI(Label::Numbered(1)), // jump to end
                Operation::Label(Label::Numbered(2)), // if false
                Operation::LoadI {
                    // 4 -> r5
                    val: 4,
                    out: Reg::Global(5)
                },
                Operation::I2i {
                    // r5 -> r0
                    lhs: Reg::Global(5),
                    rhs: Reg::Global(0)
                },
                Operation::JumpI(Label::Numbered(1)), // jump to end
                Operation::Label(Label::Numbered(1))  // end
            ],
            ir_this(
                r#"
                if (1 < 2) {
                    3;
                } else {
                    4;
                }
            "#
            )
            .instructions
        );
    }

    fn ir_this(input: &'static str) -> IR {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        let ast_root = Parser::new(lexemes.into()).build_ast().unwrap();
        IRBuilder::new().build(ast_root).unwrap()
    }
}
