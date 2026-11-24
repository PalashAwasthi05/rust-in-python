use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<i64>,
}

impl Matrix {
    fn new(rows: usize, cols: usize, data: Vec<i64>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Matrix { rows, cols, data }
    }
    fn identity(n: usize) -> Self {
        let mut data = vec![0; n * n];
        for i in 0..n { data[i * n + i] = 1; }
        Matrix { rows: n, cols: n, data }
    }
    fn get(&self, r: usize, c: usize) -> i64 { self.data[r * self.cols + c] }
    fn transpose(&self) -> Matrix {
        let mut data = Vec::with_capacity(self.data.len());
        for c in 0..self.cols {
            for r in 0..self.rows { data.push(self.get(r, c)); }
        }
        Matrix::new(self.cols, self.rows, data)
    }
}

impl Add for &Matrix {
    type Output = Matrix;
    fn add(self, other: &Matrix) -> Matrix {
        let data = self.data.iter().zip(&other.data).map(|(a, b)| a + b).collect();
        Matrix::new(self.rows, self.cols, data)
    }
}

impl Mul for &Matrix {
    type Output = Matrix;
    fn mul(self, other: &Matrix) -> Matrix {
        assert_eq!(self.cols, other.rows);
        let mut out = vec![0i64; self.rows * other.cols];
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut s = 0;
                for k in 0..self.cols { s += self.get(i, k) * other.get(k, j); }
                out[i * other.cols + j] = s;
            }
        }
        Matrix::new(self.rows, other.cols, out)
    }
}

impl Mul<i64> for Matrix {
    type Output = Matrix;
    fn mul(mut self, k: i64) -> Matrix {
        for x in self.data.iter_mut() { *x *= k; }
        self
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.rows {
            let row: Vec<String> = (0..self.cols).map(|c| format!("{:>4}", self.get(r, c))).collect();
            writeln!(f, "[{} ]", row.join(""))?;
        }
        Ok(())
    }
}

fn main() {
    let a = Matrix::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
    let b = Matrix::new(3, 2, vec![7, 8, 9, 10, 11, 12]);
    println!("A =\n{}", a);
    println!("A^T =\n{}", a.transpose());
    let c = &a * &b;
    println!("A*B =\n{}", c);
    println!("A*B + I =\n{}", &c + &Matrix::identity(2));
    println!("3*(A*B) =\n{}", c.clone() * 3);
    let mut fib = Matrix::new(2, 2, vec![1, 1, 1, 0]);
    let base = fib.clone();
    for _ in 0..9 { fib = &fib * &base; }
    println!("fib matrix^10 top-right = {}", fib.get(0, 1));
    println!("I*A == A? {}", &Matrix::identity(2) * &a == a);
}
