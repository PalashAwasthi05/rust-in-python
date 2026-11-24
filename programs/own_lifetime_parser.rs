// A tokenizer that borrows its input and hands out slices tied to it.
#[derive(Debug, PartialEq)]
enum Token<'a> {
    Ident(&'a str),
    Number(i64),
    Punct(char),
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer { input, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.rest().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        self.skip_ws();
        let rest = self.rest();
        let first = rest.chars().next()?;
        if first.is_alphabetic() || first == '_' {
            let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(rest.len());
            self.pos += end;
            Some(Token::Ident(&rest[..end]))
        } else if first.is_ascii_digit() {
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            self.pos += end;
            Some(Token::Number(rest[..end].parse().unwrap()))
        } else {
            self.pos += first.len_utf8();
            Some(Token::Punct(first))
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Token<'a>> {
        self.next_token()
    }
}

fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn first_ident<'a>(tokens: &[Token<'a>]) -> Option<&'a str> {
    for t in tokens {
        if let Token::Ident(name) = t {
            return Some(name);
        }
    }
    None
}

struct Highlighter<'src, 'buf> {
    source: &'src str,
    out: &'buf mut String,
}

impl<'src, 'buf> Highlighter<'src, 'buf> {
    fn emit(&mut self, tok: &Token<'src>) {
        match tok {
            Token::Ident(s) => self.out.push_str(&format!("<{}>", s)),
            Token::Number(n) => self.out.push_str(&format!("#{}", n)),
            Token::Punct(c) => self.out.push(*c),
        }
    }
}

fn main() {
    let source = String::from("let total_sum = alpha + 42 * beta_2;");
    let tokens: Vec<Token> = {
        let mut lexer = Lexer::new(&source);
        let mut out = Vec::new();
        while let Some(tok) = lexer.next_token() {
            out.push(tok);
        }
        out
    };
    for t in &tokens {
        println!("{:?}", t);
    }
    let idents: Vec<&str> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(*s),
            _ => None,
        })
        .collect();
    let mut best = "";
    for id in &idents {
        best = longest(best, id);
    }
    println!("longest ident: {}", best);
    println!("first ident: {:?}", first_ident(&tokens));
    println!("eq: {}", tokens[1] == Token::Ident("total_sum"));
    let sum: i64 = tokens.iter().map(|t| if let Token::Number(n) = t { *n } else { 0 }).sum();
    println!("numbers sum: {}", sum);

    let mut buf = String::new();
    {
        let mut h = Highlighter { source: &source, out: &mut buf };
        for tok in Lexer::new(h.source) {
            h.emit(&tok);
        }
    }
    println!("{}", buf);
    let count = Lexer::new("a b c 1 2").filter(|t| matches!(t, Token::Ident(_))).count();
    println!("idents: {}", count);
}
