use std::fmt::{self, Display, Formatter};

struct City { name: &'static str, lat: f32, lon: f32 }

impl Display for City {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let lat_c = if self.lat >= 0.0 { 'N' } else { 'S' };
        let lon_c = if self.lon >= 0.0 { 'E' } else { 'W' };
        write!(f, "{}: {:.3}°{} {:.3}°{}", self.name, self.lat.abs(), lat_c, self.lon.abs(), lon_c)
    }
}

#[derive(Debug)]
struct Color { red: u8, green: u8, blue: u8 }

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RGB ({}, {}, {}) 0x{:02X}{:02X}{:02X}", self.red, self.green, self.blue, self.red, self.green, self.blue)
    }
}

struct List(Vec<i32>);

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, v) in self.0.iter().enumerate() {
            if i != 0 { write!(f, ", ")?; }
            write!(f, "{}: {}", i, v)?;
        }
        write!(f, "]")
    }
}

struct Complex { re: f64, im: f64 }

impl Display for Complex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let sign = if self.im < 0.0 { '-' } else { '+' };
        write!(f, "{} {} {}i", self.re, sign, self.im.abs())
    }
}

impl fmt::Debug for Complex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Complex").field("real", &self.re).field("imag", &self.im).finish()
    }
}

struct Padded(&'static str);

impl Display for Padded {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // forward width/alignment flags to the inner str
        f.pad(self.0)
    }
}

#[derive(Debug)]
struct Matrix(f32, f32, f32, f32);

#[derive(Debug)]
enum Shape { Circle { r: f32 }, Square(f32), Empty }

#[derive(Debug)]
struct Scene { name: String, shapes: Vec<Shape>, origin: (i32, i32) }

fn main() {
    for city in [City { name: "Dublin", lat: 53.347778, lon: -6.259722 }, City { name: "Oslo", lat: 59.95, lon: 10.75 }, City { name: "Vancouver", lat: 49.25, lon: -123.1 }] {
        println!("{}", city);
    }
    for c in [Color { red: 128, green: 255, blue: 90 }, Color { red: 0, green: 3, blue: 254 }] {
        println!("{} / {:?}", c, c);
    }
    println!("{}", List(vec![1, 2, 3]));
    let z = Complex { re: 3.3, im: -7.2 };
    println!("display: {}", z);
    println!("debug: {:?}", z);
    println!("pretty: {:#?}", z);
    println!("[{:>8}] [{:<8}] [{:^8}]", Padded("pad"), Padded("pad"), Padded("pad"));
    println!("{:?}", Matrix(1.1, 1.2, 2.1, 2.2));
    let scene = Scene { name: "demo".into(), shapes: vec![Shape::Circle { r: 1.5 }, Shape::Square(2.0), Shape::Empty], origin: (0, -3) };
    println!("{:?}", scene);
    println!("{:#?}", scene);
    let s = z.to_string();
    println!("to_string gives {:?} of length {}", s, s.len());
}
