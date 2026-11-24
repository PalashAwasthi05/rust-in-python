fn main() {
    let x: u32 = 0x12345678;
    println!("{:?}", x.to_be_bytes());
    println!("{:?}", x.to_le_bytes());
    println!("{:?}", x.to_ne_bytes());
    println!("{:?}", (-2i16).to_be_bytes());
    println!("{:?}", 1u64.to_le_bytes());
    let b = [0x78u8, 0x56, 0x34, 0x12];
    println!("{}", u32::from_le_bytes(b));
    println!("{}", u32::from_be_bytes(b));
    println!("{}", i16::from_be_bytes([0xFF, 0xFE]));
    println!("{}", u64::from_be_bytes([0, 0, 0, 0, 0, 0, 1, 0]));
    let bytes = 0xCAFEBABEu32.to_be_bytes();
    let back = u32::from_be_bytes(bytes);
    println!("{:x}", back);
    let mut buf = Vec::new();
    buf.extend_from_slice(&1000u16.to_le_bytes());
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    println!("{:?}", buf);
    let arr: [u8; 2] = [buf[0], buf[1]];
    println!("{}", u16::from_le_bytes(arr));
    // parsing
    println!("{:?}", i32::from_str_radix("-ff", 16));
    println!("{:?}", u8::from_str_radix("11111111", 2));
    println!("{:?}", u8::from_str_radix("100000000", 2).is_err());
    println!("{:?}", i64::from_str_radix("zz", 36));
    println!("{:?}", u32::from_str_radix("777", 8));
    println!("{:?}", "42".parse::<i32>());
    println!("{:?}", "-42".parse::<u32>().is_err());
    println!("{:?}", "+7".parse::<u8>());
    println!("{:?}", "256".parse::<u8>().map_err(|e| e.to_string()));
    println!("{:?}", "-129".parse::<i8>().map_err(|e| e.to_string()));
    println!("{:?}", "".parse::<i32>().map_err(|e| e.to_string()));
    println!("{:?}", "12a".parse::<i32>().map_err(|e| e.to_string()));
    println!("{:?}", " 12".parse::<i32>().is_err());
    println!("{:?}", "3.25".parse::<f64>());
    println!("{:?}", "1e3".parse::<f64>());
    println!("{:?}", "inf".parse::<f64>());
    println!("{:?}", "-0".parse::<f64>());
    println!("{:?}", "abc".parse::<f64>().map_err(|e| e.to_string()));
    println!("{:?}", "0.1".parse::<f32>());
    println!("{:?}", "340282366920938463463374607431768211455".parse::<u128>());
    let n: u64 = "18446744073709551615".parse().unwrap();
    println!("{}", n);
    let hex = "1A2B";
    let mut acc = 0u32;
    for c in hex.chars() {
        acc = acc * 16 + c.to_digit(16).unwrap();
    }
    println!("{}", acc);
    println!("{}", format!("{:x}", acc));
}
