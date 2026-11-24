#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Selecting { credit: u32 },
    Dispensing { item: char, change: u32 },
    OutOfOrder,
}

#[derive(Debug)]
enum Event {
    Coin(u32),
    Choose(char),
    Done,
    Cancel,
    Kick,
}

fn price(item: char) -> Option<u32> {
    match item {
        'A' => Some(75),
        'B' => Some(100),
        'C' => Some(150),
        _ => None,
    }
}

fn step(state: State, event: &Event) -> State {
    use State::*;
    match (state, event) {
        (OutOfOrder, _) => OutOfOrder,
        (_, Event::Kick) => OutOfOrder,
        (Idle, Event::Coin(c)) => Selecting { credit: *c },
        (Selecting { credit }, Event::Coin(c)) => Selecting { credit: credit + c },
        (Selecting { credit }, Event::Choose(item)) => match price(*item) {
            Some(p) if credit >= p => Dispensing { item: *item, change: credit - p },
            Some(p) => {
                println!("  need {} more for {}", p - credit, item);
                state
            }
            None => {
                println!("  unknown item {}", item);
                state
            }
        },
        (Selecting { credit }, Event::Cancel) => {
            println!("  refunding {}", credit);
            Idle
        }
        (Dispensing { item, change }, Event::Done) => {
            println!("  vended {} with change {}", item, change);
            Idle
        }
        (s, e) => {
            println!("  ignored {:?} in {:?}", e, s);
            s
        }
    }
}

fn main() {
    let events = vec![
        Event::Choose('A'),
        Event::Coin(50),
        Event::Choose('A'),
        Event::Coin(50),
        Event::Choose('Z'),
        Event::Choose('A'),
        Event::Done,
        Event::Coin(25),
        Event::Cancel,
        Event::Coin(200),
        Event::Choose('C'),
        Event::Done,
        Event::Kick,
        Event::Coin(100),
    ];
    let mut state = State::Idle;
    for ev in &events {
        let next = step(state, ev);
        println!("{:?} --{:?}--> {:?}", state, ev, next);
        state = next;
    }
    println!("final: {:?}, broken = {}", state, state == State::OutOfOrder);
}
