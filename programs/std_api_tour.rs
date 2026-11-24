// Arrays, conversions, Box<dyn Error> with ?, mem helpers, and collection odds and ends.
use std::collections::*;
use std::error::Error;
fn parse_kv(s: &str) -> Result<(String, i64), Box<dyn Error>> {
    let (k, v) = s.split_once(':').ok_or("missing colon")?;
    Ok((k.to_string(), v.trim().parse()?))
}
fn main() {
    let v = vec![10, 20, 30, 40];
    println!("{}", v.iter().len());
    println!("{:?}", v.iter().rev().enumerate().map(|(i, x)| i * x).collect::<Vec<_>>());
    let arr = [1, 2, 3];
    println!("{:?}", arr.map(|x| x * 2));
    let built: [usize; 4] = std::array::from_fn(|i| i * i);
    println!("{:?}", built);
    let mut w = vec![1, 2, 3, 4, 5, 6];
    w.retain_mut(|x| { *x *= 10; *x % 20 != 0 });
    println!("{:?}", w);
    let mut d = vec!["a", "A", "b", "B", "b"];
    d.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    println!("{:?}", d);
    let mut pairs = vec![("x", 2.5), ("y", 9.0), ("z", -1.0)];
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("{:?}", pairs);
    for x in w.iter_mut().rev().take(1) {
        *x += 1;
    }
    println!("{:?}", w);
    let mut grid = [[0u8; 3]; 2];
    grid[1][1] = 9;
    println!("{:?} {}", grid, grid.iter().flatten().count());
    println!("{:?}", parse_kv("a: 5"));
    println!("{:?}", parse_kv("a 5").map_err(|e| e.to_string()));
    println!("{:?}", parse_kv("a: x").map_err(|e| e.to_string()));
    let names = ["ann", "bob"];
    println!("{}", names.contains(&"bob"));
    println!("{:?}", names.iter().position(|&n| n == "bob"));
    let mut dq: VecDeque<_> = (1..=3).collect();
    dq.make_contiguous().reverse();
    println!("{:?}", dq);
    println!("{:?}", dq.iter().sum::<i32>());
    let mut heap = BinaryHeap::from(vec![2, 7, 1]);
    heap.extend([9, 0]);
    println!("{:?}", heap.into_sorted_vec());
    let mut counts = [0; 5];
    for x in [1, 3, 3, 4, 1, 1] {
        counts[x] += 1;
    }
    println!("{:?}", counts.iter().enumerate().max_by_key(|&(_, c)| c));
    let m: BTreeMap<char, usize> = "hello".chars().rev().enumerate().map(|(i, c)| (c, i)).collect();
    println!("{:?}", m);
    let s: HashSet<char> = "abc".chars().collect();
    let t: HashSet<char> = "bcd".chars().collect();
    let mut u: Vec<_> = s.intersection(&t).collect();
    u.sort();
    println!("{:?}", u);
    let total: f32 = [1.5f32, 2.25].iter().sum();
    println!("{}", total);
    let avg = [3, 4, 5].iter().map(|&x| x as f64).sum::<f64>() / 3.0;
    println!("{:.2}", avg);
    let maxf = [1.0f64, 3.5, 2.0].iter().cloned().fold(f64::MIN, f64::max);
    println!("{}", maxf);
    println!("{:?}", [1.0f64, f64::NAN, 2.0].iter().copied().filter(|x| !x.is_nan()).reduce(f64::min));
    println!("{:?}", (1..=5).rev().map(|x| x.to_string()).collect::<Vec<_>>().join(">"));
    println!("{:?}", (1..4).zip(4..).zip("abc".chars()).map(|((a, b), c)| format!("{}{}{}", a, b, c)).collect::<Vec<_>>());
    println!("{:?}", vec![3, 1, 2].into_iter().max_by_key(|&x| std::cmp::Reverse(x)));
    let opt: Option<&mut Vec<i32>> = None;
    println!("{:?}", opt.map(|v| v.len()));
    let mut maybe = Some(vec![1]);
    if let Some(v) = maybe.as_mut() { v.push(2); }
    println!("{:?}", maybe.as_deref());
    println!("{:?}", maybe.map_or(0, |v| v.len()));
    println!("{:?}", "a".parse::<i32>().is_err_and(|e| e.to_string().contains("digit")));
    println!("{:?}", u64::pow(2, 40));
    println!("{:?}", i32::abs(-4));
    println!("{:?}", f64::sqrt(16.0));
    println!("{:?}", f64::max(1.0, 2.0));
    println!("{:?}", u8::try_from('a').ok());
    println!("{:?}", char::from(65u8));
    println!("{:?}", u32::from('a'));
    println!("{:?}", ('a'..'e').rev().collect::<String>());
    println!("{:?}", ('a'..='e').map(|c| c.to_ascii_uppercase()).nth(2));
    println!("{:?}", (b'a'..=b'c').map(char::from).collect::<Vec<_>>());
    println!("{:?}", std::mem::swap(&mut 1, &mut 2));
    let mut a = 1;
    let mut b = 2;
    std::mem::swap(&mut a, &mut b);
    let r = std::mem::replace(&mut a, 7) + std::mem::take(&mut b);
    println!("{} {} {}", a, b, r);
    println!("{:?}", vec![1, 2, 3].iter().rev().skip_while(|&&x| x > 2).collect::<Vec<_>>());
    println!("{:?}", vec![vec![1, 2], vec![3]].iter().map(|v| v.len()).collect::<Vec<_>>());
    println!("{:?}", [[1, 2], [3, 4]].concat());
    println!("{:?}", ["ab", "cd"].join(""));
    println!("{:?}", vec![1u8; 3].iter().map(|&b| b as u32).product::<u32>());
}
