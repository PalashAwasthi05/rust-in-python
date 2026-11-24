fn main() {
    let s = "héllo wörld";
    println!("{}", &s[0..1]);
    println!("{}", &s[0..3]);
    println!("{}", &s[3..]);
    println!("{}", &s[..7]);
    for i in 0..=s.len() {
        print!("{}", if s.is_char_boundary(i) { 'Y' } else { 'n' });
    }
    println!();
    let jp = "日本語テキスト";
    println!("{}", &jp[3..9]);
    println!("{:?}", jp.get(0..3));
    println!("{:?}", jp.get(0..2));
    println!("{:?}", jp.get(100..));
    let crab = String::from("I 🦀 Rust");
    let start = crab.find('🦀').unwrap();
    let end = start + '🦀'.len_utf8();
    println!("start {} end {} -> {} | {}", start, end, &crab[start..end], &crab[end..]);
    let (left, right) = s.split_at(6);
    println!("[{}] [{}]", left, right);
    println!("{}", s.is_char_boundary(2));
}
