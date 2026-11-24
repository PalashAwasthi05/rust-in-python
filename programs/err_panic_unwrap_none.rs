fn find(v: &[i32], target: i32) -> Option<usize> {
    v.iter().position(|&x| x == target)
}

fn main() {
    let v = vec![3, 1, 4, 1, 5];
    println!("found 4 at {}", find(&v, 4).unwrap());
    print!("looking for 9... ");
    let idx = find(&v, 9).unwrap();
    println!("found 9 at {}", idx);
}
