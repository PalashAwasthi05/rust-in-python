fn main() {
    let s = String::from("Hello, World");
    println!("{} {} {}", s.to_uppercase(), s.to_lowercase(), s.replace("l", "L"));
    let parts: Vec<&str> = s.split(", ").collect();
    println!("{:?} {}", parts, s.starts_with("He"));
    let t = "  pad  ".trim();
    let mut r: String = s.chars().rev().collect();
    r.push_str("!");
    println!("[{t}] {r} {}", format!("{:>8}|{:<5}|{:^7}", "a", "b", "c"));
    let n: Vec<String> = vec![1, 2, 3].iter().map(|x| x.to_string()).collect();
    println!("{}", n.join("-"));
    let csv = "name,age\nann,30\nbob,25\n";
    for line in csv.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        let age: u32 = fields[1].parse().unwrap();
        println!("{} is {} next year {}", fields[0], age, age + 1);
    }
    println!("{:?}", "a1b2c3".split(|c: char| c.is_ascii_digit()).collect::<Vec<_>>());
    println!("{:?} {:?}", "key=value".split_once('='), "abc".strip_prefix("a"));
    println!("{:?}", "one two  three".split_whitespace().collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", "42".parse::<i32>(), "x".parse::<i32>(), "-1".parse::<u8>());
    println!("{:?} {:?}", "3.5".parse::<f64>(), "300".parse::<u8>().is_err());
    let word = "héllo";
    println!("{} {} {:?}", word.len(), word.chars().count(), word.char_indices().nth(2));
    println!("{}", word.chars().filter(|c| c.is_alphabetic()).count());
    let c = 'a';
    println!("{} {} {:?} {}", c.is_alphabetic(), c.to_ascii_uppercase(), '7'.to_digit(10), ('a' as u8 + 2) as char);
    let mut st = String::new();
    for (i, w) in ["x", "y", "z"].iter().enumerate() {
        if i > 0 { st.push(','); }
        st += w;
    }
    println!("{st} {}", st.contains('y'));
    let num = 2i32.pow(10);
    println!("{} {} {:?} {:?} {}", num, (-7i32).rem_euclid(3), 200u8.checked_add(100), 5u32.checked_sub(6), 250u8.wrapping_add(10));
    println!("{} {} {}", 10i64.abs(), (-3i32).signum(), u64::MAX.count_ones());
    let x = 2.0f64;
    println!("{:.4} {} {} {}", x.sqrt(), x.powi(3), (2.5f64).round(), (-2.5f64).floor());
    println!("{:.3} {} {}", std::f64::consts::PI, 1.0f64.max(f64::NAN), 9.0f64.sqrt() as i32);
    println!("{}", "abc".repeat(3));
    println!("{:?}", "Mary had a little lamb".split(' ').map(|w| w.len()).collect::<Vec<usize>>());
    println!("{}", "  x ".trim_start().len());
    let bytes = "hi".as_bytes();
    println!("{:?} {}", bytes, String::from_utf8(vec![104, 105]).unwrap());
}
