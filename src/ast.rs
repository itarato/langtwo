#[derive(Debug)]
pub struct AstProgram<'s> {
    pub statements: Vec<AstStatement<'s>>,
}

#[derive(Debug)]
pub enum AstStatement<'s> {
    FnDef {
        name: &'s str,
        block: Vec<AstBlockLine<'s>>,
    },
    BlockLine(AstBlockLine<'s>),
}

#[derive(Debug, Clone)]
pub enum AstBlockLine<'s> {
    Expr(AstExpr<'s>),
}

#[derive(Debug, Clone)]
pub enum AstExpr<'s> {
    FnCall {
        name: &'s str,
        args: Vec<AstExpr<'s>>,
    },
    Str(&'s str),
    Int(i32),
}
