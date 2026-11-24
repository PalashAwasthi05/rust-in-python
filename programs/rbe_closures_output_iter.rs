fn create_fn() -> impl Fn() {
    let text = "Fn".to_owned();
    move || println!("This is a: {}", text)
}

fn create_fnmut() -> impl FnMut() {
    let mut text = "FnMut".to_owned();
    move || {
        text.push('!');
        println!("This is a: {}", text)
    }
}

fn create_fnonce() -> impl FnOnce() -> String {
    let text = "FnOnce".to_owned();
    move || text + " consumed"
}

fn make_validator(min: usize, max: usize) -> Box<dyn Fn(&str) -> Result<(), String>> {
    Box::new(move |s| {
        let n = s.chars().count();
        if n < min { Err(format!("{:?} shorter than {}", s, min)) }
        else if n > max { Err(format!("{:?} longer than {}", s, max)) }
        else { Ok(()) }
    })
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn main() {
    let fn_plain = create_fn();
    let mut fn_mut = create_fnmut();
    let fn_once = create_fnonce();
    fn_plain();
    fn_mut();
    fn_mut();
    println!("{}", fn_once());

    let check = make_validator(3, 8);
    for name in ["al", "ferris", "a-very-long-name"] {
        println!("{:<18} {:?}", name, check(name));
    }
    let shout_len = compose(|s: &str| s.to_uppercase(), |s: String| format!("{} ({})", s, s.len()));
    println!("{}", shout_len("compose"));

    // Iterator::any
    let vec1 = vec![1, 2, 3];
    let vec2 = vec![4, 5, 6];
    println!("2 in vec1: {}", vec1.iter().any(|&x| x == 2));
    println!("2 in vec2: {}", vec2.into_iter().any(|x| x == 2));
    println!("vec1 len {}, first element {}", vec1.len(), vec1[0]);
    let array1 = [1, 2, 3];
    let array2 = [4, 5, 6];
    println!("2 in array1: {}", array1.iter().any(|&x| x == 2));
    println!("2 in array2: {}", array2.into_iter().any(|x| x == 2));

    // Iterator::find
    let mut iter = vec1.iter();
    println!("find 2 in vec1: {:?}", iter.find(|&&x| x == 2));
    println!("next after find: {:?}", iter.next());
    let words = ["apple", "banana", "cherry", "date"];
    println!("first with 'n': {:?}", words.iter().find(|w| w.contains('n')));
    println!("first longer than 10: {:?}", words.iter().find(|w| w.len() > 10));
    println!("find_map first parsable: {:?}", ["x", "12", "7"].iter().find_map(|s| s.parse::<i32>().ok()));

    // Iterator::position
    let vec = vec![1, 9, 3, 3, 13, 2];
    let index_of_first_even = vec.iter().position(|&x| x % 2 == 0);
    println!("first even at {:?}", index_of_first_even);
    let index_of_first_negative = vec.iter().position(|&x| x < 0);
    println!("first negative at {:?}", index_of_first_negative);
    println!("last 3 at {:?}", vec.iter().rposition(|&x| x == 3));
    println!("all odd? {} all positive? {}", vec.iter().all(|x| x % 2 == 1), vec.iter().all(|&x| x > 0));

    let counters: Vec<Box<dyn FnMut() -> u32>> = (1..=3u32).map(|step| {
        let mut n = 0;
        Box::new(move || { n += step; n }) as Box<dyn FnMut() -> u32>
    }).collect();
    let results: Vec<Vec<u32>> = counters.into_iter().map(|mut c| (0..3).map(|_| c()).collect()).collect();
    println!("independent counters: {:?}", results);
}
