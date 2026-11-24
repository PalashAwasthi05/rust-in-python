// A JSON-ish value type with recursive parsing, nested destructuring, and if-let chains.
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Null,
    Bool(bool),
    Num(i64),
    Str(String),
    List(Vec<Value>),
    Obj(BTreeMap<String, Value>),
}

struct Parser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(b' ' | b'\n') = self.peek() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: u8) -> Result<(), String> {
        self.skip_ws();
        match self.peek() {
            Some(x) if x == c => { self.pos += 1; Ok(()) }
            Some(x) => Err(format!("expected '{}' at {}, found '{}'", c as char, self.pos, x as char)),
            None => Err(format!("expected '{}' at end", c as char)),
        }
    }

    fn word(&mut self, w: &str, v: Value) -> Result<Value, String> {
        if self.chars[self.pos..].starts_with(w.as_bytes()) {
            self.pos += w.len();
            Ok(v)
        } else {
            Err(format!("bad literal at {}", self.pos))
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_ws();
        match self.peek() {
            None => Err("unexpected end".to_string()),
            Some(b'n') => self.word("null", Value::Null),
            Some(b't') => self.word("true", Value::Bool(true)),
            Some(b'f') => self.word("false", Value::Bool(false)),
            Some(c @ (b'-' | b'0'..=b'9')) => {
                let start = self.pos;
                self.pos += 1;
                while let Some(b'0'..=b'9') = self.peek() { self.pos += 1; }
                let text = std::str::from_utf8(&self.chars[start..self.pos]).unwrap();
                text.parse().map(Value::Num).map_err(|_| format!("bad number starting with {}", c as char))
            }
            Some(b'"') => {
                self.pos += 1;
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if c == b'"' { break; }
                    self.pos += 1;
                }
                let s = String::from_utf8(self.chars[start..self.pos].to_vec()).unwrap();
                self.expect(b'"')?;
                Ok(Value::Str(s))
            }
            Some(b'[') => {
                self.pos += 1;
                let mut items = vec![];
                loop {
                    self.skip_ws();
                    if let Some(b']') = self.peek() { self.pos += 1; break; }
                    if !items.is_empty() { self.expect(b',')?; }
                    items.push(self.value()?);
                }
                Ok(Value::List(items))
            }
            Some(b'{') => {
                self.pos += 1;
                let mut map = BTreeMap::new();
                loop {
                    self.skip_ws();
                    if let Some(b'}') = self.peek() { self.pos += 1; break; }
                    if !map.is_empty() { self.expect(b',')?; }
                    let Value::Str(key) = self.value()? else { return Err("object key must be a string".into()) };
                    self.expect(b':')?;
                    let v = self.value()?;
                    map.insert(key, v);
                }
                Ok(Value::Obj(map))
            }
            Some(other) => Err(format!("unexpected '{}'", other as char)),
        }
    }
}

fn summarize(v: &Value) -> String {
    if let Value::Obj(m) = v {
        if let (Some(Value::Str(name)), Some(Value::List(tags))) = (m.get("name"), m.get("tags")) {
            return format!("{} with {} tags", name, tags.len());
        } else if let Some(Value::Num(id)) = m.get("id") {
            return format!("object #{}", id);
        }
        format!("object with keys {:?}", m.keys().collect::<Vec<_>>())
    } else if let Value::List(items) = v {
        let nums: i64 = items.iter().filter_map(|x| if let Value::Num(n) = x { Some(*n) } else { None }).sum();
        format!("list of {} (num sum {})", items.len(), nums)
    } else if let Value::Bool(true) | Value::Null = v {
        "truthy-or-null".to_string()
    } else {
        format!("scalar {:?}", v)
    }
}

fn main() {
    let docs = [
        r#"{"name": "widget", "tags": ["a", "b"], "id": 7}"#,
        r#"{"id": 42, "ok": true}"#,
        r#"{"x": null}"#,
        "[1, 2, -3, \"s\", [4]]",
        "null",
        "false",
        "-17",
        "[1, 2",
        "{\"k\" 1}",
        "{3: 4}",
        "@",
    ];
    for d in docs.iter() {
        let mut p = Parser { chars: d.as_bytes(), pos: 0 };
        match p.value() {
            Ok(v) => println!("{:<48} {}", d, summarize(&v)),
            Err(e) => println!("{:<48} ERROR {}", d, e),
        }
    }
}
