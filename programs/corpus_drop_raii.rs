struct Guard {
    name: String,
    depth: usize,
}

impl Guard {
    fn new(name: &str, depth: usize) -> Guard {
        println!("{}acquire {}", "  ".repeat(depth), name);
        Guard { name: name.to_string(), depth }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("{}release {}", "  ".repeat(self.depth), self.name);
    }
}

struct Transaction<'a> {
    log: &'a mut Vec<String>,
    pending: Vec<String>,
    committed: bool,
}

impl<'a> Transaction<'a> {
    fn begin(log: &'a mut Vec<String>) -> Self {
        Transaction { log, pending: Vec::new(), committed: false }
    }
    fn write(&mut self, s: &str) { self.pending.push(s.to_string()); }
    fn commit(mut self) {
        self.log.extend(self.pending.drain(..));
        self.committed = true;
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.committed {
            println!("tx: committed");
        } else {
            println!("tx: rolled back {} pending writes", self.pending.len());
        }
    }
}

fn work(fail: bool) -> Result<u32, String> {
    let _g = Guard::new("work-lock", 1);
    if fail {
        return Err("work failed".into());
    }
    let _inner = Guard::new("inner", 2);
    Ok(42)
}

fn main() {
    let _outer = Guard::new("outer", 0);
    {
        let _a = Guard::new("a", 1);
        let _b = Guard::new("b", 1);
        println!("  in scope with a and b");
    }
    println!("work(false) = {:?}", work(false));
    println!("work(true) = {:?}", work(true));
    let early = Guard::new("early", 1);
    drop(early);
    println!("after explicit drop");
    let guards: Vec<Guard> = (1..=3).map(|i| Guard::new(&format!("vec{}", i), 1)).collect();
    println!("dropping vec of {}", guards.len());
    drop(guards);

    let moved = Guard::new("moved", 1);
    let holder = Some(moved);
    if let Some(g) = &holder {
        println!("holder has {}", g.name);
    }

    let mut log = Vec::new();
    {
        let mut tx = Transaction::begin(&mut log);
        tx.write("x=1");
        tx.write("y=2");
    }
    {
        let mut tx = Transaction::begin(&mut log);
        tx.write("z=3");
        tx.commit();
    }
    println!("log: {:?}", log);
    println!("end of main");
}
