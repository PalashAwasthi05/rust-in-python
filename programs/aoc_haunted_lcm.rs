// Day 8 2023: walking a node network, part 2 needs lcm of cycle lengths.
use std::collections::HashMap;

const INPUT: &str = r"LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
33A = (33B, XXX)
33B = (33C, 33C)
33C = (33D, 33D)
33D = (33E, 33E)
33E = (33Z, 33Z)
33Z = (33B, 33B)
XXX = (XXX, XXX)";

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

fn steps<'a>(
    start: &'a str,
    path: &[u8],
    net: &HashMap<&'a str, (&'a str, &'a str)>,
    done: impl Fn(&str) -> bool,
) -> u64 {
    let mut node = start;
    let mut count = 0u64;
    for &turn in path.iter().cycle() {
        if done(node) {
            break;
        }
        let (l, r) = net[node];
        node = if turn == b'L' { l } else { r };
        count += 1;
    }
    count
}

fn main() {
    let mut lines = INPUT.lines();
    let path = lines.next().unwrap().as_bytes();
    let net: HashMap<&str, (&str, &str)> = lines
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (name, rest) = l.split_once(" = ").unwrap();
            let rest = rest.trim_matches(|c| c == '(' || c == ')');
            let (a, b) = rest.split_once(", ").unwrap();
            (name, (a, b))
        })
        .collect();
    let mut starts: Vec<&str> = net.keys().copied().filter(|k| k.ends_with('A')).collect();
    starts.sort();
    let mut total = 1u64;
    for s in &starts {
        let n = steps(s, path, &net, |n| n.ends_with('Z'));
        println!("{s}: {n}");
        total = lcm(total, n);
    }
    println!("part 2: {total}");
    let big: u64 = [17u64, 19, 23, 29, 31, 37, 41, 43].iter().fold(1, |acc, &x| lcm(acc, x * 2));
    println!("big lcm: {big}");
}
