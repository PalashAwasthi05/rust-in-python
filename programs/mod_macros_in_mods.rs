// macro_rules in modules: #[macro_use], $crate paths, and macro re-exports by use.
#[macro_use]
mod macros {
    macro_rules! square {
        ($x:expr) => {
            $x * $x
        };
    }

    macro_rules! greet {
        ($name:expr) => {
            $crate::util::greeting($name)
        };
    }
}

mod util {
    pub fn greeting(name: &str) -> String {
        format!("hello, {}", name)
    }

    macro_rules! triple {
        ($x:expr) => {
            3 * $x
        };
    }
    pub(crate) use triple;
}

mod user {
    pub fn area(side: i32) -> i32 {
        square!(side)
    }

    pub fn tripled(x: i32) -> i32 {
        crate::util::triple!(x)
    }
}

#[macro_export]
macro_rules! make_point {
    ($x:expr, $y:expr) => {
        $crate::geo::Point { x: $x, y: $y }
    };
}

mod geo {
    #[derive(Debug)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    pub fn origin() -> Point {
        make_point!(0, 0)
    }
}

use util::triple;

fn main() {
    println!("{}", square!(7));
    println!("{}", greet!("modules"));
    println!("{} {}", user::area(5), user::tripled(4));
    println!("{}", triple!(10));
    println!("{:?} {:?}", make_point!(1, 2), geo::origin());
    println!("{:?}", crate::make_point!(3, 4));
}
