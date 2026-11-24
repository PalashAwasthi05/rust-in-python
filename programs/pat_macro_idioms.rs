// Macro idioms: replace_expr counting, recursive sum, trait impls per type, nested repetitions, hygiene.
use std::fmt::Display;

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => { $sub };
}

macro_rules! count_tts {
    ($($tts:tt)*) => { <[()]>::len(&[$(replace_expr!($tts ())),*]) };
}

macro_rules! sum {
    () => { 0 };
    ($x:expr $(, $rest:expr)*) => { $x + sum!($($rest),*) };
}

trait Describe {
    fn describe(&self) -> String;
}

macro_rules! impl_describe {
    ($($t:ty => $label:literal),+ $(,)?) => {
        $(
            impl Describe for $t {
                fn describe(&self) -> String {
                    format!("{} {}", $label, self)
                }
            }
        )+
    };
}

impl_describe!(u8 => "byte", i64 => "long", &str => "text", f64 => "float");

macro_rules! record {
    ($name:ident { $($field:ident = $value:expr),* $(,)? }) => {{
        let mut parts: Vec<String> = Vec::new();
        $( parts.push(format!("{}={}", stringify!($field), $value)); )*
        format!("{}({})", stringify!($name), parts.join(", "))
    }};
}

macro_rules! grid {
    ($($row:ident => [$($col:literal)*]),*) => {{
        let mut out = String::new();
        $(
            out.push_str(stringify!($row));
            out.push(':');
            $( out.push_str(&format!(" {}{}", stringify!($row), $col)); )*
            out.push('\n');
        )*
        out
    }};
}

macro_rules! hygiene {
    ($e:expr) => {{
        let x = 100;
        $e + x
    }};
}

macro_rules! define_var {
    ($name:ident = $value:expr) => {
        let $name = $value;
    };
}

macro_rules! show_all {
    ($($item:expr),*) => {{
        let items: Vec<&dyn Display> = vec![$(&$item),*];
        items.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(" | ")
    }};
}

macro_rules! fizz {
    ($n:expr) => {
        match ($n % 3, $n % 5) {
            (0, 0) => "FizzBuzz".to_string(),
            (0, _) => "Fizz".to_string(),
            (_, 0) => "Buzz".to_string(),
            _ => $n.to_string(),
        }
    };
}

fn main() {
    println!("{}", count_tts!(a b c d e));
    println!("{}", count_tts!());
    println!("{}", sum!(1, 2, 3, 4));
    println!("{}", sum!());
    println!("{}", 7u8.describe());
    println!("{}", (-3i64).describe());
    println!("{}", "hi".describe());
    println!("{}", 2.5f64.describe());
    println!("{}", record!(Point { x = 1, y = 2 + 3, label = "p" }));
    print!("{}", grid!(a => [1 2 3], b => [4]));
    let x = 1;
    println!("{}", hygiene!(x * 2));
    define_var!(answer = 6 * 7);
    println!("{}", answer);
    println!("{}", show_all!(1, "two", 3.5, 'c'));
    let line: Vec<String> = (1..=15).map(|n| fizz!(n)).collect();
    println!("{}", line.join(","));
    let v = vec![1, 2, 3];
    let w = vec!{4, 5};
    let z = vec![(); 2];
    println!("{:?} {:?} {:?}", v, w, z);
}
