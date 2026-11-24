#[derive(Debug, Clone, PartialEq)]
enum Method { Get, Post }

#[derive(Debug, Clone)]
struct Request {
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    timeout_ms: u32,
    retries: u8,
    body: Option<String>,
}

impl Default for Request {
    fn default() -> Self {
        Request { method: Method::Get, url: String::from("/"), headers: vec![], timeout_ms: 3000, retries: 0, body: None }
    }
}

#[derive(Default)]
struct RequestBuilder { req: Request }

impl RequestBuilder {
    fn new(url: &str) -> Self {
        RequestBuilder { req: Request { url: url.to_string(), ..Default::default() } }
    }
    fn method(mut self, m: Method) -> Self { self.req.method = m; self }
    fn header(mut self, k: &str, v: &str) -> Self {
        self.req.headers.push((k.to_string(), v.to_string()));
        self
    }
    fn timeout(mut self, ms: u32) -> Self { self.req.timeout_ms = ms; self }
    fn retries(self, n: u8) -> Self { RequestBuilder { req: Request { retries: n, ..self.req } } }
    fn json(self, body: &str) -> Self {
        self.method(Method::Post).header("Content-Type", "application/json").body(body)
    }
    fn body(mut self, b: &str) -> Self { self.req.body = Some(b.to_string()); self }
    fn build(self) -> Result<Request, String> {
        if self.req.method == Method::Get && self.req.body.is_some() {
            return Err("GET cannot have a body".into());
        }
        if !self.req.url.starts_with('/') {
            return Err(format!("bad url {:?}", self.req.url));
        }
        Ok(self.req)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Style { bold: bool, italic: bool, size: u8, color: (u8, u8, u8) }

fn main() {
    let r1 = RequestBuilder::new("/users").header("Accept", "text/plain").build().unwrap();
    println!("{:?}", r1);
    let r2 = RequestBuilder::new("/items").json("{\"id\":1}").timeout(500).retries(3).build().unwrap();
    println!("{:?} {} retries={} timeout={}", r2.method, r2.url, r2.retries, r2.timeout_ms);
    println!("headers: {:?}", r2.headers);
    println!("body: {}", r2.body.as_deref().unwrap_or("-"));
    println!("{:?}", RequestBuilder::new("/x").body("oops").build().map(|r| r.url));
    println!("{:?}", RequestBuilder::new("nope").build().map(|r| r.url));
    let empty = RequestBuilder::default().build().unwrap();
    println!("default url={} timeout={}", empty.url, empty.timeout_ms);

    let base = Style { size: 12, ..Default::default() };
    let heading = Style { bold: true, size: 20, ..base };
    let warn = Style { color: (200, 30, 0), italic: true, ..heading };
    for (name, s) in [("base", base), ("heading", heading), ("warn", warn)] {
        println!("{:<8} {:?}", name, s);
    }
}
