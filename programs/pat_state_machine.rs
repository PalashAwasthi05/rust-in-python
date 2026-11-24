// A door lock state machine using matches on (state, event) tuples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Locked { attempts: u8 },
    Closed,
    Open,
    Alarm,
}

#[derive(Debug, Clone, Copy)]
enum Event<'a> {
    Code(&'a str),
    Push,
    Pull,
    Reset,
}

const SECRET: &str = "1234";
const MAX_ATTEMPTS: u8 = 3;

fn step(state: State, event: Event) -> State {
    match (state, event) {
        (State::Locked { .. }, Event::Code(c)) if c == SECRET => State::Closed,
        (State::Locked { attempts }, Event::Code(_)) if attempts + 1 >= MAX_ATTEMPTS => State::Alarm,
        (State::Locked { attempts }, Event::Code(_)) => State::Locked { attempts: attempts + 1 },
        (State::Closed, Event::Push) => State::Open,
        (State::Open, Event::Pull) => State::Closed,
        (State::Closed, Event::Code(_)) => State::Locked { attempts: 0 },
        (State::Alarm, Event::Reset) => State::Locked { attempts: 0 },
        (s @ (State::Alarm | State::Open | State::Closed | State::Locked { .. }), _) => s,
    }
}

fn label(s: &State) -> String {
    match *s {
        State::Locked { attempts: 0 } => "locked".to_string(),
        State::Locked { attempts: n } => format!("locked ({} bad)", n),
        State::Closed => "closed".into(),
        State::Open => "OPEN".into(),
        State::Alarm => "ALARM!".into(),
    }
}

fn main() {
    let script = [
        Event::Push,
        Event::Code("0000"),
        Event::Code("1234"),
        Event::Push,
        Event::Push,
        Event::Pull,
        Event::Code("lock"),
        Event::Code("1"),
        Event::Code("2"),
        Event::Code("3"),
        Event::Code("1234"),
        Event::Reset,
        Event::Code("1234"),
    ];
    let mut state = State::Locked { attempts: 0 };
    let mut history = vec![state];
    for (i, ev) in script.iter().enumerate() {
        let next = step(state, *ev);
        let changed = if next != state { "*" } else { " " };
        println!("{:2} {} {:<14} -> {}", i, changed, format!("{:?}", ev), label(&next));
        state = next;
        history.push(state);
    }
    let alarms = history.iter().filter(|s| matches!(s, State::Alarm)).count();
    let max_bad = history
        .iter()
        .filter_map(|s| match s { State::Locked { attempts } => Some(*attempts), _ => None })
        .max()
        .unwrap_or(0);
    println!("alarms={} max_bad={} final={:?}", alarms, max_bad, state);
}
