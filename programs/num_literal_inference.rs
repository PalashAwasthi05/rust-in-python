struct Counter { hits: u16 }

fn takes_i8(x: i8) -> i8 { x }

fn sum_bytes(bytes: &[u8]) -> u8 {
    let mut total = 0u8;
    for &b in bytes {
        total = total.wrapping_add(b);
    }
    total
}

fn main() {
    let mut x = 0;
    let arr: [u8; 3] = [100, 100, 100];
    for v in arr.iter() {
        x += v / 2;
    }
    println!("{}", x);
    let mut acc = 1;
    let factor: u64 = 3;
    for _ in 0..30 { acc *= factor; }
    println!("{}", acc);
    let n = 200;
    let m = takes_i8(n as i8);
    println!("{}", m);
    let mut c = Counter { hits: 0 };
    let step = 1000;
    for _ in 0..60 { c.hits += step; }
    println!("{}", c.hits);
    println!("{}", sum_bytes(&[200, 100, 7]));
    let values = vec![1.5, 2.5];
    let doubled: Vec<f32> = values.iter().map(|v| v * 2.0).collect();
    println!("{:?}", doubled);
    let y = 1 << 33;
    let z: i64 = y;
    println!("{}", z);
    let big = 3_000_000_000;
    let as_u: u32 = big;
    println!("{}", as_u);
    let mut shift = 1;
    shift <<= 40;
    let s2: u64 = shift;
    println!("{}", s2);
    let w = [1, 2, 3].iter().map(|&x| x * 100).collect::<Vec<i64>>();
    println!("{:?}", w);
    let mut k = 250;
    let limit: u8 = 255;
    while k < limit { k += 1; }
    println!("{}", k);
    let mut h = 0;
    for _ in 0..10 { h += 30; }
    let last: u8 = h;
    println!("{}", last);
    let r = 0..5;
    let total: u8 = r.map(|i| i * 50).sum();
    println!("{}", total);
}
