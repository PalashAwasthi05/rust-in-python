fn describe(s: &str) {
    println!("{:?}: bytes={} chars={} empty={}", s, s.len(), s.chars().count(), s.is_empty());
}

fn main() {
    let samples = ["hello", "café", "日本語", "🦀 crab", "", "naïve résumé"];
    for s in samples.iter() {
        describe(s);
    }
    let owned = String::from("Grüße");
    println!("owned len {} chars {}", owned.len(), owned.chars().count());
    for c in ['a', 'é', '日', '🦀', 'ß'] {
        println!("{} len_utf8={} len_utf16={} as u32={}", c, c.len_utf8(), c.len_utf16(), c as u32);
    }
    let total: usize = samples.iter().map(|s| s.len()).sum();
    println!("total bytes {}", total);
    let widest = samples.iter().max_by_key(|s| s.chars().count()).unwrap();
    println!("most chars: {}", widest);
}
