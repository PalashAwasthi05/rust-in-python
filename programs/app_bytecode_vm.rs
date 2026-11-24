use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instr {
    Push(i64),
    Pop,
    Dup,
    Swap,
    Over,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Lt,
    Not,
    Load(usize),
    Store(usize),
    Jmp(usize),
    Jz(usize),
    Call(usize),
    Ret,
    Print,
    Halt,
}

#[derive(Debug)]
enum VmError {
    StackUnderflow { pc: usize, instr: Instr },
    DivideByZero(usize),
    BadJump(usize),
    StepLimit(u64),
    CallDepth,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow { pc, instr } => write!(f, "stack underflow at pc={} ({:?})", pc, instr),
            VmError::DivideByZero(pc) => write!(f, "divide by zero at pc={}", pc),
            VmError::BadJump(t) => write!(f, "jump target {} out of range", t),
            VmError::StepLimit(n) => write!(f, "gave up after {} steps", n),
            VmError::CallDepth => write!(f, "call stack overflow"),
        }
    }
}

struct Frame {
    return_pc: usize,
    locals: [i64; 8],
}

struct Vm<'p> {
    program: &'p [Instr],
    stack: Vec<i64>,
    frames: Vec<Frame>,
    pc: usize,
    steps: u64,
    output: Vec<String>,
    histogram: HashMap<&'static str, u32>,
}

fn opname(i: &Instr) -> &'static str {
    match i {
        Instr::Push(_) => "push",
        Instr::Pop | Instr::Dup | Instr::Swap | Instr::Over => "stack",
        Instr::Add | Instr::Sub | Instr::Mul | Instr::Div | Instr::Mod => "arith",
        Instr::Eq | Instr::Lt | Instr::Not => "logic",
        Instr::Load(_) | Instr::Store(_) => "local",
        Instr::Jmp(_) | Instr::Jz(_) => "branch",
        Instr::Call(_) | Instr::Ret => "call",
        Instr::Print | Instr::Halt => "misc",
    }
}

impl<'p> Vm<'p> {
    fn new(program: &'p [Instr]) -> Vm<'p> {
        Vm {
            program,
            stack: Vec::with_capacity(64),
            frames: vec![Frame { return_pc: usize::MAX, locals: [0; 8] }],
            pc: 0,
            steps: 0,
            output: vec![],
            histogram: HashMap::new(),
        }
    }

    fn pop(&mut self, instr: Instr) -> Result<i64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow { pc: self.pc, instr })
    }

    fn binary(&mut self, instr: Instr, f: impl Fn(i64, i64) -> Option<i64>) -> Result<(), VmError> {
        let b = self.pop(instr)?;
        let a = self.pop(instr)?;
        let r = f(a, b).ok_or(VmError::DivideByZero(self.pc))?;
        self.stack.push(r);
        Ok(())
    }

    fn jump(&mut self, target: usize) -> Result<(), VmError> {
        if target >= self.program.len() {
            return Err(VmError::BadJump(target));
        }
        self.pc = target;
        Ok(())
    }

    fn run(&mut self, max_steps: u64) -> Result<(), VmError> {
        loop {
            if self.steps >= max_steps {
                return Err(VmError::StepLimit(self.steps));
            }
            let instr = match self.program.get(self.pc) {
                Some(i) => *i,
                None => return Ok(()),
            };
            self.steps += 1;
            *self.histogram.entry(opname(&instr)).or_insert(0) += 1;
            self.pc += 1;
            match instr {
                Instr::Push(v) => self.stack.push(v),
                Instr::Pop => { self.pop(instr)?; }
                Instr::Dup => {
                    let v = *self.stack.last().ok_or(VmError::StackUnderflow { pc: self.pc - 1, instr })?;
                    self.stack.push(v);
                }
                Instr::Swap => {
                    let n = self.stack.len();
                    if n < 2 {
                        return Err(VmError::StackUnderflow { pc: self.pc - 1, instr });
                    }
                    self.stack.swap(n - 1, n - 2);
                }
                Instr::Over => {
                    let n = self.stack.len();
                    if n < 2 {
                        return Err(VmError::StackUnderflow { pc: self.pc - 1, instr });
                    }
                    self.stack.push(self.stack[n - 2]);
                }
                Instr::Add => self.binary(instr, |a, b| Some(a.wrapping_add(b)))?,
                Instr::Sub => self.binary(instr, |a, b| Some(a.wrapping_sub(b)))?,
                Instr::Mul => self.binary(instr, |a, b| Some(a.wrapping_mul(b)))?,
                Instr::Div => self.binary(instr, |a, b| a.checked_div(b))?,
                Instr::Mod => self.binary(instr, i64::checked_rem_euclid)?,
                Instr::Eq => self.binary(instr, |a, b| Some((a == b) as i64))?,
                Instr::Lt => self.binary(instr, |a, b| Some(if a < b { 1 } else { 0 }))?,
                Instr::Not => {
                    let v = self.pop(instr)?;
                    self.stack.push((v == 0) as i64);
                }
                Instr::Load(slot) => {
                    let v = self.frames.last().unwrap().locals[slot];
                    self.stack.push(v);
                }
                Instr::Store(slot) => {
                    let v = self.pop(instr)?;
                    self.frames.last_mut().unwrap().locals[slot] = v;
                }
                Instr::Jmp(t) => self.jump(t)?,
                Instr::Jz(t) => {
                    if self.pop(instr)? == 0 {
                        self.jump(t)?;
                    }
                }
                Instr::Call(t) => {
                    if self.frames.len() >= 64 {
                        return Err(VmError::CallDepth);
                    }
                    let arg = self.pop(instr)?;
                    let mut locals = [0i64; 8];
                    locals[0] = arg;
                    self.frames.push(Frame { return_pc: self.pc, locals });
                    self.jump(t)?;
                }
                Instr::Ret => {
                    let frame = self.frames.pop().expect("frame");
                    if frame.return_pc == usize::MAX {
                        return Ok(());
                    }
                    self.pc = frame.return_pc;
                }
                Instr::Print => {
                    let v = self.pop(instr)?;
                    self.output.push(v.to_string());
                }
                Instr::Halt => return Ok(()),
            }
        }
    }
}

