use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Dir {
    fn opposite(self) -> Dir {
        use Dir::*;
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
            Up => Down,
            Down => Up,
        }
    }
}

impl FromStr for Dir {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "n" | "north" => Dir::North,
            "s" | "south" => Dir::South,
            "e" | "east" => Dir::East,
            "w" | "west" => Dir::West,
            "u" | "up" => Dir::Up,
            "d" | "down" => Dir::Down,
            _ => return Err(format!("'{}' is not a direction", s)),
        })
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self).to_lowercase();
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum RoomId {
    Cellar,
    Kitchen,
    Hall,
    Library,
    Tower,
    Garden,
}

struct Room {
    name: &'static str,
    description: &'static str,
    exits: HashMap<Dir, RoomId>,
    items: Vec<String>,
    dark: bool,
}

#[derive(Debug, PartialEq)]
enum Command {
    Go(Dir),
    Take(String),
    Drop(String),
    Use(String),
    Look,
    Inventory,
    Quit,
}

fn parse(line: &str) -> Result<Command, String> {
    let words: Vec<String> = line
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| !matches!(w.as_str(), "the" | "a" | "an"))
        .collect();
    let rest = || words[1..].join(" ");
    match words.first().map(String::as_str) {
        None => Err("say something".to_string()),
        Some("go") | Some("walk") if words.len() == 2 => words[1].parse().map(Command::Go),
        Some("go") => Err("go where?".into()),
        Some(w) if w.parse::<Dir>().is_ok() && words.len() == 1 => Ok(Command::Go(w.parse()?)),
        Some("take") | Some("get") if words.len() > 1 => Ok(Command::Take(rest())),
        Some("drop") if words.len() > 1 => Ok(Command::Drop(rest())),
        Some("use") if words.len() > 1 => Ok(Command::Use(rest())),
        Some("look") | Some("l") => Ok(Command::Look),
        Some("inventory") | Some("i") => Ok(Command::Inventory),
        Some("quit") => Ok(Command::Quit),
        Some(other) => Err(format!("I don't know how to '{}'", other)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Playing,
    Won,
    Quit,
}

struct Game {
    rooms: HashMap<RoomId, Room>,
    here: RoomId,
    inventory: Vec<String>,
    visited: HashSet<RoomId>,
    moves: u32,
    score: i32,
    lamp_lit: bool,
    state: State,
}

impl Game {
    fn new() -> Game {
        let mut rooms = HashMap::new();
        let mut add = |id, name, description, items: &[&str], dark| {
            rooms.insert(id, Room { name, description, exits: HashMap::new(), items: items.iter().map(|s| s.to_string()).collect(), dark });
        };
        add(RoomId::Cellar, "Cellar", "Damp stone walls. Stairs lead up.", &["lamp"], false);
        add(RoomId::Kitchen, "Kitchen", "Pots hang from hooks.", &["bread", "knife"], false);
        add(RoomId::Hall, "Great Hall", "A long table runs the length of the hall.", &[], false);
        add(RoomId::Library, "Library", "Shelves of crumbling books.", &["old key", "book"], true);
        add(RoomId::Tower, "Tower Top", "Wind howls. A locked chest sits here.", &["chest"], false);
        add(RoomId::Garden, "Garden", "Overgrown roses.", &["rose"], false);
        let mut g = Game { rooms, here: RoomId::Cellar, inventory: vec![], visited: HashSet::new(), moves: 0, score: 0, lamp_lit: false, state: State::Playing };
        g.link(RoomId::Cellar, Dir::Up, RoomId::Kitchen);
        g.link(RoomId::Kitchen, Dir::East, RoomId::Hall);
        g.link(RoomId::Hall, Dir::North, RoomId::Library);
        g.link(RoomId::Hall, Dir::Up, RoomId::Tower);
        g.link(RoomId::Kitchen, Dir::South, RoomId::Garden);
        g.visited.insert(g.here);
        g
    }

    fn link(&mut self, a: RoomId, d: Dir, b: RoomId) {
        self.rooms.get_mut(&a).unwrap().exits.insert(d, b);
        self.rooms.get_mut(&b).unwrap().exits.insert(d.opposite(), a);
    }

    fn room(&self) -> &Room {
        &self.rooms[&self.here]
    }

    fn can_see(&self) -> bool {
        !self.room().dark || self.lamp_lit
    }

    fn describe(&self) -> String {
        if !self.can_see() {
            return "It is pitch dark. You might be eaten by a grue.".to_string();
        }
        let room = self.room();
        let mut exits: Vec<String> = room.exits.keys().map(|d| d.to_string()).collect();
        exits.sort();
        let mut s = format!("== {} ==\n{}\nExits: {}", room.name, room.description, exits.join(", "));
        if !room.items.is_empty() {
            s += &format!("\nYou see: {}", room.items.join(", "));
        }
        s
    }

    fn step(&mut self, cmd: Command) -> String {
        self.moves += 1;
        match cmd {
            Command::Look => self.describe(),
            Command::Inventory if self.inventory.is_empty() => "You are empty-handed.".into(),
            Command::Inventory => format!("You carry: {}", self.inventory.join(", ")),
            Command::Quit => {
                self.state = State::Quit;
                "Bye.".to_string()
            }
            Command::Go(d) => match self.room().exits.get(&d).copied() {
                None => format!("You can't go {} from here.", d),
                Some(next) => {
                    self.here = next;
                    if self.visited.insert(next) {
                        self.score += 5;
                    }
                    self.describe()
                }
            },
            Command::Take(item) => {
                if !self.can_see() {
                    return "You fumble around in the dark.".into();
                }
                if item == "chest" {
                    return "The chest is far too heavy.".into();
                }
                let here = self.here;
                let room = self.rooms.get_mut(&here).unwrap();
                if let Some(pos) = room.items.iter().position(|i| *i == item) {
                    let taken = room.items.remove(pos);
                    self.score += if taken == "old key" { 10 } else { 1 };
                    self.inventory.push(taken);
                    format!("Taken: {}.", item)
                } else {
                    format!("There is no {} here.", item)
                }
            }
            Command::Drop(item) => match self.inventory.iter().position(|i| i == &item) {
                Some(pos) => {
                    let it = self.inventory.swap_remove(pos);
                    self.rooms.get_mut(&self.here).unwrap().items.push(it);
                    "Dropped.".to_string()
                }
                None => format!("You don't have a {}.", item),
            },
            Command::Use(item) => {
                if !self.inventory.contains(&item) {
                    return format!("You don't have a {}.", item);
                }
                match (item.as_str(), self.here) {
                    ("lamp", _) => {
                        self.lamp_lit = !self.lamp_lit;
                        format!("The lamp is now {}.", if self.lamp_lit { "on" } else { "off" })
                    }
                    ("old key", RoomId::Tower) => {
                        self.score += 50;
                        self.state = State::Won;
                        "The chest creaks open, revealing treasure!".into()
                    }
                    ("bread", _) => {
                        self.inventory.retain(|i| i != "bread");
                        "Delicious.".into()
                    }
                    (other, room) => format!("Using the {} in the {:?} does nothing.", other, room),
                }
            }
        }
    }
}

fn main() {
    let script = [
        "look", "take lamp", "n", "go up", "take the bread", "take knife", "s", "take rose",
        "dance", "go", "north", "e", "n", "take old key", "use lamp", "look", "take the old key",
        "take book", "drop knife", "look", "i", "use bread", "s", "u", "take chest", "use rose",
        "use old key", "look",
    ];
    let mut game = Game::new();
    for line in script.iter() {
        if game.state != State::Playing {
            println!("(ignoring '{}': game over)", line);
            continue;
        }
        println!("> {}", line);
        match parse(line) {
            Ok(cmd) => println!("{}", game.step(cmd)),
            Err(e) => println!("?? {}", e),
        }
    }
    let mut seen: Vec<_> = game.visited.iter().collect();
    seen.sort();
    println!("state={:?} moves={} score={} visited={:?}", game.state, game.moves, game.score, seen);
    assert_eq!(parse("WALK North"), Ok(Command::Go(Dir::North)));
    assert!(parse("go sideways").is_err());
}
