fn main() {
    let small: Vec<u8> = vec![100, 100, 50];
    let s: u32 = small.iter().map(|&x| x as u32).sum();
    println!("{}", s);
    let wide: u8 = small.iter().fold(0u8, |a, &b| a.wrapping_add(b));
    println!("{}", wide);
    let checked = small.iter().try_fold(0u8, |a, &b| a.checked_add(b));
    println!("{:?}", checked);
    let p: u64 = (1..=20u64).product();
    println!("{}", p);
    let total: u8 = small.iter().sum();
    println!("unreachable {}", total);
}
