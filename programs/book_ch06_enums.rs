// Chapter 6: enums, Option, match, if let, let else.
#[derive(Debug)]
enum IpAddrKind {
    V4,
    V6,
}

#[derive(Debug)]
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit"),
            Message::Move { x, y } => println!("Move to {x},{y}"),
            Message::Write(text) => println!("Write {text:?}"),
            Message::ChangeColor(r, g, b) => println!("Color #{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UsState {
    Alabama,
    Alaska,
    Delaware,
}

impl UsState {
    fn existed_in(&self, year: u16) -> bool {
        match self {
            UsState::Alabama => year >= 1819,
            UsState::Alaska => year >= 1959,
            UsState::Delaware => year >= 1787,
        }
    }
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}

fn value_in_cents(coin: &Coin) -> u8 {
    match coin {
        Coin::Penny => {
            println!("Lucky penny!");
            1
        }
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {state:?}!");
            25
        }
    }
}

fn describe_state_quarter(coin: Coin) -> Option<String> {
    let Coin::Quarter(state) = coin else {
        return None;
    };
    if state.existed_in(1900) {
        Some(format!("{state:?} is pretty old, for America!"))
    } else {
        Some(format!("{state:?} is relatively new."))
    }
}

fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

fn route(kind: IpAddrKind) -> &'static str {
    match kind {
        IpAddrKind::V4 => "routing v4",
        IpAddrKind::V6 => "routing v6",
    }
}

fn main() {
    let four = IpAddrKind::V4;
    let six = IpAddrKind::V6;
    println!("{:?} {:?}", four, six);
    println!("{} / {}", route(four), route(six));

    let home = IpAddr::V4(127, 0, 0, 1);
    let loopback = IpAddr::V6(String::from("::1"));
    println!("{:?} {:?}", home, loopback);
    if let IpAddr::V4(a, b, c, d) = home {
        println!("{a}.{b}.{c}.{d}");
    }

    let messages = vec![
        Message::Write(String::from("hello")),
        Message::Move { x: 3, y: -4 },
        Message::ChangeColor(255, 128, 0),
        Message::Quit,
    ];
    for m in &messages {
        m.call();
    }
    println!("{:?}", messages[1]);

    let some_number = Some(5);
    let some_char = Some('e');
    let absent_number: Option<i32> = None;
    println!("{:?} {:?} {:?}", some_number, some_char, absent_number);
    let x: i8 = 5;
    let y: Option<i8> = Some(5);
    let sum = x + y.unwrap_or(0);
    println!("sum = {sum}");

    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);
    println!("{:?} {:?}", six, none);

    let coins = [Coin::Penny, Coin::Nickel, Coin::Dime, Coin::Quarter(UsState::Alaska), Coin::Quarter(UsState::Alabama)];
    let mut total = 0u32;
    for c in &coins {
        total += value_in_cents(c) as u32;
    }
    println!("total = {total} cents");

    let mut count = 0;
    for coin in &coins {
        if let Coin::Quarter(state) = coin {
            println!("State quarter from {state:?}!");
        } else {
            count += 1;
        }
    }
    println!("non-quarters: {count}");

    for coin in [Coin::Quarter(UsState::Delaware), Coin::Quarter(UsState::Alaska), Coin::Dime] {
        match describe_state_quarter(coin) {
            Some(desc) => println!("{desc}"),
            None => println!("not a quarter"),
        }
    }

    let dice_roll = 9;
    for roll in [3, 7, dice_roll] {
        match roll {
            3 => println!("add fancy hat"),
            7 => println!("remove fancy hat"),
            other => println!("move player {other} spaces"),
        }
    }
    for roll in [3, 4] {
        match roll {
            3 => println!("add fancy hat"),
            _ => (),
        }
    }

    let config_max = Some(3u8);
    if let Some(max) = config_max {
        println!("The maximum is configured to be {max}");
    }
    let nothing: Option<u8> = None;
    if let Some(max) = nothing {
        println!("unexpected {max}");
    } else {
        println!("no maximum configured");
    }
    println!("{}", if let Some(n) = some_number { n * 10 } else { 0 });
    println!("{:?}", some_number.map(|n| n * 2).filter(|n| *n > 5).is_some());
}
