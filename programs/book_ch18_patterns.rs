// Chapter 18: every place patterns appear and all the pattern syntax.
struct Point {
    x: i32,
    y: i32,
}

struct Point3 {
    x: i32,
    y: i32,
    z: i32,
}

enum Color {
    Rgb(i32, i32, i32),
    Hsv(i32, i32, i32),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color),
    Hello { id: i32 },
}

fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("Current location: ({x}, {y})");
}

fn foo(_: i32, y: i32) {
    println!("This code only uses the y parameter: {y}");
}

fn describe(msg: &Message) -> String {
    match msg {
        Message::Quit => String::from("The Quit variant has no data to destructure."),
        Message::Move { x, y: 0 } => format!("Move horizontally to x={x}"),
        Message::Move { x, y } => format!("Move in the x direction {x} and in the y direction {y}"),
        Message::Write(text) => format!("Text message: {text}"),
        Message::ChangeColor(Color::Rgb(r, g, b)) => format!("Change color to red {r}, green {g}, and blue {b}"),
        Message::ChangeColor(Color::Hsv(h, s, v)) => format!("Change color to hue {h}, saturation {s}, value {v}"),
        Message::Hello { id: id_variable @ 3..=7 } => format!("Found an id in range: {id_variable}"),
        Message::Hello { id: 10..=12 } => String::from("Found an id in another range"),
        Message::Hello { id } => format!("Found some other id: {id}"),
    }
}

fn classify(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1 | 2 => "one or two",
        3..=9 => "single digit",
        _ => "large",
    }
}

fn slice_shape(v: &[i32]) -> String {
    match v {
        [] => String::from("empty"),
        [one] => format!("one element {one}"),
        [first, .., last] if first == last => format!("bookends of {first}"),
        [first, second, rest @ ..] => format!("starts {first},{second} then {} more", rest.len()),
    }
}

