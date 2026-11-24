// std::iter constructors: from_fn, successors, repeat_with, once, empty, repeat.
use std::iter;

fn main() {
    let mut state = (0u32, 1u32);
    let fib: Vec<u32> = iter::from_fn(|| {
        let out = state.0;
        state = (state.1, state.0 + state.1);
        if out > 100 { None } else { Some(out) }
    })
    .collect();
    println!("from_fn fib {:?}", fib);

    let powers: Vec<u64> = iter::successors(Some(1u64), |&x| x.checked_mul(10)).collect();
    println!("powers of ten: {} values, last {}", powers.len(), powers.last().unwrap());

    let halves: Vec<f64> = iter::successors(Some(100.0f64), |x| if *x > 1.0 { Some(x / 2.0) } else { None }).collect();
    println!("halves {:?}", halves);

    let digits: Vec<u32> = iter::successors(Some(98765u32), |n| if *n >= 10 { Some(n / 10) } else { None })
        .map(|n| n % 10)
        .collect();
    println!("digits reversed {:?}", digits);

    let mut seed = 7u32;
    let pseudo: Vec<u32> = iter::repeat_with(|| {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        seed % 100
    })
    .take(6)
    .collect();
    println!("pseudo {:?}", pseudo);

    let mut n = 0;
    let labels: Vec<String> = iter::repeat_with(|| {
        n += 1;
        format!("item{}", n)
    })
    .take(3)
    .collect();
    println!("labels {:?}", labels);

    let framed: Vec<i32> = iter::once(0).chain(vec![5, 6, 7]).chain(iter::once(99)).collect();
    println!("framed {:?}", framed);

    let nothing: Vec<String> = iter::empty::<String>().collect();
    println!("empty {:?} count {}", nothing, iter::empty::<u8>().count());

    let dashes: String = iter::repeat('-').take(10).collect();
    println!("{}", dashes);
    let row: Vec<Vec<u8>> = iter::repeat(vec![0u8; 3]).take(2).collect();
    println!("rows {:?}", row);

    let mut countdown = 3;
    let launch: Vec<String> = iter::from_fn(|| {
        if countdown == 0 {
            return None;
        }
        countdown -= 1;
        Some(format!("T-{}", countdown + 1))
    })
    .chain(iter::once("liftoff".to_string()))
    .collect();
    println!("{}", launch.join(" "));

    let interleaved: Vec<i32> = (1..4).flat_map(|x| iter::once(x).chain(iter::once(-x))).collect();
    println!("interleaved {:?}", interleaved);

    let triangle: Vec<u32> = iter::successors(Some((1u32, 1u32)), |&(i, t)| Some((i + 1, t + i + 1)))
        .map(|(_, t)| t)
        .take_while(|&t| t < 50)
        .collect();
    println!("triangle {:?}", triangle);

    let sum: i32 = iter::repeat(3).take(4).chain(iter::empty()).sum();
    println!("sum {}", sum);
}
