use std::str;

fn main() {
    let good = vec![104u8, 195, 169, 108, 108, 111];
    match String::from_utf8(good.clone()) {
        Ok(s) => println!("ok: {} ({} bytes)", s, s.len()),
        Err(e) => println!("err: {}", e),
    }
    let bad = vec![104u8, 0xff, 105];
    match String::from_utf8(bad.clone()) {
        Ok(s) => println!("ok: {}", s),
        Err(e) => println!("err: {}", e),
    }
    let r = String::from_utf8(bad.clone());
    println!("{}", r.is_err());
    println!("{}", String::from_utf8_lossy(&bad));
    println!("{}", String::from_utf8_lossy(&good));
    match str::from_utf8(&good) {
        Ok(s) => println!("str ok {}", s),
        Err(_) => println!("str err"),
    }
    match str::from_utf8(&[0xe6, 0x97]) {
        Ok(s) => println!("str ok {}", s),
        Err(e) => println!("str err: {}", e),
    }
    match std::str::from_utf8(&bad) {
        Ok(_) => println!("no"),
        Err(e) => println!("valid up to {}", e.valid_up_to()),
    }
    let v: Vec<u8> = "round trip ✓".bytes().collect();
    let back = String::from_utf8(v).unwrap();
    println!("{}", back);
    let s = String::from("bytes");
    let owned_bytes: Vec<u8> = s.into_bytes();
    println!("{:?}", owned_bytes);
    let chars: Vec<u8> = (b'a'..=b'e').collect();
    println!("{}", String::from_utf8(chars).unwrap());
}
