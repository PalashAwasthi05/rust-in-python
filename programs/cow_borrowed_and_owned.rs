use std::borrow::Cow;
fn abs_all(input: &mut Cow<[i32]>) { for i in 0..input.len() { let v = input[i]; if v < 0 { input.to_mut()[i] = -v; } } }
fn label(n: i32) -> Cow<'static, str> { if n == 0 { "zero".into() } else { Cow::Owned(format!("n{}", n)) } }
fn main() {
    let slice = [0, 1, 2]; let mut c = Cow::from(&slice[..]); abs_all(&mut c); println!("{} {:?}", matches!(c, Cow::Borrowed(_)), c);
    let slice2 = [-1, 0]; let mut c2 = Cow::from(&slice2[..]); abs_all(&mut c2); println!("{} {:?}", matches!(c2, Cow::Owned(_)), c2);
    let s: String = label(3).into_owned(); println!("{} {} {}", s, label(0), label(0).len());
    let mut cs = Cow::Borrowed("abc"); cs.to_mut().push('d'); println!("{} {}", cs, cs == Cow::Borrowed("abcd"));
    let owned: String = "x".to_owned(); let v: Vec<i32> = [1, 2][..].to_owned(); println!("{} {:?}", owned, v);
}
