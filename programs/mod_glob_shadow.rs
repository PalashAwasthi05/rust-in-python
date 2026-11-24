// Glob imports, explicit imports shadowing globs, and local items shadowing imports.
mod a {
    pub fn name() -> &'static str {
        "a::name"
    }
    pub fn only_a() -> &'static str {
        "only_a"
    }
    pub const VALUE: i32 = 1;
    pub struct Thing;
    impl Thing {
        pub fn id(&self) -> &'static str {
            "a::Thing"
        }
    }
}

mod b {
    pub fn name() -> &'static str {
        "b::name"
    }
    pub const VALUE: i32 = 2;
}

mod c {
    use super::a::*;
    use super::b::name;

    pub fn report() -> String {
        format!("{} {} {}", name(), only_a(), VALUE)
    }

    pub mod d {
        use super::super::a::*;

        fn name() -> &'static str {
            "d::name"
        }

        pub fn report() -> String {
            format!("{} {}", name(), Thing.id())
        }
    }
}

mod e {
    pub use crate::a::*;
    pub fn extra() -> i32 {
        VALUE + 100
    }
}

use a::*;

fn main() {
    println!("{}", name());
    println!("{}", c::report());
    println!("{}", c::d::report());
    println!("{} {}", e::extra(), e::only_a());
    println!("{}", e::Thing.id());
    let value_local = VALUE * 10;
    println!("{}", value_local);
    fn name() -> &'static str {
        "main::name"
    }
    println!("{}", name());
    println!("{}", b::name());
}
