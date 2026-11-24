use std::collections::BTreeMap;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug)]
enum ParseError {
    UnexpectedEnd,
    UnexpectedChar(char, usize),
    BadNumber(String),
    BadEscape(char),
    TrailingInput(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::UnexpectedChar(c, pos) => write!(f, "unexpected '{}' at {}", c, pos),
            ParseError::BadNumber(s) => write!(f, "bad number {:?}", s),
            ParseError::BadEscape(c) => write!(f, "bad escape \\{}", c),
            ParseError::TrailingInput(pos) => write!(f, "trailing input at {}", pos),
        }
    }
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser { chars: src.chars().peekable(), pos: 0 }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, want: char) -> Result<(), ParseError> {
        match self.bump() {
            Some(c) if c == want => Ok(()),
            Some(c) => Err(ParseError::UnexpectedChar(c, self.pos - 1)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_document(&mut self) -> Result<Value, ParseError> {
        let v = self.parse_value()?;
        self.skip_ws();
        if self.chars.peek().is_some() {
            return Err(ParseError::TrailingInput(self.pos));
        }
        Ok(v)
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.skip_ws();
        match self.chars.peek().copied() {
            None => Err(ParseError::UnexpectedEnd),
            Some('n') => self.parse_keyword("null", Value::Null),
            Some('t') => self.parse_keyword("true", Value::Bool(true)),
            Some('f') => self.parse_keyword("false", Value::Bool(false)),
            Some('"') => Ok(Value::Str(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(ParseError::UnexpectedChar(c, self.pos)),
        }
    }

    fn parse_keyword(&mut self, word: &str, value: Value) -> Result<Value, ParseError> {
        for expected in word.chars() {
            self.expect(expected)?;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let mut text = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
        text.parse::<f64>()
            .map(Value::Number)
            .map_err(|_| ParseError::BadNumber(text.clone()))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(ParseError::UnexpectedEnd),
                Some('"') => return Ok(out),
                Some('\\') => {
                    let esc = self.bump().ok_or(ParseError::UnexpectedEnd)?;
                    let decoded = match esc {
                        'n' => '\n',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'u' => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let h = self.bump().ok_or(ParseError::UnexpectedEnd)?;
                                let d = h.to_digit(16).ok_or(ParseError::BadEscape(h))?;
                                code = code * 16 + d;
                            }
                            char::from_u32(code).unwrap_or('?')
                        }
                        other => return Err(ParseError::BadEscape(other)),
                    };
                    out.push(decoded);
                }
                Some(c) => out.push(c),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.chars.peek() == Some(&']') {
            self.bump();
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => break,
                Some(c) => return Err(ParseError::UnexpectedChar(c, self.pos - 1)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }
        Ok(Value::Array(items))
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        self.expect('{')?;
        let mut map = BTreeMap::new();
        self.skip_ws();
        if let Some(&'}') = self.chars.peek() {
            self.bump();
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_ws();
            match self.bump() {
                Some(',') => {}
                Some('}') => return Ok(Value::Object(map)),
                Some(c) => return Err(ParseError::UnexpectedChar(c, self.pos - 1)),
                None => return Err(ParseError::UnexpectedEnd),
            }
        }
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn pretty(v: &Value, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&format_number(*n)),
        Value::Str(s) => out.push_str(&escape(s)),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad);
                out.push_str("  ");
                pretty(item, indent + 1, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            out.push_str("{\n");
            let n = map.len();
            for (i, (k, val)) in map.iter().enumerate() {
                out.push_str(&format!("{}  {}: ", pad, escape(k)));
                pretty(val, indent + 1, out);
                out.push_str(if i + 1 < n { ",\n" } else { "\n" });
            }
            out.push_str(&pad);
            out.push('}');
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        pretty(self, 0, &mut s);
        f.write_str(&s)
    }
}

impl Value {
    fn get(&self, path: &str) -> Option<&Value> {
        let mut cur = self;
        for part in path.split('.').filter(|p| !p.is_empty()) {
            cur = match cur {
                Value::Object(map) => map.get(part)?,
                Value::Array(items) => items.get(part.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(cur)
    }

    fn depth(&self) -> usize {
        match self {
            Value::Array(items) => 1 + items.iter().map(|v| v.depth()).max().unwrap_or(0),
            Value::Object(map) => 1 + map.values().map(Value::depth).max().unwrap_or(0),
            _ => 0,
        }
    }

    fn sum_numbers(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Array(items) => items.iter().map(|v| v.sum_numbers()).sum(),
            Value::Object(map) => map.values().map(|v| v.sum_numbers()).sum(),
            _ => 0.0,
        }
    }
}

fn main() {
    let doc = r#"
    {
        "name": "inventory",
        "version": 3,
        "ratio": 0.75,
        "tags": ["tools", "garden", "a\"quoted\""],
        "owner": {"first": "Ada", "last": "Lovelace", "active": true, "manager": null},
        "items": [
            {"id": 1, "price": 9.5, "qty": 4},
            {"id": 2, "price": 12, "qty": 0},
            {"id": 3, "price": -1.25e1, "qty": 2}
        ],
        "empty": {}, "none": [],
        "unicode": "caf\u00e9 \t tab"
    }"#;

    let value = match Parser::new(doc).parse_document() {
        Ok(v) => v,
        Err(e) => {
            println!("error: {}", e);
            return;
        }
    };
    println!("{}", value);
    println!("depth = {}", value.depth());
    println!("sum of numbers = {}", value.sum_numbers());

    for path in ["owner.first", "items.1.price", "tags.2", "items.7", "version.x", "empty"] {
        match value.get(path) {
            Some(v) => println!("{:<14} => {}", path, v),
            None => println!("{:<14} => <missing>", path),
        }
    }

    if let Some(Value::Array(items)) = value.get("items") {
        let total: f64 = items
            .iter()
            .filter_map(|it| match (it.get("price"), it.get("qty")) {
                (Some(Value::Number(p)), Some(Value::Number(q))) => Some(p * q),
                _ => None,
            })
            .sum();
        println!("inventory value = {:.2}", total);
    }

    let roundtrip = Parser::new(&value.to_string()).parse_document().unwrap();
    println!("roundtrip equal: {}", roundtrip == value);

    let bad_inputs = ["[1, 2,", "{\"a\" 1}", "tru", "[1] x", "\"bad \\q\"", "-", "@"];
    for src in bad_inputs.iter() {
        match Parser::new(src).parse_document() {
            Ok(v) => println!("{:>12} parsed?? {:?}", src, v),
            Err(e) => println!("{:>12} -> {}", src, e),
        }
    }

    let small = Parser::new("[1, [true, null], {\"k\": \"v\"}]").parse_document().unwrap();
    println!("{:?}", small);
}
