// Day 21 2022: expression tree from named monkeys, evaluate and solve for the unknown.
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Job {
    Num(i64),
    Op(String, char, String),
}

#[derive(Debug)]
enum Expr {
    Lit(i64),
    Human,
    Bin(Box<Expr>, char, Box<Expr>),
}

const INPUT: &str = r"root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";

fn apply(a: i64, op: char, b: i64) -> i64 {
    match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => a / b,
        _ => unreachable!(),
    }
}

fn eval(name: &str, jobs: &HashMap<String, Job>, cache: &mut HashMap<String, i64>) -> i64 {
    if let Some(&v) = cache.get(name) {
        return v;
    }
    let v = match &jobs[name] {
        Job::Num(n) => *n,
        Job::Op(a, op, b) => {
            let x = eval(a, jobs, cache);
            let y = eval(b, jobs, cache);
            apply(x, *op, y)
        }
    };
    cache.insert(name.to_string(), v);
    v
}

fn build(name: &str, jobs: &HashMap<String, Job>) -> Expr {
    if name == "humn" {
        return Expr::Human;
    }
    match &jobs[name] {
        Job::Num(n) => Expr::Lit(*n),
        Job::Op(a, op, b) => {
            let (l, r) = (build(a, jobs), build(b, jobs));
            match (&l, &r) {
                (Expr::Lit(x), Expr::Lit(y)) => Expr::Lit(apply(*x, *op, *y)),
                _ => Expr::Bin(Box::new(l), *op, Box::new(r)),
            }
        }
    }
}

fn solve(expr: &Expr, target: i64) -> i64 {
    match expr {
        Expr::Human => target,
        Expr::Lit(_) => panic!("no unknown"),
        Expr::Bin(l, op, r) => match (l.as_ref(), r.as_ref()) {
            (Expr::Lit(a), rest) => match op {
                '+' => solve(rest, target - a),
                '-' => solve(rest, a - target),
                '*' => solve(rest, target / a),
                '/' => solve(rest, a / target),
                _ => unreachable!(),
            },
            (rest, Expr::Lit(b)) => match op {
                '+' => solve(rest, target - b),
                '-' => solve(rest, target + b),
                '*' => solve(rest, target / b),
                '/' => solve(rest, target * b),
                _ => unreachable!(),
            },
            _ => panic!("unknown on both sides"),
        },
    }
}

fn depth(e: &Expr) -> usize {
    match e {
        Expr::Bin(l, _, r) => 1 + depth(l).max(depth(r)),
        _ => 0,
    }
}

fn main() {
    let jobs: HashMap<String, Job> = INPUT
        .lines()
        .map(|line| {
            let (name, rest) = line.split_once(": ").unwrap();
            let job = match rest.parse::<i64>() {
                Ok(n) => Job::Num(n),
                Err(_) => {
                    let parts: Vec<&str> = rest.split(' ').collect();
                    Job::Op(parts[0].to_string(), parts[1].chars().next().unwrap(), parts[2].to_string())
                }
            };
            (name.to_string(), job)
        })
        .collect();
    let mut cache = HashMap::new();
    println!("part 1: {}", eval("root", &jobs, &mut cache));
    let Job::Op(left, _, right) = jobs["root"].clone() else { panic!("root is a number") };
    let (l, r) = (build(&left, &jobs), build(&right, &jobs));
    let answer = match (&l, &r) {
        (Expr::Lit(v), e) | (e, Expr::Lit(v)) => solve(e, *v),
        _ => panic!(),
    };
    println!("tree depth {} / {}", depth(&l), depth(&r));
    println!("part 2: {answer}");
    println!("{:?}", r);
}
