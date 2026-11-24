// Markdown subset to HTML: headings, paragraphs, emphasis, code spans, links,
// fenced code blocks, block quotes, ordered/unordered lists, and horizontal rules.
use std::fmt::Write as _;

#[derive(Debug, PartialEq, Clone)]
enum Block {
    Heading(u8, String),
    Paragraph(Vec<String>),
    Code { lang: Option<String>, lines: Vec<String> },
    Quote(Vec<Block>),
    List { ordered: bool, start: u32, items: Vec<String> },
    Rule,
}

fn escape_html(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn list_marker(line: &str) -> Option<(bool, u32, &str)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        return Some((false, 1, rest));
    }
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &trimmed[digits.len()..];
    rest.strip_prefix(". ").map(|r| (true, digits.parse().unwrap(), r))
}

fn is_rule(line: &str) -> bool {
    let t: Vec<char> = line.chars().filter(|c| !c.is_whitespace()).collect();
    t.len() >= 3 && (t.iter().all(|&c| c == '-') || t.iter().all(|&c| c == '*'))
}

fn parse_blocks(lines: &[&str]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        if let Some(fence) = line.strip_prefix("```") {
            let lang = if fence.trim().is_empty() { None } else { Some(fence.trim().to_string()) };
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                body.push(lines[i].to_string());
                i += 1;
            }
            i += 1; // closing fence (or end of input)
            blocks.push(Block::Code { lang, lines: body });
            continue;
        }
        let hashes = line.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
            let text = line[hashes..].trim().trim_end_matches('#').trim_end();
            blocks.push(Block::Heading(hashes as u8, text.to_string()));
            i += 1;
            continue;
        }
        if is_rule(line) {
            blocks.push(Block::Rule);
            i += 1;
            continue;
        }
        if line.starts_with('>') {
            let mut inner = Vec::new();
            while i < lines.len() && lines[i].starts_with('>') {
                let l = &lines[i][1..];
                inner.push(l.strip_prefix(' ').unwrap_or(l));
                i += 1;
            }
            blocks.push(Block::Quote(parse_blocks(&inner)));
            continue;
        }
        if let Some((ordered, start, _)) = list_marker(line) {
            let mut items: Vec<String> = Vec::new();
            while i < lines.len() {
                match list_marker(lines[i]) {
                    Some((o, _, text)) if o == ordered => items.push(text.to_string()),
                    Some(_) => break,
                    None if lines[i].starts_with("  ") && !items.is_empty() => {
                        let last = items.last_mut().unwrap();
                        last.push(' ');
                        last.push_str(lines[i].trim());
                    }
                    None => break,
                }
                i += 1;
            }
            blocks.push(Block::List { ordered, start, items });
            continue;
        }
        let mut para = Vec::new();
        while i < lines.len() {
            let l = lines[i];
            if l.trim().is_empty() || l.starts_with('#') || l.starts_with("```") || l.starts_with('>')
                || list_marker(l).is_some() || is_rule(l)
            {
                break;
            }
            para.push(l.trim().to_string());
            i += 1;
        }
        blocks.push(Block::Paragraph(para));
    }
    blocks
}

/// Inline formatting: `code`, **strong**, *em* / _em_, [text](url), and backslash escapes.
fn inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let find = |from: usize, pat: &[char]| -> Option<usize> {
        (from..chars.len()).find(|&j| chars[j..].starts_with(pat))
    };
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if i + 1 < chars.len() => {
                out.push_str(&escape_html(&chars[i + 1].to_string()));
                i += 2;
            }
            '`' => match find(i + 1, &['`']) {
                Some(end) => {
                    let code: String = chars[i + 1..end].iter().collect();
                    write!(out, "<code>{}</code>", escape_html(&code)).unwrap();
                    i = end + 1;
                }
                None => {
                    out.push('`');
                    i += 1;
                }
            },
            '*' if chars.get(i + 1) == Some(&'*') => match find(i + 2, &['*', '*']) {
                Some(end) => {
                    let inner: String = chars[i + 2..end].iter().collect();
                    write!(out, "<strong>{}</strong>", inline(&inner)).unwrap();
                    i = end + 2;
                }
                None => {
                    out.push_str("**");
                    i += 2;
                }
            },
            '*' | '_' => match find(i + 1, &[c]) {
                Some(end) if end > i + 1 => {
                    let inner: String = chars[i + 1..end].iter().collect();
                    out += &format!("<em>{}</em>", inline(&inner));
                    i = end + 1;
                }
                _ => {
                    out.push(c);
                    i += 1;
                }
            },
            '[' => {
                let close = find(i + 1, &[']', '(']);
                let paren = close.and_then(|j| find(j + 2, &[')']).map(|k| (j, k)));
                if let Some((j, k)) = paren {
                    let label: String = chars[i + 1..j].iter().collect();
                    let url: String = chars[j + 2..k].iter().collect();
                    write!(out, "<a href=\"{}\">{}</a>", escape_html(&url), inline(&label)).unwrap();
                    i = k + 1;
                } else {
                    out.push('[');
                    i += 1;
                }
            }
            _ => {
                out.push_str(&escape_html(&c.to_string()));
                i += 1;
            }
        }
    }
    out
}

