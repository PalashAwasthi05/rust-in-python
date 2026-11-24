// Matrix multiply, transpose, rotate with const-sized 2D arrays.
const N: usize = 3;

type Mat = [[i32; N]; N];

fn mul(a: &Mat, b: &Mat) -> Mat {
    let mut out = [[0; N]; N];
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                out[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    out
}

fn transpose(m: Mat) -> Mat {
    let mut t = m;
    for i in 0..N {
        for j in 0..N {
            t[j][i] = m[i][j];
        }
    }
    t
}

fn transpose_in_place(m: &mut Mat) {
    for i in 0..N {
        for j in (i + 1)..N {
            let tmp = m[i][j];
            m[i][j] = m[j][i];
            m[j][i] = tmp;
        }
    }
}

fn rotate_cw(m: &Mat) -> Mat {
    let mut r = [[0; N]; N];
    for i in 0..N {
        for j in 0..N {
            r[j][N - 1 - i] = m[i][j];
        }
    }
    r
}

fn print(name: &str, m: &Mat) {
    println!("{}:", name);
    for row in m.iter() {
        let cells: Vec<String> = row.iter().map(|v| format!("{:4}", v)).collect();
        println!("{}", cells.join(""));
    }
}

fn identity() -> Mat {
    let mut m = [[0; N]; N];
    for i in 0..N {
        m[i][i] = 1;
    }
    m
}

fn main() {
    let a: Mat = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let b: Mat = [[9, 8, 7], [6, 5, 4], [3, 2, 1]];
    print("a", &a);
    print("a*b", &mul(&a, &b));
    print("a*I", &mul(&a, &identity()));
    println!("a*I == a: {}", mul(&a, &identity()) == a);
    print("a^T", &transpose(a));
    let mut c = a;
    transpose_in_place(&mut c);
    println!("in place matches: {}", c == transpose(a));
    print("rot", &rotate_cw(&a));
    let mut r = a;
    for _ in 0..4 {
        r = rotate_cw(&r);
    }
    println!("four rotations identity: {}", r == a);
    let trace: i32 = (0..N).map(|i| a[i][i]).sum();
    println!("trace = {}", trace);
    let mut p = identity();
    for _ in 0..3 {
        p = mul(&p, &a);
    }
    print("a^3", &p);
    println!("{:?}", a);
}
