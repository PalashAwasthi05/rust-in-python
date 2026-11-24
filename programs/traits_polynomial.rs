use std::fmt;
use std::ops::{Add, Mul, MulAssign, Neg, Sub};

#[derive(Debug, Clone, PartialEq)]
struct Poly {
    c: Vec<i64>,
}

impl Poly {
    fn new(mut c: Vec<i64>) -> Self {
        while c.len() > 1 && *c.last().unwrap() == 0 {
            c.pop();
        }
        Poly { c }
    }
    fn degree(&self) -> usize {
        self.c.len() - 1
    }
    fn eval(&self, x: i64) -> i64 {
        self.c.iter().rev().fold(0, |acc, &k| acc * x + k)
    }
    fn derive(&self) -> Poly {
        if self.c.len() == 1 {
            return Poly::new(vec![0]);
        }
        Poly::new(self.c.iter().enumerate().skip(1).map(|(i, &k)| k * i as i64).collect())
    }
}

impl fmt::Display for Poly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for (i, &k) in self.c.iter().enumerate().rev() {
            if k == 0 && self.c.len() > 1 {
                continue;
            }
            let sign = if k < 0 { "-" } else { "+" };
            if first {
                if k < 0 {
                    write!(f, "-")?;
                }
            } else {
                write!(f, " {} ", sign)?;
            }
            first = false;
            let a = k.abs();
            match i {
                0 => write!(f, "{}", a)?,
                1 if a == 1 => write!(f, "x")?,
                1 => write!(f, "{}x", a)?,
                _ if a == 1 => write!(f, "x^{}", i)?,
                _ => write!(f, "{}x^{}", a, i)?,
            }
        }
        Ok(())
    }
}

impl<'a> Add<&'a Poly> for &'a Poly {
    type Output = Poly;
    fn add(self, o: &Poly) -> Poly {
        let n = self.c.len().max(o.c.len());
        Poly::new((0..n).map(|i| self.c.get(i).unwrap_or(&0) + o.c.get(i).unwrap_or(&0)).collect())
    }
}

impl Add for Poly {
    type Output = Poly;
    fn add(self, o: Poly) -> Poly {
        &self + &o
    }
}

impl Neg for Poly {
    type Output = Poly;
    fn neg(self) -> Poly {
        Poly::new(self.c.into_iter().map(|k| -k).collect())
    }
}

impl Sub for Poly {
    type Output = Poly;
    fn sub(self, o: Poly) -> Poly {
        self + (-o)
    }
}

impl Mul for &Poly {
    type Output = Poly;
    fn mul(self, o: &Poly) -> Poly {
        let mut out = vec![0; self.c.len() + o.c.len() - 1];
        for (i, a) in self.c.iter().enumerate() {
            for (j, b) in o.c.iter().enumerate() {
                out[i + j] += a * b;
            }
        }
        Poly::new(out)
    }
}

impl MulAssign<i64> for Poly {
    fn mul_assign(&mut self, k: i64) {
        for x in self.c.iter_mut() {
            *x *= k;
        }
        *self = Poly::new(std::mem::take(&mut self.c));
    }
}

fn main() {
    let p = Poly::new(vec![1, -3, 0, 2]);
    let q = Poly::new(vec![-1, 1]);
    println!("p = {}", p);
    println!("q = {}", q);
    println!("p + q = {}", &p + &q);
    println!("p - q = {}", p.clone() - q.clone());
    println!("p * q = {}", &p * &q);
    println!("p' = {}", p.derive());
    println!("deg {} {}", p.degree(), (&p * &q).degree());
    let vals: Vec<i64> = (-2..=2).map(|x| p.eval(x)).collect();
    println!("{:?}", vals);
    let mut r = q.clone();
    r *= 3;
    println!("3q = {}", r);
    r *= 0;
    println!("0 = {} {:?}", r, r);
    let sq = &q * &q;
    let cube = &sq * &q;
    println!("(x-1)^3 = {}", cube);
    println!("{}", cube == Poly::new(vec![-1, 3, -3, 1]));
}
