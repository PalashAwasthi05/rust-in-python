// dbg! returns its value (prints to stderr), then todo! panics with exit 101.
#[derive(Debug, Clone)]
enum Cmd {
    Inc(i32),
    Dec(i32),
    Reset,
    Fancy(String),
}

fn apply(total: i32, cmd: &Cmd) -> i32 {
    match cmd {
        Cmd::Inc(n) => total + n,
        Cmd::Dec(n) => total - n,
        Cmd::Reset => 0,
        Cmd::Fancy(name) => todo!("fancy command {}", name),
    }
}

fn main() {
    let a = dbg!(2 + 3);
    let (b, c) = dbg!(a * 2, "pair");
    println!("a={} b={} c={}", a, b, c);
    let cmds = vec![Cmd::Inc(5), Cmd::Dec(2), Cmd::Inc(10), Cmd::Reset, Cmd::Inc(1), Cmd::Fancy("spin".into())];
    let mut total = 0;
    for cmd in &cmds {
        total = apply(total, cmd);
        println!("{:?} -> {}", cmd, total);
    }
    println!("done {}", total);
}
