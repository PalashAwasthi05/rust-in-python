fn total(v: &[u32], upto: usize) -> u32 {
    let mut sum = 0;
    for i in 0..upto {
        sum += v[i];
        println!("after {} sum is {}", i, sum);
    }
    sum
}

fn main() {
    let data = vec![10, 20, 30];
    println!("total {}", total(&data, 2));
    println!("total {}", total(&data, 5));
}
