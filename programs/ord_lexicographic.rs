use std::cmp::Ordering;

fn show(label: &str, o: Ordering) {
    println!("{:<28} {:?} lt={} le={} gt={} ge={} eq={} ne={}", label, o, o.is_lt(), o.is_le(), o.is_gt(), o.is_ge(), o.is_eq(), o.is_ne());
}

fn main() {
    // Option: None < Some(_)
    show("None vs Some(0)", None.cmp(&Some(0)));
    show("Some(3) vs Some(2)", Some(3).cmp(&Some(2)));
    println!("Some(1) < None: {}", Some(1) < None);
    let mut opts = vec![Some(3), None, Some(-1), None, Some(0)];
    opts.sort();
    println!("{:?}", opts);
    println!("max opt: {:?}", opts.iter().max());

    // Result: Ok < Err
    let r1: Result<i32, String> = Ok(100);
    let r2: Result<i32, String> = Err("a".to_string());
    show("Ok(100) vs Err(a)", r1.cmp(&r2));
    let mut rs: Vec<Result<u8, u8>> = vec![Err(1), Ok(5), Err(0), Ok(2)];
    rs.sort();
    println!("{:?}", rs);

    // tuples
    show("(1,2,3) vs (1,2,4)", (1, 2, 3).cmp(&(1, 2, 4)));
    show("(2,'a') vs (1,'z')", (2, 'a').cmp(&(1, 'z')));
    println!("(1, \"b\") < (1, \"ab\"): {}", (1, "b") < (1, "ab"));
    let mut ts = vec![(2, "b", 1.5), (1, "z", 0.0), (2, "a", 9.0), (2, "b", -1.0)];
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:?}", ts);
    println!("tuple eq: {}", (1, (2, 3)) == (1, (2, 3)));
    println!("unit cmp: {:?}", ().cmp(&()));

    // Vec and slices
    show("[1,2,3] vs [1,2]", vec![1, 2, 3].cmp(&vec![1, 2]));
    show("[1,2] vs [1,3]", [1, 2].cmp(&[1, 3]));
    show("[] vs [0]", Vec::<i32>::new().cmp(&vec![0]));
    let mut vs = vec![vec![2], vec![1, 9, 9], vec![1, 9], vec![], vec![1, 10]];
    vs.sort();
    println!("{:?}", vs);
    println!("slice lt: {}", &[1, 2, 3][..] < &[1, 2, 4][..]);
    println!("array eq: {}", [1, 2, 3] == [1, 2, 3]);
    println!("vec == array: {}", vec![1, 2] == [1, 2]);
    println!("vec == slice: {}", vec![1, 2] == &[1, 2][..]);

    // strings
    show("abc vs abd", "abc".cmp("abd"));
    show("Zebra vs apple", "Zebra".cmp("apple"));
    show("ab vs abc", "ab".cmp("abc"));
    let s1 = String::from("hello");
    let s2 = String::from("help");
    println!("String < String: {}", s1 < s2);
    println!("String == &str: {}", s1 == "hello");
    println!("&str == String: {}", "hello" == s1);
    println!("String.as_str() cmp: {:?}", s1.as_str().cmp("hello"));
    let mut names = vec!["émile", "Bob", "alice", "bob", "Alice", "zed", "ábc"];
    names.sort();
    println!("{:?}", names);
    names.sort_by_key(|s| s.to_lowercase());
    println!("{:?}", names);
    println!("char cmp: {:?} {}", 'a'.cmp(&'B'), 'a' > 'Z');
    println!("bool cmp: {:?} {}", false.cmp(&true), true > false);

    // Ordering helpers
    let o = Ordering::Less;
    show("reverse(Less)", o.reverse());
    show("Equal.then(Greater)", Ordering::Equal.then(Ordering::Greater));
    show("Less.then(Greater)", Ordering::Less.then(Ordering::Greater));
    println!("Ordering as i32: {} {} {}", Ordering::Less as i32, Ordering::Equal as i32, Ordering::Greater as i32);
    println!("Ordering sort: {:?}", {
        let mut os = vec![Ordering::Greater, Ordering::Less, Ordering::Equal];
        os.sort();
        os
    });
    match 5.cmp(&3) {
        Ordering::Less => println!("less"),
        Ordering::Equal => println!("equal"),
        Ordering::Greater => println!("greater"),
    }
    let (x, y) = (&5, &3);
    println!("cmp on refs: {:?} {:?}", x.cmp(y), Some(&x).cmp(&Some(&y)));
    println!("is_sorted-ish: {}", [1, 2, 2, 5].windows(2).all(|w| w[0] <= w[1]));
}
