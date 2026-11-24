use std::collections::BTreeMap;

#[derive(Debug)]
enum Status { Active, Suspended { days: u32 }, Banned(String) }

#[derive(Debug)]
struct Address { city: String, zip: Option<u32> }

#[derive(Debug)]
struct User {
    id: u32,
    name: &'static str,
    tags: Vec<&'static str>,
    status: Status,
    address: Address,
    scores: BTreeMap<&'static str, f32>,
}

fn main() {
    let width = 12;
    let prec = 3;
    let value = 3.14159265;
    println!("[{:>width$}]", "right", width = width);
    println!("[{:<width$}]", "left");
    println!("[{:^w$.p$}]", value, w = width, p = prec);
    println!("[{:^1$}]", "mid", width);
    println!("[{:08.3}] [{:08.3}] [{:+08.2}]", value, -value, value);
    println!("[{:+e}] [{:e}] [{:+E}] [{:.3e}]", 123456.0f64, 0.00042f64, -9.5f64, 6.02214076e23f64);
    println!("[{:>+10.4}] [{:<10.1}] [{:^+10}]", 2.5f32, 99.95, 7);
    println!("[{:#>8}] [{:~^9}] [{:0>5}] [{:<05}]", 42, "x", 7, 3);
    println!("[{:#06x}] [{:#010b}] [{:o}] [{:X}]", 255, 5, 64, 48879);

    let rows = [("apples", 3, 0.5), ("kiwis", 12, 0.25), ("watermelon", 1, 4.0)];
    let name_w = rows.iter().map(|r| r.0.len()).max().unwrap();
    for (name, qty, price) in rows {
        println!("{:<nw$} | {:>3} | {:>6.2} | {:>7.2}", name, qty, price, qty as f64 * price, nw = name_w);
    }

    let mut scores = BTreeMap::new();
    scores.insert("math", 91.5);
    scores.insert("art", 78.0);
    let u = User {
        id: 17,
        name: "dana",
        tags: vec!["admin", "beta"],
        status: Status::Suspended { days: 3 },
        address: Address { city: "Oslo".into(), zip: Some(150) },
        scores,
    };
    println!("{:?}", u.status);
    println!("{:#?}", u);
    let others = vec![Status::Active, Status::Banned("spam".into())];
    println!("{:#?}", others);
    let nested: BTreeMap<u8, Vec<Option<(i32, &str)>>> = [(1, vec![Some((1, "a")), None]), (2, vec![])].into_iter().collect();
    println!("{:#?}", nested);
    println!("{:?}", nested);
    let name = "inline";
    let n = 5;
    println!("{name:>10}|{n:<4}|{n:^5}|{:?}", (n, name));
}
