// Day 6 2021: lanternfish population with a rotating [u64; 9] array, plus Day 11 2024 stones.
use std::collections::HashMap;

const FISH: &str = "3,4,3,1,2";
const STONES: &str = "125 17";

fn fish_after(days: usize) -> u64 {
    let mut timers = [0u64; 9];
    for t in FISH.trim().split(',').map(|s| s.parse::<usize>().unwrap()) {
        timers[t] += 1;
    }
    for _ in 0..days {
        timers.rotate_left(1);
        timers[6] += timers[8];
    }
    timers.iter().sum()
}

fn blink(stones: &HashMap<u64, u64>) -> HashMap<u64, u64> {
    let mut next = HashMap::with_capacity(stones.len());
    for (&stone, &count) in stones {
        if stone == 0 {
            *next.entry(1).or_default() += count;
            continue;
        }
        let digits = stone.ilog10() + 1;
        if digits % 2 == 0 {
            let div = 10u64.pow(digits / 2);
            *next.entry(stone / div).or_default() += count;
            *next.entry(stone % div).or_default() += count;
        } else {
            *next.entry(stone * 2024).or_default() += count;
        }
    }
    next
}

fn main() {
    println!("fish after 18: {}", fish_after(18));
    println!("fish after 80: {}", fish_after(80));
    println!("fish after 256: {}", fish_after(256));
    let mut stones: HashMap<u64, u64> = STONES.split(' ').map(|s| (s.parse().unwrap(), 1)).collect();
    for i in 1..=75 {
        stones = blink(&stones);
        if i == 6 || i == 25 || i == 75 {
            println!("stones after {i}: {} ({} distinct)", stones.values().sum::<u64>(), stones.len());
        }
    }
    let largest = stones.keys().max().unwrap();
    println!("largest stone: {largest}, bits {}", 64 - largest.leading_zeros());
}
