#[derive(Debug, Clone, Copy, PartialEq)]
enum Instr {
    Push(i64),
    Add, Sub, Mul, Div,
    Dup, Swap, Pop,
    Load(usize), Store(usize),
    Jmp(usize), Jz(usize),
    Print, Halt,
}

#[derive(Debug)]
enum VmError { StackUnderflow(usize), DivZero(usize), BadJump(usize) }

struct Vm { stack: Vec<i64>, vars: [i64; 4], pc: usize, steps: usize, output: Vec<i64> }

impl Vm {
    fn new() -> Self { Vm { stack: Vec::new(), vars: [0; 4], pc: 0, steps: 0, output: Vec::new() } }

    fn pop(&mut self) -> Result<i64, VmError> { self.stack.pop().ok_or(VmError::StackUnderflow(self.pc)) }

    fn run(&mut self, prog: &[Instr]) -> Result<(), VmError> {
        while self.pc < prog.len() {
            let ins = prog[self.pc];
            self.pc += 1;
            self.steps += 1;
            match ins {
                Instr::Push(v) => self.stack.push(v),
                Instr::Add | Instr::Sub | Instr::Mul | Instr::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(match ins {
                        Instr::Add => a + b,
                        Instr::Sub => a - b,
                        Instr::Mul => a * b,
                        _ => { if b == 0 { return Err(VmError::DivZero(self.pc - 1)); } a / b }
                    });
                }
                Instr::Dup => { let a = self.pop()?; self.stack.extend([a, a]); }
                Instr::Swap => { let b = self.pop()?; let a = self.pop()?; self.stack.extend([b, a]); }
                Instr::Pop => { self.pop()?; }
                Instr::Load(i) => self.stack.push(self.vars[i]),
                Instr::Store(i) => self.vars[i] = self.pop()?,
                Instr::Jmp(t) => { if t > prog.len() { return Err(VmError::BadJump(t)); } self.pc = t; }
                Instr::Jz(t) => { if self.pop()? == 0 { self.pc = t; } }
                Instr::Print => { let v = self.pop()?; self.output.push(v); }
                Instr::Halt => break,
            }
        }
        Ok(())
    }
}

fn main() {
    use Instr::*;
    // factorial of 10: var0 = n, var1 = acc
    let fact = vec![
        Push(10), Store(0), Push(1), Store(1),
        Load(0), Jz(15),
        Load(1), Load(0), Mul, Store(1),
        Load(0), Push(1), Sub, Store(0),
        Jmp(4),
        Load(1), Print, Halt,
    ];
    let mut vm = Vm::new();
    println!("fact: {:?} output {:?} steps {}", vm.run(&fact), vm.output, vm.steps);

    // print the first 8 fibonacci numbers
    let fib = vec![
        Push(0), Store(0), Push(1), Store(1), Push(8), Store(2),
        Load(2), Jz(22),
        Load(0), Print,
        Load(0), Load(1), Add, Load(1), Store(0), Store(1),
        Load(2), Push(1), Sub, Store(2),
        Jmp(6), Halt,
        Halt,
    ];
    let mut vm = Vm::new();
    println!("fib: {:?} output {:?}", vm.run(&fib), vm.output);

    let programs: Vec<(&str, Vec<Instr>)> = vec![
        ("arith", vec![Push(7), Push(3), Swap, Sub, Dup, Mul, Print]),
        ("underflow", vec![Push(1), Add]),
        ("divzero", vec![Push(4), Push(0), Div]),
        ("badjump", vec![Jmp(99)]),
    ];
    for (name, p) in programs {
        let mut vm = Vm::new();
        let r = vm.run(&p);
        println!("{:<9} {:?} out={:?} stack={:?}", name, r, vm.output, vm.stack);
    }
}
