#[derive(Debug, Clone, Copy, PartialEq)]
enum Token { Num(i64), Op(char), LParen, RParen }

fn lex(s: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut num: Option<i64> = None;
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            num = Some(num.unwrap_or(0) * 10 + d as i64);
            continue;
        }
        if let Some(n) = num.take() { out.push(Token::Num(n)); }
        match c {
            '+' | '-' | '*' | '/' => out.push(Token::Op(c)),
            '(' => out.push(Token::LParen),
            ')' => out.push(Token::RParen),
            ' ' => {}
            _ => unreachable!("lexer given unexpected char {:?}", c),
        }
    }
    if let Some(n) = num { out.push(Token::Num(n)); }
    out
}

fn apply(op: char, a: i64, b: i64) -> i64 {
    match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => {
            assert!(b != 0, "division by zero");
            a / b
        }
        _ => unreachable!(),
    }
}

fn rpn(tokens: &[&str]) -> i64 {
    let mut stack: Vec<i64> = Vec::new();
    for t in tokens {
        match t.parse::<i64>() {
            Ok(n) => stack.push(n),
            Err(_) => {
                let b = stack.pop().expect("stack underflow");
                let a = stack.pop().expect("stack underflow");
                stack.push(apply(t.chars().next().unwrap(), a, b));
            }
        }
    }
    assert_eq!(stack.len(), 1, "malformed expression {:?}", tokens);
    stack[0]
}

fn main() {
    let toks = lex("12 + (3*45) - 6");
    println!("{:?}", toks);
    let ops = toks.iter().filter(|t| matches!(t, Token::Op(_))).count();
    let big_nums = toks.iter().filter(|t| matches!(t, Token::Num(n) if *n > 10)).count();
    let parens = toks.iter().filter(|t| matches!(t, Token::LParen | Token::RParen)).count();
    println!("ops={} big_nums={} parens={}", ops, big_nums, parens);
    assert_eq!(ops, 3);
    assert_ne!(big_nums, 0);
    assert!(parens % 2 == 0, "unbalanced");

    let exprs: [&[&str]; 3] = [&["3", "4", "+", "2", "*"], &["10", "2", "/", "3", "-"], &["2", "3", "4", "*", "+"]];
    for e in exprs.iter() {
        let v = rpn(e);
        println!("{} = {}", e.join(" "), v);
    }
    assert_eq!(rpn(&["6", "7", "*"]), 42, "the answer");
    debug_assert!(rpn(&["1", "1", "+"]) == 2);

    let grade = |score: u32| match score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        0..=69 => 'F',
        _ => unreachable!("score {} out of range", score),
    };
    let grades: String = [95, 83, 70, 12, 100].iter().map(|&s| grade(s)).collect();
    println!("grades: {}", grades);
    assert!(matches!(grades.as_bytes(), [b'A', .., b'A']));
    let opt = Some(3);
    println!("is small some: {}", matches!(opt, Some(x) if x < 5));
    println!("all assertions passed");
}
