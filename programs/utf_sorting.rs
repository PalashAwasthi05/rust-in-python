fn main() {
    let mut words = vec!["zebra", "Äpfel", "apple", "Zoo", "éclair", "banana", "Banana"];
    words.sort();
    println!("{:?}", words);
    words.sort_by_key(|w| w.to_lowercase());
    println!("{:?}", words);
    words.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    println!("{:?}", words);
    println!("{}", "a" < "b");
    println!("{}", "Z" < "a");
    println!("{}", "é" > "z");
    println!("{:?}", "abc".cmp("abd"));
    println!("{:?}", "ab".cmp("abc"));
    println!("{:?}", "b".partial_cmp("abc"));
    let mut owned: Vec<String> = words.iter().map(|s| s.to_string()).collect();
    owned.sort_unstable();
    owned.dedup_by_key(|s| s.to_lowercase());
    println!("{:?}", owned);
    let max = owned.iter().max().unwrap();
    println!("{}", max);
    let mut cs: Vec<char> = "hello wörld".chars().collect();
    cs.sort();
    cs.dedup();
    println!("{}", cs.into_iter().collect::<String>());
    println!("{}", String::from("x") == "x");
    println!("{}", "x" == String::from("x"));
}
