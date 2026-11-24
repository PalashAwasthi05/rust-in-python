// Day 2 2023: parsing games with nested separators, iterator-heavy.
const INPUT: &str = r"Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Rgb {
    r: u32,
    g: u32,
    b: u32,
}

impl Rgb {
    fn max(self, o: Rgb) -> Rgb {
        Rgb { r: self.r.max(o.r), g: self.g.max(o.g), b: self.b.max(o.b) }
    }
    fn power(&self) -> u32 {
        self.r * self.g * self.b
    }
}

fn parse_draw(s: &str) -> Rgb {
    let mut rgb = Rgb::default();
    for part in s.split(',') {
        let (n, color) = part.trim().split_once(' ').unwrap();
        let n: u32 = n.parse().unwrap();
        match color {
            "red" => rgb.r = n,
            "green" => rgb.g = n,
            "blue" => rgb.b = n,
            _ => unreachable!("color {}", color),
        }
    }
    rgb
}

fn main() {
    let games: Vec<(u32, Vec<Rgb>)> = INPUT
        .lines()
        .map(|line| {
            let (head, draws) = line.split_once(": ").unwrap();
            let id = head["Game ".len()..].parse().unwrap();
            (id, draws.split("; ").map(parse_draw).collect())
        })
        .collect();
    let limit = Rgb { r: 12, g: 13, b: 14 };
    let part1: u32 = games
        .iter()
        .filter(|(_, draws)| draws.iter().all(|d| d.r <= limit.r && d.g <= limit.g && d.b <= limit.b))
        .map(|(id, _)| id)
        .sum();
    println!("part 1: {}", part1);
    let mins: Vec<Rgb> = games
        .iter()
        .map(|(_, d)| d.iter().fold(Rgb::default(), |acc, &x| acc.max(x)))
        .collect();
    for (i, m) in mins.iter().enumerate() {
        println!("game {}: {:?} power {}", i + 1, m, m.power());
    }
    println!("part 2: {}", mins.iter().map(Rgb::power).sum::<u32>());
    let best = games.iter().max_by_key(|(_, d)| d.len()).map(|(id, _)| *id);
    println!("most draws: {:?}", best);
}
