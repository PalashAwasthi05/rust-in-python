fn main() {
    let s = "Aé🦀";
    let bytes = s.as_bytes();
    println!("{:?}", bytes);
    println!("{}", bytes.len());
    for b in s.bytes() {
        print!("{:02x} ", b);
    }
    println!();
    let sum: u32 = s.bytes().map(|b| b as u32).sum();
    println!("sum {}", sum);
    let lit = b"abc";
    println!("{:?} {}", lit, lit.len());
    let arr: &[u8; 3] = b"xyz";
    println!("{:?}", arr);
    let slice: &[u8] = b"hello\n";
    println!("{:?}", slice);
    let ch = b'a';
    println!("{} {}", ch, (ch + 1) as char);
    println!("{}", b'\n');
    println!("{}", b'\x7f');
    println!("{:?}", b"\x00\xff\t");
    let count_l = b"hello world".iter().filter(|&&b| b == b'l').count();
    println!("{}", count_l);
    let upper: Vec<u8> = b"shout".iter().map(|b| b.to_ascii_uppercase()).collect();
    println!("{}", String::from_utf8(upper).unwrap());
    println!("{}", bytes[0] == b'A');
    println!("{}", s.as_bytes().starts_with(b"A"));
    let raw = br"\n";
    println!("{:?}", raw);
}
