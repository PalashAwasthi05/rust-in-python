fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

fn sieve(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit >= 1 { is_prime[1] = false; }
    let mut i = 2;
    while i * i <= limit {
        if is_prime[i] {
            let mut j = i * i;
            while j <= limit { is_prime[j] = false; j += i; }
        }
        i += 1;
    }
    is_prime.iter().enumerate().filter(|(_, &p)| p).map(|(n, _)| n).collect()
}

fn fib_u128(n: u32) -> u128 {
    let (mut a, mut b) = (0u128, 1u128);
    for _ in 0..n {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

fn factorial_checked(n: u64) -> Option<u64> {
    (1..=n).try_fold(1u64, |acc, x| acc.checked_mul(x))
}

fn digit_sum(mut n: u64) -> u64 {
    let mut s = 0;
    while n > 0 { s += n % 10; n /= 10; }
    s
}

fn pow_mod(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { result = result * base % m; }
        base = base * base % m;
        exp >>= 1;
    }
    result
}

fn main() {
    println!("gcd(84, 36)={} lcm(4, 6)={} lcm(1..=20)={}", gcd(84, 36), lcm(4, 6), (1..=20).fold(1, lcm));
    let primes = sieve(60);
    println!("primes <= 60: {:?}", primes);
    println!("count < 10000: {}", sieve(10000).len());
    println!("fib(10)={} fib(90)={} fib(150)={}", fib_u128(10), fib_u128(90), fib_u128(150));
    for n in [5, 20, 21] {
        match factorial_checked(n) {
            Some(f) => println!("{}! = {}", n, f),
            None => println!("{}! overflows u64", n),
        }
    }
    let x: u8 = 250;
    println!("checked_add {:?} {:?}", x.checked_add(5), x.checked_add(6));
    println!("wrapping_add {} saturating_add {} overflowing_add {:?}", x.wrapping_add(10), x.saturating_add(10), x.overflowing_add(10));
    println!("wrapping_mul i32: {}", i32::MAX.wrapping_mul(3));
    println!("i8 neg wrap: {}, abs diff {}", i8::MIN.wrapping_neg(), 3u32.abs_diff(10));
    println!("-7 / 2 = {}, -7 % 2 = {}, rem_euclid = {}, div_euclid = {}", -7 / 2, -7 % 2, (-7i32).rem_euclid(2), (-7i32).div_euclid(2));
    println!("2^10={} 3^4={} pow_mod(2, 100, 1e9+7)={}", 2u32.pow(10), 3i64.pow(4), pow_mod(2, 100, 1_000_000_007));
    println!("digit_sum(2^40)={}", digit_sum(1 << 40));
    println!("bits: ones(255)={} leading_zeros(1u32)={} trailing_zeros(48)={}", 255u32.count_ones(), 1u32.leading_zeros(), 48u32.trailing_zeros());
    println!("u64::MAX={} i128::MIN={}", u64::MAX, i128::MIN);
    println!("casts: {} {} {} {}", 300i32 as u8, -1i32 as u32, 3.99f64 as i32, (-1.5f64) as u8);
    println!("isqrt-ish: {}", (1..).take_while(|i: &u64| i * i <= 1000).last().unwrap());
}
