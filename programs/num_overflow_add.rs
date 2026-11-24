fn main() {
    let v: Vec<u64> = vec![10, 3];
    println!("{}", v[0] % v[1]);
    let mut x: u8 = 250;
    for _ in 0..10 {
        x += 1;
        println!("{}", x);
    }
}
