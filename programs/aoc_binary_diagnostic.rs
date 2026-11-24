// Day 3 2021: bit counting and filtering.
const INPUT: &str = r"00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010";

fn most_common(nums: &[u32], bit: usize) -> u32 {
    let ones = nums.iter().filter(|&&n| n >> bit & 1 == 1).count();
    if ones * 2 >= nums.len() { 1 } else { 0 }
}

fn rating(mut nums: Vec<u32>, width: usize, keep_most: bool) -> u32 {
    for bit in (0..width).rev() {
        if nums.len() == 1 {
            break;
        }
        let mc = most_common(&nums, bit);
        let want = if keep_most { mc } else { mc ^ 1 };
        nums.retain(|&n| (n >> bit) & 1 == want);
    }
    nums[0]
}

fn main() {
    let width = INPUT.lines().next().unwrap().len();
    let nums: Vec<u32> = INPUT.lines().map(|l| u32::from_str_radix(l, 2).unwrap()).collect();
    let gamma = (0..width).fold(0, |acc, bit| acc | most_common(&nums, bit) << bit);
    let epsilon = !gamma & ((1 << width) - 1);
    println!("gamma {gamma:0w$b} epsilon {epsilon:0w$b}", w = width);
    println!("part 1: {}", gamma * epsilon);
    let oxygen = rating(nums.clone(), width, true);
    let co2 = rating(nums.clone(), width, false);
    println!("oxygen {oxygen} co2 {co2}");
    println!("part 2: {}", oxygen * co2);
    let xor_all = nums.iter().fold(0u32, |a, &b| a ^ b);
    println!("xor {:#07b}, leading zeros {}, rotate {}", xor_all, xor_all.leading_zeros(), xor_all.rotate_left(3));
}
