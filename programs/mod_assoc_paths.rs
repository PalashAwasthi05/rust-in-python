// Associated fns, consts, and trait methods called via full paths.
mod geo {
    pub mod shapes {
        #[derive(Debug, Default, Clone, Copy)]
        pub struct Rect {
            pub w: u32,
            pub h: u32,
        }

        impl Rect {
            pub const UNIT: Rect = Rect { w: 1, h: 1 };

            pub fn new(w: u32, h: u32) -> Self {
                Self { w, h }
            }

            pub fn square(s: u32) -> Self {
                Self::new(s, s)
            }

            pub fn area(&self) -> u32 {
                self.w * self.h
            }
        }

        pub trait Scale {
            fn scale(&self, k: u32) -> Self;
            fn identity() -> Self;
        }

        impl Scale for Rect {
            fn scale(&self, k: u32) -> Self {
                Rect::new(self.w * k, self.h * k)
            }
            fn identity() -> Self {
                Rect::UNIT
            }
        }
    }
}

fn main() {
    let r = geo::shapes::Rect::new(2, 3);
    println!("{:?} {}", r, r.area());
    println!("{}", geo::shapes::Rect::area(&r));
    println!("{:?}", crate::geo::shapes::Rect::square(4));
    println!("{:?}", geo::shapes::Rect::UNIT);
    let s = <geo::shapes::Rect as geo::shapes::Scale>::scale(&r, 10);
    println!("{:?}", s);
    let i: geo::shapes::Rect = geo::shapes::Scale::identity();
    println!("{:?}", i);
    println!("{:?}", <geo::shapes::Rect as Default>::default());
    println!("{:?}", geo::shapes::Rect::default());
    let make: fn(u32, u32) -> geo::shapes::Rect = geo::shapes::Rect::new;
    println!("{}", make(5, 5).area());
    let areas: Vec<u32> = [r, s].iter().map(geo::shapes::Rect::area).collect();
    println!("{:?}", areas);
    let v: Vec<String> = vec![1, 2].iter().map(ToString::to_string).collect();
    println!("{:?}", v);
    println!("{}", i32::pow(2, 10));
    println!("{}", str::len("hello"));
    println!("{:?}", String::from_utf8(vec![104, 105]));
}
