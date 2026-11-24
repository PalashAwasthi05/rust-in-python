fn fact(n: u64) -> u64 { if n == 0 { 1 } else { n * fact(n - 1) } }
fn main() {
    for i in [5u64, 10, 20] {
        println!("{}! = {}", i, fact(i));
    }
    println!("{}", fact(21));
}
