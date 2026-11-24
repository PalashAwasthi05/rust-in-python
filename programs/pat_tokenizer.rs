// A tokenizer with char patterns, peekable iterators, and let-else.
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
enum Tok {
    Num(f64),
    Ident(String),
    Str(String),
    Op(char),
    Arrow,
    Eq,
    EqEq,
    LParen,
    RParen,
}

fn lex_number(first: char, it: &mut Peekable<Chars>) -> Tok {
    let mut s = String::from(first);
    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' | '.' | '_' => {
                if c != '_' { s.push(c); }
                it.next();
            }
            _ => break,
        }
    }
    Tok::Num(s.parse().unwrap())
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        let tok = match c {
            ' ' | '\t' | '\n' => continue,
            '0'..='9' => lex_number(c, &mut it),
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut s = c.to_string();
                while let Some(&n) = it.peek() {
                    if n.is_alphanumeric() || n == '_' { s.push(n); it.next(); } else { break; }
                }
                Tok::Ident(s)
            }
            '"' => {
                let mut s = String::new();
                loop {
                    let Some(n) = it.next() else { return Err("unterminated string".into()) };
                    match n {
                        '"' => break,
                        '\\' => match it.next() {
                            Some('n') => s.push('\n'),
                            Some(other) => s.push(other),
                            None => return Err("bad escape".into()),
                        },
                        _ => s.push(n),
                    }
                }
                Tok::Str(s)
            }
            '-' if it.peek() == Some(&'>') => { it.next(); Tok::Arrow }
            '=' => if let Some('=') = it.peek() { it.next(); Tok::EqEq } else { Tok::Eq },
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '+' | '-' | '*' | '/' => Tok::Op(c),
            other => return Err(format!("unexpected {:?}", other)),
        };
        out.push(tok);
    }
    Ok(out)
}

fn main() {
    let inputs = [
        "x = 3.5 + y_2 * (10 - 4)",
        "f -> \"hi\\nthere\" == 1_000",
        "a == b",
        "oops $",
        "\"never ends",
    ];
    for src in inputs.iter() {
        match tokenize(src) {
            Ok(toks) => {
                let idents = toks.iter().filter(|t| matches!(t, Tok::Ident(_))).count();
                let ops = toks.iter().filter(|t| matches!(t, Tok::Op('+' | '-') | Tok::Arrow)).count();
                println!("{} tokens, {} idents, {} additive: {:?}", toks.len(), idents, ops, toks);
            }
            Err(e) => println!("error in {:?}: {}", src, e),
        }
    }
    let toks = tokenize("n = 42").unwrap();
    let [Tok::Ident(name), Tok::Eq, Tok::Num(v)] = toks.as_slice() else {
        panic!("shape mismatch");
    };
    println!("{} := {}", name, v);
}
