#[derive(Debug, Clone, PartialEq)]
enum Token { Num(f64), Plus, Minus, Star, Slash, LParen, RParen }

fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' => i += 1,
            '+' => { out.push(Token::Plus); i += 1; }
            '-' => { out.push(Token::Minus); i += 1; }
            '*' => { out.push(Token::Star); i += 1; }
            '/' => { out.push(Token::Slash); i += 1; }
            '(' => { out.push(Token::LParen); i += 1; }
            ')' => { out.push(Token::RParen); i += 1; }
            d if d.is_ascii_digit() || d == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
                let s: String = chars[start..i].iter().collect();
                out.push(Token::Num(s.parse().map_err(|_| format!("bad number {}", s))?));
            }
            other => return Err(format!("unexpected char '{}' at {}", other, i)),
        }
    }
    Ok(out)
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn next(&mut self) -> Option<Token> { let t = self.tokens.get(self.pos).cloned(); self.pos += 1; t }
    fn expr(&mut self) -> Result<f64, String> {
        let mut v = self.term()?;
        while let Some(t) = self.peek() {
            match t {
                Token::Plus => { self.next(); v += self.term()?; }
                Token::Minus => { self.next(); v -= self.term()?; }
                _ => break,
            }
        }
        Ok(v)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut v = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => { self.next(); v *= self.factor()?; }
                Some(Token::Slash) => {
                    self.next();
                    let d = self.factor()?;
                    if d == 0.0 { return Err("division by zero".into()); }
                    v /= d;
                }
                _ => return Ok(v),
            }
        }
    }
    fn factor(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Num(n)) => Ok(n),
            Some(Token::Minus) => Ok(-self.factor()?),
            Some(Token::LParen) => {
                let v = self.expr()?;
                match self.next() { Some(Token::RParen) => Ok(v), _ => Err("expected ')'".into()) }
            }
            t => Err(format!("unexpected token {:?}", t)),
        }
    }
}

fn eval(src: &str) -> Result<f64, String> {
    let mut p = Parser { tokens: tokenize(src)?, pos: 0 };
    let v = p.expr()?;
    if p.pos < p.tokens.len() { return Err("trailing input".into()); }
    Ok(v)
}

fn main() {
    let inputs = ["1 + 2 * 3", "(1 + 2) * 3", "10 / 4 - 1", "-(3 - 5) * 2.5", "2 * (3 + 4) * (5 - 1)", "1 / 0", "4 + ", "7 $ 2", "(1 + 2"];
    for src in inputs.iter() {
        match eval(src) {
            Ok(v) => println!("{:<22} = {}", src, v),
            Err(e) => println!("{:<22} ! {}", src, e),
        }
    }
}
