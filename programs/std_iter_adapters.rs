// Iterator adapters and consumers.
fn main() {
    let a = [1, 2, 3];
    let b = ["one", "two", "three"];
    let c = [1.5, 2.5, 3.5];
    for ((n, word), f) in a.iter().zip(b.iter()).zip(c.iter()) {
        println!("{} {} {}", n, word, f);
    }
    let chained: Vec<i32> = a.iter().chain([10, 20].iter()).cloned().collect();
    println!("chain: {:?}", chained);
    let flat: Vec<char> = b.iter().flat_map(|w| w.chars().take(2)).collect();
    println!("flat_map: {:?}", flat);
    let flattened: Vec<i32> = vec![vec![1], vec![2, 3], vec![]].into_iter().flatten().collect();
    println!("flatten: {:?}", flattened);
    let running: Vec<i32> = (1..=6).scan(0, |acc, x| {
        *acc += x;
        Some(*acc)
    }).collect();
    println!("scan: {:?}", running);
    let limited: Vec<i32> = (1..).scan(1, |acc, _| {
        *acc *= 3;
        if *acc > 500 { None } else { Some(*acc) }
    }).collect();
    println!("scan stop: {:?}", limited);
    let small: Vec<i32> = [1, 4, 2, 8, 3].iter().copied().take_while(|&x| x < 5).collect();
    let rest: Vec<i32> = [1, 4, 2, 8, 3].iter().copied().skip_while(|&x| x < 5).collect();
    println!("take_while {:?} skip_while {:?}", small, rest);
    let stepped: Vec<u32> = (0..20).step_by(7).collect();
    println!("step_by: {:?}", stepped);
    let down: Vec<i32> = (1..=5).rev().step_by(2).collect();
    println!("rev step: {:?}", down);
    let cyc: String = "ab".chars().cycle().take(5).collect();
    println!("cycle: {}", cyc);
    let repeated: Vec<&str> = std::iter::repeat("hi").take(3).collect();
    println!("repeat: {:?}", repeated);
    let mut peek = [1, 1, 2, 3, 3, 3, 4].iter().peekable();
    let mut groups = Vec::new();
    while let Some(&x) = peek.next() {
        let mut count = 1;
        while peek.next_if(|&&y| y == x).is_some() {
            count += 1;
        }
        groups.push((x, count));
    }
    println!("runs: {:?}", groups);
    let mut digits = "123abc".chars().peekable();
    let mut num = 0;
    while let Some(d) = digits.next_if(|c| c.is_ascii_digit()) {
        num = num * 10 + d.to_digit(10).unwrap();
    }
    println!("num {} peek {:?}", num, digits.peek());
    println!("next_if_eq: {:?}", digits.next_if_eq(&'a'));
    let total: f64 = c.iter().sum();
    let prod: f64 = c.iter().product();
    println!("float sum {} product {}", total, prod);
    let ints: i64 = (1..=20).product();
    println!("factorial 20: {}", ints);
    let words = ["apple", "fig", "banana", "kiwi"];
    println!("min_by_key len: {:?}", words.iter().min_by_key(|w| w.len()));
    println!("max_by_key len: {:?}", words.iter().max_by_key(|w| w.len()));
    let floats = [2.5, -1.0, 9.75, 3.0];
    let max = floats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let maxp = floats.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
    let minp = floats.iter().copied().min_by(|a, b| a.total_cmp(b));
    println!("max {} {:?} {:?}", max, maxp, minp);
    let (evens, odds): (Vec<i32>, Vec<i32>) = (1..10).partition(|x| x % 2 == 0);
    println!("partition {:?} {:?}", evens, odds);
    let (names, lens): (Vec<&str>, Vec<usize>) = words.iter().map(|w| (*w, w.len())).unzip();
    println!("unzip {:?} {:?}", names, lens);
    let sentence = words.iter().fold(String::new(), |mut acc, w| {
        if !acc.is_empty() { acc.push(' '); }
        acc.push_str(w);
        acc
    });
    println!("fold: {}", sentence);
    let checked: Option<i32> = [10, 20, 30].iter().try_fold(0i32, |acc, &x| acc.checked_add(x));
    let overflow: Option<i8> = [100i8, 27, 1].iter().try_fold(0i8, |acc, &x| acc.checked_add(x));
    println!("try_fold {:?} {:?}", checked, overflow);
    let res: Result<i32, String> = ["1", "2", "x"].iter().try_fold(0, |acc, s| s.parse::<i32>().map(|v| acc + v).map_err(|e| e.to_string()));
    println!("try_fold result {:?}", res);
    println!("last {:?} nth {:?} count {}", words.iter().last(), words.iter().nth(1), words.iter().filter(|w| w.len() > 3).count());
    println!("any {} all {}", words.iter().any(|w| w.starts_with('k')), words.iter().all(|w| w.len() >= 3));
    println!("find_map {:?}", ["a", "12", "b", "7"].iter().find_map(|s| s.parse::<i32>().ok()));
    println!("find {:?}", (1..100).find(|x| x * x > 50));
    println!("rev range {:?}", (0..5).rev().collect::<Vec<_>>());
    println!("chars range {:?}", ('x'..='z').collect::<Vec<char>>());
    let mut fused = vec![Some(1), None, Some(3)].into_iter().map_while(|x| x).fuse();
    println!("fuse {:?} {:?} {:?}", fused.next(), fused.next(), fused.next());
    let grouped = [1, 1, 2, 2, 2, 5, 1].iter().fold(Vec::<(i32, usize)>::new(), |mut acc, &x| {
        match acc.last_mut() {
            Some((v, n)) if *v == x => *n += 1,
            _ => acc.push((x, 1)),
        }
        acc
    });
    println!("group-by: {:?}", grouped);
    let mut sum_sq = 0;
    (1..=4).for_each(|x| sum_sq += x * x);
    println!("for_each sum_sq: {}", sum_sq);
    let pos = words.iter().position(|&w| w == "banana");
    println!("position: {:?}", pos);
    let inspected: Vec<i32> = (1..4).inspect(|x| print!("<{}>", x)).map(|x| x * 2).collect();
    println!(" inspect: {:?}", inspected);
    let s: Vec<(usize, char)> = "abc".chars().enumerate().skip(1).collect();
    println!("enumerate skip: {:?}", s);
    let mx = (1..=3).map(|x| x as f64 / 2.0).fold(0.0, |a: f64, b| a.max(b));
    println!("max float fold: {}", mx);
    println!("eq/lt: {} {}", (1..3).eq([1, 2].iter().copied()), [1, 2].iter().lt([1, 3].iter()));
    println!("sum of empty: {} {}", Vec::<f64>::new().iter().sum::<f64>(), std::iter::empty::<i32>().count());
    println!("successors: {:?}", std::iter::successors(Some(1u32), |x| if *x < 100 { Some(x * 3) } else { None }).collect::<Vec<_>>());
    println!("rev chars enumerate: {:?}", "xyz".chars().rev().enumerate().collect::<Vec<_>>());
    println!("zip unequal: {:?}", (1..10).zip("ab".chars()).collect::<Vec<_>>());
}
