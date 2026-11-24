// Day 10 2022: a tiny CPU drawing on a CRT, with FromStr for instructions.
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Instr {
    Noop,
    Addx(i32),
}

#[derive(Debug)]
struct ParseErr(String);

impl FromStr for Instr {
    type Err = ParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        match (words.next(), words.next()) {
            (Some("noop"), None) => Ok(Instr::Noop),
            (Some("addx"), Some(n)) => n.parse().map(Instr::Addx).map_err(|e| ParseErr(format!("{n}: {e}"))),
            _ => Err(ParseErr(s.to_string())),
        }
    }
}

const INPUT: &str = r"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";

fn main() {
    let program: Vec<Instr> = INPUT.lines().map(|l| l.parse().unwrap()).collect();
    let xs: Vec<i32> = program
        .iter()
        .scan(1, |x, ins| {
            let before = *x;
            Some(match *ins {
                Instr::Noop => vec![before],
                Instr::Addx(n) => {
                    *x += n;
                    vec![before, before]
                }
            })
        })
        .flatten()
        .collect();
    let strength: i32 = (20..=220).step_by(40).map(|cycle| cycle as i32 * xs[cycle - 1]).sum();
    println!("part 1: {strength}");
    let mut screen = String::new();
    for (i, &x) in xs.iter().enumerate().take(240) {
        let col = (i % 40) as i32;
        screen.push(if (col - x).abs() <= 1 { '#' } else { '.' });
        if col == 39 {
            screen.push('\n');
        }
    }
    print!("{screen}");
    match "jump 3".parse::<Instr>() {
        Ok(i) => println!("parsed {:?}", i),
        Err(e) => println!("error: {:?}", e),
    }
    if let Err(ParseErr(msg)) = "addx x".parse::<Instr>() {
        println!("error: {msg}");
    }
}
