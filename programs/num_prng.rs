struct XorShift64 { state: u64 }
impl XorShift64 {
    fn new(seed: u64) -> Self { XorShift64 { state: seed } }
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn next_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo) as u64) as i32
    }
}

struct Lcg(u32);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }
}

struct Pcg32 { state: u64, inc: u64 }
impl Pcg32 {
    fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6364136223846793005).wrapping_add(self.inc | 1);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        (xorshifted >> rot) | (xorshifted << ((!rot).wrapping_add(1) & 31))
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

fn main() {
    let mut rng = XorShift64::new(88172645463325252);
    for _ in 0..5 {
        println!("{}", rng.next());
    }
    for _ in 0..3 {
        println!("{}", rng.next_f64());
    }
    let rolls: Vec<i32> = (0..10).map(|_| rng.range(1, 7)).collect();
    println!("{:?}", rolls);
    let mut lcg = Lcg(42);
    let v: Vec<u32> = (0..5).map(|_| lcg.next()).collect();
    println!("{:?}", v);
    let mut pcg = Pcg32 { state: 0x853c49e6748fea9b, inc: 0xda3e39cb94b95bdb };
    for _ in 0..4 {
        println!("{}", pcg.next_u32());
    }
    let mut s = 1234567u64;
    for _ in 0..3 {
        println!("{:x}", splitmix64(&mut s));
    }
    let mut s32 = 2463534242u32;
    let mut hist = [0u32; 4];
    for _ in 0..10000 {
        hist[(xorshift32(&mut s32) >> 30) as usize] += 1;
    }
    println!("{:?}", hist);
    // Fisher-Yates shuffle
    let mut deck: Vec<u32> = (1..=10).collect();
    let mut r = XorShift64::new(7);
    for i in (1..deck.len()).rev() {
        let j = (r.next() % (i as u64 + 1)) as usize;
        deck.swap(i, j);
    }
    println!("{:?}", deck);
    // monte carlo pi
    let mut r = XorShift64::new(12345);
    let mut inside = 0;
    let n = 20000;
    for _ in 0..n {
        let x = r.next_f64();
        let y = r.next_f64();
        if x * x + y * y <= 1.0 { inside += 1; }
    }
    println!("{:.4}", 4.0 * inside as f64 / n as f64);
}
