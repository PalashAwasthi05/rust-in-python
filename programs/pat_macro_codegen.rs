// Macros that generate structs, impls, enums, and functions.
use std::fmt;

macro_rules! make_struct {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, Default)]
        struct $name {
            $($field: $ty),*
        }

        impl $name {
            fn field_names() -> Vec<&'static str> {
                vec![$(stringify!($field)),*]
            }
        }
    };
}

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $text:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum $name { $($variant),+ }

        impl $name {
            const ALL: &'static [$name] = &[$($name::$variant),+];

            fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }

            fn parse(s: &str) -> Option<$name> {
                match s {
                    $(x if x == $text => Some($name::$variant),)+
                    _ => None,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

macro_rules! getters {
    ($t:ty, $($field:ident: $ret:ty),*) => {
        impl $t {
            $(
                fn $field(&self) -> $ret {
                    self.$field.clone()
                }
            )*
        }
    };
}

macro_rules! binop_fns {
    ($($name:ident => $op:tt),*) => {
        $(
            fn $name(a: i32, b: i32) -> i32 { a $op b }
        )*
    };
}

trait Area {
    fn area(&self) -> f64;
}

macro_rules! impl_area {
    ($($t:ident => |$s:ident| $body:expr);+ $(;)?) => {
        $(
            impl Area for $t {
                fn area(&self) -> f64 {
                    let $s = self;
                    $body
                }
            }
        )+
    };
}

make_struct!(Person { name: String, age: u32, email: String });
make_struct!(Square { side: f64 });
make_struct!(Circle { radius: f64, });
string_enum!(Color { Red => "red", Green => "green", Blue => "blue" });
getters!(Person, name: String, age: u32);
binop_fns!(add => +, sub => -, mul => *, rem => %);
impl_area! {
    Square => |s| s.side * s.side;
    Circle => |c| 3.0 * c.radius * c.radius;
}

fn main() {
    let p = Person { name: "Ada".into(), age: 36, ..Default::default() };
    println!("{:?}", p);
    println!("{:?} / {} {}", Person::field_names(), p.name(), p.age());
    println!("{:?}", Square::field_names());
    for c in Color::ALL {
        println!("{} {:?} roundtrip={:?}", c, c, Color::parse(c.as_str()));
    }
    println!("{:?}", Color::parse("purple"));
    println!("{} {} {} {}", add(7, 3), sub(7, 3), mul(7, 3), rem(7, 3));
    let shapes: Vec<Box<dyn Area>> = vec![Box::new(Square { side: 2.0 }), Box::new(Circle { radius: 1.5 })];
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total area {:.2}", total);
}
