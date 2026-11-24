use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, Once, OnceLock};
use std::thread;

static CONFIG: OnceLock<String> = OnceLock::new();
static TABLE: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    println!("building table");
    let mut m = HashMap::new();
    m.insert("one", 1);
    m.insert("two", 2);
    m.insert("three", 3);
    m
});
static REGISTRY: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SETUP: Once = Once::new();

fn config() -> &'static str {
    CONFIG.get_or_init(|| {
        println!("initializing config");
        String::from("production")
    })
}

fn register(name: &str) {
    SETUP.call_once(|| println!("one-time setup"));
    REGISTRY.lock().unwrap().push(name.to_string());
}

fn main() {
    println!("before: {:?}", CONFIG.get());
    let handles: Vec<_> = (0..3).map(|_| thread::spawn(|| config().len())).collect();
    for h in handles {
        println!("len {}", h.join().unwrap());
    }
    println!("after: {:?}", CONFIG.get());
    println!("set again: {:?}", CONFIG.set(String::from("dev")));

    println!("main before table");
    println!("two = {}", TABLE["two"]);
    println!("three = {:?}", TABLE.get("three"));
    let total: u32 = TABLE.values().sum();
    println!("total = {}", total);

    for name in ["x", "y", "z"] {
        let name = name.to_string();
        thread::spawn(move || register(&name)).join().unwrap();
    }
    println!("registry {:?}", REGISTRY.lock().unwrap());
    println!("setup done: {}", SETUP.is_completed());

    let cell: OnceLock<u32> = OnceLock::new();
    assert!(cell.get().is_none());
    let v = cell.get_or_init(|| 5);
    println!("cell {} {:?}", v, cell.get());
    println!("into_inner {:?}", cell.into_inner());
    let lazy = LazyLock::new(|| vec![1, 2, 3]);
    println!("lazy len {}", lazy.len());
    println!("forced {:?}", *LazyLock::force(&lazy));
}
