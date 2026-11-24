// A lending library: members and books shared through Rc<RefCell<..>>, a reservation
// queue per book, an event queue ordered by day (BinaryHeap with a custom Ord),
// pluggable fee policies as boxed closures, and reports built from iterator chains.
use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, VecDeque};
use std::fmt;
use std::rc::{Rc, Weak};

type Day = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Genre {
    Fiction,
    Science,
    History,
    Kids,
}

struct Book {
    isbn: u32,
    title: String,
    genre: Genre,
    holder: Option<Weak<RefCell<Member>>>,
    due: Option<Day>,
    waitlist: VecDeque<String>,
    times_lent: u32,
}

struct Member {
    name: String,
    loans: Vec<Rc<RefCell<Book>>>,
    fees: u32,
    history: Vec<u32>,
}

impl fmt::Debug for Member {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Member")
            .field("name", &self.name)
            .field("loans", &self.loans.iter().map(|b| b.borrow().isbn).collect::<Vec<_>>())
            .field("fees", &self.fees)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Borrow { who: String, isbn: u32 },
    Return { who: String, isbn: u32 },
    Reserve { who: String, isbn: u32 },
    Audit,
}

#[derive(Debug, PartialEq, Eq)]
struct Scheduled {
    day: Day,
    seq: u32,
    event: Event,
}

impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        // Earliest day first, then insertion order: BinaryHeap is a max-heap, so reverse.
        other.day.cmp(&self.day).then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

type FeePolicy = Box<dyn Fn(Genre, u32) -> u32>;

struct Library {
    books: BTreeMap<u32, Rc<RefCell<Book>>>,
    members: HashMap<String, Rc<RefCell<Member>>>,
    loan_days: HashMap<Genre, Day>,
    fee_policy: FeePolicy,
    log: Vec<String>,
    overdue_days: BTreeSet<Day>,
}

impl Library {
    fn new(fee_policy: FeePolicy) -> Self {
        let loan_days = [(Genre::Fiction, 14), (Genre::Science, 7), (Genre::History, 21), (Genre::Kids, 10)]
            .into_iter()
            .collect();
        Library { books: BTreeMap::new(), members: HashMap::new(), loan_days, fee_policy, log: Vec::new(), overdue_days: BTreeSet::new() }
    }

    fn add_book(&mut self, isbn: u32, title: &str, genre: Genre) {
        let book = Book { isbn, title: title.into(), genre, holder: None, due: None, waitlist: VecDeque::new(), times_lent: 0 };
        self.books.insert(isbn, Rc::new(RefCell::new(book)));
    }

    fn join(&mut self, name: &str) {
        self.members.entry(name.to_string()).or_insert_with(|| {
            Rc::new(RefCell::new(Member { name: name.to_string(), loans: vec![], fees: 0, history: vec![] }))
        });
    }

    fn lookup(&self, who: &str, isbn: u32) -> Result<(Rc<RefCell<Member>>, Rc<RefCell<Book>>), String> {
        let member = self.members.get(who).cloned().ok_or_else(|| format!("no member {}", who))?;
        let book = self.books.get(&isbn).map(Rc::clone).ok_or(format!("no book {}", isbn))?;
        Ok((member, book))
    }

    fn handle(&mut self, day: Day, event: &Event) -> Result<String, String> {
        match event {
            Event::Borrow { who, isbn } => {
                let (member, book) = self.lookup(who, *isbn)?;
                if member.borrow().fees > 300 {
                    return Err(format!("{} owes {} and cannot borrow", who, member.borrow().fees));
                }
                let mut b = book.borrow_mut();
                if let Some(holder) = b.holder.as_ref().and_then(|w| w.upgrade()) {
                    return Err(format!("'{}' is out with {}", b.title, holder.borrow().name));
                }
                if let Some(front) = b.waitlist.front() {
                    if front != who {
                        return Err(format!("'{}' is reserved for {}", b.title, front));
                    }
                    b.waitlist.pop_front();
                }
                let due = day + self.loan_days[&b.genre];
                b.holder = Some(Rc::downgrade(&member));
                b.due = Some(due);
                b.times_lent += 1;
                drop(b);
                let mut m = member.borrow_mut();
                m.loans.push(Rc::clone(&book));
                m.history.push(*isbn);
                Ok(format!("{} borrows '{}' until day {}", who, book.borrow().title, due))
            }
            Event::Return { who, isbn } => {
                let (member, book) = self.lookup(who, *isbn)?;
                let pos = member
                    .borrow()
                    .loans
                    .iter()
                    .position(|b| Rc::ptr_eq(b, &book))
                    .ok_or_else(|| format!("{} does not hold {}", who, isbn))?;
                member.borrow_mut().loans.remove(pos);
                let mut b = book.borrow_mut();
                let late = b.due.map_or(0, |due| day.saturating_sub(due));
                b.holder = None;
                b.due = None;
                let fee = (self.fee_policy)(b.genre, late);
                member.borrow_mut().fees += fee;
                let next = b.waitlist.front().cloned();
                Ok(match (late, next) {
                    (0, None) => format!("{} returns '{}' on time", who, b.title),
                    (0, Some(n)) => format!("{} returns '{}'; {} is next", who, b.title, n),
                    (d, n) => format!("{} returns '{}' {} day{} late, fee {}{}", who, b.title, d,
                                      if d == 1 { "" } else { "s" }, fee,
                                      n.map(|n| format!("; {} is next", n)).unwrap_or_default()),
                })
            }
            Event::Reserve { who, isbn } => {
                let (_, book) = self.lookup(who, *isbn)?;
                let mut b = book.borrow_mut();
                if b.waitlist.iter().any(|w| w == who) {
                    return Err(format!("{} already waiting for '{}'", who, b.title));
                }
                b.waitlist.push_back(who.clone());
                Ok(format!("{} reserves '{}' (position {})", who, b.title, b.waitlist.len()))
            }
            Event::Audit => {
                let overdue: Vec<String> = self
                    .books
                    .values()
                    .filter_map(|b| {
                        let b = b.borrow();
                        let due = b.due?;
                        let holder = b.holder.as_ref()?.upgrade()?;
                        let name = holder.borrow().name.clone();
                        (due < day).then(|| format!("{}@{}", b.isbn, name))
                    })
                    .collect();
                if !overdue.is_empty() {
                    self.overdue_days.insert(day);
                }
                Ok(format!("audit: {} out, overdue {:?}", self.books.values().filter(|b| b.borrow().due.is_some()).count(), overdue))
            }
        }
    }

