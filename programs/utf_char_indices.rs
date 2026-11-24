fn main() {
    let s = "aé日🦀z";
    for (i, c) in s.char_indices() {
        println!("{} {} {}", i, c, c.len_utf8());
    }
    let offsets: Vec<usize> = s.char_indices().map(|(i, _)| i).collect();
    println!("{:?}", offsets);
    let rev: Vec<(usize, char)> = s.char_indices().rev().collect();
    println!("{:?}", rev);
    let third = s.char_indices().nth(2).map(|(i, c)| (i, c));
    println!("{:?}", third);
    let text = "Füße und Hände";
    let cut = match text.char_indices().nth(6) { Some((i, _)) => &text[..i], None => text };
    println!("{}", cut);
    let last = s.chars().last().unwrap();
    let first = s.chars().next().unwrap();
    println!("{} {}", first, last);
    println!("{:?}", s.chars().rev().collect::<String>());
    println!("{:?}", s.bytes().count());
}
