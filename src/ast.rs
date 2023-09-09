pub trait AstDump {
    fn ast_dump(&self) -> String;
}

#[derive(Debug)]
pub struct AstProgram<'s> {
    pub statements: Vec<AstStatement<'s>>,
}

impl AstDump for AstProgram<'_> {
    fn ast_dump(&self) -> String {
        format!(
            "prg({})",
            self.statements
                .iter()
                .map(|e| e.ast_dump())
                .collect::<Vec<String>>()
                .join(",")
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
    fn ast_dump(&self) -> String {
        match self {
            AstStatement::FnDef {
                name: _,
                args: _,
                block,
            } => format!(
                "stmt(fndef({}))",
                block
                    .iter()
                    .map(|e| e.ast_dump())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            AstStatement::BlockLine(line) => format!("stmt({})", line.ast_dump()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstBlockLine<'s> {
    Expr(AstExpr<'s>),
}

impl AstDump for AstBlockLine<'_> {
    fn ast_dump(&self) -> String {
        match self {
            AstBlockLine::Expr(expr) => format!("blockline({})", expr.ast_dump()),
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
    fn ast_dump(&self) -> String {
        match self {
            AstExpr::FnCall { .. } => "expr(fncall)".into(),
            AstExpr::Str(_) => "expr(str)".into(),
            AstExpr::Int(_) => "expr(int)".into(),
            AstExpr::Name(_) => "expr(name)".into(),
            AstExpr::Assignment { varname: _, expr } => {
                format!("expr(assign({}))", (*expr).ast_dump())
            }
        }
    }
}
