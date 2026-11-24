struct Deferred { tasks: Vec<Box<dyn FnOnce() -> String>> }

impl Deferred {
    fn new() -> Self { Deferred { tasks: Vec::new() } }
    fn defer<F: FnOnce() -> String + 'static>(&mut self, f: F) { self.tasks.push(Box::new(f)); }
    fn run_all(self) -> Vec<String> { self.tasks.into_iter().map(|t| t()).collect() }
}

fn consume<F: FnOnce() -> Vec<String>>(f: F) -> usize { f().len() }

fn call_twice<F: FnMut() -> u32>(mut f: F) -> (u32, u32) { (f(), f()) }

fn make_counter(start: u32) -> impl FnMut() -> u32 {
    let mut n = start;
    move || { n += 1; n }
}

fn make_greeter(greeting: String) -> Box<dyn Fn(&str) -> String> {
    Box::new(move |name| format!("{}, {}!", greeting, name))
}

fn once_only<T>(value: T) -> impl FnOnce() -> T { move || value }

fn main() {
    let names = vec!["ann".to_string(), "bo".to_string()];
    let report = move || {
        let mut v = names;
        v.push("cat".into());
        v
    };
    println!("consumed {} names", consume(report));

    let mut d = Deferred::new();
    let owned = String::from("config.toml");
    d.defer(move || format!("closing {}", owned));
    let nums = vec![3, 1, 2];
    d.defer(move || { let mut n = nums; n.sort(); format!("sorted {:?}", n) });
    let shared = 10;
    d.defer(move || format!("shared copy {}", shared * 2));
    println!("shared still usable: {}", shared);
    for line in d.run_all() { println!("  {}", line); }

    let counter = make_counter(5);
    println!("call_twice: {:?}", call_twice(counter));
    let mut c2 = make_counter(100);
    c2();
    println!("c2 third call: {}", c2() + c2());

    let hi = make_greeter("Hello".to_string());
    let yo = make_greeter(String::from("Yo"));
    println!("{} / {}", hi("Ferris"), yo("crab"));

    let take = once_only(vec![1, 2, 3]);
    let v = take();
    println!("once_only gave {:?}", v);

    let mut log = Vec::new();
    {
        let mut push = |s: &str| log.push(s.to_uppercase());
        push("a");
        push("b");
    }
    log.push("C".into());
    println!("log {:?}", log);

    let data = vec![1, 2, 3, 4];
    let sum_ref = || data.iter().sum::<i32>();
    println!("borrowed sum {} and data still {:?}", sum_ref(), data);
    let into_total = move || data.into_iter().map(|x| x * x).sum::<i32>();
    println!("moved sum of squares {}", into_total());

    let mut opt_task: Option<Box<dyn FnOnce(i32) -> i32>> = Some(Box::new(|x| x * 3));
    if let Some(t) = opt_task.take() { println!("ran task: {}", t(14)); }
    println!("task left? {}", opt_task.is_some());
}
