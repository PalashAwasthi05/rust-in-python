// Nested inline module trees with self/super/crate paths.
mod outer {
    pub const LEVEL: u32 = 1;

    pub fn describe() -> String {
        format!("outer at level {}", LEVEL)
    }

    pub mod middle {
        pub const LEVEL: u32 = super::LEVEL + 1;

        pub fn describe() -> String {
            format!("middle at level {} under [{}]", self::LEVEL, super::describe())
        }

        pub mod inner {
            pub const LEVEL: u32 = super::LEVEL + 1;

            pub fn describe() -> String {
                format!("inner at level {} under [{}]", LEVEL, super::describe())
            }

            pub fn from_root() -> u32 {
                crate::outer::LEVEL + crate::outer::middle::LEVEL + self::LEVEL
            }

            pub fn grandparent() -> u32 {
                super::super::LEVEL
            }
        }

        fn private_helper() -> u32 {
            100
        }

        pub fn uses_private() -> u32 {
            private_helper() + inner::LEVEL
        }
    }
}

mod sibling {
    pub fn call_cousin() -> String {
        super::outer::middle::inner::describe()
    }

    pub fn total() -> u32 {
        use crate::outer::middle::inner;
        inner::from_root() * 10
    }
}

fn main() {
    println!("{}", outer::describe());
    println!("{}", outer::middle::describe());
    println!("{}", outer::middle::inner::describe());
    println!("root sum = {}", outer::middle::inner::from_root());
    println!("grandparent = {}", outer::middle::inner::grandparent());
    println!("private via public = {}", outer::middle::uses_private());
    println!("cousin: {}", sibling::call_cousin());
    println!("total = {}", sibling::total());
    println!("absolute = {}", crate::outer::middle::inner::LEVEL);
}