    fn top_titles(&self, n: usize) -> impl Iterator<Item = (String, u32)> + '_ {
        let mut ranked: Vec<(String, u32)> = self.books.values().map(|b| {
            let b = b.borrow();
            (b.title.clone(), b.times_lent)
        }).collect();
        ranked.sort_by_key(|(title, count)| (Reverse(*count), title.clone()));
        ranked.into_iter().take(n)
    }
}

fn main() {
    let flat_rate: FeePolicy = Box::new(|genre, late| {
        let per_day = match genre {
            Genre::Kids => 5,
            Genre::Science | Genre::History => 25,
            _ => 15,
        };
        (late * per_day).min(500)
    });
    let mut lib = Library::new(flat_rate);
    let catalogue = [
        (101, "Dune", Genre::Fiction),
        (102, "Cosmos", Genre::Science),
        (103, "SPQR", Genre::History),
        (104, "Matilda", Genre::Kids),
        (105, "Foundation", Genre::Fiction),
    ];
    for &(isbn, title, genre) in &catalogue {
        lib.add_book(isbn, title, genre);
    }
    for name in "ana ben cho dev".split_whitespace() {
        lib.join(name);
    }

    let script: Vec<(Day, Event)> = vec![
        (1, Event::Borrow { who: "ana".into(), isbn: 101 }),
        (1, Event::Borrow { who: "ben".into(), isbn: 101 }),
        (2, Event::Reserve { who: "ben".into(), isbn: 101 }),
        (2, Event::Reserve { who: "cho".into(), isbn: 101 }),
        (2, Event::Reserve { who: "ben".into(), isbn: 101 }),
        (3, Event::Borrow { who: "cho".into(), isbn: 102 }),
        (3, Event::Borrow { who: "dev".into(), isbn: 104 }),
        (4, Event::Borrow { who: "zed".into(), isbn: 104 }),
        (5, Event::Borrow { who: "ana".into(), isbn: 999 }),
        (12, Event::Audit),
        (20, Event::Return { who: "ana".into(), isbn: 101 }),
        (20, Event::Borrow { who: "cho".into(), isbn: 101 }),
        (21, Event::Borrow { who: "ben".into(), isbn: 101 }),
        (22, Event::Return { who: "cho".into(), isbn: 102 }),
        (22, Event::Audit),
        (25, Event::Return { who: "dev".into(), isbn: 104 }),
        (26, Event::Return { who: "dev".into(), isbn: 104 }),
        (30, Event::Borrow { who: "ana".into(), isbn: 105 }),
        (30, Event::Borrow { who: "dev".into(), isbn: 103 }),
        (60, Event::Audit),
        (61, Event::Return { who: "ben".into(), isbn: 101 }),
        (70, Event::Return { who: "dev".into(), isbn: 103 }),
        (71, Event::Borrow { who: "cho".into(), isbn: 103 }),
    ];
    // Feed the script through the heap in reverse so ordering comes from (day, seq), not input order.
    let mut queue: BinaryHeap<Scheduled> = script
        .into_iter()
        .enumerate()
        .rev()
        .map(|(seq, (day, event))| Scheduled { day, seq: seq as u32, event })
        .collect();
    while let Some(Scheduled { day, event, .. }) = queue.pop() {
        let line = match lib.handle(day, &event) {
            Ok(msg) => format!("day {:>2}: {}", day, msg),
            Err(err) => format!("day {:>2}: ERROR {}", day, err),
        };
        lib.log.push(line);
    }
    lib.log.iter().for_each(|l| println!("{}", l));

    println!("-- members --");
    let mut names: Vec<&String> = lib.members.keys().collect();
    names.sort_unstable();
    for name in names {
        let m = lib.members[name].borrow();
        println!("{:?} history={:?}", m, m.history);
    }
    println!("-- popular --");
    for (rank, (title, count)) in lib.top_titles(3).enumerate() {
        println!("{}. {:<12} x{}", rank + 1, title, count);
    }
    let by_genre = lib.books.values().fold(BTreeMap::new(), |mut acc: BTreeMap<Genre, Vec<u32>>, b| {
        let b = b.borrow();
        acc.entry(b.genre).or_default().push(b.isbn);
        acc
    });
    println!("by genre: {:?}", by_genre);
    println!("overdue audit days: {:?}, after day 20: {:?}", lib.overdue_days, lib.overdue_days.range(20..).collect::<Vec<_>>());
    let total_fees: u32 = lib.members.values().map(|m| m.borrow().fees).sum();
    let rc_counts: Vec<usize> = lib.books.values().map(Rc::strong_count).collect();
    println!("total fees {}, book strong counts {:?}", total_fees, rc_counts);
    let lent_pairs: Vec<String> = lib
        .books
        .values()
        .map(|b| b.borrow().times_lent)
        .collect::<Vec<_>>()
        .windows(2)
        .map(|w| format!("{}{}", w[0], if w[0] >= w[1] { ">=" } else { "<" }))
        .collect();
    println!("{}", lent_pairs.concat());
}
