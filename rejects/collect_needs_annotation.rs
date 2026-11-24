// error: type annotations needed
fn main() {
    println!("start");
    let v = vec![1, 2];
    let w = v.iter().map(|x| x * 2).collect();
    println!("{:?}", w);
}
