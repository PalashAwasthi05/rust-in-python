// Sieve of Eratosthenes up to 200_000: Vec<bool> index writes in a tight loop.
fn sieve(limit: usize) -> Vec<bool> {
    let mut composite = vec![false; limit + 1];
    composite[0] = true;
    composite[1] = true;
    let mut i = 2;
    while i * i <= limit {
        if !composite[i] {
            let mut j = i * i;
            while j <= limit {
                composite[j] = true;
                j += i;
            }
        }
        i += 1;
    }
    composite
}

fn main() {
    let limit = 200_000;
    let composite = sieve(limit);
    let mut count = 0;
    let mut sum: u64 = 0;
    let mut last = 0;
    for n in 0..=limit {
        if !composite[n] {
            count += 1;
            sum += n as u64;
            last = n;
        }
    }
    println!("primes up to {}: {}", limit, count);
    println!("sum: {}", sum);
    println!("largest: {}", last);
    let twins = (3..limit - 1).filter(|&n| !composite[n] && !composite[n + 2]).count();
    println!("twin pairs: {}", twins);
}
