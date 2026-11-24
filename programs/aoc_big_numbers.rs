// Big-number AoC chores: CRT for bus schedules (Day 13 2020), modular exponentiation (Day 25 2020),
// overflow-aware products, and a parse that fails at the end like a bad puzzle input would.
const SCHEDULE: &str = r"939
7,13,x,x,59,x,31,19";

fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = (result as u128 * base as u128 % m as u128) as u64;
        }
        base = (base as u128 * base as u128 % m as u128) as u64;
        exp >>= 1;
    }
    result
}

fn ext_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = ext_gcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

fn crt(pairs: &[(i128, i128)]) -> (i128, i128) {
    pairs.iter().fold((0, 1), |(r1, m1), &(r2, m2)| {
        let (_, p, _) = ext_gcd(m1, m2);
        let m = m1 * m2;
        let r = (r1 + (r2 - r1) * p % m2 * m1).rem_euclid(m);
        (r, m)
    })
}

fn main() {
    let (earliest, ids) = SCHEDULE.split_once('\n').unwrap();
    let earliest: u64 = earliest.parse().unwrap();
    let buses: Vec<(usize, u64)> = ids
        .split(',')
        .enumerate()
        .filter_map(|(i, s)| s.parse().ok().map(|id| (i, id)))
        .collect();
    let (wait, id) = buses.iter().map(|&(_, id)| (id - earliest % id, id)).min().unwrap();
    println!("part 1: {} (bus {id} after {wait})", wait * id);
    let pairs: Vec<(i128, i128)> = buses.iter().map(|&(i, id)| ((-(i as i128)).rem_euclid(id as i128), id as i128)).collect();
    let (t, modulus) = crt(&pairs);
    println!("part 2: {t} (mod {modulus})");

    let card = mod_pow(7, 8, 20201227);
    let door = mod_pow(7, 11, 20201227);
    println!("keys {card} {door} -> {} {}", mod_pow(door, 8, 20201227), mod_pow(card, 11, 20201227));

    let big: Vec<u64> = vec![4_000_000_000, 5_000_000_000, 7];
    let checked = big.iter().try_fold(1u64, |acc, &x| acc.checked_mul(x));
    let wide: u128 = big.iter().map(|&x| x as u128).product();
    println!("checked {:?}, u128 {wide}, wrapping {}", checked, big.iter().fold(1u64, |a, &x| a.wrapping_mul(x)));
    println!("{} {} {}", u64::MAX.count_ones(), i64::MIN, (i64::MAX as u64 + 1).trailing_zeros());
    let sat = 250u8.saturating_add(10);
    let (val, over) = 250u8.overflowing_add(10);
    println!("{sat} {val} {over}");
    let bad: u8 = "300".parse().expect("value fits in a byte");
    println!("unreachable {bad}");
}
