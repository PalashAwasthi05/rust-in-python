use std::mem;

fn apply<F>(f: F) where F: FnOnce() {
    f();
}

fn apply_to_3<F>(f: F) -> i32 where F: Fn(i32) -> i32 {
    f(3)
}

fn call_n_times<F: FnMut()>(mut f: F, n: usize) {
    for _ in 0..n { f(); }
}

fn call_me<F: Fn()>(f: F) { f(); }

fn function() { println!("I'm a function!"); }

fn main() {
    // closures vs functions and type annotations
    let outer_var = 42;
    let closure_annotated = |i: i32| -> i32 { i + outer_var };
    let closure_inferred = |i| i + outer_var;
    println!("closure_annotated: {}", closure_annotated(1));
    println!("closure_inferred: {}", closure_inferred(1));
    let one = || 1;
    println!("closure returning one: {}", one());

    // capturing by reference
    let color = String::from("green");
    let print = || println!("`color`: {}", color);
    print();
    let _reborrow = &color;
    print();
    let _color_moved = color;

    // capturing by mutable reference
    let mut count = 0;
    let mut inc = || {
        count += 1;
        println!("`count`: {}", count);
    };
    inc();
    inc();
    let _count_reborrowed = &mut count;
    println!("count after closure is done: {}", count);

    // capturing by value
    let movable = Box::new(3);
    let consume = || {
        println!("`movable`: {:?}", movable);
        mem::drop(movable);
    };
    consume();

    // move keyword
    let haystack = vec![1, 2, 3];
    let contains = move |needle| haystack.contains(needle);
    println!("contains 1: {}, contains 4: {}", contains(&1), contains(&4));

    // closures as input parameters
    let greeting = "hello";
    let mut farewell = "goodbye".to_owned();
    let diary = || {
        println!("I said {}.", greeting);
        farewell.push_str("!!!");
        println!("Then I screamed {}.", farewell);
        mem::drop(farewell);
    };
    apply(diary);
    let double = |x| 2 * x;
    println!("3 doubled: {}", apply_to_3(double));
    let mut tally = Vec::new();
    call_n_times(|| tally.push(tally.len() * 10), 4);
    println!("tally {:?}", tally);

    // input functions: a plain fn works where a closure is expected
    let closure = || println!("I'm a closure!");
    call_me(closure);
    call_me(function);

    // type anonymity: each closure has a unique type, generics accept them all
    fn run_all<F: Fn(u32) -> u32>(fs: &[F], x: u32) -> Vec<u32> { fs.iter().map(|f| f(x)).collect() }
    let offsets: Vec<_> = (1..=3).map(|k| move |x: u32| x + k).collect();
    println!("same closure type, many captures: {:?}", run_all(&offsets, 10));
    let boxed: Vec<Box<dyn Fn(u32) -> u32>> = vec![Box::new(|x| x + 1), Box::new(|x| x * x)];
    println!("different closure types boxed: {:?}", boxed.iter().map(|f| f(7)).collect::<Vec<_>>());
    println!("size of non-capturing closure: {}", mem::size_of_val(&one));
    println!("size of closure capturing a u64: {}", { let v = 5u64; let c = move || v; mem::size_of_val(&c) });
}
