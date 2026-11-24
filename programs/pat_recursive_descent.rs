// Recursive-descent parser: while-let with @ or-patterns, loop break values, Box trees.
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Token { Num(i64), Plus, Minus, Star, Slash, LParen, RParen }

#[derive(Debug)]
enum Ast { Lit(i64), Bin(Box<Ast>, char, Box<Ast>), Neg(Box<Ast>) }

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ast::Lit(n) => write!(f, "{}", n),
            Ast::Bin(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Ast::Neg(inner) => write!(f, "-{}", inner),
        }
    }
}

struct P<'a> { toks: &'a [Token], i: usize }

impl<'a> P<'a> {
    fn peek(&self) -> Option<&Token> { self.toks.get(self.i) }
    fn bump(&mut self) -> Option<Token> { let t = self.toks.get(self.i).cloned(); self.i += 1; t }
    fn expr(&mut self) -> Result<Ast, String> {
        let mut lhs = self.term()?;
        while let Some(op @ (Token::Plus | Token::Minus)) = self.peek().cloned() {
            self.bump();
            let rhs = self.term()?;
            lhs = Ast::Bin(Box::new(lhs), if op == Token::Plus { '+' } else { '-' }, Box::new(rhs));
        }
        Ok(lhs)
    }
    fn term(&mut self) -> Result<Ast, String> {
        let mut lhs = self.atom()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => '*',
                Some(Token::Slash) => '/',
                _ => break Ok(lhs),
            };
            self.bump();
            lhs = Ast::Bin(Box::new(lhs), op, Box::new(self.atom()?));
        }
    }
    fn atom(&mut self) -> Result<Ast, String> {
        match self.bump() {
            Some(Token::Num(n)) => Ok(Ast::Lit(n)),
            Some(Token::Minus) => Ok(Ast::Neg(Box::new(self.atom()?))),
            Some(Token::LParen) => {
                let e = self.expr()?;
                match self.bump() {
                    Some(Token::RParen) => Ok(e),
                    other => Err(format!("expected ), got {:?}", other)),
                }
            }
            t => Err(format!("unexpected {:?}", t)),
        }
    }
}

fn eval(a: &Ast) -> Option<i64> {
    Some(match a {
        Ast::Lit(n) => *n,
        Ast::Neg(x) => eval(x)?.checked_neg()?,
        Ast::Bin(l, '/', r) => {
            let d = eval(r)?;
            if d == 0 { return None; }
            eval(l)? / d
        }
        Ast::Bin(l, op, r) => {
            let (a, b) = (eval(l)?, eval(r)?);
            match op { '+' => a + b, '-' => a - b, '*' => a * b, _ => unreachable!() }
        }
    })
}

fn lex(s: &str) -> Vec<Token> {
    let mut out = vec![];
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            c if c.is_ascii_digit() => {
                let start = i;
                while i < cs.len() && cs[i].is_ascii_digit() { i += 1; }
                out.push(Token::Num(cs[start..i].iter().collect::<String>().parse().unwrap()));
                continue;
            }
            '+' => out.push(Token::Plus),
            '-' => out.push(Token::Minus),
            '*' => out.push(Token::Star),
            '/' => out.push(Token::Slash),
            '(' => out.push(Token::LParen),
            ')' => out.push(Token::RParen),
            _ => {}
        }
        i += 1;
    }
    out
}

fn main() {
    let mut cache: HashMap<&str, Option<i64>> = HashMap::new();
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "-(4 - 10) / 2", "7 / (3 - 3)", "2 * (3", "((5))"] {
        let toks = lex(src);
        let mut p = P { toks: &toks, i: 0 };
        match p.expr() {
            Ok(ast) => {
                let v = *cache.entry(src).or_insert_with(|| eval(&ast));
                println!("{:<16} {:<24} {:?}", src, ast.to_string(), v);
            }
            Err(e) => println!("{:<16} error: {}", src, e),
        }
    }
    let x = 5;
    let msg = match x { n if n < 0 => "neg", 0 => "zero", 1 | 2 | 3 => "small", _ => "large" };
    println!("{msg}");
    let nested = Some(Ok::<(i32, &str), String>((3, "three")));
    if let Some(Ok((n @ 1..=5, name))) = nested {
        println!("{n} {name}");
    }
    let v = vec![Some(1), None, Some(3)];
    let total: i32 = v.iter().flatten().sum();
    let firsts: Vec<_> = v.iter().map(|o| match *o { Some(x) if x > 1 => x * 10, Some(x) => x, None => -1 }).collect();
    println!("{total} {firsts:?}");
    let t = (1, (2.5, 'c'), [4, 5]);
    let (a, (b, c), [d, e]) = t;
    println!("{a} {b} {c} {d} {e}");
    let refs = (&1, &mut 2);
    match refs {
        (&1, &mut ref mut y) => { *y += 1; println!("y {}", y) }
        _ => {}
    }
}
