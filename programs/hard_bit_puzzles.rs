fn is_power_of_two(n: u32) -> bool { n != 0 && n & (n - 1) == 0 }

fn single_number(nums: &[i32]) -> i32 { nums.iter().fold(0, |acc, &x| acc ^ x) }

fn reverse_bits(mut n: u32) -> u32 {
    let mut r = 0;
    for _ in 0..32 {
        r = (r << 1) | (n & 1);
        n >>= 1;
    }
    r
}

fn popcount_manual(mut n: u64) -> u32 {
    let mut c = 0;
    while n != 0 {
        n &= n - 1;
        c += 1;
    }
    c
}

fn gray(n: u32) -> u32 { n ^ (n >> 1) }

fn subsets<T: Copy>(items: &[T]) -> Vec<Vec<T>> {
    (0..1u32 << items.len())
        .map(|mask| (0..items.len()).filter(|i| mask >> i & 1 == 1).map(|i| items[i]).collect())
        .collect()
}

fn swap_xor(a: &mut u32, b: &mut u32) {
    *a ^= *b;
    *b ^= *a;
    *a ^= *b;
}

#[derive(Clone, Copy)]
struct Flags(u8);

impl Flags {
    const READ: u8 = 0b001;
    const WRITE: u8 = 0b010;
    const EXEC: u8 = 0b100;
    fn has(self, f: u8) -> bool { self.0 & f == f }
    fn set(self, f: u8) -> Flags { Flags(self.0 | f) }
    fn clear(self, f: u8) -> Flags { Flags(self.0 & !f) }
    fn toggle(self, f: u8) -> Flags { Flags(self.0 ^ f) }
    fn show(self) -> String {
        [(Self::READ, 'r'), (Self::WRITE, 'w'), (Self::EXEC, 'x')].iter().map(|&(f, c)| if self.has(f) { c } else { '-' }).collect()
    }
}

fn main() {
    let powers: Vec<u32> = (0..70).filter(|&n| is_power_of_two(n)).collect();
    println!("powers of two < 70: {:?}", powers);
    println!("single number: {}", single_number(&[4, 7, 2, 7, 4, 9, 2]));
    println!("reverse_bits(1) = {:#034b}", reverse_bits(1));
    println!("reverse_bits(43261596) = {}", reverse_bits(43261596));
    for n in [0u64, 7, 255, 1 << 40, u64::MAX] {
        println!("popcount({}) = {} (builtin {})", n, popcount_manual(n), n.count_ones());
    }
    let codes: Vec<String> = (0..8).map(|i| format!("{:03b}", gray(i))).collect();
    println!("gray codes: {}", codes.join(" "));
    println!("subsets of [a,b,c]: {:?}", subsets(&['a', 'b', 'c']));
    let (mut a, mut b) = (12u32, 99u32);
    swap_xor(&mut a, &mut b);
    println!("after xor swap: a={} b={}", a, b);
    let f = Flags(0).set(Flags::READ | Flags::WRITE);
    println!("flags: {} -> {} -> {}", f.show(), f.clear(Flags::WRITE).show(), f.toggle(Flags::EXEC | Flags::READ).show());
    println!("lowest set bit of 40: {}, highest: {}", 40u32 & 40u32.wrapping_neg(), 31 - 40u32.leading_zeros());
    println!("rotate_left(0x80000001, 4) = {:#x}", 0x8000_0001u32.rotate_left(4));
    println!("i8 -1 as u8 = {}, -128 >> 2 = {}, 0xF0u8 >> 4 = {}", -1i8 as u8, -128i8 >> 2, 0xF0u8 >> 4);
    println!("next power of two of 100: {}", 100u32.next_power_of_two());
}
