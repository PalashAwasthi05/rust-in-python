// error: does not live long enough
struct H<'a> { r: &'a i32 }
fn main() {
    println!("start");
    let h;
    {
        let x = 1;
        h = H { r: &x };
    }
    println!("{}", h.r);
}
