use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

trait Describe {
    fn describe(&self) -> String;
    fn shout(&self) -> String {
        self.describe().to_uppercase()
    }
}

impl<T: Describe + ?Sized> Describe for Box<T> {
    fn describe(&self) -> String {
        format!("box({})", (**self).describe())
    }
}

impl<T: Describe> Describe for &T {
    fn describe(&self) -> String {
        format!("ref({})", (*self).describe())
    }
}

impl<T: Describe> Describe for Rc<T> {
    fn describe(&self) -> String {
        format!("rc({})", self.as_ref().describe())
    }
}

impl<T: Describe> Describe for Option<T> {
    fn describe(&self) -> String {
        match self {
            Some(x) => format!("some {}", x.describe()),
            None => "nothing".to_string(),
        }
    }
}

impl<A: Describe, B: Describe> Describe for (A, B) {
    fn describe(&self) -> String {
        format!("<{} & {}>", self.0.describe(), self.1.describe())
    }
}

impl<T: Describe> Describe for [T] {
    fn describe(&self) -> String {
        let parts: Vec<String> = self.iter().map(|x| x.describe()).collect();
        format!("[{}]", parts.join("; "))
    }
}

struct Cat;
impl Describe for Cat {
    fn describe(&self) -> String {
        "cat".into()
    }
}

impl Describe for i32 {
    fn describe(&self) -> String {
        format!("int {}", self)
    }
}

fn print_it<D: Describe + ?Sized>(d: &D) {
    println!("{}", d.describe());
}

#[derive(Debug, Clone, Eq)]
struct Tag {
    name: String,
    weight: u32,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.to_lowercase().hash(state);
    }
}

impl PartialEq<str> for Tag {
    fn eq(&self, other: &str) -> bool {
        self.name.eq_ignore_ascii_case(other)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.name)
    }
}

fn main() {
    print_it(&Cat);
    print_it(&Box::new(Cat));
    print_it(&&Cat);
    print_it(&Rc::new(7));
    print_it(&Some(Cat));
    print_it(&None::<i32>);
    print_it(&(Cat, 3));
    print_it(&[1, 2, 3][..]);
    let boxed: Box<dyn Describe> = Box::new(Cat);
    println!("{}", boxed.describe());
    println!("{}", boxed.shout());
    let v: Vec<Box<dyn Describe>> = vec![Box::new(5), Box::new(Cat), Box::new((1, Cat))];
    for d in &v {
        println!("{}", d.shout());
    }
    let mut tags = HashSet::new();
    for (n, w) in [("Rust", 3), ("rust", 9), ("Go", 1), ("RUST", 2), ("go", 5), ("Zig", 4)] {
        let inserted = tags.insert(Tag { name: n.to_string(), weight: w });
        println!("{} {}", n, inserted);
    }
    let mut kept: Vec<&Tag> = tags.iter().collect();
    kept.sort_by_key(|t| t.weight);
    let shown: Vec<String> = kept.iter().map(|t| t.to_string()).collect();
    println!("{}", shown.join(" "));
    let t = Tag { name: "Python".into(), weight: 0 };
    println!("{} {} {}", t == *"python", t == *"py", t == Tag { name: "PYTHON".into(), weight: 1 });
}
