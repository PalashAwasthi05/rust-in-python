fn classify(n: i32) -> &'static str {
    // the last expression of a block is its value
    if n < 0 { "negative" } else if n == 0 { "zero" } else { "positive" }
}

fn area(w: f64, h: f64) -> f64 {
    let a = w * h;
    a
}

fn nothing(x: i32) {
    let _ = x + 1;
}

fn main() {
    let x = 5u32;
    let y = {
        let x_squared = x * x;
        let x_cube = x_squared * x;
        x_cube + x_squared + x
    };
    let z = {
        2 * x;
    };
    println!("x is {:?}", x);
    println!("y is {:?}", y);
    println!("z is {:?}", z);
    println!("nothing returns {:?}", nothing(3));

    // statements vs expressions
    let mut counter = 0;
    counter += 1;
    let assign_result = { counter += 1; };
    println!("counter {} assignment evaluates to {:?}", counter, assign_result);

    // if, match and loop are expressions
    for n in [-4, 0, 9] {
        println!("{} is {}", n, classify(n));
    }
    let parity = match y % 2 { 0 => "even", _ => "odd" };
    println!("y is {}", parity);
    let mut i = 1;
    let first_square_over_50 = loop {
        if i * i > 50 { break i * i; }
        i += 1;
    };
    println!("first square over 50: {}", first_square_over_50);

    // nested blocks produce values
    let description = {
        let width = 3.0;
        let height = {
            let base = 2.0;
            base * 1.5
        };
        format!("{}x{} = {}", width, height, area(width, height))
    };
    println!("{}", description);

    // blocks can contain items and early-exit via labeled break
    let verdict = 'check: {
        let readings = [12, 15, 99, 14];
        for (idx, r) in readings.iter().enumerate() {
            if *r > 50 {
                break 'check format!("anomaly {} at index {}", r, idx);
            }
        }
        String::from("all readings normal")
    };
    println!("{}", verdict);

    // expressions in arguments, tuple and array positions
    let pair = (if x > 3 { "big" } else { "small" }, { let t = x + 1; t * 2 });
    let arr = [x, { x + 1 }, match x { 5 => 50, _ => 0 }];
    println!("pair {:?} arr {:?}", pair, arr);
    let total: u32 = arr.iter().map(|v| { let doubled = v * 2; doubled + 1 }).sum();
    println!("total {}", total);
    let unit = if x > 100 { println!("never") };
    println!("if without else is unit: {:?}", unit);
    let s = { let mut s = String::new(); for c in ['a', 'b', 'c'] { s.push(c); s.push(c.to_ascii_uppercase()); } s };
    println!("built in a block: {}", s);
}
