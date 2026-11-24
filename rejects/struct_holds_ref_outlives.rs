// error: does not live long enough
// known-gap: E0597 through a struct field holding a reference (only plain reference locals are traced)
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
