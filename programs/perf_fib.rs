// Call-heavy recursion: naive fib(25) and the Takeuchi function.
fn fib(n: u32) -> u64 {
    if n < 2 { n as u64 } else { fib(n - 1) + fib(n - 2) }
}

fn tak(x: i32, y: i32, z: i32) -> i32 {
    if y < x {
        tak(tak(x - 1, y, z), tak(y - 1, z, x), tak(z - 1, x, y))
    } else {
        z
    }
}

fn main() {
    println!("fib(25) = {}", fib(25));
    println!("tak(18, 12, 6) = {}", tak(18, 12, 6));
}
