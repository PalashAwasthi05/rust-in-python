use std::collections::HashMap;
use std::mem::size_of_val;

type NanoSecond = u64;
type Inch = u64;
type U64 = u64;
type Grid<T> = Vec<Vec<T>>;
type Lookup = HashMap<String, Vec<(u32, f32)>>;
type Callback = fn(i32) -> i32;

fn make_grid<T: Clone>(rows: usize, cols: usize, fill: T) -> Grid<T> {
    vec![vec![fill; cols]; rows]
}

fn twice(x: i32) -> i32 { x * 2 }

fn main() {
    // suffixed and unsuffixed literals
    let x = 1u8;
    let y = 2u32;
    let z = 3f32;
    let i = 1;
    let f = 1.0;
    println!("size of `x` in bytes: {}", size_of_val(&x));
    println!("size of `y` in bytes: {}", size_of_val(&y));
    println!("size of `z` in bytes: {}", size_of_val(&z));
    println!("size of `i` in bytes: {}", size_of_val(&i));
    println!("size of `f` in bytes: {}", size_of_val(&f));

    // inference from later usage
    let elem = 5u8;
    let mut vec = Vec::new();
    vec.push(elem);
    println!("inferred Vec<u8>: {:?}, element size {}", vec, size_of_val(&vec[0]));
    let mut counts = HashMap::new();
    counts.insert("apples", 3);
    let total: i64 = counts.values().sum();
    println!("inferred value type via sum: {}", total);
    let parsed = "42".parse::<i16>().unwrap();
    let collected: Vec<_> = "a,b,c".split(',').collect();
    println!("turbofish parse {} and collect {:?}", parsed, collected);
    let doubled = vec.iter().map(|&v| v as u32 * 2).collect::<Vec<_>>();
    println!("doubled {:?}", doubled);
    let default_float = 2.0 / 3.0;
    let default_int = 7 / 2;
    println!("default float {} default int {}", default_float, default_int);

    // aliases
    let nanoseconds: NanoSecond = 5 as U64;
    let inches: Inch = 2 as U64;
    println!("{} nanoseconds + {} inches = {} unit?", nanoseconds, inches, nanoseconds + inches);
    let mut grid: Grid<char> = make_grid(2, 3, '.');
    grid[1][2] = '#';
    for row in &grid {
        println!("{}", row.iter().collect::<String>());
    }
    let mut lookup: Lookup = HashMap::new();
    lookup.entry("sensor-a".into()).or_default().push((1, 0.5));
    lookup.entry("sensor-a".into()).or_default().push((2, 0.75));
    lookup.entry("sensor-b".into()).or_default().push((1, 9.0));
    let mut keys: Vec<_> = lookup.keys().cloned().collect();
    keys.sort();
    for k in keys {
        println!("{} -> {:?}", k, lookup[&k]);
    }
    let cb: Callback = twice;
    let cbs: [Callback; 2] = [twice, |v| v - 1];
    println!("callback {} and array {:?}", cb(21), cbs.iter().map(|c| c(10)).collect::<Vec<_>>());
    println!("type size of Callback: {}", std::mem::size_of::<Callback>());
}
