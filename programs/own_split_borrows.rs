// Borrowing different fields of a struct at the same time.
struct Inventory {
    items: Vec<(String, u32)>,
    total: u32,
    history: Vec<String>,
}

impl Inventory {
    fn restock(&mut self, amount: u32) {
        let Inventory { items, total, history } = self;
        for (name, qty) in items.iter_mut() {
            *qty += amount;
            *total += amount;
            history.push(format!("restocked {}", name));
        }
    }

    fn drain_low(&mut self, threshold: u32) -> Vec<String> {
        let history = &mut self.history;
        let mut removed = Vec::new();
        self.items.retain(|(name, qty)| {
            if *qty < threshold {
                history.push(format!("removed {}", name));
                removed.push(name.clone());
                false
            } else {
                true
            }
        });
        removed
    }
}

struct Pair {
    left: Vec<i32>,
    right: Vec<i32>,
}

fn main() {
    let mut inv = Inventory {
        items: vec![("apple".to_string(), 3), ("pear".to_string(), 10), ("fig".to_string(), 1)],
        total: 0,
        history: Vec::new(),
    };
    inv.restock(2);
    let removed = inv.drain_low(4);
    println!("removed: {:?}", removed);
    println!("total: {}", inv.total);
    for h in &inv.history {
        println!("  {}", h);
    }

    let mut p = Pair { left: vec![1, 2, 3], right: vec![10, 20] };
    let l = &mut p.left;
    let r = &p.right;
    for x in r {
        l.push(*x);
    }
    println!("{:?} {:?}", p.left, p.right);

    let mut arr = [1, 2, 3, 4, 5, 6];
    let (a, b) = arr.split_at_mut(3);
    a[0] = b[2];
    b[0] += a[1];
    println!("{:?}", arr);

    let mut t = (String::from("x"), 5);
    let s = &mut t.0;
    let n = &mut t.1;
    s.push('y');
    *n += 1;
    println!("{:?}", t);

    let mut v = vec![5, 1, 4];
    if let [first, .., last] = v.as_mut_slice() {
        std::mem::swap(first, last);
    }
    println!("{:?}", v);

    let mut matrix = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
    let (top, bottom) = matrix.split_at_mut(1);
    top[0][0] += bottom[1][1];
    bottom[0].push(top[0][1]);
    println!("{:?}", matrix);

    let mut pts = [(1, 2), (3, 4)];
    for (x, y) in pts.iter_mut() {
        std::mem::swap(x, y);
    }
    let [ref mut a0, ref mut a1] = pts;
    a0.0 += a1.1;
    println!("{:?}", pts);
}
