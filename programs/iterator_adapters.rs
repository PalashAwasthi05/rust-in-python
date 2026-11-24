fn main() {
    let v = vec![1, 2, 3, 4, 5, 6];
    let evens: Vec<i32> = v.iter().filter(|x| *x % 2 == 0).map(|x| x * 10).collect();
    println!("{:?}", evens);
    let total: i32 = v.iter().fold(0, |a, b| a + b);
    for (i, x) in v.iter().enumerate().skip(1).take(2) { println!("{i}:{x}"); }
    let z: Vec<(i32, char)> = v.iter().copied().zip("abc".chars()).collect();
    println!("{total} {:?} {}", z, v.iter().any(|&x| x > 5));
    println!("{:?} {:?}", v.iter().max(), v.iter().position(|&x| x == 3));
}
