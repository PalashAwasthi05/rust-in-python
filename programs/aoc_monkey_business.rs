// Day 11 2022: monkeys with operations, modular arithmetic on u64.
#[derive(Debug, Clone)]
enum Op {
    Add(u64),
    Mul(u64),
    Square,
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<u64>,
    op: Op,
    test: u64,
    if_true: usize,
    if_false: usize,
    inspected: u64,
}

const INPUT: &str = r"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

fn last_num<T: std::str::FromStr>(line: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    line.split_whitespace().last().unwrap().parse().unwrap()
}

fn parse(block: &str) -> Monkey {
    let lines: Vec<&str> = block.lines().map(str::trim).collect();
    let items = lines[1]
        .trim_start_matches("Starting items: ")
        .split(", ")
        .map(|n| n.parse().unwrap())
        .collect();
    let expr: Vec<&str> = lines[2].split_whitespace().collect();
    let op = match (expr[4], expr[5]) {
        ("*", "old") => Op::Square,
        ("*", n) => Op::Mul(n.parse().unwrap()),
        ("+", n) => Op::Add(n.parse().unwrap()),
        other => panic!("unknown op {:?}", other),
    };
    Monkey {
        items,
        op,
        test: last_num(lines[3]),
        if_true: last_num(lines[4]),
        if_false: last_num(lines[5]),
        inspected: 0,
    }
}

fn run(mut monkeys: Vec<Monkey>, rounds: usize, relief: bool) -> u64 {
    let modulus: u64 = monkeys.iter().map(|m| m.test).product();
    for _ in 0..rounds {
        for i in 0..monkeys.len() {
            let items = std::mem::take(&mut monkeys[i].items);
            monkeys[i].inspected += items.len() as u64;
            for item in items {
                let mut w = match monkeys[i].op {
                    Op::Add(n) => item + n,
                    Op::Mul(n) => item * n,
                    Op::Square => item * item,
                };
                if relief {
                    w /= 3;
                } else {
                    w %= modulus;
                }
                let to = if w % monkeys[i].test == 0 { monkeys[i].if_true } else { monkeys[i].if_false };
                monkeys[to].items.push(w);
            }
        }
    }
    let mut counts: Vec<u64> = monkeys.iter().map(|m| m.inspected).collect();
    counts.sort_by(|a, b| b.cmp(a));
    println!("inspections: {:?}", counts);
    counts[0] * counts[1]
}

fn main() {
    let monkeys: Vec<Monkey> = INPUT.split("\n\n").map(parse).collect();
    println!("{:?}", monkeys[2].op);
    println!("part 1: {}", run(monkeys.clone(), 20, true));
    println!("part 2: {}", run(monkeys, 1000, false));
}
