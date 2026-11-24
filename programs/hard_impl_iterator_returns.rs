fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
}

fn squares_of<'a>(xs: &'a [i64]) -> impl Iterator<Item = i64> + 'a {
    xs.iter().map(|x| x * x)
}

fn words<'a>(text: &'a str) -> impl Iterator<Item = String> + 'a {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
}

fn primes() -> impl Iterator<Item = u64> {
    (2u64..).filter(|&n| (2..).take_while(|d| d * d <= n).all(|d| n % d != 0))
}

fn triangle_numbers() -> impl Iterator<Item = u64> {
    (1u64..).scan(0, |acc, n| {
        *acc += n;
        Some(*acc)
    })
}

fn pairs(n: u32) -> impl Iterator<Item = (u32, u32)> {
    (1..=n).flat_map(move |a| (a + 1..=n).map(move |b| (a, b)))
}

fn running_max<I>(it: I) -> impl Iterator<Item = i32>
where
    I: IntoIterator<Item = i32>,
{
    it.into_iter().scan(i32::MIN, |m, x| {
        *m = (*m).max(x);
        Some(*m)
    })
}

fn chunks_summed(xs: &[u32], size: usize) -> impl Iterator<Item = u32> + '_ {
    xs.chunks(size).map(|c| c.iter().sum())
}

fn either(flag: bool) -> Box<dyn Iterator<Item = u8>> {
    if flag { Box::new((1..=3).rev()) } else { Box::new(std::iter::repeat(7).take(2)) }
}

fn main() {
    println!("evens: {:?}", evens(11).collect::<Vec<_>>());
    println!("squares: {:?}", squares_of(&[-3, 2, 5]).collect::<Vec<_>>());
    println!("words: {:?}", words("Hello, World! Rust's iterators -- rock.").collect::<Vec<_>>());
    println!("first primes: {:?}", primes().take(10).collect::<Vec<_>>());
    println!("100th prime: {}", primes().nth(99).unwrap());
    println!("triangles: {:?}", triangle_numbers().take_while(|&t| t < 60).collect::<Vec<_>>());
    println!("pairs(4): {:?}", pairs(4).collect::<Vec<_>>());
    println!("pair count(10): {}", pairs(10).count());
    println!("running max: {:?}", running_max(vec![3, 1, 4, 1, 5, 9, 2, 6]).collect::<Vec<_>>());
    println!("chunk sums: {:?}", chunks_summed(&[1, 2, 3, 4, 5, 6, 7], 3).collect::<Vec<_>>());
    println!("either: {:?} {:?}", either(true).collect::<Vec<_>>(), either(false).collect::<Vec<_>>());
    let total: u64 = primes().skip_while(|&p| p < 50).take(5).sum();
    println!("sum of 5 primes from 50: {}", total);
}
