use std::fmt;

struct Celsius(f64);

impl fmt::Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(p) = f.precision() {
            write!(f, "{:.*}°C", p, self.0)
        } else {
            write!(f, "{}°C", self.0)
        }
    }
}

struct Name(&'static str);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.0)
    }
}

struct Matrix([[i32; 2]; 2]);

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

struct Pair(i32, String);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

struct Config {
    name: String,
    retries: u8,
    secret: String,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("name", &self.name)
            .field("retries", &self.retries)
            .field("secret", &"***")
            .finish()
    }
}

enum Token {
    Num(f64),
    Op(char),
    Ident(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Num(n) if f.alternate() => write!(f, "Num({})", n),
            Token::Num(n) => write!(f, "{}", n),
            Token::Op(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "${}", s),
        }
    }
}

fn render<T: fmt::Display>(items: &[T]) -> String {
    items.iter().map(|t| format!("[{:>6}]", t.to_string())).collect::<Vec<_>>().join("")
}

fn main() {
    let t = Celsius(21.456);
    println!("{} | {:.1} | {:.0}", t, t, t);
    println!("[{:>8}] [{:<8}] [{:^8}] [{:*^9}]", Name("bob"), Name("al"), Name("eve"), Name("x"));
    println!("[{:>8}]", Celsius(1.5));
    println!("{:?}", Matrix([[1, 2], [3, 4]]));
    println!("{:#?}", Matrix([[1, 2], [3, 4]]));
    println!("{:?}", Pair(7, "seven".into()));
    let c = Config { name: "svc".into(), retries: 3, secret: "hunter2".into() };
    println!("{:?}", c);
    println!("{:#?}", c);
    println!("{}", c.secret.len());
    let toks = vec![Token::Num(2.5), Token::Op('*'), Token::Ident("x".into())];
    for tk in &toks {
        print!("{} / {:#} ; ", tk, tk);
    }
    println!();
    println!("{}", render(&toks));
    let s = format!("{:>10}|{:<5}|", Name("right"), Name("l"));
    println!("{} {}", s, s.len());
}
