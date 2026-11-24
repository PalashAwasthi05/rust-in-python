// Dense matrix struct with operator traits: Add, Sub, Mul (matrix and scalar), Neg, Index, AddAssign.
use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn zeros(rows: usize, cols: usize) -> Self {
        Matrix { rows, cols, data: vec![0.0; rows * cols] }
    }

    fn identity(n: usize) -> Self {
        let mut m = Matrix::zeros(n, n);
        for i in 0..n {
            m[(i, i)] = 1.0;
        }
        m
    }

    fn from_rows(rows: &[&[f64]]) -> Self {
        let r = rows.len();
        let c = rows[0].len();
        assert!(rows.iter().all(|row| row.len() == c), "ragged rows");
        let data = rows.iter().flat_map(|row| row.iter().cloned()).collect();
        Matrix { rows: r, cols: c, data }
    }

    fn transpose(&self) -> Matrix {
        let mut t = Matrix::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                t[(j, i)] = self[(i, j)];
            }
        }
        t
    }

    fn pow(&self, mut e: u32) -> Matrix {
        assert_eq!(self.rows, self.cols);
        let mut base = self.clone();
        let mut acc = Matrix::identity(self.rows);
        while e > 0 {
            if e & 1 == 1 {
                acc = &acc * &base;
            }
            base = &base * &base;
            e >>= 1;
        }
        acc
    }

    fn determinant(&self) -> f64 {
        assert_eq!(self.rows, self.cols, "determinant of non-square matrix");
        let n = self.rows;
        let mut m = self.clone();
        let mut det = 1.0;
        for col in 0..n {
            let pivot = (col..n).max_by(|&a, &b| m[(a, col)].abs().partial_cmp(&m[(b, col)].abs()).unwrap()).unwrap();
            if m[(pivot, col)].abs() < 1e-12 {
                return 0.0;
            }
            if pivot != col {
                for k in 0..n {
                    let tmp = m[(col, k)];
                    m[(col, k)] = m[(pivot, k)];
                    m[(pivot, k)] = tmp;
                }
                det = -det;
            }
            det *= m[(col, col)];
            for r in col + 1..n {
                let factor = m[(r, col)] / m[(col, col)];
                for k in col..n {
                    let v = m[(col, k)];
                    m[(r, k)] -= factor * v;
                }
            }
        }
        det
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        &self.data[r * self.cols + c]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        &mut self.data[r * self.cols + c]
    }
}

impl<'a> Add<&'a Matrix> for &'a Matrix {
    type Output = Matrix;
    fn add(self, other: &Matrix) -> Matrix {
        assert_eq!((self.rows, self.cols), (other.rows, other.cols), "shape mismatch in add");
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().zip(&other.data).map(|(a, b)| a + b).collect(),
        }
    }
}

impl Add for Matrix {
    type Output = Matrix;
    fn add(self, other: Matrix) -> Matrix {
        &self + &other
    }
}

impl Sub for &Matrix {
    type Output = Matrix;
    fn sub(self, other: &Matrix) -> Matrix {
        self + &(-other)
    }
}

impl Neg for &Matrix {
    type Output = Matrix;
    fn neg(self) -> Matrix {
        self * -1.0
    }
}

impl Mul<f64> for &Matrix {
    type Output = Matrix;
    fn mul(self, k: f64) -> Matrix {
        Matrix { rows: self.rows, cols: self.cols, data: self.data.iter().map(|x| x * k).collect() }
    }
}

impl Mul<&Matrix> for f64 {
    type Output = Matrix;
    fn mul(self, m: &Matrix) -> Matrix {
        m * self
    }
}

impl Mul for &Matrix {
    type Output = Matrix;
    fn mul(self, other: &Matrix) -> Matrix {
        assert_eq!(self.cols, other.rows, "shape mismatch in mul");
        let mut out = Matrix::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut s = 0.0;
                for k in 0..self.cols {
                    s += self[(i, k)] * other[(k, j)];
                }
                out[(i, j)] = s;
            }
        }
        out
    }
}

impl AddAssign<&Matrix> for Matrix {
    fn add_assign(&mut self, other: &Matrix) {
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a += *b;
        }
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prec = f.precision().unwrap_or(1);
        for r in 0..self.rows {
            let row: Vec<String> = (0..self.cols).map(|c| format!("{:>7.*}", prec, self[(r, c)])).collect();
            writeln!(f, "|{} |", row.join(""))?;
        }
        Ok(())
    }
}

fn main() {
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = Matrix::from_rows(&[&[0.5, -1.0], &[2.0, 0.0]]);
    println!("a =\n{}", a);
    println!("a + b =\n{}", &a + &b);
    println!("a - b =\n{}", &a - &b);
    println!("a * b =\n{}", &a * &b);
    println!("2.5 * a =\n{:.2}", 2.5 * &a);
    println!("-a =\n{}", -&a);
    println!("a^T =\n{}", a.transpose());
    println!("det(a) = {:.3}", a.determinant());

    let mut acc = Matrix::zeros(2, 2);
    for _ in 0..3 {
        acc += &a;
    }
    println!("acc =\n{}", acc);
    println!("acc == 3a? {}", acc == &a * 3.0);
    let sum = a.clone() + b.clone();
    println!("owned add [1][0] = {}", sum[(1, 0)]);

    let fib = Matrix::from_rows(&[&[1.0, 1.0], &[1.0, 0.0]]);
    let f30 = fib.pow(30);
    println!("fib(30) via matrix power = {}", f30[(0, 1)]);

    let rect = Matrix::from_rows(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
    let gram = &rect * &rect.transpose();
    println!("rect * rect^T =\n{:.0}", gram);
    println!("shape {}x{}", gram.rows, gram.cols);

    let m3 = Matrix::from_rows(&[&[2.0, -3.0, 1.0], &[2.0, 0.0, -1.0], &[1.0, 4.0, 5.0]]);
    println!("det(m3) = {:.4}", m3.determinant());
    let singular = Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 4.0]]);
    println!("det(singular) = {}", singular.determinant());
    println!("I3 =\n{:.0}", Matrix::identity(3));
    println!("{:?}", Matrix::identity(2));

    let _bad = &a * &rect.transpose();
}
