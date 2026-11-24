use std::fmt;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
enum Token { Num(f64), Op(char), Ident(String) }

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Num(n) => write!(f, "{}", n),
            Token::Op(c) => write!(f, "op({c})"),
            Token::Ident(name) => f.write_str(name),
        }
    }
}

struct Matrix { rows: Vec<Vec<i32>> }
impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.rows {
            let cells: Vec<String> = row.iter().map(|c| format!("{:>3}", c)).collect();
            writeln!(f, "[{}]", cells.join(","))?;
        }
        Ok(())
    }
}

struct Wrapper<T>(Vec<T>);
impl<T: fmt::Display> fmt::Display for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<")?;
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 { write!(f, " | ")?; }
            write!(f, "{}", item)?;
        }
        write!(f, ">")
    }
}

struct Celsius(f64);
impl fmt::Debug for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:.1}°C", self.0) }
}

#[derive(Debug)]
struct Reading { place: String, temp: Celsius }

fn main() {
    let tokens = vec![Token::Num(1.5), Token::Op('+'), Token::Ident("x".to_string())];
    for t in &tokens { print!("{} ", t); }
    println!();
    println!("{:?}", tokens[0]);
    let m = Matrix { rows: vec![vec![1, 20], vec![300, 4]] };
    print!("{}", m);
    println!("{}", Wrapper(vec![1, 2, 3]));
    println!("{}", Wrapper(tokens.clone()));
    let r = Reading { place: "lab".to_string(), temp: Celsius(21.456) };
    println!("{:?}", r);
    println!("{:#?}", r);
    let mut s = String::new();
    write!(s, "{}-{}", 1, 2).unwrap();
    writeln!(s, "!").unwrap();
    print!("{s}");
    let t = Token::Op('*').to_string();
    println!("{} {}", t, t.len());
}
