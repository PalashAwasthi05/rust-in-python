// A hand-written tokenizer and recursive-descent parser built on Peekable.
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer { chars: src.chars().peekable() }
    }

    fn number(&mut self, first: char) -> Token {
        let mut text = String::new();
        text.push(first);
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                text.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Num(text.parse().unwrap())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        while let Some(c) = self.chars.next_if(|c| c.is_whitespace()) {
            let _ = c;
        }
        let c = self.chars.next()?;
        Some(match c {
            '0'..='9' => self.number(c),
            '(' => Token::LParen,
            ')' => Token::RParen,
            '+' | '-' | '*' | '/' | '^' => Token::Op(c),
            c if c.is_alphabetic() => {
                let mut name = c.to_string();
                while let Some(c) = self.chars.next_if(|c| c.is_alphanumeric() || *c == '_') {
                    name.push(c);
                }
                Token::Ident(name)
            }
            other => panic!("unexpected {}", other),
        })
    }
}

struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    fn expr(&mut self) -> Result<f64, String> {
        let mut value = self.term()?;
        while let Some(Token::Op(op)) = self.tokens.peek().cloned() {
            if op != '+' && op != '-' {
                break;
            }
            self.tokens.next();
            let rhs = self.term()?;
            value = if op == '+' { value + rhs } else { value - rhs };
        }
        Ok(value)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut value = self.power()?;
        loop {
            match self.tokens.peek() {
                Some(Token::Op('*')) => {
                    self.tokens.next();
                    value *= self.power()?;
                }
                Some(Token::Op('/')) => {
                    self.tokens.next();
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err("division by zero".to_string());
                    }
                    value /= d;
                }
                _ => return Ok(value),
            }
        }
    }

    fn power(&mut self) -> Result<f64, String> {
        let base = self.atom()?;
        if self.tokens.next_if_eq(&Token::Op('^')).is_some() {
            let exp = self.power()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<f64, String> {
        match self.tokens.next() {
            Some(Token::Num(n)) => Ok(n),
            Some(Token::Op('-')) => Ok(-self.atom()?),
            Some(Token::Ident(name)) => match name.as_str() {
                "pi" => Ok(3.14159),
                "two" => Ok(2.0),
                _ => Err(format!("unknown name {}", name)),
            },
            Some(Token::LParen) => {
                let v = self.expr()?;
                match self.tokens.next() {
                    Some(Token::RParen) => Ok(v),
                    other => Err(format!("expected ), got {:?}", other)),
                }
            }
            other => Err(format!("unexpected {:?}", other)),
        }
    }
}

fn eval(src: &str) -> Result<f64, String> {
    let mut parser = Parser { tokens: Lexer::new(src).peekable() };
    let v = parser.expr()?;
    if let Some(t) = parser.tokens.peek() {
        return Err(format!("trailing {:?}", t));
    }
    Ok(v)
}

fn group_runs(s: &str) -> Vec<(char, usize)> {
    let mut out = Vec::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        let mut n = 1;
        while it.peek() == Some(&c) {
            it.next();
            n += 1;
        }
        out.push((c, n));
    }
    out
}

fn main() {
    let toks: Vec<Token> = Lexer::new("3 + foo_1*(2.5 - x)").collect();
    println!("{:?}", toks);
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "2 ^ 3 ^ 2", "-4 + 10 / 4", "two * pi", "8 / (3 - 3)", "1 + bogus", "(1 + 2", "1 2"] {
        match eval(src) {
            Ok(v) => println!("{} = {}", src, v),
            Err(e) => println!("{} -> error: {}", src, e),
        }
    }
    println!("{:?}", group_runs("aaabccdddd"));

    let mut nums = vec![1, 2, 3, 10, 11, 20].into_iter().peekable();
    let mut groups = Vec::new();
    while let Some(first) = nums.next() {
        let mut group = vec![first];
        while let Some(n) = nums.next_if(|&n| n - group[group.len() - 1] == 1) {
            group.push(n);
        }
        groups.push(group);
    }
    println!("{:?}", groups);

    let mut p = [1, 2, 3].iter().peekable();
    if let Some(v) = p.peek_mut() {
        println!("peeked {}", v);
    }
    println!("peek {:?}", p.peek());
    println!("peek again {:?}", p.peek());
    let rest: Vec<&i32> = p.collect();
    println!("rest {:?}", rest);
}
