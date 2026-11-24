// Consts, statics, and arrays in modules, and const fns across modules.
mod config {
    pub const NAME: &str = "app";
    pub const VERSION: (u32, u32) = (1, 4);
    pub static GREETING: &str = "hello";
    pub static PRIMES: [u32; 5] = [2, 3, 5, 7, 11];
    pub const MAX_USERS: usize = limits::BASE * 4;

    pub mod limits {
        pub const BASE: usize = 25;
        pub const fn doubled(x: usize) -> usize {
            x * 2
        }
        pub const DOUBLE_BASE: usize = doubled(BASE);
    }

    pub struct Settings {
        pub verbose: bool,
        pub level: u8,
    }

    pub const DEFAULT: Settings = Settings { verbose: false, level: 3 };

    impl Settings {
        pub const LOUD: Settings = Settings { verbose: true, level: 9 };
    }
}

use config::limits::{self, DOUBLE_BASE};

fn main() {
    println!("{} v{}.{}", config::NAME, config::VERSION.0, config::VERSION.1);
    println!("{}", config::GREETING);
    println!("{:?} sum={}", config::PRIMES, config::PRIMES.iter().sum::<u32>());
    println!("{} {} {}", config::MAX_USERS, DOUBLE_BASE, limits::doubled(7));
    let d = config::DEFAULT;
    println!("{} {}", d.verbose, d.level);
    let l = config::Settings::LOUD;
    println!("{} {}", l.verbose, l.level);
    const LOCAL: usize = config::limits::BASE + 1;
    let arr = [0u8; LOCAL];
    println!("{}", arr.len());
    println!("{}", config::PRIMES[config::PRIMES.len() - 1]);
}
