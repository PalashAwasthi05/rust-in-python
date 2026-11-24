struct Solver {
    n: usize,
    cols: Vec<bool>,
    diag1: Vec<bool>,
    diag2: Vec<bool>,
    placement: Vec<usize>,
    solutions: Vec<Vec<usize>>,
}

impl Solver {
    fn new(n: usize) -> Self {
        Solver { n, cols: vec![false; n], diag1: vec![false; 2 * n], diag2: vec![false; 2 * n], placement: Vec::new(), solutions: Vec::new() }
    }

    fn solve(&mut self, row: usize) {
        if row == self.n {
            self.solutions.push(self.placement.clone());
            return;
        }
        for c in 0..self.n {
            let d1 = row + c;
            let d2 = row + self.n - c;
            if self.cols[c] || self.diag1[d1] || self.diag2[d2] { continue; }
            self.cols[c] = true; self.diag1[d1] = true; self.diag2[d2] = true;
            self.placement.push(c);
            self.solve(row + 1);
            self.placement.pop();
            self.cols[c] = false; self.diag1[d1] = false; self.diag2[d2] = false;
        }
    }
}

fn subset_sums(nums: &[i32], target: i32) -> Vec<Vec<i32>> {
    fn go(nums: &[i32], start: usize, remaining: i32, cur: &mut Vec<i32>, out: &mut Vec<Vec<i32>>) {
        if remaining == 0 { out.push(cur.clone()); return; }
        for i in start..nums.len() {
            if i > start && nums[i] == nums[i - 1] { continue; }
            if nums[i] > remaining { break; }
            cur.push(nums[i]);
            go(nums, i + 1, remaining - nums[i], cur, out);
            cur.pop();
        }
    }
    let mut sorted = nums.to_vec();
    sorted.sort();
    let mut out = Vec::new();
    go(&sorted, 0, target, &mut Vec::new(), &mut out);
    out
}

fn main() {
    for n in 1..=8 {
        let mut s = Solver::new(n);
        s.solve(0);
        println!("{}-queens: {} solutions", n, s.solutions.len());
    }
    let mut s = Solver::new(6);
    s.solve(0);
    for (i, sol) in s.solutions.iter().enumerate() {
        println!("solution {}: {:?}", i + 1, sol);
    }
    for &c in &s.solutions[0] {
        let row: String = (0..6).map(|x| if x == c { 'Q' } else { '.' }).collect();
        println!("  {}", row);
    }
    let combos = subset_sums(&[10, 1, 2, 7, 6, 1, 5], 8);
    println!("subsets summing to 8: {:?}", combos);
}
