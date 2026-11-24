// Segment tree with a generic monoid (sum / min / max) and a Fenwick (binary indexed) tree.
use std::ops::Range;

trait Monoid {
    type T: Copy + std::fmt::Debug;
    const NAME: &'static str;
    fn identity() -> Self::T;
    fn combine(a: Self::T, b: Self::T) -> Self::T;
}

struct Sum;
struct Min;
struct Max;

impl Monoid for Sum {
    type T = i64;
    const NAME: &'static str = "sum";
    fn identity() -> i64 {
        0
    }
    fn combine(a: i64, b: i64) -> i64 {
        a + b
    }
}

impl Monoid for Min {
    type T = i64;
    const NAME: &'static str = "min";
    fn identity() -> i64 {
        i64::MAX
    }
    fn combine(a: i64, b: i64) -> i64 {
        a.min(b)
    }
}

impl Monoid for Max {
    type T = i64;
    const NAME: &'static str = "max";
    fn identity() -> i64 {
        i64::MIN
    }
    fn combine(a: i64, b: i64) -> i64 {
        std::cmp::max(a, b)
    }
}

struct SegTree<M: Monoid> {
    n: usize,
    tree: Vec<M::T>,
}

impl<M: Monoid> SegTree<M> {
    fn new(values: &[M::T]) -> Self {
        let n = values.len().next_power_of_two();
        let mut tree = vec![M::identity(); 2 * n];
        tree[n..n + values.len()].copy_from_slice(values);
        for i in (1..n).rev() {
            tree[i] = M::combine(tree[2 * i], tree[2 * i + 1]);
        }
        SegTree { n, tree }
    }

    fn set(&mut self, mut i: usize, v: M::T) {
        i += self.n;
        self.tree[i] = v;
        while i > 1 {
            i /= 2;
            self.tree[i] = M::combine(self.tree[2 * i], self.tree[2 * i + 1]);
        }
    }

    fn query(&self, range: Range<usize>) -> M::T {
        let (mut l, mut r) = (range.start + self.n, range.end + self.n);
        let mut left = M::identity();
        let mut right = M::identity();
        while l < r {
            if l & 1 == 1 {
                left = M::combine(left, self.tree[l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                right = M::combine(self.tree[r], right);
            }
            l >>= 1;
            r >>= 1;
        }
        M::combine(left, right)
    }

    fn query_recursive(&self, node: usize, node_lo: usize, node_hi: usize, lo: usize, hi: usize) -> M::T {
        if hi <= node_lo || node_hi <= lo {
            return M::identity();
        }
        if lo <= node_lo && node_hi <= hi {
            return self.tree[node];
        }
        let mid = (node_lo + node_hi) / 2;
        M::combine(
            self.query_recursive(2 * node, node_lo, mid, lo, hi),
            self.query_recursive(2 * node + 1, mid, node_hi, lo, hi),
        )
    }
}

struct Fenwick {
    tree: Vec<i64>,
}

impl Fenwick {
    fn new(n: usize) -> Self {
        Fenwick { tree: vec![0; n + 1] }
    }

    fn from_slice(values: &[i64]) -> Self {
        let mut f = Fenwick::new(values.len());
        for (i, &v) in values.iter().enumerate() {
            f.add(i, v);
        }
        f
    }

    fn add(&mut self, i: usize, delta: i64) {
        let mut i = i + 1;
        while i < self.tree.len() {
            self.tree[i] += delta;
            i += i & i.wrapping_neg();
        }
    }

    fn prefix(&self, end: usize) -> i64 {
        let mut i = end;
        let mut s = 0;
        while i > 0 {
            s += self.tree[i];
            i &= i - 1;
        }
        s
    }

    fn range(&self, r: Range<usize>) -> i64 {
        self.prefix(r.end) - self.prefix(r.start)
    }

    // smallest index whose prefix sum (inclusive) reaches target, for non-negative values
    fn lower_bound(&self, mut target: i64) -> usize {
        let mut pos = 0;
        let mut step = (self.tree.len() - 1).next_power_of_two();
        while step > 0 {
            if pos + step < self.tree.len() && self.tree[pos + step] < target {
                pos += step;
                target -= self.tree[pos];
            }
            step >>= 1;
        }
        pos
    }
}

fn report<M: Monoid>(values: &[i64]) where M: Monoid<T = i64> {
    let mut st: SegTree<M> = SegTree::new(values);
    let ranges = [0..values.len(), 2..5, 3..4, 1..7, 5..5];
    let answers: Vec<i64> = ranges.iter().map(|r| st.query(r.clone())).collect();
    println!("{} queries {:?}", M::NAME, answers);
    st.set(3, -10);
    let r1 = st.query(0..values.len());
    let r2 = st.query_recursive(1, 0, st.n, 2, 6);
    println!("{} after set(3,-10): all={} [2,6)={}", M::NAME, r1, r2);
}

fn main() {
    let values = [5i64, 3, 8, 6, 1, 4, 7, 2, 9];
    report::<Sum>(&values);
    report::<Min>(&values);
    report::<Max>(&values);

    let mut fw = Fenwick::from_slice(&values);
    let prefixes: Vec<i64> = (0..=values.len()).map(|i| fw.prefix(i)).collect();
    println!("prefix sums {:?}", prefixes);
    println!("range [2,5) = {}", fw.range(2..5));
    fw.add(4, 10);
    println!("after add(4, 10): range [3,6) = {} total = {}", fw.range(3..6), fw.prefix(values.len()));
    for target in [1, 5, 9, 16, 30, 55, 100] {
        println!("lower_bound({}) = {}", target, fw.lower_bound(target));
    }

    // inversion count with a Fenwick tree
    let arr = [3usize, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    let maxv = *arr.iter().max().unwrap();
    let mut seen = Fenwick::new(maxv + 1);
    let mut inversions = 0;
    for (i, &x) in arr.iter().enumerate() {
        inversions += i as i64 - seen.prefix(x + 1);
        seen.add(x, 1);
    }
    println!("inversions in {:?}: {}", arr, inversions);
}
