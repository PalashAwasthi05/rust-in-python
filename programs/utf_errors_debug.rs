use std::str::Utf8Error;
use std::string::FromUtf8Error;

fn decode(bytes: Vec<u8>) -> Result<String, FromUtf8Error> {
    String::from_utf8(bytes)
}

fn check(bytes: &[u8]) -> Result<&str, Utf8Error> {
    let s = std::str::from_utf8(bytes)?;
    Ok(s)
}

fn main() {
    let err = decode(vec![0x61, 0xc3]).unwrap_err();
    println!("{}", err);
    println!("{:?}", err.utf8_error());
    println!("{:?}", err.utf8_error().error_len());
    println!("{:?}", err.as_bytes());
    let recovered = err.into_bytes();
    println!("{:?}", recovered);
    println!("{:?}", check(b"ok"));
    println!("{:?}", check(&[0x61, 0x80, 0x62]));
    for case in [&[0xf0u8, 0x9f, 0xa6][..], &[0xed, 0xa0, 0x80], &[0xc0, 0xaf], &[0x41, 0xe2, 0x82, 0x41]] {
        match std::str::from_utf8(case) {
            Ok(s) => println!("ok {}", s),
            Err(e) => println!("{} | up_to={} len={:?}", e, e.valid_up_to(), e.error_len()),
        }
    }
    println!("{}", String::from_utf8_lossy(&[0x41, 0xe2, 0x82, 0x41, 0xff, 0xfe]));
    let parsed: Result<i32, _> = "١٢".parse::<i32>();
    println!("{:?}", parsed.is_err());
    let r: Result<String, String> = String::from_utf8(vec![0xff]).map_err(|e| e.to_string());
    println!("{:?}", r);
    let text = "héllo";
    println!("{:?}", text.get(1..2));
    println!("{:?}", text.get(1..3));
    println!("{:?}", text.get(..=2));
}