fn main() {
    let favorite_color: Option<&str> = None;
    let is_tuesday = false;
    let age: Result<u8, _> = "34".parse();
    if let Some(color) = favorite_color {
        println!("Using your favorite color, {color}, as the background");
    } else if is_tuesday {
        println!("Tuesday is green day!");
    } else if let Ok(age) = age {
        if age > 30 {
            println!("Using purple as the background color");
        } else {
            println!("Using orange as the background color");
        }
    } else {
        println!("Using blue as the background color");
    }

    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{top}");
    }

    let v = vec!['a', 'b', 'c'];
    for (index, value) in v.iter().enumerate() {
        println!("{value} is at index {index}");
    }

    let (x, y, z) = (1, 2, 3);
    println!("{x} {y} {z}");
    let point = (3, 5);
    print_coordinates(&point);

    let Some(first_char) = "hello".chars().next() else {
        panic!("empty");
    };
    println!("first char {first_char}");

    let x = 1;
    match x {
        1 => println!("one"),
        2 => println!("two"),
        3 => println!("three"),
        _ => println!("anything"),
    }

    let x = Some(5);
    let y = 10;
    match x {
        Some(50) => println!("Got 50"),
        Some(y) => println!("Matched, y = {y}"),
        _ => println!("Default case, x = {x:?}"),
    }
    println!("at the end: x = {x:?}, y = {y}");

    let x = 5;
    match x {
        1..=5 => println!("one through five"),
        _ => println!("something else"),
    }
    let x = 'c';
    match x {
        'a'..='j' => println!("early ASCII letter"),
        'k'..='z' => println!("late ASCII letter"),
        _ => println!("something else"),
    }
    for n in [-7, 0, 2, 5, 99] {
        println!("{n} is {}", classify(n));
    }

    let p = Point { x: 0, y: 7 };
    let Point { x: a, y: b } = p;
    assert_eq!(0, a);
    assert_eq!(7, b);
    let Point { x, y } = p;
    println!("destructured {x} {y}");
    for p in [Point { x: 0, y: 7 }, Point { x: 3, y: 0 }, Point { x: 1, y: 1 }] {
        match p {
            Point { x, y: 0 } => println!("On the x axis at {x}"),
            Point { x: 0, y } => println!("On the y axis at {y}"),
            Point { x, y } => println!("On neither axis: ({x}, {y})"),
        }
    }

    let msgs = [
        Message::Quit,
        Message::Move { x: 4, y: 0 },
        Message::Move { x: 1, y: 2 },
        Message::Write(String::from("hi")),
        Message::ChangeColor(Color::Hsv(0, 160, 255)),
        Message::ChangeColor(Color::Rgb(1, 2, 3)),
        Message::Hello { id: 5 },
        Message::Hello { id: 11 },
        Message::Hello { id: 42 },
    ];
    for m in &msgs {
        println!("{}", describe(m));
    }

    let ((feet, inches), Point { x, y }) = ((3, 10), Point { x: 3, y: -10 });
    println!("{feet}' {inches}\" at ({x}, {y})");

    foo(3, 4);
    let mut setting_value = Some(5);
    let new_setting_value = Some(10);
    match (setting_value, new_setting_value) {
        (Some(_), Some(_)) => {
            println!("Can't overwrite an existing customized value");
        }
        _ => {
            setting_value = new_setting_value;
        }
    }
    println!("setting is {setting_value:?}");

    let numbers = (2, 4, 8, 16, 32);
    match numbers {
        (first, _, third, _, fifth) => println!("Some numbers: {first}, {third}, {fifth}"),
    }
    let _x = 5;
    let s = Some(String::from("Hello!"));
    if let Some(_) = s {
        println!("found a string");
    }
    println!("{s:?}");

    let origin = Point3 { x: 0, y: 0, z: 0 };
    match origin {
        Point3 { x, .. } => println!("x is {x}"),
    }
    println!("y, z = {}, {}", origin.y, origin.z);
    match numbers {
        (first, .., last) => println!("Some numbers: {first}, {last}"),
    }

    let num = Some(4);
    for candidate in [num, Some(5), None] {
        match candidate {
            Some(x) if x % 2 == 0 => println!("The number {x} is even"),
            Some(x) => println!("The number {x} is odd"),
            None => println!("no number"),
        }
    }
    let x = 4;
    let y = false;
    match x {
        4 | 5 | 6 if y => println!("yes"),
        _ => println!("no"),
    }

    let robot_name = Some(String::from("Bors"));
    match &robot_name {
        Some(name) => println!("robot named {name}"),
        None => (),
    }
    if let Some(ref name) = robot_name {
        println!("ref binding {name}");
    }
    let mut maybe = Some(String::from("mutable"));
    if let Some(ref mut inner) = maybe {
        inner.push_str("!!");
    }
    println!("{maybe:?}");

    for v in [&[][..], &[7], &[1, 2, 1], &[1, 2, 3, 4]] {
        println!("{}", slice_shape(v));
    }
    let arr = [10, 20, 30];
    let [head, tail @ ..] = arr;
    println!("head {head} tail {:?}", tail);
    let pair = (5, -5);
    let msg = match pair {
        (x, y) if x + y == 0 => "opposites",
        (x, _) if x % 2 == 1 => "odd first",
        _ => "no correlation",
    };
    println!("{msg}");
    let n = 15u32;
    let size = match n {
        small @ 0..=9 => format!("small {small}"),
        mid @ (10..=19 | 30..=39) => format!("teen-ish {mid}"),
        other => format!("other {other}"),
    };
    println!("{size}");
    let reference = &4;
    match reference {
        &val => println!("Got a value via destructuring: {val}"),
    }
    match *reference {
        val => println!("Got a value via dereferencing: {val}"),
    }
    let tuples = vec![(1, 'a'), (2, 'b')];
    let letters: Vec<char> = tuples.iter().map(|&(_, c)| c).collect();
    println!("{letters:?}");
    println!("{}", matches!(Some(3), Some(1..=5)));
}
