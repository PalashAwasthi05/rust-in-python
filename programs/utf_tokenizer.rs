#[derive(Debug, PartialEq)]
enum Token<'a> {
    Ident(&'a str),
    Number(i64),
    Str(String),
    Punct(char),
}

fn tokenize(src: &str) -> Result<Vec<(usize, Token<'_>)>, String> {
    let mut tokens = Vec::new();
    let mut iter = src.char_indices().peekable();
    while let Some(&(start, c)) = iter.peek() {
        if c.is_whitespace() {
            iter.next();
        } else if c.is_alphabetic() || c == '_' {
            let mut end = start;
            while let Some(&(i, ch)) = iter.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    end = i + ch.len_utf8();
                    iter.next();
                } else {
                    break;
                }
            }
            tokens.push((start, Token::Ident(&src[start..end])));
        } else if c.is_ascii_digit() {
            let mut value = 0i64;
            while let Some(&(_, ch)) = iter.peek() {
                match ch.to_digit(10) {
                    Some(d) => {
                        value = value * 10 + d as i64;
                        iter.next();
                    }
                    None => break,
                }
            }
            tokens.push((start, Token::Number(value)));
        } else if c == '"' {
            iter.next();
            let mut text = String::new();
            let mut closed = false;
            while let Some((_, ch)) = iter.next() {
                match ch {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => match iter.next() {
                        Some((_, 'n')) => text.push('\n'),
                        Some((_, 'u')) => text.push('\u{FFFD}'),
                        Some((_, other)) => text.push(other),
                        None => break,
                    },
                    other => text.push(other),
                }
            }
            if !closed {
                return Err(format!("unterminated string at byte {}", start));
            }
            tokens.push((start, Token::Str(text)));
        } else if "+-*/=(){};,→".contains(c) {
            iter.next();
            tokens.push((start, Token::Punct(c)));
        } else {
            return Err(format!("unexpected {:?} at byte {} (char {})", c, start, src[..start].chars().count()));
        }
    }
    Ok(tokens)
}

fn main() {
    let sources = [
        "let größe = 42 + x_1;",
        "fn café(ñ) → \"héllo\\n🦀\"",
        "日本 = \"unterminated",
        "a = b ¤ c",
        "λx → x * 2",
    ];
    for src in sources {
        println!("source {:?} ({} bytes, {} chars)", src, src.len(), src.chars().count());
        match tokenize(src) {
            Ok(tokens) => {
                for (offset, token) in &tokens {
                    println!("  {:>2} {:?}", offset, token);
                }
            }
            Err(e) => println!("  error: {}", e),
        }
    }
}
