// A small stack VM: instruction decoding, while-let over a stack, unreachable!.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Push(i64),
    Add,
    Sub,
    Mul,
    Dup,
    Swap,
    Jz(usize),
    Jmp(usize),
    Print,
    Halt,
}

#[derive(Debug)]
enum VmError {
    StackUnderflow { pc: usize, op: Op },
    BadJump(usize),
}

struct Vm {
    stack: Vec<i64>,
    pc: usize,
    output: Vec<i64>,
}

impl Vm {
    fn pop(&mut self, op: Op) -> Result<i64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow { pc: self.pc, op })
    }

    fn run(&mut self, prog: &[Op]) -> Result<usize, VmError> {
        let mut steps = 0;
        while let Some(&op) = prog.get(self.pc) {
            steps += 1;
            self.pc += 1;
            match op {
                Op::Push(v) => self.stack.push(v),
                Op::Add | Op::Sub | Op::Mul => {
                    let b = self.pop(op)?;
                    let a = self.pop(op)?;
                    self.stack.push(match op {
                        Op::Add => a + b,
                        Op::Sub => a - b,
                        Op::Mul => a * b,
                        _ => unreachable!("not arithmetic: {:?}", op),
                    });
                }
                Op::Dup => {
                    let v = *self.stack.last().ok_or(VmError::StackUnderflow { pc: self.pc - 1, op })?;
                    self.stack.push(v);
                }
                Op::Swap => {
                    let n = self.stack.len();
                    if n < 2 {
                        return Err(VmError::StackUnderflow { pc: self.pc - 1, op });
                    }
                    self.stack.swap(n - 1, n - 2);
                }
                Op::Jz(t) | Op::Jmp(t) if t > prog.len() => return Err(VmError::BadJump(t)),
                Op::Jz(t) => {
                    if self.pop(op)? == 0 {
                        self.pc = t;
                    }
                }
                Op::Jmp(t) => self.pc = t,
                Op::Print => {
                    let v = self.pop(op)?;
                    self.output.push(v);
                }
                Op::Halt => break,
            }
        }
        Ok(steps)
    }
}

fn main() {
    // countdown from 5, printing each, then 5*4 at the end
    let countdown = vec![
        Op::Push(5),
        Op::Dup, Op::Print,
        Op::Push(1), Op::Sub,
        Op::Dup, Op::Jz(8),
        Op::Jmp(1),
        Op::Push(5), Op::Push(4), Op::Mul, Op::Print,
        Op::Halt,
        Op::Push(999), Op::Print,
    ];
    let programs: Vec<(&str, Vec<Op>)> = vec![
        ("countdown", countdown),
        ("underflow", vec![Op::Push(1), Op::Add]),
        ("badjump", vec![Op::Push(0), Op::Jz(99)]),
        ("swap", vec![Op::Push(2), Op::Push(7), Op::Swap, Op::Sub, Op::Print]),
    ];
    for (name, prog) in &programs {
        let mut vm = Vm { stack: Vec::new(), pc: 0, output: vec![] };
        match vm.run(prog) {
            Ok(steps) => println!("{}: ok in {} steps, output {:?}, stack {:?}", name, steps, vm.output, vm.stack),
            Err(VmError::StackUnderflow { pc, op }) => println!("{}: underflow at {} ({:?})", name, pc, op),
            Err(e @ VmError::BadJump(_)) => println!("{}: {:?}", name, e),
        }
    }
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        if top == 2 {
            stack.push(20);
            stack.push(10);
            continue;
        }
        print!("{} ", top);
    }
    println!();
}
