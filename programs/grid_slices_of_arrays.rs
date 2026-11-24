// Slices of arrays passed to &[T] and &mut [T] functions.
fn total(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

fn negate(xs: &mut [i32]) {
    for x in xs.iter_mut() {
        *x = -*x;
    }
}

fn max_window(xs: &[i32], k: usize) -> (usize, i32) {
    let mut best = (0, i32::MIN);
    for (i, w) in xs.windows(k).enumerate() {
        let s = total(w);
        if s > best.1 {
            best = (i, s);
        }
    }
    best
}

fn bubble(xs: &mut [i32]) {
    let n = xs.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if xs[j] > xs[j + 1] {
                xs.swap(j, j + 1);
            }
        }
    }
}

fn describe(xs: &[i32]) -> String {
    match xs {
        [] => "empty".to_string(),
        [x] => format!("one: {}", x),
        [first, .., last] => format!("{} items, {}..{}", xs.len(), first, last),
    }
}

fn main() {
    let mut arr = [5, -2, 9, 1, 7, 3, 8];
    println!("total {}", total(&arr));
    println!("middle {:?} = {}", &arr[1..3], total(&arr[1..3]));
    println!("head {:?} tail {:?}", &arr[..2], &arr[5..]);
    negate(&mut arr[2..4]);
    println!("{:?}", arr);
    negate(&mut arr[2..4]);
    println!("{:?}", max_window(&arr, 3));
    bubble(&mut arr[..4]);
    println!("{:?}", arr);
    bubble(&mut arr);
    println!("{:?}", arr);
    println!("{}", describe(&arr));
    println!("{}", describe(&arr[3..4]));
    println!("{}", describe(&arr[3..3]));

    for chunk in arr.chunks(3) {
        println!("chunk {:?} sum {}", chunk, total(chunk));
    }
    for chunk in arr.chunks_mut(2) {
        chunk[0] += 100;
    }
    println!("{:?}", arr);

    let (left, right) = arr.split_at(3);
    println!("{:?} | {:?}", left, right);
    let (l, r) = arr.split_at_mut(4);
    l[0] = r[0];
    println!("{:?}", arr);

    let grid = [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]];
    let inner: Vec<i32> = grid[1..].iter().map(|row| total(&row[1..3])).collect();
    println!("inner sums {:?}", inner);
    let s: &[[i32; 4]] = &grid;
    println!("rows {} first row {:?}", s.len(), s[0]);
    println!("contains row {}", grid.contains(&[5, 6, 7, 8]));
    println!("starts_with {} ends_with {}", arr.starts_with(&[108]), arr.ends_with(&[9]));
    let v = arr.to_vec();
    println!("{:?} {}", v, v.len());
    let mut copy = [0; 3];
    copy.copy_from_slice(&arr[1..4]);
    println!("{:?}", copy);
    println!("{:?}", arr.iter().skip(2).step_by(2).collect::<Vec<_>>());
    println!("{:?}", arr.binary_search(&7));
}
