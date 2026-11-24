// Sorting arrays in place and searching them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Card {
    rank: u8,
    suit: char,
}

fn main() {
    let mut a = [9, 3, 7, 1, 8, 2, 6];
    a.sort();
    println!("{:?}", a);
    a.sort_unstable_by(|x, y| y.cmp(x));
    println!("{:?}", a);
    a.sort_by_key(|&x| (x % 3, x));
    println!("{:?}", a);
    a.sort_unstable();
    println!("{:?} sorted={}", a, a.windows(2).all(|w| w[0] <= w[1]));
    println!("{:?} {:?}", a.binary_search(&7), a.binary_search(&5));
    println!("{:?}", a.iter().position(|&x| x > 5));
    println!("{:?}", a.iter().find(|&&x| x % 4 == 0));
    println!("{:?}", a.iter().max_by_key(|&&x| (x as i32 - 5).abs()));

    let mut fl = [3.2f64, -1.0, 2.5, 0.0];
    fl.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("{:?}", fl);
    fl.sort_by(|x, y| y.total_cmp(x));
    println!("{:?}", fl);

    let mut words = ["pear", "fig", "banana", "kiwi", "apple"];
    words.sort();
    println!("{:?}", words);
    words.sort_by(|x, y| x.len().cmp(&y.len()).then_with(|| y.cmp(x)));
    println!("{:?}", words);
    words.sort_by_key(|w| std::cmp::Reverse(w.chars().last()));
    println!("{:?}", words);

    let mut hand = [
        Card { rank: 10, suit: 'h' },
        Card { rank: 2, suit: 's' },
        Card { rank: 10, suit: 'c' },
        Card { rank: 14, suit: 'd' },
    ];
    hand.sort();
    println!("{:?}", hand);
    hand.sort_by(|a, b| b.suit.cmp(&a.suit));
    for c in &hand {
        print!("{}{} ", c.rank, c.suit);
    }
    println!();
    println!("{:?}", hand.iter().max());
    println!("{}", hand.contains(&Card { rank: 2, suit: 's' }));

    let mut grid = [[3, 1, 2], [9, 8, 7], [5, 6, 4]];
    for row in grid.iter_mut() {
        row.sort();
    }
    println!("{:?}", grid);
    grid.sort_by_key(|r| std::cmp::Reverse(r[0]));
    println!("{:?}", grid);
    grid.sort();
    println!("{:?}", grid);

    let mut chars = ['d', 'a', 'c', 'b'];
    chars.reverse();
    println!("{:?}", chars);
    chars.sort_unstable();
    let s: String = chars.iter().collect();
    println!("{}", s);
    let mut nums = [5u32, 3, 5, 1, 3];
    nums.sort();
    let mut v = nums.to_vec();
    v.dedup();
    println!("{:?} {:?}", nums, v);
    println!("{:?}", nums.iter().min_by_key(|&&x| x));
    println!("select: {:?}", { let mut t = [4, 9, 1, 7]; t.select_nth_unstable(2); t[2] });
}
