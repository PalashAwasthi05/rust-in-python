// Building strings with write!/writeln! and Display impls.
use std::fmt::{self, Write};

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn identity(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Matrix { rows: n, cols: n, data }
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prec = f.precision().unwrap_or(1);
        for r in 0..self.rows {
            write!(f, "[")?;
            for c in 0..self.cols {
                if c > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{:.*}", prec, self.data[r * self.cols + c])?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

struct Money(i64);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let s = format!("{}${}.{:02}", sign, self.0.abs() / 100, self.0.abs() % 100);
        f.pad(&s)
    }
}

struct Hex(u32);

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "Hex({:#x})", self.0)
        } else {
            write!(f, "{:x}", self.0)
        }
    }
}

fn table(rows: &[(&str, i64)]) -> Result<String, fmt::Error> {
    let mut out = String::new();
    writeln!(out, "{:<10}{:>12}", "name", "balance")?;
    writeln!(out, "{}", "=".repeat(22))?;
    for (name, cents) in rows {
        writeln!(out, "{:<10}{:>12}", name, Money(*cents))?;
    }
    write!(out, "{:<10}{:>12}", "total", Money(rows.iter().map(|r| r.1).sum()))?;
    Ok(out)
}

fn main() {
    let mut s = String::new();
    write!(s, "{}-{}", 1, 2).unwrap();
    write!(&mut s, "|{:>4}|", "x").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "line {}", 2).unwrap();
    s.write_str("raw").unwrap();
    s.write_char('!').unwrap();
    print!("{}", s);
    println!();
    print!("{}", Matrix::identity(2));
    print!("{:.3}", Matrix { rows: 1, cols: 3, data: vec![1.0, 2.5, -0.125] });
    println!("{}", table(&[("alice", 12345), ("bob", -250), ("carol", 7)]).unwrap());
    println!("[{:>10}] [{:<10}] [{:^10}]", Money(199), Money(-5), Money(100000));
    println!("{:?} {:#?}", Hex(255), Hex(255));
    let parts: Vec<String> = (1..=3).map(|i| format!("{:02}", i * 7)).collect();
    println!("{}", parts.join(":"));
    let mut csv = String::new();
    for (i, row) in [[1, 2, 3], [4, 5, 6]].iter().enumerate() {
        if i > 0 {
            csv.push('\n');
        }
        let line = row.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
        csv += &line;
    }
    println!("{}", csv);
    let joined = ["a", "b"].concat() + &String::from("c") + "d";
    println!("{} {}", joined, format!("{joined:?}"));
    let width = 6;
    let label = format!("{:width$}|{:>width$.2}|", "ab", 1.0 / 3.0);
    println!("{}", label);
    let report = format_args!("{} {}", "args", 1).to_string();
    println!("{}", report);
    eprintln!("this goes to stderr");
    let mut buf = String::with_capacity(16);
    for c in ['x', 'y'] {
        write!(buf, "{c}{c}").unwrap();
    }
    println!("{}", buf);
}
