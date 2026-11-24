// Mutable row borrows, iter_mut over 2D, swapping cells and rows, split_at_mut.
fn show(g: &[[i32; 4]; 3]) {
    for row in g {
        println!("{:?}", row);
    }
    println!("--");
}

fn bump_all(g: &mut [[i32; 4]; 3]) {
    for row in g.iter_mut() {
        for cell in row.iter_mut() {
            *cell += 1;
        }
    }
}

fn double_row(row: &mut [i32; 4]) {
    for x in row {
        *x *= 2;
    }
}

fn main() {
    let mut g = [[0i32; 4]; 3];
    for (r, row) in g.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (r * 4 + c) as i32;
        }
    }
    show(&g);
    bump_all(&mut g);
    show(&g);

    // swap individual cells
    let tmp = g[0][0];
    g[0][0] = g[2][3];
    g[2][3] = tmp;
    g[1][1] = g[1][2] + g[0][1];
    show(&g);

    // swap whole rows
    g.swap(0, 2);
    show(&g);

    // swap rows with split_at_mut and mem::swap
    {
        let (top, bottom) = g.split_at_mut(1);
        std::mem::swap(&mut top[0], &mut bottom[1]);
    }
    show(&g);

    // swap elements between two rows
    {
        let (a, b) = g.split_at_mut(2);
        std::mem::swap(&mut a[0][3], &mut b[0][0]);
    }
    show(&g);

    double_row(&mut g[1]);
    show(&g);

    let row_sums: Vec<i32> = g.iter().map(|r| r.iter().sum()).collect();
    println!("row sums {:?}", row_sums);
    let col_sums: Vec<i32> = (0..4).map(|c| g.iter().map(|r| r[c]).sum()).collect();
    println!("col sums {:?}", col_sums);

    // Vec<Vec<_>> version
    let mut v: Vec<Vec<char>> = vec![vec!['a', 'b', 'c'], vec!['d', 'e', 'f']];
    v[0].swap(0, 2);
    let t = v[1][0];
    v[1][0] = v[0][1];
    v[0][1] = t;
    v.swap(0, 1);
    for row in v.iter_mut() {
        row.reverse();
    }
    for row in &mut v {
        row.push('!');
    }
    println!("{:?}", v);
    let last = v.last_mut().unwrap();
    last[0] = 'Z';
    if let Some(first) = v.first_mut() {
        first.insert(0, '>');
    }
    println!("{:?}", v);

    // rotate rows in place
    let mut ring = [1, 2, 3, 4, 5];
    ring.rotate_left(2);
    println!("{:?}", ring);
    ring.rotate_right(1);
    println!("{:?}", ring);
    ring.reverse();
    println!("{:?}", ring);
    ring.fill(7);
    println!("{:?}", ring);
}
