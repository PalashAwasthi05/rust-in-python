// Iterators that hand out mutable references, generic state machines, and iterators of iterators.
struct Ring<T> {
    items: Vec<T>,
    start: usize,
}

impl<T: Clone> Ring<T> {
    fn cycle_from(&self, start: usize) -> impl Iterator<Item = T> + '_ {
        let n = self.items.len();
        (0..n).map(move |i| self.items[(start + i) % n].clone())
    }
    fn rotations(&self) -> impl Iterator<Item = Vec<T>> + '_ {
        (0..self.items.len()).map(move |s| self.cycle_from(s).collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Light {
    Red,
    Green,
    Yellow,
}

struct Signal {
    state: Light,
    ticks: u32,
}

impl Iterator for Signal {
    type Item = (u32, Light);
    fn next(&mut self) -> Option<(u32, Light)> {
        if self.ticks == 0 {
            return None;
        }
        self.ticks -= 1;
        let current = self.state;
        self.state = match current {
            Light::Red => Light::Green,
            Light::Green => Light::Yellow,
            Light::Yellow => Light::Red,
        };
        Some((self.ticks, current))
    }
}

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<i32>,
}

impl Matrix {
    fn cells_mut(&mut self) -> impl Iterator<Item = (usize, &mut i32)> + '_ {
        let cols = self.cols;
        self.data.iter_mut().enumerate().map(move |(i, x)| (i / cols, x))
    }
    fn column(&self, c: usize) -> impl Iterator<Item = &i32> + '_ {
        self.data.iter().skip(c).step_by(self.cols)
    }
}

fn main() {
    let ring = Ring { items: vec!['a', 'b', 'c', 'd'], start: 0 };
    let s: String = ring.cycle_from(2 + ring.start).collect();
    println!("from 2: {}", s);
    for r in ring.rotations() {
        println!("{:?}", r);
    }

    let lights: Vec<String> = Signal { state: Light::Red, ticks: 5 }.map(|(t, l)| format!("{}:{:?}", t, l)).collect();
    println!("{}", lights.join(" "));
    let greens = Signal { state: Light::Yellow, ticks: 9 }.filter(|&(_, l)| l == Light::Green).count();
    println!("greens {}", greens);

    let mut m = Matrix { rows: 3, cols: 3, data: (1..=9).collect() };
    for (row, x) in m.cells_mut() {
        *x *= (row + 1) as i32;
    }
    println!("{:?}", m.data);
    let col1: Vec<&i32> = m.column(1).collect();
    println!("col 1 {:?} of {} rows", col1, m.rows);

    let mut values = vec![3, 1, 4, 1, 5, 9, 2, 6];
    values.iter_mut().filter(|x| **x % 2 == 1).for_each(|x| *x *= 100);
    println!("{:?}", values);
    for (i, v) in values.iter_mut().enumerate().rev().take(2) {
        *v = -(i as i32);
    }
    println!("{:?}", values);
    if let Some(m) = values.iter_mut().max() {
        *m = 0;
    }
    println!("{:?}", values);

    let nested = vec![vec![1, 2], vec![], vec![3]];
    let lens: Vec<usize> = nested.iter().map(Vec::len).collect();
    let flat: Vec<i32> = nested.iter().flatten().copied().collect();
    let flat_map: Vec<String> = nested.iter().enumerate().flat_map(|(i, v)| v.iter().map(move |x| format!("{}.{}", i, x))).collect();
    println!("{:?} {:?} {:?}", lens, flat, flat_map);
}
