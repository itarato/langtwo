use std::collections::HashMap;

use crate::ir::*;

const STACK_SIZE: usize = 256;

struct Scope {
    registers: [i32; STACK_SIZE],
}

impl Scope {
    fn new() -> Scope {
        Scope {
            registers: [0; STACK_SIZE],
        }
    }
}

pub struct VM {
    ir: IR,
    stack: Vec<i32>,
    // Instruction pointer.
    ip: usize,
    label_map: HashMap<Label, usize>,
    frames: Vec<Scope>,
    return_ips: Vec<usize>,
}

impl VM {
    pub fn new(ir: IR) -> VM {
        let mut label_map = HashMap::new();
        for i in 0..ir.instructions.len() {
            match &ir.instructions[i] {
                Operation::Label(label) => {
                    label_map.insert(label.clone(), i);
                }
                _ => {}
            };
        }

        VM {
            ir,
            stack: vec![],
            ip: 0,
            label_map,
            frames: vec![Scope::new()],
            return_ips: vec![],
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.ip >= self.ir.instructions.len() {
                break;
            }

            match &self.ir.instructions[self.ip] {
                Operation::Label(_) => {} // Skip.
                Operation::Call(label) => {
                    self.return_ips.push(self.ip);
                    self.frames.push(Scope::new());
                    self.ip = self.label_map[label];
                }
                Operation::Return => {
                    self.ip = self.return_ips.pop().expect("Missing return IP");
                    self.frames.pop().expect("Cannot pop more frames");
                }
                Operation::Push(reg) => {
                    let value = self.reg_get(reg);
                    self.stack.push(value);
                }
                Operation::Pop(reg) => {
                    let value = self.stack.pop().expect("Empty stack");
                    self.reg_set(*reg, value);
                }
                Operation::Add { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, lhs_val + rhs_val);
                }
                Operation::Sub { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, lhs_val - rhs_val);
                }
                Operation::Mul { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, lhs_val * rhs_val);
                }
                Operation::Div { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, lhs_val / rhs_val);
                }
                Operation::LoadI { val, out } => {
                    self.reg_set(*out, *val);
                }
                Operation::I2i { lhs, rhs } => {
                    self.reg_set(*rhs, self.reg_get(lhs));
                }
                Operation::JumpI(label) => {
                    self.ip = self.label_map[label];
                }
                Operation::CmpEq { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, if lhs_val == rhs_val { 1 } else { 0 });
                }
                Operation::CmpLt { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, if lhs_val < rhs_val { 1 } else { 0 });
                }
                Operation::CmpLte { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, if lhs_val <= rhs_val { 1 } else { 0 });
                }
                Operation::CmpGt { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, if lhs_val > rhs_val { 1 } else { 0 });
                }
                Operation::CmpGte { lhs, rhs, out } => {
                    let lhs_val = self.reg_get(lhs);
                    let rhs_val = self.reg_get(rhs);
                    self.reg_set(*out, if lhs_val >= rhs_val { 1 } else { 0 });
                }
                Operation::CondBranch {
                    cond,
                    label_true,
                    label_false,
                } => {
                    let val = self.reg_get(cond);
                    self.ip = if val == 1 {
                        self.label_map[label_true]
                    } else {
                        self.label_map[label_false]
                    };
                }
                op => unimplemented!("Operation {:?} not implemented.", op),
            }

            self.ip += 1;
        }
    }

    #[inline]
    fn reg_set(&mut self, reg: Reg, value: i32) {
        match reg {
            Reg::Arp(arp_offs) => {
                self.frames.last_mut().expect("Missing frame").registers[arp_offs] = value
            }
            Reg::Global(offs) => {
                self.frames.first_mut().expect("Missing frame").registers[offs] = value
            }
        };
    }

    #[inline]
    fn reg_get(&self, reg: &Reg) -> i32 {
        match reg {
            Reg::Arp(arp_offs) => {
                self.frames.last().expect("Missing last frame").registers[*arp_offs]
            }
            Reg::Global(offs) => self.frames.first().expect("Missing first frame").registers[*offs],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ir_builder::*;
    use crate::lexer::*;
    use crate::parser::*;
    use crate::source_reader::*;
    use crate::vm::*;

    #[test]
    fn test_empty_program() {
        assert_eq!(None, vm_this(""));
    }

    #[test]
    fn test_expr_int() {
        assert_eq!(Some(3), vm_this("3;"));
    }

    #[test]
    fn test_expr_variable_assignment() {
        assert_eq!(Some(2), vm_this("a = 5; a - 3;"));
    }

    #[test]
    fn test_expr_variable_re_assignment() {
        assert_eq!(Some(9), vm_this("a = 3; b = 1; b = 9; a = b; a;"));
    }

    #[test]
    fn test_fn_call() {
        assert_eq!(
            Some(6),
            vm_this("fn addfive(x) { x + 5; } x = 1; addfive(x);")
        );
    }

    #[test]
    fn test_nested_calls() {
        assert_eq!(
            Some(10),
            vm_this(
                r#"
                fn powadd(a, b) {
                    pow(a) + pow(b);
                }
                fn pow(a) {
                    a * a;
                }
                fn half(x) {
                    out = x / two();
                    out;
                }
                fn two() {
                    2;
                }
                half(powadd(2, 4));
            "#
            )
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            Some(3),
            vm_this(
                r#"
                if (1 < 2) {
                    3;
                } else {
                    4;
                }
            "#
            )
        );

        assert_eq!(
            Some(4),
            vm_this(
                r#"
                if (6 <= 3) {
                    3;
                } else {
                    4;
                }
            "#
            )
        );

        assert_eq!(
            Some(0),
            vm_this(
                r#"
                if (3 == 2) {
                    3;
                }
            "#
            )
        );
    }

    #[test]
    fn test_fib() {
        assert_eq!(
            Some(144),
            vm_this(
                r#"
                fn fib(a, b, n) {
                    sum = a + b;
                    if (n > 1) {
                        fib(b, sum, n - 1);
                    } else {
                        sum;
                    }
                }

                fib(1, 1, 10);
            "#
            )
        );
    }

    #[test]
    fn test_loop_and_break() {
        assert_eq!(
            Some(10),
            vm_this("a = 1; loop { if (a >= 10) { break; } a = a + 1; } a;")
        );
    }

    fn vm_this(input: &'static str) -> Option<i32> {
        let reader = Box::new(StrReader::new(input));
        let lexemes = Lexer::new(reader).read_any().unwrap();
        let ast_root = Parser::new(lexemes.into()).build_ast().unwrap();
        let ir = IRBuilder::new().build(ast_root).unwrap();
        let mut vm = VM::new(ir);
        vm.run();
        vm.ir.return_reg.map(|reg| vm.reg_get(&reg))
    }
}
