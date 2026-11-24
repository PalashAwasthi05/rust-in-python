// Ranking word frequencies with iterator pipelines and multi-key sorts.
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Player {
    name: String,
    wins: u32,
    losses: u32,
}

impl Player {
    fn ratio(&self) -> f64 {
        if self.wins + self.losses == 0 {
            0.0
        } else {
            self.wins as f64 / (self.wins + self.losses) as f64
        }
    }
}

fn main() {
    let text = "it was the best of times it was the worst of times it was the age of wisdom";
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in text.split_whitespace() {
        *freq.entry(w).or_default() += 1;
    }
    let mut ranked: Vec<(&str, usize)> = freq.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (w, n) in ranked.iter().take(5) {
        println!("{:<6}{}", w, n);
    }

    let mut players = vec![
        Player { name: "zoe".into(), wins: 5, losses: 2 },
        Player { name: "amy".into(), wins: 5, losses: 2 },
        Player { name: "bob".into(), wins: 7, losses: 7 },
        Player { name: "cal".into(), wins: 0, losses: 0 },
        Player { name: "dan".into(), wins: 9, losses: 1 },
    ];
    players.sort_by(|a, b| {
        b.ratio()
            .partial_cmp(&a.ratio())
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    for (rank, p) in players.iter().enumerate() {
        println!("#{} {} {:.3}", rank + 1, p.name, p.ratio());
    }

    let mut by_len: Vec<&str> = text.split(' ').collect();
    by_len.sort_by_key(|w| (std::cmp::Reverse(w.len()), *w));
    by_len.dedup();
    println!("{:?}", &by_len[..6]);

    let mut pairs = vec![(3, 'c'), (1, 'z'), (3, 'a'), (2, 'b'), (1, 'y')];
    pairs.sort_by(|a, b| a.0.cmp(&b.0).reverse().then_with(|| b.1.cmp(&a.1)));
    println!("{:?}", pairs);

    let mut nums = vec![10, -3, 7, -8, 0, 4];
    nums.sort_unstable_by(|a: &i32, b: &i32| a.abs().cmp(&b.abs()).then(b.cmp(a)));
    println!("{:?}", nums);

    let best = players.iter().max_by(|a, b| a.wins.cmp(&b.wins).then(b.name.cmp(&a.name))).unwrap();
    println!("most wins: {}", best.name);
    let worst = players.iter().min_by_key(|p| (p.wins, std::cmp::Reverse(p.losses))).unwrap();
    println!("fewest wins: {}", worst.name);

    let grouped: Vec<(u32, Vec<String>)> = {
        let mut m: std::collections::BTreeMap<u32, Vec<String>> = std::collections::BTreeMap::new();
        for p in &players {
            m.entry(p.wins).or_default().push(p.name.clone());
        }
        m.into_iter().rev().collect()
    };
    println!("{:?}", grouped);

    let is_sorted = ranked.windows(2).all(|w| w[0].1 >= w[1].1);
    println!("ranked sorted: {}", is_sorted);
    let ordering = [1, 2, 3].iter().cmp([1, 2, 4].iter());
    println!("{:?}", ordering);
}
