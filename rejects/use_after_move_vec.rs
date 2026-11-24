// error: use of moved value 'v'
fn consume(v: Vec<i32>) -> usize { v.len() }
fn main() {
    println!("start");
    let v = vec![1, 2, 3];
    let n = consume(v);
    println!("{} {}", n, v.len());
}
