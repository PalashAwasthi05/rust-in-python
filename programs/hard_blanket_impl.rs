use std::fmt::Display;

trait Shout {
    fn shout(&self) -> String;
    fn boxed(&self) -> String {
        let s = self.shout();
        let bar = "-".repeat(s.chars().count() + 2);
        format!("+{}+\n| {} |\n+{}+", bar, s, bar)
    }
}

impl<T: Display> Shout for T {
    fn shout(&self) -> String {
        format!("{}!", self.to_string().to_uppercase())
    }
}

trait Describe {
    fn describe(&self) -> String;
}

impl<T: Display + PartialOrd + Default> Describe for [T] {
    fn describe(&self) -> String {
        let zero = T::default();
        let pos = self.iter().filter(|x| **x > zero).count();
        let shown: Vec<String> = self.iter().map(|x| x.to_string()).collect();
        format!("[{}] with {} positive", shown.join(", "), pos)
    }
}

struct Celsius(f64);
impl Display for Celsius {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.1} deg c", self.0)
    }
}

trait DoubleIt {
    fn double_it(&self) -> Self;
}

impl<T> DoubleIt for T
where
    T: std::ops::Add<Output = T> + Copy,
{
    fn double_it(&self) -> T { *self + *self }
}

fn print_all<T: Shout>(items: &[T]) {
    for i in items { println!("{}", i.shout()); }
}

fn main() {
    println!("{}", "hello".shout());
    println!("{}", 42.shout());
    println!("{}", 'x'.shout());
    println!("{}", Celsius(21.456).shout());
    println!("{}", String::from("boxed text").boxed());
    print_all(&["one", "two"]);
    println!("{}", [3, -1, 0, 8][..].describe());
    println!("{}", vec![-0.5, 2.25][..].describe());
    println!("{} {} {}", 21.double_it(), 1.25f64.double_it(), 100u8.double_it());
    let words: Vec<String> = vec!["a", "bc"].iter().map(|w| w.shout()).collect();
    println!("{:?}", words);
}
