// Trait methods need the trait in scope; blanket impls across modules.
mod ext {
    pub trait Doubled {
        fn doubled(&self) -> Self;
    }

    impl Doubled for i32 {
        fn doubled(&self) -> i32 {
            self * 2
        }
    }

    impl Doubled for String {
        fn doubled(&self) -> String {
            format!("{}{}", self, self)
        }
    }

    pub trait Describe {
        fn describe(&self) -> String;
    }

    impl<T: std::fmt::Debug> Describe for T {
        fn describe(&self) -> String {
            format!("value {:?}", self)
        }
    }
}

mod user {
    use crate::ext::Doubled as _;

    pub fn quad(x: i32) -> i32 {
        x.doubled().doubled()
    }
}

mod other {
    pub fn run() -> String {
        use crate::ext::{Describe, Doubled};
        format!("{} / {}", "ab".to_string().doubled(), vec![1, 2].describe())
    }
}

fn main() {
    println!("{}", user::quad(3));
    println!("{}", other::run());
    use ext::Describe;
    println!("{}", (1, 'x').describe());
    println!("{}", ext::Doubled::doubled(&21));
    println!("{}", <String as ext::Doubled>::doubled(&"z".to_string()));
}
