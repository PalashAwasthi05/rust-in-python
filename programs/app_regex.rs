// A tiny backtracking regex engine: literals, '.', character classes with ranges and
// negation, anchors, groups with alternation, and the quantifiers * + ? {m,n}.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Char(char),
    Any,
    Class { negated: bool, ranges: Vec<(char, char)> },
    Start,
    End,
    Group(Box<Node>, usize),
    Alt(Vec<Node>),
    Concat(Vec<Node>),
    Repeat { node: Box<Node>, min: usize, max: Option<usize>, greedy: bool },
}

#[derive(Debug)]
struct RegexError {
    pos: usize,
    msg: &'static str,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at position {}", self.msg, self.pos)
    }
}

struct Compiler<'s> {
    src: &'s [char],
    pos: usize,
    groups: usize,
}

impl<'s> Compiler<'s> {
    fn err<T>(&self, msg: &'static str) -> Result<T, RegexError> {
        Err(RegexError { pos: self.pos, msg })
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn alternation(&mut self) -> Result<Node, RegexError> {
        let mut branches = vec![self.concat()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.concat()?);
        }
        Ok(if branches.len() == 1 { branches.pop().unwrap() } else { Node::Alt(branches) })
    }

    fn concat(&mut self) -> Result<Node, RegexError> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.atom()?;
            items.push(self.quantified(atom)?);
        }
        Ok(Node::Concat(items))
    }

    fn number(&mut self) -> Option<usize> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        let s: String = self.src[start..self.pos].iter().collect();
        s.parse().ok()
    }

    fn quantified(&mut self, atom: Node) -> Result<Node, RegexError> {
        let (min, max) = match self.peek() {
            Some('*') => (0, None),
            Some('+') => (1, None),
            Some('?') => (0, Some(1)),
            Some('{') => {
                self.pos += 1;
                let min = match self.number() {
                    Some(n) => n,
                    None => return self.err("expected number in {}"),
                };
                let max = if self.peek() == Some(',') {
                    self.pos += 1;
                    self.number()
                } else {
                    Some(min)
                };
                if self.peek() != Some('}') {
                    return self.err("unclosed {");
                }
                if let Some(m) = max {
                    if m < min {
                        return self.err("bad repetition range");
                    }
                }
                (min, max)
            }
            _ => return Ok(atom),
        };
        if matches!(atom, Node::Start | Node::End) {
            return self.err("nothing to repeat");
        }
        self.pos += 1;
        let greedy = if self.peek() == Some('?') {
            self.pos += 1;
            false
        } else {
            true
        };
        Ok(Node::Repeat { node: Box::new(atom), min, max, greedy })
    }

    fn atom(&mut self) -> Result<Node, RegexError> {
        let c = self.peek().unwrap();
        self.pos += 1;
        Ok(match c {
            '.' => Node::Any,
            '^' => Node::Start,
            '$' => Node::End,
            '(' => {
                self.groups += 1;
                let index = self.groups;
                let inner = self.alternation()?;
                if self.peek() != Some(')') {
                    return self.err("missing )");
                }
                self.pos += 1;
                Node::Group(Box::new(inner), index)
            }
            '[' => self.class()?,
            '\\' => {
                let e = match self.peek() {
                    Some(e) => e,
                    None => return self.err("trailing backslash"),
                };
                self.pos += 1;
                match e {
                    'd' => Node::Class { negated: false, ranges: vec![('0', '9')] },
                    'w' => Node::Class { negated: false, ranges: vec![('a', 'z'), ('A', 'Z'), ('0', '9'), ('_', '_')] },
                    's' => Node::Class { negated: false, ranges: vec![(' ', ' '), ('\t', '\t'), ('\n', '\n')] },
                    other => Node::Char(other),
                }
            }
            '*' | '+' | '?' => return Err(RegexError { pos: self.pos - 1, msg: "nothing to repeat" }),
            other => Node::Char(other),
        })
    }

    fn class(&mut self) -> Result<Node, RegexError> {
        let negated = self.peek() == Some('^');
        if negated {
            self.pos += 1;
        }
        let mut ranges = vec![];
        loop {
            let lo = match self.peek() {
                None => return self.err("unclosed ["),
                Some(']') if !ranges.is_empty() => break,
                Some(c) => c,
            };
            self.pos += 1;
            if self.peek() == Some('-') && self.src.get(self.pos + 1).map_or(false, |&c| c != ']') {
                let hi = self.src[self.pos + 1];
                self.pos += 2;
                ranges.push((lo, hi));
            } else {
                ranges.push((lo, lo));
            }
        }
        self.pos += 1;
        Ok(Node::Class { negated, ranges })
    }
}

struct Regex {
    root: Node,
    groups: usize,
}

type Caps = Vec<Option<(usize, usize)>>;

