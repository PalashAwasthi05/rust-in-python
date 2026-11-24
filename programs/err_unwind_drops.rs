struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn level(depth: u32) -> u32 {
    let _guard = Noisy(match depth {
        0 => "level 0",
        1 => "level 1",
        _ => "level 2",
    });
    if depth == 2 {
        let parsed: Result<u32, String> = Err(format!("depth {} unsupported", depth));
        return parsed.unwrap();
    }
    level(depth + 1) + 1
}

fn main() {
    let _outer = Noisy("outer");
    {
        let _inner = Noisy("inner");
    }
    let values = vec![Noisy("a"), Noisy("b")];
    println!("start with {} values", values.len());
    println!("result {}", level(0));
}
