// Chapter 19.5 and Rust by Example's macro_rules! section: declarative macros.
macro_rules! my_vec {
    () => {
        Vec::new()
    };
    ( $( $x:expr ),+ $(,)? ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )+
            temp_vec
        }
    };
}

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
        println!("{:?} = {:?}", stringify!($expression), $expression)
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
    ($x:expr, $($y:expr),+) => (
        std::cmp::min($x, find_min!($($y),+))
    )
}

macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::BTreeMap::new();
        $( map.insert($key, $value); )*
        map
    }};
}

macro_rules! point_struct {
    ($name:ident { $($field:ident),* }) => {
        #[derive(Debug, Default)]
        struct $name {
            $($field: i32),*
        }
        impl $name {
            fn sum(&self) -> i32 {
                0 $(+ self.$field)*
            }
        }
    };
}

point_struct!(P3 { x, y, z });

macro_rules! calculate {
    (eval $e:expr) => {
        {
            let val: usize = $e;
            println!("{} = {}", stringify!{$e}, val);
        }
    };
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

macro_rules! impl_shape {
    ($($t:ident => $area:expr),* $(,)?) => {
        $(
            struct $t(f64);
            impl $t {
                fn area(&self) -> f64 {
                    let s = self.0;
                    $area(s)
                }
            }
        )*
    };
}

impl_shape! {
    Square => |s: f64| s * s,
    Circle => |r: f64| 3.0 * r * r,
}

macro_rules! maybe_add {
    ($a:expr $(, $b:expr)?) => {
        $a $(+ $b)?
    };
}

macro_rules! block_with_label {
    ($label:literal, $body:block) => {{
        print!("[{}] ", $label);
        $body
    }};
}

fn main() {
    let v: Vec<u32> = my_vec![1, 2, 3];
    let empty: Vec<i32> = my_vec![];
    let trailing = my_vec!["a", "b",];
    println!("{:?} {:?} {:?}", v, empty, trailing);

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

    println!("{}", find_min!(1));
    println!("{}", find_min!(1 + 2, 2));
    println!("{}", find_min!(5, 2 * 3, 4));

    let ages = hashmap! {
        "ann" => 31,
        "bob" => 27,
    };
    println!("{:?}", ages);

    let p = P3 { x: 1, y: 2, z: 3 };
    println!("{:?} sum {} default {:?}", p, p.sum(), P3::default());

    calculate! {
        eval 1 + 2
    }
    calculate! {
        eval (1 + 2) * (3 / 4),
        eval 7 * 6,
        eval 2 + 3
    }

    println!("count = {}", count!(a b c d));
    println!("count = {}", count!());
    println!("{} {}", Square(2.0).area(), Circle(1.5).area());
    println!("{} {}", maybe_add!(1), maybe_add!(1, 2));
    let result = block_with_label!("calc", { 6 * 7 });
    println!("{result}");
}
