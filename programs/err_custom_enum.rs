use std::error::Error;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    BadInt(ParseIntError),
    BadFloat(ParseFloatError),
    OutOfRange { key: String, value: i64, max: i64 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing key `{}`", key),
            ConfigError::BadInt(e) => write!(f, "bad integer: {}", e),
            ConfigError::BadFloat(e) => write!(f, "bad float: {}", e),
            ConfigError::OutOfRange { key, value, max } => {
                write!(f, "{} = {} exceeds {}", key, value, max)
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::BadInt(e) => Some(e),
            ConfigError::BadFloat(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::BadInt(e)
    }
}

impl From<ParseFloatError> for ConfigError {
    fn from(e: ParseFloatError) -> Self {
        ConfigError::BadFloat(e)
    }
}

fn lookup<'a>(pairs: &'a [(&'a str, &'a str)], key: &str) -> Result<&'a str, ConfigError> {
    pairs
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .ok_or_else(|| ConfigError::Missing(key.to_string()))
}

fn read_port(pairs: &[(&str, &str)]) -> Result<i64, ConfigError> {
    let port: i64 = lookup(pairs, "port")?.trim().parse()?;
    if port > 65535 {
        return Err(ConfigError::OutOfRange { key: "port".into(), value: port, max: 65535 });
    }
    Ok(port)
}

fn read_ratio(pairs: &[(&str, &str)]) -> Result<f64, ConfigError> {
    let ratio: f64 = lookup(pairs, "ratio")?.parse()?;
    Ok(ratio * 2.0)
}

fn report(label: &str, result: Result<String, ConfigError>) {
    match result {
        Ok(v) => println!("{}: ok {}", label, v),
        Err(e) => {
            println!("{}: error: {}", label, e);
            println!("  debug: {:?}", e);
            match e.source() {
                Some(src) => println!("  caused by: {}", src),
                None => println!("  no source"),
            }
        }
    }
}

fn main() {
    let good = [("port", " 8080 "), ("ratio", "0.25")];
    let bad_int = [("port", "80a"), ("ratio", "x1.5")];
    let huge = [("port", "70000")];
    let empty: [(&str, &str); 0] = [];

    report("good port", read_port(&good).map(|p| p.to_string()));
    report("good ratio", read_ratio(&good).map(|r| format!("{:.3}", r)));
    report("bad port", read_port(&bad_int).map(|p| p.to_string()));
    report("bad ratio", read_ratio(&bad_int).map(|r| r.to_string()));
    report("huge", read_port(&huge).map(|p| p.to_string()));
    report("empty", read_port(&empty).map(|p| p.to_string()));
    report("empty int", "".parse::<i32>().map(|p| p.to_string()).map_err(ConfigError::from));
    report("overflow", "300".parse::<u8>().map(|p| p.to_string()).map_err(ConfigError::from));
    report("negative", "-1".parse::<u32>().map(|p| p.to_string()).map_err(Into::into));
}
