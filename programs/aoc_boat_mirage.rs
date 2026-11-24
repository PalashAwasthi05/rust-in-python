// Day 6 2023 (quadratic with f64) + Day 9 2023 (recursive extrapolation).
const RACES: &str = r"Time:      7  15   30
Distance:  9  40  200";

const HISTORY: &str = r"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

fn ways(time: u64, record: u64) -> u64 {
    // hold*(time-hold) > record  =>  roots of h^2 - t h + r = 0
    let (t, r) = (time as f64, record as f64);
    let disc = (t * t - 4.0 * r).sqrt();
    let lo = ((t - disc) / 2.0).floor() as u64 + 1;
    let hi = ((t + disc) / 2.0).ceil() as u64 - 1;
    hi - lo + 1
}

fn extrapolate(seq: &[i64]) -> (i64, i64) {
    if seq.iter().all(|&x| x == 0) {
        return (0, 0);
    }
    let diffs: Vec<i64> = seq.iter().zip(seq.iter().skip(1)).map(|(a, b)| b - a).collect();
    let (before, after) = extrapolate(&diffs);
    (seq[0] - before, seq[seq.len() - 1] + after)
}

fn main() {
    let rows: Vec<Vec<u64>> = RACES
        .lines()
        .map(|l| l.split_whitespace().skip(1).map(|n| n.parse().unwrap()).collect())
        .collect();
    let part1: u64 = rows[0].iter().zip(&rows[1]).map(|(&t, &d)| ways(t, d)).product();
    println!("part 1: {part1}");
    let joined: Vec<u64> = RACES
        .lines()
        .map(|l| l.split_once(':').unwrap().1.replace(' ', "").parse().unwrap())
        .collect();
    println!("part 2: {} ({} / {})", ways(joined[0], joined[1]), joined[0], joined[1]);
    let brute = (0..=30u64).filter(|h| h * (30 - h) > 200).count();
    println!("brute check: {brute}");

    let (mut back, mut fwd) = (0, 0);
    for line in HISTORY.lines() {
        let seq: Vec<i64> = line.split(' ').map(|n| n.parse().unwrap()).collect();
        let (b, f) = extrapolate(&seq);
        println!("{b:>4} {line} {f}");
        back += b;
        fwd += f;
    }
    println!("next sum {fwd}, previous sum {back}");
    let ratio = fwd as f64 / back.abs().max(1) as f64;
    println!("ratio {:.3} sqrt {:.5} hypot {}", ratio, ratio.sqrt(), 3f64.hypot(4.0));
}
