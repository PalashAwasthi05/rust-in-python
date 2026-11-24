use std::collections::HashMap;
use std::rc::Rc;

struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
impl D {
    fn get(&self) -> u32 { self.0.len() as u32 }
}

struct S { a: D, b: D, c: D }

fn params(x: D, y: D) {
    let _z = D("z-local");
    println!("in params {} {}", x.0, y.0);
}

fn maybe(flag: bool) -> Option<u32> {
    let _g = D("q-guard");
    let v: Option<u32> = if flag { Some(1) } else { None };
    let n = v?;
    println!("got {}", n);
    Some(n)
}

fn make(n: &'static str) -> D { D(n) }

fn main() {
    println!("-- match temp");
    match make("scrut").get() {
        5 => println!("five"),
        _ => println!("other"),
    }
    println!("-- if let temp");
    if let Some(x) = Some(make("iflet")).as_ref().map(|d| d.get()) {
        println!("len {}", x);
    }
    println!("-- update syntax");
    {
        let s1 = S { a: D("a1"), b: D("b1"), c: D("c1") };
        let s2 = S { a: D("a2"), ..s1 };
        println!("s2 built {}", s2.a.0);
    }
    println!("-- params");
    params(D("x"), D("y"));
    println!("-- closure");
    {
        let d = D("cap");
        let f = move || d.get();
        println!("f = {}", f());
        drop(f);
        println!("closure dropped");
    }
    println!("-- hashmap");
    {
        let mut m = HashMap::new();
        m.insert(1, D("m1"));
        m.insert(1, D("m1b"));
        println!("replaced");
        m.remove(&1);
        println!("removed");
        m.insert(2, D("m2"));
    }
    println!("-- question");
    maybe(true);
    maybe(false);
    println!("-- field assign");
    {
        let mut s = S { a: D("fa"), b: D("fb"), c: D("fc") };
        s.b = D("fb2");
        println!("assigned");
    }
    println!("-- pop");
    {
        let mut v = vec![D("p1"), D("p2"), D("p3")];
        v.pop();
        println!("popped");
        let _kept = v.pop();
        println!("kept");
    }
    println!("-- into_iter partial");
    {
        let v = vec![D("i1"), D("i2"), D("i3")];
        let mut it = v.into_iter();
        let first = it.next();
        println!("first taken");
        drop(it);
        println!("iter dropped");
        drop(first);
    }
    println!("-- rc");
    {
        let a = Rc::new(D("rc"));
        let b = Rc::clone(&a);
        drop(a);
        println!("a gone");
        drop(b);
        println!("b gone");
    }
    println!("-- forget");
    std::mem::forget(D("forgotten"));
    println!("-- break value");
    let r = loop {
        let _l = D("loop-local");
        break 5;
    };
    println!("r = {}", r);
    println!("-- nested vec");
    {
        let _vv = vec![vec![D("n00"), D("n01")], vec![D("n10")]];
    }
    println!("-- drain");
    {
        let mut v = vec![D("d0"), D("d1"), D("d2"), D("d3")];
        let drained: Vec<D> = v.drain(1..3).collect();
        println!("drained {}", drained.len());
    }
    println!("-- replace");
    {
        let mut x = D("old");
        let _prev = std::mem::replace(&mut x, D("new"));
        println!("replaced");
    }
    println!("end");
}
