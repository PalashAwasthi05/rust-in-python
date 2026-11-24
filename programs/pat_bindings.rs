// @ bindings with ranges, nested or-patterns, guards, ref / ref mut.
#[derive(Debug)]
enum Shape {
    Circle { r: u32 },
    Rect { w: u32, h: u32 },
    Tri(u32, u32, u32),
}

fn classify(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        d @ 1..=9 if d % 2 == 0 => "even digit",
        1..=9 => "odd digit",
        10 | 20 | 30 => "round",
        _ => "big",
    }
}

fn shape_info(s: &Shape) -> String {
    match s {
        Shape::Circle { r: radius @ 0..=2 } => format!("small circle r={}", radius),
        Shape::Circle { r } => format!("circle r={}", r),
        Shape::Rect { w, h } if w == h => format!("square {}", w),
        Shape::Rect { w: 1, h } | Shape::Rect { w: h, h: 1 } => format!("line of {}", h),
        Shape::Rect { w, .. } => format!("rect w={}", w),
        Shape::Tri(a, b, c) if a == b && b == c => format!("equilateral {}", a),
        Shape::Tri(a, b, _) | Shape::Tri(a, _, b) if a == b => format!("isoceles {}", a),
        Shape::Tri(..) => "scalene".to_string(),
    }
}

fn tuple_or(p: (Option<i32>, Result<i32, String>)) -> i32 {
    match p {
        (Some(1 | 2 | 3), Ok(n @ (10 | 20))) => n * 100,
        (Some(x), Ok(_)) | (Some(x), Err(_)) if x > 100 => x,
        (Some(x), Ok(y)) => x + y,
        (None, Ok(y)) => y,
        (_, Err(ref e)) => e.len() as i32,
    }
}

fn main() {
    for n in [-5, 0, 4, 7, 20, 99] {
        println!("{} is {}", n, classify(n));
    }
    let shapes = [
        Shape::Circle { r: 1 },
        Shape::Circle { r: 5 },
        Shape::Rect { w: 3, h: 3 },
        Shape::Rect { w: 1, h: 8 },
        Shape::Rect { w: 6, h: 1 },
        Shape::Rect { w: 4, h: 2 },
        Shape::Tri(2, 2, 2),
        Shape::Tri(3, 4, 3),
        Shape::Tri(3, 4, 5),
    ];
    for s in shapes.iter() {
        println!("{:?}: {}", s, shape_info(s));
    }
    println!("{}", tuple_or((Some(2), Ok(20))));
    println!("{}", tuple_or((Some(200), Err("x".into()))));
    println!("{}", tuple_or((Some(5), Ok(6))));
    println!("{}", tuple_or((None, Ok(9))));
    println!("{}", tuple_or((None, Err("oops".into()))));

    let mut pair = (String::from("left"), 5);
    {
        let (ref mut name, ref mut count) = pair;
        name.push_str("-side");
        *count += 10;
    }
    println!("{:?}", pair);
    let mut opt = Some(3);
    if let Some(ref mut x) = opt {
        *x *= 7;
    }
    println!("{:?}", opt);
    let word = "hello";
    let msg @ ("hi" | "hello") = word else { panic!() };
    println!("greeting {}", msg);
    let big = 1_000u32;
    let size = match big {
        n @ 0..=99 => format!("small {}", n),
        n @ 100..=9_999 => format!("medium {}", n),
        n => format!("large {}", n),
    };
    println!("{}", size);
}
