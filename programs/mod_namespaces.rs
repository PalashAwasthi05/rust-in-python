// Type, value, and module namespaces: a module shares its name with a fn, a struct, and a local.
mod parse {
    pub fn parse(s: &str) -> Vec<i32> {
        s.split(',').filter_map(|p| p.trim().parse().ok()).collect()
    }

    pub mod rules {
        pub const DEPTH: u32 = 2;
    }
}

mod config {
    #[derive(Debug, Default)]
    pub struct config {
        pub debug: bool,
    }
}

fn config() -> config::config {
    config::config { debug: true }
}

use parse::parse;

fn main() {
    println!("{:?}", parse("1, 2, x, 3"));
    println!("{}", parse::rules::DEPTH);
    println!("{}", self::parse::parse("9")[0]);
    println!("{:?}", config());
    println!("{:?}", config::config::default());
    println!("{}", config::config { debug: false }.debug);
    let parse = 5;
    println!("{}", parse + parse::rules::DEPTH as i32);
}
