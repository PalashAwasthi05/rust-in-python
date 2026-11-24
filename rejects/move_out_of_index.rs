// error: cannot move out of array index in 'v'
fn main() {
    println!("start");
    let v = vec![String::from("a"), String::from("b")];
    let s = v[0];
    println!("{}", s);
}
