// error: borrowed value 'x' does not live long enough
fn main() {
    println!("start");
    let r;
    {
        let x = 5;
        r = &x;
    }
    println!("{}", r);
}
