// #[macro_export] with $crate, vis/lifetime/pat/block fragments, local macros, char and exclusive range patterns.
mod util {
    #[macro_export]
    macro_rules! square {
        ($x:expr) => { $crate::util::mul($x, $x) };
    }
    pub fn mul(a: i64, b: i64) -> i64 { a * b }
}

macro_rules! builder {
    ($vis:vis struct $name:ident { $($field:ident : $t:ty = $default:expr),* $(,)? }) => {
        #[derive(Debug)]
        $vis struct $name { $($field: $t),* }
        impl $name {
            $vis fn new() -> Self { $name { $($field: $default),* } }
            $(
                fn $field(mut self, value: $t) -> Self { self.$field = value; self }
            )*
        }
    };
}

builder!(pub struct Server { host: String = "localhost".to_string(), port: u16 = 8080, verbose: bool = false });

macro_rules! enum_with_count {
    ($name:ident: $($v:ident),+) => {
        #[derive(Debug, Clone, Copy)]
        enum $name { $($v),+ }
        impl $name {
            const COUNT: usize = [$(stringify!($v)),+].len();
        }
    };
}
enum_with_count!(Dir: North, East, South, West);

macro_rules! pairs {
    ($($a:expr => [$($b:expr),*]);* $(;)?) => {
        vec![$( $( ($a, $b) ),* ),*]
    };
}

macro_rules! op {
    (add $a:expr, $b:expr) => { $a + $b };
    (neg $a:expr) => { -$a };
    ($other:ident $($rest:tt)*) => { { let _ = stringify!($($rest)*); concat!("unknown op ", stringify!($other)) } };
}

macro_rules! maybe {
    ($e:expr) => { Some($e) };
    () => { None::<i32> };
}

macro_rules! lifetimes {
    ($lt:lifetime, $t:ty) => { fn first<$lt>(xs: &$lt [$t]) -> &$lt $t { &xs[0] } };
}
lifetimes!('a, u32);

macro_rules! pat_test {
    ($v:expr, $p:pat) => { if let $p = $v { true } else { false } };
}

macro_rules! block_twice {
    ($b:block) => { ($b, $b) };
}

fn classify(c: char) -> &'static str {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
        'a'..='z' => "consonant",
        'A'..='Z' => "upper",
        '0'..='9' => "digit",
        _ => "other",
    }
}

fn bucket(n: u32) -> &'static str {
    match n {
        0..10 => "single",
        10..100 => "double",
        _ => "many",
    }
}

fn main() {
    println!("{}", square!(7));
    let s = Server::new().port(9000).verbose(true);
    println!("{:?}", s);
    println!("{} {:?}", Dir::COUNT, [Dir::North, Dir::West]);
    println!("{:?}", pairs!(1 => [2, 3]; 4 => [5]; 6 => []));
    println!("{} {} {}", op!(add 1, 2), op!(neg 5), op!(mul 3 4));
    println!("{:?} {:?}", maybe!(3), maybe!());
    println!("{}", first(&[9, 8]));
    println!("{} {}", pat_test!(Some(3), Some(1..=5)), pat_test!(None::<i32>, Some(_)));
    let mut counter = 0;
    let pair = block_twice!({ counter += 1; counter * 10 });
    println!("{:?}", pair);
    for c in "aZk7!".chars() {
        print!("{}={} ", c, classify(c));
    }
    println!();
    for n in [3, 42, 512] {
        print!("{} ", bucket(n));
    }
    println!();
    let data = [(1, "x"), (2, "y")];
    for &(ref num, name) in data.iter() {
        print!("{}{} ", num, name);
    }
    println!();
    macro_rules! local { () => { "local macro" } }
    println!("{}", local!());
    let v: Vec<u8> = vec![];
    let r = match v.as_slice() { [] => "empty", [..] => "some" };
    println!("{r}");
    let text = String::from("hello");
    match text.as_str() {
        "hi" | "hello" => println!("greet"),
        s if s.starts_with('h') => println!("h-word"),
        _ => println!("?"),
    }
    let opt: Option<&(i32, i32)> = Some(&(1, 2));
    if let Some(&(x, y)) = opt { println!("{}", x + y); }
    let boxed: Box<(i32, String)> = Box::new((5, "five".into()));
    let (num, ref word) = *boxed;
    println!("{num} {word}");
}
