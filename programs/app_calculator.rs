use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

impl Op {
    fn precedence(self) -> u8 {
        match self {
            Op::Add | Op::Sub => 1,
            Op::Mul | Op::Div | Op::Rem => 2,
            Op::Pow => 3,
        }
    }

    fn right_assoc(self) -> bool {
        self == Op::Pow
    }

    fn symbol(&self) -> char {
        match *self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
            Op::Rem => '%',
            Op::Pow => '^',
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i64),
    Ident(String),
    Op(Op),
    LParen,
    RParen,
    Comma,
    Assign,
}

#[derive(Debug)]
enum Expr {
    Num(i64),
    Var(String),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
enum CalcError {
    Lex { pos: usize, ch: char },
    Parse(String),
    UnknownVar(String),
    UnknownFn(String),
    DivByZero,
    Overflow,
    Arity { name: String, expected: usize, got: usize },
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CalcError::*;
        match self {
            Lex { pos, ch } => write!(f, "lex error: unexpected {:?} at column {}", ch, pos + 1),
            Parse(msg) => write!(f, "parse error: {}", msg),
            UnknownVar(v) => write!(f, "unknown variable `{}`", v),
            UnknownFn(n) => write!(f, "unknown function `{}`", n),
            DivByZero => f.write_str("division by zero"),
            Overflow => f.write_str("arithmetic overflow"),
            Arity { name, expected, got } => {
                write!(f, "{}() takes {} argument{}, got {}", name, expected,
                       if *expected == 1 { "" } else { "s" }, got)
            }
        }
    }
}

type Result<T> = std::result::Result<T, CalcError>;

