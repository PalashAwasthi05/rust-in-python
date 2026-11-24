#[derive(Debug)]
enum Temperature { Celsius(i32), Fahrenheit(i32) }

#[derive(Debug)]
enum Msg { Hello { id: i32 }, Text(String), Quit }

fn age() -> u32 { 15 }

fn some_number() -> Option<u32> { Some(42) }

fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let Some((a, b)) = s.split_once(':') else {
        return None;
    };
    let (Ok(x), Ok(y)) = (a.trim().parse(), b.trim().parse()) else {
        return None;
    };
    Some((x, y))
}

fn main() {
    // guards
    for t in [Temperature::Celsius(35), Temperature::Celsius(20), Temperature::Fahrenheit(100), Temperature::Fahrenheit(40)] {
        match t {
            Temperature::Celsius(t) if t > 30 => println!("{}C is above 30 Celsius", t),
            Temperature::Celsius(t) => println!("{}C is equal to or below 30 Celsius", t),
            Temperature::Fahrenheit(t) if t > 86 => println!("{}F is above 86 Fahrenheit", t),
            Temperature::Fahrenheit(t) => println!("{}F is equal to or below 86 Fahrenheit", t),
        }
    }
    let number: u8 = 4;
    match number {
        i if i == 0 => println!("zero"),
        i if i > 0 => println!("greater than zero"),
        _ => unreachable!("u8 is never negative"),
    }

    // @ bindings
    match age() {
        0 => println!("not born yet"),
        n @ 1..=12 => println!("child of age {:?}", n),
        n @ 13..=19 => println!("teen of age {:?}", n),
        n => println!("adult of age {:?}", n),
    }
    match some_number() {
        Some(n @ 42) => println!("the answer: {}!", n),
        Some(n) => println!("not interesting... {}", n),
        _ => (),
    }
    for m in [Msg::Hello { id: 5 }, Msg::Hello { id: 11 }, Msg::Hello { id: 99 }, Msg::Text("hi".into()), Msg::Quit] {
        match m {
            Msg::Hello { id: id_var @ 3..=7 } => println!("found an id in range: {}", id_var),
            Msg::Hello { id: 10..=12 } => println!("found an id in another range"),
            Msg::Hello { id } => println!("some other id: {}", id),
            Msg::Text(ref s) if s.len() < 3 => println!("short text {:?}", s),
            other => println!("other message {:?}", other),
        }
    }

    // if let
    let letter: Option<i32> = None;
    let emoticon: Option<i32> = None;
    let i_like_letters = false;
    if let Some(i) = letter {
        println!("matched {:?}!", i);
    } else if i_like_letters {
        println!("didn't match a number. let's go with a letter!");
    } else {
        println!("I don't like letters. let's go with an emoticon :)! {:?}", emoticon);
    }
    let values = [Some(3), None, Some(8)];
    for v in values {
        if let Some(x @ 5..) = v {
            println!("big value {}", x);
        }
    }
    let total: i32 = values.iter().map(|v| if let Some(x) = v { *x } else { 0 }).sum();
    println!("if-let sum {}", total);

    // let-else
    for s in ["3:4", "10 : -2", "nope", "1:x"] {
        println!("{:?} -> {:?}", s, parse_pair(s));
    }

    // while let
    let mut optional = Some(0);
    while let Some(i) = optional {
        if i > 4 {
            println!("greater than 4, quit!");
            optional = None;
        } else {
            print!("`i` is `{:?}`. ", i);
            optional = Some(i + 1);
        }
    }
    let mut stack = vec!["a", "b", "c"];
    while let Some(top) = stack.pop() {
        print!("{} ", top);
    }
    println!();
    let mut it = (1..=10).filter(|n| n % 3 == 0);
    while let Some(n) = it.next() {
        print!("[{}]", n);
    }
    println!();
}
