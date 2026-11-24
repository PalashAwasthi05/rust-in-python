use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Mat<const N: usize>([[f64; N]; N]);

impl<const N: usize> Mat<N> {
    fn identity() -> Self {
        let mut m = [[0.0; N]; N];
        for i in 0..N { m[i][i] = 1.0; }
        Mat(m)
    }

    fn transpose(&self) -> Self {
        let mut t = [[0.0; N]; N];
        for i in 0..N { for j in 0..N { t[j][i] = self.0[i][j]; } }
        Mat(t)
    }

    fn mul(&self, o: &Self) -> Self {
        let mut r = [[0.0; N]; N];
        for i in 0..N { for j in 0..N { r[i][j] = (0..N).map(|k| self.0[i][k] * o.0[k][j]).sum(); } }
        Mat(r)
    }

    fn determinant(&self) -> f64 {
        let mut a = self.0;
        let mut det = 1.0;
        for col in 0..N {
            let pivot = (col..N).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap()).unwrap();
            if a[pivot][col].abs() < 1e-12 { return 0.0; }
            if pivot != col { a.swap(pivot, col); det = -det; }
            det *= a[col][col];
            for row in col + 1..N {
                let f = a[row][col] / a[col][col];
                for k in col..N { a[row][k] -= f * a[col][k]; }
            }
        }
        det
    }

    fn trace(&self) -> f64 { (0..N).map(|i| self.0[i][i]).sum() }
}

impl<const N: usize> fmt::Display for Mat<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.0 {
            let cells: Vec<String> = row.iter().map(|x| format!("{:>7.2}", x)).collect();
            writeln!(f, "|{} |", cells.join(""))?;
        }
        Ok(())
    }
}

fn sum_array<const N: usize>(xs: [i32; N]) -> i32 { xs.iter().sum() }

fn main() {
    let a = Mat([[2.0, -1.0, 0.0], [1.0, 3.0, 2.0], [0.0, 1.0, 4.0]]);
    print!("A =\n{}", a);
    print!("A^T =\n{}", a.transpose());
    println!("det(A) = {:.3}, trace = {}", a.determinant(), a.trace());
    print!("A * A^T =\n{}", a.mul(&a.transpose()));
    println!("A * I == A: {}", a.mul(&Mat::identity()) == a);

    let b = Mat([[1.0, 2.0], [3.0, 4.0]]);
    println!("det 2x2 = {}", b.determinant());
    let singular = Mat([[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [1.0, 0.0, 1.0]]);
    println!("det singular = {}", singular.determinant());
    let mut m4 = Mat::<4>::identity();
    m4.0[0][3] = 5.0;
    m4.0[2][2] = 3.0;
    m4.0[3][0] = 1.0;
    println!("det 4x4 = {:.1}", m4.determinant());
    println!("det(AB) = {:.3} vs det(A)det(B) = {:.3}", a.mul(&a.transpose()).determinant(), a.determinant() * a.transpose().determinant());
    println!("sum_array 3 = {}, 5 = {}", sum_array([1, 2, 3]), sum_array([10; 5]));
    println!("size of Mat<3> = {} bytes", std::mem::size_of::<Mat<3>>());
}
