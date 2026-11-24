fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let wlen = word.chars().count();
        let clen = current.chars().count();
        if clen > 0 && clen + 1 + wlen > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn center(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n >= width {
        return s.to_string();
    }
    let left = (width - n) / 2;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(width - n - left))
}

fn main() {
    let text = "Ünïcödé text façade naïve coöperate résumé with crabs 🦀🦀 and 日本語 words mixed in for good measure";
    for line in wrap(text, 20) {
        println!("|{:<20}|", line);
    }
    for line in wrap(text, 32) {
        println!("[{}]", center(&line, 32));
    }
    println!("|{:>8}|{:^8}|{:<8}|", "é", "日本", "🦀");
    println!("|{:-^9}|", "ß");
    let s = "abcdéfgh";
    println!("{:.5}", s);
    println!("{:>10.3}", s);
}
