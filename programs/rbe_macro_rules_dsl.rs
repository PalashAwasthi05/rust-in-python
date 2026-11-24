use std::ops::{Add, Mul, Sub};

macro_rules! assert_equal_len {
    ($a:expr, $b:expr, $func:ident, $op:tt) => {
        assert!($a.len() == $b.len(), "{:?}: dimension mismatch: {:?} {:?} {:?}", stringify!($func), ($a.len(),), stringify!($op), ($b.len(),));
    };
}

macro_rules! op {
    ($func:ident, $bound:ident, $op:tt, $method:ident) => {
        fn $func<T: $bound<T, Output = T> + Copy>(xs: &mut Vec<T>, ys: &Vec<T>) {
            assert_equal_len!(xs, ys, $func, $op);
            for (x, y) in xs.iter_mut().zip(ys.iter()) {
                *x = $bound::$method(*x, *y);
            }
        }
    };
}

op!(add_assign, Add, +=, add);
op!(mul_assign, Mul, *=, mul);
op!(sub_assign, Sub, -=, sub);

// a tiny calculator DSL
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

// a declarative state machine DSL
macro_rules! state_machine {
    ($name:ident [$($state:ident),+] { $($from:ident --$event:ident--> $to:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum $name { $($state),+ }
        impl $name {
            fn on(self, event: &str) -> Option<$name> {
                $( if self == $name::$from && event == stringify!($event) { return Some($name::$to); } )*
                None
            }
        }
    };
}

state_machine!(Door [Closed, Opened, Locked] {
    Closed --open--> Opened,
    Opened --close--> Closed,
    Closed --lock--> Locked,
    Locked --unlock--> Closed,
});

macro_rules! html {
    ($tag:ident [ $($child:tt)* ]) => { format!("<{}>{}</{}>", stringify!($tag), html!(@children $($child)*), stringify!($tag)) };
    ($text:literal) => { $text.to_string() };
    (@children) => { String::new() };
    (@children $tag:ident [ $($inner:tt)* ] $($rest:tt)*) => { html!($tag [ $($inner)* ]) + &html!(@children $($rest)*) };
    (@children $text:literal $($rest:tt)*) => { $text.to_string() + &html!(@children $($rest)*) };
}

fn main() {
    let mut xs = vec![1, 2, 3];
    let ys = vec![10, 20, 30];
    add_assign(&mut xs, &ys);
    println!("add_assign: {:?}", xs);
    mul_assign(&mut xs, &vec![2, 2, 2]);
    println!("mul_assign: {:?}", xs);
    sub_assign(&mut xs, &ys);
    println!("sub_assign: {:?}", xs);
    let mut fs = vec![0.5f64, 1.5];
    mul_assign(&mut fs, &vec![4.0, 4.0]);
    println!("float mul_assign: {:?}", fs);

    calculate! {
        eval 1 + 2
    }
    calculate! {
        eval (1 + 2) * (3 / 4),
        eval 7 * 6,
        eval (1 << 10) - 24
    }

    let mut door = Door::Closed;
    for ev in ["open", "lock", "close", "lock", "unlock", "open"] {
        match door.on(ev) {
            Some(next) => { println!("{:?} --{}--> {:?}", door, ev, next); door = next; }
            None => println!("{:?} ignores {}", door, ev),
        }
    }

    let page = html!(div [ h1 [ "Title" ] p [ "Hello, " b [ "world" ] "!" ] ]);
    println!("{}", page);
}
