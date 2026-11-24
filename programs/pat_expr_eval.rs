// Box-based expression tree with simplification and evaluation.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Num(i64),
    Var(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

use Expr::*;

fn num(n: i64) -> Box<Expr> { Box::new(Num(n)) }
fn var(s: &str) -> Box<Expr> { Box::new(Var(s.to_string())) }

fn simplify(e: &Expr) -> Expr {
    match e {
        Add(a, b) => match (simplify(a), simplify(b)) {
            (Num(0), x) | (x, Num(0)) => x,
            (Num(x), Num(y)) => Num(x + y),
            (x, y) => Add(Box::new(x), Box::new(y)),
        },
        Mul(a, b) => match (simplify(a), simplify(b)) {
            (Num(0), _) | (_, Num(0)) => Num(0),
            (Num(1), x) | (x, Num(1)) => x,
            (Num(x), Num(y)) => Num(x * y),
            (x, y) => Mul(Box::new(x), Box::new(y)),
        },
        Neg(inner) => match &**inner {
            Neg(x) => simplify(x),
            Num(n) => Num(-n),
            other => Neg(Box::new(simplify(other))),
        },
        Let(name, v, body) => Let(name.clone(), Box::new(simplify(v)), Box::new(simplify(body))),
        other => other.clone(),
    }
}

fn eval(e: &Expr, env: &mut HashMap<String, i64>) -> Result<i64, String> {
    Ok(match e {
        Num(n) => *n,
        Var(name) => *env.get(name).ok_or_else(|| format!("unbound variable `{}`", name))?,
        Neg(x) => -eval(x, env)?,
        Add(a, b) => eval(a, env)? + eval(b, env)?,
        Mul(a, b) => eval(a, env)? * eval(b, env)?,
        Let(name, v, body) => {
            let value = eval(v, env)?;
            let old = env.insert(name.clone(), value);
            let result = eval(body, env);
            match old {
                Some(prev) => { env.insert(name.clone(), prev); }
                None => { env.remove(name); }
            }
            result?
        }
    })
}

fn show(e: &Expr) -> String {
    match e {
        Num(n) if *n < 0 => format!("({})", n),
        Num(n) => n.to_string(),
        Var(v) => v.clone(),
        Neg(x) => format!("-{}", show(x)),
        Add(a, b) => format!("({} + {})", show(a), show(b)),
        Mul(a, b) => format!("{} * {}", show(a), show(b)),
        Let(n, v, b) => format!("let {} = {} in {}", n, show(v), show(b)),
    }
}

fn main() {
    let exprs = vec![
        Add(num(0), Box::new(Mul(num(1), var("x")))),
        Neg(Box::new(Neg(Box::new(Add(num(2), num(3)))))),
        Let("y".into(), num(4), Box::new(Mul(var("y"), Box::new(Add(var("y"), num(-1)))))),
        Mul(Box::new(Add(var("x"), num(0))), num(0)),
        Add(var("z"), num(1)),
        Neg(num(7)),
    ];
    let mut env = HashMap::new();
    env.insert("x".to_string(), 10);
    for e in &exprs {
        let s = simplify(e);
        match eval(&s, &mut env) {
            Ok(v) => println!("{:<40} => {:<20} = {}", show(e), show(&s), v),
            Err(msg) => println!("{:<40} => error: {}", show(e), msg),
        }
    }
    println!("{:?}", simplify(&exprs[1]));
    assert_eq!(simplify(&exprs[3]), Num(0));
}
