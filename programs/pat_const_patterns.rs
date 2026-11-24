// Associated and local consts as patterns and range bounds, including Self:: paths inside matches!.
fn level(n: u8) -> &'static str {
    match n {
        Grade::TOP => "top",
        Grade::PASS..=Grade::NEAR_TOP => "pass",
        _ => "fail",
    }
}
struct Grade;
impl Grade {
    const TOP: u8 = 10;
    const NEAR_TOP: u8 = Self::TOP - 1;
    const PASS: u8 = 5;
    fn check(&self, n: u8) -> bool {
        matches!(n, Self::TOP)
    }
}
fn main() {
    for n in [10, 9, 5, 2] {
        println!("{} {} {}", n, level(n), Grade.check(n));
    }
    const LOW: i32 = -3;
    match -3 {
        LOW..=0 => println!("low"),
        _ => println!("other"),
    }
}
