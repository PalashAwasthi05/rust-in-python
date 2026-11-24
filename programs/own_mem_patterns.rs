use std::mem;

#[derive(Debug)]
enum State {
    Idle,
    Running { name: String, ticks: u32 },
    Done(Vec<String>),
}

struct Machine {
    state: State,
    buffer: Vec<String>,
    current: String,
}

impl Machine {
    fn step(&mut self) {
        self.state = match mem::replace(&mut self.state, State::Idle) {
            State::Idle => State::Running { name: "job".to_string(), ticks: 0 },
            State::Running { name, ticks } if ticks >= 2 => {
                let mut out = mem::take(&mut self.buffer);
                out.push(format!("{} finished", name));
                State::Done(out)
            }
            State::Running { name, ticks } => {
                self.buffer.push(format!("{} tick {}", name, ticks));
                State::Running { name, ticks: ticks + 1 }
            }
            done @ State::Done(_) => done,
        };
    }

    fn flush_line(&mut self) -> String {
        mem::take(&mut self.current)
    }
}

struct Node {
    val: i32,
    next: Option<Box<Node>>,
}

fn reverse(mut head: Option<Box<Node>>) -> Option<Box<Node>> {
    let mut prev = None;
    while let Some(mut node) = head {
        head = node.next.take();
        node.next = prev;
        prev = Some(node);
    }
    prev
}

fn main() {
    let mut m = Machine { state: State::Idle, buffer: Vec::new(), current: String::new() };
    for _ in 0..5 {
        m.step();
        println!("{:?}", m.state);
    }
    m.current.push_str("hello");
    let line = m.flush_line();
    println!("line={:?} current={:?}", line, m.current);

    let mut a = vec![1, 2];
    let mut b = vec![3];
    mem::swap(&mut a, &mut b);
    println!("{:?} {:?}", a, b);
    let old = mem::replace(&mut a, vec![9, 9, 9]);
    println!("old={:?} a={:?}", old, a);

    let mut head = None;
    for i in 1..=4 {
        head = Some(Box::new(Node { val: i, next: head }));
    }
    let head = reverse(head);
    let mut cur = &head;
    let mut vals = Vec::new();
    while let Some(n) = cur {
        vals.push(n.val);
        cur = &n.next;
    }
    println!("{:?}", vals);

    let mut opt = Some(3);
    let taken = opt.take();
    println!("{:?} {:?}", taken, opt);
    let mut opt2 = Some(4);
    let prev = opt2.replace(7);
    println!("{:?} {:?}", prev, opt2);
    let mut x = 5;
    let mut y = 6;
    mem::swap(&mut x, &mut y);
    println!("{} {}", x, y);

    let mut names = vec![String::from("x"), String::from("y"), String::from("z")];
    let middle = mem::take(&mut names[1]);
    names.swap(0, 2);
    println!("{:?} {:?}", middle, names);
    let mut s1 = String::from("left");
    let mut s2 = String::from("right");
    {
        let (r1, r2) = (&mut s1, &mut s2);
        mem::swap(r1, r2);
        r1.push('!');
    }
    println!("{} {}", s1, s2);
}
