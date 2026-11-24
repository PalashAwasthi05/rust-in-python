// Chapter 7 (and chapter 14's re-exports): packages, modules, paths, use, pub, and privacy.
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> String {
            String::from("added to waitlist")
        }

        pub fn seat_at_table() -> String {
            format!("seated ({})", super::serving::take_order_hint())
        }
    }

    pub mod serving {
        pub fn take_order_hint() -> &'static str {
            "order soon"
        }

        pub fn serve_order() -> &'static str {
            "served"
        }

        #[allow(dead_code)]
        fn take_payment() {}
    }
}

fn deliver_order() -> &'static str {
    "delivered"
}

mod back_of_house {
    #[derive(Debug)]
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }

        pub fn fruit(&self) -> &str {
            &self.seasonal_fruit
        }
    }

    #[derive(Debug)]
    pub enum Appetizer {
        Soup,
        Salad,
    }

    pub fn fix_incorrect_order() -> String {
        let cooked = cook_order();
        format!("{cooked}, then {}", super::deliver_order())
    }

    fn cook_order() -> &'static str {
        "cooked"
    }

    pub mod kitchen {
        pub(crate) const OVEN_TEMP: u32 = 220;

        pub fn status() -> String {
            format!("oven at {} via {}", OVEN_TEMP, self::helpers::name())
        }

        mod helpers {
            pub fn name() -> &'static str {
                "helpers"
            }
        }
    }
}

pub use crate::front_of_house::hosting;

mod art {
    pub use self::kinds::PrimaryColor;
    pub use self::kinds::SecondaryColor;
    pub use self::utils::mix;

    pub mod kinds {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum PrimaryColor {
            Red,
            Yellow,
            Blue,
        }

        #[derive(Debug, PartialEq)]
        pub enum SecondaryColor {
            Orange,
            Green,
            Purple,
        }
    }

    pub mod utils {
        use crate::art::kinds::*;

        pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> Option<SecondaryColor> {
            use PrimaryColor::*;
            match (c1, c2) {
                (Red, Yellow) | (Yellow, Red) => Some(SecondaryColor::Orange),
                (Yellow, Blue) | (Blue, Yellow) => Some(SecondaryColor::Green),
                (Red, Blue) | (Blue, Red) => Some(SecondaryColor::Purple),
                _ => None,
            }
        }
    }
}

use art::mix;
use art::PrimaryColor;
use std::collections::HashMap;
use std::fmt::Result as FmtResult;
use std::fmt::Write as _;
use std::{cmp::Ordering, collections::BTreeSet};

pub fn eat_at_restaurant() {
    println!("{}", crate::front_of_house::hosting::add_to_waitlist());
    println!("{}", front_of_house::hosting::add_to_waitlist());
    println!("{}", hosting::seat_at_table());
    println!("{}", front_of_house::serving::serve_order());

    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = String::from("Wheat");
    println!("I'd like {} toast please, with {}", meal.toast, meal.fruit());
    println!("{:?}", meal);

    let order1 = back_of_house::Appetizer::Soup;
    let order2 = back_of_house::Appetizer::Salad;
    println!("{:?} and {:?}", order1, order2);
    println!("{}", back_of_house::fix_incorrect_order());
    println!("{}", back_of_house::kitchen::status());
    println!("oven const {}", back_of_house::kitchen::OVEN_TEMP);
}

fn formatted() -> FmtResult {
    let mut s = String::new();
    write!(s, "{}-{}", 1, 2)?;
    println!("written: {s}");
    Ok(())
}

fn main() {
    eat_at_restaurant();

    let mut map = HashMap::new();
    map.insert(1, 2);
    println!("map {:?}", map);
    let set: BTreeSet<_> = [3, 1, 2].into_iter().collect();
    println!("set {:?} cmp {:?}", set, 1.cmp(&2) == Ordering::Less);
    formatted().unwrap();

    let red = PrimaryColor::Red;
    let yellow = art::kinds::PrimaryColor::Yellow;
    println!("{:?}", mix(red, yellow));
    println!("{:?}", art::mix(PrimaryColor::Blue, PrimaryColor::Yellow));
    println!("{:?}", art::utils::mix(red, red));
    println!("{}", mix(PrimaryColor::Blue, red) == Some(art::SecondaryColor::Purple));
}
