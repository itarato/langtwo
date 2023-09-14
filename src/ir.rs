type RegVal = usize;
type ImmVal = i32;
type Label = String;
type CondCode = Vec<CondResult>;

pub enum CondResult {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

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
