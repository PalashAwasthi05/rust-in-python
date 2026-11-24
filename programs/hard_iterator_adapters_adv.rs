fn main() {
    let prices = [10.5, 11.2, 10.9, 12.4, 12.1, 13.0, 12.7];
    let days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let volumes = [300u32, 450, 200, 800, 650, 900, 400];

    for ((d, p), v) in days.iter().zip(prices.iter()).zip(volumes.iter()).filter(|(_, v)| **v >= 650) {
        println!("busy {} price {:.1} vol {}", d, p, v);
    }

    let best = prices.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let worst = prices.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    println!("best {} worst {}", best, worst);

    let (sum, count, max_vol) = volumes.iter().fold((0u32, 0usize, 0u32), |(s, c, m), &v| (s + v, c + 1, m.max(v)));
    println!("fold tuple: sum={} count={} max={} avg={:.1}", sum, count, max_vol, sum as f64 / count as f64);

    let changes: Vec<String> = prices.iter().scan(None, |prev: &mut Option<f64>, &p| {
        let out = prev.map(|q| format!("{:+.1}", p - q));
        *prev = Some(p);
        Some(out)
    }).flatten().collect();
    println!("changes: {:?}", changes);

    let every_other: Vec<&str> = days.iter().step_by(2).copied().collect();
    println!("step_by(2): {:?}", every_other);
    let weekend_first: Vec<&&str> = days[5..].iter().chain(days[..5].iter()).collect();
    println!("chain: {:?}", weekend_first);
    let rota: Vec<&str> = ["ann", "bo", "cy"].iter().cycle().take(7).copied().collect();
    println!("cycle().take(7): {:?}", rota);
    for (i, d) in days.iter().rev().enumerate().take(3) {
        println!("rev().enumerate(): {} {}", i, d);
    }
    let idx_rev: Vec<(usize, &&str)> = days.iter().enumerate().rev().take(2).collect();
    println!("enumerate().rev(): {:?}", idx_rev);

    let mut it = volumes.iter().peekable();
    let mut runs = Vec::new();
    while let Some(&v) = it.next() {
        let mut run = vec![v];
        while let Some(&&n) = it.peek() {
            if n > *run.last().unwrap() { run.push(n); it.next(); } else { break; }
        }
        runs.push(run);
    }
    println!("increasing runs: {:?}", runs);
    let mut tokens = "let x = 42 ;".split(' ').peekable();
    let mut out = vec![];
    while let Some(t) = tokens.next_if(|t| *t != "=") { out.push(t); }
    println!("next_if stopped before {:?}, got {:?}", tokens.peek(), out);

    let pairs: Vec<(char, u32)> = "abc".chars().zip(1..).collect();
    println!("zip infinite: {:?}", pairs);
    let (small, big): (Vec<u32>, Vec<u32>) = volumes.iter().partition(|&&v| v < 500);
    println!("partition: {:?} {:?}", small, big);
    let last_rise = prices.windows(2).rposition(|w| w[1] > w[0]);
    println!("last rise index: {:?}", last_rise);
    let total_cost: f64 = prices.iter().zip(volumes.iter()).map(|(p, &v)| p * v as f64).sum();
    println!("total turnover: {:.2}", total_cost);
    let flat: Vec<u32> = vec![vec![1, 2], vec![], vec![3]].into_iter().flatten().collect();
    println!("flatten: {:?}, product: {}", flat, (1..=6u64).product::<u64>());
}
