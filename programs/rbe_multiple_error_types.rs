use std::error;
use std::fmt;
use std::num::ParseIntError;

type BoxResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct EmptyVec;

impl fmt::Display for EmptyVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl error::Error for EmptyVec {}

fn double_first_boxed(vec: Vec<&str>) -> BoxResult<i32> {
    let first = vec.first().ok_or(EmptyVec)?;
    let parsed = first.parse::<i32>()?;
    Ok(2 * parsed)
}

#[derive(Debug)]
enum DoubleError {
    EmptyVec,
    Parse(ParseIntError),
}

type WrapResult<T> = std::result::Result<T, DoubleError>;

impl fmt::Display for DoubleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DoubleError::EmptyVec => write!(f, "please use a vector with at least one element"),
            DoubleError::Parse(..) => write!(f, "the provided string could not be parsed as int"),
        }
    }
}

impl error::Error for DoubleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DoubleError::EmptyVec => None,
            DoubleError::Parse(ref e) => Some(e),
        }
    }
}

impl From<ParseIntError> for DoubleError {
    fn from(err: ParseIntError) -> DoubleError { DoubleError::Parse(err) }
}

fn double_first_wrapped(vec: Vec<&str>) -> WrapResult<i32> {
    let first = vec.first().ok_or(DoubleError::EmptyVec)?;
    let parsed = first.parse::<i32>()?;
    Ok(2 * parsed)
}

fn double_first_nested(vec: Vec<&str>) -> Result<Option<i32>, ParseIntError> {
    let opt = vec.first().map(|first| first.parse::<i32>().map(|n| 2 * n));
    opt.transpose()
}

fn print_boxed(result: BoxResult<i32>) {
    match result {
        Ok(n) => println!("the first doubled is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn print_wrapped(result: WrapResult<i32>) {
    match result {
        Ok(n) => println!("the first doubled is {}", n),
        Err(e) => {
            print!("Error: {}", e);
            if let Some(source) = error::Error::source(&e) {
                print!(" (caused by: {})", source);
            }
            println!();
        }
    }
}

fn main() {
    let numbers = vec!["42", "93", "18"];
    let empty = vec![];
    let strings = vec!["tofu", "93", "18"];

    print_boxed(double_first_boxed(numbers.clone()));
    print_boxed(double_first_boxed(empty.clone()));
    print_boxed(double_first_boxed(strings.clone()));
    print_wrapped(double_first_wrapped(numbers.clone()));
    print_wrapped(double_first_wrapped(empty.clone()));
    print_wrapped(double_first_wrapped(strings.clone()));
    println!("nested: {:?} {:?} {:?}", double_first_nested(numbers), double_first_nested(empty), double_first_nested(strings).map_err(|e| e.to_string()));

    // iterating over Results
    let strings = vec!["tofu", "93", "18"];
    let numbers: Vec<_> = strings.iter().map(|s| s.parse::<i32>()).collect();
    println!("results: {:?}", numbers.iter().map(|r| r.as_ref().map_err(|e| e.to_string())).collect::<Vec<_>>());
    let filtered: Vec<_> = strings.iter().filter_map(|s| s.parse::<i32>().ok()).collect();
    println!("filter_map ok: {:?}", filtered);
    let mut errors = vec![];
    let collected: Vec<_> = strings.iter().map(|s| s.parse::<u8>()).filter_map(|r| r.map_err(|e| errors.push(e)).ok()).collect();
    println!("numbers {:?}, {} error(s) collected", collected, errors.len());
    let all: Result<Vec<_>, _> = strings.iter().map(|s| s.parse::<i32>()).collect();
    println!("collect into Result: {:?}", all.map_err(|e| e.to_string()));
    let good: Result<Vec<_>, ParseIntError> = ["1", "2"].iter().map(|s| s.parse::<i32>()).collect();
    println!("collect all good: {:?}", good);
    let (numbers, errors): (Vec<_>, Vec<_>) = strings.iter().map(|s| s.parse::<i32>()).partition(Result::is_ok);
    let numbers: Vec<_> = numbers.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(|e| e.unwrap_err().to_string()).collect();
    println!("partition numbers: {:?}", numbers);
    println!("partition errors: {:?}", errors);
}
