// Fixed-size arrays: init, indexing, equality, ordering, Debug, len, contains.
fn main() {
    let zeros = [0; 5];
    let mut a = [3, 1, 4, 1, 5];
    let b: [i64; 3] = [10, 20, 30];
    println!("{:?} {:?} {:?}", zeros, a, b);
    println!("len={} first={} last={}", a.len(), a[0], a[a.len() - 1]);
    a[2] = 9;
    println!("{:?}", a);
    println!("contains 9: {}, contains 7: {}", a.contains(&9), a.contains(&7));
    println!("position of 1: {:?}", a.iter().position(|&x| x == 1));
    println!("rposition of 1: {:?}", a.iter().rposition(|&x| x == 1));
    println!("position of 8: {:?}", a.iter().position(|&x| x == 8));
    let c = [1, 2, 3];
    let d = [1, 2, 4];
    println!("eq {} ne {} lt {} cmp {:?}", c == [1, 2, 3], c != d, c < d, d.cmp(&c));
    println!("max {:?} min {:?}", [c, d].iter().max(), [c, d].iter().min());
    let sum: i32 = a.iter().sum();
    let prod: i64 = b.iter().product();
    println!("sum={} prod={}", sum, prod);
    println!("{:#?}", [1u8, 2]);
    let words = ["apple", "banana", "cherry"];
    for (i, w) in words.iter().enumerate() {
        println!("{}: {} ({})", i, w, w.len());
    }
    let empty: [i32; 0] = [];
    println!("is_empty {} {}", words.is_empty(), empty.is_empty());
    let copy = a;
    a[0] = 100;
    println!("copy {:?} orig {:?}", copy, a);
    println!("first {:?} last {:?}", a.first(), a.last());
    let bools = [true; 4];
    println!("{:?} all={}", bools, bools.iter().all(|&b| b));
    let chars = ['x'; 3];
    let s: String = chars.iter().collect();
    println!("{}", s);
    let floats = [1.5f64, 2.25, -0.5];
    println!("{:?} {:.2}", floats, floats.iter().sum::<f64>());
    println!("{:?}", a.iter().rev().collect::<Vec<_>>());
    println!("{}", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
}
