fn average(values: &[i64]) -> i64 {
    let sum: i64 = values.iter().sum();
    let count = values.len() as i64;
    sum / count
}

fn main() {
    println!("{}", average(&[4, 8, 15, 16, 23, 42]));
    println!("{}", 17i64.checked_div(0).is_none());
    println!("{:?}", 17i64.checked_rem(5));
    println!("{}", 7.0f64 / 0.0);
    let empty: Vec<i64> = Vec::new();
    println!("{}", average(&empty));
}
