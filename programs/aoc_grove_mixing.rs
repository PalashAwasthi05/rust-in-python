// Day 20 2022: mixing a circular list of i64 by moving entries, with rem_euclid.
const INPUT: &str = r"1
2
-3
3
-2
0
4";

fn mix(values: &[i64], key: i64, rounds: usize) -> i64 {
    let n = values.len();
    let mut ring: Vec<(usize, i64)> = values.iter().map(|&v| v * key).enumerate().collect();
    for _ in 0..rounds {
        for original in 0..n {
            let pos = ring.iter().position(|&(i, _)| i == original).unwrap();
            let item = ring.remove(pos);
            let target = (pos as i64 + item.1).rem_euclid(n as i64 - 1) as usize;
            ring.insert(target, item);
        }
    }
    let zero = ring.iter().position(|&(_, v)| v == 0).unwrap();
    [1000, 2000, 3000].iter().map(|off| ring[(zero + off) % n].1).sum()
}

fn main() {
    let values: Vec<i64> = INPUT.lines().map(|l| l.parse().unwrap()).collect();
    println!("part 1: {}", mix(&values, 1, 1));
    println!("part 2: {}", mix(&values, 811589153, 10));
    let mut sorted = values.clone();
    sorted.sort_unstable();
    sorted.dedup();
    let (neg, pos): (Vec<i64>, Vec<i64>) = sorted.iter().partition(|&&v| v < 0);
    println!("{:?} {:?}", neg, pos);
    let running: Vec<i64> = values.iter().scan(0i64, |acc, &v| { *acc += v; Some(*acc) }).collect();
    println!("running {:?}", running);
    let big = values.iter().map(|&v| v as i128 * 811589153 * 811589153).max().unwrap();
    println!("big {big}");
}
