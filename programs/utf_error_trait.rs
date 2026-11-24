use std::convert::TryFrom;
use std::error::Error;

fn decode_name(raw: &[u8]) -> Result<String, Box<dyn Error>> {
    let text = std::str::from_utf8(raw)?;
    let first = text.chars().next().ok_or("empty name")?;
    let code = u8::try_from(first)?;
    Ok(format!("{} starts with byte {}", text, code))
}

fn to_char(code: u32) -> Result<char, Box<dyn Error>> {
    Ok(char::try_from(code)?)
}

fn owned(bytes: Vec<u8>) -> Result<usize, Box<dyn Error>> {
    let s = String::from_utf8(bytes)?;
    Ok(s.chars().count())
}

fn main() {
    let inputs: [&[u8]; 4] = [b"alice", "\u{e9}lodie".as_bytes(), "\u{65e5}\u{672c}".as_bytes(), b"bad\xff"];
    for raw in inputs.iter() {
        match decode_name(raw) {
            Ok(msg) => println!("ok: {}", msg),
            Err(e) => println!("error: {}", e),
        }
    }
    match decode_name(b"") {
        Ok(msg) => println!("ok: {}", msg),
        Err(e) => println!("error: {}", e),
    }
    for code in [0x41, 0x1F980, 0xDFFF, 0x110000] {
        match to_char(code) {
            Ok(c) => println!("{:#x} -> {:?}", code, c),
            Err(e) => println!("{:#x} -> error: {}", code, e),
        }
    }
    println!("{:?}", owned(vec![0xce, 0xbb]).ok());
    match owned(vec![0xce]) {
        Ok(n) => println!("{}", n),
        Err(e) => println!("error: {}", e),
    }
    let e = char::try_from(0xD800u32).unwrap_err();
    println!("{} | {:?}", e, e);
    let f = u8::try_from('\u{65e5}').unwrap_err();
    println!("{} | {:?}", f, f);
    println!("{:?}", u16::try_from('\u{65e5}'));
    let t: String = 'y'.into();
    println!("{}", t);
}
