#[derive(Debug, Clone, Copy)]
struct Meters(f64);
#[derive(Debug, Clone, Copy)]
struct Feet(f64);

impl PartialEq for Meters {
    fn eq(&self, o: &Meters) -> bool { (self.0 - o.0).abs() < 1e-9 }
}
impl PartialEq<Feet> for Meters {
    fn eq(&self, o: &Feet) -> bool { (self.0 - o.0 * 0.3048).abs() < 1e-6 }
}
impl PartialEq<Meters> for Feet {
    fn eq(&self, o: &Meters) -> bool { o == self }
}
impl PartialOrd<Feet> for Meters {
    fn partial_cmp(&self, o: &Feet) -> Option<std::cmp::Ordering> { self.0.partial_cmp(&(o.0 * 0.3048)) }
}

#[derive(Debug)]
struct Username(String);

impl PartialEq<str> for Username {
    fn eq(&self, o: &str) -> bool { self.0.eq_ignore_ascii_case(o) }
}
impl PartialEq<&str> for Username {
    fn eq(&self, o: &&str) -> bool { self == *o }
}
impl PartialEq<Username> for &str {
    fn eq(&self, o: &Username) -> bool { o == self }
}

#[derive(Debug, PartialEq)]
enum Code { Ok, NotFound, Other(u16) }

impl PartialEq<u16> for Code {
    fn eq(&self, n: &u16) -> bool {
        match self {
            Code::Ok => *n == 200,
            Code::NotFound => *n == 404,
            Code::Other(c) => c == n,
        }
    }
}

fn main() {
    let m = Meters(3.048);
    let f = Feet(10.0);
    println!("{:?} == {:?}: {}", m, f, m == f);
    println!("{:?} == {:?}: {}", f, m, f == m);
    println!("Meters(1) == Feet(3): {}", Meters(1.0) == Feet(3.0));
    println!("Meters(1) > Feet(3): {}", Meters(1.0) > Feet(3.0));
    println!("Meters(1) < Feet(4): {}", Meters(1.0) < Feet(4.0));
    println!("Meters eq Meters: {}", Meters(0.1 + 0.2) == Meters(0.3));

    let u = Username("Ferris".into());
    println!("u == \"ferris\": {}", u == "ferris");
    println!("\"FERRIS\" == u: {}", "FERRIS" == u);
    println!("u != \"crab\": {}", u != "crab");
    let users = vec![Username("ann".into()), Username("Bob".into()), Username("cy".into())];
    let pos = users.iter().position(|x| *x == "BOB");
    println!("found bob at {:?}", pos);

    for (c, n) in [(Code::Ok, 200u16), (Code::NotFound, 200), (Code::NotFound, 404), (Code::Other(503), 503)] {
        println!("{:?} == {}: {}", c, n, c == n);
    }
    println!("Code eq Code: {} {}", Code::Other(1) == Code::Other(1), Code::Ok != Code::NotFound);
    let codes = [Code::Ok, Code::Other(500), Code::NotFound, Code::Other(500)];
    let errors = codes.iter().filter(|c| **c == 500u16 || **c == 404u16).count();
    println!("error codes: {}", errors);
    println!("String == &str: {}, Vec == array: {}", String::from("a") == "a", vec![1, 2] == [1, 2]);
}
