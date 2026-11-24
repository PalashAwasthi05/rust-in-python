use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
enum Account<'a> { Username(&'a str), Email(&'a str) }

#[derive(Debug, Clone)]
struct AccountInfo<'a> { name: &'a str, logins: u32 }

fn call(number: &str) -> &str {
    match number {
        "798-1364" => "We're sorry, the call cannot be completed as dialed.",
        "645-7689" => "Hello, this is Mr. Awesome's Pizza. My name is Fred.",
        _ => "Hi! Who is this again?",
    }
}

fn try_logon<'a>(accounts: &mut HashMap<Account<'a>, AccountInfo<'a>>, key: Account<'a>) -> String {
    match accounts.get_mut(&key) {
        Some(info) => {
            info.logins += 1;
            format!("{:?} -> {} (login #{})", key, info.name, info.logins)
        }
        None => format!("{:?} -> login failed", key),
    }
}

fn sorted<T: Ord + Clone>(set: impl Iterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = set.collect();
    v.sort();
    v
}

fn main() {
    // HashMap
    let mut contacts = HashMap::new();
    contacts.insert("Daniel", "798-1364");
    contacts.insert("Ashley", "645-7689");
    contacts.insert("Katie", "435-8291");
    contacts.insert("Robert", "956-1745");
    match contacts.get(&"Daniel") {
        Some(&number) => println!("calling Daniel: {}", call(number)),
        _ => println!("don't have Daniel's number."),
    }
    contacts.insert("Daniel", "164-6743");
    match contacts.get(&"Ashley") {
        Some(&number) => println!("calling Ashley: {}", call(number)),
        _ => println!("don't have Ashley's number."),
    }
    contacts.remove(&"Ashley");
    let mut entries: Vec<_> = contacts.iter().collect();
    entries.sort();
    for (contact, &number) in entries {
        println!("calling {}: {}", contact, call(number));
    }

    // custom key types
    let mut accounts = HashMap::new();
    accounts.insert(Account::Username("j.everyman"), AccountInfo { name: "John Everyman", logins: 0 });
    accounts.insert(Account::Email("j@example.test"), AccountInfo { name: "John Everyman", logins: 3 });
    println!("{}", try_logon(&mut accounts, Account::Username("j.everyman")));
    println!("{}", try_logon(&mut accounts, Account::Email("j@example.test")));
    println!("{}", try_logon(&mut accounts, Account::Username("nobody")));
    let mut keys = sorted(accounts.keys().copied());
    keys.reverse();
    println!("keys desc: {:?}", keys);
    let mut word_lengths: HashMap<usize, Vec<&str>> = HashMap::new();
    for w in "a bb cc ddd e".split(' ') { word_lengths.entry(w.len()).or_default().push(w); }
    println!("by length: {:?}", sorted(word_lengths.into_iter()));

    // HashSet
    let mut a: HashSet<i32> = vec![1i32, 2, 3].into_iter().collect();
    let mut b: HashSet<i32> = vec![2i32, 3, 4].into_iter().collect();
    println!("insert 4 into a: {}, insert 4 again: {}", a.insert(4), a.insert(4));
    b.insert(5);
    println!("A: {:?}", sorted(a.iter()));
    println!("B: {:?}", sorted(b.iter()));
    println!("union: {:?}", sorted(a.union(&b)));
    println!("difference: {:?}", sorted(a.difference(&b)));
    println!("intersection: {:?}", sorted(a.intersection(&b)));
    println!("symmetric difference: {:?}", sorted(a.symmetric_difference(&b)));
    println!("subset {} disjoint {}", [2, 3].iter().copied().collect::<HashSet<i32>>().is_subset(&a), a.is_disjoint(&b));

    // Rc
    let rc_examples = "Rc examples".to_string();
    {
        println!("--- rc_a is created ---");
        let rc_a: Rc<String> = Rc::new(rc_examples);
        println!("reference count of rc_a: {}", Rc::strong_count(&rc_a));
        {
            let rc_b: Rc<String> = Rc::clone(&rc_a);
            println!("reference count of rc_b: {}", Rc::strong_count(&rc_b));
            println!("rc_a and rc_b are equal: {}", rc_a.eq(&rc_b));
            println!("length of the value inside rc_a: {}", rc_a.len());
            println!("value of rc_b: {}", rc_b);
        }
        println!("reference count of rc_a after rc_b dropped: {}", Rc::strong_count(&rc_a));
        let weak = Rc::downgrade(&rc_a);
        println!("weak upgrade while alive: {:?}", weak.upgrade().map(|s| s.len()));
        drop(rc_a);
        println!("weak upgrade after drop: {:?}", weak.upgrade());
    }

    // Arc without threads: shared ownership with atomic counts
    let apple = Arc::new("the same apple");
    let handles: Vec<Arc<&str>> = (0..3).map(|_| Arc::clone(&apple)).collect();
    println!("arc strong count: {}", Arc::strong_count(&apple));
    for (i, h) in handles.iter().enumerate() { println!("holder {} sees {:?}", i, h); }
    drop(handles);
    println!("after dropping holders: {}", Arc::strong_count(&apple));
    let mut unique = Arc::new(vec![1, 2]);
    Arc::make_mut(&mut unique).push(3);
    println!("make_mut on unique Arc: {:?}, ptr_eq self {}", unique, Arc::ptr_eq(&unique, &unique.clone()));
}
