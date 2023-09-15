use crate::ast::*;
use crate::shared::*;

type RegVal = usize;
type ImmVal = i32;
// This might be a hack for now, but a simple auto-inc usize will do it.
type Label = usize;
type CondCode = Vec<CondResult>;

#[derive(Debug)]
pub enum CondResult {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug)]
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
        addr: ImmVal,
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

pub struct IRBuilder;

impl IRBuilder {
    pub fn new() -> IRBuilder {
        IRBuilder
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
        unimplemented!()
    }
}

pub struct IR {
    instructions: Vec<Operation>,
}
