// Iterating arrays: by value, by ref, map, zip, enumerate on rows, flatten, collect into arrays.
use std::convert::TryInto;

fn main() {
    let a = [1, 2, 3, 4];
    let doubled = a.map(|x| x * 2);
    println!("{:?} {:?}", a, doubled);
    let strs = a.map(|x| format!("<{}>", x));
    println!("{:?}", strs);
    let pairs: Vec<(i32, i32)> = a.into_iter().zip(doubled).collect();
    println!("{:?}", pairs);
    let by_ref: Vec<&i32> = a.iter().filter(|x| **x % 2 == 0).collect();
    println!("{:?}", by_ref);
    for x in &a {
        print!("{} ", x * x);
    }
    println!();
    let total: i32 = a.into_iter().map(|x| x * 10).sum();
    println!("{}", total);

    let grid = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    for (r, row) in grid.iter().enumerate() {
        for (c, v) in row.iter().enumerate() {
            if r == c {
                print!("[{}]", v);
            } else {
                print!(" {} ", v);
            }
        }
        println!();
    }
    let flat: Vec<i32> = grid.iter().flatten().copied().collect();
    println!("{:?}", flat);
    let flat2: Vec<i32> = grid.into_iter().flatten().collect();
    println!("{}", flat == flat2);
    let diag: Vec<i32> = grid.iter().enumerate().map(|(i, r)| r[i]).collect();
    println!("{:?}", diag);
    let anti: Vec<i32> = grid.iter().enumerate().map(|(i, r)| r[2 - i]).collect();
    println!("{:?}", anti);
    let transposed: [[i32; 3]; 3] = std::array::from_fn(|c| grid.map(|row| row[c]));
    println!("{:?}", transposed);

    let v = vec![10, 20, 30];
    let arr: [i32; 3] = v.clone().try_into().unwrap();
    println!("{:?}", arr);
    let bad: Result<[i32; 2], Vec<i32>> = v.try_into();
    println!("{:?}", bad);
    let slice = &[5u8, 6, 7, 8][..];
    let head: [u8; 2] = slice[..2].try_into().unwrap();
    println!("{:?}", head);
    let wrong: Result<[u8; 3], _> = <[u8; 3]>::try_from(slice);
    println!("{}", wrong.is_err());
    println!("{}", u32::from_le_bytes([1, 0, 0, 0]));
    println!("{:?}", 258u16.to_be_bytes());

    let mut counts = [0usize; 5];
    for w in "the quick brown fox jumps over the lazy dog".split(' ') {
        counts[w.len().min(4)] += 1;
    }
    println!("{:?}", counts);
    let mut hist = [0u32; 26];
    for b in "hello world".bytes().filter(|b| (b'a'..=b'z').contains(b)) {
        hist[(b - b'a') as usize] += 1;
    }
    let top: Vec<(char, u32)> = hist
        .iter()
        .enumerate()
        .filter(|(_, &n)| n > 1)
        .map(|(i, &n)| ((b'a' + i as u8) as char, n))
        .collect();
    println!("{:?}", top);

    let it = [1, 2, 3].into_iter();
    println!("{:?}", it.clone().rev().collect::<Vec<_>>());
    println!("{}", it.count());
    let arr2 = [[0u8; 2]; 3];
    println!("{} {}", arr2.len(), arr2[0].len());
    let chunks: Vec<[i32; 2]> = [1, 2, 3, 4, 5, 6].chunks(2).map(|c| [c[0], c[1]]).collect();
    println!("{:?}", chunks);
    println!("{:?}", [1, 2, 3].iter().chain([4, 5].iter()).count());
    println!("{:?}", [[1, 2], [3, 4]].concat());
    println!("{:?}", [["a", "b"], ["c", "d"]].iter().map(|r| r.join("")).collect::<Vec<_>>());
}
