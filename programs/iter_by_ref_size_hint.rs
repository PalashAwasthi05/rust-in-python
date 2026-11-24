// by_ref to partially consume, size_hint on std adapters, and ExactSizeIterator::len.
struct Lines<'a> {
    rest: &'a str,
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }
        match self.rest.find('\n') {
            Some(i) => {
                let line = &self.rest[..i];
                self.rest = &self.rest[i + 1..];
                Some(line)
            }
            None => {
                let line = self.rest;
                self.rest = "";
                Some(line)
            }
        }
    }
}

fn parse_section<'a, I: Iterator<Item = &'a str>>(it: &mut I) -> Vec<&'a str> {
    it.by_ref().take_while(|l| !l.is_empty()).collect()
}

fn main() {
    let doc = "title: notes\nauthor: me\n\nfirst body line\nsecond body line\n\ntrailer";
    let mut lines = Lines { rest: doc };
    let header = parse_section(&mut lines);
    let body = parse_section(&mut lines);
    let tail: Vec<&str> = lines.collect();
    println!("header {:?}", header);
    println!("body {:?}", body);
    println!("tail {:?}", tail);

    let v = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let mut it = v.iter();
    println!("len {}", it.len());
    let firsts: Vec<&i32> = it.by_ref().take(3).collect();
    println!("took {:?}, len now {}", firsts, it.len());
    let skipped = it.by_ref().nth(1);
    println!("nth(1) {:?}, size_hint {:?}", skipped, it.size_hint());
    let rest: i32 = it.sum();
    println!("rest sum {}", rest);

    println!("{:?}", (0..10).size_hint());
    println!("{:?}", (0..10).filter(|x| x % 2 == 0).size_hint());
    println!("{:?}", (0..10).map(|x| x + 1).size_hint());
    println!("{:?}", (0..10).skip(3).size_hint());
    println!("{:?}", (0..10).take(4).size_hint());
    println!("{:?}", (0..10).chain(0..5).size_hint());
    println!("{:?}", (0..10).zip(0..3).size_hint());
    println!("{:?}", "héllo".chars().size_hint());
    println!("{:?}", vec![1, 2].into_iter().size_hint());
    println!("{:?}", std::iter::repeat(1).size_hint());
    println!("{:?}", std::iter::once(1).size_hint());
    println!("{:?}", Lines { rest: "a\nb" }.size_hint());

    println!("{}", (0..10).len());
    println!("{}", [1, 2, 3].iter().map(|x| x * 2).len());
    println!("{}", (0..10).rev().skip(2).len());
    println!("{}", vec!['a', 'b'].iter().enumerate().len());

    let mut chars = "abc123def".chars();
    let letters: String = chars.by_ref().take_while(|c| c.is_alphabetic()).collect();
    let digits: String = chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
    let remainder: String = chars.collect();
    println!("{} {} {}", letters, digits, remainder);

    let mut numbers = 1..=10;
    let small: Vec<i32> = numbers.by_ref().filter(|n| n % 3 == 0).take(2).collect();
    println!("{:?} then {:?}", small, numbers.collect::<Vec<_>>());
}
