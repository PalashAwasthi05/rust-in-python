// Iterator, From, and operator impls written in modules other than their types.
mod types {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Celsius(pub f64);

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Fahrenheit(pub f64);

    pub struct Countdown {
        pub(crate) n: u32,
    }

    pub fn countdown(n: u32) -> Countdown {
        Countdown { n }
    }
}

mod conversions {
    use super::types::{Celsius, Fahrenheit};

    impl From<Celsius> for Fahrenheit {
        fn from(c: Celsius) -> Self {
            Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
        }
    }

    impl std::ops::Add for Celsius {
        type Output = Celsius;
        fn add(self, other: Celsius) -> Celsius {
            Celsius(self.0 + other.0)
        }
    }
}

mod iteration {
    impl Iterator for crate::types::Countdown {
        type Item = u32;
        fn next(&mut self) -> Option<u32> {
            if self.n == 0 {
                None
            } else {
                self.n -= 1;
                Some(self.n + 1)
            }
        }
    }
}

use types::{Celsius, Fahrenheit};

fn main() {
    let f: Fahrenheit = Celsius(100.0).into();
    println!("{:?}", f);
    println!("{:?}", Fahrenheit::from(Celsius(-40.0)));
    println!("{:?}", Celsius(1.5) + Celsius(2.0));
    let v: Vec<u32> = types::countdown(4).collect();
    println!("{:?}", v);
    let total: u32 = types::countdown(10).filter(|n| n % 2 == 0).sum();
    println!("{}", total);
    for n in types::countdown(2) {
        println!("tick {}", n);
    }
}
