pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl Op {
    /**
     * Lower value is weaker precendence = needs to go higher in the AST.
     */
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Eq => 0,
            Op::Gt => 0,
            Op::Gte => 0,
            Op::Lt => 0,
            Op::Lte => 0,

            Op::Add => 1,
            Op::Sub => 1,
            Op::Mod => 1,

            Op::Mul => 2,
            Op::Div => 2,
        }
    }
}

pub fn char_n(c: char, n: usize) -> String {
    let mut out = String::new();

    for _ in 0..n {
        out.push(c);
    }

    out
}
