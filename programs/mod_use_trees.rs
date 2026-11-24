// Nested brace imports, renames, self in use lists, and glob imports.
mod shapes {
    pub mod round {
        pub fn circle_area(r: f64) -> f64 {
            3.0 * r * r
        }
        pub fn ring_area(outer: f64, inner: f64) -> f64 {
            circle_area(outer) - circle_area(inner)
        }
    }

    pub mod flat {
        pub fn square(s: i32) -> i32 {
            s * s
        }
        pub fn rect(w: i32, h: i32) -> i32 {
            w * h
        }
        pub mod tri {
            pub fn right(a: i32, b: i32) -> i32 {
                a * b / 2
            }
            pub fn equal(a: i32) -> i32 {
                a * a / 2
            }
        }
    }

    pub mod consts {
        pub const SIDES_TRI: u8 = 3;
        pub const SIDES_SQUARE: u8 = 4;
        pub const SIDES_HEX: u8 = 6;
    }
}

use shapes::{
    consts::*,
    flat::{self, rect, tri::{equal as iso, right}},
    round::{self as circles, ring_area},
};

fn main() {
    println!("square {}", flat::square(4));
    println!("rect {}", rect(3, 5));
    println!("right {}", right(4, 6));
    println!("iso {}", iso(6));
    println!("circle {}", circles::circle_area(2.0));
    println!("ring {}", ring_area(3.0, 1.0));
    println!("sides {} {} {}", SIDES_TRI, SIDES_SQUARE, SIDES_HEX);
    {
        use shapes::flat::tri;
        println!("inner scope {}", tri::right(2, 2));
    }
    use std::collections::{HashMap, BTreeMap as Ordered};
    let mut m: Ordered<&str, i32> = Ordered::new();
    m.insert("b", 2);
    m.insert("a", 1);
    let h: HashMap<i32, i32> = HashMap::new();
    println!("{:?} {}", m, h.len());
}
