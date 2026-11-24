static LANGUAGE: &str = "Rust";
const THRESHOLD: i32 = 10;
const LIMITS: [u32; 3] = [100, 1_000, 10_000];
const GREETING_LEN: usize = "hello".len();
static UNITS: [(&str, u64); 4] = [("B", 1), ("KB", 1 << 10), ("MB", 1 << 20), ("GB", 1 << 30)];

const fn cube(x: u64) -> u64 { x * x * x }
const CUBE_OF_SEVEN: u64 = cube(7);
const SECONDS_IN_WEEK: u64 = 60 * 60 * 24 * 7;

struct Config;

impl Config {
    const MAX_USERS: usize = 3;
    const NAME: &'static str = "demo-service";
    const VERSION: (u8, u8, u8) = (1, 4, 2);
}

trait Shape {
    const SIDES: u32;
    fn describe() -> String { format!("{} sides", Self::SIDES) }
}

struct Triangle;
struct Hexagon;
impl Shape for Triangle { const SIDES: u32 = 3; }
impl Shape for Hexagon { const SIDES: u32 = 6; }

fn is_big(n: i32) -> bool { n > THRESHOLD }

fn human_size(bytes: u64) -> String {
    let (unit, size) = UNITS.iter().rev().find(|(_, s)| bytes >= *s).copied().unwrap_or(("B", 1));
    format!("{:.1} {}", bytes as f64 / size as f64, unit)
}

fn tier(n: u32) -> usize {
    LIMITS.iter().position(|&l| n < l).unwrap_or(LIMITS.len())
}

fn main() {
    println!("This is {}", LANGUAGE);
    println!("The threshold is {}", THRESHOLD);
    for n in [5, 10, 16] {
        println!("{} is {}", n, if is_big(n) { "big" } else { "small" });
    }
    println!("GREETING_LEN = {}, CUBE_OF_SEVEN = {}, SECONDS_IN_WEEK = {}", GREETING_LEN, CUBE_OF_SEVEN, SECONDS_IN_WEEK);
    for b in [512, 2048, 5_000_000, 3 << 30] {
        println!("{:>12} bytes = {}", b, human_size(b));
    }
    for n in [7, 150, 9_999, 50_000] {
        println!("{} is in tier {}", n, tier(n));
    }
    println!("{} v{}.{}.{} allows {} users", Config::NAME, Config::VERSION.0, Config::VERSION.1, Config::VERSION.2, Config::MAX_USERS);
    println!("Triangle: {}; Hexagon: {}", Triangle::describe(), Hexagon::describe());
    println!("total sides {}", Triangle::SIDES + Hexagon::SIDES);
    let buf = [0u8; Config::MAX_USERS * 2];
    println!("buffer sized from const: {:?}", buf);
    let lang_ref: &'static str = LANGUAGE;
    println!("static ref lives forever: {} ({} chars)", lang_ref, lang_ref.len());
}
