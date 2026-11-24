// Lazy evaluation: side effects inside adapters interleave exactly as rustc runs them.
struct Source {
    n: u32,
    limit: u32,
}

impl Iterator for Source {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n >= self.limit {
            println!("source: exhausted");
            return None;
        }
        self.n += 1;
        println!("source: yield {}", self.n);
        Some(self.n)
    }
}

fn main() {
    println!("-- building the chain does nothing");
    let chain = Source { n: 0, limit: 6 }
        .map(|x| {
            println!("map: {} -> {}", x, x * 10);
            x * 10
        })
        .filter(|x| {
            let keep = x % 20 == 0;
            println!("filter: {} {}", x, keep);
            keep
        });
    println!("-- now pulling two");
    let firsts: Vec<u32> = chain.take(2).collect();
    println!("got {:?}", firsts);

    println!("-- inspect + take_while");
    let total: u32 = Source { n: 0, limit: 10 }
        .inspect(|x| println!("inspect {}", x))
        .take_while(|&x| x < 3)
        .sum();
    println!("total {}", total);

    println!("-- zip pulls left first");
    let z: Vec<(u32, u32)> = Source { n: 0, limit: 2 }.zip(Source { n: 100, limit: 103 }).collect();
    println!("zipped {:?}", z);

    println!("-- chain switches over");
    let c = Source { n: 0, limit: 1 }.chain(Source { n: 50, limit: 51 }).count();
    println!("count {}", c);

    println!("-- find stops early");
    let f = Source { n: 0, limit: 100 }.find(|x| x % 3 == 0);
    println!("found {:?}", f);

    println!("-- skip consumes on first next");
    let mut s = Source { n: 0, limit: 5 }.skip(2);
    println!("skip built");
    println!("next {:?}", s.next());

    println!("-- flat_map");
    let fm: Vec<u32> = Source { n: 0, limit: 2 }
        .flat_map(|x| {
            println!("expand {}", x);
            vec![x, x + 100]
        })
        .collect();
    println!("{:?}", fm);

    println!("-- for loop with break");
    for x in (Source { n: 0, limit: 10 }).map(|x| x * x) {
        if x > 5 {
            println!("break at {}", x);
            break;
        }
        println!("body {}", x);
    }

    println!("-- peekable peeks once");
    let mut p = Source { n: 0, limit: 3 }.peekable();
    println!("peek {:?}", p.peek());
    println!("peek {:?}", p.peek());
    println!("next {:?}", p.next());
    println!("next {:?}", p.next());

    println!("-- scan with early stop");
    let sc: Vec<u32> = Source { n: 0, limit: 10 }
        .scan(0, |acc, x| {
            *acc += x;
            println!("scan acc {}", acc);
            if *acc > 6 { None } else { Some(*acc) }
        })
        .collect();
    println!("{:?}", sc);

    println!("-- fuse after exhaustion");
    let mut fused = Source { n: 0, limit: 1 }.fuse();
    println!("{:?}", fused.next());
    println!("{:?}", fused.next());
    println!("{:?}", fused.next());
}
