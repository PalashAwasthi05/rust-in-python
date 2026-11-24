// error: constant evaluation requires a const function
fn compute() -> i32 { 42 }
const VALUE: i32 = compute();
fn main() {
    println!("start");
    println!("{}", VALUE);
}
