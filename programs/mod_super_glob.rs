// use super::* in child modules, nested function items, and use inside fn bodies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Suit {
    Hearts,
    Spades,
}

pub const DECK_SIZE: usize = 52;

pub fn deal(n: usize) -> usize {
    n.min(DECK_SIZE)
}

mod rules {
    use super::*;

    pub fn is_red(s: Suit) -> bool {
        matches!(s, Suit::Hearts)
    }

    pub fn max_hand() -> usize {
        deal(DECK_SIZE / 4)
    }

    pub mod scoring {
        use super::*;
        use super::super::Suit::*;

        pub fn score(s: Suit) -> u32 {
            match s {
                Hearts => if is_red(s) { 10 } else { 0 },
                Spades => 5,
            }
        }
    }
}

fn main() {
    use rules::scoring::score;
    println!("{} {}", rules::is_red(Suit::Hearts), rules::is_red(Suit::Spades));
    println!("{}", rules::max_hand());
    println!("{} {}", score(Suit::Hearts), score(Suit::Spades));

    fn helper(x: usize) -> usize {
        use inner::twice;
        mod inner {
            pub fn twice(x: usize) -> usize {
                x * 2
            }
        }
        twice(x) + super_deal()
    }

    fn super_deal() -> usize {
        deal(100)
    }

    println!("{}", helper(4));
}
