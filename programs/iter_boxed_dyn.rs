// Box<dyn Iterator> chains built conditionally at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    All,
    Evens,
    Odds,
    Squares,
}

fn numbers(mode: Mode, limit: u32) -> Box<dyn Iterator<Item = u32>> {
    let base = 1..=limit;
    match mode {
        Mode::All => Box::new(base),
        Mode::Evens => Box::new(base.filter(|n| n % 2 == 0)),
        Mode::Odds => Box::new(base.filter(|n| n % 2 == 1)),
        Mode::Squares => Box::new(base.map(|n| n * n)),
    }
}

fn pipeline(data: Vec<i32>, reverse: bool, skip_negative: bool, limit: Option<usize>) -> Vec<i32> {
    let mut it: Box<dyn Iterator<Item = i32>> = Box::new(data.into_iter());
    if skip_negative {
        it = Box::new(it.filter(|x| *x >= 0));
    }
    if reverse {
        let collected: Vec<i32> = it.collect();
        it = Box::new(collected.into_iter().rev());
    }
    if let Some(n) = limit {
        it = Box::new(it.take(n));
    }
    it.collect()
}

struct Up(u8);
impl Iterator for Up {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 >= 5 {
            None
        } else {
            self.0 += 1;
            Some(self.0)
        }
    }
}

fn sources() -> Vec<Box<dyn Iterator<Item = u8>>> {
    vec![
        Box::new(Up(0)),
        Box::new(vec![9u8, 8, 7].into_iter()),
        Box::new(std::iter::once(42)),
        Box::new(std::iter::empty()),
        Box::new(std::iter::repeat(1).take(3)),
    ]
}

fn sum_dyn(it: &mut dyn Iterator<Item = u8>) -> u32 {
    let mut total = 0u32;
    for x in it {
        total += x as u32;
    }
    total
}

fn main() {
    for mode in [Mode::All, Mode::Evens, Mode::Odds, Mode::Squares] {
        let v: Vec<u32> = numbers(mode, 8).collect();
        println!("{:?}: {:?}", mode, v);
    }
    let summed: u32 = numbers(Mode::Squares, 5).sum();
    println!("sum of squares {}", summed);

    let data = vec![5, -3, 8, -1, 0, 12, 7];
    println!("{:?}", pipeline(data.clone(), false, false, None));
    println!("{:?}", pipeline(data.clone(), true, true, None));
    println!("{:?}", pipeline(data.clone(), false, true, Some(2)));
    println!("{:?}", pipeline(data, true, false, Some(3)));

    for (i, src) in sources().into_iter().enumerate() {
        let items: Vec<u8> = src.collect();
        println!("source {} -> {:?}", i, items);
    }
    let all: Vec<u8> = sources().into_iter().flatten().collect();
    println!("flattened {:?}", all);

    let mut up = Up(2);
    println!("dyn sum {}", sum_dyn(&mut up));
    let mut v = vec![1u8, 2, 3].into_iter();
    println!("dyn sum {}", sum_dyn(&mut v));

    let chained: Box<dyn Iterator<Item = u8>> = Box::new(Up(3).chain(Up(0).take(2)));
    let shown: Vec<String> = chained.map(|b| b.to_string()).collect();
    println!("chained {}", shown.join(","));
}
