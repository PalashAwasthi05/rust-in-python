fn main() {
    let s = "Straße";
    println!("{}", s.to_uppercase());
    println!("{}", s.to_lowercase());
    println!("{}", "ÉCOLE Ünïcödé".to_lowercase());
    println!("{}", "groß".to_ascii_uppercase());
    println!("{}", "HeLLo".eq_ignore_ascii_case("hello"));
    println!("{}", "Straße".eq_ignore_ascii_case("STRASSE"));
    let mut m = String::from("make me loud é");
    m.make_ascii_uppercase();
    println!("{}", m);
    m.make_ascii_lowercase();
    println!("{}", m);
    for c in ['ß', 'é', 'İ', 'a', '1'] {
        let up: String = c.to_uppercase().collect();
        let low: String = c.to_lowercase().collect();
        println!("{} -> {} / {} alpha={} upper={}", c, up, low, c.is_alphabetic(), c.is_uppercase());
    }
    println!("{}", 'ß'.to_uppercase().count());
    println!("{}", 'a'.to_ascii_uppercase());
    println!("{}", 'Z'.is_ascii_uppercase());
    println!("{}", "abc".is_ascii());
    println!("{}", "äbc".is_ascii());
    let title: String = "hello wörld".split(' ').map(|w| {
        let mut cs = w.chars();
        match cs.next() {
            Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
            None => String::new(),
        }
    }).collect::<Vec<_>>().join(" ");
    println!("{}", title);
}
