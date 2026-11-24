use std::collections::HashMap;

// fixed-point Q16.16
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Fx(i32);
impl Fx {
    const ONE: Fx = Fx(1 << 16);
    fn from_f64(v: f64) -> Fx { Fx((v * 65536.0).round() as i32) }
    fn to_f64(self) -> f64 { self.0 as f64 / 65536.0 }
    fn mul(self, o: Fx) -> Fx { Fx(((self.0 as i64 * o.0 as i64) >> 16) as i32) }
    fn div(self, o: Fx) -> Fx { Fx((((self.0 as i64) << 16) / o.0 as i64) as i32) }
}

fn bitboard_moves(knight: u64) -> u64 {
    let not_a = 0xfefefefefefefefeu64;
    let not_ab = 0xfcfcfcfcfcfcfcfcu64;
    let not_h = 0x7f7f7f7f7f7f7f7fu64;
    let not_gh = 0x3f3f3f3f3f3f3f3fu64;
    ((knight << 17) & not_a) | ((knight << 15) & not_h) | ((knight << 10) & not_ab) | ((knight << 6) & not_gh)
        | ((knight >> 17) & not_h) | ((knight >> 15) & not_a) | ((knight >> 10) & not_gh) | ((knight >> 6) & not_ab)
}

fn collatz_len(mut n: u64, cache: &mut HashMap<u64, u32>) -> u32 {
    let start = n;
    let mut steps = 0;
    let mut path = Vec::new();
    while n != 1 {
        if let Some(&c) = cache.get(&n) { steps += c; break; }
        path.push(n);
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    let total = steps;
    for (i, p) in path.iter().enumerate() {
        cache.insert(*p, total - i as u32);
    }
    let _ = start;
    total
}

fn mod_pow(mut b: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64;
    b %= m;
    while e > 0 {
        if e & 1 == 1 { r = ((r as u128 * b as u128) % m as u128) as u64; }
        b = ((b as u128 * b as u128) % m as u128) as u64;
        e >>= 1;
    }
    r
}

fn is_prime_mr(n: u64) -> bool {
    if n < 2 { return false; }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 { return n == p; }
    }
    let mut d = n - 1;
    let mut s = 0;
    while d % 2 == 0 { d /= 2; s += 1; }
    'outer: for a in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..s - 1 {
            x = ((x as u128 * x as u128) % n as u128) as u64;
            if x == n - 1 { continue 'outer; }
        }
        return false;
    }
    true
}

fn main() {
    let a = Fx::from_f64(3.25);
    let b = Fx::from_f64(-1.5);
    println!("{:?} {:?} {} {}", a, b, a.mul(b).to_f64(), a.div(b).to_f64());
    println!("{}", Fx::ONE.0);
    println!("{}", a > b);
    let moves = bitboard_moves(1 << 27);
    println!("{:016x} {}", moves, moves.count_ones());
    println!("{}", bitboard_moves(1).count_ones());
    let mut cache = HashMap::new();
    let best = (1..3000u64).max_by_key(|&n| collatz_len(n, &mut cache)).unwrap();
    println!("{} {}", best, collatz_len(best, &mut cache));
    println!("{}", mod_pow(2, 1_000_000_007 - 2, 1_000_000_007));
    let primes: Vec<u64> = [1_000_000_007u64, 1_000_000_009, 999_999_999_989, 18446744073709551557, 18446744073709551559]
        .iter().copied().filter(|&n| is_prime_mr(n)).collect();
    println!("{:?}", primes);
    let x: i32 = -17;
    println!("{} {} {}", x / 4, x >> 2, x.div_euclid(4));
    println!("{} {}", (x as u32) >> 28, x as u32 >> 28);
    let mut bits: u128 = 0;
    for i in (0..128).step_by(3) { bits |= 1 << i; }
    println!("{:x} {}", bits, bits.count_ones());
    println!("{}", u64::MAX.leading_zeros() + (1u64 << 63).leading_zeros());
    let ratio = 22.0_f64 / 7.0;
    println!("{:.10} {}", ratio, (ratio * 1e6).floor() / 1e6);
    let total_cents: i64 = [1999i64, 2501, -300].iter().sum();
    println!("{}.{:02}", total_cents / 100, total_cents % 100);
}
