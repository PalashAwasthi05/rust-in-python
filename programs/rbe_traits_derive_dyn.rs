#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
struct Centimeters(f64);

#[derive(Debug)]
struct Inches(i32);

impl Inches {
    fn to_centimeters(&self) -> Centimeters {
        let &Inches(inches) = self;
        Centimeters(inches as f64 * 2.54)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Version { major: u16, minor: u16, patch: u16 }

struct Sheep { naked: bool, name: &'static str }
struct Cow { noise_level: u8 }

trait Animal {
    fn name(&self) -> String;
    fn noise(&self) -> String;
    fn talk(&self) -> String { format!("{} says {}", self.name(), self.noise()) }
}

impl Sheep {
    fn is_naked(&self) -> bool { self.naked }
    fn shear(&mut self) -> String {
        if self.is_naked() {
            format!("{} is already naked...", self.name())
        } else {
            self.naked = true;
            format!("{} gets a haircut!", self.name)
        }
    }
}

impl Animal for Sheep {
    fn name(&self) -> String { self.name.to_string() }
    fn noise(&self) -> String { if self.is_naked() { "baaaaah?".into() } else { "baaaaah!".into() } }
    fn talk(&self) -> String { format!("{} pauses briefly... {}", self.name, self.noise()) }
}

impl Animal for Cow {
    fn name(&self) -> String { "Cow".into() }
    fn noise(&self) -> String { format!("m{}", "o".repeat(self.noise_level as usize)) }
}

fn random_animal(number: f64) -> Box<dyn Animal> {
    if number < 0.5 { Box::new(Sheep { naked: false, name: "Dolly" }) } else { Box::new(Cow { noise_level: 4 }) }
}

fn loudest(animals: &[Box<dyn Animal>]) -> &dyn Animal {
    animals.iter().max_by_key(|a| a.noise().len()).map(|b| b.as_ref()).unwrap()
}

fn main() {
    let _one_second = Centimeters(1.0);
    let foot = Inches(12);
    println!("one foot equals {:?}", foot);
    let meter = Centimeters(100.0);
    let cmp = if foot.to_centimeters() < meter { "smaller" } else { "bigger" };
    println!("one foot is {} than one meter.", cmp);

    let mut versions = vec![
        Version { major: 1, minor: 2, patch: 3 },
        Version::default(),
        Version { major: 1, minor: 10, patch: 0 },
        Version { major: 0, minor: 9, patch: 9 },
    ];
    versions.sort();
    println!("sorted: {:?}", versions);
    println!("max: {:?}", versions.iter().max());
    let cloned = versions[1].clone();
    println!("clone equal: {}", cloned == versions[1]);
    let mut set = std::collections::HashSet::new();
    for v in &versions { set.insert(v.clone()); }
    set.insert(Version::default());
    println!("unique versions: {}", set.len());

    let mut dolly = Sheep { naked: false, name: "Dolly" };
    println!("{}", dolly.talk());
    println!("{}", dolly.shear());
    println!("{}", dolly.shear());
    println!("{}", dolly.talk());

    let zoo: Vec<Box<dyn Animal>> = vec![random_animal(0.234), random_animal(0.9), Box::new(Cow { noise_level: 7 })];
    for a in &zoo { println!("{}", a.talk()); }
    println!("loudest: {}", loudest(&zoo).name());
    println!("size of &dyn Animal: {} words", std::mem::size_of::<&dyn Animal>() / std::mem::size_of::<usize>());
}
