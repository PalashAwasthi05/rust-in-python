use std::fmt;

struct Matrix<const N: usize>([[i64; N]; N]);

impl<const N: usize> Matrix<N> {
    fn identity() -> Self {
        let mut m = [[0; N]; N];
        for i in 0..N {
            m[i][i] = 1;
        }
        Matrix(m)
    }
    fn mul(&self, other: &Matrix<N>) -> Matrix<N> {
        let mut out = [[0i64; N]; N];
        for i in 0..N {
            for j in 0..N {
                out[i][j] = (0..N).map(|k| self.0[i][k] * other.0[k][j]).sum();
            }
        }
        Matrix(out)
    }
    fn pow(&self, mut e: u32) -> Matrix<N> {
        let mut base = Matrix(self.0);
        let mut acc = Matrix::identity();
        while e > 0 {
            if e & 1 == 1 {
                acc = acc.mul(&base);
            }
            base = base.mul(&base);
            e >>= 1;
        }
        acc
    }
}

impl<const N: usize> fmt::Display for Matrix<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.0 {
            writeln!(f, "{:?}", row)?;
        }
        Ok(())
    }
}

fn rotate<const N: usize>(m: [[i32; N]; N]) -> [[i32; N]; N] {
    std::array::from_fn(|r| std::array::from_fn(|c| m[N - 1 - c][r]))
}

fn main() {
    let fib = Matrix([[1, 1], [1, 0]]);
    print!("{}", fib.pow(10));
    println!("{}", fib.pow(50).0[0][1]);
    let idx = std::array::from_fn::<usize, 5, _>(|i| i * 2);
    println!("{:?}", idx);
    let r = rotate([[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    println!("{:?}", r);
    let arr = [4, 8, 15, 16, 23, 42];
    println!("{}", arr.iter().sum::<i32>() / arr.len() as i32);
    let [r0, g0, b0]: [u8; 3] = [1, 2, 3];
    println!("{} {} {}", r0, g0, b0);
    let d = <[i32; 3]>::default();
    println!("{:?}", d);
    for (i, x) in arr.into_iter().enumerate().skip(4) {
        println!("{} {}", i, x);
    }
    match arr {
        [first, rest @ ..] => println!("{} {:?} {}", first, rest, rest.len()),
    }
    let mut v = vec![[0; 3]; 2];
    v[1][2] = 5;
    v.push([1, 1, 1]);
    v.sort();
    v.dedup();
    println!("{:?}", v);
    let mut grid = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
    grid[0].swap(0, 1);
    let (a, b) = (grid[0][1], grid[1][0]);
    grid[0][1] = b;
    grid[1][0] = a;
    let mut tmp = 100;
    std::mem::swap(&mut grid[2][1], &mut tmp);
    {
        let (top, rest) = grid.split_at_mut(1);
        std::mem::swap(&mut top[0], &mut rest[1]);
    }
    println!("{:?} {}", grid, tmp);
    let g2 = [[1, 9], [3, 4]];
    println!("{:?} {:?}", g2.concat(), g2.iter().flatten().max());
    let mut boxes: [Option<Box<i32>>; 3] = Default::default();
    boxes[1] = Some(Box::new(7));
    println!("{:?}", boxes);
    let mut m2 = [[0u32; 3]; 3];
    m2[1] = [1, 2, 3];
    let row = m2[1];
    m2[2] = row;
    m2[0][0] = m2[2][2] * 10;
    println!("{:?}", m2);
    let total: u32 = m2.iter().map(|r| r.iter().sum::<u32>()).sum();
    println!("{}", total);
    let col: Vec<u32> = m2.iter().map(|r| r[1]).collect();
    println!("{:?}", col);
    let strs = [["a", "b"], ["c", "d"]];
    let t: [[&str; 2]; 2] = std::array::from_fn(|i| std::array::from_fn(|j| strs[j][i]));
    println!("{:?}", t);
    println!("{}", [1.5f64, 2.5].iter().cloned().fold(0.0, f64::max));
    let words = ["a", "bb", "ccc"];
    let lens: [usize; 3] = words.map(str::len);
    println!("{:?}", lens);
    let mut sorted = [5, 1, 4];
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?} {}", sorted, sorted.iter().any(|&x| x == 4));
    let flags = [[false; 3]; 2];
    println!("{}", flags.iter().all(|r| r.iter().all(|&f| !f)));
    println!("{:?}", [3u8; 2].iter().map(|&b| b as u32 * 1000).collect::<Vec<_>>());
}
