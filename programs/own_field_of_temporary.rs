struct D(&'static str, u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct Pair {
    a: D,
    b: D,
    n: u32,
}
fn mk() -> Pair {
    Pair { a: D("pa", 1), b: D("pb", 2), n: 7 }
}
fn mkv() -> Vec<D> {
    vec![D("v0", 0), D("v1", 1)]
}
fn take_a() -> D {
    mk().a
}
fn unwrap_it() -> D {
    Some(D("some", 3)).unwrap()
}
fn num() -> u32 {
    mkv()[1].1 + mk().n
}
fn main() {
    let a = take_a();
    println!("took {}", a.0);
    let u = unwrap_it();
    println!("unwrapped {}", u.0);
    println!("num {}", num());
    let n = mk().b.1;
    println!("n {}", n);
    let first = mkv().into_iter().next().unwrap();
    println!("first {}", first.0);
    let len = mkv().len();
    println!("len {}", len);
    let s = (mk().n, mk().a.1);
    println!("{:?}", s);
    println!("end");
}
