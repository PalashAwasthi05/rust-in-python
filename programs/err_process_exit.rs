use std::process;

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("dropping {}", self.0);
    }
}

fn validate(args: &[&str]) -> Result<u32, String> {
    if args.is_empty() {
        return Err("no arguments".to_string());
    }
    args.iter()
        .map(|a| a.parse::<u32>().map_err(|e| format!("bad argument {:?}: {}", a, e)))
        .sum()
}

fn main() {
    {
        let _scoped = Guard("scoped");
    }
    let _kept = Guard("kept");
    println!("{:?}", validate(&["1", "2", "3"]));
    match validate(&["4", "five"]) {
        Ok(n) => println!("sum {}", n),
        Err(msg) => {
            eprintln!("error: {}", msg);
            println!("exiting with code 3");
            process::exit(3);
        }
    }
    println!("not reached");
}
