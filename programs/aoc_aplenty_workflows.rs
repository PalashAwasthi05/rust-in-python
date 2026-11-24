// Day 19 2023: workflows of conditional rules, part 2 splits 4D ranges.
use std::collections::HashMap;

const INPUT: &str = r"px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

#[derive(Debug)]
enum Rule<'a> {
    Cond { field: usize, less: bool, value: u64, target: &'a str },
    Goto(&'a str),
}

type Ranges = [(u64, u64); 4];

fn field_index(c: char) -> usize {
    "xmas".find(c).expect("field")
}

fn parse_workflows(text: &str) -> HashMap<&str, Vec<Rule>> {
    text.lines()
        .map(|line| {
            let (name, body) = line.split_once('{').unwrap();
            let rules = body
                .trim_end_matches('}')
                .split(',')
                .map(|r| match r.split_once(':') {
                    Some((cond, target)) => {
                        let mut chars = cond.chars();
                        let field = field_index(chars.next().unwrap());
                        let less = chars.next() == Some('<');
                        Rule::Cond { field, less, value: cond[2..].parse().unwrap(), target }
                    }
                    None => Rule::Goto(r),
                })
                .collect();
            (name, rules)
        })
        .collect()
}

fn accepted(flows: &HashMap<&str, Vec<Rule>>, part: &[u64; 4]) -> bool {
    let mut at = "in";
    loop {
        match at {
            "A" => return true,
            "R" => return false,
            _ => {}
        }
        at = flows[at]
            .iter()
            .find_map(|rule| match *rule {
                Rule::Cond { field, less, value, target } => {
                    let v = part[field];
                    if (less && v < value) || (!less && v > value) { Some(target) } else { None }
                }
                Rule::Goto(t) => Some(t),
            })
            .unwrap();
    }
}

fn combos(flows: &HashMap<&str, Vec<Rule>>, at: &str, mut r: Ranges) -> u64 {
    match at {
        "A" => return r.iter().map(|&(lo, hi)| hi - lo + 1).product(),
        "R" => return 0,
        _ => {}
    }
    let mut total = 0;
    for rule in &flows[at] {
        match rule {
            Rule::Goto(t) => {
                total += combos(flows, t, r);
                break;
            }
            Rule::Cond { field, less, value, target } => {
                let (lo, hi) = r[*field];
                let (pass, fail) = if *less {
                    ((lo, hi.min(value - 1)), (lo.max(*value), hi))
                } else {
                    ((lo.max(value + 1), hi), (lo, hi.min(*value)))
                };
                if pass.0 <= pass.1 {
                    let mut sub = r;
                    sub[*field] = pass;
                    total += combos(flows, target, sub);
                }
                if fail.0 > fail.1 {
                    break;
                }
                r[*field] = fail;
            }
        }
    }
    total
}

fn main() {
    let (flows_text, parts_text) = INPUT.split_once("\n\n").unwrap();
    let flows = parse_workflows(flows_text);
    println!("{:?}", flows["in"]);
    let parts: Vec<[u64; 4]> = parts_text
        .lines()
        .map(|l| {
            let mut p = [0; 4];
            for kv in l.trim_matches(|c| c == '{' || c == '}').split(',') {
                let (k, v) = kv.split_once('=').unwrap();
                p[field_index(k.chars().next().unwrap())] = v.parse().unwrap();
            }
            p
        })
        .collect();
    let part1: u64 = parts.iter().filter(|p| accepted(&flows, p)).map(|p| p.iter().sum::<u64>()).sum();
    println!("part 1: {part1}");
    println!("part 2: {}", combos(&flows, "in", [(1, 4000); 4]));
}
