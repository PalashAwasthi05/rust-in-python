use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Matrix<const R: usize, const C: usize> {
    data: [[i64; C]; R],
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    fn zero() -> Self {
        Matrix { data: [[0; C]; R] }
    }
    fn transpose(&self) -> Matrix<C, R> {
        let mut out = Matrix::<C, R>::zero();
        for i in 0..R {
            for j in 0..C {
                out.data[j][i] = self.data[i][j];
            }
        }
        out
    }
    fn dims(&self) -> (usize, usize) {
        (R, C)
    }
}

impl<const N: usize> Matrix<N, N> {
    fn identity() -> Self {
        let mut m = Self::zero();
        for i in 0..N {
            m.data[i][i] = 1;
        }
        m
    }
    fn trace(&self) -> i64 {
        (0..N).map(|i| self.data[i][i]).sum()
    }
}

fn mul<const R: usize, const K: usize, const C: usize>(a: &Matrix<R, K>, b: &Matrix<K, C>) -> Matrix<R, C> {
    let mut out = Matrix::<R, C>::zero();
    for i in 0..R {
        for j in 0..C {
            for k in 0..K {
                out.data[i][j] += a.data[i][k] * b.data[k][j];
            }
        }
    }
    out
}

impl<const R: usize, const C: usize> fmt::Display for Matrix<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.data.iter() {
            let cells: Vec<String> = row.iter().map(|x| format!("{:>3}", x)).collect();
            writeln!(f, "[{}]", cells.join(""))?;
        }
        Ok(())
    }
}

fn sum_array<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()
}

struct Buffer<T, const N: usize> {
    items: [Option<T>; N],
    len: usize,
}

impl<T: Copy + fmt::Debug, const N: usize> Buffer<T, N> {
    fn new() -> Self {
        Buffer { items: [None; N], len: 0 }
    }
    fn push(&mut self, x: T) -> Result<(), T> {
        if self.len == N {
            return Err(x);
        }
        self.items[self.len] = Some(x);
        self.len += 1;
        Ok(())
    }
    fn capacity(&self) -> usize {
        N
    }
}

fn main() {
    let a = Matrix::<2, 3> { data: [[1, 2, 3], [4, 5, 6]] };
    let t = a.transpose();
    print!("{}", t);
    println!("{:?} -> {:?}", a.dims(), t.dims());
    let p = mul(&a, &t);
    print!("{}", p);
    println!("trace = {}", p.trace());
    let i3 = Matrix::<3, 3>::identity();
    println!("{}", mul(&t, &Matrix::<2, 2>::identity()) == t);
    println!("{}", i3.trace());
    println!("{} {}", sum_array([1, 2, 3]), sum_array([10; 5]));
    let mut b: Buffer<char, 3> = Buffer::new();
    for c in "abcd".chars() {
        match b.push(c) {
            Ok(()) => println!("pushed {}", c),
            Err(x) => println!("full, dropped {}", x),
        }
    }
    println!("{:?} cap {}", b.items, b.capacity());
}
