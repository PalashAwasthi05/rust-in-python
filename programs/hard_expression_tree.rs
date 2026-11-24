use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, Op, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op { Add, Sub, Mul, Div, Pow }

impl Op {
    fn prec(self) -> u8 { match self { Op::Add | Op::Sub => 1, Op::Mul | Op::Div => 2, Op::Pow => 3 } }
    fn sym(self) -> char { match self { Op::Add => '+', Op::Sub => '-', Op::Mul => '*', Op::Div => '/', Op::Pow => '^' } }
}

use Expr::*;

fn num(n: f64) -> Box<Expr> { Box::new(Num(n)) }
fn var(s: &str) -> Box<Expr> { Box::new(Var(s.into())) }
fn bin(l: Box<Expr>, op: Op, r: Box<Expr>) -> Box<Expr> { Box::new(Bin(l, op, r)) }

fn eval(e: &Expr, env: &mut BTreeMap<String, f64>) -> Result<f64, String> {
    Ok(match e {
        Num(n) => *n,
        Var(v) => *env.get(v).ok_or(format!("unbound {}", v))?,
        Neg(inner) => -eval(inner, env)?,
        Bin(l, op, r) => {
            let (a, b) = (eval(l, env)?, eval(r, env)?);
            match op {
                Op::Add => a + b,
                Op::Sub => a - b,
                Op::Mul => a * b,
                Op::Div if b == 0.0 => return Err("divide by zero".into()),
                Op::Div => a / b,
                Op::Pow => a.powf(b),
            }
        }
        Let(name, val, body) => {
            let v = eval(val, env)?;
            let old = env.insert(name.clone(), v);
            let result = eval(body, env);
            match old { Some(o) => { env.insert(name.clone(), o); } None => { env.remove(name); } }
            result?
        }
    })
}

fn show(e: &Expr, parent: u8) -> String {
    match e {
        Num(n) => format!("{}", n),
        Var(v) => v.clone(),
        Neg(inner) => format!("-{}", show(inner, 4)),
        Bin(l, op, r) => {
            let s = format!("{} {} {}", show(l, op.prec()), op.sym(), show(r, op.prec() + 1));
            if op.prec() < parent { format!("({})", s) } else { s }
        }
        Let(n, v, b) => format!("(let {} = {} in {})", n, show(v, 0), show(b, 0)),
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", show(self, 0)) }
}

fn count_nodes(e: &Expr) -> usize {
    match e {
        Num(_) | Var(_) => 1,
        Neg(i) => 1 + count_nodes(i),
        Bin(l, _, r) | Let(_, l, r) => 1 + count_nodes(l) + count_nodes(r),
    }
}

fn main() {
    let mut env = BTreeMap::new();
    env.insert("x".to_string(), 3.0);
    let exprs: Vec<Box<Expr>> = vec![
        bin(num(1.0), Op::Add, bin(num(2.0), Op::Mul, num(3.0))),
        bin(bin(num(1.0), Op::Add, num(2.0)), Op::Mul, num(3.0)),
        bin(num(10.0), Op::Sub, bin(num(4.0), Op::Sub, num(1.0))),
        bin(var("x"), Op::Pow, num(2.0)),
        Box::new(Neg(bin(var("x"), Op::Add, num(1.0)))),
        Box::new(Let("y".into(), bin(var("x"), Op::Mul, num(2.0)), bin(var("y"), Op::Div, num(4.0)))),
        bin(num(1.0), Op::Div, bin(var("x"), Op::Sub, num(3.0))),
        bin(var("z"), Op::Add, num(1.0)),
    ];
    for e in &exprs {
        println!("{:<34} = {:?}  [{} nodes]", e.to_string(), eval(e, &mut env), count_nodes(e));
    }
    println!("env after: {:?}", env);
}
