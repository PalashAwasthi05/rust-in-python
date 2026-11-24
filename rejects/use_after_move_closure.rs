// error: use of moved value 'data'
fn main() {
    println!("start");
    let data = vec![1, 2, 3];
    let f = move || data.len();
    println!("{}", f());
    println!("{:?}", data);
}
