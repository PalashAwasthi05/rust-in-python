fn extract<'a>(s: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = s.find(open)? + open.len();
    let end = start + s[start..].find(close)?;
    Some(&s[start..end])
}

fn main() {
    let s = "naïve café ☕ menu";
    let i = s.find("café").unwrap();
    println!("{} {}", i, &s[i..]);
    let j = s.find('☕').unwrap();
    println!("{} {:?}", j, &s[..j]);
    println!("{:?}", s.find('x'));
    println!("{:?}", s.rfind('a'));
    println!("{:?}", s.find(char::is_whitespace));
    println!("{:?}", s.find(|c: char| c as u32 > 0x2000));
    println!("{:?}", s.rfind(&['é', 'ï'][..]));
    println!("{:?}", extract("tag <日本> end", "<", ">"));
    println!("{:?}", extract("«quoted» text", "«", "»"));
    println!("{:?}", extract("nothing here", "<", ">"));
    let path = "/home/ünser/docs/file.tar.gz";
    let slash = path.rfind('/').unwrap();
    let name = &path[slash + 1..];
    let dot = name.find('.').unwrap();
    println!("{} {} {}", &path[..slash], &name[..dot], &name[dot + 1..]);
    let mut pos = 0;
    let hay = "ab🦀ab🦀ab";
    while let Some(k) = hay[pos..].find("ab") {
        print!("{} ", pos + k);
        pos += k + 2;
    }
    println!();
    let pos_char = hay.char_indices().position(|(_, c)| c == '🦀');
    println!("{:?}", pos_char);
}
