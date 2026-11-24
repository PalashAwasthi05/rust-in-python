fn factorial(n: u32) -> Option<u128> {
    (1..=n as u128).try_fold(1u128, |acc, x| acc.checked_mul(x))
}

fn binomial(n: u64, k: u64) -> Option<u128> {
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for i in 0..k {
        result = result.checked_mul((n - i) as u128)? / (i + 1) as u128;
    }
    Some(result)
}

fn mul_mod(a: u128, b: u128, m: u128) -> u128 {
    let (mut a, mut b, mut r) = (a % m, b, 0u128);
    while b > 0 {
        if b & 1 == 1 { r = (r + a) % m; }
        a = (a << 1) % m;
        b >>= 1;
    }
    r
}

fn pow_mod(mut base: u128, mut exp: u128, m: u128) -> u128 {
    let mut r = 1;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { r = mul_mod(r, base, m); }
        base = mul_mod(base, base, m);
        exp >>= 1;
    }
    r
}

fn is_probable_prime(n: u128) -> bool {
    if n < 2 { return false; }
    for p in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 { return n == p; }
    }
    let mut d = n - 1;
    let mut s = 0;
    while d % 2 == 0 { d /= 2; s += 1; }
    'witness: for a in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = pow_mod(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..s - 1 {
            x = mul_mod(x, x, n);
            if x == n - 1 { continue 'witness; }
        }
        return false;
    }
    true
}

fn digits(mut n: u128) -> Vec<u8> {
    let mut d = Vec::new();
    while n > 0 { d.push((n % 10) as u8); n /= 10; }
    d.reverse();
    d
}

fn main() {
    for n in [20, 25, 30, 34, 35] {
        match factorial(n) {
            Some(f) => println!("{}! = {}", n, f),
            None => println!("{}! overflows u128", n),
        }
    }
    println!("C(100, 50) = {:?}", binomial(100, 50));
    println!("C(130, 65) = {:?}", binomial(130, 65));
    println!("2^127 = {}", 1u128 << 127);
    println!("u128::MAX = {} ({} digits)", u128::MAX, digits(u128::MAX).len());
    let big = 170_141_183_460_469_231_731_687_303_715_884_105_727u128;
    println!("2^127-1 prime? {}", is_probable_prime(big));
    println!("2^127+1 prime? {}", is_probable_prime(big + 2));
    println!("1_000_000_007 prime? {}", is_probable_prime(1_000_000_007));
    let a: u128 = 0xFFFF_FFFF_FFFF_FFFF;
    println!("a*a = {:?}, a*a*a = {:?}", a.checked_mul(a), a.checked_mul(a).and_then(|x| x.checked_mul(a)));
    println!("wrapping: {}, overflowing: {:?}", u128::MAX.wrapping_add(5), u128::MAX.overflowing_mul(2));
    let sum: u128 = digits(factorial(30).unwrap()).iter().map(|&d| d as u128).sum();
    println!("digit sum of 30! = {}", sum);
    println!("i128 range: {} .. {}", i128::MIN, i128::MAX);
    println!("hex of 2^100: {:#x}", 1u128 << 100);
}
