// Structs that borrow from collections, iterators over borrowed data.
#[derive(Debug)]
struct Record {
    id: u32,
    tags: Vec<String>,
    amount: f64,
}

struct Report<'a> {
    title: &'a str,
    rows: Vec<&'a Record>,
}

impl<'a> Report<'a> {
    fn new(title: &'a str, all: &'a [Record], min: f64) -> Report<'a> {
        Report { title, rows: all.iter().filter(|r| r.amount >= min).collect() }
    }

    fn total(&self) -> f64 {
        self.rows.iter().map(|r| r.amount).sum()
    }

    fn largest(&self) -> Option<&'a Record> {
        self.rows.iter().copied().max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap())
    }

    fn tags(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.rows.iter().flat_map(|r| r.tags.iter().map(|s| s.as_str()))
    }
}

struct Window<'a, T> {
    data: &'a [T],
    size: usize,
    pos: usize,
}

impl<'a, T> Iterator for Window<'a, T> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<&'a [T]> {
        if self.pos + self.size > self.data.len() {
            return None;
        }
        let w = &self.data[self.pos..self.pos + self.size];
        self.pos += 1;
        Some(w)
    }
}

fn split_pair<'a>(s: &'a str, sep: char) -> (&'a str, &'a str) {
    match s.find(sep) {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

fn pick<'a, 'b>(primary: &'a str, _fallback: &'b str) -> &'a str {
    primary
}

fn main() {
    let records = vec![
        Record { id: 1, tags: vec!["food".into()], amount: 12.5 },
        Record { id: 2, tags: vec!["rent".into(), "fixed".into()], amount: 900.0 },
        Record { id: 3, tags: vec![], amount: 3.25 },
        Record { id: 4, tags: vec!["fun".into()], amount: 45.0 },
    ];
    let title = String::from("Big spends");
    let report = Report::new(&title, &records, 10.0);
    println!("{}: {} rows, total {:.2}", report.title, report.rows.len(), report.total());
    let big = report.largest();
    println!("largest: {:?}", big.map(|r| r.id));
    let tags: Vec<&str> = report.tags().collect();
    println!("tags: {:?}", tags);
    drop(report);
    println!("first after report: {:?}", records[0]);

    let nums = [1, 4, 2, 8, 5, 7];
    let w = Window { data: &nums, size: 3, pos: 0 };
    let sums: Vec<i32> = w.map(|s| s.iter().sum()).collect();
    println!("{:?}", sums);
    let best = Window { data: &nums, size: 2, pos: 0 }.max_by_key(|s| s[0] * s[1]).unwrap();
    println!("best pair {:?}", best);

    let line = String::from("key=value=more");
    let (k, v) = split_pair(&line, '=');
    println!("{} | {}", k, v);
    let (k2, v2) = split_pair(v, '=');
    println!("{} | {}", k2, v2);
    let result;
    {
        let fallback = String::from("fb");
        result = pick(&line, &fallback);
    }
    println!("{}", result);

    let mut words: Vec<&str> = line.split('=').collect();
    words.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    println!("{:?}", words);
    let lens: Vec<(usize, &&str)> = words.iter().map(|w| (w.len(), w)).collect();
    println!("{:?}", lens);
}
