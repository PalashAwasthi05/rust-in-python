fn sum_slice(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

fn describe(slice: &[i32]) -> String {
    format!("len {} first {:?} last {:?}", slice.len(), slice.first(), slice.last())
}

fn reverse(pair: (i32, bool)) -> (bool, i32) {
    let (int_param, bool_param) = pair;
    (bool_param, int_param)
}

fn main() {
    // literals with suffixes and underscores
    let a = 1_000_000u32;
    let b = 0xff_u8;
    let c = 0o7_7i16;
    let d = 0b1111_0000u8;
    let e = 1e3_f64;
    let f = 2.5e-3f32;
    let big = 9_223_372_036_854_775_807i64;
    println!("{} {} {} {} {} {} {}", a, b, c, d, e, f, big);
    println!("byte literals: {} {:?} {:?}", b'A', b"hi", br"raw\n");
    println!("char sizes: {} {}", std::mem::size_of::<char>(), '🦀'.len_utf8());
    println!("bool ops: {} {} {}", true && false, true || false, !true);
    println!("int ops: {} {} {} {}", 7 / 2, -7 / 2, 7 % 3, -7 % 3);
    println!("float ops: {} {} {}", 7.0 / 2.0, 1.0f64 - 0.9, (0.1f64 + 0.2).to_bits() == 0.3f64.to_bits());
    println!("bit ops: {:04b} {:04b} {:04b} {} {}", 0b0011u32 & 0b0101, 0b0011u32 | 0b0101, 0b0011u32 ^ 0b0101, 1u32 << 5, 0x80u32 >> 2);
    println!("unit value: {:?}", ());

    // tuples
    let long_tuple = (1u8, 2u16, 3u32, 4u64, -1i8, -2i16, 0.1f32, 'a', true);
    println!("first {} fifth {} last {}", long_tuple.0, long_tuple.4, long_tuple.8);
    let nested = ((1u8, 2u16), (4u32, -1i8), -2i16);
    println!("nested: {:?}", nested);
    println!("single-element tuple {:?} vs plain {:?}", (5u32,), (5u32));
    println!("reversed: {:?}", reverse((42, true)));
    let (x, y, z) = (1, "two", 3.0);
    println!("destructured {} {} {}", x, y, z);
    let mut point = (0, 0);
    point.0 += 5;
    point.1 -= 2;
    println!("point {:?}", point);

    // arrays and slices
    let xs: [i32; 5] = [1, 2, 3, 4, 5];
    let ys: [i32; 6] = [0; 6];
    println!("xs {:?} ys {:?}", xs, ys);
    println!("xs[0]={} xs[4]={} len={} size={} bytes", xs[0], xs[4], xs.len(), std::mem::size_of_val(&xs));
    println!("whole: {}", describe(&xs));
    println!("part: {}", describe(&xs[1..4]));
    println!("empty: {}", describe(&ys[..0]));
    println!("sum of xs[2..] = {}", sum_slice(&xs[2..]));
    for i in 0..xs.len() + 1 {
        match xs.get(i) {
            Some(v) => print!("{}:{} ", i, v),
            None => println!("index {} out of range", i),
        }
    }
    let grid = [[1, 2, 3], [4, 5, 6]];
    let col_sums: Vec<i32> = (0..3).map(|c| grid.iter().map(|row| row[c]).sum()).collect();
    println!("grid {:?} column sums {:?}", grid, col_sums);
    let mut arr = [5, 3, 8, 1];
    arr.sort();
    arr.reverse();
    println!("sorted desc {:?}, contains 8: {}", arr, arr.contains(&8));
    let (left, right) = arr.split_at(2);
    println!("split {:?} {:?}", left, right);
}
