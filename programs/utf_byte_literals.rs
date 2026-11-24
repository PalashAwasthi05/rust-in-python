const MAGIC: &[u8] = b"\x89PNG";
const GREETING: &str = "h\u{e9}llo";
static TABLE: [u8; 4] = *b"wxyz";

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

fn main() {
    println!("{:?} {}", MAGIC, MAGIC.len());
    println!("{} {}", GREETING, GREETING.len());
    println!("{:?}", TABLE);
    let long = b"line one \
                 line two";
    println!("{}", String::from_utf8_lossy(long));
    let rb = br#"raw "bytes" \x00"#;
    println!("{} {:?}", rb.len(), std::str::from_utf8(rb));
    println!("{}", checksum(b"hello world"));
    println!("{}", checksum("héllo".as_bytes()));
    let arr = *b"ab";
    println!("{:?}", arr);
    let nested = [b"ab", b"cd"];
    println!("{:?}", nested);
    let as_str: Vec<String> = nested.iter().map(|b| String::from_utf8(b.to_vec()).unwrap()).collect();
    println!("{:?}", as_str);
    let ch = '\u{1F600}';
    println!("{} {:?} {}", ch, ch, ch.len_utf8());
    let tab = '\t';
    println!("{:?} {}", tab, tab as u8);
    println!("{}", "\u{48}\u{49}");
    println!("{:?}", "\x7f");
    println!("{:?}", "\u{0}\u{1}");
    println!("{:?}", "\u{10ffff}");
    println!("{:?}", '\u{fffd}');
    println!("{}", r"\u{41}");
    println!("{}", r###"## "# "## "###);
    let s = "multi
line";
    println!("{:?}", s);
    let c = b'\\';
    println!("{} {}", c, b'\'' as char);
    println!("{:?}", b"\r\n\"");
    let x = 'Z' as u32 - 'A' as u32;
    println!("{}", x);
    let ascii = (32u8..127).filter(|b| b.is_ascii_punctuation()).map(|b| b as char).collect::<String>();
    println!("{}", ascii);
}
