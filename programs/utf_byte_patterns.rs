fn classify(b: u8) -> &'static str {
    match b {
        b'0'..=b'9' => "digit",
        b'a'..=b'z' | b'A'..=b'Z' => "letter",
        b' ' | b'\t' | b'\n' => "space",
        0x80..=0xff => "non-ascii",
        _ => "other",
    }
}

fn parse_number(input: &[u8]) -> Option<(u32, &[u8])> {
    let mut value = 0u32;
    let mut used = 0;
    for &b in input {
        if !b.is_ascii_digit() {
            break;
        }
        value = value * 10 + (b - b'0') as u32;
        used += 1;
    }
    if used == 0 { None } else { Some((value, &input[used..])) }
}

fn main() {
    for b in "a1 é!".bytes() {
        println!("{:3} {:?} {}", b, b as char, classify(b));
    }
    let header: &[u8] = b"GET /index.html";
    match header {
        [b'G', b'E', b'T', b' ', rest @ ..] => println!("get {}", String::from_utf8_lossy(rest)),
        _ => println!("unknown"),
    }
    if header.starts_with(b"GET") {
        println!("starts with GET");
    }
    let mut rest: &[u8] = b"12+345+6";
    let mut total = 0;
    while let Some((n, tail)) = parse_number(rest) {
        total += n;
        rest = if tail.first() == Some(&b'+') { &tail[1..] } else { tail };
    }
    println!("total {}", total);
    println!("{}", matches!(b'q', b'a'..=b'z'));
    println!("{}", 233u8 as char);
    println!("{:?}", 0xe9u8 as char);
    let word = b"level";
    let palindrome = word.iter().eq(word.iter().rev());
    println!("{}", palindrome);
    let pos = b"hello".iter().position(|&b| b == b'l');
    println!("{:?}", pos);
    let mut counts = [0u32; 256];
    for b in "banana".bytes() {
        counts[b as usize] += 1;
    }
    println!("a={} b={} n={}", counts[b'a' as usize], counts[b'b' as usize], counts[b'n' as usize]);
    let bytes = [72u8, 105];
    println!("{}", std::str::from_utf8(&bytes).unwrap());
    println!("{:?}", b"tab\tquote\"");
    let owned: Vec<u8> = b"abc".iter().rev().cloned().collect();
    println!("{:?}", owned);
}
