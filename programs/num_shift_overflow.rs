fn shift(x: u32, by: u32) -> u32 {
    x << by
}

fn main() {
    println!("{}", shift(1, 31));
    println!("{}", shift(1, 32));
}
