use std::mem;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct Point { x: f64, y: f64 }

#[allow(dead_code)]
struct Rectangle { top_left: Point, bottom_right: Point }

fn origin() -> Point { Point { x: 0.0, y: 0.0 } }

fn boxed_origin() -> Box<Point> { Box::new(Point { x: 0.0, y: 0.0 }) }

fn main() {
    // Box, stack and heap
    let point: Point = origin();
    let rectangle: Rectangle = Rectangle { top_left: origin(), bottom_right: Point { x: 3.0, y: -4.0 } };
    let boxed_rectangle: Box<Rectangle> = Box::new(Rectangle { top_left: origin(), bottom_right: Point { x: 3.0, y: -4.0 } });
    let boxed_point: Box<Point> = Box::new(origin());
    let box_in_a_box: Box<Box<Point>> = Box::new(boxed_origin());
    println!("Point occupies {} bytes on the stack", mem::size_of_val(&point));
    println!("Rectangle occupies {} bytes on the stack", mem::size_of_val(&rectangle));
    println!("Boxed point occupies {} bytes on the stack", mem::size_of_val(&boxed_point));
    println!("Boxed rectangle occupies {} bytes on the stack", mem::size_of_val(&boxed_rectangle));
    println!("Boxed box occupies {} bytes on the stack", mem::size_of_val(&box_in_a_box));
    let unboxed_point: Point = *boxed_point;
    println!("Unboxed point occupies {} bytes on the stack, x={}", mem::size_of_val(&unboxed_point), unboxed_point.x);
    println!("bottom right via box: {:?}", boxed_rectangle.bottom_right);

    // Vec
    let collected_iterator: Vec<i32> = (0..10).collect();
    println!("collected (0..10) into: {:?}", collected_iterator);
    let mut xs = vec![1i32, 2, 3];
    println!("initial vector: {:?}", xs);
    xs.push(4);
    println!("vector: {:?}, length {}, second {}", xs, xs.len(), xs[1]);
    println!("pop last element: {:?}", xs.pop());
    xs.insert(0, 0);
    xs.extend([7, 8]);
    xs.retain(|&x| x != 2);
    println!("after insert/extend/retain: {:?}", xs);
    for (i, x) in xs.iter().enumerate() { print!("{}:{} ", i, x); }
    println!();
    for x in xs.iter_mut() { *x *= 3; }
    println!("updated vector: {:?}", xs);
    let removed = xs.remove(1);
    xs.truncate(3);
    println!("removed {}, truncated {:?}, capacity >= len: {}", removed, xs, xs.capacity() >= xs.len());
    let mut v2 = vec![5, 1, 4, 1, 5, 9];
    v2.sort_unstable();
    v2.dedup();
    println!("sorted+dedup {:?}, windows sums {:?}", v2, v2.windows(2).map(|w| w[0] + w[1]).collect::<Vec<_>>());
    let drained: Vec<i32> = v2.drain(1..3).collect();
    println!("drained {:?} left {:?}", drained, v2);

    // str and String
    let pangram: &'static str = "the quick brown fox jumps over the lazy dog";
    println!("pangram: {}", pangram);
    println!("words in reverse");
    for word in pangram.split_whitespace().rev() { print!("{} ", word); }
    println!();
    let mut chars: Vec<char> = pangram.chars().collect();
    chars.sort();
    chars.dedup();
    let mut string = String::new();
    for c in chars {
        string.push(c);
        string.push_str(", ");
    }
    let chars_to_trim: &[char] = &[' ', ','];
    let trimmed_str: &str = string.trim_matches(chars_to_trim);
    println!("used characters: {}", trimmed_str);
    let alice = String::from("I like dogs");
    let bob: String = alice.replace("dog", "cat");
    println!("alice says: {}", alice);
    println!("bob says: {}", bob);
    let byte_escape = "I'm writing \x52\x75\x73\x74!";
    let unicode_codepoint = "\u{211D}";
    let raw_str = r"Escapes don't work here: \x3F \u{211D}";
    let quotes = r#"And then I said: "There is no escape!""#;
    println!("{} / {} / {} / {}", byte_escape, unicode_codepoint, raw_str, quotes);
    let bytestring: &[u8; 21] = b"this is a byte string";
    println!("a byte string: {:?}", bytestring);
    println!("as str: {:?}", std::str::from_utf8(bytestring));
    println!("invalid utf8: {:?}", std::str::from_utf8(&[0xff, 0x61]).is_err());
    let mut s = String::with_capacity(8);
    s += "héllo";
    s.insert(0, '¡');
    println!("{} has {} bytes and {} chars, upper {}", s, s.len(), s.chars().count(), s.to_uppercase());
}
