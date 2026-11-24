mod my_mod {
    fn private_function() -> &'static str { "called `my_mod::private_function()`" }

    pub fn function() -> &'static str { "called `my_mod::function()`" }

    pub fn indirect_access() -> String {
        format!("called `my_mod::indirect_access()`, that\n> {}", private_function())
    }

    pub mod nested {
        pub fn function() -> &'static str { "called `my_mod::nested::function()`" }

        #[allow(dead_code)]
        fn private_function() -> &'static str { "called `my_mod::nested::private_function()`" }

        pub(in crate::my_mod) fn public_function_in_my_mod() -> &'static str {
            "called `my_mod::nested::public_function_in_my_mod()`"
        }

        pub(self) fn public_function_in_nested() -> &'static str {
            "called `my_mod::nested::public_function_in_nested()`"
        }

        pub(super) fn public_function_in_super_mod() -> &'static str {
            "called `my_mod::nested::public_function_in_super_mod()`"
        }

        pub fn uses_self() -> String {
            format!("{} / {}", self::public_function_in_nested(), super::function())
        }
    }

    pub fn call_public_function_in_my_mod() -> String {
        format!("{}\n> {}", nested::public_function_in_my_mod(), nested::public_function_in_super_mod())
    }

    pub(crate) fn public_function_in_crate() -> &'static str { "called `my_mod::public_function_in_crate()`" }
}

mod shapes {
    pub struct OpenBox<T> { pub contents: T }

    pub struct ClosedBox<T> { contents: T }

    impl<T: Copy> ClosedBox<T> {
        pub fn new(contents: T) -> ClosedBox<T> { ClosedBox { contents } }
        pub fn peek(&self) -> T { self.contents }
    }

    pub mod geometry {
        #[derive(Debug)]
        pub struct Square { pub side: f64, id: u32 }

        impl Square {
            pub fn new(side: f64) -> Self { Square { side, id: next_id() } }
            pub fn id(&self) -> u32 { self.id }
            pub fn area(&self) -> f64 { super::helpers::square(self.side) }
        }

        fn next_id() -> u32 { 7 }
    }

    mod helpers {
        pub fn square(x: f64) -> f64 { x * x }
    }
}

mod deeply {
    pub mod nested {
        pub fn function() -> &'static str { "called `deeply::nested::function()`" }
    }
}

use deeply::nested::function as other_function;
use shapes::geometry::{self, Square};

fn function() -> &'static str { "called `function()`" }

fn main() {
    println!("{}", function());
    println!("{}", my_mod::function());
    println!("{}", my_mod::indirect_access());
    println!("{}", my_mod::nested::function());
    println!("{}", my_mod::call_public_function_in_my_mod());
    println!("{}", my_mod::public_function_in_crate());
    println!("{}", my_mod::nested::uses_self());
    println!("{}", other_function());
    {
        use crate::deeply::nested::function;
        println!("shadowed in block: {}", function());
    }

    let open_box = shapes::OpenBox { contents: "public information" };
    println!("the open box contains: {}", open_box.contents);
    let closed_box = shapes::ClosedBox::new("classified information");
    println!("the closed box peeks: {}", closed_box.peek());
    let sq = Square::new(2.5);
    println!("square side {} id {} area {}", sq.side, sq.id(), sq.area());
    let sq2 = geometry::Square::new(1.0);
    println!("{:?}", sq2);
}
