fn main() {
    let a = "\x41\x42\x43";
    println!("{}", a);
    let crab = "\u{1F980}";
    println!("{} {}", crab, crab.len());
    let e = "\u{e9}";
    println!("{} {}", e, e == "é");
    let raw = r"C:\path\no\escape";
    println!("{}", raw);
    let hashes = r#"She said "hi" \n"#;
    println!("{}", hashes);
    let more = r##"a "# b"##;
    println!("{}", more);
    println!("{:?}", "tab\there\nnewline \"q\" \\ \0");
    println!("{:?}", 'é');
    println!("{:?}", '\'');
    println!("{:?}", "\u{7f}\u{1b}");
    let multi = "line one \
                 line two";
    println!("{}", multi);
    let c = '\u{263A}';
    println!("{} {}", c, c as u32);
    println!("{}", '\x41');
    println!("{:?}", "é\u{301}");
    println!("{}", "e\u{301}".chars().count());
}
