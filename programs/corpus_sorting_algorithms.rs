fn quicksort(v: &mut [i32]) {
    if v.len() <= 1 { return; }
    let p = partition(v);
    let (left, right) = v.split_at_mut(p);
    quicksort(left);
    quicksort(&mut right[1..]);
}

fn partition(v: &mut [i32]) -> usize {
    let hi = v.len() - 1;
    let pivot = v[hi];
    let mut i = 0;
    for j in 0..hi {
        if v[j] <= pivot { v.swap(i, j); i += 1; }
    }
    v.swap(i, hi);
    i
}

fn merge_sort(v: Vec<i32>) -> Vec<i32> {
    if v.len() <= 1 { return v; }
    let mid = v.len() / 2;
    let right = merge_sort(v[mid..].to_vec());
    let left = merge_sort(v[..mid].to_vec());
    let mut out = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] { out.push(left[i]); i += 1; } else { out.push(right[j]); j += 1; }
    }
    out.extend_from_slice(&left[i..]);
    out.extend_from_slice(&right[j..]);
    out
}

fn insertion_sort(v: &mut Vec<i32>) -> usize {
    let mut shifts = 0;
    for i in 1..v.len() {
        let key = v[i];
        let mut j = i;
        while j > 0 && v[j - 1] > key {
            v[j] = v[j - 1];
            j -= 1;
            shifts += 1;
        }
        v[j] = key;
    }
    shifts
}

fn bubble_sort(v: &mut [i32]) -> u32 {
    let mut passes = 0;
    loop {
        let mut swapped = false;
        passes += 1;
        for i in 1..v.len() {
            if v[i - 1] > v[i] { v.swap(i - 1, i); swapped = true; }
        }
        if !swapped { break passes; }
    }
}

fn is_sorted(v: &[i32]) -> bool { v.windows(2).all(|w| w[0] <= w[1]) }

fn main() {
    let data = vec![38, 27, 43, 3, 9, 82, 10, -5, 27, 0, 64, -12];
    let mut q = data.clone();
    quicksort(&mut q);
    println!("quicksort:  {:?}", q);
    let m = merge_sort(data.clone());
    println!("merge sort: {:?}", m);
    let mut ins = data.clone();
    let shifts = insertion_sort(&mut ins);
    println!("insertion:  {:?} ({} shifts)", ins, shifts);
    let mut b = data.clone();
    let passes = bubble_sort(&mut b);
    println!("bubble:     {:?} ({} passes)", b, passes);
    let mut std_sorted = data.clone();
    std_sorted.sort();
    println!("all agree: {}", q == m && m == ins && ins == b && b == std_sorted);
    println!("sorted? {} {}", is_sorted(&data), is_sorted(&q));
    let mut desc = data.clone();
    desc.sort_by(|a, b| b.cmp(a));
    println!("descending: {:?}", desc);
    println!("binary_search 43: {:?}, 44: {:?}", q.binary_search(&43), q.binary_search(&44));
    let mut dedup = q.clone();
    dedup.dedup();
    println!("dedup: {:?} median: {}", dedup, q[q.len() / 2]);
}
