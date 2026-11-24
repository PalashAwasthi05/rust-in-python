// Drop order: scope end, shadowing, mem::drop, temporaries, reassignment.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

impl Noisy {
    fn name(&self) -> &'static str {
        self.0
    }
}

fn make(name: &'static str) -> Noisy {
    println!("make {}", name);
    Noisy(name)
}

fn pass_through(n: Noisy) -> Noisy {
    println!("passing {}", n.0);
    n
}

fn consume(n: Noisy) {
    println!("consuming {}", n.0);
}

fn main() {
    let _a = make("a");
    let b = make("b");
    {
        let _inner = make("inner");
        println!("in block");
    }
    let x = make("x1");
    let x = make("x2");
    println!("shadowed, x = {}", x.name());
    drop(b);
    println!("after drop(b)");

    println!("temp name: {}", make("temp").name());
    println!("after temp");

    let _ = make("underscore");
    println!("after underscore");

    let mut r = make("r1");
    r = make("r2");
    println!("reassigned to {}", r.name());

    let moved = pass_through(make("p"));
    consume(moved);
    println!("after consume");

    let maybe = Some(make("opt"));
    if let Some(n) = maybe {
        println!("took {}", n.name());
    }
    println!("after if let");

    let pair = (make("t0"), make("t1"));
    let (first, _) = pair;
    println!("destructured {}", first.name());

    let boxed: Box<Noisy> = Box::new(make("boxed"));
    let unboxed = *boxed;
    println!("unboxed {}", unboxed.name());

    let cond = true;
    let chosen = if cond { make("yes") } else { make("no") };
    println!("chosen {}", chosen.name());
    let _ = x;
    println!("end of main");
}
