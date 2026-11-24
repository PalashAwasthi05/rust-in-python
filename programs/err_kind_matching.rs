use std::num::IntErrorKind;

fn classify(s: &str) -> String {
    match s.parse::<i8>() {
        Ok(n) => format!("ok {}", n),
        Err(e) => match e.kind() {
            IntErrorKind::Empty => "empty".to_string(),
            IntErrorKind::InvalidDigit => format!("invalid digit in {:?}", s),
            IntErrorKind::PosOverflow => "too big".to_string(),
            IntErrorKind::NegOverflow => "too small".to_string(),
            _ => format!("other: {}", e),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Low,
    High,
}

#[derive(Debug)]
enum OpError {
    Timeout(u32),
    Denied { user: String, severity: Severity },
    Io(std::io::ErrorKind),
}

fn handle(e: &OpError) -> &'static str {
    match e {
        OpError::Timeout(ms) if *ms > 1000 => "give up",
        OpError::Timeout(_) => "retry",
        OpError::Denied { severity: Severity::High, .. } => "alert",
        OpError::Denied { .. } => "log",
        OpError::Io(std::io::ErrorKind::NotFound) => "create it",
        OpError::Io(_) => "fail",
    }
}

fn main() {
    for s in ["12", "", "-", "+", "1_0", "128", "-129", "-128", "  5"] {
        println!("{:>6?} => {}", s, classify(s));
    }
    println!("{:?}", "".parse::<u64>());
    println!("{:?}", "x".parse::<u64>());
    println!("{:?}", "99999999999999999999".parse::<u64>());
    println!("{:?}", "1.5e".parse::<f64>());
    println!("{:?}", "".parse::<f32>());
    println!("{:?}", "yes".parse::<bool>());
    println!("{:?}", "ab".parse::<char>());
    if let Err(e) = "".parse::<bool>() {
        println!("{}", e);
    }
    if let Err(e) = "".parse::<char>() {
        println!("{}", e);
    }
    if let Err(e) = "ab".parse::<char>() {
        println!("{}", e);
    }
    let errs = vec![
        OpError::Timeout(50),
        OpError::Timeout(5000),
        OpError::Denied { user: "eve".into(), severity: Severity::High },
        OpError::Denied { user: "bob".into(), severity: Severity::Low },
        OpError::Io(std::io::ErrorKind::NotFound),
        OpError::Io(std::io::ErrorKind::InvalidData),
    ];
    for e in &errs {
        println!("{:?} -> {}", e, handle(e));
    }
    let e = u8::try_from(300i32).unwrap_err();
    println!("{} / {:?}", e, e);
    let e = i8::try_from(-200i64);
    println!("{:?}", e);
}
