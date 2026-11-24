// Chapter 4: ownership, references and borrowing, slices.
fn takes_ownership(some_string: String) {
    println!("took ownership of {some_string}");
}

fn makes_copy(some_integer: i32) {
    println!("copied {some_integer}");
}

fn gives_ownership() -> String {
    let some_string = String::from("yours");
    some_string
}

fn takes_and_gives_back(a_string: String) -> String {
    a_string
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn calculate_length_tuple(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)
}

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

fn first_word_index(s: &String) -> usize {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return i;
        }
    }
    s.len()
}

fn second_word(s: &str) -> Option<&str> {
    let mut words = s.split_whitespace();
    words.next();
    words.next()
}

fn sum_slice(values: &[i32]) -> i32 {
    let mut total = 0;
    for v in values {
        total += v;
    }
    total
}

fn main() {
    {
        let s = "hello";
        println!("{s} is valid in this scope");
    }

    let mut s = String::from("hello");
    s.push_str(", world!");
    println!("{s}");

    let x = 5;
    let y = x;
    println!("x = {x}, y = {y}");

    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("s1 = {s1}, s2 = {s2}");

    let s = String::from("hello");
    takes_ownership(s);
    let x = 5;
    makes_copy(x);
    println!("x is still {x}");

    let s1 = gives_ownership();
    let s2 = String::from("hello");
    let s3 = takes_and_gives_back(s2);
    println!("{s1} {s3}");

    let s1 = String::from("hello");
    let (s2, len) = calculate_length_tuple(s1);
    println!("The length of '{s2}' is {len}.");

    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{s1}' is {len}.");

    let mut s = String::from("hello");
    change(&mut s);
    println!("{s}");

    let mut s = String::from("hello");
    {
        let r1 = &mut s;
        r1.push('!');
    }
    let r2 = &mut s;
    r2.push('?');
    println!("{s}");

    let mut s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{r1} and {r2}");
    let r3 = &mut s;
    r3.make_ascii_uppercase();
    println!("{r3}");

    let s = String::from("hello world");
    let word = first_word_index(&s);
    println!("first word ends at {word}");
    let hello = &s[0..5];
    let world = &s[6..11];
    println!("[{hello}] [{world}]");
    let slice = &s[..2];
    let slice2 = &s[3..];
    let whole = &s[..];
    println!("{slice} {slice2} {whole}");

    let my_string = String::from("hello world");
    let word = first_word(&my_string[0..6]);
    println!("{word}");
    let word = first_word(&my_string[..]);
    println!("{word}");
    let word = first_word(&my_string);
    println!("{word}");
    let my_string_literal = "hello world";
    let word = first_word(&my_string_literal[0..6]);
    println!("{word}");
    let word = first_word(my_string_literal);
    println!("{word}");
    println!("{:?} {:?}", second_word("one two three"), second_word("solo"));

    let a = [1, 2, 3, 4, 5];
    let slice = &a[1..3];
    assert_eq!(slice, &[2, 3]);
    println!("slice = {:?}, sum = {}, whole sum = {}", slice, sum_slice(slice), sum_slice(&a));
    let v = vec![10, 20, 30];
    println!("vec slice sum = {}", sum_slice(&v[1..]));

    let mut owned = vec![String::from("a"), String::from("b")];
    let moved = std::mem::take(&mut owned[0]);
    println!("moved {moved:?}, left {:?}", owned);
}
