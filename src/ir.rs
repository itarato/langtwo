pub type ImmVal = i32;
pub type CondCode = Vec<CondResult>;
pub type OutRegAndOps = (Reg, Vec<Operation>);
pub type MaybeOutRegAndOps = (Option<Reg>, Vec<Operation>);
pub type RegAddr = usize;

#[derive(Debug, PartialEq, Clone)]
pub enum Label {
    Named(String),
    Numbered(usize),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reg {
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
    Push(Reg),
    PushI(i32),
    Pop(Reg),

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
