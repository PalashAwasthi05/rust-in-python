fn main() {
    let a = b'a';
    for i in 0..5u8 {
        print!("{}", (a + i) as char);
    }
    println!();
    println!("{:?}", char::from_u32(0x1F980));
    println!("{:?}", char::from_u32(0xD800));
    println!("{:?}", char::from_digit(7, 10));
    println!("{:?}", char::from_digit(11, 16));
    println!("{:?}", '7'.to_digit(10));
    println!("{:?}", 'f'.to_digit(16));
    println!("{:?}", 'z'.to_digit(10));
    println!("{}", char::from(65u8));
    let c = 'é';
    println!("{} {}", c as u32, c as u8);
    let next = char::from_u32(c as u32 + 1).unwrap();
    println!("{}", next);
    let alphabet: String = ('a'..='z').collect();
    println!("{}", alphabet);
    let greek: String = ('α'..='ε').collect();
    println!("{}", greek);
    println!("{}", ('a'..'e').count());
    println!("{}", (b'z' - b'a') as usize);
    let shifted: String = "HAL".chars().map(|c| ((c as u8) + 1) as char).collect();
    println!("{}", shifted);
    println!("{}", 'A' < 'a');
    println!("{}", u32::from('🦀'));
    println!("{}", '9'.is_ascii_digit());
    println!("{}", '٣'.is_numeric());
    println!("{}", ' '.is_whitespace());
    println!("{}", '\u{3000}'.is_whitespace());
    println!("{}", 'x'.is_alphanumeric());
    let digits: u32 = "a1b2c3".chars().filter_map(|c| c.to_digit(10)).sum();
    println!("{}", digits);
    println!("{}", std::char::MAX as u32);
}
