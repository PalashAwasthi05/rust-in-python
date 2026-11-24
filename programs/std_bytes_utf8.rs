// Bytes, UTF-8 conversion, hex encoding and a checksum that ends in an overflow panic.
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err(format!("odd length {}", s.len()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("at {}: {}", i, e)))
        .collect()
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn run_length(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let mut n = 1;
        while chars.peek() == Some(&c) {
            chars.next();
            n += 1;
        }
        if n > 1 {
            out += &n.to_string();
        }
        out.push(c);
    }
    out
}

fn main() {
    let text = "Grüße, Jürgen ❤";
    let bytes = text.as_bytes();
    println!("{} chars, {} bytes", text.chars().count(), bytes.len());
    println!("hex {}", hex_encode(&bytes[..8]));
    println!("decode {:?}", hex_decode("48656c6c6f"));
    println!("decode odd {:?}", hex_decode("abc"));
    println!("decode bad {:?}", hex_decode("zz11"));
    let owned = bytes.to_vec();
    println!("from_utf8 {:?}", String::from_utf8(owned.clone()).map(|s| s.len()));
    println!("str from_utf8 {:?}", std::str::from_utf8(&owned[..2]));
    println!("invalid {:?}", std::str::from_utf8(&owned[..3]).is_err());
    println!("lossy {}", String::from_utf8_lossy(&[72, 105, 255, 33]));
    println!("adler32 {:08x}", adler32(b"Wikipedia"));
    println!("byte literal {:?} {}", b"hi\n", b'A');
    let upper: Vec<u8> = b"shout".iter().map(|b| b.to_ascii_uppercase()).collect();
    println!("upper {}", String::from_utf8(upper).unwrap());
    println!("is_ascii {} {}", "plain".is_ascii(), text.is_ascii());
    println!("char_indices rev {:?}", "añb".char_indices().rev().collect::<Vec<_>>());
    println!("nth char {:?} {:?}", text.chars().nth(3), text.chars().nth(99));
    println!("rle {}", run_length("aaabccddddde"));
    println!("escape {:?} {}", "tab\there ❤", "q\"uote".escape_default());
    println!("unicode escape {}", '❤'.escape_unicode());
    println!("char code {:x} {}", '❤' as u32, char::from_u32(0x2764).map_or(0, |c| c.len_utf8()));
    let words = ["zeta", "Alpha", "beta"];
    let mut sorted = words.to_vec();
    sorted.sort_by_key(|w| w.to_lowercase());
    println!("case-insensitive sort {:?}", sorted);
    sorted.sort();
    println!("byte sort {:?}", sorted);
    let joined = sorted.iter().map(|s| s.to_string()).reduce(|a, b| a + "|" + &b);
    println!("reduce {:?}", joined);
    println!("checked_next_power_of_two {:?} {:?}", 5u8.checked_next_power_of_two(), 200u8.checked_next_power_of_two());
    println!("div_ceil {} {}", 7u32.div_ceil(2), 8u32.div_ceil(2));
    println!("{:x?} {:X?}", vec![10u8, 255], (171, 205u16));
    let counts: Vec<u8> = vec![200, 40, 30];
    let total: u32 = counts.iter().map(|&c| c as u32).sum();
    println!("widened total {}", total);
    let narrow: u8 = counts.iter().sum();
    println!("unreachable {}", narrow);
}
