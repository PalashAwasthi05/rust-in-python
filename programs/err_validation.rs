use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum FieldError {
    Empty(&'static str),
    TooLong { field: &'static str, max: usize, got: usize },
    NotNumeric(&'static str, String),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(field) => write!(f, "{} must not be empty", field),
            Self::TooLong { field, max, got } => {
                write!(f, "{} is {} chars, max {}", field, got, max)
            }
            Self::NotNumeric(field, v) => write!(f, "{} must be numeric, got {:?}", field, v),
        }
    }
}

impl std::error::Error for FieldError {}

#[derive(Debug)]
struct Form {
    name: String,
    age: u8,
}

fn non_empty(field: &'static str, v: &str) -> Result<String, FieldError> {
    if v.is_empty() { Err(FieldError::Empty(field)) } else { Ok(v.to_string()) }
}

fn max_len(field: &'static str, max: usize) -> impl Fn(String) -> Result<String, FieldError> {
    move |v| {
        let got = v.chars().count();
        if got > max { Err(FieldError::TooLong { field, max, got }) } else { Ok(v) }
    }
}

fn parse_form(name: &str, age: &str) -> Result<Form, FieldError> {
    let name = non_empty("name", name).and_then(max_len("name", 8))?;
    let age = non_empty("age", age)?
        .parse::<u8>()
        .map_err(|_| FieldError::NotNumeric("age", age.to_string()))?;
    Ok(Form { name, age })
}

fn validate_all(rows: &[(&str, &str)]) -> (Vec<Form>, Vec<(usize, FieldError)>) {
    let mut ok = Vec::new();
    let mut errs = Vec::new();
    for (i, (n, a)) in rows.iter().enumerate() {
        match parse_form(n, a) {
            Ok(form) => ok.push(form),
            Err(e) => errs.push((i, e)),
        }
    }
    (ok, errs)
}

fn main() {
    let rows = [
        ("Zoë", "33"),
        ("", "20"),
        ("Bartholomew", "40"),
        ("Kim", ""),
        ("Lee", "old"),
        ("Max", "300"),
        ("Ivy", "7"),
    ];
    let (ok, errs) = validate_all(&rows);
    for form in &ok {
        println!("ok: {:?}", form);
    }
    for (i, e) in &errs {
        println!("row {}: {}", i, e);
    }
    let first_err: Option<&FieldError> = errs.iter().map(|(_, e)| e).next();
    println!("{:?}", first_err);
    println!("{}", errs.iter().any(|(_, e)| matches!(e, FieldError::TooLong { .. })));
    let boxed: Result<Form, Box<dyn std::error::Error>> = parse_form("", "1").map_err(|e| e.into());
    if let Err(e) = boxed {
        println!("boxed: {}", e);
    }
}
