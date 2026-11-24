struct Tokens<'a> { src: &'a str, pos: usize }

impl<'a> Tokens<'a> {
    fn new(src: &'a str) -> Self { Tokens { src, pos: 0 } }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let rest = &self.src[self.pos..];
        let start = rest.find(|c: char| !c.is_whitespace())?;
        let rest = &rest[start..];
        let len = if rest.starts_with(|c: char| c.is_alphanumeric()) {
            rest.find(|c: char| !c.is_alphanumeric()).unwrap_or(rest.len())
        } else {
            rest.chars().next().unwrap().len_utf8()
        };
        let tok = &rest[..len];
        self.pos += start + len;
        Some(tok)
    }
}

struct Lines { n: u32 }
impl Iterator for Lines {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.n += 1;
        if self.n > 10 { None } else { Some(format!("line{}", self.n)) }
    }
}

fn main() {
    let src = "header: v1 ; body a b c ; footer z";
    let mut toks = Tokens::new(src);
    let head: Vec<&str> = toks.by_ref().take_while(|t| *t != ";").collect();
    let body: Vec<&str> = toks.by_ref().take_while(|t| *t != ";").collect();
    let rest: Vec<&str> = toks.collect();
    println!("head {:?}\nbody {:?}\nrest {:?}", head, body, rest);

    let mut lines = Lines { n: 0 };
    let first_three: Vec<String> = lines.by_ref().take(3).collect();
    let skipped = lines.by_ref().skip(2).next();
    let count_rest = lines.by_ref().count();
    println!("first {:?}, after skip {:?}, remaining {}", first_three, skipped, count_rest);
    println!("exhausted: {:?}", lines.next());

    let mut nums = (1..=20).into_iter();
    let small_sum: i32 = nums.by_ref().take_while(|&n| n < 5).sum();
    let evens: Vec<i32> = nums.by_ref().filter(|n| n % 2 == 0).take(3).collect();
    let next = nums.next();
    println!("small sum {} (5 was consumed), evens {:?}, next {:?}", small_sum, evens, next);

    let mut toks = Tokens::new("let total = price * (qty + 1);");
    let mut idents = Vec::new();
    for t in toks.by_ref() {
        if t == "=" { break; }
        idents.push(t);
    }
    let expr: String = toks.collect::<Vec<_>>().join(" ");
    println!("binding {:?} expr {:?}", idents, expr);

    let mut chars = "key=value=more".chars();
    let key: String = chars.by_ref().take_while(|&c| c != '=').collect();
    let value: String = chars.collect();
    println!("key {:?} value {:?}", key, value);
    let counts: Vec<usize> = ["a b", "  ", "x+y*z", "héllo wörld"].iter().map(|s| Tokens::new(s).count()).collect();
    println!("token counts: {:?}", counts);
}
