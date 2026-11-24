fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn fnv1a_32(data: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for b in data.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn djb2(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u64);
    }
    hash
}

fn sdbm(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in s.bytes() {
        hash = (c as u32).wrapping_add(hash << 6).wrapping_add(hash << 16).wrapping_sub(hash);
    }
    hash
}

fn murmur_mix(mut k: u64) -> u64 {
    k ^= k >> 33;
    k = k.wrapping_mul(0xff51afd7ed558ccd);
    k ^= k >> 33;
    k = k.wrapping_mul(0xc4ceb9fe1a85ec53);
    k ^= k >> 33;
    k
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &d in data {
        a = (a + d as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (!(crc & 1)).wrapping_add(1);
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    !crc
}

fn java_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.chars() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

fn main() {
    for word in ["", "a", "hello", "The quick brown fox jumps over the lazy dog"] {
        println!("{:016x} {:08x} {} {} {:08x} {:08x} {}", fnv1a(word.as_bytes()), fnv1a_32(word), djb2(word),
                 sdbm(word), adler32(word.as_bytes()), crc32(word.as_bytes()), java_hash(word));
    }
    for k in [0u64, 1, 42, u64::MAX] {
        println!("{:016x}", murmur_mix(k));
    }
    let mut buckets = [0usize; 8];
    for i in 0..1000u64 {
        buckets[(murmur_mix(i) % 8) as usize] += 1;
    }
    println!("{:?}", buckets);
    let combined = fnv1a(b"abc").rotate_left(17) ^ djb2("abc");
    println!("{}", combined);
}
