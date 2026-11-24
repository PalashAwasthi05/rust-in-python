#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Student {
    grade: u8,
    name: String,
    age: u8,
}

fn st(grade: u8, name: &str, age: u8) -> Student {
    Student { grade, name: name.to_string(), age }
}

fn main() {
    let roster = vec![
        st(3, "mia", 9),
        st(1, "leo", 7),
        st(3, "ava", 8),
        st(2, "noa", 8),
        st(1, "eli", 6),
        st(2, "kai", 7),
        st(3, "zoe", 9),
    ];

    // stable: within a grade, original order is kept
    let mut by_grade = roster.clone();
    by_grade.sort_by_key(|s| s.grade);
    for s in &by_grade {
        println!("{} {} {}", s.grade, s.name, s.age);
    }

    // multi-pass stable sort: sort by secondary key first, then primary
    let mut two_pass = roster.clone();
    two_pass.sort_by(|a, b| a.name.cmp(&b.name));
    two_pass.sort_by_key(|s| s.age);
    println!("{:?}", two_pass.iter().map(|s| format!("{}:{}", s.age, s.name)).collect::<Vec<_>>());

    // descending by key while staying stable
    let mut desc = roster.clone();
    desc.sort_by_key(|s| std::cmp::Reverse(s.age));
    println!("{:?}", desc.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());
    let mut desc2 = roster.clone();
    desc2.sort_by(|a, b| b.grade.cmp(&a.grade));
    println!("{:?}", desc2.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());

    // cached key and tuple keys
    let mut cached = roster.clone();
    cached.sort_by_cached_key(|s| s.name.len() as i32 * -1 + s.age as i32);
    println!("{:?}", cached.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());
    let mut tup = roster.clone();
    tup.sort_by_key(|s| (s.age, std::cmp::Reverse(s.grade)));
    println!("{:?}", tup.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());

    // whole-struct derived order
    let mut full = roster.clone();
    full.sort();
    println!("{:?}", full.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());
    full.reverse();
    println!("{:?}", full.iter().map(|s| s.name.as_str()).collect::<Vec<_>>());

    // binary search on sorted structs
    let mut sorted = roster.clone();
    sorted.sort();
    let probe = st(2, "noa", 8);
    println!("search exact: {:?}", sorted.binary_search(&probe));
    println!("search missing: {:?}", sorted.binary_search(&st(2, "mmm", 1)));
    println!("search before all: {:?}", sorted.binary_search(&st(0, "", 0)));
    println!("search after all: {:?}", sorted.binary_search(&st(9, "", 0)));
    let by_name = {
        let mut v = roster.clone();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    };
    println!("by name: {:?}", by_name.binary_search_by(|s| s.name.as_str().cmp("leo")));
    println!("by name missing: {:?}", by_name.binary_search_by(|s| s.name.as_str().cmp("bob")));
    println!("by key: {:?}", by_name.binary_search_by_key(&"zoe", |s| s.name.as_str()));
    let ages: Vec<u8> = {
        let mut a: Vec<u8> = roster.iter().map(|s| s.age).collect();
        a.sort();
        a
    };
    println!("ages {:?}", ages);
    let pp = ages.partition_point(|&a| a < 8);
    println!("partition_point(<8) = {}", pp);
    match ages.binary_search(&5) {
        Ok(i) => println!("found at {}", i),
        Err(i) => println!("insert at {}", i),
    }
    let mut insert_into = ages.clone();
    let pos = insert_into.binary_search(&7).unwrap_or_else(|e| e);
    insert_into.insert(pos, 7);
    println!("{:?}", insert_into);

    // unstable sort on unique keys is deterministic
    let mut uniq = vec![(5, 'e'), (1, 'a'), (4, 'd'), (2, 'b'), (3, 'c')];
    uniq.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    println!("{:?}", uniq);
    uniq.sort_unstable_by_key(|p| p.1);
    println!("{:?}", uniq);
    println!("is_sorted: {}", uniq.windows(2).all(|w| w[0] <= w[1]));
}
