use std::marker::PhantomData;
trait Pilot { fn fly(&self) -> String; fn name() -> String { "pilot".into() } }
trait Wizard { fn fly(&self) -> String; fn name() -> String { "wizard".into() } }
struct Human;
impl Pilot for Human { fn fly(&self) -> String { "captain speaking".into() } }
impl Wizard for Human { fn fly(&self) -> String { "up!".into() } }
impl Human { fn fly(&self) -> String { "*waving arms*".into() } }
#[derive(Debug, Clone, Copy, PartialEq)]
enum Color { Red = 0xff0000, Green = 0x00ff00, Blue = 0x0000ff }
struct Meters<T> { value: f64, unit: PhantomData<T> }
struct Metric;
fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let Some((a, b)) = s.split_once(',') else { return None; };
    let (Ok(x), Ok(y)) = (a.trim().parse::<i32>(), b.trim().parse::<i32>()) else { return None; };
    Some((x, y))
}
fn main() {
    let h = Human;
    println!("{} | {} | {} | {}", h.fly(), Pilot::fly(&h), <Human as Wizard>::fly(&h), <Human as Pilot>::name());
    for c in 'a'..='e' { print!("{c}"); }
    println!();
    let letters: String = ('a'..'f').rev().collect();
    println!("{letters} {:?}", ('x'..='z').map(|c| c as u32).collect::<Vec<_>>());
    println!("{:06x} {}", Color::Green as i32, Color::Blue as u32);
    let m: Meters<Metric> = Meters { value: 3.5, unit: PhantomData };
    println!("{}", m.value);
    println!("{} {} {} {}", 300.7f64 as u8, -5.9f64 as u32, f64::NAN as i32, 1e20 as i64);
    println!("{} {}", -1i32 as u32, 256u32 as u8);
    println!("{:?} {:?}", parse_pair("3, 4"), parse_pair("x,1"));
    let v = vec![10, 20, 30];
    let total: i32 = v.iter().copied().reduce(|a, b| a + b).unwrap_or(0);
    println!("{total}");
    let s = String::from("hello");
    let r: &dyn std::fmt::Display = &s;
    println!("{r:>8}|{:<6}|{:^9}|", "ab", format!("{}", 42));
    println!("{{literal}} {0} {name} {0}", 7, name = "n");
}
