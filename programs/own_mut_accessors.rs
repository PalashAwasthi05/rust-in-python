// Methods returning &mut into self, two-phase borrows and NLL.
#[derive(Debug, Default)]
struct Stats {
    hits: u32,
    misses: u32,
    log: Vec<String>,
}

impl Stats {
    fn hits_mut(&mut self) -> &mut u32 {
        &mut self.hits
    }
    fn log_mut(&mut self) -> &mut Vec<String> {
        &mut self.log
    }
    fn bump(&mut self, hit: bool) -> &mut Self {
        if hit { self.hits += 1 } else { self.misses += 1 }
        self
    }
}

struct Grid {
    cells: Vec<Vec<i32>>,
}

impl Grid {
    fn cell_mut(&mut self, r: usize, c: usize) -> Option<&mut i32> {
        self.cells.get_mut(r)?.get_mut(c)
    }
    fn row_max(&self, r: usize) -> &i32 {
        self.cells[r].iter().max().unwrap()
    }
}

fn largest_mut(v: &mut Vec<i32>) -> &mut i32 {
    let mut idx = 0;
    for i in 1..v.len() {
        if v[i] > v[idx] {
            idx = i;
        }
    }
    &mut v[idx]
}

fn main() {
    let mut s = Stats::default();
    *s.hits_mut() += 5;
    s.log_mut().push("start".to_string());
    s.bump(true).bump(false).bump(true);
    {
        let h = s.hits_mut();
        *h *= 2;
    }
    let msg = format!("hits={}", s.hits);
    s.log_mut().push(msg);
    println!("{:?}", s);

    // two-phase borrows
    let mut v = vec![1, 2, 3];
    v.push(v.len() as i32);
    v.push(v[0] + v[1]);
    v.insert(0, v.iter().sum());
    println!("{:?}", v);

    // NLL: shared borrow ends before mutation
    let first = &v[0];
    println!("first = {}", first);
    v.push(100);
    let m = largest_mut(&mut v);
    *m -= 1;
    println!("after largest_mut: {:?}", v);

    let mut g = Grid { cells: vec![vec![1, 2, 3], vec![4, 5, 6]] };
    if let Some(c) = g.cell_mut(1, 2) {
        *c = 60;
    }
    match g.cell_mut(5, 0) {
        Some(_) => println!("unexpected"),
        None => println!("out of range"),
    }
    let mx = *g.row_max(1);
    *g.cell_mut(0, 0).unwrap() = mx;
    println!("{:?}", g.cells);

    // Option<&mut T> via as_mut
    let mut maybe: Option<String> = Some("abc".to_string());
    if let Some(s) = maybe.as_mut() {
        s.push_str("def");
    }
    maybe.as_mut().map(|s| s.make_ascii_uppercase());
    println!("{:?}", maybe);
    let mut none: Option<String> = None;
    println!("{:?}", none.as_mut().map(|s| s.len()));
    let mut boxed: Option<Box<i32>> = Some(Box::new(1));
    if let Some(b) = &mut boxed {
        **b += 41;
    }
    println!("{:?}", boxed);
    let mut words = vec![String::from("a"), String::from("b")];
    if let Some(w) = words.get_mut(1) {
        w.push('!');
    }
    let len = words.len();
    words[len - 1].push('?');
    println!("{:?}", words);
}
