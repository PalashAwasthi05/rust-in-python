use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    const ZERO: Vec2 = Vec2 { x: 0, y: 0 };
    fn new(x: i32, y: i32) -> Self {
        Vec2 { x, y }
    }
    fn manhattan(self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<i32> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: i32) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

impl Mul<Vec2> for i32 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        v * self
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}

struct Grid {
    w: usize,
    cells: Vec<char>,
}

impl Index<Vec2> for Grid {
    type Output = char;
    fn index(&self, p: Vec2) -> &char {
        &self.cells[p.y as usize * self.w + p.x as usize]
    }
}

impl IndexMut<Vec2> for Grid {
    fn index_mut(&mut self, p: Vec2) -> &mut char {
        &mut self.cells[p.y as usize * self.w + p.x as usize]
    }
}

fn main() {
    let a = Vec2::new(3, -4);
    let b = Vec2::new(1, 2);
    println!("{:?} {:?} {:?}", a + b, a - b, -a);
    println!("{:?} {:?}", a * 3, 2 * b);
    let mut pos = Vec2::ZERO;
    let moves = [Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(-1, 0)];
    for m in moves.iter() {
        pos += *m;
    }
    println!("pos = {:?}, dist = {}", pos, pos.manhattan());
    println!("eq: {} {}", a + b == Vec2::new(4, -2), pos == Vec2::ZERO);
    let mut g = Grid { w: 4, cells: vec!['.'; 12] };
    let mut p = Vec2::ZERO;
    for d in [Vec2::new(1, 0), Vec2::new(1, 1), Vec2::new(1, 0), Vec2::new(0, 1)] {
        p += d;
        g[p] = '#';
    }
    g[Vec2::ZERO] = 'S';
    for row in 0..3 {
        let line: String = (0..4).map(|c| g[Vec2::new(c, row)]).collect();
        println!("{}", line);
    }
    let sum = moves.iter().fold(Vec2::ZERO, |acc, &m| acc + m);
    println!("sum = {:?}", sum);
}
