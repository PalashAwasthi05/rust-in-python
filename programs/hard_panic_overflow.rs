#[derive(Debug)]
struct Meter { name: &'static str, reading: u8 }

impl Meter {
    fn checked_bump(&self, by: u8) -> Option<u8> {
        self.reading.checked_add(by)
    }

    fn bump(&mut self, by: u8) {
        self.reading += by;
        println!("{} now reads {}", self.name, self.reading);
    }
}

fn checked_total(items: &[u8]) -> Option<u8> {
    items.iter().try_fold(0u8, |acc, &x| acc.checked_add(x))
}

fn main() {
    let small = [10u8, 20, 30];
    let big = [100u8, 100, 60, 1];
    println!("checked small: {:?}", checked_total(&small));
    println!("checked big: {:?}", checked_total(&big));
    println!("wrapping big: {}", big.iter().fold(0u8, |a, &x| a.wrapping_add(x)));

    let mut m = Meter { name: "gauge", reading: 200 };
    println!("checked bump 50: {:?}", m.checked_bump(50));
    println!("checked bump 60: {:?}", m.checked_bump(60));
    for step in [30u8, 20, 10] {
        println!("bumping by {}", step);
        m.bump(step);
    }
    println!("never printed {:?}", m);
}
