use std::fmt::{self, Debug, Display, Write};

#[derive(Debug, Clone)]
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

impl<T: Display> Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tree::Leaf(v) => write!(f, "{}", v),
            Tree::Node(l, r) => write!(f, "({} {})", l, r),
        }
    }
}

struct Celsius(f64);
impl Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = f.precision() {
            write!(f, "{:.*}°C", p, self.0)
        } else {
            write!(f, "{}°C", self.0)
        }
    }
}

struct Labeled<T> {
    label: &'static str,
    value: T,
}
impl<T: Debug> Debug for Labeled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.label).field(&self.value).finish()
    }
}

fn show_all<T: Display>(items: &[T]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write!(out, "<{}>", item).unwrap();
    }
    out
}

fn describe(value: &dyn Display) -> String {
    format!("[{:>6}]", value.to_string())
}

fn debug_twice<T: Debug>(value: T) -> String {
    format!("{:?} / {:#?}", value, value)
}

fn main() {
    let tree = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Node(Box::new(Tree::Leaf(2)), Box::new(Tree::Leaf(3)))));
    println!("{}", tree);
    println!("{:?}", tree);
    println!("{:#?}", Tree::Node(Box::new(Tree::Leaf("a")), Box::new(Tree::Leaf("b"))));
    println!("{} {:.1} {:.3}", Celsius(21.456), Celsius(21.456), Celsius(-0.5));
    println!("{:?}", Labeled { label: "Meters", value: 3.5 });
    println!("{:#?}", Labeled { label: "Pair", value: (1, 'x') });
    println!("{}", show_all(&[1.5, 2.0, -0.25]));
    println!("{}", show_all(&["x", "y"]));
    println!("{}", describe(&42));
    println!("{}", describe(&Celsius(1.0)));
    let things: Vec<Box<dyn Display>> = vec![Box::new(1u8), Box::new("two"), Box::new(3.0f32), Box::new('4'), Box::new(Celsius(5.0))];
    for thing in &things {
        print!("{} ", thing);
    }
    println!();
    let padded: Vec<String> = things.iter().map(|t| format!("{:>5}", t)).collect();
    println!("{:?}", padded);
    println!("{}", debug_twice(Some((1, "a"))));
    println!("{}", debug_twice(vec![Labeled { label: "L", value: () }]));
    let s = 255u8.to_string() + &(-1i64).to_string() + &1e-3.to_string() + &true.to_string() + &'c'.to_string();
    println!("{}", s);
    println!("{}", 0.1f32.to_string());
    println!("{}", format!("{:?}", 1.0f64).len());
}
