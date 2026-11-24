use std::fmt::{self, Debug, Display};

trait Named {
    fn name(&self) -> String;
}

trait Animal: Named {
    fn sound(&self) -> &'static str;
    fn legs(&self) -> u32 {
        4
    }
    fn intro(&self) -> String {
        format!("{} says {} on {} legs", self.name(), self.sound(), self.legs())
    }
}

trait Pet: Animal + Debug {
    fn owner(&self) -> &str;
    fn card(&self) -> String {
        format!("{} (owner: {}) {:?}", self.intro(), self.owner(), self)
    }
}

#[derive(Debug)]
struct Dog {
    name: String,
    owner: String,
}

#[derive(Debug)]
struct Parrot {
    words: u32,
}

impl Named for Dog {
    fn name(&self) -> String {
        self.name.clone()
    }
}
impl Animal for Dog {
    fn sound(&self) -> &'static str {
        "woof"
    }
}
impl Pet for Dog {
    fn owner(&self) -> &str {
        &self.owner
    }
}

impl Named for Parrot {
    fn name(&self) -> String {
        format!("parrot#{}", self.words)
    }
}
impl Animal for Parrot {
    fn sound(&self) -> &'static str {
        "squawk"
    }
    fn legs(&self) -> u32 {
        2
    }
}
impl Pet for Parrot {
    fn owner(&self) -> &str {
        "the zoo"
    }
}

fn loudest(pets: &[Box<dyn Pet>]) -> String {
    pets.iter().map(|p| p.name()).max_by_key(|n| n.len()).unwrap_or_default()
}

fn intro_all(animals: &Vec<&dyn Animal>) {
    for a in animals {
        println!("- {}", a.intro());
    }
}

struct Wrapper<T>(Vec<T>);

impl<T: Display> Display for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<")?;
        for (i, x) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "|")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, ">")
    }
}

trait Loud: Display {
    fn loud(&self) -> String {
        self.to_string().to_uppercase()
    }
}
impl<T: Display> Loud for Wrapper<T> {}

fn main() {
    let pets: Vec<Box<dyn Pet>> = vec![
        Box::new(Dog { name: "Rex".to_string(), owner: "Sam".to_string() }),
        Box::new(Parrot { words: 12 }),
    ];
    for p in pets.iter() {
        println!("{}", p.card());
    }
    println!("longest name: {}", loudest(&pets));
    let d = Dog { name: "Fido".into(), owner: "Al".into() };
    let pa = Parrot { words: 3 };
    intro_all(&vec![&d, &pa]);
    let total_legs: u32 = pets.iter().map(|p| p.legs()).sum();
    println!("legs: {}", total_legs);
    let w = Wrapper(vec!["a", "bc", "d"]);
    println!("{} {}", w, w.loud());
    println!("{}", Wrapper(vec![1.5, 2.0]));
}
