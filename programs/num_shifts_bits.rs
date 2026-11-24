fn main() {
    let x: i32 = -16;
    println!("{} {}", x >> 2, (x as u32) >> 2);
    println!("{}", x << 3);
    println!("{}", 1u8 << 7);
    println!("{}", 1i32 << 31);
    println!("{}", 0x80u8 >> 7);
    println!("{}", -1i64 >> 63);
    println!("{}", (-1i64 as u64) >> 63);
    println!("{}", 1u64 << 63);
    println!("{}", 1u128 << 127);
    println!("{}", -1i128 >> 100);
    let s: u32 = 5;
    println!("{}", 3u16 << s);
    println!("{}", 0xFFu8 << 4);
    println!("{}", 0b1010_1010u8 & 0x0F);
    println!("{}", 0b1010u8 | 0b0101);
    println!("{}", 0xFFu8 ^ 0x0F);
    println!("{}", !0u8);
    println!("{}", !0i32);
    println!("{}", !5i32);
    println!("{}", !0u64);
    println!("{}", !0u128);
    println!("{}", -8i32 & 0xFF);
    println!("{}", -8i32 | 1);
    println!("{}", -8i32 ^ -1);
    let mut m = 0u32;
    m |= 1 << 4;
    m |= 1 << 9;
    m &= !(1 << 4);
    m ^= 0xF;
    m <<= 2;
    m >>= 1;
    println!("{}", m);
    println!("{:032b}", 0xDEADBEEFu32);
    println!("{:08b}", -3i8);
    println!("{:x} {:X} {:o}", 255, 255, 8);
    println!("{:#x} {:#b} {:#o}", 255, 5, 64);
    println!("{:x}", -1i32);
    println!("{:x}", -1i8);
    println!("{:#010x}", 0xBEEF);
    println!("{:x}", u128::MAX);
    // bit counting
    let v: u32 = 0b0001_0110_0000;
    println!("{} {} {} {}", v.count_ones(), v.count_zeros(), v.leading_zeros(), v.trailing_zeros());
    println!("{} {}", 0u32.leading_zeros(), 0u32.trailing_zeros());
    println!("{} {}", 0u8.leading_zeros(), 0u64.trailing_zeros());
    println!("{} {}", (-1i32).count_ones(), (-1i8).leading_zeros());
    println!("{} {}", i32::MIN.leading_zeros(), i32::MIN.trailing_zeros());
    println!("{} {}", 0xF0u8.leading_ones(), 0x0Fu8.trailing_ones());
    println!("{}", (-2i16).leading_ones());
    println!("{}", 0x12345678u32.rotate_left(8));
    println!("{}", 0x12345678u32.rotate_right(4));
    println!("{}", 0x81u8.rotate_left(1));
    println!("{}", (-2i8).rotate_left(1));
    println!("{}", i32::MIN.rotate_right(31));
    println!("{:x}", 0x12345678u32.swap_bytes());
    println!("{:x}", 0x1234u16.swap_bytes());
    println!("{}", 0x1234i16.swap_bytes());
    println!("{:x}", 0x0102030405060708u64.swap_bytes());
    println!("{:08b}", 0b0000_0110u8.reverse_bits());
    println!("{}", 1i32.reverse_bits());
    println!("{} {}", 64u32.is_power_of_two(), 0u32.is_power_of_two());
    println!("{} {}", 17u32.next_power_of_two(), 0u8.next_power_of_two());
    println!("{:?} {:?}", 1u32.checked_shl(31), 1u32.checked_shl(32));
    println!("{:?}", 8i32.checked_shr(40));
    println!("{} {}", 1u32.wrapping_shl(33), (-128i8).wrapping_shr(9));
    println!("{:?}", 1u8.overflowing_shl(9));
    // popcount loop
    let mut total = 0;
    for i in 0u32..256 {
        total += i.count_ones();
    }
    println!("{}", total);
    // bit tricks
    let n: u32 = 40;
    println!("{}", n & n.wrapping_neg());
    println!("{}", n & (n - 1));
    let mut bits = 0b1011_0100u8;
    let mut positions = Vec::new();
    while bits != 0 {
        positions.push(bits.trailing_zeros());
        bits &= bits - 1;
    }
    println!("{:?}", positions);
}