fn tokenize(src: &str) -> Result<Vec<Token>> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = vec![];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => { i += 1; }
            '0'..='9' => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text.parse().map_err(|_| CalcError::Overflow)?;
                tokens.push(Token::Num(n));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
            }
            _ => {
                let tok = match c {
                    '+' => Token::Op(Op::Add),
                    '-' => Token::Op(Op::Sub),
                    '*' => Token::Op(Op::Mul),
                    '/' => Token::Op(Op::Div),
                    '%' => Token::Op(Op::Rem),
                    '^' => Token::Op(Op::Pow),
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    ',' => Token::Comma,
                    '=' => Token::Assign,
                    other => return Err(CalcError::Lex { pos: i, ch: other }),
                };
                tokens.push(tok);
                i += 1;
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self, min_prec: u8) -> Result<Expr> {
        let mut lhs = self.unary()?;
        while let Some(Token::Op(op)) = self.peek() {
            let op = *op;
            if op.precedence() < min_prec {
                break;
            }
            self.pos += 1;
            let next_min = if op.right_assoc() { op.precedence() } else { op.precedence() + 1 };
            let rhs = self.expr(next_min)?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(Token::Op(Op::Sub)) = self.peek() {
            self.pos += 1;
            let inner = self.unary()?;
            return Ok(Expr::Neg(Box::new(inner)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.next() {
            Some(Token::Num(n)) => Ok(Expr::Num(n)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(self.expr(1)?);
                            match self.next() {
                                Some(Token::Comma) => continue,
                                Some(Token::RParen) => break,
                                other => return Err(CalcError::Parse(format!("expected ',' or ')' in call, found {:?}", other))),
                            }
                        }
                    } else {
                        self.pos += 1;
                    }
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(Token::LParen) => {
                let e = self.expr(1)?;
                match self.next() {
                    Some(Token::RParen) => Ok(e),
                    _ => Err(CalcError::Parse("missing ')'".to_string())),
                }
            }
            Some(t) => Err(CalcError::Parse(format!("unexpected token {:?}", t))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }
}

fn show(e: &Expr) -> String {
    match e {
        Expr::Num(n) => n.to_string(),
        Expr::Var(v) => v.clone(),
        Expr::Neg(inner) => format!("(-{})", show(inner)),
        Expr::Bin(op, a, b) => format!("({} {} {})", show(a), op.symbol(), show(b)),
        Expr::Call(name, args) => {
            let parts: Vec<String> = args.iter().map(show).collect();
            format!("{}({})", name, parts.join(", "))
        }
    }
}

struct Env {
    vars: HashMap<String, i64>,
}

impl Env {
    fn eval(&self, e: &Expr) -> Result<i64> {
        Ok(match e {
            Expr::Num(n) => *n,
            Expr::Var(name) => *self.vars.get(name).ok_or_else(|| CalcError::UnknownVar(name.clone()))?,
            Expr::Neg(inner) => self.eval(inner)?.checked_neg().ok_or(CalcError::Overflow)?,
            Expr::Bin(op, a, b) => {
                let (x, y) = (self.eval(a)?, self.eval(b)?);
                let r = match op {
                    Op::Add => x.checked_add(y),
                    Op::Sub => x.checked_sub(y),
                    Op::Mul => x.checked_mul(y),
                    Op::Div | Op::Rem if y == 0 => return Err(CalcError::DivByZero),
                    Op::Div => x.checked_div(y),
                    Op::Rem => x.checked_rem(y),
                    Op::Pow => {
                        if y < 0 { Some(0) } else { x.checked_pow(y as u32) }
                    }
                };
                r.ok_or(CalcError::Overflow)?
            }
            Expr::Call(name, args) => {
                let vals = args.iter().map(|a| self.eval(a)).collect::<Result<Vec<i64>>>()?;
                let check = |n: usize| -> Result<()> {
                    if vals.len() == n { Ok(()) } else {
                        Err(CalcError::Arity { name: name.clone(), expected: n, got: vals.len() })
                    }
                };
                match name.as_str() {
                    "abs" => { check(1)?; vals[0].abs() }
                    "max" => vals.iter().copied().max().ok_or(CalcError::Arity { name: name.clone(), expected: 1, got: 0 })?,
                    "min" => *vals.iter().min().ok_or(CalcError::Arity { name: name.clone(), expected: 1, got: 0 })?,
                    "gcd" => {
                        check(2)?;
                        let (mut a, mut b) = (vals[0].abs(), vals[1].abs());
                        while b != 0 {
                            let t = a % b;
                            a = b;
                            b = t;
                        }
                        a
                    }
                    "sum" => vals.iter().sum(),
                    _ => return Err(CalcError::UnknownFn(name.to_string())),
                }
            }
        })
    }
}

fn run_line(env: &mut Env, line: &str) -> Result<String> {
    let mut tokens = tokenize(line)?;
    let mut target = None;
    if let [Token::Ident(name), Token::Assign, ..] = tokens.as_slice() {
        target = Some(name.clone());
        tokens.drain(..2);
    }
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.expr(1)?;
    if parser.pos < parser.tokens.len() {
        return Err(CalcError::Parse(format!("trailing tokens starting at {:?}", parser.tokens[parser.pos])));
    }
    let value = env.eval(&expr)?;
    let rendered = show(&expr);
    Ok(match target {
        Some(name) => {
            let old = env.vars.insert(name.clone(), value);
            match old {
                Some(prev) => format!("{} = {} (was {})", name, value, prev),
                None => format!("{} = {}", name, value),
            }
        }
        None => format!("{} => {}", rendered, value),
    })
}

fn main() {
    let mut env = Env { vars: HashMap::new() };
    env.vars.insert("answer".to_string(), 42);
    let script = [
        "1 + 2 * 3",
        "(1 + 2) * 3",
        "2 ^ 3 ^ 2",
        "-2 ^ 2",
        "100 / 7 % 4",
        "x = 10",
        "y = x * x - 1",
        "x = x + y",
        "max(x, y, answer) - min(3, -4)",
        "gcd(84, 36) + abs(-5)",
        "sum(1, 2, 3, 4, 5) * 2",
        "sum()",
        "10 / (5 - 5)",
        "9223372036854775807 + 1",
        "2 ^ 70",
        "z + 1",
        "foo(1)",
        "abs(1, 2)",
        "gcd(1)",
        "3 + # 4",
        "(1 + 2",
        "1 2",
        "max()",
        "",
    ];
    let mut ok = 0;
    let mut failed = Vec::new();
    for (n, line) in script.iter().enumerate() {
        match run_line(&mut env, line) {
            Ok(out) => {
                ok += 1;
                println!("[{:2}] {}", n + 1, out);
            }
            Err(e) => {
                println!("[{:2}] error in {:?}: {}", n + 1, line, e);
                failed.push(n + 1);
            }
        }
    }
    println!("{} ok, {} failed: {:?}", ok, failed.len(), failed);
    let mut names: Vec<_> = env.vars.iter().collect();
    names.sort();
    for (k, v) in names {
        println!("  {} = {}", k, v);
    }
    assert_eq!(run_line(&mut env, "1/0"), Err(CalcError::DivByZero));
}
