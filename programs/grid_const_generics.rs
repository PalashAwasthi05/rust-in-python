// Const generic functions and structs over [T; N].
fn sum<const N: usize>(a: [i32; N]) -> i32 {
    let mut s = 0;
    for x in a {
        s += x;
    }
    s
}

fn largest<T: PartialOrd + Copy, const N: usize>(a: &[T; N]) -> T {
    let mut best = a[0];
    for &x in a.iter() {
        if x > best {
            best = x;
        }
    }
    best
}

fn reversed<T: Copy + Default, const N: usize>(a: [T; N]) -> [T; N] {
    let mut out = [T::default(); N];
    for i in 0..N {
        out[i] = a[N - 1 - i];
    }
    out
}

fn len_of<T, const N: usize>(_: &[T; N]) -> usize {
    N
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Grid<const W: usize, const H: usize> {
    cells: [[u8; W]; H],
}

impl<const W: usize, const H: usize> Grid<W, H> {
    fn new() -> Self {
        Grid { cells: [[0; W]; H] }
    }
    fn set(&mut self, x: usize, y: usize, v: u8) {
        self.cells[y][x] = v;
    }
    fn area(&self) -> usize {
        W * H
    }
    fn render(&self) -> String {
        let mut s = String::new();
        for row in &self.cells {
            for &c in row {
                s.push(if c > 0 { '#' } else { '.' });
            }
            s.push('\n');
        }
        s
    }
    fn count(&self) -> usize {
        self.cells.iter().flat_map(|r| r.iter()).filter(|&&c| c > 0).count()
    }
}

struct Buffer<T, const CAP: usize> {
    data: [T; CAP],
    len: usize,
}

impl<T: Copy + Default + std::fmt::Debug, const CAP: usize> Buffer<T, CAP> {
    fn new() -> Self {
        Buffer { data: [T::default(); CAP], len: 0 }
    }
    fn push(&mut self, v: T) -> Result<(), T> {
        if self.len == CAP {
            return Err(v);
        }
        self.data[self.len] = v;
        self.len += 1;
        Ok(())
    }
    fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}

fn main() {
    println!("{}", sum([1, 2, 3]));
    println!("{}", sum([10; 7]));
    println!("{}", sum([]));
    println!("{}", largest(&[3.5, 9.25, -1.0]));
    println!("{}", largest(&['q', 'z', 'a']));
    println!("{:?}", reversed([1, 2, 3, 4]));
    println!("{:?}", reversed(["a", "b"]));
    println!("{}", len_of(&[0u8; 12]));

    let mut g: Grid<5, 3> = Grid::new();
    g.set(0, 0, 1);
    g.set(4, 2, 1);
    g.set(2, 1, 7);
    print!("{}", g.render());
    println!("area {} count {}", g.area(), g.count());
    let g2 = g;
    println!("{}", g == g2);
    let small = Grid::<2, 2>::new();
    println!("{:?}", small);

    let mut buf: Buffer<i32, 3> = Buffer::new();
    for i in 1..=5 {
        match buf.push(i * 10) {
            Ok(()) => println!("pushed {}", i * 10),
            Err(v) => println!("full, rejected {}", v),
        }
    }
    println!("{:?}", buf.as_slice());
}