impl Regex {
    fn new(pattern: &str) -> Result<Regex, RegexError> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut c = Compiler { src: &chars, pos: 0, groups: 0 };
        let root = c.alternation()?;
        if c.pos < chars.len() {
            return Err(RegexError { pos: c.pos, msg: "unmatched )" });
        }
        Ok(Regex { root, groups: c.groups })
    }

    /// Match `node` at `i`, then call `k` with the new position; backtrack if `k` fails.
    fn m(&self, node: &Node, text: &[char], i: usize, caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        match node {
            Node::Char(c) => i < text.len() && text[i] == *c && k(i + 1, caps),
            Node::Any => i < text.len() && text[i] != '\n' && k(i + 1, caps),
            Node::Class { negated, ranges } => {
                i < text.len() && (ranges.iter().any(|&(lo, hi)| lo <= text[i] && text[i] <= hi) != *negated) && k(i + 1, caps)
            }
            Node::Start => i == 0 && k(i, caps),
            Node::End => i == text.len() && k(i, caps),
            Node::Group(inner, idx) => {
                let idx = *idx;
                self.m(inner, text, i, caps, &mut |j, caps: &mut Caps| {
                    let saved = caps[idx];
                    caps[idx] = Some((i, j));
                    if k(j, caps) {
                        return true;
                    }
                    caps[idx] = saved;
                    false
                })
            }
            Node::Alt(branches) => branches.iter().any(|b| self.m(b, text, i, caps, k)),
            Node::Concat(items) => self.seq(items, text, i, caps, k),
            Node::Repeat { node, min, max, greedy } => self.rep(node, *min, *max, *greedy, 0, text, i, caps, k),
        }
    }

    fn seq(&self, items: &[Node], text: &[char], i: usize, caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        match items.split_first() {
            None => k(i, caps),
            Some((first, rest)) => self.m(first, text, i, caps, &mut |j, caps: &mut Caps| self.seq(rest, text, j, caps, k)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn rep(&self, node: &Node, min: usize, max: Option<usize>, greedy: bool, count: usize, text: &[char], i: usize,
           caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool) -> bool {
        let can_more = max.map_or(true, |m| count < m);
        let mut more = |caps: &mut Caps, k: &mut dyn FnMut(usize, &mut Caps) -> bool| {
            can_more && self.m(node, text, i, caps, &mut |j, caps: &mut Caps| {
                j != i && self.rep(node, min, max, greedy, count + 1, text, j, caps, k)
            })
        };
        if count < min {
            return more(caps, k);
        }
        if greedy {
            more(caps, k) || k(i, caps)
        } else {
            k(i, caps) || more(caps, k)
        }
    }

    fn find(&self, s: &str) -> Option<(String, Vec<Option<String>>)> {
        let text: Vec<char> = s.chars().collect();
        for start in 0..=text.len() {
            let mut caps: Caps = vec![None; self.groups + 1];
            let mut end = None;
            if self.m(&self.root, &text, start, &mut caps, &mut |j, _| {
                end = Some(j);
                true
            }) {
                let slice = |(a, b): (usize, usize)| text[a..b].iter().collect::<String>();
                let groups = caps[1..].iter().map(|c| c.map(slice)).collect();
                return Some((slice((start, end.unwrap())), groups));
            }
        }
        None
    }
}

fn main() {
    let cases: &[(&str, &[&str])] = &[
        ("a.c", &["abc", "xxa-cxx", "ac"]),
        ("^h(e|a)llo$", &["hello", "hallo", "hullo", "hello!"]),
        ("colou?r", &["color", "the colour red", "colouur"]),
        ("[a-c]+[^0-9]?\\d{2,3}", &["zzabba!123", "c7", "ab12345", "x99"]),
        ("(\\w+)@(\\w+)\\.com", &["mail bob@example.com now", "no at sign"]),
        ("<.+>", &["<a><b>"]),
        ("<.+?>", &["<a><b>"]),
        ("(a|ab)(c|bcd)(d*)", &["abcd"]),
        ("x*", &["", "yyy"]),
        ("(\\d+)-(\\d+)?", &["10-", "10-20"]),
        ("([A-Z][a-z]*\\s?){2}", &["say Hello World ok"]),
        ("(a*)*b", &["aaab", "aaa"]),
    ];
    for (pattern, inputs) in cases {
        let re = Regex::new(pattern).unwrap();
        for input in inputs.iter() {
            match re.find(input) {
                Some((whole, groups)) if groups.is_empty() => println!("{:<24} {:<26} => {:?}", pattern, format!("{:?}", input), whole),
                Some((whole, groups)) => println!("{:<24} {:<26} => {:?} {:?}", pattern, format!("{:?}", input), whole, groups),
                None => println!("{:<24} {:<26} => no match", pattern, format!("{:?}", input)),
            }
        }
    }
    for bad in ["a{3", "*a", "(ab", "ab)", "[abc", "a{5,2}", "^*", "a\\"] {
        match Regex::new(bad) {
            Ok(re) => println!("{:?} compiled?? {:?}", bad, re.root),
            Err(e) => println!("{:?}: {}", bad, e),
        }
    }
    let re = Regex::new("a(b|c)*").unwrap();
    println!("groups={} root={:?}", re.groups, re.root);
}
