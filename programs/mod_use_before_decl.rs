// Items used before they are declared, at every level.
fn main() {
    let c = Counter::start(helpers::seed());
    println!("{}", c.describe());
    println!("{}", later::deep::value());
    println!("{:?}", MODE);
    println!("{}", LIMIT + helpers::bump(1));
    let s: Score = 12;
    println!("score {}", s);
}

type Score = u64;

const LIMIT: i32 = helpers::BASE * 2;

static MODE: Mode = Mode::Fast;

#[derive(Debug)]
enum Mode {
    Fast,
    #[allow(dead_code)]
    Slow,
}

struct Counter {
    n: i32,
}

impl Counter {
    fn start(n: i32) -> Counter {
        Counter { n: n + later::OFFSET }
    }
    fn describe(&self) -> String {
        format!("counter={}", self.n)
    }
}

mod helpers {
    pub const BASE: i32 = super::later::OFFSET + 5;

    pub fn seed() -> i32 {
        bump(BASE)
    }

    pub fn bump(x: i32) -> i32 {
        x + 1
    }
}

mod later {
    pub const OFFSET: i32 = 10;

    pub mod deep {
        pub fn value() -> String {
            format!("deep {}", super::OFFSET + crate::helpers::BASE)
        }
    }
}
