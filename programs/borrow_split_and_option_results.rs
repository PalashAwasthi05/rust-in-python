fn pick<'a>(a: &'a [i32], b: &'a [i32]) -> (Option<&'a i32>, Option<&'a i32>) { (a.first(), b.last()) }
fn longest<'a>(xs: &'a [String]) -> Option<&'a String> { xs.iter().max_by_key(|s| s.len()) }
fn main() {
    let a = vec![1, 2];
    let b = vec![3, 4];
    let (x, y) = pick(&a, &b);
    println!("{:?} {:?}", x, y);
    let mut words = vec![String::from("hi"), String::from("hello")];
    let l = longest(&words).cloned();
    words.push(String::from("x"));
    println!("{:?}", l);
    let mut v = vec![5, 6, 7];
    let (left, right) = v.split_at_mut(1);
    left[0] += right[0];
    v.push(1);
    println!("{:?}", v);
    let mut grid = vec![vec![0; 3]; 3];
    if let Some(row) = grid.get_mut(1) { row[1] = 5; }
    let row = grid.get(1);
    let s: i32 = row.map(|r| r.iter().sum()).unwrap_or(0);
    grid.push(vec![s]);
    println!("{:?}", grid);
    let mut opt: Option<&i32> = None;
    for x in a.iter() { if *x > 1 { opt = Some(x); } }
    println!("{:?}", opt);
    let text = String::from("a b c");
    let mut it = text.split(' ');
    let first = it.next();
    println!("{:?} {:?}", first, it.next());
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() { if top == 2 { stack.push(0); } println!("{}", top); }
    let mut s = String::from("abc");
    let c = s.chars().next();
    s.push('d');
    println!("{:?} {}", c, s);
    let pair = (&a, 3);
    let (ra, k) = pair;
    println!("{:?} {}", ra, k);
}