fn render(blocks: &[Block], depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    for block in blocks {
        match block {
            Block::Heading(level, text) => {
                let slug: String = text
                    .to_lowercase()
                    .chars()
                    .filter_map(|c| if c.is_alphanumeric() { Some(c) } else if c == ' ' { Some('-') } else { None })
                    .collect();
                writeln!(out, "{}<h{} id=\"{}\">{}</h{}>", pad, level, slug, inline(text), level).unwrap();
            }
            Block::Paragraph(lines) => {
                writeln!(out, "{}<p>{}</p>", pad, inline(&lines.join(" "))).unwrap();
            }
            Block::Code { lang, lines } => {
                let class = lang.as_ref().map(|l| format!(" class=\"language-{}\"", l)).unwrap_or_default();
                writeln!(out, "{}<pre><code{}>{}</code></pre>", pad, class, escape_html(&lines.join("\n"))).unwrap();
            }
            Block::Quote(inner) => {
                writeln!(out, "{}<blockquote>", pad).unwrap();
                render(inner, depth + 1, out);
                writeln!(out, "{}</blockquote>", pad).unwrap();
            }
            Block::List { ordered, start, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let attr = if *ordered && *start != 1 { format!(" start=\"{}\"", start) } else { String::new() };
                writeln!(out, "{}<{}{}>", pad, tag, attr).unwrap();
                for item in items {
                    writeln!(out, "{}  <li>{}</li>", pad, inline(item)).unwrap();
                }
                writeln!(out, "{}</{}>", pad, tag).unwrap();
            }
            Block::Rule => out.push_str(&format!("{}<hr />\n", pad)),
        }
    }
}

fn to_html(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let blocks = parse_blocks(&lines);
    let mut out = String::new();
    render(&blocks, 0, &mut out);
    out
}

const DOC: &str = "# Release *notes* for v2.0 #

This release is **big**. It has `fn main() {}` and a [link to docs](https://example.com/a?b=1&c=2).
Lines in a paragraph
are joined.

## What's new

- Faster *parsing* of `<tags>`
- A **bold _nested_ claim**
  that wraps onto two lines
* star bullets too

3. third
4. fourth item with \\*escaped stars\\*

> Quoted text with **emphasis**.
> # Heading inside quote
>
> - nested list

---

```rust
fn main() {
    println!(\"<hi> & bye\");
}
```

Unclosed `backtick and *star and [bracket
***
Trailing paragraph with 2 * 3 = 6 and snake_case_name.";

fn main() {
    let html = to_html(DOC);
    print!("{}", html);
    let blocks = parse_blocks(&DOC.lines().collect::<Vec<_>>());
    let counts = blocks.iter().fold([0usize; 6], |mut acc, b| {
        let slot = match b {
            Block::Heading(..) => 0,
            Block::Paragraph(_) => 1,
            Block::Code { .. } => 2,
            Block::Quote(_) => 3,
            Block::List { .. } => 4,
            Block::Rule => 5,
        };
        acc[slot] += 1;
        acc
    });
    println!("blocks: {} {:?}", blocks.len(), counts);
    println!("{:?}", blocks[0]);
    if let Some(Block::List { items, .. }) = blocks.iter().find(|b| matches!(b, Block::List { ordered: true, .. })) {
        println!("ordered items: {:?}", items);
    }
    println!("{}", inline("plain <text> & *more*"));
    assert_eq!(to_html(""), "");
}
