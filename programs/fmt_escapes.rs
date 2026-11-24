fn main() {
    let s = "tab\there \"quoted\" back\\slash\nnewline\r\0nul 'single'";
    println!("{:?}", s);
    println!("{}", s.len());
    let chars = ['a', '\'', '"', '\n', '\t', '\\', '\0', '\u{7f}', '\u{1b}', 'é', '日', '🦀', ' ', '\u{200b}', '\u{a0}', '\u{301}'];
    for c in chars.iter() {
        print!("{:?} ", c);
    }
    println!();
    println!("{:?}", "\u{1}\u{2}\u{1f}\u{80}\u{9f}");
    println!("{:?}", "unicode kept: ñ ü ß Ω Ж 中文 😀");
    println!("{:?}", "e\u{301}");
    println!("{:?}", "\u{301}e");
    println!("{:?}", "zero\u{200d}width");
    println!("{:?}", "\u{feff}bom");
    println!("{:?}", "line\u{2028}sep");
    println!("{:?}", String::from("owned \"string\""));
    let v = vec!["a\nb", "c\"d"];
    println!("{:?}", v);
    let t = ('x', "y", '\n');
    println!("{:?}", t);
    println!("{:#?}", ("multi\nline", '\t'));
    println!("{}", '\u{41}');
    println!("{:?}", 'A'.escape_default().to_string());
    println!("{}", "tab\t".escape_debug());
    println!("{}", "é\n".escape_default());
    println!("{}", "é\n".escape_unicode());
    println!("{:?}", Some("x\ty"));
    println!("{:?}", '\r');
    println!("{:?}", "\x7e\x7f");
}
