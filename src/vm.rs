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
