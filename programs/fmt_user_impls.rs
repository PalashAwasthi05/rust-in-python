use std::fmt;

struct Plain(i32);
impl fmt::Display for Plain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P{}", self.0)
    }
}

struct Padded(&'static str);
impl fmt::Display for Padded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.0)
    }
}

struct Delegate(f64);
impl fmt::Display for Delegate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

struct Inspect;
impl fmt::Display for Inspect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "w={:?} p={:?} fill={:?} alt={} plus={} zero={} align={:?}",
               f.width(), f.precision(), f.fill(), f.alternate(), f.sign_plus(),
               f.sign_aware_zero_pad(), f.align())
    }
}

struct Money(i64);
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = format!("${}.{:02}", self.0 / 100, self.0 % 100);
        if let Some(w) = f.width() {
            write!(f, "{:>w$}", text, w = w)
        } else {
            write!(f, "{}", text)
        }
    }
}

struct Alt(u8);
impl fmt::Display for Alt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() { write!(f, "Alt[{}]", self.0) } else { write!(f, "{}", self.0) }
    }
}
impl fmt::Debug for Alt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Alt").field("value", &self.0).field("hex", &format_args!("{:#x}", self.0)).finish()
    }
}
impl fmt::LowerHex for Alt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.0;
        fmt::LowerHex::fmt(&v, f)
    }
}

struct Matrix(Vec<Vec<i32>>);
impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, row) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            for (j, v) in row.iter().enumerate() {
                if j > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{:>3}", v)?;
            }
        }
        Ok(())
    }
}

struct List(Vec<i32>);
impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

struct Named { id: u32, tags: Vec<String> }
impl fmt::Debug for Named {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Named").field("id", &self.id).field("tags", &self.tags).finish()
    }
}

fn main() {
    println!("[{}] [{:>8}] [{:<8}] [{:^8}]", Plain(1), Plain(2), Plain(3), Plain(4));
    println!("[{}] [{:>8}] [{:-<8}] [{:^8}] [{:.2}]", Padded("ab"), Padded("ab"), Padded("ab"), Padded("ab"), Padded("abc"));
    println!("[{}] [{:8.2}] [{:+}] [{:08.1}]", Delegate(1.5), Delegate(3.14159), Delegate(2.0), Delegate(-1.25));
    println!("{}", Inspect);
    println!("{:>10.3}", Inspect);
    println!("{:*^+#08}", Inspect);
    println!("{:<}", Inspect);
    println!("[{}] [{:10}]", Money(1234), Money(505));
    println!("{} {:#}", Alt(5), Alt(5));
    println!("{:?}", Alt(255));
    println!("{:#?}", Alt(255));
    println!("{:x} {:#06x}", Alt(255), Alt(10));
    println!("{}", Matrix(vec![vec![1, 20], vec![300, 4]]));
    println!("{:?}", List(vec![1, 2]));
    println!("{:#?}", List(vec![1, 2]));
    let n = Named { id: 9, tags: vec!["x".into()] };
    println!("{:?}", n);
    println!("{:#?}", n);
    let v = vec![Plain(1), Plain(2)];
    let joined: Vec<String> = v.iter().map(|p| p.to_string()).collect();
    println!("{}", joined.join(","));
    println!("{}", format!("{:>6}", Plain(7)).len());
    let s = Padded("pad").to_string();
    println!("{:?}", s);
}
