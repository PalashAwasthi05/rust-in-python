use std::convert::From;

#[derive(Debug, PartialEq)]
struct Number { value: i32 }

impl From<i32> for Number {
    fn from(item: i32) -> Self { Number { value: item } }
}

#[derive(Debug)]
struct Celsius(f64);
#[derive(Debug)]
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}

impl From<Fahrenheit> for Celsius {
    fn from(f: Fahrenheit) -> Self { Celsius((f.0 - 32.0) * 5.0 / 9.0) }
}

#[derive(Debug)]
struct Rgb(u8, u8, u8);

impl From<u32> for Rgb {
    fn from(hex: u32) -> Self { Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8) }
}

impl From<Rgb> for u32 {
    fn from(c: Rgb) -> u32 { (c.0 as u32) << 16 | (c.1 as u32) << 8 | c.2 as u32 }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self { Rgb(r, g, b) }
}

#[derive(Debug)]
struct Tags(Vec<String>);

impl From<&str> for Tags {
    fn from(s: &str) -> Self { Tags(s.split(',').map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()).collect()) }
}

fn print_temp<T: Into<Celsius>>(t: T) {
    let c: Celsius = t.into();
    println!("temperature is {:.2} C", c.0);
}

fn total<I: Into<i64>>(items: Vec<I>) -> i64 {
    items.into_iter().map(Into::into).sum()
}

fn main() {
    let my_str = "hello";
    let my_string = String::from(my_str);
    println!("String::from: {}", my_string);
    let num = Number::from(30);
    println!("My number is {:?}", num);
    let int = 5;
    let num: Number = int.into();
    println!("Into gives {:?}, equal to From? {}", num, num == Number::from(5));

    let boiling: Fahrenheit = Celsius(100.0).into();
    println!("{:?}", boiling);
    let body = Celsius::from(Fahrenheit(98.6));
    println!("{:?} -> {:.1}", Fahrenheit(98.6), body.0);
    print_temp(Fahrenheit(-40.0));
    print_temp(Celsius(21.5));

    let teal = Rgb::from(0x008080);
    println!("{:?}", teal);
    let back: u32 = teal.into();
    println!("back to hex {:#08x}", back);
    let c: Rgb = (255, 128, 0).into();
    let shown = format!("{:?}", c);
    println!("from tuple {} -> {:#08x}", shown, u32::from(c));

    let tags: Tags = " Rust, systems,,WebAssembly ".into();
    println!("{:?}", tags);
    println!("widening sum: {}", total(vec![1i8, -2, 3]));
    println!("widening sum: {}", total(vec![u32::MAX, 1]));
    let chars: Vec<char> = String::from("abc").chars().collect();
    let s: String = chars.into_iter().rev().collect();
    let boxed: Box<str> = s.clone().into();
    let v: Vec<u8> = s.into();
    println!("boxed {:?} bytes {:?}", boxed, v);
    let f: f64 = 7u32.into();
    let wide = i64::from(-3i16);
    let o: Option<u8> = 9.into();
    println!("f64 {} i64 {} option {:?}", f, wide, o);
}
