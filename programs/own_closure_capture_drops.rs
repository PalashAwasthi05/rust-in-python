use std::cell::RefCell;
use std::rc::Rc;

struct Bell(&'static str);
impl Drop for Bell {
    fn drop(&mut self) {
        println!("ring {}", self.0);
    }
}

fn make_ringer(b: Bell) -> impl Fn() -> &'static str {
    move || b.0
}

fn main() {
    let shared = Rc::new(RefCell::new(0));
    let handlers: Vec<Box<dyn Fn(i32)>> = (0..3)
        .map(|i| {
            let s = Rc::clone(&shared);
            Box::new(move |x: i32| *s.borrow_mut() += x * i) as Box<dyn Fn(i32)>
        })
        .collect();
    println!("count with handlers {}", Rc::strong_count(&shared));
    for h in &handlers {
        h(10);
    }
    println!("value {}", shared.borrow());
    drop(handlers);
    println!("count after drop {}", Rc::strong_count(&shared));

    let s2 = Rc::clone(&shared);
    let f = move || *s2.borrow();
    println!("count with closure {}", Rc::strong_count(&shared));
    println!("f = {}", f());
    drop(f);
    println!("count after closure {}", Rc::strong_count(&shared));

    let r = make_ringer(Bell("one"));
    println!("ringer says {}", r());
    {
        let inner = make_ringer(Bell("two"));
        println!("inner says {}", inner());
    }
    let opt_closure: Option<Box<dyn Fn() -> &'static str>> = Some(Box::new(make_ringer(Bell("three"))));
    if let Some(c) = &opt_closure {
        println!("opt says {}", c());
    }
    let moved_bell = Bell("four");
    let consume = move || {
        let b = moved_bell;
        println!("consuming {}", b.0);
    };
    println!("before consume");
    consume();
    println!("after consume");
    println!("end");
}
