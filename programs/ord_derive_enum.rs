#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    Num(i64),
    Ident(String),
    Pair(i32, i32),
    Eof,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Explicit {
    A = 10,
    B = 2,
    C = 7,
}

fn main() {
    let mut ps = vec![Priority::High, Priority::Low, Priority::Critical, Priority::Medium, Priority::Low];
    ps.sort();
    println!("{:?}", ps);
    println!("High > Medium: {}", Priority::High > Priority::Medium);
    println!("Low >= Low: {}", Priority::Low >= Priority::Low);
    println!("max: {:?}", ps.iter().max().unwrap());
    println!("cmp: {:?}", Priority::Critical.cmp(&Priority::Low));

    let shapes = vec![
        Shape::Empty,
        Shape::Rect { w: 2.0, h: 1.0 },
        Shape::Circle(3.0),
        Shape::Rect { w: 1.0, h: 5.0 },
        Shape::Circle(1.5),
    ];
    let mut sorted = shapes.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for s in &sorted {
        println!("{:?}", s);
    }
    println!("circle < rect: {}", Shape::Circle(100.0) < Shape::Rect { w: 0.0, h: 0.0 });
    println!("rect cmp: {:?}", Shape::Rect { w: 1.0, h: 2.0 }.partial_cmp(&Shape::Rect { w: 1.0, h: 3.0 }));
    println!("nan: {:?}", Shape::Circle(f64::NAN).partial_cmp(&Shape::Circle(1.0)));
    println!("nan vs other variant: {:?}", Shape::Circle(f64::NAN).partial_cmp(&Shape::Empty));

    let mut tokens = vec![
        Token::Eof,
        Token::Pair(1, 2),
        Token::Ident("zeta".to_string()),
        Token::Num(42),
        Token::Ident("alpha".to_string()),
        Token::Num(-7),
        Token::Pair(1, -5),
        Token::Pair(0, 100),
    ];
    tokens.sort();
    for t in &tokens {
        println!("{:?}", t);
    }
    let biggest = tokens.iter().max().unwrap();
    println!("biggest: {:?}", biggest);
    println!("Num(5) vs Ident: {:?}", Token::Num(5).cmp(&Token::Ident(String::new())));
    println!("equal: {}", Token::Pair(3, 4) == Token::Pair(3, 4));

    let mut ex = vec![Explicit::A, Explicit::C, Explicit::B];
    ex.sort();
    println!("{:?}", ex);
    println!("A > B: {}", Explicit::A > Explicit::B);
    println!("as i32: {:?}", ex.iter().map(|e| *e as i32).collect::<Vec<_>>());

    let opt: Vec<Option<Priority>> = vec![Some(Priority::Low), None, Some(Priority::High)];
    let mut o2 = opt.clone();
    o2.sort();
    println!("{:?}", o2);
}
