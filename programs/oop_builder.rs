// Builder pattern: consuming self -> Self chains and &mut self -> &mut Self chains.
#[derive(Debug, Clone, PartialEq)]
enum Method {
    Get,
    Post,
}

#[derive(Debug)]
struct Request {
    url: String,
    method: Method,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u32,
}

struct RequestBuilder {
    url: String,
    method: Method,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u32,
}

impl RequestBuilder {
    fn new(url: &str) -> Self {
        RequestBuilder {
            url: url.to_string(),
            method: Method::Get,
            headers: Vec::new(),
            body: None,
            timeout_ms: 30_000,
        }
    }

    fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    fn timeout(self, timeout_ms: u32) -> Self {
        RequestBuilder { timeout_ms, ..self }
    }

    fn build(self) -> Result<Request, String> {
        if self.method == Method::Get && self.body.is_some() {
            return Err(format!("GET {} cannot have a body", self.url));
        }
        Ok(Request {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: self.body,
            timeout_ms: self.timeout_ms,
        })
    }
}

#[derive(Default, Debug)]
struct Command {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: Option<String>,
}

impl Command {
    fn new(program: &str) -> Command {
        Command { program: program.to_string(), ..Default::default() }
    }

    fn arg(&mut self, arg: &str) -> &mut Command {
        self.args.push(arg.to_string());
        self
    }

    fn args(&mut self, args: &[&str]) -> &mut Self {
        for a in args {
            self.arg(a);
        }
        self
    }

    fn env(&mut self, key: &str, val: &str) -> &mut Self {
        self.env.push((key.into(), val.into()));
        self
    }

    fn current_dir(&mut self, dir: &str) -> &mut Self {
        self.cwd = Some(dir.to_owned());
        self
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.env {
            out.push_str(&format!("{}={} ", k, v));
        }
        out.push_str(&self.program);
        for a in self.args.iter() {
            out.push(' ');
            out.push_str(a);
        }
        if let Some(dir) = &self.cwd {
            out = format!("(cd {} && {})", dir, out);
        }
        out
    }
}

fn main() {
    let req = RequestBuilder::new("https://example.com/api")
        .method(Method::Post)
        .header("Content-Type", "application/json")
        .header("Accept", "*/*")
        .body("{\"x\": 1}")
        .timeout(500)
        .build()
        .unwrap();
    println!("{:?} {} timeout={}", req.method, req.url, req.timeout_ms);
    for (k, v) in &req.headers {
        println!("  {}: {}", k, v);
    }
    println!("  body = {:?}", req.body);
    println!("{:#?}", req);

    let bad = RequestBuilder::new("/x").body(String::from("oops")).build();
    match bad {
        Ok(_) => println!("unexpected"),
        Err(e) => println!("error: {}", e),
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build").args(&["--release", "--quiet"]).env("RUST_LOG", "debug");
    println!("{}", cmd.render());
    cmd.current_dir("/tmp").arg("--offline");
    println!("{}", cmd.render());
    println!("{:?}", cmd);

    // a builder threaded through a loop
    let mut builder = RequestBuilder::new("/loop");
    for i in 0..3 {
        builder = builder.header(&format!("X-{}", i), &i.to_string());
    }
    let r = builder.build().unwrap();
    println!("{} headers, method {:?}", r.headers.len(), r.method);
}
