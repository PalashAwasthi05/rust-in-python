fn main() {
    let temps = [12, 14, 13, 17, 21, 22, 19, 15, 16, 24];
    let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun", "Mon2", "Tue2", "Wed2"];

    for (day, t) in names.iter().zip(temps.iter()).filter(|(_, &t)| t >= 20) {
        println!("warm day: {} {}", day, t);
    }

    let rises: Vec<i32> = temps.windows(2).map(|w| w[1] - w[0]).collect();
    println!("deltas: {:?}", rises);
    let biggest_jump = temps.windows(2).enumerate().max_by_key(|(_, w)| w[1] - w[0]).unwrap();
    println!("biggest jump at index {}: {:?}", biggest_jump.0, biggest_jump.1);

    for (i, chunk) in temps.chunks(3).enumerate() {
        let avg = chunk.iter().sum::<i32>() as f64 / chunk.len() as f64;
        println!("chunk {} {:?} avg {:.2}", i, chunk, avg);
    }

    let (min, max) = temps.iter().fold((i32::MAX, i32::MIN), |(lo, hi), &t| (lo.min(t), hi.max(t)));
    println!("min {} max {}", min, max);

    let running: Vec<i32> = temps.iter().scan(0, |acc, &t| { *acc += t; Some(*acc) }).collect();
    println!("running totals: {:?}", running);

    let rising_start: Vec<_> = temps.iter().take_while(|&&t| t < 17).collect();
    println!("before first 17+: {:?}", rising_start);
    let after: Vec<_> = temps.iter().skip_while(|&&t| t < 20).copied().collect();
    println!("from first 20+: {:?}", after);

    let words = vec!["hello world", "rust is fun", "iterators"];
    let letters: Vec<String> = words.iter().flat_map(|s| s.split(' ')).map(|w| w.to_uppercase()).collect();
    println!("flat_map: {:?}", letters);

    let rev: String = "stressed".chars().rev().collect();
    println!("reversed: {}", rev);
    let countdown: Vec<u32> = (1..=5).rev().step_by(2).collect();
    println!("countdown: {:?}", countdown);

    let input = "aaabccddddde";
    let mut it = input.chars().peekable();
    let mut encoded = String::new();
    while let Some(c) = it.next() {
        let mut n = 1;
        while it.peek() == Some(&c) {
            it.next();
            n += 1;
        }
        encoded.push_str(&format!("{}{}", n, c));
    }
    println!("rle: {}", encoded);

    let evens_sq_sum: i32 = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).sum();
    let any_neg = temps.iter().any(|&t| t < 0);
    let all_pos = temps.iter().all(|&t| t > 0);
    let pos = temps.iter().position(|&t| t == 22);
    println!("sum even squares {} any_neg {} all_pos {} pos22 {:?}", evens_sq_sum, any_neg, all_pos, pos);
    let (even, odd): (Vec<i32>, Vec<i32>) = temps.iter().partition(|&&t| t % 2 == 0);
    println!("even {:?} odd {:?}", even, odd);
    let last_two: Vec<_> = temps.iter().rev().take(2).rev().collect();
    println!("last two {:?}, count>15 {}", last_two, temps.iter().filter(|&&t| t > 15).count());
}
