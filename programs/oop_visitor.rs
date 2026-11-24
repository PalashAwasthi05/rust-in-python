// Visitor over an AST enum: a trait with default walk methods, plus several visitors.
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, Op, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn symbol(&self) -> char {
        match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
        }
    }
}

trait Visitor {
    type Out;
    fn visit(&mut self, e: &Expr) -> Self::Out;
}

// Mutable-state visitor with a default "walk" helper.
trait Walker {
    fn enter(&mut self, _e: &Expr) {}
    fn walk(&mut self, e: &Expr) {
        self.enter(e);
        match e {
            Expr::Num(_) | Expr::Var(_) => {}
            Expr::Neg(inner) => self.walk(inner),
            Expr::Bin(l, _, r) => {
                self.walk(l);
                self.walk(r);
            }
            Expr::Let(_, v, body) => {
                self.walk(v);
                self.walk(body);
            }
            Expr::Call(_, args) => {
                for a in args {
                    self.walk(a);
                }
            }
        }
    }
}

struct Printer;

impl Visitor for Printer {
    type Out = String;
    fn visit(&mut self, e: &Expr) -> String {
        match e {
            Expr::Num(n) => format!("{}", n),
            Expr::Var(v) => v.clone(),
            Expr::Neg(inner) => format!("-{}", self.visit(inner)),
            Expr::Bin(l, op, r) => format!("({} {} {})", self.visit(l), op.symbol(), self.visit(r)),
            Expr::Let(name, v, body) => format!("let {} = {} in {}", name, self.visit(v), self.visit(body)),
            Expr::Call(f, args) => {
                let parts: Vec<String> = args.iter().map(|a| self.visit(a)).collect();
                format!("{}({})", f, parts.join(", "))
            }
        }
    }
}

struct Evaluator {
    scopes: Vec<HashMap<String, f64>>,
}

impl Evaluator {
    fn lookup(&self, name: &str) -> Result<f64, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Ok(*v);
            }
        }
        Err(format!("unbound variable `{}`", name))
    }
}

impl Visitor for Evaluator {
    type Out = Result<f64, String>;
    fn visit(&mut self, e: &Expr) -> Result<f64, String> {
        Ok(match e {
            Expr::Num(n) => *n,
            Expr::Var(v) => self.lookup(v)?,
            Expr::Neg(inner) => -self.visit(inner)?,
            Expr::Bin(l, op, r) => {
                let a = self.visit(l)?;
                let b = self.visit(r)?;
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => {
                        if b == 0.0 {
                            return Err("division by zero".to_string());
                        }
                        a / b
                    }
                }
            }
            Expr::Let(name, v, body) => {
                let value = self.visit(v)?;
                let mut scope = HashMap::new();
                scope.insert(name.clone(), value);
                self.scopes.push(scope);
                let result = self.visit(body);
                self.scopes.pop();
                result?
            }
            Expr::Call(f, args) => {
                let vals = args.iter().map(|a| self.visit(a)).collect::<Result<Vec<f64>, String>>()?;
                match (f.as_str(), vals.as_slice()) {
                    ("max", [a, b]) => a.max(*b),
                    ("sqrt", [x]) => x.sqrt(),
                    ("sum", rest) => rest.iter().sum(),
                    (name, _) => return Err(format!("unknown function {}/{}", name, vals.len())),
                }
            }
        })
    }
}

#[derive(Default)]
struct Stats {
    nodes: usize,
    vars: Vec<String>,
    depth: usize,
    max_depth: usize,
}

impl Walker for Stats {
    fn enter(&mut self, e: &Expr) {
        self.nodes += 1;
        if let Expr::Var(v) = e {
            if !self.vars.contains(v) {
                self.vars.push(v.clone());
            }
        }
    }
}

impl Stats {
    fn depth_of(&mut self, e: &Expr) {
        self.depth += 1;
        self.max_depth = self.max_depth.max(self.depth);
        match e {
            Expr::Neg(i) => self.depth_of(i),
            Expr::Bin(l, _, r) | Expr::Let(_, l, r) => {
                self.depth_of(l);
                self.depth_of(r);
            }
            Expr::Call(_, args) => args.iter().for_each(|a| self.depth_of(a)),
            _ => {}
        }
        self.depth -= 1;
    }
}

// A folding visitor that rewrites the tree (constant folding).
fn fold(e: Expr) -> Expr {
    match e {
        Expr::Bin(l, op, r) => match (fold(*l), fold(*r)) {
            (Expr::Num(a), Expr::Num(b)) if op != Op::Div || b != 0.0 => Expr::Num(match op {
                Op::Add => a + b,
                Op::Sub => a - b,
                Op::Mul => a * b,
                Op::Div => a / b,
            }),
            (l, r) => Expr::Bin(Box::new(l), op, Box::new(r)),
        },
        Expr::Neg(inner) => match fold(*inner) {
            Expr::Num(n) => Expr::Num(-n),
            other => Expr::Neg(Box::new(other)),
        },
        Expr::Let(n, v, b) => Expr::Let(n, Box::new(fold(*v)), Box::new(fold(*b))),
        Expr::Call(f, args) => Expr::Call(f, args.into_iter().map(fold).collect()),
        leaf => leaf,
    }
}

fn num(n: f64) -> Box<Expr> {
    Box::new(Expr::Num(n))
}
fn var(s: &str) -> Box<Expr> {
    Box::new(Expr::Var(s.to_string()))
}

fn main() {
    let program = Expr::Let(
        "x".into(),
        Box::new(Expr::Bin(num(2.0), Op::Mul, num(3.0))),
        Box::new(Expr::Bin(
            var("x"),
            Op::Add,
            Box::new(Expr::Call("max".into(), vec![Expr::Neg(var("x")), Expr::Bin(var("x"), Op::Div, num(4.0))])),
        )),
    );
    let mut printer = Printer;
    println!("{}", printer.visit(&program));
    let mut ev = Evaluator { scopes: vec![HashMap::new()] };
    println!("= {:?}", ev.visit(&program));

    let mut stats = Stats::default();
    stats.walk(&program);
    stats.depth_of(&program);
    println!("nodes={} vars={:?} max_depth={}", stats.nodes, stats.vars, stats.max_depth);

    let bad = vec![
        Expr::Bin(num(1.0), Op::Div, num(0.0)),
        *var("y"),
        Expr::Call("nope".into(), vec![Expr::Num(1.0)]),
        Expr::Call("sum".into(), vec![Expr::Num(1.0), Expr::Num(2.5), Expr::Num(3.0)]),
        Expr::Call("sqrt".into(), vec![Expr::Num(16.0)]),
    ];
    for e in &bad {
        println!("{} => {:?}", Printer.visit(e), ev.visit(e));
    }

    let folded = fold(Expr::Bin(
        Box::new(Expr::Bin(num(1.0), Op::Add, num(2.0))),
        Op::Mul,
        Box::new(Expr::Bin(var("z"), Op::Sub, Box::new(Expr::Neg(num(4.0))))),
    ));
    println!("folded: {}", Printer.visit(&folded));
    println!("{:?}", folded);
}
