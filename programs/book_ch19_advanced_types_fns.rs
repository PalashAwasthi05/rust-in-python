// Chapter 19.3-19.4: type aliases, the never type, function pointers, returning closures,
// and a little unsafe Rust.
use std::fmt;

type Kilometers = i32;
type Thunk = Box<dyn Fn() -> String>;
type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}

fn parse_km(s: &str) -> Result<Kilometers> {
    s.trim().parse().map_err(|_| ParseError(s.to_string()))
}

fn takes_long_type(f: Thunk) -> String {
    f()
}

fn returns_long_type(name: &str) -> Thunk {
    let owned = name.to_string();
    Box::new(move || format!("hi {owned}"))
}

fn bar() -> ! {
    panic!("never returns");
}

fn first_even(values: &[i32]) -> i32 {
    for &v in values {
        if v % 2 == 0 {
            return v;
        }
    }
    -1
}

fn sum_guesses(guesses: &[&str]) -> u32 {
    let mut total = 0;
    for g in guesses {
        let guess: u32 = match g.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        total += guess;
    }
    total
}

fn add_one(x: i32) -> i32 {
    x + 1
}

fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

fn apply_all(fs: &[fn(i32) -> i32], x: i32) -> Vec<i32> {
    fs.iter().map(|f| f(x)).collect()
}

#[derive(Debug)]
enum Status {
    Value(u32),
    Stop,
}

fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}

fn returns_impl_closure(step: i32) -> impl Fn(i32) -> i32 {
    move |x| x + step
}

fn make_op(name: &str) -> Box<dyn Fn(i32, i32) -> i32> {
    match name {
        "add" => Box::new(|a, b| a + b),
        "mul" => Box::new(|a, b| a * b),
        _ => Box::new(|a, _| a),
    }
}

fn generic_size<T>(_t: &T) -> usize {
    std::mem::size_of::<T>()
}

static HELLO_WORLD: &str = "Hello, world!";
static mut COUNTER: u32 = 0;

fn add_to_count(inc: u32) {
    unsafe {
        COUNTER += inc;
    }
}

unsafe fn dangerous() -> &'static str {
    "called an unsafe fn"
}

fn split_at_mut_demo(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    values.split_at_mut(mid)
}

fn main() {
    let x: i32 = 5;
    let y: Kilometers = 5;
    println!("x + y = {}", x + y);
    println!("{:?} {:?}", parse_km(" 42 ").map(|k| k * 1000), parse_km("far").is_err());
    if let Err(e) = parse_km("??") {
        println!("{e}");
    }

    let f: Thunk = Box::new(|| String::from("hi"));
    println!("{}", takes_long_type(f));
    println!("{}", takes_long_type(returns_long_type("there")));

    println!("{}", sum_guesses(&["3", "x", " 4", "-1", "10"]));
    println!("{} {}", first_even(&[1, 3, 8, 5]), first_even(&[1]));
    let maybe: Option<i32> = None;
    let value = match maybe {
        Some(v) => v,
        None => -99,
    };
    println!("value {value}");
    let lazy_bar: fn() -> ! = bar;
    println!("bar is a fn pointer: {}", std::mem::size_of_val(&lazy_bar) > 0);

    let answer = do_twice(add_one, 5);
    println!("The answer is: {answer}");
    println!("{:?}", apply_all(&[add_one, |x| x * x, i32::abs], -3));

    let list_of_numbers = vec![1, 2, 3];
    let list_of_strings: Vec<String> = list_of_numbers.iter().map(|i| i.to_string()).collect();
    let list_of_strings2: Vec<String> = list_of_numbers.iter().map(ToString::to_string).collect();
    println!("{:?} {:?}", list_of_strings, list_of_strings2);
    let list_of_statuses: Vec<Status> = (0u32..4).map(Status::Value).collect();
    println!("{:?} {:?}", list_of_statuses, Status::Stop);
    let wrapped: Vec<Option<char>> = "ab".chars().map(Some).collect();
    println!("{:?}", wrapped);

    let handlers = vec![returns_closure(), Box::new(returns_impl_closure(10)), Box::new(|x| x * -1)];
    println!("{:?}", handlers.iter().map(|h| h(5)).collect::<Vec<_>>());
    for op in ["add", "mul", "other"] {
        println!("{op}(6, 7) = {}", make_op(op)(6, 7));
    }

    println!("sizes: {} {} {}", generic_size(&1u8), generic_size(&1u64), generic_size(&(1u32, 2u16)));
    println!("name is: {HELLO_WORLD}");
    add_to_count(3);
    add_to_count(4);
    unsafe {
        println!("COUNTER: {}", COUNTER);
        println!("{}", dangerous());
    }

    let mut v = vec![1, 2, 3, 4, 5, 6];
    let (a, b) = split_at_mut_demo(&mut v, 3);
    a[0] = 100;
    b[0] = 400;
    println!("{:?}", v);
}
