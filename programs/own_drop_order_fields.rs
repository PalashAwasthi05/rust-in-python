// Drop order of struct fields, Vec elements, enums, closures and early returns.
struct D(String);

impl Drop for D {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

fn d(s: &str) -> D {
    D(s.to_string())
}

struct Outer {
    first: D,
    second: D,
    inner: Inner,
}

struct Inner {
    x: D,
    y: D,
}

impl Drop for Outer {
    fn drop(&mut self) {
        println!("  drop Outer (before fields)");
    }
}

enum Holder {
    One(D),
    Two(D, D),
    Empty,
}

fn early(flag: bool) -> u32 {
    let _guard = d("guard");
    let _a = d("early-a");
    if flag {
        println!("returning early");
        return 1;
    }
    let _b = d("early-b");
    println!("returning late");
    2
}

fn main() {
    println!("struct:");
    {
        let _o = Outer {
            second: d("second"),
            first: d("first"),
            inner: Inner { y: d("y"), x: d("x") },
        };
    }
    println!("vec:");
    {
        let mut v = vec![d("v0"), d("v1"), d("v2")];
        v.push(d("v3"));
        let removed = v.remove(1);
        println!("  removed {}", removed.0);
        v.truncate(2);
        println!("  truncated");
    }
    println!("tuple and array:");
    {
        let _t = (d("t0"), d("t1"));
        let _arr = [d("a0"), d("a1")];
    }
    println!("enum:");
    {
        let hs = vec![Holder::Two(d("h2a"), d("h2b")), Holder::Empty, Holder::One(d("h1"))];
        println!("  {} holders", hs.len());
    }
    println!("closure:");
    {
        let captured = d("captured");
        let c = move || println!("  closure sees {}", captured.0);
        c();
        println!("  closure done");
    }
    println!("early:");
    println!("  -> {}", early(true));
    println!("  -> {}", early(false));
    println!("option replace:");
    {
        let mut slot = Some(d("slot1"));
        slot = Some(d("slot2"));
        println!("  replaced");
        slot.take();
        println!("  taken");
        let _ = slot;
    }
    println!("loop:");
    for i in 0..2 {
        let _it = D(format!("iter{}", i));
        println!("  body {}", i);
    }
    println!("box chain:");
    {
        let _b = Box::new(Inner { x: d("bx"), y: d("by") });
    }
    println!("end");
}
