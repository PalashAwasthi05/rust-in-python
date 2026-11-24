// Chapter 2: the guessing game, with a seeded generator and scripted input lines instead of stdin.
use std::cmp::Ordering;

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    fn gen_range(&mut self, low: u32, high_inclusive: u32) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let span = (high_inclusive - low + 1) as u64;
        low + ((self.state >> 33) % span) as u32
    }
}

fn play(secret_number: u32, lines: &[&str]) -> Option<usize> {
    println!("Guess the number!");
    let mut attempts = 0;
    for line in lines {
        println!("Please input your guess.");
        let guess = String::from(*line);
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("(ignoring {:?})", guess.trim());
                continue;
            }
        };
        attempts += 1;
        println!("You guessed: {guess}");
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                return Some(attempts);
            }
        }
    }
    None
}

fn bisect(secret: u32) -> Vec<String> {
    let (mut low, mut high) = (1, 100);
    let mut guesses = Vec::new();
    loop {
        let mid = (low + high) / 2;
        guesses.push(mid.to_string());
        match mid.cmp(&secret) {
            Ordering::Less => low = mid + 1,
            Ordering::Greater => high = mid - 1,
            Ordering::Equal => break,
        }
    }
    guesses
}

fn main() {
    let mut rng = Lcg::new(2024);
    let secret_number = rng.gen_range(1, 100);
    println!("(secret is {secret_number})");
    let script = ["50", "  25\n", "abc", "", "75", "62", "56"];
    match play(secret_number, &script) {
        Some(n) => println!("won in {n} attempts"),
        None => println!("ran out of input"),
    }

    let mut wins = 0;
    for round in 0..5 {
        let secret = rng.gen_range(1, 100);
        let guesses = bisect(secret);
        let lines: Vec<&str> = guesses.iter().map(|s| s.as_str()).collect();
        println!("--- round {round}: secret {secret}, {} guesses", lines.len());
        if let Some(n) = play(secret, &lines) {
            assert_eq!(n, lines.len());
            wins += 1;
        }
    }
    println!("wins: {wins}");
    let rolls: Vec<u32> = (0..10).map(|_| rng.gen_range(1, 6)).collect();
    println!("dice: {:?}", rolls);
}
