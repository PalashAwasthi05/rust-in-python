use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;

fn run_in_thread<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    thread::spawn(f).join().unwrap()
}

fn parallel_map<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> U + Send + Sync + Copy + 'static,
{
    let handles: Vec<thread::JoinHandle<U>> = items.into_iter().map(|item| thread::spawn(move || f(item))).collect();
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

fn share_debug<T: Debug + Send + Sync + 'static>(value: Arc<T>) -> String {
    let v = Arc::clone(&value);
    thread::spawn(move || format!("{:?}", v)).join().unwrap()
}

trait Job: Send {
    fn run(&self) -> String;
}

struct Print(u32);
impl Job for Print {
    fn run(&self) -> String {
        format!("job {}", self.0)
    }
}

struct Stats<T: Send> {
    values: Vec<T>,
}

impl<T: Send + Copy + Into<f64> + 'static> Stats<T> {
    fn mean_in_thread(self) -> f64 {
        thread::spawn(move || {
            let n = self.values.len() as f64;
            self.values.iter().map(|&v| v.into()).sum::<f64>() / n
        })
        .join()
        .unwrap()
    }
}

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
    println!("{}", run_in_thread(|| 6 * 7));
    println!("{:?}", parallel_map(vec![1, 2, 3, 4], |x: i32| x * x + 1));
    println!("{:?}", parallel_map(vec!["a", "bb", "ccc"], |s: &str| s.len()));
    println!("{}", share_debug(Arc::new(vec![(1, 'a'), (2, 'b')])));

    let jobs: Vec<Box<dyn Job>> = vec![Box::new(Print(1)), Box::new(Print(2))];
    let out = thread::spawn(move || jobs.iter().map(|j| j.run()).collect::<Vec<_>>()).join().unwrap();
    println!("{:?}", out);

    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let tasks: Vec<Box<dyn FnOnce() -> String + Send>> = vec![
        Box::new(|| "first".to_string()),
        Box::new(|| "second".to_string()),
    ];
    let mut handles = Vec::new();
    for task in tasks {
        let log = Arc::clone(&log);
        handles.push(thread::spawn(move || {
            let s = task();
            log.lock().unwrap().push(s);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let mut entries = log.lock().unwrap().clone();
    entries.sort();
    println!("{:?}", entries);

    let stats = Stats { values: vec![2.0f32, 4.0, 9.0] };
    println!("mean {:.3}", stats.mean_in_thread());
    let ints = Stats { values: vec![1u8, 2, 3, 4] };
    println!("mean {}", ints.mean_in_thread());

    assert_send::<Arc<Mutex<i32>>>();
    assert_sync::<Mutex<Vec<u8>>>();
    assert_send::<String>();
    println!("bounds ok");
}
