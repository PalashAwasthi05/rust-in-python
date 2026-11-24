// Chapter 3: variables, mutability, shadowing, data types, functions, control flow.
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;

fn plus_one(x: i32) -> i32 {
    x + 1
}

fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn fibonacci(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

fn main() {
    let mut x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");
    println!("Three hours is {} seconds", THREE_HOURS_IN_SECONDS);

    let y = 5;
    let y = y + 1;
    {
        let y = y * 2;
        println!("The value of y in the inner scope is: {y}");
    }
    println!("The value of y is: {y}");

    let spaces = "   ";
    let spaces = spaces.len();
    println!("spaces = {spaces}");

    let guess: u32 = "42".parse().expect("Not a number!");
    println!("guess = {guess}");

    // Scalar types
    let big = 98_222;
    let hex = 0xff;
    let octal = 0o77;
    let binary = 0b1111_0000;
    let byte = b'A';
    println!("{big} {hex} {octal} {binary} {byte}");
    let sum = 5 + 10;
    let difference = 95.5 - 4.3;
    let product = 4 * 30;
    let quotient = 56.7 / 32.2;
    let truncated = -5 / 3;
    let remainder = 43 % 5;
    println!("{sum} {difference} {product} {quotient} {truncated} {remainder}");
    let t = true;
    let f: bool = false;
    let c = 'z';
    let z: char = 'ℤ';
    let heart_eyed_cat = '😻';
    println!("{t} {f} {c} {z} {heart_eyed_cat}");

    // Compound types
    let tup: (i32, f64, u8) = (500, 6.4, 1);
    let (a, b, cc) = tup;
    println!("The value of b is: {b} (a={a}, c={cc})");
    println!("tup.0={} tup.1={} tup.2={}", tup.0, tup.1, tup.2);
    let unit = ();
    println!("unit = {:?}", unit);

    let months = ["January", "February", "March", "April", "May", "June", "July",
                  "August", "September", "October", "November", "December"];
    let first = months[0];
    let last = months[months.len() - 1];
    println!("{first} .. {last}");
    let threes = [3; 5];
    println!("{:?}", threes);
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let index = 3;
    println!("The value of the element at index {index} is: {}", arr[index]);

    // Functions and expressions
    print_labeled_measurement(5, 'h');
    let block_value = {
        let x = 3;
        x + 1
    };
    println!("The value of block_value is: {block_value}");
    println!("plus_one(5) = {}", plus_one(5));

    // Control flow
    let number = 6;
    if number % 4 == 0 {
        println!("number is divisible by 4");
    } else if number % 3 == 0 {
        println!("number is divisible by 3");
    } else if number % 2 == 0 {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4, 3, or 2");
    }
    let condition = true;
    let number = if condition { 5 } else { 6 };
    println!("The value of number is: {number}");

    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2;
        }
    };
    println!("The result is {result}");

    let mut count = 0;
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;
        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break;
            }
            if count == 2 {
                break 'counting_up;
            }
            remaining -= 1;
        }
        count += 1;
    }
    println!("End count = {count}");

    let mut number = 3;
    while number != 0 {
        println!("{number}!");
        number -= 1;
    }
    println!("LIFTOFF!!!");

    let a = [10, 20, 30, 40, 50];
    for element in a {
        println!("the value is: {element}");
    }
    for number in (1..4).rev() {
        println!("{number}!");
    }

    for f in [32.0, 98.6, 212.0, -40.0] {
        println!("{f}F = {:.1}C", fahrenheit_to_celsius(f));
    }
    let fibs: Vec<u64> = (0..15).map(fibonacci).collect();
    println!("fib: {:?}", fibs);

    let gifts = ["a partridge in a pear tree", "two turtle doves", "three French hens", "four calling birds"];
    let ordinals = ["first", "second", "third", "fourth"];
    for day in 0..gifts.len() {
        println!("On the {} day of Christmas my true love sent to me:", ordinals[day]);
        for g in (0..=day).rev() {
            if g == 0 && day > 0 {
                println!("  and {}", gifts[g]);
            } else {
                println!("  {}", gifts[g]);
            }
        }
    }
}
