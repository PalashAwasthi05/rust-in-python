use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

/// Exact rational numbers so elimination results print identically everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rational {
    num: i64,
    den: i64,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

impl Rational {
    const ZERO: Rational = Rational { num: 0, den: 1 };
    const ONE: Rational = Rational { num: 1, den: 1 };

    fn new(num: i64, den: i64) -> Rational {
        assert!(den != 0, "zero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        Rational { num: sign * num / g, den: sign * den / g }
    }

    fn is_zero(&self) -> bool {
        self.num == 0
    }

    fn recip(self) -> Rational {
        Rational::new(self.den, self.num)
    }
}

impl From<i64> for Rational {
    fn from(n: i64) -> Self {
        Rational::new(n, 1)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = if self.den == 1 { format!("{}", self.num) } else { format!("{}/{}", self.num, self.den) };
        f.pad(&s)
    }
}

impl Add for Rational {
    type Output = Rational;
    fn add(self, o: Rational) -> Rational {
        Rational::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
}

impl Sub for Rational {
    type Output = Rational;
    fn sub(self, o: Rational) -> Rational {
        self + (-o)
    }
}

impl Neg for Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        Rational { num: -self.num, den: self.den }
    }
}

impl Mul for Rational {
    type Output = Rational;
    fn mul(self, o: Rational) -> Rational {
        Rational::new(self.num * o.num, self.den * o.den)
    }
}

impl AddAssign for Rational {
    fn add_assign(&mut self, o: Rational) {
        *self = *self + o;
    }
}

trait Scalar: Copy + PartialEq + fmt::Display + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self> {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Scalar for Rational {
    fn zero() -> Self { Rational::ZERO }
    fn one() -> Self { Rational::ONE }
}

impl Scalar for i64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
}

#[derive(Clone, PartialEq, Debug)]
struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: Scalar> Matrix<T> {
    fn zeros(rows: usize, cols: usize) -> Self {
        Matrix { rows, cols, data: vec![T::zero(); rows * cols] }
    }

    fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m[(i, i)] = T::one();
        }
        m
    }

    fn from_rows(rows: &[Vec<T>]) -> Self {
        let cols = rows.first().map_or(0, |r| r.len());
        assert!(rows.iter().all(|r| r.len() == cols), "ragged rows");
        Matrix { rows: rows.len(), cols, data: rows.iter().flat_map(|r| r.iter().cloned()).collect() }
    }

    fn transpose(&self) -> Self {
        let mut t = Self::zeros(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                t[(c, r)] = self[(r, c)];
            }
        }
        t
    }

    fn row(&self, r: usize) -> &[T] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for c in 0..self.cols {
            self.data.swap(a * self.cols + c, b * self.cols + c);
        }
    }

    fn trace(&self) -> T {
        (0..self.rows.min(self.cols)).map(|i| self[(i, i)]).fold(T::zero(), |a, b| a + b)
    }

    fn map<U: Scalar>(&self, f: impl Fn(T) -> U) -> Matrix<U> {
        Matrix { rows: self.rows, cols: self.cols, data: self.data.iter().map(|&x| f(x)).collect() }
    }

    fn pow(&self, mut e: u32) -> Self {
        let mut base = self.clone();
        let mut acc = Self::identity(self.rows);
        while e > 0 {
            if e & 1 == 1 {
                acc = &acc * &base;
            }
            base = &base * &base;
            e >>= 1;
        }
        acc
    }
}

impl Matrix<Rational> {
    /// Gauss-Jordan elimination; returns (determinant, inverse if any).
    fn invert(&self) -> (Rational, Option<Matrix<Rational>>) {
        assert_eq!(self.rows, self.cols);
        let n = self.rows;
        let mut a = self.clone();
        let mut inv = Matrix::identity(n);
        let mut det = Rational::ONE;
        for col in 0..n {
            let pivot = match (col..n).find(|&r| !a[(r, col)].is_zero()) {
                Some(p) => p,
                None => return (Rational::ZERO, None),
            };
            if pivot != col {
                a.swap_rows(pivot, col);
                inv.swap_rows(pivot, col);
                det = -det;
            }
            let p = a[(col, col)];
            det = det * p;
            let scale = p.recip();
            for c in 0..n {
                a[(col, c)] = a[(col, c)] * scale;
                inv[(col, c)] = inv[(col, c)] * scale;
            }
            for r in 0..n {
                if r == col {
                    continue;
                }
                let factor = a[(r, col)];
                if factor.is_zero() {
                    continue;
                }
                for c in 0..n {
                    let (ac, ic) = (a[(col, c)], inv[(col, c)]);
                    a[(r, c)] = a[(r, c)] - factor * ac;
                    inv[(r, c)] = inv[(r, c)] - factor * ic;
                }
            }
        }
        (det, Some(inv))
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, (r, c): (usize, usize)) -> &T {
        assert!(r < self.rows && c < self.cols, "index ({}, {}) out of bounds", r, c);
        &self.data[r * self.cols + c]
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        &mut self.data[r * self.cols + c]
    }
}

