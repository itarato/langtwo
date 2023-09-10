use crate::shared::*;

macro_rules! space {
    ($indent:expr) => {
        char_n(' ', $indent)
    };
}

const INDENT_INC: usize = 4;

pub trait AstDump {
    fn ast_dump(&self, indent: usize) -> String;
}

#[derive(Debug)]
pub struct AstProgram<'s> {
    pub statements: Vec<AstStatement<'s>>,
}

impl AstDump for AstProgram<'_> {
    fn ast_dump(&self, indent: usize) -> String {
        format!(
            "{}prg\n{}",
            space!(indent),
            self.statements
                .iter()
                .map(|e| e.ast_dump(indent + INDENT_INC))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Debug)]
pub enum AstStatement<'s> {
    FnDef {
        name: &'s str,
        args: Vec<&'s str>,
        block: Vec<AstBlockLine<'s>>,
    },
    BlockLine(AstBlockLine<'s>),
}

impl AstDump for AstStatement<'_> {
    fn ast_dump(&self, indent: usize) -> String {
        match self {
            AstStatement::FnDef {
                name: _,
                args: _,
                block,
            } => format!(
                "{}stmt / fndef\n{}",
                space!(indent),
                block
                    .iter()
                    .map(|e| e.ast_dump(indent + INDENT_INC))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            AstStatement::BlockLine(line) => {
                format!(
                    "{}stmt\n{}",
                    space!(indent),
                    line.ast_dump(indent + INDENT_INC)
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstBlockLine<'s> {
    Expr(AstExpr<'s>),
}

impl AstDump for AstBlockLine<'_> {
    fn ast_dump(&self, indent: usize) -> String {
        match self {
            AstBlockLine::Expr(expr) => {
                format!(
                    "{}blockline\n{}",
                    space!(indent),
                    expr.ast_dump(indent + INDENT_INC)
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstExpr<'s> {
    FnCall {
        name: &'s str,
        args: Vec<AstExpr<'s>>,
    },
    Str(&'s str),
    Int(i32),
    Name(&'s str),
    Assignment {
        varname: &'s str,
        expr: Box<AstExpr<'s>>,
    },
}

impl AstDump for AstExpr<'_> {
    fn ast_dump(&self, indent: usize) -> String {
        match self {
            AstExpr::FnCall { .. } => format!("{}expr / fncall", space!(indent)),
            AstExpr::Str(_) => format!("{}expr / str", space!(indent)),
            AstExpr::Int(_) => format!("{}expr / int", space!(indent)),
            AstExpr::Name(_) => format!("{}expr / name", space!(indent)),
            AstExpr::Assignment { varname: _, expr } => {
                format!(
                    "{}expr / assign\n{}",
                    space!(indent),
                    (*expr).ast_dump(indent + INDENT_INC)
                )
            }
        }
    }
}
