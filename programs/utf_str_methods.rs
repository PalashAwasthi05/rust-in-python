fn main() {
    let s = "ünï, cödé;  wörld\r\nnext→line";
    println!("{:?}", s.split_inclusive(',').collect::<Vec<_>>());
    println!("{:?}", "a;b;".rsplit_terminator(';').collect::<Vec<_>>());
    println!("{:?}", "ababc".trim_start_matches("ab"));
    println!("{:?}", "éclair".strip_prefix('é'));
    let needle = String::from("cödé");
    println!("{} {:?}", s.contains(&needle), s.find(&needle));
    println!("{:?}", s.replace(&needle, &String::from("X")));
    println!("{:?}", s.splitn(3, char::is_whitespace).collect::<Vec<_>>());
    println!("{:?}", s.split_whitespace().rev().collect::<Vec<_>>());
    println!("{:?}", s.char_indices().skip_while(|(_, c)| *c != ',').map(|(i, _)| i).next());
    let a = String::from("Zürich");
    let b = String::from("Zug");
    println!("{} {} {:?}", a < b, a.as_str() > "Z", a.cmp(&b));
    println!("{:?} {:?}", s.chars().max(), s.chars().filter(|c| c.is_alphabetic()).min());
    println!("{:?}", "é".parse::<char>());
    match "ab".parse::<char>() {
        Ok(c) => println!("{}", c),
        Err(e) => println!("err {}", e),
    }
    match "".parse::<char>() {
        Ok(c) => println!("{}", c),
        Err(e) => println!("err {}", e),
    }
    println!("{:?}", s.split("→").map(str::len).collect::<Vec<_>>());
    println!("{:?}", s.lines().map(|l| l.chars().count()).collect::<Vec<_>>());
    let joined = ["α", "β", "γ"].join("·");
    println!("{} {}", joined, joined.len());
    let concat = ["日", "本"].concat();
    println!("{}", concat);
    let mut words: Vec<String> = s.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()).map(String::from).collect();
    words.sort_by(|x, y| y.cmp(x));
    println!("{:?}", words);
    println!("{:>6}|{:<6}|{:^6}|", 'é', "日本", "🦀");
    println!("{:?}", format!("{:5}", "é").len());
    println!("{:>8?}", "é");
    let c = 'ü';
    println!("{}", c.is_lowercase() && !c.is_ascii());
    let caps = s.chars().filter(char::is_ascii_lowercase).count();
    println!("{}", caps);
    println!("{:?}", "ß".chars().flat_map(char::to_uppercase).collect::<String>());
    println!("{}", s.chars().rev().skip(4).take(4).collect::<String>());
    println!("{}", "abc".char_indices().map(|(i, c)| c.to_string().repeat(i + 1)).collect::<Vec<_>>().join("-"));
}
