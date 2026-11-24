fn hexdump(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

fn main() {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"Hi ");
    buf.extend_from_slice("ñ".as_bytes());
    buf.push(b'!');
    println!("{}", hexdump(&buf));
    let s = String::from_utf8(buf.clone()).unwrap();
    println!("{} {}", s, s.len());
    let text = std::str::from_utf8(&buf[0..2]).unwrap();
    println!("{}", text);
    println!("{}", std::str::from_utf8(&buf[3..4]).is_err());
    let mut bytes = "hello".to_string().into_bytes();
    bytes.reverse();
    println!("{}", String::from_utf8(bytes).unwrap());
    let ascii: Vec<u8> = (0..26).map(|i| b'A' + i).collect();
    println!("{}", String::from_utf8_lossy(&ascii));
    let mut v = b"lower".to_vec();
    v.make_ascii_uppercase();
    println!("{:?}", std::str::from_utf8(&v));
    println!("{}", b"abc" == &[97, 98, 99]);
    let counts = "mississippi".bytes().fold([0usize; 26], |mut acc, b| {
        acc[(b - b'a') as usize] += 1;
        acc
    });
    for (i, n) in counts.iter().enumerate() {
        if *n > 0 {
            print!("{}{} ", (b'a' + i as u8) as char, n);
        }
    }
    println!();
    let s2 = String::from("héllo");
    let b0 = s2.as_bytes()[1];
    println!("{} {:#x} {:08b}", b0, b0, b0);
    println!("{}", b"\xe2\x9c\x93".len());
    println!("{}", String::from_utf8(b"\xe2\x9c\x93".to_vec()).unwrap());
    println!("{}", s2.bytes().rev().map(|b| b.to_string()).collect::<Vec<_>>().join(","));
    let lossy = String::from_utf8_lossy(b"ok\xF0\x90\x80end");
    println!("{} {}", lossy, lossy.len());
    println!("{}", s2.as_bytes().is_ascii());
    println!("{:?}", "AbC".bytes().map(|b| b.is_ascii_uppercase()).collect::<Vec<_>>());
}
