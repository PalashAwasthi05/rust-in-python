use std::num::ParseIntError;

fn parse_all(items: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    items.iter().map(|s| s.parse::<i32>()).collect()
}

fn checked_sum(items: &[&str]) -> Result<i32, String> {
    items
        .iter()
        .map(|s| s.parse::<i32>().map_err(|e| format!("{:?}: {}", s, e)))
        .sum()
}

fn main() {
    println!("{:?}", parse_all(&["1", "2", "3"]));
    println!("{:?}", parse_all(&["1", "two", "3", "four"]));
    println!("{:?}", parse_all(&[]));
    println!("{:?}", checked_sum(&["10", "20"]));
    println!("{:?}", checked_sum(&["10", "2.5"]));

    let opts: Option<Vec<u32>> = ["7", "8", "9"].iter().map(|s| s.parse().ok()).collect();
    println!("{:?}", opts);
    let opts: Option<Vec<u32>> = ["7", "-8", "9"].iter().map(|s| s.parse().ok()).collect();
    println!("{:?}", opts);

    let (good, bad): (Vec<_>, Vec<_>) = ["1", "x", "3", "y"]
        .iter()
        .map(|s| s.parse::<i32>())
        .partition(|r| r.is_ok());
    let good: Vec<i32> = good.into_iter().map(Result::unwrap).collect();
    let bad: Vec<String> = bad.into_iter().map(|r| r.unwrap_err().to_string()).collect();
    println!("good={:?} bad={:?}", good, bad);

    let flat: Vec<i32> = ["4", "z", "6"].iter().flat_map(|s| s.parse::<i32>()).collect();
    println!("{:?}", flat);
    let filtered: Vec<i32> = ["4", "z", "6"].iter().filter_map(|s| s.parse().ok()).collect();
    println!("{:?}", filtered);

    let product: Option<i32> = [Some(2), Some(3), Some(4)].into_iter().product();
    println!("{:?}", product);
    let product: Option<i32> = [Some(2), None, Some(4)].into_iter().product();
    println!("{:?}", product);

    let mut count = 0;
    let result: Result<Vec<i32>, String> = (1..10)
        .map(|n| {
            count += 1;
            if n == 4 { Err(format!("stopped at {}", n)) } else { Ok(n * n) }
        })
        .collect();
    println!("{:?} after {} calls", result, count);
    let try_fold = (1..=5).try_fold(0i32, |acc, n| acc.checked_add(n * 100));
    println!("{:?}", try_fold);
    let try_fold = [i32::MAX, 1].iter().try_fold(0i32, |acc, &n| acc.checked_add(n));
    println!("{:?}", try_fold);
}
