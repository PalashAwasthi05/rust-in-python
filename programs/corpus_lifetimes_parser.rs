#[derive(Debug)]
struct Header<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Debug)]
struct Request<'a> {
    method: &'a str,
    path: &'a str,
    query: Vec<(&'a str, &'a str)>,
    headers: Vec<Header<'a>>,
    body: &'a str,
}

struct Parser<'a> {
    lines: std::str::Lines<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self { Parser { lines: input.lines() } }

    fn parse(mut self, input: &'a str) -> Option<Request<'a>> {
        let first = self.lines.next()?;
        let mut parts = first.split_whitespace();
        let method = parts.next()?;
        let target = parts.next()?;
        let (path, qs) = match target.split_once('?') {
            Some((p, q)) => (p, q),
            None => (target, ""),
        };
        let query = qs.split('&').filter(|s| !s.is_empty())
            .map(|kv| kv.split_once('=').unwrap_or((kv, ""))).collect();
        let mut headers = Vec::new();
        for line in self.lines.by_ref() {
            if line.is_empty() { break; }
            let (name, value) = line.split_once(':')?;
            headers.push(Header { name: name.trim(), value: value.trim() });
        }
        let body = match input.find("\n\n") {
            Some(i) => &input[i + 2..],
            None => "",
        };
        Some(Request { method, path, query, headers, body })
    }
}

impl<'a> Request<'a> {
    fn header(&self, name: &str) -> Option<&'a str> {
        self.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value)
    }
}

fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn main() {
    let raw = String::from("POST /api/items?page=2&sort=name&debug HTTP/1.1\nHost: example.test\nContent-Type: application/json\nX-Trace: abc123\n\n{\"name\": \"widget\"}");
    let req = Parser::new(&raw).parse(&raw).expect("parse failed");
    println!("method={} path={}", req.method, req.path);
    for (k, v) in &req.query {
        println!("  query {} = {:?}", k, v);
    }
    for h in &req.headers {
        println!("  header {:<14} {}", h.name, h.value);
    }
    println!("content-type: {:?}", req.header("content-type"));
    println!("missing: {:?}", req.header("Accept"));
    println!("body: {}", req.body);
    let longer = longest(req.method, req.path);
    println!("longer of method/path: {}", longer);
    println!("bad input: {:?}", Parser::new("GARBAGE").parse("GARBAGE").map(|r| r.method));
}
