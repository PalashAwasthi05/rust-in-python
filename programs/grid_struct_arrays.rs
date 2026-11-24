#[derive(Debug, Clone, PartialEq)]
struct Tile {
    label: String,
    neighbors: [Option<usize>; 4],
}

fn transpose<T: Copy, const R: usize, const C: usize>(m: [[T; C]; R]) -> [[T; R]; C] {
    let mut out = [[m[0][0]; R]; C];
    for r in 0..R {
        for c in 0..C {
            out[c][r] = m[r][c];
        }
    }
    out
}

fn main() {
    let tiles: [Tile; 3] = std::array::from_fn(|i| Tile {
        label: format!("t{}", i),
        neighbors: [if i > 0 { Some(i - 1) } else { None }, if i < 2 { Some(i + 1) } else { None }, None, None],
    });
    for t in &tiles {
        println!("{} {:?}", t.label, t.neighbors);
    }
    let [first, second, third] = tiles;
    println!("{} {} {}", first.label, second.label, third.label);
    let t = transpose([[1, 2, 3], [4, 5, 6]]);
    println!("{:?}", t);
    let t2 = transpose(t);
    println!("{:?}", t2);
    let mut labels = [String::new(), String::new()];
    for (i, l) in labels.iter_mut().enumerate() {
        l.push_str(&"#".repeat(i + 2));
    }
    println!("{:?}", labels);
    let total_len: usize = labels.iter().map(String::len).sum();
    println!("{}", total_len);
    let refs: [&str; 2] = labels.each_ref().map(|s| s.as_str());
    println!("{:?}", refs);
    let mut counters = [0u32; 3];
    let bump = |arr: &mut [u32; 3], i: usize| arr[i] += 1;
    bump(&mut counters, 1);
    bump(&mut counters, 1);
    bump(&mut counters, 2);
    println!("{:?}", counters);
    let grid = vec![[1u8, 2], [3, 4]];
    let sum: u32 = grid.iter().flat_map(|row| row.iter()).map(|&v| v as u32).sum();
    println!("{}", sum);
    let mut opt_grid: [[Option<char>; 2]; 2] = [[None; 2]; 2];
    opt_grid[0][1] = Some('q');
    let shown: Vec<String> = opt_grid
        .iter()
        .map(|row| row.iter().map(|c| c.unwrap_or('_')).collect())
        .collect();
    println!("{}", shown.join("\n"));
    let arr = [10, 20, 30, 40];
    let (evens, odds): (Vec<usize>, Vec<usize>) = (0..arr.len()).partition(|i| i % 2 == 0);
    println!("{:?} {:?}", evens, odds);
    let picked: Vec<i32> = evens.iter().map(|&i| arr[i]).collect();
    println!("{:?}", picked);
    let mut stack = [0; 8];
    let mut top = 0;
    for x in [5, 6, 7] {
        stack[top] = x;
        top += 1;
    }
    while top > 0 {
        top -= 1;
        print!("{} ", stack[top]);
    }
    println!();
}
