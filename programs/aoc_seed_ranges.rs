// Day 5 2023: range mapping with i64/u64 arithmetic and range splitting.
const INPUT: &str = r"seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

struct Map {
    name: String,
    rules: Vec<(i64, i64, i64)>,
}

impl Map {
    fn apply(&self, x: i64) -> i64 {
        for &(dst, src, len) in &self.rules {
            if (src..src + len).contains(&x) {
                return x - src + dst;
            }
        }
        x
    }

    fn apply_ranges(&self, mut todo: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
        let mut done = Vec::new();
        while let Some((lo, hi)) = todo.pop() {
            let mut matched = false;
            for &(dst, src, len) in &self.rules {
                let (a, b) = (lo.max(src), hi.min(src + len));
                if a < b {
                    done.push((a - src + dst, b - src + dst));
                    if lo < a {
                        todo.push((lo, a));
                    }
                    if b < hi {
                        todo.push((b, hi));
                    }
                    matched = true;
                    break;
                }
            }
            if !matched {
                done.push((lo, hi));
            }
        }
        done
    }
}

fn main() {
    let mut blocks = INPUT.split("\n\n");
    let seeds: Vec<i64> = blocks
        .next()
        .unwrap()
        .strip_prefix("seeds: ")
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();
    let maps: Vec<Map> = blocks
        .map(|b| {
            let mut lines = b.lines();
            let name = lines.next().unwrap().trim_end_matches(" map:").to_string();
            let rules = lines
                .map(|l| {
                    let v: Vec<i64> = l.split(' ').map(|n| n.parse().unwrap()).collect();
                    (v[0], v[1], v[2])
                })
                .collect();
            Map { name, rules }
        })
        .collect();
    for m in &maps {
        println!("{} ({} rules)", m.name, m.rules.len());
    }
    let p1 = seeds
        .iter()
        .map(|&s| maps.iter().fold(s, |x, m| m.apply(x)))
        .min()
        .unwrap();
    println!("part 1: {p1}");
    let ranges: Vec<(i64, i64)> = seeds.chunks(2).map(|c| (c[0], c[0] + c[1])).collect();
    let fin = maps.iter().fold(ranges, |rs, m| m.apply_ranges(rs));
    let p2 = fin.iter().map(|r| r.0).min().unwrap();
    println!("part 2: {p2} from {} ranges", fin.len());
    let big: u64 = seeds.iter().map(|&s| s as u64 * 3_000_000_007).sum();
    println!("checksum: {}", big % 1_000_000_007);
}
