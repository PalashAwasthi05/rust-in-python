fn main() {
    let mut s = String::from("日本語");
    s.insert_str(3, "-");
    println!("{}", s);
    let d: String = s.drain(..4).collect();
    println!("{} {}", d, s);
    s.insert(1, 'x');
    println!("never {}", s);
}
