fn fizzbuzz(n: u32) -> String {
    if n % 15 == 0 {
        "fizzbuzz".to_string()
    } else if n % 3 == 0 {
        "fizz".to_string()
    } else if n % 5 == 0 {
        "buzz".to_string()
    } else {
        n.to_string()
    }
}

fn main() {
    let n = 5;
    if n < 0 {
        print!("{} is negative", n);
    } else if n > 0 {
        print!("{} is positive", n);
    } else {
        print!("{} is zero", n);
    }
    let big_n = if n < 10 && n > -10 {
        println!(", and is a small number, increase ten-fold");
        10 * n
    } else {
        println!(", and is a big number, halve the number");
        n / 2
    };
    println!("{} -> {}", n, big_n);

    // loop with break and continue
    let mut count = 0u32;
    println!("counting until 5, skipping 3:");
    loop {
        count += 1;
        if count == 3 { continue; }
        print!("{} ", count);
        if count == 5 { println!(); break; }
    }

    // nested labeled loops
    'outer: loop {
        println!("entered the outer loop");
        #[allow(unused_labels)]
        'inner: loop {
            println!("entered the inner loop");
            break 'outer;
        }
    }
    println!("exited the outer loop");

    let mut grid_hits = Vec::new();
    'rows: for r in 0..5 {
        for c in 0..5 {
            if c > r { continue 'rows; }
            if r * c == 6 { grid_hits.push((r, c)); }
            if r + c == 7 { break 'rows; }
        }
    }
    println!("grid hits {:?}", grid_hits);

    // returning from loops
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 { break counter * 2; }
    };
    println!("loop result {}", result);
    let (mut a, mut b) = (1071u32, 462u32);
    let gcd = loop {
        if b == 0 { break a; }
        (a, b) = (b, a % b);
    };
    println!("gcd(1071, 462) = {}", gcd);

    // while
    let mut n = 1;
    let mut line = Vec::new();
    while n < 16 {
        line.push(fizzbuzz(n));
        n += 1;
    }
    println!("{}", line.join(" "));
    let mut x = 27u64;
    let mut steps = 0;
    while x != 1 {
        x = if x % 2 == 0 { x / 2 } else { 3 * x + 1 };
        steps += 1;
    }
    println!("collatz(27) takes {} steps", steps);
    let mut stack = vec![1, 2, 3];
    while !stack.is_empty() {
        let top = stack.pop().unwrap();
        print!("popped {} ", top);
    }
    println!();
}
