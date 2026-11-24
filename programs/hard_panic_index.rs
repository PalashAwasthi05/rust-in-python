struct Ring { slots: Vec<&'static str> }

impl Ring {
    fn get_wrapped(&self, i: usize) -> &str {
        self.slots[i % self.slots.len()]
    }

    fn get_raw(&self, i: usize) -> &str {
        self.slots[i]
    }

    fn try_get(&self, i: usize) -> Option<&str> {
        self.slots.get(i).copied()
    }
}

fn main() {
    let ring = Ring { slots: vec!["north", "east", "south", "west"] };
    for i in 0..6 {
        println!("wrapped {} -> {}", i, ring.get_wrapped(i));
    }
    println!("try_get(2): {:?}", ring.try_get(2));
    println!("try_get(9): {:?}", ring.try_get(9));
    let lookups = [1, 3, 4, 0];
    let mut seen = Vec::new();
    for &i in &lookups {
        let dir = ring.get_raw(i);
        seen.push(dir);
        println!("raw {} -> {} (seen {})", i, dir, seen.len());
    }
    println!("unreachable: {:?}", seen);
}
