// Longest common subsequence of two 300-character strings with a 2D DP table.
fn pseudo_text(seed: u32, length: usize) -> String {
    let mut state = seed;
    let mut text = String::new();
    for _ in 0..length {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        let letter = (b'a' + ((state >> 16) % 4) as u8) as char;
        text.push(letter);
    }
    text
}

fn lcs(a: &str, b: &str) -> (usize, String) {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut table = vec![vec![0usize; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            table[i][j] = if a[i - 1] == b[j - 1] {
                table[i - 1][j - 1] + 1
            } else if table[i - 1][j] >= table[i][j - 1] {
                table[i - 1][j]
            } else {
                table[i][j - 1]
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            out.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if table[i - 1][j] >= table[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    out.reverse();
    (table[n][m], out.into_iter().collect())
}

fn main() {
    let a = pseudo_text(7, 300);
    let b = pseudo_text(42, 300);
    println!("{}", &a[..40]);
    println!("{}", &b[..40]);
    let (length, common) = lcs(&a, &b);
    println!("lcs length: {}", length);
    println!("lcs prefix: {}", &common[..30]);
}
