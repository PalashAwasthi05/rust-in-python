use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct MyStr {
    inner: String,
}

impl MyStr {
    fn new(s: &str) -> MyStr {
        MyStr { inner: s.to_string() }
    }
}

impl PartialEq for MyStr {
    fn eq(&self, other: &MyStr) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<&str> for MyStr {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

impl PartialEq<str> for MyStr {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<MyStr> for &str {
    fn eq(&self, other: &MyStr) -> bool {
        *self == other.inner
    }
}

impl PartialEq<String> for MyStr {
    fn eq(&self, other: &String) -> bool {
        &self.inner == other
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Celsius(f64);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fahrenheit(f64);

impl PartialEq<Fahrenheit> for Celsius {
    fn eq(&self, other: &Fahrenheit) -> bool {
        (self.0 * 9.0 / 5.0 + 32.0 - other.0).abs() < 1e-9
    }
}

impl PartialOrd<Fahrenheit> for Celsius {
    fn partial_cmp(&self, other: &Fahrenheit) -> Option<Ordering> {
        (self.0 * 9.0 / 5.0 + 32.0).partial_cmp(&other.0)
    }
}

impl PartialOrd for Celsius {
    fn partial_cmp(&self, other: &Celsius) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Cents(i64);

impl PartialEq<i64> for Cents {
    fn eq(&self, other: &i64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<i64> for Cents {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

fn find<'a, T: PartialEq<&'a str>>(items: &[T], needle: &'a str) -> Option<usize> {
    items.iter().position(|x| *x == needle)
}

fn main() {
    let s = MyStr::new("hello");
    println!("s == \"hello\": {}", s == "hello");
    println!("s != \"world\": {}", s != "world");
    println!("\"hello\" == s: {}", "hello" == s);
    println!("s == *\"hello\": {}", s == *"hello");
    println!("s == String: {}", s == String::from("hello"));
    println!("s == MyStr: {}", s == MyStr::new("hello"));
    println!("s.eq(&\"x\"): {}", s.eq(&"x"));
    println!("s.ne(&\"x\"): {}", s.ne(&"x"));

    let list = vec![MyStr::new("a"), MyStr::new("b"), MyStr::new("c")];
    println!("find b: {:?}", find(&list, "b"));
    println!("find z: {:?}", find(&list, "z"));
    println!("contains c: {}", list.iter().any(|m| *m == "c"));
    println!("vec eq: {}", list == vec![MyStr::new("a"), MyStr::new("b"), MyStr::new("c")]);

    let boiling = Celsius(100.0);
    println!("100C == 212F: {}", boiling == Fahrenheit(212.0));
    println!("100C != 211F: {}", boiling != Fahrenheit(211.0));
    println!("100C < 300F: {}", boiling < Fahrenheit(300.0));
    println!("100C > 100F: {}", boiling > Fahrenheit(100.0));
    println!("100C >= 212F: {}", boiling >= Fahrenheit(212.0));
    println!("cmp: {:?}", boiling.partial_cmp(&Fahrenheit(32.0)));
    println!("C vs C: {:?}", Celsius(1.0).partial_cmp(&Celsius(2.0)));
    println!("C < C: {}", Celsius(1.0) < Celsius(2.0));

    let price = Cents(499);
    println!("price == 499: {}", price == 499);
    println!("price < 500: {}", price < 500);
    println!("price > 1000: {}", price > 1000);
    println!("price <= 499: {}", price <= 499);
    let prices = [Cents(100), Cents(250), Cents(999)];
    let cheap: Vec<&Cents> = prices.iter().filter(|p| **p < 300).collect();
    println!("cheap: {:?}", cheap);

    let owned = String::from("abc");
    let borrowed: &str = "abc";
    println!("String == &str: {}", owned == borrowed);
    println!("&String == &str: {}", &owned == &borrowed);
    println!("str == String: {}", *borrowed == owned);
    let chars = vec!['a', 'b'];
    println!("Vec<char> == [char;2]: {}", chars == ['a', 'b']);
    let opt_s: Option<&str> = Some("x");
    println!("Option<&str> == Some(\"x\"): {}", opt_s == Some("x"));
    println!("as_deref: {}", Some(String::from("y")).as_deref() == Some("y"));
}
