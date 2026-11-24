// pub use re-exports, facade modules, and renamed re-exports.
mod engine {
    mod parts {
        pub struct Piston {
            pub bore: u32,
        }

        impl Piston {
            pub fn displacement(&self) -> u32 {
                self.bore * self.bore
            }
        }

        pub fn spark() -> &'static str {
            "spark"
        }
    }

    mod tuning {
        pub fn boost(x: u32) -> u32 {
            x + x / 4
        }
    }

    pub use self::parts::Piston;
    pub use self::parts::spark as ignite;
    pub use tuning::*;

    pub mod prelude {
        pub use super::{ignite, Piston};
        pub use super::boost;
    }
}

pub use engine::prelude::*;

mod garage {
    use crate::engine::prelude::*;

    pub fn build() -> u32 {
        let p = Piston { bore: 4 };
        boost(p.displacement())
    }
}

fn main() {
    let p = engine::Piston { bore: 3 };
    println!("displacement {}", p.displacement());
    println!("{}", engine::ignite());
    println!("boost {}", engine::boost(40));
    println!("prelude {}", ignite());
    let q = Piston { bore: 10 };
    println!("boost via prelude {}", boost(q.displacement()));
    println!("garage {}", garage::build());
}
