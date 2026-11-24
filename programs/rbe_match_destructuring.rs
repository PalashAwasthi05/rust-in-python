#[allow(dead_code)]
enum Color {
    Red,
    Blue,
    Green,
    RGB(u32, u32, u32),
    HSV(u32, u32, u32),
    CMYK(u32, u32, u32, u32),
}

struct Foo { x: (u32, u32), y: u32 }

struct Bar { name: String, tags: Vec<&'static str> }

fn describe_color(color: Color) -> String {
    match color {
        Color::Red => "the color is Red!".to_string(),
        Color::Blue => "the color is Blue!".to_string(),
        Color::Green => "the color is Green!".to_string(),
        Color::RGB(r, g, b) => format!("red {}, green {}, and blue {}", r, g, b),
        Color::HSV(h, s, v) => format!("hue {}, saturation {}, value {}", h, s, v),
        Color::CMYK(c, m, y, k) => format!("cyan {}, magenta {}, yellow {}, key {}", c, m, y, k),
    }
}

fn main() {
    // tuples
    for triple in [(0, -2, 3), (3, -2, 0), (1, 5, 2), (2, 1, 2)] {
        let s = match triple {
            (0, y, z) => format!("first is 0, y = {}, z = {}", y, z),
            (1, ..) => "first is 1 and the rest doesn't matter".to_string(),
            (.., 2) => "last is 2 and the rest doesn't matter".to_string(),
            _ => "it doesn't matter what they are".to_string(),
        };
        println!("{:?}: {}", triple, s);
    }

    // arrays and slices
    let arrays: [[i32; 3]; 4] = [[0, 2, 3], [1, -7, 9], [-1, 2, 4], [5, 5, 5]];
    for array in arrays {
        let s = match array {
            [0, second, third] => format!("0, {}, {}", second, third),
            [1, _, third] => format!("1, _, {}", third),
            [-1, second, ..] => format!("-1, {}, ...", second),
            [first, middle @ ..] => format!("{} then {:?}", first, middle),
        };
        println!("{:?} -> {}", array, s);
    }
    let dynamic: &[i32] = &[3, 4, 5, 6, 7];
    match dynamic {
        [first, middle @ .., last] => println!("first {} middle {:?} last {}", first, middle, last),
        [single] => println!("just {}", single),
        [] => println!("empty"),
    }

    // enums
    println!("{}", describe_color(Color::RGB(122, 17, 40)));
    println!("{}", describe_color(Color::CMYK(1, 2, 3, 4)));
    println!("{}", describe_color(Color::Green));

    // pointers and references
    let reference = &4;
    match reference {
        &val => println!("got a value via destructuring: {:?}", val),
    }
    match *reference {
        val => println!("got a value via dereferencing: {:?}", val),
    }
    let ref is_a_reference = 3;
    let value = 5;
    let mut mut_value = 6;
    match value {
        ref r => println!("got a reference to a value: {:?}", r),
    }
    match mut_value {
        ref mut m => {
            *m += 10;
            println!("we added 10. `mut_value`: {:?}", m);
        }
    }
    println!("is_a_reference {} mut_value now {}", *is_a_reference, mut_value);
    let boxed = Box::new((7, "seven"));
    let (n, word) = *boxed;
    println!("box destructured {} {}", n, word);

    // structs
    let foo = Foo { x: (1, 2), y: 3 };
    match foo {
        Foo { x: (1, b), y } => println!("first of x is 1, b = {}, y = {}", b, y),
        Foo { y: 2, x: i } => println!("y is 2, i = {:?}", i),
        Foo { y, .. } => println!("y = {}, we don't care about x", y),
    }
    let Foo { x: (a, b), y } = Foo { x: (9, 8), y: 7 };
    println!("let-destructured {} {} {}", a, b, y);
    let bar = Bar { name: "release".into(), tags: vec!["stable", "lts"] };
    let Bar { ref name, tags: ref t } = bar;
    println!("name {} has {} tags, first {:?}", name, t.len(), t.first());
    let pairs = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let joined: String = pairs.iter().map(|&(n, c)| format!("{}{}", c, n)).collect();
    println!("closure param destructuring: {}", joined);
}
