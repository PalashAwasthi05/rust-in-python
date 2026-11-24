// String splitting and searching variants.
fn main() {
    let line = "alpha,beta,,gamma,delta";
    let parts: Vec<&str> = line.split(',').collect();
    println!("split: {:?}", parts);
    let first_two: Vec<&str> = line.splitn(3, ',').collect();
    println!("splitn: {:?}", first_two);
    let from_back: Vec<&str> = line.rsplit(',').collect();
    println!("rsplit: {:?}", from_back);
    let rn: Vec<&str> = line.rsplitn(2, ',').collect();
    println!("rsplitn: {:?}", rn);
    let terminated: Vec<&str> = "a;b;c;".split_terminator(';').collect();
    println!("split_terminator: {:?}", terminated);
    if let Some((key, value)) = "name=Ferris=crab".split_once('=') {
        println!("split_once: {} -> {}", key, value);
    }
    if let Some((key, value)) = "name=Ferris=crab".rsplit_once('=') {
        println!("rsplit_once: {} -> {}", key, value);
    }
    println!("none: {:?}", "plain".split_once(':'));
    let ws: Vec<&str> = "  lots   of\tspace\n here ".split_whitespace().collect();
    println!("whitespace: {:?}", ws);
    let by_str: Vec<&str> = "one::two::three".split("::").collect();
    println!("by str: {:?}", by_str);
    let by_closure: Vec<&str> = "a1b22c333d".split(|c: char| c.is_ascii_digit()).collect();
    println!("by closure: {:?}", by_closure);
    let by_chars: Vec<&str> = "x-y_z w".split(&['-', '_', ' '][..]).collect();
    println!("by char set: {:?}", by_chars);
    let inclusive: Vec<&str> = "line one\nline two\nlast".split_inclusive('\n').collect();
    println!("split_inclusive: {:?}", inclusive);
    let lines: Vec<&str> = "first\r\nsecond\n\nfourth".lines().collect();
    println!("lines: {:?}", lines);

    let text = "the quick brown fox jumps over the lazy dog";
    println!("find the: {:?}", text.find("the"));
    println!("rfind the: {:?}", text.rfind("the"));
    println!("find vowel: {:?}", text.find(|c: char| "aeiou".contains(c)));
    println!("rfind vowel: {:?}", text.rfind(|c: char| "aeiou".contains(c)));
    println!("find z: {:?}", text.find('z'));
    println!("find missing: {:?}", text.find("cat"));
    let positions: Vec<usize> = text.match_indices("o").map(|(i, _)| i).collect();
    println!("o positions: {:?}", positions);
    println!("matches of 'the': {}", text.matches("the").count());
    println!("contains fox: {}, starts: {}, ends: {}", text.contains("fox"), text.starts_with("the"), text.ends_with("dog"));

    let padded = "xxhello worldxx";
    println!("trim_matches: {}", padded.trim_matches('x'));
    println!("trim_start_matches: {}", padded.trim_start_matches("xx"));
    println!("trim_end_matches: {}", padded.trim_end_matches(|c| c == 'x'));
    println!("strip_prefix: {:?}", "v1.2.3".strip_prefix('v'));
    println!("strip_prefix none: {:?}", "1.2.3".strip_prefix("v"));
    println!("strip_suffix: {:?}", "file.rs".strip_suffix(".rs"));
    let version: Vec<u32> = "v1.20.3".trim_start_matches('v').split('.').map(|p| p.parse().unwrap()).collect();
    println!("version: {:?}", version);
    println!("replace: {}", text.replace("the", "a"));
    println!("replacen: {}", text.replacen('o', "0", 2));
    let csv = "id,name\n1,ann\n2,bob\n";
    let rows: Vec<Vec<&str>> = csv.lines().skip(1).map(|l| l.split(',').collect()).collect();
    println!("rows: {:?}", rows);
    let words = "Hello World";
    println!("char_indices last: {:?}", words.char_indices().last());
    println!("is_char_boundary: {} {}", "é".is_char_boundary(1), "é".is_char_boundary(2));
    println!("get: {:?} {:?}", words.get(0..5), words.get(3..100));
    println!("slice: {} {} {}", &words[6..], &words[..5], &words[2..4]);
    let (left, right) = words.split_at(5);
    println!("split_at: {:?} {:?}", left, right);
    println!("eq_ignore_ascii_case: {}", "HeLLo".eq_ignore_ascii_case("hello"));
    println!("cmp: {:?} {}", "apple".cmp("banana"), "b" > "abc");
}
