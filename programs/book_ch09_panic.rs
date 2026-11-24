// Chapter 9.1: unrecoverable errors. Recoverable work prints first, then an index past the end panics.
fn checked_get(v: &[i32], index: usize) -> Option<i32> {
    v.get(index).copied()
}

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("cleaning up {}", self.0);
    }
}

fn risky(v: &[i32], index: usize) -> i32 {
    let _g = Guard("risky");
    println!("reading index {index}");
    v[index]
}

fn main() {
    let v = vec![1, 2, 3];
    println!("{:?} {:?}", checked_get(&v, 1), checked_get(&v, 99));
    println!("value {}", risky(&v, 2));
    let _outer = Guard("main");
    let total: i32 = v.iter().sum();
    println!("total {total}");
    println!("about to panic");
    let x = risky(&v, 99);
    println!("unreachable {x}");
}
