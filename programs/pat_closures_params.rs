// Irrefutable patterns in fn params and closures; match on tuples of references.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

fn dot(Vec2 { x: ax, y: ay }: Vec2, Vec2 { x: bx, y: by }: Vec2) -> f64 {
    ax * bx + ay * by
}

fn swap((a, b): (i32, &str)) -> (&str, i32) {
    (b, a)
}

fn total(&(count, price): &(u32, f64)) -> f64 {
    count as f64 * price
}

fn compare(a: &Option<i32>, b: &Option<i32>) -> &'static str {
    match (a, b) {
        (Some(x), Some(y)) if x == y => "same",
        (Some(x), Some(y)) if x > y => "left bigger",
        (Some(_), Some(_)) => "right bigger",
        (Some(_), None) | (None, Some(_)) => "one missing",
        (None, None) => "both missing",
    }
}

fn main() {
    let u = Vec2 { x: 1.0, y: 2.0 };
    let v = Vec2 { x: 3.0, y: -1.0 };
    println!("dot = {}", dot(u, v));
    println!("{:?}", swap((7, "seven")));
    let orders = vec![(2u32, 1.5f64), (1, 10.0), (4, 0.25)];
    let sum: f64 = orders.iter().map(total).sum();
    println!("sum = {}", sum);

    let points = vec![(1, 2), (3, 4), (5, 6)];
    let xs: Vec<i32> = points.iter().map(|&(x, _)| x).collect();
    let prods: Vec<i32> = points.iter().map(|(a, b)| a * b).collect();
    let big: Vec<&(i32, i32)> = points.iter().filter(|&&(a, b)| a + b > 5).collect();
    println!("{:?} {:?} {:?}", xs, prods, big);

    let names = ["ann", "bob", "cy"];
    for (i, &name) in names.iter().enumerate() {
        print!("{}:{} ", i, name);
    }
    println!();
    let nested = vec![((1, 'a'), "one"), ((2, 'b'), "two")];
    for &((n, c), word) in &nested {
        println!("{} {} {}", n, c, word);
    }
    let lookup = |key: char| nested.iter().find(|((_, c), _)| *c == key).map(|&(_, w)| w);
    println!("{:?} {:?}", lookup('b'), lookup('z'));

    let pairs = [(Some(3), Some(3)), (Some(5), Some(1)), (None, Some(2)), (None, None), (Some(0), Some(9))];
    for (a, b) in pairs.iter() {
        println!("{:?} vs {:?}: {}", a, b, compare(a, b));
    }
    let words = vec!["apple".to_string(), "kiwi".to_string(), "banana".to_string()];
    let by_len = |w: &&String| w.len();
    let longest = words.iter().max_by_key(by_len).unwrap();
    println!("longest {}", longest);
    let mut grid = vec![vec![0; 3]; 2];
    grid.iter_mut().enumerate().for_each(|(r, row)| {
        row.iter_mut().enumerate().for_each(|(c, cell)| *cell = r * 3 + c)
    });
    println!("{:?}", grid);
    let ref r = grid[1];
    let &[_, mid, _] = r.as_slice() else { return };
    println!("mid {}", mid);
}