impl<'a, T: Scalar> Mul<&'a Matrix<T>> for &'a Matrix<T> {
    type Output = Matrix<T>;
    fn mul(self, rhs: &Matrix<T>) -> Matrix<T> {
        assert_eq!(self.cols, rhs.rows, "dimension mismatch");
        let mut out = Matrix::zeros(self.rows, rhs.cols);
        for i in 0..self.rows {
            for j in 0..rhs.cols {
                let mut acc = T::zero();
                for k in 0..self.cols {
                    acc = acc + self[(i, k)] * rhs[(k, j)];
                }
                out[(i, j)] = acc;
            }
        }
        out
    }
}

impl<T: Scalar> Add for Matrix<T> {
    type Output = Matrix<T>;
    fn add(mut self, rhs: Matrix<T>) -> Matrix<T> {
        assert_eq!((self.rows, self.cols), (rhs.rows, rhs.cols));
        for (a, b) in self.data.iter_mut().zip(rhs.data.into_iter()) {
            *a = *a + b;
        }
        self
    }
}

impl<T: Scalar> Mul<T> for Matrix<T> {
    type Output = Matrix<T>;
    fn mul(self, k: T) -> Matrix<T> {
        self.map(|x| x * k)
    }
}

impl<T: Scalar> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cells: Vec<String> = self.data.iter().map(|x| x.to_string()).collect();
        let width = cells.iter().map(|s| s.len()).max().unwrap_or(1);
        for r in 0..self.rows {
            let line: Vec<String> = (0..self.cols).map(|c| format!("{:>w$}", cells[r * self.cols + c], w = width)).collect();
            writeln!(f, "[ {} ]", line.join(" "))?;
        }
        Ok(())
    }
}

fn r(n: i64) -> Rational {
    Rational::from(n)
}

fn main() {
    let a = Matrix::from_rows(&[vec![r(2), r(1), r(1)], vec![r(1), r(3), r(2)], vec![r(1), r(0), r(0)]]);
    println!("A =\n{}", a);
    println!("A^T =\n{}", a.transpose());
    println!("trace(A) = {}", a.trace());
    let (det, inv) = a.invert();
    println!("det(A) = {}", det);
    if let Some(inv) = &inv {
        println!("A^-1 =\n{}", inv);
        let check = &a * inv;
        println!("A * A^-1 == I: {}", check == Matrix::identity(3));
    }

    let singular = Matrix::from_rows(&[vec![r(1), r(2)], vec![r(2), r(4)]]);
    let (d, maybe) = singular.invert();
    println!("singular det = {}, inverse = {:?}", d, maybe.map(|m| m.rows));

    let fib: Matrix<i64> = Matrix::from_rows(&[vec![1, 1], vec![1, 0]]);
    for n in [1u32, 5, 10, 30, 60] {
        let p = fib.pow(n);
        println!("fib({:>2}) = {}", n, p[(0, 1)]);
    }

    let b = Matrix::from_rows(&[vec![1i64, 2, 3], vec![4, 5, 6]]);
    let c = Matrix::from_rows(&[vec![7i64, 8], vec![9, 10], vec![11, 12]]);
    let bc = &b * &c;
    println!("B * C =\n{}", bc);
    println!("(B*C)*3 + I =\n{}", bc.clone() * 3 + Matrix::identity(2));
    println!("row 1 of B: {:?}", b.row(1));
    let halves = b.map(|x| Rational::new(x, 2));
    print!("B / 2 =\n{}", halves);
    let mut acc = Rational::ZERO;
    for x in halves.data.iter() {
        acc += *x;
    }
    println!("sum = {}, mixed = {:>6}|{:<6}|", acc, Rational::new(-6, 4), Rational::new(3, -9));

    let hilbert = Matrix { rows: 4, cols: 4, data: (0..16).map(|k| Rational::new(1, (k / 4 + k % 4 + 1) as i64)).collect() };
    let (hdet, hinv) = hilbert.invert();
    println!("det(H4) = {}", hdet);
    println!("H4^-1 =\n{}", hinv.unwrap());
    println!("{:?}", Rational::new(10, -4));
}
