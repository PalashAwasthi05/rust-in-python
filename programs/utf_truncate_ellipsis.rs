fn truncate_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn ellipsize(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let n = s.chars().count();
    let mut out = String::from(s);
    for _ in n..width {
        out.push('.');
    }
    out
}

fn main() {
    let items = ["short", "naïveté", "日本語のテキスト", "🦀🦀🦀🦀", "exactly10!"];
    for item in items.iter() {
        println!("{:<12} b6={:?} e6={:?}", pad_right(item, 12), truncate_bytes(item, 6), ellipsize(item, 6));
    }
    let mut s = String::from("Ünïcödé");
    while !s.is_empty() {
        let last = s.pop().unwrap();
        print!("{}({}) ", last, s.len());
    }
    println!();
    let mut owned = String::from("añoñaño");
    let cut = owned.char_indices().nth(3).map(|(i, _)| i).unwrap_or(owned.len());
    owned.truncate(cut);
    println!("{} {}", owned, owned.len());
    let ref_str: &str = &owned;
    let first_two: &str = match ref_str.char_indices().nth(2) {
        Some((i, _)) => &ref_str[..i],
        None => ref_str,
    };
    println!("{}", first_two);
    let s2 = "abc→def";
    let arrow = s2.find('→').unwrap();
    let (before, after) = (&s2[..arrow], &s2[arrow + '→'.len_utf8()..]);
    println!("{} | {}", before, after);
    println!("{}", &s2[arrow..=arrow + 2]);
}
