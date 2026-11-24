// A user iterator implementing DoubleEndedIterator and ExactSizeIterator.
struct Span {
    front: i32,
    back: i32,
}

impl Span {
    fn new(lo: i32, hi: i32) -> Span {
        Span { front: lo, back: hi }
    }
}

impl Iterator for Span {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.front >= self.back {
            return None;
        }
        let v = self.front;
        self.front += 1;
        Some(v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.back - self.front).max(0) as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Span {
    fn next_back(&mut self) -> Option<i32> {
        if self.front >= self.back {
            return None;
        }
        self.back -= 1;
        Some(self.back)
    }
}

impl ExactSizeIterator for Span {}

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

struct Noisy {
    lo: i32,
    hi: i32,
}

impl Iterator for Noisy {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.lo >= self.hi {
            return None;
        }
        println!("  front yields {}", self.lo);
        self.lo += 1;
        Some(self.lo - 1)
    }
}

impl DoubleEndedIterator for Noisy {
    fn next_back(&mut self) -> Option<i32> {
        if self.lo >= self.hi {
            return None;
        }
        self.hi -= 1;
        println!("  back yields {}", self.hi);
        Some(self.hi)
    }
}

fn main() {
    let forward: Vec<i32> = Span::new(1, 6).collect();
    let backward: Vec<i32> = Span::new(1, 6).rev().collect();
    println!("{:?} {:?}", forward, backward);

    let mut s = Span::new(0, 10);
    println!("len {}", s.len());
    let a = s.next();
    let b = s.next_back();
    let c = s.next_back();
    println!("{:?} {:?} {:?} len now {}", a, b, c, s.len());
    println!("size_hint {:?}", s.size_hint());

    let mut s = Span::new(0, 5);
    let mut out = Vec::new();
    loop {
        match (s.next(), s.next_back()) {
            (Some(x), Some(y)) => out.push((x, y)),
            (Some(x), None) => {
                out.push((x, -1));
                break;
            }
            _ => break,
        }
    }
    println!("{:?}", out);

    let tail: Vec<i32> = Span::new(0, 20).rev().take(3).collect();
    println!("last three reversed: {:?}", tail);

    let mapped: Vec<i32> = Span::new(1, 5).map(|x| x * x).rev().collect();
    println!("squares reversed: {:?}", mapped);

    let last = Span::new(3, 9).rev().filter(|x| x % 2 == 0).next();
    println!("largest even: {:?}", last);

    let rpos = Span::new(10, 20).rposition(|x| x % 3 == 0);
    println!("rposition: {:?}", rpos);

    let zipped: Vec<(i32, i32)> = Span::new(0, 4).zip(Span::new(0, 4).rev()).collect();
    println!("{:?}", zipped);

    let countdown: Vec<u32> = Countdown(5).collect();
    println!("countdown {:?}", countdown);
    println!("countdown hint {:?}", Countdown(3).size_hint());

    let empty = Span::new(5, 5);
    println!("empty len {} is zero {}", empty.len(), empty.len() == 0);

    println!("lazy rev take(2):");
    let firsts: Vec<i32> = Noisy { lo: 0, hi: 100 }.rev().take(2).collect();
    println!("got {:?}", firsts);
    println!("rev of map:");
    let mut back = Noisy { lo: 0, hi: 3 }.map(|x| x * 10).rev();
    println!("first from back {:?}", back.next());
    println!("next_back on rev {:?}", back.next_back());
    let last_back = Noisy { lo: 5, hi: 8 }.rfold(0, |acc, x| acc * 10 + x);
    println!("rfold {}", last_back);
    let found = Noisy { lo: 0, hi: 9 }.rfind(|x| x % 4 == 0);
    println!("rfind {:?}", found);
}
