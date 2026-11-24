use std::fmt::Debug;

fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T {
    let mut best = xs[0];
    for &x in xs { if x > best { best = x; } }
    best
}

fn largest_ref<T: PartialOrd>(xs: &[T]) -> Option<&T> {
    let mut it = xs.iter();
    let first = it.next()?;
    Some(it.fold(first, |b, x| if x > b { x } else { b }))
}

fn min_max<T>(xs: &[T]) -> Option<(T, T)>
where
    T: PartialOrd + Copy,
{
    let first = *xs.first()?;
    Some(xs.iter().fold((first, first), |(lo, hi), &x| (if x < lo { x } else { lo }, if x > hi { x } else { hi })))
}

fn top_n<T: Ord + Clone + Debug>(xs: &[T], n: usize) -> Vec<T> {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| b.cmp(a));
    v.truncate(n);
    v
}

fn clamp_all<T: PartialOrd + Copy>(xs: &mut [T], lo: T, hi: T) {
    for x in xs.iter_mut() {
        if *x < lo { *x = lo } else if *x > hi { *x = hi }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Score { points: u32, time: f32 }

fn main() {
    println!("largest i32: {}", largest(&[34, 50, 25, 100, 65]));
    println!("largest f64: {}", largest(&[0.5, -2.0, 9.75, 9.7]));
    println!("largest char: {}", largest(&['y', 'm', 'a', 'q']));
    println!("largest tuple: {:?}", largest(&[(1, 'z'), (2, 'a'), (2, 'b')]));
    println!("largest score: {:?}", largest(&[Score { points: 10, time: 3.0 }, Score { points: 10, time: 5.5 }, Score { points: 9, time: 1.0 }]));

    let words = vec!["pear".to_string(), "zucchini".to_string(), "apple".to_string()];
    println!("largest string: {:?}", largest_ref(&words));
    println!("largest of empty: {:?}", largest_ref::<u8>(&[]));
    println!("min_max: {:?}", min_max(&[7, -3, 12, 0]));
    println!("min_max f32: {:?}", min_max(&[2.5f32]));
    println!("min_max empty: {:?}", min_max::<i8>(&[]));
    println!("top 3: {:?}", top_n(&[5, 1, 9, 3, 7, 9], 3));
    println!("top 2 strs: {:?}", top_n(&["b", "d", "a", "c"], 2));
    let mut temps = [-12.0, 5.5, 41.0, 20.0, -3.0];
    clamp_all(&mut temps, -5.0, 35.0);
    println!("clamped: {:?}", temps);
    let mut bytes = [3u8, 200, 99];
    clamp_all(&mut bytes, 10, 100);
    println!("clamped bytes: {:?}", bytes);
}
