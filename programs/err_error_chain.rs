use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct LowLevel {
    byte: usize,
}

impl fmt::Display for LowLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unexpected byte at offset {}", self.byte)
    }
}

impl Error for LowLevel {}

#[derive(Debug)]
struct MidLevel {
    file: String,
    cause: LowLevel,
}

impl fmt::Display for MidLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse {}", self.file)
    }
}

impl Error for MidLevel {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

#[derive(Debug)]
struct TopLevel {
    inner: Box<dyn Error>,
}

impl fmt::Display for TopLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("could not load configuration")
    }
}

impl Error for TopLevel {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

fn parse(file: &str, data: &str) -> Result<usize, MidLevel> {
    match data.find('!') {
        Some(byte) => Err(MidLevel { file: file.to_string(), cause: LowLevel { byte } }),
        None => Ok(data.len()),
    }
}

fn load(file: &str, data: &str) -> Result<usize, TopLevel> {
    parse(file, data).map_err(|e| TopLevel { inner: Box::new(e) })
}

fn print_chain(err: &dyn Error) {
    println!("error: {}", err);
    let mut current = err.source();
    let mut depth = 1;
    while let Some(cause) = current {
        println!("{:width$}caused by: {}", "", cause, width = depth * 2);
        current = cause.source();
        depth += 1;
    }
}

fn main() {
    match load("ok.toml", "key = value") {
        Ok(n) => println!("loaded {} bytes", n),
        Err(e) => print_chain(&e),
    }
    match load("bad.toml", "key = va!ue") {
        Ok(n) => println!("loaded {} bytes", n),
        Err(e) => {
            print_chain(&e);
            println!("{:?}", e.source().map(|s| s.to_string()));
        }
    }
    let as_box: Box<dyn Error> = Box::new(LowLevel { byte: 9 });
    print_chain(as_box.as_ref());
    println!("{:?}", as_box);
}
