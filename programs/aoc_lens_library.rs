// Day 15 2023: HASH algorithm with wrapping u8 math, boxes of labelled lenses.
const INPUT: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7\n";

fn hash(s: &str) -> u8 {
    s.bytes().fold(0u8, |h, b| h.wrapping_add(b).wrapping_mul(17))
}

#[derive(Debug)]
enum Step<'a> {
    Remove(&'a str),
    Insert(&'a str, u8),
}

fn parse(step: &str) -> Step {
    if let Some(label) = step.strip_suffix('-') {
        Step::Remove(label)
    } else {
        let (label, focal) = step.split_once('=').unwrap();
        Step::Insert(label, focal.parse().unwrap())
    }
}

fn main() {
    let steps: Vec<&str> = INPUT.trim_end().split(',').collect();
    let part1: u32 = steps.iter().map(|s| hash(s) as u32).sum();
    println!("HASH = {}", hash("HASH"));
    println!("part 1: {part1}");
    let mut boxes: Vec<Vec<(&str, u8)>> = (0..256).map(|_| Vec::new()).collect();
    for step in steps.iter().map(|s| parse(s)) {
        match step {
            Step::Remove(label) => {
                let b = &mut boxes[hash(label) as usize];
                b.retain(|(l, _)| *l != label);
            }
            Step::Insert(label, focal) => {
                let b = &mut boxes[hash(label) as usize];
                match b.iter_mut().find(|(l, _)| *l == label) {
                    Some(slot) => slot.1 = focal,
                    None => b.push((label, focal)),
                }
            }
        }
    }
    for (i, b) in boxes.iter().enumerate().filter(|(_, b)| !b.is_empty()) {
        let lenses: Vec<String> = b.iter().map(|(l, f)| format!("[{l} {f}]")).collect();
        println!("Box {}: {}", i, lenses.join(" "));
    }
    let power: usize = boxes
        .iter()
        .enumerate()
        .flat_map(|(bi, b)| b.iter().enumerate().map(move |(si, &(_, f))| (bi + 1) * (si + 1) * f as usize))
        .sum();
    println!("part 2: {power}");
    println!("{:?}", parse("ab=5"));
}
