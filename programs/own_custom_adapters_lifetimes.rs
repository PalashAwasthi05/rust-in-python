struct MapIter<I, F> {
    inner: I,
    f: F,
    calls: usize,
}

impl<I: Iterator, B, F: FnMut(I::Item) -> B> Iterator for MapIter<I, F> {
    type Item = B;
    fn next(&mut self) -> Option<B> {
        let x = self.inner.next()?;
        self.calls += 1;
        Some((self.f)(x))
    }
}

fn my_map<I: Iterator, B, F: FnMut(I::Item) -> B>(inner: I, f: F) -> MapIter<I, F> {
    MapIter { inner, f, calls: 0 }
}

trait WordSource<'a> {
    fn next_word(&mut self) -> Option<&'a str>;
}

struct Splitter<'a> {
    rest: &'a str,
}

impl<'a> WordSource<'a> for Splitter<'a> {
    fn next_word(&mut self) -> Option<&'a str> {
        let trimmed = self.rest.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed.find(' ').unwrap_or(trimmed.len());
        let (word, rest) = trimmed.split_at(end);
        self.rest = rest;
        Some(word)
    }
}

fn collect_words<'a>(src: &mut dyn WordSource<'a>) -> Vec<&'a str> {
    let mut out = Vec::new();
    while let Some(w) = src.next_word() {
        out.push(w);
    }
    out
}

struct Retry<F: FnMut() -> Result<u32, String>> {
    op: F,
    attempts: u32,
}

impl<F: FnMut() -> Result<u32, String>> Retry<F> {
    fn run(&mut self, max: u32) -> Result<u32, Vec<String>> {
        let mut errors = Vec::new();
        while self.attempts < max {
            self.attempts += 1;
            match (self.op)() {
                Ok(v) => return Ok(v),
                Err(e) => errors.push(e),
            }
        }
        Err(errors)
    }
}

fn main() {
    let offset = 100;
    let mut seen = Vec::new();
    let mut it = my_map(vec![1, 2, 3].into_iter(), |x| {
        seen.push(x);
        x + offset
    });
    let a = it.next();
    let b = it.next();
    println!("{:?} {:?} calls={}", a, b, it.calls);
    let rest: Vec<i32> = it.collect();
    println!("{:?} seen={:?}", rest, seen);

    let text = String::from("  alpha beta   gamma ");
    let words;
    {
        let mut sp = Splitter { rest: &text };
        words = collect_words(&mut sp);
    }
    println!("{:?}", words);

    let mut n = 0;
    let mut r = Retry {
        op: || {
            n += 1;
            if n < 3 { Err(format!("fail {}", n)) } else { Ok(n * 10) }
        },
        attempts: 0,
    };
    println!("{:?}", r.run(5));
    println!("attempts {}", r.attempts);
    let mut never = Retry { op: || Err("nope".to_string()), attempts: 0 };
    println!("{:?}", never.run(2));
    println!("n = {}", n);
}
