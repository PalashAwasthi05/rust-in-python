fn shout(s: &mut String) {
    s.make_ascii_uppercase();
    s.push('!');
}

fn dedupe_spaces(s: &mut String) {
    let mut prev_space = false;
    s.retain(|c| {
        let keep = !(c == ' ' && prev_space);
        prev_space = c == ' ';
        keep
    });
}

fn rot13(s: &mut String) {
    let mut bytes = std::mem::take(s).into_bytes();
    for b in bytes.iter_mut() {
        if b.is_ascii_lowercase() {
            *b = (*b - b'a' + 13) % 26 + b'a';
        } else if b.is_ascii_uppercase() {
            *b = (*b - b'A' + 13) % 26 + b'A';
        }
    }
    *s = String::from_utf8(bytes).unwrap();
}

fn main() {
    let mut s = String::from("héllo   wörld  ");
    shout(&mut s);
    println!("{:?}", s);
    dedupe_spaces(&mut s);
    println!("{:?}", s);
    s.as_mut_str().make_ascii_lowercase();
    println!("{:?}", s);
    s.extend(["→", "end"]);
    s.extend(vec!["!".to_string()]);
    s += "ç";
    let other = String::from("ø");
    s += &other;
    println!("{} {}", s, s.len());
    let tail = s.split_off(s.find('→').unwrap());
    println!("{:?} | {:?}", s, tail);
    let back: String = s.drain(..).rev().collect();
    println!("{:?} {:?}", back, s);
    let mut r = String::from("Hello, Wörld!");
    rot13(&mut r);
    println!("{}", r);
    rot13(&mut r);
    println!("{}", r);
    let mut a = String::from("α");
    let mut b = String::from("β");
    std::mem::swap(&mut a, &mut b);
    println!("{}{}", a, b);
    let mut v = String::new();
    for (i, w) in "un deux trois".split(' ').enumerate() {
        if i > 0 {
            v.insert(0, '|');
        }
        v.insert_str(0, w);
    }
    println!("{}", v);
    let mut t = String::from("añb");
    let idx = t.char_indices().nth(1).unwrap().0;
    let removed = t.remove(idx);
    println!("{} {} {}", removed, t, t.len());
    t.insert(idx, 'Ñ');
    println!("{}", t);
    let mut count = 0;
    let mut u = String::from("a-b-c-d");
    u.retain(|c| { if c == '-' { count += 1; count > 2 } else { true } });
    println!("{} {}", u, count);
}
