// error: use of moved value 'name'
struct Holder { name: String }
fn main() {
    println!("start");
    let name = String::from("n");
    let h = Holder { name };
    println!("{} {}", h.name, name);
}
