// macro_rules with repetitions, multiple arms, trailing commas, and fragments.
macro_rules! max_of {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+ $(,)?) => {{
        let a = $x;
        let b = max_of!($($rest),+);
        if a > b { a } else { b }
    }};
}

macro_rules! hashmap {
    () => { std::collections::HashMap::new() };
    ($($k:expr => $v:expr),+ $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $( m.insert($k, $v); )+
        m
    }};
}

macro_rules! square_all {
    ($($x:literal)*) => { [$($x * $x),*] };
}

macro_rules! show {
    ($($e:expr),* $(,)?) => {
        $( println!("{} = {:?}", stringify!($e), $e); )*
    };
}

macro_rules! check {
    ($cond:expr => $msg:literal) => {
        if !$cond { println!("check failed: {}", $msg); } else { println!("ok: {}", stringify!($cond)); }
    };
}

macro_rules! my_vec {
    ($elem:expr; $n:expr) => { (0..$n).map(|_| $elem).collect::<Vec<_>>() };
    ($($x:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $( v.push($x); )*
        v
    }};
}

macro_rules! with_block {
    ($name:ident, $body:block) => {
        let $name = $body;
    };
}

macro_rules! typed_zero {
    ($t:ty) => { <$t>::default() };
}

fn main() {
    println!("{}", max_of!(3));
    println!("{}", max_of!(3, 9, 2, 7,));
    println!("{}", max_of!(1.5, -2.0));
    let m: std::collections::HashMap<&str, i32> = hashmap! { "a" => 1, "b" => 2, "c" => 3, };
    let mut keys: Vec<_> = m.iter().collect();
    keys.sort();
    println!("{:?}", keys);
    let empty: std::collections::HashMap<u8, u8> = hashmap!();
    println!("empty len {}", empty.len());
    println!("{:?}", square_all!(1 2 3 4));
    let v = vec![1, 2, 3];
    show!(v.len(), v.iter().sum::<i32>(), "lit", 1 + 2 * 3);
    check!(v.len() == 3 => "length");
    check!(v.is_empty() => "emptiness");
    println!("{:?} {:?}", my_vec![0u8; 4], my_vec!['a', 'b']);
    with_block!(answer, {
        let base = 6;
        base * 7
    });
    println!("answer {}", answer);
    let z: (i32, String, bool) = (typed_zero!(i32), typed_zero!(String), typed_zero!(bool));
    println!("{:?}", z);
    println!("{}", concat!("a", 1, 2.5, true, 'c'));
    println!("{}", stringify!(x + y * (z - 1)));
    let line = line!();
    println!("line is positive: {}", line > 0);
    let name = format_args!("{}-{}", 1, 2).to_string();
    println!("{}", name);
}
