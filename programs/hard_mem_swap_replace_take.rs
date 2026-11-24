use std::mem;

#[derive(Debug, Default)]
struct Buffer { name: String, data: Vec<u8>, flushes: u32 }

impl Buffer {
    fn flush(&mut self) -> Vec<u8> {
        self.flushes += 1;
        mem::take(&mut self.data)
    }
    fn rename(&mut self, new: &str) -> String {
        mem::replace(&mut self.name, new.to_string())
    }
}

#[derive(Debug)]
enum Conn { Idle, Active { id: u32, sent: usize }, Closed(String) }

impl Conn {
    fn close(&mut self, reason: &str) -> Conn {
        mem::replace(self, Conn::Closed(reason.to_string()))
    }
}

fn rotate3<T>(a: &mut T, b: &mut T, c: &mut T) {
    mem::swap(a, b);
    mem::swap(b, c);
}

fn main() {
    let mut buf = Buffer { name: "log".into(), data: vec![1, 2, 3], ..Default::default() };
    let out = buf.flush();
    println!("flushed {:?}, buffer now {:?}", out, buf);
    buf.data.extend([9, 9]);
    let old = buf.rename("audit");
    println!("old name {}, new {:?}", old, buf);

    let mut a = vec![1, 2];
    let mut b = vec![10, 20, 30];
    mem::swap(&mut a, &mut b);
    println!("swapped a={:?} b={:?}", a, b);

    let (mut x, mut y, mut z) = ("x".to_string(), "y".to_string(), "z".to_string());
    rotate3(&mut x, &mut y, &mut z);
    println!("rotated: {} {} {}", x, y, z);

    let mut conn = Conn::Active { id: 7, sent: 1024 };
    let prev = conn.close("timeout");
    println!("prev {:?}, now {:?}", prev, conn);
    let again = conn.close("again");
    println!("prev {:?}, now {:?}", again, conn);
    let idle = mem::replace(&mut conn, Conn::Idle);
    println!("took {:?}, left {:?}", idle, conn);

    let mut words = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let taken = mem::take(&mut words[1]);
    println!("taken {:?}, words {:?}", taken, words);
    words.swap(0, 2);
    println!("slice swap {:?}", words);

    let mut maybe: Option<u32> = Some(5);
    let got = maybe.take();
    println!("Option::take -> {:?}, left {:?}", got, maybe);
    let old = maybe.replace(11);
    println!("Option::replace -> {:?}, now {:?}", old, maybe);

    let mut total = 100;
    let before = mem::replace(&mut total, 0);
    println!("before {} after {}, size_of u64={} Option<Box<u8>>={}", before, total, mem::size_of::<u64>(), mem::size_of::<Option<Box<u8>>>());

    let mut list = vec![3, 1, 2];
    let mut sorted = mem::take(&mut list);
    sorted.sort();
    println!("list {:?} sorted {:?}", list, sorted);
}
