use std::convert::TryInto;
use std::fmt::Display;

fn show_all<T: Display, const N: usize>(items: [T; N]) -> String {
    items.iter().map(|x| format!("<{}>", x)).collect::<Vec<_>>().concat()
}

fn double(x: i32) -> i32 { x * 2 }
fn square(x: i32) -> i32 { x * x }
fn negate(x: i32) -> i32 { -x }

fn main() {
    let bytes: Vec<u8> = vec![0, 0, 1, 2, 255, 255, 255, 255, 9, 8, 7];
    let a = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let b = i32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
    println!("{} {}", a, b);
    let triples: Vec<[u8; 3]> = bytes.chunks_exact(3).map(|c| c.try_into().unwrap()).collect();
    println!("{:?}", triples);
    let t = <[u8; 3]>::try_from(&bytes[8..11]).unwrap();
    println!("{:?}", t);
    println!("{}", show_all([1.5, 2.0]));
    println!("{}", show_all(["x", "y", "z"]));
    let ops: [fn(i32) -> i32; 3] = [double, square, negate];
    println!("{:?}", ops.iter().map(|f| f(7)).collect::<Vec<_>>());
    let composed = ops.iter().fold(3, |acc, f| f(acc));
    println!("{}", composed);
    let fs = [1.5f64, -2.0, 9.25];
    println!("{:?}", fs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()));
    let grid = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let lower: &[[i32; 3]] = &grid[1..];
    println!("{:?} {}", lower, lower.len());
    for w in [1, 3, 6, 10].windows(2) {
        if let [x, y] = w {
            print!("{} ", y - x);
        }
    }
    println!();
    let pairs: Vec<(i32, i32)> = [1, 2, 3, 4].chunks(2).map(|c| (c[0], c[1])).collect();
    println!("{:?}", pairs);
    let mut m = [[0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = grid[j][i] - grid[i][j];
        }
    }
    println!("{:?}", m);
    let is_antisym = (0..3).all(|i| (0..3).all(|j| m[i][j] == -m[j][i]));
    println!("{}", is_antisym);
    let mut cells = [0u8; 9];
    cells[4] = 1;
    let as2d: Vec<&[u8]> = cells.chunks(3).collect();
    println!("{:?}", as2d);
    let s = format!("{:?}", [[1u8; 2]; 2]);
    println!("{} {}", s, s.len());
    println!("{:>12}|{:<8?}|", format!("{:?}", [1, 2]), [3]);
    let total: usize = [[1usize; 4]; 4].iter().flatten().sum();
    println!("{}", total);
    let mut rows = [[3, 1], [2, 2], [1, 3]];
    rows.sort_by_key(|r| r[1]);
    println!("{:?}", rows);
    rows.reverse();
    println!("{:?} {:?}", rows.first(), rows.last().map(|r| r[0]));
    let found = rows.iter().find(|r| r.iter().sum::<i32>() == 4 && r[0] == 2);
    println!("{:?}", found);
    let idx = rows.iter().position(|&[a, _]| a == 3);
    println!("{:?}", idx);
    let readings = vec![1.25f64, -3.5, 10.0];
    println!("{:.1?}", readings);
    println!("{:6.2?}", [[0.5f32, 1.0], [2.25, -1.125]]);
    println!("{:>4?}", ["ab", "c"]);
    println!("{:03?}", [7u8, 42]);
    println!("{:+?}", &[1, -2][..]);
    println!("{:^5?}", ['x', 'y']);
}
