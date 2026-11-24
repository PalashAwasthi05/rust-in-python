use std::fmt::Display;

trait Shout {
    fn shout(&self) -> String;
}

impl<T: Display> Shout for T {
    fn shout(&self) -> String {
        format!("{}!", self.to_string().to_uppercase())
    }
}

trait Describe {
    fn describe(&self) -> String;
}

impl<T: Display> Describe for Vec<T> {
    fn describe(&self) -> String {
        let parts: Vec<String> = self.iter().map(|x| x.to_string()).collect();
        format!("[{}] ({} items)", parts.join(", "), self.len())
    }
}

trait Double {
    fn double(&self) -> Self;
}

impl Double for i32 {
    fn double(&self) -> i32 {
        self * 2
    }
}

impl Double for String {
    fn double(&self) -> String {
        format!("{}{}", self, self)
    }
}

impl<T: Double + Clone> Double for Option<T> {
    fn double(&self) -> Self {
        self.as_ref().map(|x| x.double())
    }
}

struct Celsius(f64);

impl Display for Celsius {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.1}c", self.0)
    }
}

fn show_all<T>(items: &[T]) -> String
where
    T: Shout,
{
    items.iter().map(|i| i.shout()).collect::<Vec<_>>().join(" ")
}

fn main() {
    println!("{}", "hello".shout());
    println!("{}", 42.shout());
    println!("{}", Celsius(21.456).shout());
    println!("{}", vec![1, 2, 3].describe());
    println!("{}", vec!["a", "b"].describe());
    println!("{}", vec![Celsius(1.0), Celsius(-3.25)].describe());
    println!("{}", 21.double());
    println!("{}", String::from("ab").double());
    println!("{:?}", Some(5).double());
    println!("{:?}", Some(String::from("xy")).double());
    let none: Option<i32> = None;
    println!("{:?}", none.double());
    println!("{}", show_all(&['a', 'b', 'c']));
    println!("{}", show_all(&[1.5, 2.25]));
}
