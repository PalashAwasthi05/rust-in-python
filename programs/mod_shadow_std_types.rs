// User types and traits that reuse names from the prelude and std.
mod mine {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Option<T> {
        Nothing,
        Just(T),
    }

    impl<T: Clone> Option<T> {
        pub fn get_or(&self, fallback: T) -> T {
            match self {
                Option::Nothing => fallback,
                Option::Just(v) => v.clone(),
            }
        }
    }

    pub trait Display {
        fn show(&self) -> String;
    }

    impl Display for i32 {
        fn show(&self) -> String {
            format!("<{}>", self)
        }
    }

    #[derive(Debug)]
    pub struct Box {
        pub width: u32,
    }

    pub fn drop(b: Box) -> u32 {
        b.width
    }
}

use mine::Display;

fn main() {
    let a: mine::Option<i32> = mine::Option::Just(4);
    let b: mine::Option<i32> = mine::Option::Nothing;
    println!("{:?} {:?} {} {}", a, b, a.get_or(0), b.get_or(9));
    let real: Option<i32> = Some(3);
    println!("{:?}", real);
    println!("{}", 5.show());
    println!("{}", format!("{}", 5));
    let bx = mine::Box { width: 3 };
    println!("{:?}", bx);
    println!("{}", mine::drop(bx));
    let heap: Box<i32> = Box::new(8);
    println!("{}", heap);
    let v = vec![1, 2];
    drop(v);
    println!("done");
}
