use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    MissingKey(String),
    BadNumber { key: String, source: ParseIntError },
    OutOfRange { key: String, value: i64, min: i64, max: i64 },
    Syntax(usize),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingKey(k) => write!(f, "missing key '{}'", k),
            ConfigError::BadNumber { key, source } => write!(f, "key '{}' is not a number ({})", key, source),
            ConfigError::OutOfRange { key, value, min, max } => {
                write!(f, "key '{}' = {} outside {}..={}", key, value, min, max)
            }
            ConfigError::Syntax(line) => write!(f, "syntax error on line {}", line),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<(String, ParseIntError)> for ConfigError {
    fn from((key, source): (String, ParseIntError)) -> Self {
        ConfigError::BadNumber { key, source }
    }
}

#[derive(Debug)]
struct Config { port: i64, workers: i64 }

fn lookup<'a>(pairs: &'a [(String, String)], key: &str) -> Result<&'a str, ConfigError> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).ok_or_else(|| ConfigError::MissingKey(key.to_string()))
}

fn number(pairs: &[(String, String)], key: &str, min: i64, max: i64) -> Result<i64, ConfigError> {
    let raw = lookup(pairs, key)?;
    let value: i64 = raw.trim().parse().map_err(|e| (key.to_string(), e))?;
    if value < min || value > max {
        return Err(ConfigError::OutOfRange { key: key.into(), value, min, max });
    }
    Ok(value)
}

fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut pairs = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let (k, v) = line.split_once('=').ok_or(ConfigError::Syntax(i + 1))?;
        pairs.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(Config { port: number(&pairs, "port", 1, 65535)?, workers: number(&pairs, "workers", 1, 64)? })
}

fn run(text: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let cfg = parse(text)?;
    let doubled: i64 = "2".parse::<i64>()? * cfg.workers;
    Ok(doubled)
}

fn main() {
    let inputs = [
        "port = 8080\nworkers = 4",
        "# comment\nport=80",
        "port = eighty\nworkers = 2",
        "port = 70000\nworkers = 2",
        "port 8080",
    ];
    for (i, text) in inputs.iter().enumerate() {
        match parse(text) {
            Ok(cfg) => println!("{}: ok {:?}", i, cfg),
            Err(e) => println!("{}: error: {}", i, e),
        }
    }
    println!("run: {:?}", run(inputs[0]).map_err(|e| e.to_string()));
    println!("run: {:?}", run(inputs[1]).map_err(|e| e.to_string()));
}
