// Day 25 2022 (balanced base 5) + Day 6 2022 (marker detection with bitmasks).
const SNAFU: &str = r"1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122";

const SIGNALS: [&str; 5] = [
    "mjqjpqmgbljsphdztnvjfqwrcgsmlb",
    "bvwbjplbgvbhsrlpgdmjqwftvncz",
    "nppdvjthqldpwncqszvftbrmjlhg",
    "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg",
    "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw",
];

fn from_snafu(s: &str) -> i64 {
    s.chars().fold(0, |acc, c| {
        acc * 5
            + match c {
                '2' => 2,
                '1' => 1,
                '0' => 0,
                '-' => -1,
                '=' => -2,
                _ => panic!("bad digit {c:?}"),
            }
    })
}

fn to_snafu(mut n: i64) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut digits = Vec::new();
    while n != 0 {
        let r = n.rem_euclid(5);
        let (c, carry) = match r {
            0 => ('0', 0),
            1 => ('1', 0),
            2 => ('2', 0),
            3 => ('=', 1),
            4 => ('-', 1),
            _ => unreachable!(),
        };
        digits.push(c);
        n = n.div_euclid(5) + carry;
    }
    digits.iter().rev().collect()
}

fn marker(s: &str, len: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    bytes
        .windows(len)
        .position(|w| w.iter().fold(0u32, |m, &b| m | 1 << (b - b'a')).count_ones() as usize == len)
        .map(|p| p + len)
}

fn main() {
    let total: i64 = SNAFU.lines().map(from_snafu).sum();
    println!("sum {total} = {}", to_snafu(total));
    for n in [1, 3, 8, 2022, 12345, 314159265, -7] {
        let s = to_snafu(n);
        assert_eq!(from_snafu(&s), n);
        println!("{n:>10} {s:>15}");
    }
    for s in SIGNALS {
        println!("{:?} {:?}", marker(s, 4), marker(s, 14));
    }
    println!("{:?}", marker("abcabc", 4));
    println!("{}", (-17i64).rem_euclid(5) + (-17i64 % 5));
}
