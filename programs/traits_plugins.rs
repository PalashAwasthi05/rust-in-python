use std::collections::BTreeMap;
use std::fmt;

trait Command: fmt::Debug {
    fn name(&self) -> &str;
    fn run(&mut self, args: &[&str]) -> Result<String, String>;
    fn calls(&self) -> usize {
        0
    }
}

#[derive(Debug, Default)]
struct Echo {
    count: usize,
}

#[derive(Debug)]
struct Sum;

#[derive(Debug)]
struct Repeat {
    times: usize,
}

impl Command for Echo {
    fn name(&self) -> &str {
        "echo"
    }
    fn run(&mut self, args: &[&str]) -> Result<String, String> {
        self.count += 1;
        Ok(args.join(" "))
    }
    fn calls(&self) -> usize {
        self.count
    }
}

impl Command for Sum {
    fn name(&self) -> &str {
        "sum"
    }
    fn run(&mut self, args: &[&str]) -> Result<String, String> {
        let mut total = 0i64;
        for a in args {
            total += a.parse::<i64>().map_err(|e| format!("{}: {}", a, e))?;
        }
        Ok(total.to_string())
    }
}

impl Command for Repeat {
    fn name(&self) -> &str {
        "repeat"
    }
    fn run(&mut self, args: &[&str]) -> Result<String, String> {
        let word = args.first().ok_or("repeat needs a word")?;
        Ok(vec![*word; self.times].join("-"))
    }
}

struct Registry {
    commands: BTreeMap<String, Box<dyn Command>>,
}

impl Registry {
    fn new() -> Self {
        Registry { commands: BTreeMap::new() }
    }
    fn register(&mut self, c: Box<dyn Command>) -> &mut Self {
        self.commands.insert(c.name().to_string(), c);
        self
    }
    fn exec(&mut self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (head, rest) = match parts.split_first() {
            Some((h, r)) => (*h, r),
            None => return "empty".to_string(),
        };
        match self.commands.get_mut(head) {
            Some(cmd) => match cmd.run(rest) {
                Ok(out) => out,
                Err(e) => format!("error: {}", e),
            },
            None => format!("unknown command {:?}", head),
        }
    }
}

fn describe(c: &dyn Command) -> String {
    format!("{}<{:?}>", c.name(), c)
}

fn main() {
    let mut reg = Registry::new();
    reg.register(Box::new(Echo::default())).register(Box::new(Sum)).register(Box::new(Repeat { times: 3 }));
    for line in ["echo hi there", "sum 1 2 3", "sum 4 x", "repeat ab", "repeat", "nope 1", "", "echo again"] {
        println!("> {:<14} => {}", line, reg.exec(line));
    }
    for (name, cmd) in &reg.commands {
        println!("{} called {} times; {}", name, cmd.calls(), describe(cmd.as_ref()));
    }
    let boxed: Vec<Box<dyn Command>> = reg.commands.into_values().collect();
    println!("{:?}", boxed);
}
