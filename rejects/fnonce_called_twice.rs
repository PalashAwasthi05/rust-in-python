// error: use of moved value 'f'
fn main() {
    println!("start");
    let s = String::new();
    let f = move || s;
    let _a = f();
    let _b = f();
}
