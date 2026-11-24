#[derive(PartialEq, Debug)]
struct T(u32);
impl Drop for T {
    fn drop(&mut self) {
        println!("drop T{}", self.0);
    }
}

fn len_of_temp() -> usize {
    let _local = T(100);
    vec![T(101), T(102)].len()
}

fn set(slot: &mut T, v: u32) {
    *slot = T(v);
}

fn consume_in_closure() {
    let t = T(200);
    let f = move || {
        println!("closure has T{}", t.0);
        drop(t);
        println!("closure done");
    };
    println!("calling");
    f();
    println!("called");
}

fn main() {
    println!("len {}", len_of_temp());
    println!("-- deref assign");
    let mut a = T(1);
    set(&mut a, 2);
    println!("a = T{}", a.0);
    let r = &mut a;
    *r = T(3);
    println!("-- loop reassign");
    let mut cur = T(10);
    for i in 11..13 {
        cur = T(i);
        println!("cur T{}", cur.0);
    }
    println!("-- vec ops");
    {
        let mut v = vec![T(20), T(21), T(21), T(22)];
        v.dedup();
        println!("deduped");
        let x = v.swap_remove(0);
        println!("swap_removed T{}", x.0);
        v.insert(0, T(23));
        v.resize_with(2, || T(99));
        println!("resized {}", v.len());
        v[1] = T(24);
        println!("indexed assign");
        v.retain(|t| t.0 != 23);
        println!("retained {}", v.len());
    }
    println!("-- option replace");
    {
        let mut o = Some(T(30));
        let old = o.replace(T(31));
        println!("old {:?}", old.as_ref().map(|t| t.0));
        o = None;
        println!("set none");
        let _ = o.insert(T(32));
        println!("inserted");
    }
    println!("-- match arm guard");
    match T(40) {
        T(n) if n > 50 => println!("big"),
        ref t => {
            let _g = T(41);
            println!("small T{}", t.0);
        }
    }
    println!("-- closure consumes");
    consume_in_closure();
    println!("-- nested shadow");
    let s = T(50);
    {
        let s = T(51);
        let s = T(s.0 + 1);
        println!("inner T{}", s.0);
    }
    println!("outer T{}", s.0);
    println!("-- struct literal temp");
    let pair = (T(60), T(61).0);
    println!("pair T{} {}", pair.0 .0, pair.1);
    println!("end");
}