/// Assemble a tiny textual assembly language with labels.
fn assemble(src: &str) -> Result<Vec<Instr>, String> {
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let mut lines: Vec<(usize, Vec<&str>)> = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.split(';').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(label) = line.strip_suffix(':') {
            labels.insert(label, lines.len());
            continue;
        }
        lines.push((lineno + 1, line.split_whitespace().collect()));
    }
    let mut out = Vec::new();
    for (lineno, words) in &lines {
        let arg = |i: usize| -> Result<&str, String> {
            words.get(i).copied().ok_or_else(|| format!("line {}: missing operand", lineno))
        };
        let target = |name: &str| -> Result<usize, String> {
            labels.get(name).copied().ok_or(format!("line {}: unknown label {}", lineno, name))
        };
        let num = |s: &str| s.parse::<i64>().map_err(|e| format!("line {}: {}", lineno, e));
        let instr = match words[0] {
            "push" => Instr::Push(num(arg(1)?)?),
            "pop" => Instr::Pop,
            "dup" => Instr::Dup,
            "swap" => Instr::Swap,
            "over" => Instr::Over,
            "add" => Instr::Add,
            "sub" => Instr::Sub,
            "mul" => Instr::Mul,
            "div" => Instr::Div,
            "mod" => Instr::Mod,
            "eq" => Instr::Eq,
            "lt" => Instr::Lt,
            "not" => Instr::Not,
            "load" => Instr::Load(num(arg(1)?)? as usize),
            "store" => Instr::Store(num(arg(1)?)? as usize),
            "jmp" => Instr::Jmp(target(arg(1)?)?),
            "jz" => Instr::Jz(target(arg(1)?)?),
            "call" => Instr::Call(target(arg(1)?)?),
            "ret" => Instr::Ret,
            "print" => Instr::Print,
            "halt" => Instr::Halt,
            other => return Err(format!("line {}: unknown op '{}'", lineno, other)),
        };
        out.push(instr);
    }
    Ok(out)
}

const FACTORIAL: &str = "
    push 10
    call fact
    print
    push 20
    call fact
    print
    halt
fact:            ; local0 = n, local1 = acc
    push 1
    store 1
loop:
    load 0
    push 2
    lt
    not
    jz done
    load 1
    load 0
    mul
    store 1
    load 0
    push 1
    sub
    store 0
    jmp loop
done:
    load 1
    ret
";

const FIB_REC: &str = "
    push 11
    call fib
    print
    halt
fib:
    load 0
    push 2
    lt
    jz recurse
    load 0
    ret
recurse:
    load 0
    push 1
    sub
    call fib
    load 0
    push 2
    sub
    call fib
    add
    ret
";

const COLLATZ: &str = "
    push 27
    store 0
    push 0
    store 1
top:
    load 0
    push 1
    eq
    not
    jz end
    load 1
    push 1
    add
    store 1
    load 0
    push 2
    mod
    jz even
    load 0
    push 3
    mul
    push 1
    add
    store 0
    jmp top
even:
    load 0
    push 2
    div
    store 0
    jmp top
end:
    load 1
    print
";

fn main() {
    let programs = [
        ("factorial", FACTORIAL.to_string(), 10_000),
        ("fib(11)", FIB_REC.to_string(), 100_000),
        ("collatz(27)", COLLATZ.to_string(), 10_000),
        ("underflow", "push 1\nadd\n".to_string(), 100),
        ("divzero", "push 1\npush 0\ndiv\nprint".to_string(), 100),
        ("spin", "top:\njmp top".to_string(), 500),
        ("deep", "f:\npush 0\ncall f".to_string(), 10_000),
        ("badlabel", "jmp nowhere".to_string(), 10),
        ("badop", "push 1\nfrobnicate".to_string(), 10),
        ("badnum", "push 12x".to_string(), 10),
    ];
    for (name, src, limit) in programs.iter() {
        print!("{:<12}", name);
        let code = match assemble(src) {
            Ok(code) => code,
            Err(msg) => {
                println!("assembly failed: {}", msg);
                continue;
            }
        };
        let mut vm = Vm::new(&code);
        let result = vm.run(*limit);
        let out = vm.output.join(" ");
        match result {
            Ok(()) => println!("ok   [{}] in {} steps, stack={:?}", out, vm.steps, vm.stack),
            Err(e) => println!("err  {} (output so far: [{}])", e, out),
        }
        if *name == "fib(11)" {
            let mut hist: Vec<(&str, u32)> = vm.histogram.into_iter().collect();
            hist.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
            for (op, count) in hist {
                println!("            {:>7} {:>6}", op, count);
            }
        }
    }
    let code = assemble(FACTORIAL).unwrap();
    println!("factorial is {} instructions; first five: {:?}", code.len(), &code[..5]);
}
