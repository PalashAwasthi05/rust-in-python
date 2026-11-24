// Enum dispatch vs dyn dispatch for the same family of types, plus generic dispatch.
use std::fmt;

trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> &'static str;
    fn legs(&self) -> u32 {
        4
    }
    fn speak(&self) -> String {
        format!("{} the {}-legged says {}", self.name(), self.legs(), self.sound())
    }
}

#[derive(Clone)]
struct Dog {
    name: String,
}
#[derive(Clone)]
struct Bird {
    name: String,
    can_fly: bool,
}
#[derive(Clone)]
struct Snake;

impl Animal for Dog {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn sound(&self) -> &'static str {
        "woof"
    }
}

impl Animal for Bird {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn sound(&self) -> &'static str {
        if self.can_fly {
            "tweet"
        } else {
            "squawk"
        }
    }
    fn legs(&self) -> u32 {
        2
    }
}

impl Animal for Snake {
    fn name(&self) -> String {
        "snake".to_string()
    }
    fn sound(&self) -> &'static str {
        "hiss"
    }
    fn legs(&self) -> u32 {
        0
    }
    fn speak(&self) -> String {
        "...hiss...".to_string()
    }
}

// Enum dispatch: each variant wraps a concrete type.
#[derive(Clone)]
enum AnyAnimal {
    Dog(Dog),
    Bird(Bird),
    Snake(Snake),
}

impl AnyAnimal {
    fn inner(&self) -> &dyn Animal {
        match self {
            AnyAnimal::Dog(d) => d,
            AnyAnimal::Bird(b) => b,
            AnyAnimal::Snake(s) => s,
        }
    }
}

impl Animal for AnyAnimal {
    fn name(&self) -> String {
        match self {
            AnyAnimal::Dog(d) => d.name(),
            AnyAnimal::Bird(b) => b.name(),
            AnyAnimal::Snake(s) => s.name(),
        }
    }
    fn sound(&self) -> &'static str {
        self.inner().sound()
    }
    fn legs(&self) -> u32 {
        self.inner().legs()
    }
    fn speak(&self) -> String {
        self.inner().speak()
    }
}

impl From<Dog> for AnyAnimal {
    fn from(d: Dog) -> Self {
        AnyAnimal::Dog(d)
    }
}
impl From<Bird> for AnyAnimal {
    fn from(b: Bird) -> Self {
        AnyAnimal::Bird(b)
    }
}

impl fmt::Display for AnyAnimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self {
            AnyAnimal::Dog(_) => "Dog",
            AnyAnimal::Bird(Bird { can_fly: true, .. }) => "FlyingBird",
            AnyAnimal::Bird(_) => "Bird",
            AnyAnimal::Snake(_) => "Snake",
        };
        write!(f, "{}({})", kind, self.name())
    }
}

fn chorus_static<A: Animal>(animals: &[A]) -> Vec<String> {
    animals.iter().map(|a| a.speak()).collect()
}

fn chorus_dyn(animals: &[Box<dyn Animal>]) -> Vec<String> {
    animals.iter().map(|a| a.speak()).collect()
}

fn total_legs<'a, I>(it: I) -> u32
where
    I: IntoIterator<Item = &'a dyn Animal>,
{
    it.into_iter().map(|a| a.legs()).sum()
}

fn loudest(a: &impl Animal, b: &impl Animal) -> String {
    if a.sound().len() >= b.sound().len() {
        a.name()
    } else {
        b.name()
    }
}

fn main() {
    let rex = Dog { name: "Rex".into() };
    let tweety = Bird { name: "Tweety".into(), can_fly: true };
    let pingu = Bird { name: "Pingu".into(), can_fly: false };

    let enum_zoo: Vec<AnyAnimal> =
        vec![rex.clone().into(), tweety.clone().into(), AnyAnimal::Snake(Snake), pingu.clone().into()];
    let dyn_zoo: Vec<Box<dyn Animal>> =
        vec![Box::new(rex.clone()), Box::new(tweety.clone()), Box::new(Snake), Box::new(pingu.clone())];

    for (e, d) in chorus_static(&enum_zoo).iter().zip(chorus_dyn(&dyn_zoo)) {
        println!("{:<34} | {}", e, d);
        assert_eq!(*e, d);
    }
    for a in &enum_zoo {
        print!("{} ", a);
    }
    println!();

    let refs: Vec<&dyn Animal> = dyn_zoo.iter().map(|b| b.as_ref()).collect();
    println!("legs (dyn): {}", total_legs(refs));
    println!("legs (enum): {}", total_legs(enum_zoo.iter().map(|a| a as &dyn Animal)));
    println!("dogs only: {:?}", chorus_static(&[rex.clone(), Dog { name: "Fido".into() }]));
    println!("loudest: {}", loudest(&rex, &tweety));
    println!("loudest: {}", loudest(&Snake, &pingu));

    let fliers = enum_zoo.iter().filter(|a| matches!(a, AnyAnimal::Bird(Bird { can_fly: true, .. }))).count();
    println!("fliers: {}", fliers);
    println!("size_of AnyAnimal > 0: {}", std::mem::size_of::<AnyAnimal>() > 0);
}
