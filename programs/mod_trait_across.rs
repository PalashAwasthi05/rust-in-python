// Traits defined in one module, implemented in another, imported to call.
mod traits {
    pub trait Speak {
        fn name(&self) -> String;
        fn speak(&self) -> String {
            format!("{} makes a sound", self.name())
        }
    }

    pub trait Loud: Speak {
        fn shout(&self) -> String {
            self.speak().to_uppercase()
        }
    }

    pub trait Counted {
        const LEGS: u32;
        fn legs(&self) -> u32 {
            Self::LEGS
        }
    }
}

mod animals {
    pub struct Dog;
    pub struct Bird {
        pub song: String,
    }
}

mod impls {
    use crate::animals::{Bird, Dog};
    use crate::traits::{Counted, Loud, Speak};

    impl Speak for Dog {
        fn name(&self) -> String {
            "dog".to_string()
        }
        fn speak(&self) -> String {
            "woof".to_string()
        }
    }

    impl Loud for Dog {}

    impl Speak for Bird {
        fn name(&self) -> String {
            format!("bird singing {}", self.song)
        }
    }

    impl Counted for Dog {
        const LEGS: u32 = 4;
    }

    impl Counted for Bird {
        const LEGS: u32 = 2;
    }
}

mod zoo {
    pub fn tally<T: crate::traits::Counted>(items: &[T]) -> u32 {
        items.iter().map(|i| i.legs()).sum()
    }
}

use animals::{Bird, Dog};
use traits::Speak;

fn introduce(s: &dyn Speak) -> String {
    format!("intro: {}", s.speak())
}

fn main() {
    let d = Dog;
    let b = Bird { song: "tweet".into() };
    println!("{}", d.speak());
    println!("{}", b.speak());
    {
        use traits::Loud;
        println!("{}", d.shout());
    }
    println!("{}", traits::Loud::shout(&d));
    println!("{}", <Dog as traits::Counted>::LEGS);
    println!("{}", zoo::tally(&[Dog, Dog]));
    println!("{}", zoo::tally(&[Bird { song: "a".into() }]));
    println!("{}", introduce(&b));
    let all: Vec<Box<dyn Speak>> = vec![Box::new(Dog), Box::new(Bird { song: "chirp".into() })];
    for a in all.iter() {
        println!("- {}", a.name());
    }
}
