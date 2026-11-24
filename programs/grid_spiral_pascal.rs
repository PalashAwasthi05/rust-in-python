// Spiral traversal, spiral fill, Pascal's triangle, 2D prefix sums.
fn spiral(m: &Vec<Vec<i32>>) -> Vec<i32> {
    let mut out = Vec::new();
    if m.is_empty() {
        return out;
    }
    let (mut top, mut left) = (0i32, 0i32);
    let mut bottom = m.len() as i32 - 1;
    let mut right = m[0].len() as i32 - 1;
    while top <= bottom && left <= right {
        for c in left..=right {
            out.push(m[top as usize][c as usize]);
        }
        top += 1;
        for r in top..=bottom {
            out.push(m[r as usize][right as usize]);
        }
        right -= 1;
        if top <= bottom {
            for c in (left..=right).rev() {
                out.push(m[bottom as usize][c as usize]);
            }
            bottom -= 1;
        }
        if left <= right {
            for r in (top..=bottom).rev() {
                out.push(m[r as usize][left as usize]);
            }
            left += 1;
        }
    }
    out
}

fn spiral_fill<const N: usize>() -> [[u32; N]; N] {
    let mut g = [[0u32; N]; N];
    let dirs = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];
    let (mut r, mut c, mut d) = (0i32, 0i32, 0usize);
    for k in 1..=(N * N) as u32 {
        g[r as usize][c as usize] = k;
        let (nr, nc) = (r + dirs[d].0, c + dirs[d].1);
        if nr < 0 || nc < 0 || nr >= N as i32 || nc >= N as i32 || g[nr as usize][nc as usize] != 0 {
            d = (d + 1) % 4;
        }
        r += dirs[d].0;
        c += dirs[d].1;
    }
    g
}

fn pascal(n: usize) -> Vec<Vec<u64>> {
    let mut rows: Vec<Vec<u64>> = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = vec![1u64; i + 1];
        for j in 1..i {
            row[j] = rows[i - 1][j - 1] + rows[i - 1][j];
        }
        rows.push(row);
    }
    rows
}

struct Prefix {
    p: Vec<Vec<i64>>,
}

impl Prefix {
    fn new(g: &[Vec<i64>]) -> Self {
        let h = g.len();
        let w = g[0].len();
        let mut p = vec![vec![0i64; w + 1]; h + 1];
        for r in 0..h {
            for c in 0..w {
                p[r + 1][c + 1] = g[r][c] + p[r][c + 1] + p[r + 1][c] - p[r][c];
            }
        }
        Prefix { p }
    }
    fn sum(&self, r0: usize, c0: usize, r1: usize, c1: usize) -> i64 {
        self.p[r1][c1] - self.p[r0][c1] - self.p[r1][c0] + self.p[r0][c0]
    }
}

fn main() {
    let m: Vec<Vec<i32>> = (0..3).map(|r| (0..4).map(|c| r * 4 + c + 1).collect()).collect();
    println!("{:?}", m);
    println!("{:?}", spiral(&m));
    let tall = vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]];
    println!("{:?}", spiral(&tall));
    println!("{:?}", spiral(&vec![vec![42]]));

    let g = spiral_fill::<4>();
    for row in &g {
        println!("{}", row.iter().map(|v| format!("{:>3}", v)).collect::<String>());
    }
    let g5: [[u32; 5]; 5] = spiral_fill();
    println!("center {}", g5[2][2]);

    let tri = pascal(10);
    let width = tri.last().unwrap().iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ").len();
    for row in &tri {
        let line = row.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
        println!("{:^width$}", line, width = width);
    }
    println!("row sums {:?}", tri.iter().map(|r| r.iter().sum::<u64>()).collect::<Vec<_>>());
    println!("C(30,15) = {}", pascal(31)[30][15]);

    let data: Vec<Vec<i64>> = vec![
        vec![3, 0, 1, 4, 2],
        vec![5, 6, 3, 2, 1],
        vec![1, 2, 0, 1, 5],
        vec![4, 1, 0, 1, 7],
        vec![1, 0, 3, 0, 5],
    ];
    let pre = Prefix::new(&data);
    println!("{}", pre.sum(2, 1, 5, 4));
    println!("{}", pre.sum(1, 1, 3, 3));
    println!("{}", pre.sum(1, 2, 3, 5));
    println!("{}", pre.sum(0, 0, 5, 5));
    println!("{:?}", pre.p[5]);
}
