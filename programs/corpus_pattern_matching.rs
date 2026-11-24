#[derive(Debug)]
enum Shape { Circle(f64), Rect { w: f64, h: f64 } }

#[derive(Debug)]
enum Command {
    Draw(Shape),
    Move { x: i32, y: i32 },
    Batch(Vec<Command>),
    Say(String),
    Quit,
}

fn describe(cmd: &Command) -> String {
    match cmd {
        Command::Draw(Shape::Circle(r)) if *r <= 0.0 => "invalid circle".to_string(),
        Command::Draw(Shape::Circle(r)) => format!("circle r={}", r),
        Command::Draw(Shape::Rect { w, h }) if w == h => format!("square side {}", w),
        Command::Draw(Shape::Rect { w, .. }) => format!("rect width {}", w),
        Command::Move { x: 0, y: 0 } => "stay put".into(),
        Command::Move { x, y: 0 } => format!("horizontal {}", x),
        Command::Move { x: dx @ -5..=5, y: dy @ -5..=5 } => format!("small step ({}, {})", dx, dy),
        Command::Move { x, y } => format!("jump ({}, {})", x, y),
        Command::Batch(cmds) if cmds.is_empty() => "empty batch".into(),
        Command::Batch(cmds) => format!("batch[{}]", cmds.iter().map(describe).collect::<Vec<_>>().join("; ")),
        Command::Say(s) if s.starts_with('!') => format!("shout {}", &s[1..]),
        Command::Say(s) => format!("say {:?}", s),
        Command::Quit => "quit".into(),
    }
}

fn classify(nums: &[i32]) -> String {
    match nums {
        [] => "empty".into(),
        [x] => format!("one: {}", x),
        [first, second] => format!("pair: {} {}", first, second),
        [first, .., last] if first == last => format!("bookends {}", first),
        [first, rest @ ..] if rest.iter().all(|r| r > first) => format!("min-first {} then {} more", first, rest.len()),
        [.., penult, _] => format!("long, penultimate {}", penult),
    }
}

fn age_group(age: u32) -> &'static str {
    match age {
        0 => "newborn",
        1..=12 => "child",
        13..=19 => "teen",
        n @ 20..=64 if n % 10 == 0 => "milestone adult",
        20..=64 => "adult",
        _ => "senior",
    }
}

fn main() {
    let cmds = vec![
        Command::Draw(Shape::Circle(2.0)),
        Command::Draw(Shape::Circle(-1.0)),
        Command::Draw(Shape::Rect { w: 3.0, h: 3.0 }),
        Command::Draw(Shape::Rect { w: 4.0, h: 1.0 }),
        Command::Move { x: 0, y: 0 },
        Command::Move { x: 9, y: 0 },
        Command::Move { x: 2, y: -3 },
        Command::Move { x: 20, y: 30 },
        Command::Batch(vec![Command::Say("hi".into()), Command::Quit]),
        Command::Batch(vec![]),
        Command::Say("!stop".into()),
    ];
    for c in &cmds { println!("{}", describe(c)); }
    for s in [&[][..], &[7], &[1, 2], &[4, 9, 4], &[1, 5, 3, 8], &[9, 1, 5, 3]] {
        println!("{:?} -> {}", s, classify(s));
    }
    for a in [0, 8, 15, 40, 41, 70] { println!("{} -> {}", a, age_group(a)); }
    let point = (3, -3);
    let desc = match point {
        (0, 0) => "origin",
        (x, y) if x == y => "diagonal",
        (x, y) if x == -y => "anti-diagonal",
        (_, _) => "elsewhere",
    };
    println!("{:?} is {}", point, desc);
    let opt: Option<Option<i32>> = Some(None);
    if let Some(None) = opt { println!("nested none"); }
    let msg = Some("hello");
    let Some(m) = msg else { panic!("nope") };
    println!("let-else got {}", m);
}
