fn main() {
    // exclusive and inclusive ranges
    let squares: Vec<u32> = (1..6).map(|n| n * n).collect();
    let inclusive: Vec<u32> = (1..=6).collect();
    println!("1..6 squared {:?}, 1..=6 {:?}", squares, inclusive);

    // char ranges
    let letters: String = ('a'..='e').collect();
    println!("letters {}", letters);
    for c in 'v'..'z' {
        print!("{}{} ", c, c.to_ascii_uppercase());
    }
    println!();
    let greek: Vec<char> = ('α'..='ε').collect();
    println!("greek {:?}", greek);

    // reverse and step_by
    let countdown: Vec<i32> = (0..=10).rev().step_by(3).collect();
    println!("countdown {:?}", countdown);
    let odds: Vec<i32> = (1..20).step_by(4).collect();
    println!("step 4 {:?}", odds);
    for i in (1..4).rev() {
        print!("{}... ", i);
    }
    println!("liftoff");
    let alphabet_skip: String = ('a'..='z').step_by(5).collect();
    println!("every fifth letter {}", alphabet_skip);
    let neg: Vec<i8> = (-3..3).collect();
    println!("negative range {:?}, len {}", neg, (-3..3).len());
    println!("empty range count {}, contains {} {}", (5..5).count(), (1..10).contains(&9), (1..10).contains(&10));

    // iter, iter_mut, into_iter
    let names = vec!["Bob", "Frank", "Ferris"];
    for name in names.iter() {
        match name {
            &"Ferris" => println!("there is a rustacean among us!"),
            _ => println!("hello {}", name),
        }
    }
    println!("names still usable: {:?}", names);
    let mut scores = vec![70, 85, 92];
    for s in scores.iter_mut() {
        *s += 5;
    }
    println!("curved scores {:?}", scores);
    for name in names.into_iter() {
        let tag = match name { "Ferris" => "crab", _ => "human" };
        print!("{}:{} ", name, tag);
    }
    println!();

    // enumerate, nested loops, and accumulating
    let words = ["zero", "one", "two"];
    for (i, w) in words.iter().enumerate() {
        println!("{} => {}", i, w);
    }
    let mut table = String::new();
    for row in 1..=3 {
        for col in 1..=4 {
            table.push_str(&format!("{:>3}", row * col));
        }
        table.push('\n');
    }
    print!("{}", table);
    let mut sum = 0;
    for n in (1..=100).filter(|n| n % 3 == 0 || n % 5 == 0) {
        sum += n;
    }
    println!("sum of multiples of 3 or 5 up to 100: {}", sum);
    let arr = [[1, 2], [3, 4], [5, 6]];
    for (r, row) in arr.iter().enumerate().rev() {
        for &v in row.iter().rev() {
            print!("({},{}) ", r, v);
        }
    }
    println!();
    let bytes = "hey".bytes();
    for (i, b) in bytes.enumerate() {
        print!("{}={} ", i, b);
    }
    println!();
}
