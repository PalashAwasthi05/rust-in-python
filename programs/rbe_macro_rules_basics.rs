macro_rules! say_hello {
    () => {
        println!("Hello!")
    };
}

macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name));
        }
    };
}

create_function!(foo);
create_function!(bar);

macro_rules! print_result {
    ($expression:expr) => {
        println!("{:?} = {:?}", stringify!($expression), $expression);
    };
}

macro_rules! test {
    ($left:expr; and $right:expr) => {
        println!("{:?} and {:?} is {:?}", stringify!($left), stringify!($right), $left && $right)
    };
    ($left:expr; or $right:expr) => {
        println!("{:?} or {:?} is {:?}", stringify!($left), stringify!($right), $left || $right)
    };
}

macro_rules! find_min {
    ($x:expr) => ($x);
    ($x:expr, $($y:expr),+) => (std::cmp::min($x, find_min!($($y),+)));
}

macro_rules! make_struct {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Default)]
        struct $name { $($field: $ty),* }
        impl $name {
            fn field_names() -> Vec<&'static str> { vec![$(stringify!($field)),*] }
        }
    };
}

make_struct!(Config { host: String, port: u16, verbose: bool, });

macro_rules! hashmap {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m = std::collections::BTreeMap::new();
        $( m.insert($k, $v); )*
        m
    }};
}

macro_rules! square_all {
    ($($x:literal),*) => { [$($x * $x),*] };
}

macro_rules! type_name {
    ($t:ty) => { std::any::type_name::<$t>() };
}

macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

macro_rules! with_block {
    ($b:block) => {{ let v = $b; v * 2 }};
}

macro_rules! match_pat {
    ($v:expr, $p:pat) => { matches!($v, $p) };
}

fn main() {
    say_hello!();
    foo();
    bar();
    print_result!(1u32 + 1);
    print_result!({
        let x = 1u32;
        x * x + 2 * x - 1
    });
    test!(1i32 + 1 == 2i32; and 2i32 * 2 == 4i32);
    test!(true; or false);
    println!("min of 1 = {}", find_min!(1));
    println!("min of 1 + 2, 2 = {}", find_min!(1 + 2, 2));
    println!("min of 5, 2 * 3, 4 = {}", find_min!(5, 2 * 3, 4));
    let c = Config { host: "localhost".into(), port: 8080, ..Default::default() };
    println!("{:?} fields {:?}", c, Config::field_names());
    let m = hashmap! { "one" => 1, "two" => 2, "three" => 3, };
    println!("{:?}", m);
    println!("squares {:?}", square_all!(1, 2, 3, 4));
    println!("type names: {} {}", type_name!(Vec<Option<u8>>), type_name!(&str));
    println!("token count: {}", count!(a b c d e));
    println!("block doubled: {}", with_block!({ let a = 4; a + 1 }));
    println!("pattern match: {} {}", match_pat!(Some(3), Some(1..=5)), match_pat!('z', 'a'..='m'));
}
