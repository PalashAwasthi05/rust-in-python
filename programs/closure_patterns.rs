fn main() { let n = "a,b;c".split(|c| c == ',' || c == ';').count(); let t = "xxhixx".trim_matches('x'); println!("{n} {t} {}", "a-b".replace(|c: char| !c.is_alphanumeric(), "_")); }
